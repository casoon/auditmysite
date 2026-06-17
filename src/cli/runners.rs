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
    analyze_crawl_links, analyze_sitemap_diagnostics, cache_matches_signature,
    compute_batch_verdict, compute_verdict, crawl_site, hydrate_cached_report, load_artifacts,
    parse_sitemap, read_url_file, run_concurrent_batch, run_single_audit, to_audit_report,
    BatchConfig, CrawlResult, PipelineConfig, Verdict,
};
use auditmysite::browser::{BrowserManager, BrowserOptions};
use auditmysite::cli::{Args, OutputFormat, RequestMode};
use auditmysite::error::{AuditError, Result};
use auditmysite::output::format_json_cached;
use auditmysite::util::truncate_url;

use crate::output_paths::output_text;
use crate::plan::{print_batch_audit_plan, print_single_audit_plan};
use crate::report_writers::{
    output_batch_as_single_reports, output_batch_report, output_screen_reader_sidecar,
    output_single_report,
};
use crate::sitemap_suggest::{
    check_url_reachable, discover_populated_sitemap, looks_like_base_url,
};

const BOT_USER_AGENT: &str = concat!(
    "auditmysite/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/casoon/auditmysite)"
);

pub async fn run_single_mode(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
) -> Result<Verdict> {
    let url = args
        .url
        .as_ref()
        .ok_or_else(|| AuditError::ConfigError("URL required".to_string()))?;

    if let Some(batch_verdict) = maybe_offer_sitemap_scan(args, url, config).await? {
        return Ok(batch_verdict);
    }

    print_single_audit_plan(args, url);

    // Quick reachability check before spinning up a browser
    check_url_reachable(url, args.quiet).await?;

    info!("Starting audit for: {}", url);

    if args.reuse_cache && !args.force_refresh {
        let expected_signature = PipelineConfig::from(args).audit_signature();
        match load_artifacts(url)? {
            Some(cached) if cache_matches_signature(&cached.meta, &expected_signature) => {
                if !args.quiet {
                    println!(
                        "{} {}",
                        "Cache hit:".green().bold(),
                        "using cached audit artifacts".dimmed()
                    );
                }

                let verdict_cfg = config
                    .as_ref()
                    .map(|c| c.effective_verdict_config())
                    .unwrap_or_default();
                // The verdict always derives from the stored NormalizedReport,
                // so it is identical regardless of --format (#404).
                let verdict_result = compute_verdict(&cached.audit, &verdict_cfg);

                // Prefer the full cached report so every module section renders
                // faithfully; fall back to the lossy reconstruction only for
                // legacy entries written before report.json existed (#404).
                let mut report = cached
                    .report
                    .clone()
                    .unwrap_or_else(|| to_audit_report(&cached, &args.lang));
                // screen_reader_audit is #[serde(skip)] and therefore absent from
                // the persisted report — rebuild it from the cached AXTree so the
                // cached report renders the same sections as a fresh run (#404).
                hydrate_cached_report(&mut report, &cached.snapshot, &args.lang);

                match args.effective_format() {
                    OutputFormat::Json => {
                        let output = format_json_cached(&cached.audit, true)?;
                        output_text(&output, &args.output, "JSON", args.quiet)?;
                        output_screen_reader_sidecar(&report, args)?;
                    }
                    OutputFormat::Table
                    | OutputFormat::Pdf
                    | OutputFormat::Ai
                    | OutputFormat::Summary => {
                        output_single_report(&report, args, Some(&verdict_result))?;
                    }
                }
                print_verdict(&verdict_result, args.quiet);
                return Ok(verdict_result.verdict);
            }
            Some(_) if !args.quiet => {
                println!(
                    "{} {}",
                    "Cache skipped:".yellow().bold(),
                    "cached artifacts were produced with a different audit configuration — \
                     running a fresh audit"
                        .dimmed()
                );
            }
            _ => {}
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
        user_agent_override: (args.request_mode == RequestMode::Bot)
            .then(|| BOT_USER_AGENT.to_string()),
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

    let normalized = normalize(&report).normalized;
    let verdict_cfg = config
        .as_ref()
        .map(|c| c.effective_verdict_config())
        .unwrap_or_default();
    let verdict_result = compute_verdict(&normalized, &verdict_cfg);
    output_single_report(&report, args, Some(&verdict_result))?;
    print_verdict(&verdict_result, args.quiet);
    Ok(verdict_result.verdict)
}

async fn maybe_offer_sitemap_scan(
    args: &Args,
    url: &str,
    config: &Option<auditmysite::cli::Config>,
) -> Result<Option<Verdict>> {
    if args.no_sitemap_suggest {
        return Ok(None);
    }
    if !looks_like_base_url(url) {
        return Ok(None);
    }

    if !args.quiet {
        print!("{} ", "Checking for sitemap...".dimmed());
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }

    let Some((sitemap_url, url_count)) = discover_populated_sitemap(url).await? else {
        if !args.quiet {
            println!();
        }
        return Ok(None);
    };

    if !args.quiet {
        println!();
    }

    if args.prefer_sitemap {
        let batch_args = suggested_sitemap_batch_args(args, sitemap_url);
        return run_batch_mode(&batch_args, config).await.map(Some);
    }

    if args.quiet {
        return Ok(None);
    }

    // dialoguer opens /dev/tty directly — only stdin needs to be a terminal
    if !io::stdin().is_terminal() {
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
        return run_batch_mode(&batch_args, config).await.map(Some);
    }

    if selection == 2 {
        let batch_args = suggested_sitemap_batch_args(args, sitemap_url);
        println!();
        return run_batch_mode(&batch_args, config).await.map(Some);
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

pub async fn run_batch_mode(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
) -> Result<Verdict> {
    let mut crawl_result: Option<CrawlResult> = None;

    let url_source: &str;
    let urls = if let Some(ref sitemap_url) = args.sitemap {
        url_source = "sitemap";
        if !args.quiet {
            println!("{} {}", "Fetching sitemap:".cyan().bold(), sitemap_url);
        }
        parse_sitemap(sitemap_url).await?
    } else if args.crawl {
        url_source = "crawl";
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
        url_source = "url_file";
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
        return Ok(Verdict::Warn);
    }

    let total_discovered = urls.len();
    let total_urls = if args.max_pages > 0 {
        args.max_pages.min(total_discovered)
    } else {
        total_discovered
    };

    let sample = auditmysite::audit::SampleMetadata {
        source: url_source.to_string(),
        total_discovered,
        audited: total_urls,
        sample_limit: (args.max_pages > 0).then_some(args.max_pages),
        selection: if total_urls < total_discovered {
            "first_n".to_string()
        } else {
            "all".to_string()
        },
        is_sample: total_urls < total_discovered,
    };

    let audited_urls: Vec<String> = urls.iter().take(total_urls).cloned().collect();

    if !args.quiet {
        if sample.is_sample {
            println!(
                "{} auditing {} of {} discovered URLs ({} order, first {})",
                "Sample:".yellow().bold(),
                total_urls,
                total_discovered,
                url_source,
                total_urls
            );
        }
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
    let progress: Option<Arc<dyn Fn(usize, usize, &str, Option<&str>) + Send + Sync>> =
        if let Some(ref pb) = progress_bar {
            let pb_clone = pb.clone();
            Some(Arc::new(move |current, _total, url, error| {
                pb_clone.set_position(current as u64);
                if let Some(err) = error {
                    pb_clone.println(format!("  ✗ {url}\n    {err}"));
                } else {
                    pb_clone.set_message(truncate_url(url, 50));
                }
            }))
        } else {
            None
        };

    let mut batch_report = run_concurrent_batch(urls, &batch_config, progress).await?;
    batch_report = batch_report.with_sample(sample);

    if url_source == "sitemap" {
        let diagnostics = analyze_sitemap_diagnostics(&audited_urls, &batch_report.reports).await;
        if !args.quiet {
            println!(
                "{} {} URLs checked, {} sitemap issues",
                "Sitemap check:".cyan().bold(),
                diagnostics.checked_urls,
                diagnostics.http_issues.len()
                    + diagnostics.orphan_sitemap_urls.len()
                    + diagnostics.linked_not_in_sitemap.len()
            );
        }
        batch_report = batch_report.with_sitemap_diagnostics(diagnostics);
    }

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

    let verdict_cfg = config
        .as_ref()
        .map(|c| c.effective_verdict_config())
        .unwrap_or_default();
    let verdict_result = compute_batch_verdict(&batch_report.summary, &verdict_cfg);

    if args.per_page_reports {
        output_batch_as_single_reports(&batch_report, args)?;
        print_verdict(&verdict_result, args.quiet);
        return Ok(verdict_result.verdict);
    }

    output_batch_report(&batch_report, args, Some(&verdict_result))?;
    print_verdict(&verdict_result, args.quiet);
    Ok(verdict_result.verdict)
}

fn print_verdict(vr: &auditmysite::VerdictResult, quiet: bool) {
    if quiet {
        return;
    }
    let label = match vr.verdict {
        auditmysite::Verdict::Pass => "PASS".green().bold(),
        auditmysite::Verdict::Warn => "WARN".yellow().bold(),
        auditmysite::Verdict::Fail => "FAIL".red().bold(),
    };
    if vr.reasons.is_empty() {
        println!("\n{}", label);
    } else {
        println!("\n{} — {}", label, vr.reasons.join(", ").dimmed());
    }
}
