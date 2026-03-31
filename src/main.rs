//! AuditMySit CLI Entry Point
//!
//! Resource-efficient WCAG 2.1 Accessibility Checker in Rust

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(any(feature = "pdf", test))]
use chrono::Local;
use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[cfg(feature = "pdf")]
use auditmysite::audit::history::preview_report_history;
use auditmysite::audit::normalize;
use auditmysite::audit::{
    history::write_report_history, load_artifacts, parse_sitemap, read_url_file,
    run_concurrent_batch, run_single_audit, to_audit_report, BatchConfig, PipelineConfig,
};
use auditmysite::browser::{
    detect_all_browsers, find_chrome, resolve_browser, BrowserInstaller, BrowserManager,
    BrowserOptions, BrowserResolveOptions, InstallTarget,
};
use auditmysite::cli::{Args, BrowserAction, Command, OutputFormat};
use auditmysite::error::{AuditError, Result};
#[cfg(feature = "pdf")]
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::{
    format_json_batch, format_json_cached, format_json_normalized, print_batch_table, print_report,
};
#[cfg(feature = "pdf")]
use auditmysite::output::{generate_batch_pdf, generate_pdf};
use auditmysite::util::truncate_url;

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    setup_logging(&args);

    // Load config file and apply defaults (CLI args take precedence)
    let config = auditmysite::cli::Config::load();
    if let Some(ref cfg) = config {
        cfg.apply_to_args(&mut args);
    }

    let exit_code = match run(args, &config).await {
        Ok(score) => {
            // Check score threshold from config
            if let Some(min_score) = auditmysite::cli::config::get_min_score_threshold(&config) {
                if score < min_score {
                    eprintln!(
                        "{} Score {:.1} is below threshold {:.1}",
                        "FAIL:".red().bold(),
                        score,
                        min_score
                    );
                    1
                } else {
                    0
                }
            } else {
                0
            }
        }
        Err(e) => {
            error!("{}", e);
            eprintln!("{} {}", "Error:".red().bold(), e);
            2
        }
    };

    std::process::exit(exit_code);
}

