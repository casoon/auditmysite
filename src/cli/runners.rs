//! Audit mode runners.
//!
//! Implements the three top-level audit modes (single URL, batch, comparison)
//! plus the interactive sitemap-suggestion flow. Extracted from main.rs.

use std::io::{self, IsTerminal};
use std::sync::Arc;

use colored::Colorize;
use dialoguer::Select;
use tracing::info;

use auditmysite::audit::normalize;
use auditmysite::audit::{
    analyze_crawl_links, analyze_sitemap_diagnostics, cache_matches_signature,
    compute_batch_verdict, compute_verdict, crawl_site, hydrate_cached_report, load_artifacts,
    parse_sitemap, read_url_file, run_concurrent_batch, run_single_audit, to_audit_report,
    BatchConfig, CrawlResult, PipelineConfig, Verdict,
};
use auditmysite::browser::{BrowserManager, BrowserOptions};
use auditmysite::cli::url_filter::select_urls;
use auditmysite::cli::{Args, OutputFormat, RequestMode};
use auditmysite::error::{AuditError, Result};

use crate::batch_lifecycle::BatchLifecyclePresenter;
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

/// Fail loudly, before the pipeline runs, if `--ai-transparency` was passed
/// but the binary wasn't built with the `ai-transparency` Cargo feature.
///
/// Without this check the flag would silently no-op: `PipelineConfig.
/// check_ai_transparency` would still be set, but `AiTransparencyModule::
/// collect()`'s `#[cfg(not(feature = "ai-transparency"))]` fallback just
/// returns `ModuleData::None` — indistinguishable from "nothing found".
/// Mirrors `report_writers.rs`'s PDF-feature error (same message shape).
fn check_ai_transparency_feature(args: &Args) -> Result<()> {
    if args.ai_transparency && !cfg!(feature = "ai-transparency") {
        return Err(AuditError::ConfigError(
            "AI transparency check requires the 'ai-transparency' feature. Rebuild with: cargo build --features ai-transparency".to_string(),
        ));
    }
    Ok(())
}

/// What one audit run leaves behind for `--display all` (#653): its verdict
/// and the audited URLs that only the `visual` mode can fully audit.
pub struct RunOutcome {
    pub verdict: Verdict,
    pub visual_only_urls: Vec<String>,
}

impl RunOutcome {
    fn verdict_only(verdict: Verdict) -> Self {
        Self {
            verdict,
            visual_only_urls: Vec::new(),
        }
    }
}

/// Audited URLs whose page carries `figure[data-viz="3d|interactive"]`.
fn visual_only_urls<'a>(
    pages: impl Iterator<Item = (&'a str, Option<&'a auditmysite::display::DisplayModesInfo>)>,
) -> Vec<String> {
    pages
        .filter(|(_, info)| info.is_some_and(|i| i.has_visual_only_content()))
        .map(|(url, _)| url.to_string())
        .collect()
}

pub async fn run_single_mode(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
) -> Result<Verdict> {
    run_single(args, config).await.map(|o| o.verdict)
}

