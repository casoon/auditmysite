//! Audit mode runners.
//!
//! Implements the three top-level audit modes (single URL, batch, comparison)
//! plus the interactive sitemap-suggestion flow. Extracted from main.rs.

use std::io::{self, IsTerminal};
use std::sync::Arc;

use colored::Colorize;
use dialoguer::Select;
use tracing::info;

use auditmysite::audit::normalize_checked;
use auditmysite::audit::{
    analyze_crawl_links, analyze_sitemap_diagnostics_with_progress, cached_report, crawl_site,
    parse_sitemap, read_url_file, replay_report, run_concurrent_batch, run_single_audit,
    BatchConfig, CacheMiss, CrawlResult, ExpectedProfile, PipelineConfig, SnapshotArtifact,
    Verdict,
};
use auditmysite::browser::{BrowserManager, BrowserOptions};
use auditmysite::cli::url_filter::{select_urls, stratified_sample};
use auditmysite::cli::{Args, OutputFormat, RequestMode, SampleStrategy};
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
pub(crate) struct RunOutcome {
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

pub(crate) async fn run_single_mode(
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
        // The stored snapshot is replayed (#790), not reconstructed: an entry
        // that is incomplete, from other versions or for another audit
        // configuration is a miss that says why, never today's semantics
        // applied to old data.
        match cached_report(url, &expected_signature)? {
            Ok(report) => {
                if !args.quiet {
                    println!(
                        "{} {}",
                        "Cache hit:".green().bold(),
                        "replaying the stored snapshot".dimmed()
                    );
                }
                return emit_replayed(report, args, config);
            }
            Err(CacheMiss::Absent) => {}
            Err(reason) if !args.quiet => {
                println!(
                    "{} {}",
                    "Cache skipped:".yellow().bold(),
                    format!("{reason} — running a fresh audit").dimmed()
                );
            }
            Err(_) => {}
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
        // A single run starts a browser with a fresh profile: nothing else
        // shares it.
        session: args.session,
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

    // The decision stored by normalization under the run's profile (#781).
    let verdict_result = normalize_checked(&report)?.normalized.decision;
    output_single_report(&report, args)?;
    print_verdict(&verdict_result, args.quiet);
    Ok(RunOutcome {
        verdict: verdict_result.verdict,
        visual_only_urls: visual_only_urls(std::iter::once((
            report.url.as_str(),
            report.accessibility.execution.display_modes.as_ref(),
        ))),
    })
}

/// Decides and emits a report that came from a stored snapshot (cache hit or
/// `replay`): judged under this run's decision profile, which is stored with
/// the report, so every output and the exit code read the same decision
/// (#781).
fn emit_replayed(
    mut report: auditmysite::AuditReport,
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
) -> Result<RunOutcome> {
    report.run_context.decision_profile = config
        .as_ref()
        .map(|c| c.effective_verdict_config().profile())
        .unwrap_or_default();
    let verdict_result = normalize_checked(&report)?.normalized.decision;
    output_single_report(&report, args)?;
    if args.effective_format() == OutputFormat::Json {
        output_screen_reader_sidecar(&report, args)?;
    }
    print_verdict(&verdict_result, args.quiet);
    Ok(RunOutcome {
        verdict: verdict_result.verdict,
        visual_only_urls: visual_only_urls(std::iter::once((
            report.url.as_str(),
            report.accessibility.execution.display_modes.as_ref(),
        ))),
    })
}

/// `auditmysite replay <target>`: the stored snapshot, evaluated again
/// without browser or network. `session_from_cli` says whether `--session`
/// was given: its default is not an assertion about the snapshot.
pub(crate) fn run_replay_mode(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
    target: &str,
    session_from_cli: bool,
) -> Result<Verdict> {
    let path = std::path::Path::new(target);
    let snapshot = if path.exists() {
        SnapshotArtifact::read(path)?
    } else if url::Url::parse(target).is_ok() {
        SnapshotArtifact::read_cached(target)?
    } else {
        return Err(AuditError::ConfigError(format!(
            "replay target '{target}' is neither an existing snapshot/cache directory nor a URL"
        )));
    };
    let expected = ExpectedProfile {
        session_policy: session_from_cli.then(|| args.session.as_str().to_string()),
        state_id: args.state_id.clone(),
        storage_state_sha256: args
            .load_storage_state()
            .map_err(AuditError::ConfigError)?
            .map(|state| state.digest),
        // Replay refuses extra-check flags (support matrix), so nothing to assert.
        extra_checks: None,
    };
    let refuse = |e: auditmysite::audit::ReplayError| AuditError::ConfigError(e.to_string());
    snapshot.check_measurement(&expected).map_err(refuse)?;
    let report = replay_report(&snapshot).map_err(refuse)?;
    if !args.quiet {
        println!(
            "{} {}",
            "Replay:".cyan().bold(),
            format!("{} (no browser, no network)", report.url).dimmed()
        );
    }
    emit_replayed(report, args, config).map(|o| o.verdict)
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

pub(crate) fn suggested_sitemap_batch_args(args: &Args, sitemap_url: String) -> Args {
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

pub(crate) async fn run_batch_mode(
    args: &Args,
    config: &Option<auditmysite::cli::Config>,
) -> Result<Verdict> {
    run_batch(args, config, None).await.map(|o| o.verdict)
}

/// `urls_override`: audit exactly these URLs instead of the configured
/// source — the `visual` pass of `--display all` (#653).
async fn run_batch(
    args: &Args,
    // The verdict profile reaches the reports through `PipelineConfig`.
    _config: &Option<auditmysite::cli::Config>,
    urls_override: Option<Vec<String>>,
) -> Result<RunOutcome> {
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

    let stratified = args.sample_strategy == SampleStrategy::Stratified;
    let (urls, draws) = if stratified {
        stratified_sample(urls, args.max_pages)
    } else {
        (urls, Vec::new())
    };
    let sampled_by_strata = !draws.is_empty();

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
        selection: if sampled_by_strata {
            "stratified".to_string()
        } else if path_filtered {
            "path_filter".to_string()
        } else if total_urls < total_discovered {
            "first_n".to_string()
        } else {
            "all".to_string()
        },
        is_sample: total_urls < total_discovered,
        strata: draws
            .into_iter()
            .map(|d| auditmysite::audit::SampleStratum {
                key: d.key,
                discovered: d.discovered,
                selected: d.selected,
            })
            .collect(),
    };

    // The pages actually attempted, in input order — the technician index
    // lists every one of them, audited or not (plan 67).
    let attempted_urls: Vec<String> = urls.iter().take(total_urls).cloned().collect();

    if !args.quiet {
        if sample.is_sample {
            eprintln!(
                "{} auditing {} of {} discovered URLs ({})",
                "Sample:".yellow().bold(),
                total_urls,
                total_discovered,
                if sampled_by_strata {
                    format!(
                        "{url_source}, stratified over {} groups",
                        sample.strata.len()
                    )
                } else {
                    format!("{url_source} order, first {total_urls}")
                }
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
        // The audited pages are done, but the sitemap check still requests
        // every sitemap URL — a second stage on the same bar, so 20/20 does
        // not read as finished while it runs.
        presenter.start(full_sitemap_urls.len(), "Checking sitemap URLs");
        let sitemap_progress: auditmysite::audit::ProgressCallback = {
            let presenter = Arc::clone(&presenter);
            Arc::new(move |current, _total, url: &str, _error: Option<&str>| {
                presenter.advance(current, url);
            })
        };
        let diagnostics = analyze_sitemap_diagnostics_with_progress(
            &full_sitemap_urls,
            &batch_report.reports,
            Some(sitemap_progress),
        )
        .await;
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

    let verdict_result = batch_report.summary.decision.clone();
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
        output_batch_report(&batch_report, args)
    })?;
    Ok(outcome)
}

/// `--display all` (#653): audit each display mode as its own run and write
/// one report per mode — never a blended score. `calm` and `text` audit every
/// page; `visual` only the pages the `calm` run found with 3D or interactive
/// visualisations, with twice the page timeout.
pub(crate) async fn run_display_all(
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
