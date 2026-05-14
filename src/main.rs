//! AuditMySit CLI Entry Point
//!
//! Bootstrap, logging setup, and top-level command dispatch.
//! Orchestration logic lives in the sibling modules declared below.

mod commands;
mod output_paths;
mod plan;
mod report_writers;

use commands::{detect_chrome_command, handle_command};
use output_paths::output_text;
use plan::{
    print_banner, print_batch_audit_plan, print_comparison_audit_plan, print_single_audit_plan,
};
use report_writers::{
    output_batch_as_single_reports, output_batch_report, output_comparison_report,
    output_single_report,
};

use std::io::{self, IsTerminal};
use std::sync::Arc;

use clap::Parser;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use auditmysite::audit::normalize;
use auditmysite::audit::{
    analyze_crawl_links, count_sitemap_entries_shallow, crawl_site, load_artifacts, parse_sitemap,
    read_url_file, run_concurrent_batch, run_single_audit, to_audit_report, BatchConfig,
    CrawlResult, PipelineConfig,
};
use auditmysite::browser::{BrowserManager, BrowserOptions};
use auditmysite::cli::{Args, OutputFormat};
use auditmysite::error::{AuditError, Result};
use auditmysite::output::format_json_cached;
use auditmysite::util::{build_browser_client, truncate_url};

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
        return handle_command(command, &args).await;
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

    if is_batch {
        run_batch_mode(&args).await
    } else {
        run_single_mode(&args, _config).await
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

    print_single_audit_plan(args, url);

    // Quick reachability check before spinning up a browser
    check_url_reachable(url, args.quiet).await?;

    info!("Starting audit for: {}", url);

    if args.reuse_cache && !args.force_refresh {
        if let Some(cached) = load_artifacts(url)? {
            if !args.quiet {
                println!(
                    "{} {}",
                    "Cache hit:".green().bold(),
                    "using cached audit artifacts".dimmed()
                );
            }

            match args.effective_format() {
                OutputFormat::Json => {
                    let output = format_json_cached(&cached.audit, true)?;
                    output_text(&output, &args.output, "JSON", args.quiet)?;
                    return Ok(cached.audit.score as f64);
                }
                OutputFormat::Table
                | OutputFormat::Pdf
                | OutputFormat::Ai
                | OutputFormat::Summary => {
                    let report = to_audit_report(&cached);
                    output_single_report(&report, args)?;
                    return Ok(normalize(&report).score as f64);
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
        println!("{}", "Starting browser...".dimmed());
    }
    let browser = BrowserManager::with_options(browser_options).await?;

    if !args.quiet {
        println!(
            "{} Chrome {} ({})",
            "Found:".green().bold(),
            browser.chrome_version().unwrap_or("unknown version"),
            browser.chrome_path().display()
        );
        println!("{} {}", "Auditing:".cyan().bold(), url);
    }

    let pipeline_config = PipelineConfig::from(args);
    let audit_result = run_single_audit(url, &browser, &pipeline_config).await;
    let close_result = browser.close().await;
    let mut report = audit_result?;
    close_result?;

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
                    "{} {}{}: {} Error{}, {} Warning{}",
                    "Budget:".yellow().bold(),
                    report.budget_violations.len(),
                    if report.budget_violations.len() == 1 {
                        " violation"
                    } else {
                        " violations"
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

    Ok(normalize(&report).score as f64)
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

    if args.quiet {
        return Ok(None);
    }

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        println!();
        println!("{}", "Sitemap found".cyan().bold());
        println!(
            "  {} {} ({} URLs)",
            "Source:".dimmed(),
            sitemap_url,
            url_count
        );
        println!(
            "  {}",
            "Non-interactive run: only the specified single URL will be audited. Use --prefer-sitemap or --sitemap for a full scan."
                .dimmed()
        );
        println!();
        return Ok(None);
    }

    println!();
    println!("{}", "Sitemap found".cyan().bold());
    println!(
        "  {} {} ({} URLs)",
        "Source:".dimmed(),
        sitemap_url,
        url_count
    );
    println!(
        "  {}",
        "For a base URL, a full sitemap scan is often more useful than just the homepage.".dimmed()
    );
    println!();

    let sample_label =
        "Sample scan (20 URLs) — average across pages, good for template issues".to_string();
    let full_label = format!("Scan sitemap (all {} URLs)", url_count);
    let items = vec![
        "Check single URL (homepage)",
        sample_label.as_str(),
        full_label.as_str(),
    ];
    let selection = Select::new()
        .with_prompt("How would you like to proceed?")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| AuditError::ConfigError(e.to_string()))?;

    if selection == 1 {
        let mut batch_args = suggested_sitemap_batch_args(args, sitemap_url);
        batch_args.max_pages = 20;
        println!();
        return run_batch_mode(&batch_args).await.map(Some);
    }

    if selection == 2 {
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
        if let Some(count) = count_sitemap_entries_shallow(&candidate).await {
            return Ok(Some((candidate, count)));
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

    let client = build_browser_client(10).unwrap_or_default();
    let Ok(response) = client.get(robots_url.clone()).send().await else {
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

/// Check whether a URL is reachable before launching Chrome.
/// Only fails on network-level errors (DNS, timeout, connection refused).
/// Any HTTP response — including 4xx/5xx from bot-protection like Cloudflare —
/// is treated as "server reachable"; Chrome handles auth and bot challenges itself.
async fn check_url_reachable(url: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{} {}", "Checking:".dimmed(), url);
    }

    let client = build_browser_client(10).map_err(|e| AuditError::ConfigError(e.to_string()))?;

    // Any HTTP response = server is up; Chrome handles auth/bot-protection itself.
    // Connection-level errors (TLS reset by Cloudflare, refused) are silently ignored —
    // Chrome uses a different TLS stack and often succeeds where reqwest fails.
    // Only abort on timeout: that means the host is genuinely unreachable.
    match client.head(url).send().await {
        Ok(_) => {}
        Err(e) if e.is_timeout() => {
            return Err(AuditError::ConfigError(format!(
                "Domain unreachable (timeout): {}\n  Please check your internet connection and URL.",
                url
            )));
        }
        Err(e) => {
            tracing::debug!("Preflight HEAD failed ({}); proceeding with Chrome", e);
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
            "{} {} URLs with {} parallel workers\n",
            "Auditing:".cyan().bold(),
            total_urls,
            args.effective_concurrency()
        );
        print_batch_audit_plan(args, total_urls);
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
                "{} {} internal links checked, {} broken",
                "Link check:".cyan().bold(),
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
        println!("{} {} domains\n", "Comparing:".cyan().bold(), urls.len());
        print_comparison_audit_plan(args);
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
            println!("{} {}", "Auditing:".dimmed(), url);
        }
        match run_single_audit(url, &browser, &config).await {
            Ok(report) => reports.push(report),
            Err(e) => {
                if !args.quiet {
                    eprintln!("{} {} — {}", "Error:".red().bold(), url, e);
                }
                failed += 1;
            }
        }
    }

    let close_result = browser.close().await;
    close_result?;

    if reports.is_empty() {
        return Err(AuditError::ConfigError(
            "No domain audit succeeded.".to_string(),
        ));
    }

    let total_ms = start.elapsed().as_millis() as u64;
    let comparison = auditmysite::audit::ComparisonReport::from_reports(reports, total_ms);

    if !args.quiet {
        if failed > 0 {
            println!(
                "{} {} domain(s) could not be audited.",
                "Warning:".yellow().bold(),
                failed
            );
        }
        // Print summary table
        println!();
        println!("{}", "Ranking:".cyan().bold());
        for (rank, entry) in comparison.entries.iter().enumerate() {
            println!(
                "  {}. {} — Overall: {}/100 ({}), Accessibility: {}/100, {} Violations",
                rank + 1,
                entry.domain,
                entry.overall_score,
                entry.grade,
                entry.accessibility_score,
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

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use crate::output_paths::{
        default_batch_pdf_output_path, default_single_json_output_path,
        default_single_pdf_output_path, output_directory, per_page_output_directory,
        per_page_output_path, report_subject_from_url,
    };
    use crate::plan::{
        active_modules_label, planned_batch_outputs, planned_comparison_outputs,
        planned_single_outputs,
    };
    use auditmysite::cli::ReportLevel;
    use std::path::PathBuf;

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
        assert!(rendered.contains("example-com-"));
        assert!(rendered.contains("-sitemap-report.pdf"));
        assert!(rendered.ends_with(".pdf"));
        assert!(!rendered.contains("reports/"));
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_default_batch_pdf_output_path_uses_crawl_kind() {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.crawl = true;
        let rendered = default_batch_pdf_output_path(&args).display().to_string();
        assert!(rendered.contains("example-com-"));
        assert!(rendered.contains("-crawl-report.pdf"));
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

    #[test]
    fn test_planned_single_outputs_include_auto_json_for_default_pdf() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(outputs.len(), 2);
        assert!(outputs[0].ends_with("-single-report.pdf"));
        assert!(outputs[1].ends_with("-single-report.json"));
    }

    #[test]
    fn test_planned_single_outputs_use_stdout_without_output_file() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "-f", "json"]);
        let outputs = planned_single_outputs(&args, "https://example.com");

        assert_eq!(outputs, vec!["stdout".to_string()]);
    }

    #[test]
    fn test_planned_batch_outputs_include_json_sidecar_for_pdf() {
        let args = Args::parse_from([
            "auditmysite",
            "--sitemap",
            "https://example.com/sitemap.xml",
            "-f",
            "pdf",
        ]);
        let outputs = planned_batch_outputs(&args);

        assert_eq!(outputs.len(), 2);
        assert!(outputs[0].ends_with("-sitemap-report.pdf"));
        assert!(outputs[1].ends_with("-sitemap-report.json"));
    }

    #[test]
    fn test_planned_comparison_outputs_default_to_pdf_file() {
        let args = Args::parse_from([
            "auditmysite",
            "--compare",
            "https://alpha.example.com",
            "https://beta.example.com",
        ]);
        let outputs = planned_comparison_outputs(&args);

        assert_eq!(outputs, vec!["comparison-report.pdf".to_string()]);
    }

    #[test]
    fn test_planned_comparison_outputs_use_stdout_for_json_without_output() {
        let args = Args::parse_from([
            "auditmysite",
            "--compare",
            "https://alpha.example.com",
            "https://beta.example.com",
            "-f",
            "json",
        ]);
        let outputs = planned_comparison_outputs(&args);

        assert_eq!(outputs, vec!["stdout".to_string()]);
    }

    #[test]
    fn test_active_modules_label_respects_skip_flags() {
        let args = Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--skip-performance",
            "--skip-mobile",
        ]);
        let label = active_modules_label(&args);

        assert_eq!(label, "Accessibility");
    }
}