async fn run_single(args: &Args, config: &Option<auditmysite::cli::Config>) -> Result<RunOutcome> {
    check_ai_transparency_feature(args)?;

    let url = args
        .url
        .as_ref()
        .ok_or_else(|| AuditError::ConfigError("URL required".to_string()))?;

    if let Some(batch_verdict) = maybe_offer_sitemap_scan(args, url, config).await? {
        return Ok(RunOutcome::verdict_only(batch_verdict));
    }

    print_single_audit_plan(args, url);

    // Quick reachability check before spinning up a browser
    check_url_reachable(url, args.effective_timeout(), args.quiet).await?;

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
                        // Build the JSON from the hydrated full report (not the
                        // normalized-only `cached.audit`) so `detail.modules`
                        // carries the same module data the PDF renders (#404) —
                        // a cache hit must not drop the module blob.
                        output_single_report(&report, args, Some(&verdict_result))?;
                        output_screen_reader_sidecar(&report, args)?;
                    }
                    OutputFormat::Table
                    | OutputFormat::Pdf
                    | OutputFormat::Ai
                    | OutputFormat::Summary
                    | OutputFormat::Sarif => {
                        output_single_report(&report, args, Some(&verdict_result))?;
                    }
                }
                print_verdict(&verdict_result, args.quiet);
                return Ok(RunOutcome {
                    verdict: verdict_result.verdict,
                    visual_only_urls: visual_only_urls(std::iter::once((
                        report.url.as_str(),
                        report.accessibility.execution.display_modes.as_ref(),
                    ))),
                });
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
        // One page, the active tab (see `BrowserManager::new_page`).
        focus_emulation: false,
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
            report.experience.budget_violations =
                auditmysite::audit::evaluate_budgets(&report, &cfg.budgets);
            if !report.experience.budget_violations.is_empty() && !args.quiet {
                use auditmysite::audit::BudgetSeverity;
                let errors = report
                    .experience
                    .budget_violations
                    .iter()
                    .filter(|v| v.severity == BudgetSeverity::Error)
                    .count();
                let warnings = report
                    .experience
                    .budget_violations
                    .iter()
                    .filter(|v| v.severity == BudgetSeverity::Warning)
                    .count();
                println!(
                    "{} {}{}: {} Error{}, {} Warning{}",
                    "Budget:".yellow().bold(),
                    report.experience.budget_violations.len(),
                    if report.experience.budget_violations.len() == 1 {
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
    Ok(RunOutcome {
        verdict: verdict_result.verdict,
        visual_only_urls: visual_only_urls(std::iter::once((
            report.url.as_str(),
            report.accessibility.execution.display_modes.as_ref(),
        ))),
    })
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
        if let Err(e) = std::io::Write::flush(&mut std::io::stdout()) {
            if io::stdout().is_terminal() {
                tracing::debug!("Failed to flush sitemap status line: {}", e);
            }
        }
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
    run_batch(args, config, None).await.map(|o| o.verdict)
}

/// `urls_override`: audit exactly these URLs instead of the configured
/// source — the `visual` pass of `--display all` (#653).
async fn run_batch(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
    urls_override: Option<Vec<String>>,
) -> Result<RunOutcome> {
    check_ai_transparency_feature(args)?;
    if args.ai_transparency && !args.quiet {
        eprintln!(
            "{} --ai-transparency is single-URL-only and has no effect on this batch run.",
            "Note:".yellow().bold()
        );
    }

    let mut crawl_result: Option<CrawlResult> = None;

    let url_source: &str;
    let urls = if let Some(urls) = urls_override {
        url_source = "display_visual";
        urls
    } else if let Some(ref sitemap_url) = args.sitemap {
        url_source = "sitemap";
        if !args.quiet {
            eprintln!("{} {}", "Fetching sitemap:".cyan().bold(), sitemap_url);
        }
        parse_sitemap(sitemap_url).await?
    } else if args.crawl {
        url_source = "crawl";
        let seed_url = args
            .url
            .as_deref()
            .ok_or_else(|| AuditError::ConfigError("No crawl seed URL specified".to_string()))?;
        if !args.quiet {
            eprintln!("{} {}", "Crawling site:".cyan().bold(), seed_url);
        }
        let crawl = crawl_site(seed_url, args.max_pages, args.crawl_depth).await?;
        if !args.quiet {
            eprintln!(
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
            eprintln!(
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
            eprintln!("{} No URLs found to audit.", "Warning:".yellow().bold());
        }
        return Ok(RunOutcome::verdict_only(Verdict::Warn));
    }

    let total_discovered = urls.len();

    // Cloned before path filtering and before `urls` is moved into
    // `run_concurrent_batch` below. Must stay the *full* discovered list, not
    // just the audited selection: sitemap diagnostics compares it against
    // crawled internal links, and a subset floods `linked_not_in_sitemap` with
    // pages that are genuinely in the sitemap, just outside the selection (#514).
    let full_sitemap_urls: Vec<String> = if url_source == "sitemap" {
        urls.clone()
    } else {
        Vec::new()
    };

    let path_filtered = !(args.include_path.is_empty() && args.exclude_path.is_empty());
    let urls = select_urls(urls, &args.include_path, &args.exclude_path);
    if path_filtered {
        if !args.quiet {
            eprintln!(
                "{} {} of {} URLs match the path filters",
                "Path filter:".cyan().bold(),
                urls.len(),
                total_discovered
            );
        }
        if urls.is_empty() {
            if !args.quiet {
                eprintln!(
                    "{} No URL matches --include-path/--exclude-path.",
                    "Warning:".yellow().bold()
                );
            }
            return Ok(RunOutcome::verdict_only(Verdict::Warn));
        }
    }

    let total_urls = if args.max_pages > 0 {
        args.max_pages.min(urls.len())
    } else {
        urls.len()
    };

    let sample = auditmysite::audit::SampleMetadata {
        source: url_source.to_string(),
        total_discovered,
        audited: total_urls,
        sample_limit: (args.max_pages > 0).then_some(args.max_pages),
        selection: if path_filtered {
            "path_filter".to_string()
        } else if total_urls < total_discovered {
            "first_n".to_string()
        } else {
            "all".to_string()
        },
        is_sample: total_urls < total_discovered,
    };

    // The pages actually attempted, in input order — the technician index
    // lists every one of them, audited or not (plan 67).
    let attempted_urls: Vec<String> = urls.iter().take(total_urls).cloned().collect();

    if !args.quiet {
        if sample.is_sample {
            eprintln!(
                "{} auditing {} of {} discovered URLs ({} order, first {})",
                "Sample:".yellow().bold(),
                total_urls,
                total_discovered,
                url_source,
                total_urls
            );
        }
        eprintln!(
            "{} {} URLs with {} parallel workers\n",
            "Auditing:".cyan().bold(),
            total_urls,
            args.effective_concurrency()
        );
        if args.concurrency.is_none() && args.effective_concurrency() == 1 {
            eprintln!(
                "{} one page at a time: keyboard journeys on parallel pages can stall a full \
                 browser on macOS. --concurrency overrides.\n",
                "Note:".yellow().bold()
            );
        }
        print_batch_audit_plan(args, total_urls);
    }

    let batch_config = BatchConfig::from(args);

    let console = runemark::Console::stderr(runemark::ColorMode::from(args.color));
    let presenter = Arc::new(BatchLifecyclePresenter::new(
        args.progress,
        args.quiet,
        console,
        io::stderr().is_terminal(),
    ));
    presenter.start(total_urls, "Auditing URLs");

    let progress: Option<auditmysite::audit::ProgressCallback> = {
        let presenter = Arc::clone(&presenter);
        Some(Arc::new(
            move |current, _total, url: &str, error: Option<&str>| {
                if let Some(err) = error {
                    presenter.notice_error(url, err);
                }
                presenter.advance(current, url);
            },
        ))
    };

    let mut batch_report = run_concurrent_batch(urls, &batch_config, progress).await?;
    batch_report = batch_report.with_sample(sample);

    if url_source == "sitemap" {
        let diagnostics =
            analyze_sitemap_diagnostics(&full_sitemap_urls, &batch_report.reports).await;
        presenter.notice(&format!(
            "Sitemap check: {} URLs checked, {} sitemap issues",
            diagnostics.checked_urls,
            diagnostics.http_issues.len()
                + diagnostics.orphan_sitemap_urls.len()
                + diagnostics.linked_not_in_sitemap.len()
        ));
        batch_report = batch_report.with_sitemap_diagnostics(diagnostics);
    }

    if let Some(ref crawl) = crawl_result {
        let diagnostics = analyze_crawl_links(crawl).await;
        presenter.notice(&format!(
            "Link check: {} internal links checked, {} broken",
            diagnostics.checked_internal_links,
            diagnostics.broken_internal_links.len()
        ));
        batch_report = batch_report.with_crawl_diagnostics(diagnostics);
    }

    presenter.notice(&format!(
        "Results: {}/{} passed, {} violations found in {}ms",
        batch_report.summary.passed,
        batch_report.summary.total_urls,
        batch_report.summary.total_violations,
        batch_report.total_duration_ms
    ));

    let verdict_cfg = config
        .as_ref()
        .map(|c| c.effective_verdict_config())
        .unwrap_or_default();
    let verdict_result = compute_batch_verdict(&batch_report.summary, &verdict_cfg);
    let verdict_message = verdict_message_text(&verdict_result);

    let outcome = RunOutcome {
        verdict: verdict_result.verdict,
        visual_only_urls: visual_only_urls(batch_report.reports.iter().map(|r| {
            (
                r.url.as_str(),
                r.accessibility.execution.display_modes.as_ref(),
            )
        })),
    };

    if args.per_page_reports {
        presenter.finish_then_render(verdict_result.verdict, &verdict_message, || {
            output_batch_as_single_reports(&batch_report, args, &attempted_urls)
        })?;
        return Ok(outcome);
    }

    presenter.finish_then_render(verdict_result.verdict, &verdict_message, || {
        output_batch_report(&batch_report, args, Some(&verdict_result))
    })?;
    Ok(outcome)
}

/// `--display all` (#653): audit each display mode as its own run and write
/// one report per mode — never a blended score. `calm` and `text` audit every
/// page; `visual` only the pages the `calm` run found with 3D or interactive
/// visualisations, with twice the page timeout.
pub async fn run_display_all(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
    is_batch: bool,
) -> Result<Verdict> {
    use auditmysite::cli::DisplaySelection;

    let base_output = args.output.clone().or_else(|| {
        (args.effective_format() == OutputFormat::Pdf).then(|| {
            if is_batch {
                crate::output_paths::default_batch_pdf_output_path(args)
            } else {
                crate::output_paths::default_single_pdf_output_path(
                    args.url.as_deref().unwrap_or(""),
                    args.report_level,
                )
            }
        })
    });
    let mode_args = |selection: DisplaySelection| {
        let mut a = args.clone();
        a.display = Some(selection);
        let suffix = selection.mode().map_or("all", |m| m.as_str());
        a.output = base_output
            .as_deref()
            .map(|p| crate::output_paths::with_display_suffix(p, suffix));
        // One URL stays one URL across all modes.
        a.no_sitemap_suggest = true;
        a.prefer_sitemap = false;
        a
    };
    let announce = |selection: DisplaySelection| {
        if !args.quiet {
            eprintln!(
                "\n{} {}",
                "Display mode:".cyan().bold(),
                selection.mode().map_or("all", |m| m.as_str())
            );
        }
    };

    let mut verdicts = Vec::new();
    announce(DisplaySelection::Calm);
    let calm_args = mode_args(DisplaySelection::Calm);
    let calm = if is_batch {
        run_batch(&calm_args, config, None).await?
    } else {
        run_single(&calm_args, config).await?
    };
    verdicts.push(calm.verdict);

    announce(DisplaySelection::Text);
    let text_args = mode_args(DisplaySelection::Text);
    let text = if is_batch {
        run_batch(&text_args, config, None).await?
    } else {
        run_single(&text_args, config).await?
    };
    verdicts.push(text.verdict);

    if calm.visual_only_urls.is_empty() {
        if !args.quiet {
            eprintln!(
                "\n{} no page has figure[data-viz=\"3d\"|\"interactive\"]; the visual pass is skipped.",
                "Display mode visual:".cyan().bold()
            );
        }
    } else {
        announce(DisplaySelection::Visual);
        let mut visual_args = mode_args(DisplaySelection::Visual);
        visual_args.timeout = Some(args.effective_timeout() * 2);
        let visual = if is_batch {
            run_batch(&visual_args, config, Some(calm.visual_only_urls)).await?
        } else {
            run_single(&visual_args, config).await?
        };
        verdicts.push(visual.verdict);
    }

    Ok(worst_verdict(&verdicts))
}

fn worst_verdict(verdicts: &[Verdict]) -> Verdict {
    if verdicts.contains(&Verdict::Fail) {
        Verdict::Fail
    } else if verdicts.contains(&Verdict::Warn) {
        Verdict::Warn
    } else {
        Verdict::Pass
    }
}

fn verdict_message_text(vr: &auditmysite::VerdictResult) -> String {
    let label = match vr.verdict {
        auditmysite::Verdict::Pass => "PASS",
        auditmysite::Verdict::Warn => "WARN",
        auditmysite::Verdict::Fail => "FAIL",
    };
    if vr.reasons.is_empty() {
        label.to_string()
    } else {
        format!("{label} — {}", vr.reasons.join(", "))
    }
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
        eprintln!("\n{}", label);
    } else {
        eprintln!("\n{} — {}", label, vr.reasons.join(", ").dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn args_with_ai_transparency(flag: bool) -> Args {
        let mut args = Args::parse_from(["auditmysite", "https://example.com"]);
        args.ai_transparency = flag;
        args
    }

    #[test]
    fn flag_off_never_errors() {
        assert!(check_ai_transparency_feature(&args_with_ai_transparency(false)).is_ok());
    }

    #[test]
    fn flag_on_matches_compiled_feature_state() {
        let result = check_ai_transparency_feature(&args_with_ai_transparency(true));
        if cfg!(feature = "ai-transparency") {
            assert!(
                result.is_ok(),
                "feature is compiled in, flag must be accepted"
            );
        } else {
            let err = result.expect_err("feature is not compiled in, flag must be rejected");
            let msg = err.to_string();
            assert!(
                msg.contains("ai-transparency"),
                "error must name the feature: {msg}"
            );
            assert!(
                msg.contains("cargo build --features ai-transparency"),
                "error must include the rebuild instruction: {msg}"
            );
        }
    }
}
