//! The audit application shared by the `auditmysite` and `auditmysite-dev`
//! binaries: logging, signal handling, exit codes and the audit run itself.

use std::future::Future;
use std::io::{self, IsTerminal};

use colored::Colorize;
use dialoguer::Input;
use tracing::error;
use tracing_subscriber::EnvFilter;

use auditmysite::cli::Args;
use auditmysite::error::{AuditError, Result};

use crate::commands::handle_command;
use crate::plan::print_banner;
use crate::runners::{run_batch_mode, run_display_all, run_single_mode};

/// Runs `run` until it ends or the process is interrupted, then exits with the
/// code of its result.
pub(crate) async fn run_and_exit(
    run: impl Future<Output = Result<auditmysite::Verdict>>,
    report_mode: bool,
) -> ! {
    let run_result = tokio::select! {
        result = run => result,
        signal = shutdown_signal() => Err(AuditError::Interrupted { signal }),
    };
    let exit_code = match run_result {
        // --report-mode: report generation succeeded, so exit 0 regardless of
        // the accessibility/verdict policy result (#503). Technical failures
        // below still exit non-zero.
        Ok(_) if report_mode => 0,
        Ok(verdict) => verdict.exit_code(),
        Err(e) => {
            error!("{}", e);
            eprintln!("{} {}", "Error:".red().bold(), e);
            3
        }
    };

    std::process::exit(exit_code);
}

async fn shutdown_signal() -> String {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => "Ctrl-C".to_string(),
            _ = terminate.recv() => "SIGTERM".to_string(),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        "Ctrl-C".to_string()
    }
}

/// Setup tracing/logging based on CLI flags
pub(crate) fn setup_logging(args: &Args) {
    let filter = if args.quiet {
        "error,chromiumoxide=off,tungstenite=off".to_string()
    } else if args.verbose {
        "debug,chromiumoxide=error,tungstenite=error,auditmysite=debug".to_string()
    } else {
        "warn,chromiumoxide=off,tungstenite=off,auditmysite=warn".to_string()
    };

    let ansi = match args.color {
        auditmysite::cli::ColorPolicy::Always => true,
        auditmysite::cli::ColorPolicy::Never => false,
        auditmysite::cli::ColorPolicy::Auto => io::stderr().is_terminal(),
    };

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        // `tracing_subscriber::fmt()` defaults to stdout; a warn/error-level
        // log line (e.g. a batch page timeout) would otherwise interleave
        // with JSON/AI/SARIF/table payloads on stdout (#529/#530's
        // stdout-must-stay-machine-readable mandate). ANSI follows the same
        // `--color` policy as the table/progress renderers instead of
        // tracing-subscriber's own default-on behavior.
        .with_writer(std::io::stderr)
        .with_ansi(ansi)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

/// Main application logic
pub(crate) async fn run(
    mut args: Args,
    config: &Option<auditmysite::cli::Config>,
) -> Result<auditmysite::Verdict> {
    // Handle subcommands first
    if let Some(ref command) = args.command {
        return handle_command(command, &args)
            .await
            .map(|_| auditmysite::Verdict::Pass);
    }

    // If no input source specified interactively ask for a domain (terminal only)
    let no_input =
        args.url.is_none() && args.sitemap.is_none() && args.url_file.is_none() && !args.crawl;

    if no_input && !args.quiet && io::stdin().is_terminal() && io::stdout().is_terminal() {
        print_banner();
        let input: String = Input::new()
            .with_prompt("  Domain or URL (e.g., example.com)")
            .validate_with(|s: &String| {
                if s.trim().is_empty() {
                    Err("Please enter a URL.")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|e| AuditError::ConfigError(e.to_string()))?;
        let url = if input.contains("://") {
            input
        } else {
            format!("https://{}", input)
        };
        args.url = Some(url);
    }

    if let Err(e) = args.validate() {
        return Err(AuditError::ConfigError(e));
    }

    if !args.quiet {
        // Only print banner if we haven't already (interactive prompt already printed it)
        if !no_input {
            print_banner();
        }
    }

    let is_batch = args.sitemap.is_some() || args.url_file.is_some() || args.crawl;

    let result = dispatch_audit(&args, config, is_batch).await;
    match result {
        // An interactive terminal on a supported platform may consent to the
        // one-time managed install (#855); everything else keeps the error.
        Err(AuditError::ChromeNotFound)
            if auditmysite::browser::bootstrap::is_interactive(args.quiet) =>
        {
            use auditmysite::browser::bootstrap::{ensure_browser, EnsureOptions, EnsureOutcome};
            let opts = EnsureOptions {
                artifact: auditmysite::browser::install_plan::Artifact::Chrome,
                assume_yes: false,
                interactive: true,
                offline: false,
            };
            match ensure_browser(&opts).await? {
                EnsureOutcome::Installed(_) | EnsureOutcome::Present(_) => {
                    dispatch_audit(&args, config, is_batch).await
                }
                EnsureOutcome::Declined | EnsureOutcome::Refused(_) => {
                    Err(AuditError::ChromeNotFound)
                }
            }
        }
        other => other,
    }
}

async fn dispatch_audit(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
    is_batch: bool,
) -> Result<auditmysite::Verdict> {
    if args.display == Some(auditmysite::cli::DisplaySelection::All) {
        run_display_all(args, config, is_batch).await
    } else if is_batch {
        run_batch_mode(args, config).await
    } else {
        run_single_mode(args, config).await
    }
}
