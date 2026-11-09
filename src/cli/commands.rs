//! CLI subcommand handlers.
//!
//! Handles the browser and doctor subcommands. No mode-runner or audit logic
//! here. The maintainer commands live in
//! the `auditmysite-dev` binary (`dev_commands.rs`, #912).

use colored::Colorize;

use auditmysite::browser::{
    diagnose_browsers, resolve_browser, BrowserInstaller, BrowserResolveOptions, InstallTarget,
};
use auditmysite::cli::{Args, BrowserAction, Command};
use auditmysite::error::Result;

pub(crate) async fn handle_command(command: &Command, args: &Args) -> Result<f64> {
    match command {
        Command::Browser { action } => handle_browser_command(action, args.no_sandbox).await,
        Command::Doctor => {
            auditmysite::cli::doctor::run_doctor(args.no_sandbox).await;
            Ok(0.0)
        }
    }
}

async fn handle_browser_command(action: &BrowserAction, no_sandbox: bool) -> Result<f64> {
    match action {
        BrowserAction::Detect => {
            println!("{}", "Detecting browsers...".cyan().bold());
            println!();

            let opts = BrowserResolveOptions::default();
            let diagnosis = diagnose_browsers(&opts, no_sandbox).await;
            auditmysite::cli::doctor::print_browser_diagnosis(&diagnosis);

            println!();
            match diagnosis.selected() {
                Some(active) => println!(
                    "  {} Active: {} v{} ({})",
                    "→".cyan(),
                    active.browser.kind.display_name(),
                    active.browser.version.as_deref().unwrap_or("unknown"),
                    active.browser.source,
                ),
                None => println!("  {} No browser can be resolved for auditing.", "✗".red()),
            }

            Ok(0.0)
        }

        BrowserAction::Install {
            headless_shell,
            version,
            force,
            offline,
        } => {
            let target = if *headless_shell {
                InstallTarget::HeadlessShell
            } else {
                InstallTarget::ChromeForTesting
            };
            if *offline {
                // `browser install` is the explicit request; offline only
                // forbids the network.
                let request = auditmysite::browser::install_txn::InstallRequest {
                    artifact: target.into(),
                    version: version.clone(),
                    force: *force,
                    offline: true,
                };
                let outcome = BrowserInstaller::run(&request).await?;
                println!(
                    "Using validated managed browser v{} at {}",
                    outcome.version(),
                    outcome.binary().display()
                );
            } else {
                BrowserInstaller::install(target, version.as_deref(), *force).await?;
            }
            Ok(0.0)
        }

        BrowserAction::Ensure {
            yes,
            headless_shell,
            offline,
        } => {
            use auditmysite::browser::bootstrap::{
                ensure_browser, is_interactive, EnsureOptions, EnsureOutcome,
            };
            let opts = EnsureOptions {
                artifact: if *headless_shell {
                    auditmysite::browser::install_plan::Artifact::HeadlessShell
                } else {
                    auditmysite::browser::install_plan::Artifact::Chrome
                },
                assume_yes: *yes,
                interactive: is_interactive(false),
                offline: *offline,
            };
            match ensure_browser(&opts).await? {
                EnsureOutcome::Present(path) => {
                    println!("Browser available: {}", path.display());
                    Ok(0.0)
                }
                EnsureOutcome::Installed(path) => {
                    println!("Installed browser: {}", path.display());
                    Ok(0.0)
                }
                EnsureOutcome::Declined => {
                    eprintln!("No browser installed (declined).");
                    std::process::exit(1);
                }
                EnsureOutcome::Refused(reason) => {
                    eprintln!("{} {}", "Error:".red().bold(), reason);
                    std::process::exit(1);
                }
            }
        }

        BrowserAction::Remove { all } => {
            if *all {
                BrowserInstaller::remove_all()?;
            } else {
                BrowserInstaller::remove(InstallTarget::ChromeForTesting)?;
            }
            Ok(0.0)
        }

        BrowserAction::Path => {
            let opts = BrowserResolveOptions::default();
            match resolve_browser(&opts) {
                Ok(resolved) => {
                    println!("{}", resolved.browser.path.display());
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
            Ok(0.0)
        }
    }
}
