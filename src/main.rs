//! AuditMySit CLI Entry Point
//!
//! Resource-efficient WCAG 2.1 Accessibility Checker in Rust

use std::fs;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Local;
use clap::Parser;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use auditmysite::audit::history::preview_report_history;
use auditmysite::audit::normalize;
use auditmysite::audit::{
    analyze_crawl_links, crawl_site, history::write_report_history, load_artifacts, parse_sitemap,
    read_url_file, run_concurrent_batch, run_single_audit, to_audit_report, BatchConfig,
    CrawlResult, PipelineConfig,
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
    format_ai_json, format_batch_table, format_json_batch, format_json_cached, print_batch_table,
    print_report, JsonReport,
};
#[cfg(feature = "pdf")]
use auditmysite::output::{format_json_normalized, generate_batch_pdf, generate_pdf};
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
        "debug,chromiumoxide=error,tungstenite=error,auditmysite=debug".to_string()
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
async fn run(mut args: Args, _config: &Option<auditmysite::cli::Config>) -> Result<f64> {
    // Handle subcommands first
    if let Some(ref command) = args.command {
        return handle_command(command).await;
    }

    if args.detect_chrome {
        return detect_chrome_command(&args);
    }

    // --compare mode: audits multiple domains for side-by-side comparison
    if !args.compare.is_empty() {
        if let Err(e) = args.validate() {
            return Err(AuditError::ConfigError(e));
        }
        if !args.quiet {
            print_banner();
        }
        return run_compare_mode(&args).await;
    }

    // If no input source specified interactively ask for a domain (terminal only)
    let no_input = args.url.is_none()
        && args.sitemap.is_none()
        && args.url_file.is_none()
        && !args.crawl
        && !args.detect_chrome;

    if no_input && !args.quiet && io::stdin().is_terminal() && io::stdout().is_terminal() {
        print_banner();
        let input: String = Input::new()
            .with_prompt("  Domain oder URL (z.B. example.com)")
            .validate_with(|s: &String| {
                if s.trim().is_empty() {
                    Err("Bitte eine URL eingeben.")
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

    if is_batch {
        run_batch_mode(&args).await
    } else {
        run_single_mode(&args, _config).await
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
async fn run_single_mode(args: &Args, config: &Option<auditmysite::cli::Config>) -> Result<f64> {
    let url = args
        .url
        .as_ref()
        .ok_or_else(|| AuditError::ConfigError("URL required".to_string()))?;

    if let Some(batch_score) = maybe_offer_sitemap_scan(args, url).await? {
        return Ok(batch_score);
    }

    // Quick reachability check before spinning up a browser
    check_url_reachable(url, args.quiet).await?;

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
                OutputFormat::Table | OutputFormat::Pdf | OutputFormat::Ai => {
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
        timeout_secs: args.effective_timeout(),
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

    let pipeline_config = PipelineConfig::from(args);
    let mut report = run_single_audit(url, &browser, &pipeline_config).await?;
    browser.close().await?;

    // Evaluate performance budgets from config
    if let Some(ref cfg) = *config {
        if !cfg.budgets.is_empty() {
            report.budget_violations = auditmysite::audit::evaluate_budgets(&report, &cfg.budgets);
            if !report.budget_violations.is_empty() && !args.quiet {
                use auditmysite::audit::BudgetSeverity;
                let errors = report
                    .budget_violations
                    .iter()
                    .filter(|v| v.severity == BudgetSeverity::Error)
                    .count();
                let warnings = report
                    .budget_violations
                    .iter()
                    .filter(|v| v.severity == BudgetSeverity::Warning)
                    .count();
                println!(
                    "{} {} Budget-Verletzung{}: {} Error{}, {} Warning{}",
                    "Budget:".yellow().bold(),
                    report.budget_violations.len(),
                    if report.budget_violations.len() == 1 {
                        ""
                    } else {
                        "en"
                    },
                    errors,
                    if errors == 1 { "" } else { "s" },
                    warnings,
                    if warnings == 1 { "" } else { "s" },
                );
            }
        }
    }

    output_single_report(&report, args)?;

    Ok(report.score as f64)
}

async fn maybe_offer_sitemap_scan(args: &Args, url: &str) -> Result<Option<f64>> {
    if args.no_sitemap_suggest {
        return Ok(None);
    }
    if !looks_like_base_url(url) {
        return Ok(None);
    }

    let Some((sitemap_url, url_count)) = discover_populated_sitemap(url).await? else {
        return Ok(None);
    };

    if args.prefer_sitemap {
        let batch_args = suggested_sitemap_batch_args(args, sitemap_url);
        return run_batch_mode(&batch_args).await.map(Some);
    }

    if args.quiet || !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(None);
    }

    println!();
    println!("{}", "Sitemap gefunden".cyan().bold());
    println!(
        "  {} {} ({} URLs)",
        "Quelle:".dimmed(),
        sitemap_url,
        url_count
    );
    println!(
        "  {}",
        "Für eine Basis-URL ist oft ein vollständiger Sitemap-Scan sinnvoller als nur die Startseite."
            .dimmed()
    );
    println!();

    let items = vec![
        "Einzelne URL prüfen (Startseite)",
        "Sitemap scannen (alle URLs)",
    ];
    let selection = Select::new()
        .with_prompt("Wie möchtest du fortfahren?")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| AuditError::ConfigError(e.to_string()))?;

    if selection == 1 {
        let batch_args = suggested_sitemap_batch_args(args, sitemap_url);
        println!();
        return run_batch_mode(&batch_args).await.map(Some);
    }

    println!();
    Ok(None)
}

fn suggested_sitemap_batch_args(args: &Args, sitemap_url: String) -> Args {
    let mut batch_args = args.clone();
    batch_args.url = None;
    batch_args.sitemap = Some(sitemap_url);

    // A user who started a single-URL audit without an explicit format expects a
    // generated report file even when switching into sitemap mode interactively.
    if batch_args.format.is_none() {
        batch_args.format = Some(OutputFormat::Pdf);
    }

    batch_args
}

async fn discover_populated_sitemap(base_url: &str) -> Result<Option<(String, usize)>> {
    let mut candidates = sitemap_candidates(base_url)?;
    for robots_sitemap in sitemap_candidates_from_robots(base_url).await {
        if !candidates.contains(&robots_sitemap) {
            candidates.push(robots_sitemap);
        }
    }

    for candidate in candidates {
        match parse_sitemap(&candidate).await {
            Ok(urls) if !urls.is_empty() => return Ok(Some((candidate, urls.len()))),
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    Ok(None)
}

fn sitemap_candidates(base_url: &str) -> Result<Vec<String>> {
    let parsed = url::Url::parse(base_url).map_err(|e| AuditError::ConfigError(e.to_string()))?;
    let base = parsed
        .join("/")
        .map_err(|e| AuditError::ConfigError(e.to_string()))?;

    let usual_suspects = [
        "sitemap.xml",
        "sitemap_index.xml",
        "sitemap-index.xml",
        "sitemaps.xml",
        "post-sitemap.xml",
        "page-sitemap.xml",
    ];

    let mut urls = Vec::new();
    for path in usual_suspects {
        if let Ok(candidate) = base.join(path) {
            urls.push(candidate.to_string());
        }
    }
    Ok(urls)
}

async fn sitemap_candidates_from_robots(base_url: &str) -> Vec<String> {
    let Ok(parsed) = url::Url::parse(base_url) else {
        return Vec::new();
    };
    let Ok(robots_url) = parsed.join("/robots.txt") else {
        return Vec::new();
    };

    let Ok(response) = reqwest::get(robots_url.clone()).await else {
        return Vec::new();
    };
    let Ok(body) = response.text().await else {
        return Vec::new();
    };

    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let (key, value) = trimmed.split_once(':')?;
            if key.trim().eq_ignore_ascii_case("sitemap") {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Check whether a URL is reachable with a lightweight HTTP request.
/// Fails fast with a human-readable error before the browser is launched.
async fn check_url_reachable(url: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{} {}", "Checking:".dimmed(), url);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent("auditmysite-preflight/1.0")
        .build()
        .map_err(|e| AuditError::ConfigError(e.to_string()))?;

    match client.head(url).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_server_error() {
                return Err(AuditError::ConfigError(format!(
                    "Server antwortet mit {} für {}. Audit wird abgebrochen.",
                    status, url
                )));
            }
            // 4xx (z.B. 405 Method Not Allowed for HEAD) → fallback to GET
            if status == reqwest::StatusCode::METHOD_NOT_ALLOWED
                || status == reqwest::StatusCode::NOT_IMPLEMENTED
            {
                // Try GET instead
                match client.get(url).send().await {
                    Ok(r) if r.status().is_server_error() => {
                        return Err(AuditError::ConfigError(format!(
                            "Server antwortet mit {} für {}. Audit wird abgebrochen.",
                            r.status(),
                            url
                        )));
                    }
                    Err(e) => {
                        return Err(AuditError::ConfigError(format!(
                            "Domain nicht erreichbar: {}",
                            e
                        )));
                    }
                    Ok(_) => {}
                }
            }
        }
        Err(e) => {
            return Err(AuditError::ConfigError(format!(
                "Domain nicht erreichbar: {}\n  Bitte Internetverbindung und URL prüfen.",
                e
            )));
        }
    }

    Ok(())
}

fn looks_like_base_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    (parsed.path().is_empty() || parsed.path() == "/")
        && parsed.query().is_none()
        && parsed.fragment().is_none()
}

/// Run batch audit mode (sitemap or URL file)
async fn run_batch_mode(args: &Args) -> Result<f64> {
    let mut crawl_result: Option<CrawlResult> = None;

    let urls = if let Some(ref sitemap_url) = args.sitemap {
        if !args.quiet {
            println!("{} {}", "Fetching sitemap:".cyan().bold(), sitemap_url);
        }
        parse_sitemap(sitemap_url).await?
    } else if args.crawl {
        let seed_url = args
            .url
            .as_deref()
            .ok_or_else(|| AuditError::ConfigError("No crawl seed URL specified".to_string()))?;
        if !args.quiet {
            println!("{} {}", "Crawling site:".cyan().bold(), seed_url);
        }
        let crawl = crawl_site(seed_url, args.max_pages, args.crawl_depth).await?;
        if !args.quiet {
            println!(
                "{} {} pages discovered at depth <= {}",
                "Discovered:".cyan().bold(),
                crawl.pages.len(),
                args.crawl_depth
            );
        }
        let urls = crawl.urls();
        crawl_result = Some(crawl);
        urls
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
            args.effective_concurrency()
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

    let mut batch_report = run_concurrent_batch(urls, &batch_config, progress).await?;

    if let Some(ref crawl) = crawl_result {
        let diagnostics = analyze_crawl_links(crawl).await;
        if !args.quiet {
            println!(
                "{} {} interne Links geprüft, {} kaputt",
                "Link-Check:".cyan().bold(),
                diagnostics.checked_internal_links,
                diagnostics.broken_internal_links.len()
            );
        }
        batch_report = batch_report.with_crawl_diagnostics(diagnostics);
    }

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

    if args.per_page_reports {
        output_batch_as_single_reports(&batch_report, args)?;
        return Ok(batch_report.summary.average_score);
    }

    output_batch_report(&batch_report, args)?;

    Ok(batch_report.summary.average_score)
}

/// Run competitive comparison mode (--compare flag)
async fn run_compare_mode(args: &Args) -> Result<f64> {
    let urls = &args.compare;

    if !args.quiet {
        println!(
            "{} {} Domains werden verglichen\n",
            "Vergleich:".cyan().bold(),
            urls.len()
        );
    }

    let browser_options = BrowserOptions {
        chrome_path: args.chrome_path.clone(),
        headless: true,
        disable_gpu: true,
        no_sandbox: args.no_sandbox,
        disable_images: args.disable_images,
        window_size: (1920, 1080),
        timeout_secs: args.effective_timeout(),
        verbose: args.verbose,
    };

    let browser = BrowserManager::with_options(browser_options).await?;
    let config = PipelineConfig::from(args);

    let start = std::time::Instant::now();
    let mut reports = Vec::new();
    let mut failed = 0usize;

    for url in urls {
        if !args.quiet {
            println!("{} {}", "Auditiere:".dimmed(), url);
        }
        match run_single_audit(url, &browser, &config).await {
            Ok(report) => reports.push(report),
            Err(e) => {
                if !args.quiet {
                    eprintln!("{} {} — {}", "Fehler:".red().bold(), url, e);
                }
                failed += 1;
            }
        }
    }

    browser.close().await?;

    if reports.is_empty() {
        return Err(AuditError::ConfigError(
            "Kein einziger Domain-Audit erfolgreich.".to_string(),
        ));
    }

    let total_ms = start.elapsed().as_millis() as u64;
    let comparison = auditmysite::audit::ComparisonReport::from_reports(reports, total_ms);

    if !args.quiet {
        if failed > 0 {
            println!(
                "{} {} Domain(s) konnten nicht auditiert werden.",
                "Warnung:".yellow().bold(),
                failed
            );
        }
        // Print summary table
        println!();
        println!("{}", "Ranking:".cyan().bold());
        for (rank, entry) in comparison.entries.iter().enumerate() {
            println!(
                "  {}. {} — Score: {}/100 ({}), {} Violations",
                rank + 1,
                entry.domain,
                entry.overall_score,
                entry.grade,
                entry.total_violations,
            );
        }
        println!();
    }

    output_comparison_report(&comparison, args)?;

    let avg = comparison
        .entries
        .iter()
        .map(|e| e.overall_score as f64)
        .sum::<f64>()
        / comparison.entries.len() as f64;
    Ok(avg)
}

fn output_comparison_report(
    comparison: &auditmysite::audit::ComparisonReport,
    args: &Args,
) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let output =
                serde_json::to_string_pretty(comparison).map_err(|e| AuditError::OutputError {
                    reason: e.to_string(),
                })?;
            output_text(&output, &args.output, "JSON comparison", args.quiet)?;
        }
        OutputFormat::Table => {
            // Summary table already printed inline above
            if let Some(path) = &args.output {
                let mut lines = vec!["Rank,Domain,Score,Grade,Violations,Critical".to_string()];
                for (i, e) in comparison.entries.iter().enumerate() {
                    lines.push(format!(
                        "{},{},{},{},{},{}",
                        i + 1,
                        e.domain,
                        e.overall_score,
                        e.grade,
                        e.total_violations,
                        e.critical_violations,
                    ));
                }
                output_text(
                    &lines.join("\n"),
                    &Some(path.clone()),
                    "CSV comparison",
                    args.quiet,
                )?;
            }
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                use auditmysite::output::generate_comparison_pdf;
                use auditmysite::output::report_model::ReportConfig;
                let config = ReportConfig {
                    level: args.report_level,

                    logo_path: args.logo.clone(),
                    locale: args.lang.clone(),
                    history_preview: None,
                };
                let pdf_bytes = generate_comparison_pdf(comparison, &config).map_err(|e| {
                    AuditError::ReportGenerationFailed {
                        reason: e.to_string(),
                    }
                })?;
                let path = args
                    .output
                    .clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("comparison-report.pdf"));
                output_bytes(&pdf_bytes, &path, "PDF comparison", args.quiet)?;
            }
            #[cfg(not(feature = "pdf"))]
            {
                return Err(AuditError::ConfigError(
                    "PDF output requires the 'pdf' feature.".to_string(),
                ));
            }
        }
        OutputFormat::Ai => {
            let output =
                serde_json::to_string_pretty(comparison).map_err(|e| AuditError::OutputError {
                    reason: e.to_string(),
                })?;
            output_text(&output, &args.output, "AI JSON comparison", args.quiet)?;
        }
    }
    Ok(())
}

/// Output a single audit report in the requested format
fn output_single_report(report: &auditmysite::AuditReport, args: &Args) -> Result<()> {
    match args.effective_format() {
        OutputFormat::Json => {
            let normalized = normalize(report);
            let mut json_report = JsonReport::from_normalized(&normalized, report);
            if let Some(path) = args.output.as_ref() {
                if let Ok(Some(preview)) =
                    preview_report_history(output_directory(path), path, &normalized)
                {
                    json_report.history = serde_json::to_value(&preview).ok();
                }
            }
            let output = json_report.to_json(true)?;
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
        OutputFormat::Ai => {
            let output = format_ai_json(report);
            output_text(&output, &args.output, "AI JSON", args.quiet)?;
        }
    }
    Ok(())
}

fn maybe_write_single_history(
    output_path: &std::path::Path,
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
            if let Some(path) = &args.output {
                let output = format_batch_table(batch_report, args.level, false);
                output_text(&output, &Some(path.clone()), "Table batch", args.quiet)?;
            } else {
                print_batch_table(batch_report, args.level);
            }
        }
        OutputFormat::Pdf => {
            #[cfg(feature = "pdf")]
            {
                let config = ReportConfig {
                    level: args.report_level,

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
                    .unwrap_or_else(|| default_batch_pdf_output_path(args));
                output_bytes(&pdf_bytes, &path, "PDF batch", args.quiet)?;

                // Auto-generate JSON alongside batch PDF
                let json_path = path.with_extension("json");
                let json_output = format_json_batch(batch_report, true)?;
                output_text(&json_output, &Some(json_path), "JSON batch", args.quiet)?;
            }
            #[cfg(not(feature = "pdf"))]
            {
                return Err(AuditError::ConfigError(
                    "PDF output requires the 'pdf' feature. Rebuild with: cargo build --features pdf".to_string(),
                ));
            }
        }
        OutputFormat::Ai => {
            // For batch mode, emit one AI JSON document per URL.
            // Each report is separated by a newline-delimited JSON array.
            let outputs: Vec<String> = batch_report.reports.iter().map(format_ai_json).collect();
            let combined = format!("[\n{}\n]", outputs.join(",\n"));
            output_text(&combined, &args.output, "AI JSON batch", args.quiet)?;
        }
    }
    Ok(())
}

fn output_batch_as_single_reports(
    batch_report: &auditmysite::audit::BatchReport,
    args: &Args,
) -> Result<()> {
    let base_dir = per_page_output_directory(args);

    if !args.quiet {
        println!(
            "{} {} Einzelreports nach {}",
            "Info:".cyan().bold(),
            batch_report.reports.len(),
            base_dir.display()
        );
    }

    for report in &batch_report.reports {
        let mut single_args = args.clone();
        single_args.url = Some(report.url.clone());
        single_args.sitemap = None;
        single_args.url_file = None;
        single_args.output = Some(per_page_output_path(
            &base_dir,
            &report.url,
            single_args.effective_format(),
            single_args.report_level,
        ));
        output_single_report(report, &single_args)?;
    }

    Ok(())
}

/// Write text content to file or stdout
fn output_text(content: &str, path: &Option<PathBuf>, label: &str, quiet: bool) -> Result<()> {
    if let Some(path) = path {
        fs::create_dir_all(output_directory(path))?;
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

#[cfg(feature = "pdf")]
fn default_single_json_output_path(pdf_path: &std::path::Path) -> PathBuf {
    pdf_path.with_extension("json")
}

fn per_page_output_directory(args: &Args) -> PathBuf {
    match args.output.as_ref() {
        Some(path) if path.extension().is_none() => path.clone(),
        Some(path) => output_directory(path).to_path_buf(),
        None => PathBuf::from("."),
    }
}

fn per_page_output_path(
    base_dir: &std::path::Path,
    url: &str,
    format: OutputFormat,
    report_level: auditmysite::cli::ReportLevel,
) -> PathBuf {
    let _ = report_level;
    let date = Local::now().format("%Y-%m-%d");
    let subject = report_subject_from_url(url);
    let filename = match format {
        OutputFormat::Pdf => default_single_pdf_output_path(url, report_level),
        OutputFormat::Json => PathBuf::from(format!("{subject}-{date}-single-report.json")),
        OutputFormat::Table => PathBuf::from(format!("{subject}-{date}-single-report.txt")),
        OutputFormat::Ai => PathBuf::from(format!("{subject}-{date}-single-report-ai.json")),
    };

    match filename.file_name() {
        Some(name) => base_dir.join(name),
        None => base_dir.join(filename),
    }
}

fn default_single_pdf_output_path(
    url: &str,
    report_level: auditmysite::cli::ReportLevel,
) -> PathBuf {
    let _ = report_level;
    let date = Local::now().format("%Y-%m-%d");
    let subject = report_subject_from_url(url);
    PathBuf::from(format!("{subject}-{date}-single-report.pdf"))
}

#[cfg(feature = "pdf")]
fn default_batch_pdf_output_path(args: &Args) -> PathBuf {
    let date = Local::now().format("%Y-%m-%d");
    let kind = if args.sitemap.is_some() {
        "sitemap"
    } else if args.crawl {
        "crawl"
    } else {
        "batch"
    };
    PathBuf::from(format!("{kind}-report-{date}.pdf"))
}

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
#[allow(clippy::items_after_test_module)]
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
        assert!(rendered.ends_with("-single-report.pdf"));
        assert!(rendered.contains("in-punkto-com-"));
        assert!(!rendered.contains('/'));
    }

    #[test]
    fn test_output_directory_defaults_to_current_directory_for_bare_filename() {
        let path = PathBuf::from("casoon-de-2026-03-31-single-report.pdf");
        assert_eq!(output_directory(&path), std::path::Path::new("."));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_default_single_json_output_path_matches_pdf_basename() {
        let pdf_path = PathBuf::from("casoon-de-2026-03-31-single-report.pdf");
        assert_eq!(
            default_single_json_output_path(&pdf_path),
            PathBuf::from("casoon-de-2026-03-31-single-report.json")
        );
    }

    #[test]
    fn test_per_page_output_directory_uses_output_directory_for_file_path() {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.per_page_reports = true;
        args.output = Some(PathBuf::from("reports/custom-name.pdf"));
        assert_eq!(per_page_output_directory(&args), PathBuf::from("reports"));
    }

    #[test]
    fn test_per_page_output_directory_uses_path_directly_for_directory_like_output() {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.per_page_reports = true;
        args.output = Some(PathBuf::from("reports/per-page"));
        assert_eq!(
            per_page_output_directory(&args),
            PathBuf::from("reports/per-page")
        );
    }

    #[test]
    fn test_per_page_output_path_uses_url_slug_and_extension() {
        let path = per_page_output_path(
            std::path::Path::new("reports"),
            "https://www.in-punkto.com/leistungen/",
            OutputFormat::Pdf,
            ReportLevel::Standard,
        );
        let rendered = path.display().to_string();
        assert!(rendered.starts_with("reports/"));
        assert!(rendered.ends_with("-single-report.pdf"));
        assert!(rendered.contains("in-punkto-com-"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_default_batch_pdf_output_path_uses_sitemap_kind() {
        let args = Args::parse_from([
            "auditmysite",
            "--sitemap",
            "https://example.com/sitemap.xml",
        ]);
        let rendered = default_batch_pdf_output_path(&args).display().to_string();
        assert!(rendered.contains("sitemap-report-"));
        assert!(rendered.ends_with(".pdf"));
        assert!(!rendered.contains("reports/"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_default_batch_pdf_output_path_uses_crawl_kind() {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.crawl = true;
        let rendered = default_batch_pdf_output_path(&args).display().to_string();
        assert!(rendered.contains("crawl-report-"));
    }

    #[test]
    fn test_per_page_output_path_uses_single_report_name_for_json() {
        let path = per_page_output_path(
            std::path::Path::new("reports"),
            "https://www.in-punkto.com/leistungen/",
            OutputFormat::Json,
            ReportLevel::Standard,
        );
        let rendered = path.display().to_string();
        assert!(rendered.ends_with("-single-report.json"));
    }

    #[test]
    fn test_looks_like_base_url_accepts_root_url() {
        assert!(looks_like_base_url("https://www.casoon.de"));
        assert!(looks_like_base_url("https://www.casoon.de/"));
        assert!(!looks_like_base_url("https://www.casoon.de/blog"));
        assert!(!looks_like_base_url("https://www.casoon.de/?p=1"));
    }

    #[test]
    fn test_sitemap_candidates_include_usual_suspects() {
        let candidates = sitemap_candidates("https://www.casoon.de").unwrap();
        assert!(candidates.contains(&"https://www.casoon.de/sitemap.xml".to_string()));
        assert!(candidates.contains(&"https://www.casoon.de/sitemap_index.xml".to_string()));
        assert!(candidates.contains(&"https://www.casoon.de/page-sitemap.xml".to_string()));
    }

    #[test]
    fn test_suggested_sitemap_batch_args_defaults_to_pdf() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let batch_args =
            suggested_sitemap_batch_args(&args, "https://example.com/sitemap.xml".into());

        assert!(batch_args.url.is_none());
        assert_eq!(
            batch_args.sitemap.as_deref(),
            Some("https://example.com/sitemap.xml")
        );
        assert_eq!(batch_args.format, Some(OutputFormat::Pdf));
    }

    #[test]
    fn test_suggested_sitemap_batch_args_preserves_explicit_format() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "json"]);
        let batch_args =
            suggested_sitemap_batch_args(&args, "https://example.com/sitemap.xml".into());

        assert_eq!(batch_args.format, Some(OutputFormat::Json));
    }
}

/// Print application banner
fn print_banner() {
    println!(
        "{}",
        r#"
    _             _ _ _   __  __       ____  _ _
   / \  _   _  __| (_) |_|  \/  |_   _/ ___|(_) |_ ___
  / _ \| | | |/ _` | | __| |\/| | | | \___ \| | __/ _ \
 / ___ \ |_| | (_| | | |_| |  | | |_| |___) | | ||  __/
/_/   \_\__,_|\__,_|_|\__|_|  |_|\__, |____/|_|\__\___|
                                 |___/
"#
        .cyan()
    );
    println!(
        "  {} v{} - WCAG 2.1 Accessibility Checker\n",
        "AuditMySite".bold(),
        env!("CARGO_PKG_VERSION")
    );
}
