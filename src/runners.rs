//! Audit mode runners.
//!
//! Implements the three top-level audit modes (single URL, batch, comparison)
//! plus the interactive sitemap-suggestion flow. Extracted from main.rs.

use std::io::{self, IsTerminal};
use std::sync::Arc;

use colored::Colorize;
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::info;

use auditmysite::audit::normalize;
use auditmysite::audit::{
    analyze_crawl_links, crawl_site, load_artifacts, parse_sitemap, read_url_file,
    run_concurrent_batch, run_single_audit, to_audit_report, BatchConfig, CrawlResult,
    PipelineConfig,
};
use auditmysite::browser::{BrowserManager, BrowserOptions};
use auditmysite::cli::{Args, OutputFormat};
use auditmysite::error::{AuditError, Result};
use auditmysite::output::format_json_cached;
use auditmysite::util::truncate_url;

use crate::output_paths::output_text;
use crate::plan::{print_batch_audit_plan, print_comparison_audit_plan, print_single_audit_plan};
use crate::report_writers::{
    output_batch_as_single_reports, output_batch_report, output_comparison_report,
    output_single_report,
};
use crate::sitemap_suggest::{
    check_url_reachable, discover_populated_sitemap, looks_like_base_url,
};

pub async fn run_single_mode(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
) -> Result<f64> {
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

pub fn suggested_sitemap_batch_args(args: &Args, sitemap_url: String) -> Args {
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

pub async fn run_batch_mode(args: &Args) -> Result<f64> {
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

pub async fn run_compare_mode(args: &Args) -> Result<f64> {
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