/// Setup tracing/logging based on CLI flags
fn setup_logging(args: &Args) {
    let filter = if args.quiet {
        "error,chromiumoxide=off,tungstenite=off".to_string()
    } else if args.verbose {
        "debug,chromiumoxide=info,tungstenite=warn".to_string()
    } else {
        "warn,chromiumoxide=off,tungstenite=off,auditmysite=warn".to_string()
    };

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

/// Main application logic
/// Returns the audit score (or 0.0 for non-scoring operations).
async fn run(args: Args, _config: &Option<auditmysite::cli::Config>) -> Result<f64> {
    // Handle subcommands first
    if let Some(ref command) = args.command {
        return handle_command(command).await;
    }

    if args.detect_chrome {
        return detect_chrome_command(&args);
    }

    if let Err(e) = args.validate() {
        return Err(AuditError::ConfigError(e));
    }

    if !args.quiet {
        print_banner();
    }

    let is_batch = args.sitemap.is_some() || args.url_file.is_some();

    if is_batch {
        run_batch_mode(&args).await
    } else {
        run_single_mode(&args).await
    }
}

/// Handle subcommands (browser, doctor)
async fn handle_command(command: &Command) -> Result<f64> {
    match command {
        Command::Browser { action } => handle_browser_command(action).await,
        Command::Doctor => {
            auditmysite::cli::doctor::run_doctor();
            Ok(0.0)
        }
    }
}

/// Handle browser subcommands
async fn handle_browser_command(action: &BrowserAction) -> Result<f64> {
    match action {
        BrowserAction::Detect => {
            println!("{}", "Detecting browsers...".cyan().bold());
            println!();

            let browsers = detect_all_browsers();
            if browsers.is_empty() {
                println!("  No browsers found.");
                println!();
                println!("  Install one:");
                println!("    brew install --cask google-chrome");
                println!("    auditmysite browser install");
            } else {
                for browser in &browsers {
                    println!(
                        "  {} {:<25} {:<15} {}",
                        "✓".green(),
                        browser.kind.display_name(),
                        browser.version.as_deref().unwrap_or("unknown"),
                        browser.path.display()
                    );
                }
            }

            // Check managed installs
            if let Some(home) = dirs::home_dir() {
                let browsers_dir = home.join(".auditmysite").join("browsers");
                if browsers_dir.exists() {
                    let cft = browsers_dir.join("chrome-for-testing");
                    let hs = browsers_dir.join("headless-shell");
                    if cft.exists() {
                        let version = std::fs::read_to_string(cft.join("version.txt"))
                            .unwrap_or_else(|_| "unknown".to_string());
                        println!(
                            "  {} {:<25} {:<15} {}",
                            "✓".green(),
                            "Chrome for Testing",
                            version.trim(),
                            cft.display()
                        );
                    }
                    if hs.exists() {
                        let version = std::fs::read_to_string(hs.join("version.txt"))
                            .unwrap_or_else(|_| "unknown".to_string());
                        println!(
                            "  {} {:<25} {:<15} {}",
                            "✓".green(),
                            "Headless Shell",
                            version.trim(),
                            hs.display()
                        );
                    }
                }
            }

            // Show active browser
            println!();
            let opts = BrowserResolveOptions::default();
            match resolve_browser(&opts) {
                Ok(resolved) => {
                    println!(
                        "  {} Active: {} v{} ({})",
                        "→".cyan(),
                        resolved.browser.kind.display_name(),
                        resolved.browser.version.as_deref().unwrap_or("unknown"),
                        resolved.browser.source,
                    );
                }
                Err(_) => {
                    println!("  {} No browser can be resolved for auditing.", "✗".red());
                }
            }

            Ok(0.0)
        }

        BrowserAction::Install {
            headless_shell,
            version,
            force,
        } => {
            let target = if *headless_shell {
                InstallTarget::HeadlessShell
            } else {
                InstallTarget::ChromeForTesting
            };
            BrowserInstaller::install(target, version.as_deref(), *force).await?;
            Ok(0.0)
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

/// Run single URL audit mode
async fn run_single_mode(args: &Args) -> Result<f64> {
    let url = args
        .url
        .as_ref()
        .ok_or_else(|| AuditError::ConfigError("URL required".to_string()))?;

    info!("Starting audit for: {}", url);

    if args.reuse_cache && !args.force_refresh {
        if let Some(cached) = load_artifacts(url)? {
            if !args.quiet {
                println!(
                    "{} {}",
                    "Cache hit:".green().bold(),
                    "verwende vorhandene Audit-Artefakte".dimmed()
                );
            }

            match args.effective_format() {
                OutputFormat::Json => {
                    let output = format_json_cached(&cached.audit, true)?;
                    output_text(&output, &args.output, "JSON", args.quiet)?;
                    return Ok(cached.audit.overall_score as f64);
                }
                OutputFormat::Table | OutputFormat::Pdf => {
                    let report = to_audit_report(&cached);
                    output_single_report(&report, args)?;
                    return Ok(report.score as f64);
                }
            }
        }
    }

    let browser_options = BrowserOptions {
        chrome_path: args.chrome_path.clone(),
        headless: true,
        disable_gpu: true,
        no_sandbox: args.no_sandbox,
        disable_images: args.disable_images,
        window_size: (1920, 1080),
        timeout_secs: args.timeout,
        verbose: args.verbose,
    };

    if !args.quiet {
        println!("{}", "Launching browser...".dimmed());
    }
    let browser = BrowserManager::with_options(browser_options).await?;

    if !args.quiet {
        println!(
            "{} Chrome {} at {}",
            "Found:".green().bold(),
            browser.chrome_version().unwrap_or("unknown version"),
            browser.chrome_path().display()
        );
        println!("{} {}", "Auditing:".cyan().bold(), url);
    }

    let config = PipelineConfig::from(args);
    let report = run_single_audit(url, &browser, &config).await?;
    browser.close().await?;

    output_single_report(&report, args)?;

    Ok(report.score as f64)
}

/// Run batch audit mode (sitemap or URL file)
async fn run_batch_mode(args: &Args) -> Result<f64> {
    let urls = if let Some(ref sitemap_url) = args.sitemap {
        if !args.quiet {
            println!("{} {}", "Fetching sitemap:".cyan().bold(), sitemap_url);
        }
        parse_sitemap(sitemap_url).await?
    } else if let Some(ref url_file) = args.url_file {
        if !args.quiet {
            println!(
                "{} {}",
                "Reading URL file:".cyan().bold(),
                url_file.display()
            );
        }
        read_url_file(url_file.to_str().unwrap_or(""))?
    } else {
        return Err(AuditError::ConfigError(
            "No batch source specified".to_string(),
        ));
    };

    if urls.is_empty() {
        if !args.quiet {
            println!("{} No URLs found to audit.", "Warning:".yellow().bold());
        }
        return Ok(0.0);
    }

    let total_urls = if args.max_pages > 0 {
        args.max_pages.min(urls.len())
    } else {
        urls.len()
    };

    if !args.quiet {
        println!(
            "{} {} URLs with {} concurrent workers\n",
            "Auditing:".cyan().bold(),
            total_urls,
            args.concurrency
        );
    }

    let batch_config = BatchConfig::from(args);

    let progress_bar = if !args.quiet {
        let pb = ProgressBar::new(total_urls as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .expect("Invalid template")
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    #[allow(clippy::type_complexity)]
    let progress: Option<Arc<dyn Fn(usize, usize, &str) + Send + Sync>> =
        if let Some(ref pb) = progress_bar {
            let pb_clone = pb.clone();
            Some(Arc::new(move |current, _total, url| {
                pb_clone.set_position(current as u64);
                pb_clone.set_message(truncate_url(url, 50));
            }))
        } else {
            None
        };

    let batch_report = run_concurrent_batch(urls, &batch_config, progress).await?;

    if let Some(pb) = progress_bar {
        pb.finish_with_message("Complete");
    }

    if !args.quiet {
        println!(
            "\n{} {}/{} passed, {} violations found in {}ms",
            "Results:".green().bold(),
            batch_report.summary.passed,
            batch_report.summary.total_urls,
            batch_report.summary.total_violations,
            batch_report.total_duration_ms
        );
    }

    output_batch_report(&batch_report, args)?;

    Ok(batch_report.summary.average_score)
}

/// Output a single audit report in the requested format
fn output_single_report(report: &auditmysite::AuditReport, args: &Args) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let normalized = normalize(report);
            let output = format_json_normalized(&normalized, report, true)?;
            output_text(&output, &args.output, "JSON", args.quiet)?;
            if let Some(path) = args.output.as_ref() {
                maybe_write_single_history(path, &normalized, args.quiet)?;
            }
        }
        OutputFormat::Table => {
            print_report(report, args.level);
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                if !args.quiet {
                    print_report(report, args.level);
                }
                let normalized = normalize(report);
                let path = args.output.clone().unwrap_or_else(|| {
                    default_single_pdf_output_path(report.url.as_str(), args.report_level)
                });
                let auto_json_path = if args.output.is_none() {
                    Some(default_single_json_output_path(&path))
                } else {
                    None
                };
                let history_preview =
                    preview_report_history(output_directory(&path), &path, &normalized)
                        .ok()
                        .flatten()
                        .map(
                            |preview| auditmysite::output::report_model::ReportHistoryPreview {
                                previous_date: preview.previous_date,
                                timeline_entries: preview.timeline_entries,
                                previous_accessibility_score: preview.previous_accessibility_score,
                                previous_overall_score: preview.previous_overall_score,
                                delta_accessibility: preview.delta.accessibility_score_delta,
                                delta_overall: preview.delta.overall_score_delta,
                                delta_total_issues: preview.delta.total_issues_delta,
                                delta_critical_issues: preview.delta.critical_issues_delta,
                                recent_entries: preview
                                    .recent_entries
                                    .into_iter()
                                    .map(|entry| {
                                        (
                                            entry.timestamp.format("%d.%m.%Y").to_string(),
                                            entry.accessibility_score,
                                            entry.overall_score,
                                            entry.grade,
                                            entry.severity_counts.total as u32,
                                        )
                                    })
                                    .collect(),
                                new_findings: preview.delta.new_findings,
                                resolved_findings: preview.delta.resolved_findings,
                            },
                        );
                let config = ReportConfig {
                    level: args.report_level,
                    company_name: args.company_name.clone(),
                    logo_path: args.logo.clone(),
                    locale: args.lang.clone(),
                    history_preview,
                };
                let pdf_bytes = generate_pdf(report, &config).map_err(|e| {
                    AuditError::ReportGenerationFailed {
                        reason: e.to_string(),
                    }
                })?;
                if let Some(json_path) = auto_json_path.as_ref() {
                    let json_output = format_json_normalized(&normalized, report, true)?;
                    output_text(&json_output, &Some(json_path.clone()), "JSON", args.quiet)?;
                    maybe_write_single_history(json_path, &normalized, args.quiet)?;
                }
                output_bytes(&pdf_bytes, &path, "PDF", args.quiet)?;
                if auto_json_path.is_none() {
                    maybe_write_single_history(&path, &normalized, args.quiet)?;
                }
            }
            #[cfg(not(feature = "pdf"))]
            {
                return Err(AuditError::ConfigError(
                    "PDF output requires the 'pdf' feature. Rebuild with: cargo build --features pdf".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn maybe_write_single_history(
    output_path: &PathBuf,
    normalized: &auditmysite::audit::NormalizedReport,
    quiet: bool,
) -> Result<()> {
    let reports_dir = output_directory(output_path);
    let written = write_report_history(reports_dir, output_path, normalized)?;
    if !quiet {
        for path in written {
            println!(
                "{} History updated: {}",
                "Info:".cyan().bold(),
                path.display()
            );
        }
    }
    Ok(())
}

/// Output batch audit results in the requested format
fn output_batch_report(batch_report: &auditmysite::audit::BatchReport, args: &Args) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let output = format_json_batch(batch_report, true)?;
            output_text(&output, &args.output, "JSON batch", args.quiet)?;
        }
        OutputFormat::Table => {
            print_batch_table(batch_report, args.level);
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                let config = ReportConfig {
                    level: args.report_level,
                    company_name: args.company_name.clone(),
                    logo_path: args.logo.clone(),
                    locale: args.lang.clone(),
                    history_preview: None,
                };
                let pdf_bytes = generate_batch_pdf(batch_report, &config).map_err(|e| {
                    AuditError::ReportGenerationFailed {
                        reason: e.to_string(),
                    }
                })?;
                let path = args
                    .output
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("reports/batch-audit-report.pdf"));
                output_bytes(&pdf_bytes, &path, "PDF batch", args.quiet)?;
            }
            #[cfg(not(feature = "pdf"))]
            {
                return Err(AuditError::ConfigError(
                    "PDF output requires the 'pdf' feature. Rebuild with: cargo build --features pdf".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Write text content to file or stdout
fn output_text(content: &str, path: &Option<PathBuf>, label: &str, quiet: bool) -> Result<()> {
    if let Some(path) = path {
        fs::write(path, content).map_err(|e| AuditError::FileError {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        if !quiet {
            println!(
                "{} {} report saved to {}",
                "Success:".green().bold(),
                label,
                path.display()
            );
        }
    } else {
        println!("{}", content);
    }
    Ok(())
}

fn output_directory(path: &std::path::Path) -> &std::path::Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => std::path::Path::new("."),
    }
}

#[cfg(any(feature = "pdf", test))]
fn default_single_json_output_path(pdf_path: &PathBuf) -> PathBuf {
    pdf_path.with_extension("json")
}

#[cfg(any(feature = "pdf", test))]
fn default_single_pdf_output_path(
    url: &str,
    report_level: auditmysite::cli::ReportLevel,
) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    let subject = report_subject_from_url(url);
    PathBuf::from(format!("{subject}-{date}-{report_level}.pdf"))
}

#[cfg(any(feature = "pdf", test))]
fn report_subject_from_url(url: &str) -> String {
    let fallback = "audit-report".to_string();
    let Ok(parsed) = url::Url::parse(url) else {
        return fallback;
    };
    let Some(host) = parsed.host_str() else {
        return fallback;
    };

    let host = host.strip_prefix("www.").unwrap_or(host);
    let slug: String = host
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        fallback
    } else {
        slug
    }
}

/// Write binary content to file
#[cfg(feature = "pdf")]
fn output_bytes(content: &[u8], path: &PathBuf, label: &str, quiet: bool) -> Result<()> {
    fs::create_dir_all(output_directory(path))?;
    fs::write(path, content)?;
    if !quiet {
        println!(
            "{} {} report saved to {}",
            "Success:".green().bold(),
            label,
            path.display()
        );
    }
    Ok(())
}

/// Handle --detect-chrome command
fn detect_chrome_command(args: &Args) -> Result<f64> {
    println!("{}", "Detecting Chrome/Chromium...".cyan().bold());
    println!();

    match find_chrome(args.chrome_path.as_deref()) {
        Ok(info) => {
            println!("{} Chrome found!", "Success:".green().bold());
            println!("  Path:    {}", info.path.display());
            println!(
                "  Version: {}",
                info.version.as_deref().unwrap_or("unknown")
            );
            println!("  Method:  {:?}", info.detection_method);
            Ok(0.0)
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auditmysite::cli::ReportLevel;

    #[test]
    fn test_report_subject_from_url_strips_www_and_normalizes() {
        assert_eq!(
            report_subject_from_url("https://www.in-punkto.com"),
            "in-punkto-com"
        );
    }

    #[test]
    fn test_default_single_pdf_output_path_uses_current_directory_filename() {
        let path =
            default_single_pdf_output_path("https://www.in-punkto.com", ReportLevel::Standard);
        let rendered = path.display().to_string();
        assert!(rendered.ends_with("-standard.pdf"));
        assert!(rendered.contains("in-punkto-com-"));
        assert!(!rendered.contains('/'));
    }

    #[test]
    fn test_output_directory_defaults_to_current_directory_for_bare_filename() {
        let path = PathBuf::from("casoon-de-2026-03-31-standard.pdf");
        assert_eq!(output_directory(&path), std::path::Path::new("."));
    }

    #[test]
    fn test_default_single_json_output_path_matches_pdf_basename() {
        let pdf_path = PathBuf::from("casoon-de-2026-03-31-standard.pdf");
        assert_eq!(
            default_single_json_output_path(&pdf_path),
            PathBuf::from("casoon-de-2026-03-31-standard.json")
        );
    }
}

/// Print application banner
fn print_banner() {
    println!(
        "{}",
        r#"
    _             _ _ _   __  __       ____  _ _
   / \  _   _  __| (_) |_|  \/  |_   _/ ___|(_) |_
  / _ \| | | |/ _` | | __| |\/| | | | \___ \| | __|
 / ___ \ |_| | (_| | | |_| |  | | |_| |___) | | |_
/_/   \_\__,_|\__,_|_|\__|_|  |_|\__, |____/|_|\__|
                                 |___/
"#
        .cyan()
    );
    println!(
        "  {} v{} - WCAG 2.1 Accessibility Checker\n",
        "AuditMySit".bold(),
        env!("CARGO_PKG_VERSION")
    );
}
