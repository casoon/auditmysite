//! Batch Processing - Concurrent URL auditing with sitemap support
//!
//! Provides efficient batch processing of multiple URLs with:
//! - Concurrent execution using browser pool
//! - Sitemap XML parsing
//! - URL file processing
//! - Progress reporting

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::{redirect::Policy, Client};
use tracing::{info, warn};
use url::Url;

use super::pipeline::{audit_page, PipelineConfig};
use super::report::{
    AuditReport, BatchError, BatchReport, CrawlDepthDiagnostics, CrawlDepthEntry,
    RobotsSitemapConflict, SitemapDiagnostics, SitemapHttpIssue,
};
use crate::browser::{BrowserOptions, BrowserPool, PoolConfig, PAGE_RETURN_BUDGET_SECS};
use crate::cli::{Args, RequestMode};
use crate::error::{AuditError, Result};
use crate::seo::{BotClass, RobotsAudit};
use crate::util::build_browser_client;

/// Batch audit configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Pipeline configuration
    pub pipeline: PipelineConfig,
    /// Maximum number of concurrent pages
    pub concurrency: usize,
    /// Maximum number of URLs to process (0 = unlimited)
    pub max_urls: usize,
    /// Pool configuration
    pub pool_config: PoolConfig,
}

impl From<&Args> for BatchConfig {
    fn from(args: &Args) -> Self {
        let pool_config = PoolConfig {
            max_pages: args.effective_concurrency(),
            browser_options: BrowserOptions {
                // Parallel pages share one window; only then are tabs hidden.
                focus_emulation: args.effective_concurrency() > 1,
                chrome_path: args.chrome_path.clone(),
                no_sandbox: args.no_sandbox,
                disable_images: args.disable_images,
                timeout_secs: args.effective_timeout(),
                verbose: args.verbose,
                user_agent_override: (args.request_mode == RequestMode::Bot).then(|| {
                    concat!(
                        "auditmysite/",
                        env!("CARGO_PKG_VERSION"),
                        " (+https://github.com/casoon/auditmysite)"
                    )
                    .to_string()
                }),
                ..BrowserOptions::default()
            },
            acquire_timeout_secs: pool_acquire_timeout_secs(args.effective_timeout()),
        };

        Self {
            pipeline: PipelineConfig::from(args),
            concurrency: args.effective_concurrency(),
            max_urls: args.max_pages,
            pool_config,
        }
    }
}

/// Error from a single URL audit within a batch.
///
/// Preserves the structured `AuditError` produced by the pipeline instead of
/// erasing it to a string. `Other` only exists for the defensive (effectively
/// unreachable — see `audit_url_with_pool`) case where the retry loop runs out
/// of attempts without ever capturing an `AuditError`.
#[derive(Debug)]
pub enum BatchAuditError {
    Audit(AuditError),
    Other(String),
}

impl std::fmt::Display for BatchAuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchAuditError::Audit(e) => write!(f, "{e}"),
            BatchAuditError::Other(s) => write!(f, "{s}"),
        }
    }
}

/// Result of a single URL audit within a batch
#[derive(Debug)]
pub struct BatchResult {
    /// The URL that was audited
    pub url: String,
    /// The audit outcome (success or error)
    pub outcome: std::result::Result<AuditReport, BatchAuditError>,
}

/// Progress callback type: (current, total, url, error_message)
/// error_message is Some if the URL failed, None on success.
pub type ProgressCallback = Arc<dyn Fn(usize, usize, &str, Option<&str>) + Send + Sync>;

/// Run concurrent batch audit on multiple URLs
///
/// # Arguments
/// * `urls` - URLs to audit
/// * `config` - Batch configuration
/// * `progress` - Optional progress callback (current, total, url)
///
/// # Returns
/// * `Ok(BatchReport)` - Batch audit results
/// * `Err(AuditError)` - If batch processing fails completely
pub async fn run_concurrent_batch(
    urls: Vec<String>,
    config: &BatchConfig,
    progress: Option<ProgressCallback>,
) -> Result<BatchReport> {
    let start_time = Instant::now();
    let total_urls = if config.max_urls > 0 {
        config.max_urls.min(urls.len())
    } else {
        urls.len()
    };

    info!(
        "Starting batch audit of {} URLs with {} concurrent workers",
        total_urls, config.concurrency
    );

    // Create browser pool
    let pool = Arc::new(BrowserPool::new(config.pool_config.clone()).await?);
    let pipeline_config = Arc::new(config.pipeline.clone());

    let completed = Arc::new(AtomicUsize::new(0));

    // Bounded work queue: at most `concurrency` futures in flight at any time.
    // No unbounded spawn — tasks are only created as slots free up.
    let mut in_flight: FuturesUnordered<_> = FuturesUnordered::new();
    // Indexed so the report lists pages in input (sitemap) order, not in the
    // order they happened to finish (plan 60).
    let mut url_iter = urls.into_iter().take(total_urls).enumerate();

    let make_task = |index: usize,
                     url: String,
                     pool: Arc<BrowserPool>,
                     config: Arc<PipelineConfig>,
                     completed: Arc<AtomicUsize>,
                     progress: Option<ProgressCallback>,
                     total: usize| {
        async move {
            let result = audit_url_with_pool(&pool, &url, &config).await;
            if is_pool_timeout(&result) {
                // Not counted as done yet: the page gets its serial retry
                // after the parallel phase (#651).
                warn!(
                    "Deferred: {} - no browser page free in time, retrying after the parallel phase",
                    url
                );
            } else {
                report_progress(&completed, total, &result, progress.as_ref());
            }
            (index, result)
        }
    };

    // Fill up to concurrency limit before starting the drain loop
    for (index, url) in url_iter.by_ref().take(config.concurrency) {
        in_flight.push(make_task(
            index,
            url,
            Arc::clone(&pool),
            Arc::clone(&pipeline_config),
            Arc::clone(&completed),
            progress.clone(),
            total_urls,
        ));
    }

    // Collect results, feeding new work in as slots free up
    let mut reports = Vec::with_capacity(total_urls);
    let mut errors = Vec::new();

    let mut deferred = Vec::new();

    while let Some((index, batch_result)) = in_flight.next().await {
        if is_pool_timeout(&batch_result) {
            deferred.push((index, batch_result.url));
        } else {
            match batch_result.outcome {
                Ok(report) => reports.push((index, report)),
                Err(e) => errors.push((index, batch_result.url, e)),
            }
        }
        if let Some((index, url)) = url_iter.next() {
            in_flight.push(make_task(
                index,
                url,
                Arc::clone(&pool),
                Arc::clone(&pipeline_config),
                Arc::clone(&completed),
                progress.clone(),
                total_urls,
            ));
        }
    }

    // A pool timeout means the pool had no free page in time — a capacity
    // problem of the parallel phase, not a problem of the page. Retry those
    // pages one at a time, with every other page finished, before the
    // report is built (#651).
    if !deferred.is_empty() {
        warn!(
            "Retrying {} page(s) one at a time after browser pool timeouts",
            deferred.len()
        );
    }
    for (index, url) in deferred {
        let result = audit_url_with_pool(&pool, &url, &pipeline_config).await;
        report_progress(&completed, total_urls, &result, progress.as_ref());
        match result.outcome {
            Ok(report) => reports.push((index, report)),
            Err(e) => errors.push((index, result.url, e)),
        }
    }

    // Close the pool gracefully — mirrors what single-URL mode already does
    // with `browser.close().await` (src/cli/runners.rs). Previously this Arc
    // was just left to drop implicitly, relying on chromiumoxide's
    // `Browser::drop` "kill_on_drop" to reap the Chrome child process via a
    // task scheduled on the Tokio runtime in the background -- but `main()`
    // calls `std::process::exit(exit_code)` immediately after this function
    // returns, tearing down the whole process (and its runtime) before that
    // background task gets a chance to run. Confirmed live (2026-09-01):
    // every batch run left an orphaned `auditmysite-chrome-*` process
    // running indefinitely, even on a normal, successful exit. By this
    // point every spawned task's clone of `pool` has already completed and
    // dropped (the `while` loop above only returns once `in_flight` is
    // fully drained), so this should be the sole remaining reference.
    match Arc::try_unwrap(pool) {
        Ok(pool) => {
            if let Err(e) = pool.close().await {
                warn!("Failed to close browser pool: {}", e);
            }
        }
        Err(_) => {
            warn!(
                "Browser pool still had outstanding references at batch end; \
                 the browser process may not shut down cleanly"
            );
        }
    }

    let total_duration_ms = start_time.elapsed().as_millis() as u64;

    info!(
        "Batch audit completed: {}/{} successful, {} failed in {}ms",
        reports.len(),
        total_urls,
        errors.len(),
        total_duration_ms
    );

    reports.sort_by_key(|(index, _)| *index);
    errors.sort_by_key(|(index, _, _)| *index);
    let reports = reports.into_iter().map(|(_, report)| report).collect();
    let batch_errors = errors
        .into_iter()
        .map(|(_, url, error)| BatchError {
            url,
            error: error.to_string(),
        })
        .collect();

    Ok(BatchReport::from_reports(
        reports,
        batch_errors,
        total_duration_ms,
    ))
}

/// Validate sitemap entries that were selected for a sitemap-driven batch.
///
/// The checks intentionally stay sitemap-specific: a sitemap should list only
/// canonical, indexable 200 URLs. Link graph comparisons use the audited pages'
/// collected internal targets, so no extra crawl is required for the orphan
/// signal.
///
/// `sitemap_urls` must be every URL the sitemap lists, not just the ones
/// actually audited this run (see #514) — when `--max-pages` samples a
/// subset, comparing crawled internal links against that sample instead of
/// the full sitemap floods `linked_not_in_sitemap` with pages that are
/// genuinely in the sitemap, just outside the sample. `reports` stays
/// audited-only: that's the only source of link-graph evidence.
pub async fn analyze_sitemap_diagnostics(
    sitemap_urls: &[String],
    reports: &[AuditReport],
) -> SitemapDiagnostics {
    let client = Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    let http_issues: Vec<SitemapHttpIssue> = futures::stream::iter(sitemap_urls.iter())
        .map(|url| check_sitemap_url(&client, url))
        .buffer_unordered(8)
        .filter_map(|issue| async move { issue })
        .collect()
        .await;

    let sitemap_set: HashSet<String> = sitemap_urls
        .iter()
        .filter_map(|u| normalize_url(u))
        .collect();
    let audited_set: HashSet<String> = reports
        .iter()
        .filter_map(|r| normalize_url(&r.url))
        .collect();
    let linked_set = collect_internal_link_targets(reports);

    let mut orphan_sitemap_urls: Vec<String> = sitemap_set
        .iter()
        .filter(|url| audited_set.contains(*url) && !linked_set.contains(*url))
        .cloned()
        .collect();
    orphan_sitemap_urls.sort();

    let mut linked_not_in_sitemap: Vec<String> =
        linked_set.difference(&sitemap_set).cloned().collect();
    linked_not_in_sitemap.sort();

    // robots.txt is domain-wide, not per-page — every audited page carries an
    // identical fetch, so any one of them is a valid source (#549).
    let robots_conflicts = reports
        .iter()
        .find_map(|r| {
            r.discoverability
                .seo
                .as_ref()
                .and_then(|s| s.robots.as_ref())
        })
        .filter(|robots| robots.fetched)
        .map(|robots| find_robots_sitemap_conflicts(sitemap_urls, robots))
        .unwrap_or_default();

    let crawl_depths = build_crawl_depth_diagnostics(sitemap_urls, reports);

    SitemapDiagnostics {
        checked_urls: sitemap_urls.len(),
        http_issues,
        orphan_sitemap_urls,
        linked_not_in_sitemap,
        robots_conflicts,
        crawl_depths,
    }
}

/// Cap on the number of robots.txt/sitemap conflicts surfaced in a batch
/// report (mirrors the existing per-list truncation used for orphan/linked
/// sitemap URLs in the PDF renderer).
const MAX_ROBOTS_SITEMAP_CONFLICTS: usize = 50;

/// Cross-check sitemap-listed URLs against the site's `robots.txt` (#549): a
/// URL listed in the sitemap declares indexing intent, but if `robots.txt`
/// blocks it via a `Disallow` rule under `User-agent: *`, that's a real
/// contradiction crawlers following `robots.txt` will hit.
///
/// Matching is deliberately simplified to plain robots.txt *prefix* matching
/// (`Disallow: /foo` blocks any path starting with `/foo`), plus a
/// same-group `Allow` override when a matching `Allow` rule is at least as
/// specific (long) as the matching `Disallow` rule — a simplified version of
/// the standard robots.txt "longest/most-specific rule wins" precedence.
/// Full wildcard (`*`, `$`) robots.txt pattern matching is out of scope: a
/// plain prefix match already covers the vast majority of real-world
/// robots.txt files.
fn find_robots_sitemap_conflicts(
    sitemap_urls: &[String],
    robots: &RobotsAudit,
) -> Vec<RobotsSitemapConflict> {
    let disallows: Vec<&str> = robots
        .groups
        .iter()
        .filter(|g| g.bot_class == BotClass::Wildcard)
        .flat_map(|g| g.disallows.iter().map(String::as_str))
        .filter(|rule| !rule.is_empty())
        .collect();
    if disallows.is_empty() {
        return Vec::new();
    }
    let allows: Vec<&str> = robots
        .groups
        .iter()
        .filter(|g| g.bot_class == BotClass::Wildcard)
        .flat_map(|g| g.allows.iter().map(String::as_str))
        .filter(|rule| !rule.is_empty())
        .collect();

    let mut conflicts = Vec::new();
    for url in sitemap_urls {
        let Ok(parsed) = Url::parse(url) else {
            continue;
        };
        let path = parsed.path();

        let Some(blocking_rule) = disallows
            .iter()
            .filter(|rule| path.starts_with(**rule))
            .max_by_key(|rule| rule.len())
        else {
            continue;
        };

        let allowed_override = allows
            .iter()
            .any(|rule| path.starts_with(*rule) && rule.len() >= blocking_rule.len());
        if allowed_override {
            continue;
        }

        conflicts.push(RobotsSitemapConflict {
            url: url.clone(),
            rule: (*blocking_rule).to_string(),
        });
        if conflicts.len() >= MAX_ROBOTS_SITEMAP_CONFLICTS {
            break;
        }
    }
    conflicts
}

async fn check_sitemap_url(client: &Client, url: &str) -> Option<SitemapHttpIssue> {
    let response = match client
        .get(url)
        .header("User-Agent", "auditmysite-sitemap-validator/1.0")
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => {
            return Some(SitemapHttpIssue {
                kind: "fetch_error".to_string(),
                url: url.to_string(),
                status_code: None,
                final_url: None,
                detail: err.to_string(),
            });
        }
    };

    let status = response.status();
    if status.is_redirection() {
        let final_url = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|location| resolve_url(url, location))
            .unwrap_or_else(|| url.to_string());
        return Some(SitemapHttpIssue {
            kind: "redirect".to_string(),
            url: url.to_string(),
            status_code: Some(status.as_u16()),
            final_url: Some(final_url.clone()),
            detail: final_url,
        });
    }

    if status.as_u16() != 200 {
        return Some(SitemapHttpIssue {
            kind: "status".to_string(),
            url: url.to_string(),
            status_code: Some(status.as_u16()),
            final_url: None,
            detail: status.to_string(),
        });
    }

    let headers_noindex = response
        .headers()
        .get("x-robots-tag")
        .and_then(|value| value.to_str().ok())
        .is_some_and(contains_noindex);
    let body = response.text().await.unwrap_or_default();
    let meta_noindex = contains_noindex_meta(&body);
    if headers_noindex || meta_noindex {
        return Some(SitemapHttpIssue {
            kind: "noindex".to_string(),
            url: url.to_string(),
            status_code: Some(200),
            final_url: None,
            detail: if headers_noindex {
                "x-robots-tag".to_string()
            } else {
                "meta robots".to_string()
            },
        });
    }

    None
}

fn contains_noindex(value: &str) -> bool {
    value
        .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .any(|part| part.eq_ignore_ascii_case("noindex"))
}

fn contains_noindex_meta(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    lower.contains("<meta")
        && lower.contains("name=\"robots\"")
        && lower.contains("content=")
        && lower.contains("noindex")
}

fn collect_internal_link_targets(reports: &[AuditReport]) -> HashSet<String> {
    let mut targets = HashSet::new();
    for report in reports {
        let Some(base) = Url::parse(&report.url).ok() else {
            continue;
        };
        let Some(seo) = &report.discoverability.seo else {
            continue;
        };
        for target in &seo.technical.internal_link_targets {
            if let Some(normalized) = resolve_link_target(&base, target) {
                targets.insert(normalized);
            }
        }
    }
    targets
}

/// Resolve one raw internal-link-target string (as extracted from a page —
/// may be relative) against that page's own URL, then normalize it the same
/// way as every other URL comparison in this module. Shared by the sitemap
/// link-graph diff (`collect_internal_link_targets`) and crawl-depth BFS
/// (`compute_crawl_depths`) so both resolve/normalize identically.
fn resolve_link_target(base: &Url, target: &str) -> Option<String> {
    let resolved = if target.starts_with("http://") || target.starts_with("https://") {
        target.to_string()
    } else {
        base.join(target)
            .map(|url| url.to_string())
            .unwrap_or_else(|_| target.to_string())
    };
    normalize_url(&resolved)
}

/// Cap on per-list entries surfaced for crawl-depth diagnostics (deepest /
/// unreachable pages). Mirrors the existing 20-row PDF truncation for
/// orphan/linked sitemap URLs — a full per-page dump isn't decision-useful
/// for a large batch (#548).
const MAX_CRAWL_DEPTH_LIST: usize = 20;

/// Build crawl-depth diagnostics for a sitemap-driven batch (#548), or
/// `None` when no reasonable start-page candidate was actually audited.
fn build_crawl_depth_diagnostics(
    sitemap_urls: &[String],
    reports: &[AuditReport],
) -> Option<CrawlDepthDiagnostics> {
    let start_url = choose_crawl_start_url(sitemap_urls, reports)?;
    let depths = compute_crawl_depths(&start_url, reports);

    let mut depth_histogram: BTreeMap<usize, usize> = BTreeMap::new();
    for depth in depths.values() {
        *depth_histogram.entry(*depth).or_insert(0) += 1;
    }

    let mut deepest_pages: Vec<CrawlDepthEntry> = depths
        .iter()
        .map(|(url, depth)| CrawlDepthEntry {
            url: url.clone(),
            depth: *depth,
        })
        .collect();
    deepest_pages.sort_by(|a, b| b.depth.cmp(&a.depth).then_with(|| a.url.cmp(&b.url)));
    deepest_pages.truncate(MAX_CRAWL_DEPTH_LIST);

    let unreachable_set: HashSet<String> = reports
        .iter()
        .filter_map(|r| normalize_url(&r.url))
        .filter(|url| !depths.contains_key(url))
        .collect();
    let mut unreachable_pages: Vec<String> = unreachable_set.into_iter().collect();
    unreachable_pages.sort();
    unreachable_pages.truncate(MAX_CRAWL_DEPTH_LIST);

    // A batch smaller than the full discovered sitemap — whether via
    // `--max-pages` sampling or per-page audit failures — leaves some
    // pages' outbound links unknown, so this depth/reachability view is
    // partial, not site-wide. Reported as an explicit caveat instead of
    // presenting a partial graph as complete (#548 acceptance criterion).
    let partial_batch = reports.len() < sitemap_urls.len();

    Some(CrawlDepthDiagnostics {
        start_url,
        depth_histogram,
        deepest_pages,
        unreachable_pages,
        partial_batch,
    })
}

/// Heuristic start page for crawl-depth BFS: the shortest-path sitemap URL
/// (root `/` naturally wins ties), restricted to URLs that were actually
/// audited — a candidate whose own outbound links were never extracted
/// can't seed a BFS. Ties broken alphabetically for determinism. Returns
/// `None` when every sitemap URL fell outside the audited/sampled set.
fn choose_crawl_start_url(sitemap_urls: &[String], reports: &[AuditReport]) -> Option<String> {
    let audited_urls: HashSet<String> = reports
        .iter()
        .filter_map(|r| normalize_url(&r.url))
        .collect();

    sitemap_urls
        .iter()
        .filter(|url| {
            normalize_url(url).is_some_and(|normalized| audited_urls.contains(&normalized))
        })
        .min_by_key(|url| {
            let path_len = Url::parse(url)
                .map(|u| u.path().len())
                .unwrap_or(usize::MAX);
            (path_len, (*url).clone())
        })
        .cloned()
}

/// BFS distance (in clicks) from `start_url` to every page in `reports`,
/// following each page's internal link targets — scoped to *this batch
/// run's* link graph, not a true site-wide crawl (#548).
///
/// A link only extends the graph when its target is itself one of the
/// audited `reports`: a link to a page outside the batch isn't itself
/// audited, so its own outbound links are unknown and it can't be used to
/// reach anything beyond it. The returned map only ever contains audited
/// page URLs; a page present in `reports` but absent from the returned map
/// was never reached from `start_url` within this batch — a distinct state
/// from "very deep", not a default/fallback depth.
fn compute_crawl_depths(start_url: &str, reports: &[AuditReport]) -> HashMap<String, usize> {
    let Some(start) = normalize_url(start_url) else {
        return HashMap::new();
    };

    let audited_urls: HashSet<String> = reports
        .iter()
        .filter_map(|r| normalize_url(&r.url))
        .collect();
    if !audited_urls.contains(&start) {
        return HashMap::new();
    }

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for report in reports {
        let Some(source) = normalize_url(&report.url) else {
            continue;
        };
        let Some(base) = Url::parse(&report.url).ok() else {
            continue;
        };
        let Some(seo) = &report.discoverability.seo else {
            continue;
        };
        let neighbors = adjacency.entry(source).or_default();
        for target in &seo.technical.internal_link_targets {
            if let Some(normalized) = resolve_link_target(&base, target) {
                if audited_urls.contains(&normalized) {
                    neighbors.push(normalized);
                }
            }
        }
    }

    let mut depths: HashMap<String, usize> = HashMap::new();
    depths.insert(start.clone(), 0);
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(start);
    while let Some(current) = queue.pop_front() {
        let current_depth = depths[&current];
        let Some(neighbors) = adjacency.get(&current) else {
            continue;
        };
        for neighbor in neighbors {
            if !depths.contains_key(neighbor) {
                depths.insert(neighbor.clone(), current_depth + 1);
                queue.push_back(neighbor.clone());
            }
        }
    }
    depths
}

fn resolve_url(base: &str, location: &str) -> Option<String> {
    Url::parse(base).ok()?.join(location).ok().map(|mut url| {
        url.set_fragment(None);
        url.to_string()
    })
}

fn normalize_url(url: &str) -> Option<String> {
    let mut parsed = Url::parse(url).ok()?;
    parsed.set_fragment(None);
    let path = parsed.path().to_string();
    if path != "/" && path.ends_with('/') {
        parsed.set_path(path.trim_end_matches('/'));
    }
    Some(parsed.to_string())
}

/// Audit a single URL using a page from the pool
async fn audit_url_with_pool(
    pool: &BrowserPool,
    url: &str,
    config: &PipelineConfig,
) -> BatchResult {
    let mut last_error: Option<AuditError> = None;
    let per_attempt_timeout = per_attempt_timeout(config.timeout_secs);

    for attempt in 0..2 {
        if attempt > 0 {
            if let Some(e) = &last_error {
                warn!(
                    "Retrying audit for {} (attempt {}): {}",
                    url,
                    attempt + 1,
                    e
                );
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // Waiting for a free page is bounded by the pool's own acquire
        // timeout, not by this page's audit budget: time spent queueing
        // behind a heavy page is not this page's fault (#651). A pool
        // timeout returns at once without spending the retry — the batch
        // retries such pages serially after the parallel phase.
        let pooled_page = match pool.acquire().await {
            Ok(pooled_page) => pooled_page,
            Err(e @ AuditError::PoolTimeout { .. }) => {
                return BatchResult {
                    url: url.to_string(),
                    outcome: Err(BatchAuditError::Audit(e)),
                };
            }
            Err(e) => {
                last_error = Some(e);
                continue;
            }
        };

        let result = tokio::time::timeout(per_attempt_timeout, async {
            let page = pooled_page.page()?;
            // audit_page handles viewport switching and navigation internally
            let (report, snapshot) = audit_page(page, url, config, pool.browser()).await?;
            // Batch applies no canonical-performance pass, so the report is
            // final here — persist it (audit_page no longer persists itself).
            if config.persist_artifacts {
                crate::audit::pipeline::persist_artifacts(url, config, &snapshot, &report);
            }
            Ok::<AuditReport, AuditError>(report)
        })
        .await
        .unwrap_or_else(|_| {
            Err(AuditError::AuditTimeout {
                url: url.to_string(),
                timeout_secs: per_attempt_timeout.as_secs(),
            })
        });

        match result {
            Ok(report) => {
                return BatchResult {
                    url: url.to_string(),
                    outcome: Ok(report),
                };
            }
            Err(
                e @ (AuditError::AuditTimeout { .. }
                | AuditError::PageLoadTimeout { .. }
                | AuditError::AccessBlocked { .. }),
            ) => {
                // Timeouts are not transient — retrying the same URL with the same
                // budget is unlikely to succeed. Neither is a bot wall. Bail immediately.
                return BatchResult {
                    url: url.to_string(),
                    outcome: Err(BatchAuditError::Audit(e)),
                };
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    BatchResult {
        url: url.to_string(),
        outcome: Err(match last_error {
            Some(e) => BatchAuditError::Audit(e),
            None => BatchAuditError::Other("Unknown error".to_string()),
        }),
    }
}

/// Budget for one audit attempt of one page: a generous multiple of the
/// navigation timeout, so that a hung page (tab unresponsive, CDP stream
/// frozen) cannot block the whole batch forever via `in_flight.next().await`.
fn per_attempt_timeout(timeout_secs: u64) -> Duration {
    Duration::from_secs(timeout_secs.max(30) * 4)
}

/// How long a batch page waits for a free browser page (#651).
///
/// A waiting page must be able to outlast one page in flight: creating that
/// page (bounded by the browser timeout), its full attempt budget, and the
/// time its slot takes to come back to the pool. A fixed wait shorter than
/// that dropped heavy pages whenever the page ahead of them legitimately took
/// longer.
pub(crate) fn pool_acquire_timeout_secs(timeout_secs: u64) -> u64 {
    timeout_secs + per_attempt_timeout(timeout_secs).as_secs() + PAGE_RETURN_BUDGET_SECS
}

fn is_pool_timeout(result: &BatchResult) -> bool {
    matches!(
        result.outcome,
        Err(BatchAuditError::Audit(AuditError::PoolTimeout { .. }))
    )
}

/// Count one page as done and report it to the log and the progress callback.
fn report_progress(
    completed: &AtomicUsize,
    total: usize,
    result: &BatchResult,
    progress: Option<&ProgressCallback>,
) {
    let current = completed.fetch_add(1, Ordering::SeqCst) + 1;
    let url = &result.url;
    match &result.outcome {
        Ok(report) => {
            info!(
                "[{}/{}] Completed: {} (score: {})",
                current, total, url, report.accessibility.score
            );
            if let Some(cb) = progress {
                cb(current, total, url, None);
            }
        }
        Err(e) => {
            let msg = e.to_string();
            warn!("[{}/{}] Failed: {} - {}", current, total, url, msg);
            if let Some(cb) = progress {
                cb(current, total, url, Some(&msg));
            }
        }
    }
}

/// Parse a sitemap XML and extract URLs
///
/// Supports both sitemap index files and regular sitemaps.
///
/// # Arguments
/// * `sitemap_url` - URL of the sitemap
///
/// # Returns
/// * `Ok(Vec<String>)` - List of URLs from the sitemap
/// * `Err(AuditError)` - If sitemap parsing fails
pub async fn parse_sitemap(sitemap_url: &str) -> Result<Vec<String>> {
    let client = build_browser_client(15).unwrap_or_default();
    const MAX_SUB_SITEMAPS: usize = 1_000;
    const MAX_SITEMAP_URLS: usize = 100_000;
    let mut queue = VecDeque::from([sitemap_url.to_string()]);
    let mut visited = HashSet::new();
    let mut all_urls = Vec::new();
    let mut seen_urls = HashSet::new();

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current.clone()) {
            warn!(
                "Skipping cyclic or duplicate sitemap reference: {}",
                current
            );
            continue;
        }
        if visited.len() > MAX_SUB_SITEMAPS {
            return Err(AuditError::SitemapParseFailed {
                url: sitemap_url.to_string(),
                reason: format!(
                    "safety limit exceeded: more than {MAX_SUB_SITEMAPS} sitemap documents"
                ),
            });
        }
        info!("Fetching sitemap from: {}", current);
        let content = client
            .get(&current)
            .send()
            .await
            .map_err(|error| AuditError::SitemapParseFailed {
                url: current.clone(),
                reason: error.to_string(),
            })?
            .text()
            .await
            .map_err(|error| AuditError::SitemapParseFailed {
                url: current.clone(),
                reason: error.to_string(),
            })?;

        if content.contains("<sitemapindex") {
            for nested in
                extract_sitemap_urls(&content).map_err(|reason| AuditError::SitemapParseFailed {
                    url: current.clone(),
                    reason,
                })?
            {
                if visited.contains(&nested) {
                    warn!(
                        "Detected cyclic sitemap reference: {} -> {}",
                        current, nested
                    );
                } else {
                    queue.push_back(nested);
                }
            }
            if visited.len() + queue.len() > MAX_SUB_SITEMAPS {
                return Err(AuditError::SitemapParseFailed {
                    url: sitemap_url.to_string(),
                    reason: format!(
                        "safety limit exceeded: more than {MAX_SUB_SITEMAPS} sitemap documents queued"
                    ),
                });
            }
            continue;
        }

        for url in extract_loc_urls(&content).map_err(|reason| AuditError::SitemapParseFailed {
            url: current.clone(),
            reason,
        })? {
            if seen_urls.insert(url.clone()) {
                all_urls.push(url);
                if all_urls.len() > MAX_SITEMAP_URLS {
                    return Err(AuditError::SitemapParseFailed {
                        url: sitemap_url.to_string(),
                        reason: format!(
                            "safety limit exceeded: more than {MAX_SITEMAP_URLS} unique URLs"
                        ),
                    });
                }
            }
        }
    }
    info!(
        "Found {} unique URLs across {} sitemap documents",
        all_urls.len(),
        visited.len()
    );
    Ok(all_urls)
}

/// Fetch a sitemap URL and return the entry count WITHOUT recursing into sub-sitemaps.
/// For sitemap indexes, each `<sitemap>` entry counts as one. Used by the discovery
/// phase to avoid fetching hundreds of sub-sitemaps just to determine whether a sitemap exists.
pub async fn count_sitemap_entries_shallow(sitemap_url: &str) -> Option<usize> {
    let client = build_browser_client(10).ok()?;

    let content = client
        .get(sitemap_url)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let count = extract_all_loc_values(&content).len();
    if count > 0 {
        Some(count)
    } else {
        None
    }
}

/// Extract all <loc> URLs from sitemap XML content.
/// Handles both `<url><loc>` (regular sitemaps) and `<sitemap><loc>` (sitemap indexes).
/// Robust against inline elements, CDATA sections, and varying whitespace.
///
/// Only called once the caller has already confirmed `<sitemapindex` is present, so
/// an empty result here means the index itself has no `<loc>` entries — genuinely
/// malformed, not just an edge case — and is reported as an error rather than
/// silently returning no sub-sitemaps (#QA-042).
fn extract_sitemap_urls(content: &str) -> std::result::Result<Vec<String>, String> {
    let urls = extract_all_loc_values(content);
    if urls.is_empty() {
        return Err("sitemap index contains no <loc> entries for nested sitemaps".to_string());
    }
    Ok(urls)
}

/// Extract <url><loc> URLs from a sitemap.
///
/// A zero-URL result is ambiguous on its own: it's the expected shape for a
/// genuinely empty (but valid) sitemap, but it's also what an HTML error page or
/// wrong content-type served with HTTP 200 produces, since `extract_all_loc_values`
/// is a plain substring scan with no content-type awareness (#QA-042). Distinguish
/// the two with a cheap "does this look like sitemap XML at all" check, so a broken
/// sitemap URL surfaces as a distinct parse failure instead of a silent "0 URLs".
fn extract_loc_urls(content: &str) -> std::result::Result<Vec<String>, String> {
    let urls = extract_all_loc_values(content);
    if urls.is_empty() && !looks_like_sitemap_xml(content) {
        return Err(
            "response does not look like sitemap XML (no <urlset>/<loc> markers found) — \
             the sitemap URL may be returning an error page or the wrong content type"
                .to_string(),
        );
    }
    Ok(urls)
}

/// Cheap heuristic: does this content look like it's at least trying to be sitemap
/// XML? Only checked against the head of the document — the actual `<loc>` scan
/// already covers the full content.
fn looks_like_sitemap_xml(content: &str) -> bool {
    // `get` (not slicing) since a byte-500 cut could land mid-character.
    let head = content.get(..content.len().min(500)).unwrap_or(content);
    head.contains("<?xml") || head.contains("<urlset") || head.contains("<sitemapindex")
}

/// Extract all <loc>...</loc> values from XML content.
/// Works regardless of line structure — handles inline, multiline, and CDATA.
fn extract_all_loc_values(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut search_from = 0;

    while let Some(start_tag) = content[search_from..].find("<loc>") {
        let abs_start = search_from + start_tag + 5; // skip "<loc>"

        if let Some(end_tag) = content[abs_start..].find("</loc>") {
            let abs_end = abs_start + end_tag;
            let mut url = content[abs_start..abs_end].trim().to_string();

            // Handle CDATA: <loc><![CDATA[https://...]]></loc>
            if url.starts_with("<![CDATA[") && url.ends_with("]]>") {
                url = url[9..url.len() - 3].to_string();
            }

            if !url.is_empty() {
                urls.push(url);
            }

            search_from = abs_end + 6; // skip "</loc>"
        } else {
            break;
        }
    }

    urls
}

/// Read URLs from a file (one per line)
///
/// # Arguments
/// * `path` - Path to the URL file
///
/// # Returns
/// * `Ok(Vec<String>)` - List of URLs
/// * `Err(AuditError)` - If file reading fails
pub fn read_url_file(path: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path).map_err(|e| AuditError::FileError {
        path: path.into(),
        reason: e.to_string(),
    })?;

    let urls: Vec<String> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter(|l| l.starts_with("http://") || l.starts_with("https://"))
        .map(String::from)
        .collect();

    info!("Read {} URLs from file: {}", urls.len(), path);
    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #651: a page waiting for a pool slot must outlast one page in flight —
    /// its whole attempt budget plus the slot's return — instead of giving
    /// up after a fixed 60 s.
    #[test]
    fn pool_acquire_timeout_outlasts_one_page_in_flight() {
        for timeout_secs in [0, 10, 30, 120, 600] {
            let wait = pool_acquire_timeout_secs(timeout_secs);
            assert!(
                wait >= timeout_secs
                    + per_attempt_timeout(timeout_secs).as_secs()
                    + PAGE_RETURN_BUDGET_SECS,
                "-t {timeout_secs}: wait {wait}s"
            );
        }
        // -t 120 as in the issue: 120 s page creation + 480 s attempt budget
        // + 15 s page return.
        assert_eq!(pool_acquire_timeout_secs(120), 615);
    }

    #[test]
    fn batch_config_derives_pool_wait_from_timeout() {
        use clap::Parser;
        let args = Args::parse_from(["auditmysite", "https://example.com", "-c", "2", "-t", "120"]);
        let config = BatchConfig::from(&args);
        assert_eq!(config.pool_config.acquire_timeout_secs, 615);

        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let config = BatchConfig::from(&args);
        assert_eq!(
            config.pool_config.acquire_timeout_secs,
            pool_acquire_timeout_secs(30)
        );
    }

    #[test]
    fn only_pool_timeouts_are_deferred_to_the_serial_retry() {
        let result = |error: AuditError| BatchResult {
            url: "https://example.com".to_string(),
            outcome: Err(BatchAuditError::Audit(error)),
        };
        assert!(is_pool_timeout(&result(AuditError::PoolTimeout {
            timeout_secs: 615
        })));
        assert!(!is_pool_timeout(&result(AuditError::AuditTimeout {
            url: "https://example.com".to_string(),
            timeout_secs: 480,
        })));
        assert!(!is_pool_timeout(&result(AuditError::PoolExhausted)));
    }

    #[test]
    fn test_extract_all_loc_values() {
        let urls = extract_all_loc_values("  <loc>https://example.com/page</loc>  ");
        assert_eq!(urls, vec!["https://example.com/page"]);

        let urls = extract_all_loc_values("<loc>https://test.com</loc>");
        assert_eq!(urls, vec!["https://test.com"]);

        let urls = extract_all_loc_values("no loc here");
        assert!(urls.is_empty());
    }

    #[test]
    fn test_extract_loc_urls() {
        let sitemap = r#"<?xml version="1.0"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/page1</loc>
  </url>
  <url>
    <loc>https://example.com/page2</loc>
  </url>
</urlset>"#;

        let urls = extract_loc_urls(sitemap).unwrap();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com/page1".to_string()));
        assert!(urls.contains(&"https://example.com/page2".to_string()));
    }

    #[test]
    fn test_extract_sitemap_urls() {
        let index = r#"<?xml version="1.0"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://example.com/sitemap1.xml</loc>
  </sitemap>
  <sitemap>
    <loc>https://example.com/sitemap2.xml</loc>
  </sitemap>
</sitemapindex>"#;

        let urls = extract_sitemap_urls(index).unwrap();
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_inline_url_loc() {
        // All on one line
        let sitemap = r#"<urlset><url><loc>https://example.com/a</loc></url><url><loc>https://example.com/b</loc></url></urlset>"#;
        let urls = extract_loc_urls(sitemap).unwrap();
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_cdata_loc() {
        let sitemap = r#"<urlset>
  <url>
    <loc><![CDATA[https://example.com/cdata]]></loc>
  </url>
</urlset>"#;
        let urls = extract_loc_urls(sitemap).unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/cdata");
    }

    #[test]
    fn test_genuinely_empty_sitemap_is_not_an_error() {
        // Valid XML, zero <url> entries — must stay Ok(vec![]), not an error (#QA-042).
        let sitemap = r#"<?xml version="1.0"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"></urlset>"#;
        let urls = extract_loc_urls(sitemap).unwrap();
        assert!(urls.is_empty());
    }

    #[test]
    fn test_html_error_page_is_a_distinct_parse_failure() {
        // An HTML error page served with HTTP 200 instead of XML must not be
        // silently treated as "0 URLs" (#QA-042).
        let html = "<!DOCTYPE html><html><body><h1>404 Not Found</h1></body></html>";
        let result = extract_loc_urls(html);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_sitemap_index_is_a_distinct_parse_failure() {
        // <sitemapindex> present but no <loc> entries inside it.
        let index = r#"<?xml version="1.0"?><sitemapindex></sitemapindex>"#;
        let result = extract_sitemap_urls(index);
        assert!(result.is_err());
    }

    #[test]
    fn sitemap_link_graph_uses_audited_internal_targets() {
        let mut report = AuditReport::new(
            "https://example.com/a".to_string(),
            crate::cli::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            10,
        );
        let mut seo = crate::seo::SeoAnalysis::default();
        seo.technical.internal_link_targets = vec!["/b".to_string(), "/linked-only".to_string()];
        report.discoverability.seo = Some(seo);

        let targets = collect_internal_link_targets(&[report.clone()]);
        assert!(targets.contains("https://example.com/b"));
        assert!(targets.contains("https://example.com/linked-only"));

        let sitemap_set: HashSet<String> = ["https://example.com/a", "https://example.com/b"]
            .into_iter()
            .filter_map(normalize_url)
            .collect();
        let audited_set: HashSet<String> = [report.url.as_str()]
            .into_iter()
            .filter_map(normalize_url)
            .collect();

        let mut orphan_sitemap_urls: Vec<String> = sitemap_set
            .iter()
            .filter(|url| audited_set.contains(*url) && !targets.contains(*url))
            .cloned()
            .collect();
        orphan_sitemap_urls.sort();

        let mut linked_not_in_sitemap: Vec<String> =
            targets.difference(&sitemap_set).cloned().collect();
        linked_not_in_sitemap.sort();

        assert_eq!(orphan_sitemap_urls, vec!["https://example.com/a"]);
        assert_eq!(
            linked_not_in_sitemap,
            vec!["https://example.com/linked-only"]
        );
    }

    /// Regression test for #514: a sampled batch (`--max-pages` smaller than
    /// the discovered sitemap) must compare against the *full* sitemap, not
    /// just the sampled/audited subset — otherwise every sitemap page that
    /// falls outside the sample but is still linked from the audited pages'
    /// nav/footer gets wrongly flagged as `linked_not_in_sitemap`, even
    /// though it's genuinely in the sitemap. `/nav-target` here stands in
    /// for a real page like casoon.de's `/webentwicklung/` that a 20-of-113
    /// sample simply didn't happen to include.
    #[test]
    fn sitemap_diagnostics_use_full_sitemap_not_just_the_audited_sample() {
        let mut report = AuditReport::new(
            "https://example.com/audited-page".to_string(),
            crate::cli::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            10,
        );
        let mut seo = crate::seo::SeoAnalysis::default();
        // Every page on the site links to this nav target, trailing slash
        // and all -- the real symptom the #514 report showed.
        seo.technical.internal_link_targets = vec!["/nav-target/".to_string()];
        report.discoverability.seo = Some(seo);

        let targets = collect_internal_link_targets(&[report.clone()]);

        // The *sample* (what was actually audited) does not include
        // `/nav-target` at all -- it was outside the first-N cutoff.
        let sample_urls = ["https://example.com/audited-page"];
        // The *full* discovered sitemap does.
        let full_sitemap_urls = [
            "https://example.com/audited-page",
            "https://example.com/nav-target",
        ];

        for (label, sitemap_urls) in [
            ("sample", &sample_urls[..]),
            ("full", &full_sitemap_urls[..]),
        ] {
            let sitemap_set: HashSet<String> = sitemap_urls
                .iter()
                .filter_map(|u| normalize_url(u))
                .collect();
            let linked_not_in_sitemap: HashSet<String> =
                targets.difference(&sitemap_set).cloned().collect();

            if label == "sample" {
                assert!(
                    linked_not_in_sitemap.contains("https://example.com/nav-target"),
                    "sanity check: comparing against the sample alone should reproduce \
                     the #514 false positive"
                );
            } else {
                assert!(
                    !linked_not_in_sitemap.contains("https://example.com/nav-target"),
                    "comparing against the full sitemap must not flag a page that's \
                     genuinely listed there, just outside the audited sample"
                );
            }
        }
    }

    fn wildcard_robots(disallows: &[&str], allows: &[&str]) -> RobotsAudit {
        RobotsAudit {
            fetched: true,
            groups: vec![crate::seo::RobotsGroup {
                user_agent: "*".to_string(),
                bot_class: BotClass::Wildcard,
                allows: allows.iter().map(|s| s.to_string()).collect(),
                disallows: disallows.iter().map(|s| s.to_string()).collect(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn robots_sitemap_conflict_found_for_wildcard_disallow() {
        let robots = wildcard_robots(&["/private/"], &[]);
        let sitemap_urls = vec![
            "https://example.com/private/page".to_string(),
            "https://example.com/public/page".to_string(),
        ];
        let conflicts = find_robots_sitemap_conflicts(&sitemap_urls, &robots);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].url, "https://example.com/private/page");
        assert_eq!(conflicts[0].rule, "/private/");
    }

    #[test]
    fn robots_sitemap_conflict_suppressed_by_more_specific_allow() {
        let robots = wildcard_robots(&["/private/"], &["/private/public-page"]);
        let sitemap_urls = vec!["https://example.com/private/public-page".to_string()];
        let conflicts = find_robots_sitemap_conflicts(&sitemap_urls, &robots);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn robots_sitemap_conflict_empty_when_no_overlap() {
        let robots = wildcard_robots(&["/admin/"], &[]);
        let sitemap_urls = vec!["https://example.com/products/page".to_string()];
        let conflicts = find_robots_sitemap_conflicts(&sitemap_urls, &robots);
        assert!(conflicts.is_empty());
    }

    /// Build a minimal audited report at `url` whose page links to `targets`
    /// (relative or absolute), matching how `internal_link_targets` is
    /// populated by the real SEO technical-analysis pass.
    fn report_with_links(url: &str, targets: &[&str]) -> AuditReport {
        let mut report = AuditReport::new(
            url.to_string(),
            crate::cli::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            10,
        );
        let mut seo = crate::seo::SeoAnalysis::default();
        seo.technical.internal_link_targets = targets.iter().map(|t| t.to_string()).collect();
        report.discoverability.seo = Some(seo);
        report
    }

    #[test]
    fn compute_crawl_depths_simple_chain() {
        let reports = vec![
            report_with_links("https://example.com/", &["/a"]),
            report_with_links("https://example.com/a", &["/b"]),
            report_with_links("https://example.com/b", &[]),
        ];
        let depths = compute_crawl_depths("https://example.com/", &reports);
        assert_eq!(depths.get("https://example.com/"), Some(&0));
        assert_eq!(depths.get("https://example.com/a"), Some(&1));
        assert_eq!(depths.get("https://example.com/b"), Some(&2));
    }

    #[test]
    fn compute_crawl_depths_shorter_path_wins() {
        // home -> a -> c (2 hops) AND home -> c directly (1 hop): the
        // shorter path must win.
        let reports = vec![
            report_with_links("https://example.com/", &["/a", "/c"]),
            report_with_links("https://example.com/a", &["/c"]),
            report_with_links("https://example.com/c", &[]),
        ];
        let depths = compute_crawl_depths("https://example.com/", &reports);
        assert_eq!(depths.get("https://example.com/c"), Some(&1));
    }

    #[test]
    fn compute_crawl_depths_unreached_page_is_absent_not_defaulted() {
        // `/d` is audited but nothing in the batch links to it.
        let reports = vec![
            report_with_links("https://example.com/", &["/a"]),
            report_with_links("https://example.com/a", &[]),
            report_with_links("https://example.com/d", &[]),
        ];
        let depths = compute_crawl_depths("https://example.com/", &reports);
        assert_eq!(depths.get("https://example.com/a"), Some(&1));
        assert_eq!(depths.get("https://example.com/d"), None);
        assert_eq!(depths.len(), 2);
    }

    #[test]
    fn compute_crawl_depths_single_page_batch() {
        let reports = vec![report_with_links("https://example.com/", &[])];
        let depths = compute_crawl_depths("https://example.com/", &reports);
        assert_eq!(depths.len(), 1);
        assert_eq!(depths.get("https://example.com/"), Some(&0));
    }

    #[test]
    fn compute_crawl_depths_empty_input() {
        let depths = compute_crawl_depths("https://example.com/", &[]);
        assert!(depths.is_empty());
    }

    #[test]
    fn choose_crawl_start_url_prefers_shortest_path_among_audited() {
        let reports = vec![
            report_with_links("https://example.com/", &[]),
            report_with_links("https://example.com/about", &[]),
        ];
        let sitemap_urls = vec![
            "https://example.com/about".to_string(),
            "https://example.com/".to_string(),
        ];
        let start = choose_crawl_start_url(&sitemap_urls, &reports);
        assert_eq!(start, Some("https://example.com/".to_string()));
    }

    #[test]
    fn choose_crawl_start_url_none_when_no_sitemap_url_was_audited() {
        let reports = vec![report_with_links("https://example.com/only-audited", &[])];
        let sitemap_urls = vec!["https://example.com/".to_string()];
        assert_eq!(choose_crawl_start_url(&sitemap_urls, &reports), None);
    }

    #[test]
    fn build_crawl_depth_diagnostics_flags_partial_batch_on_sample() {
        let reports = vec![
            report_with_links("https://example.com/", &["/a"]),
            report_with_links("https://example.com/a", &[]),
        ];
        // Full sitemap has more URLs than were actually audited.
        let sitemap_urls = vec![
            "https://example.com/".to_string(),
            "https://example.com/a".to_string(),
            "https://example.com/b".to_string(),
        ];
        let diag = build_crawl_depth_diagnostics(&sitemap_urls, &reports).unwrap();
        assert_eq!(diag.start_url, "https://example.com/");
        assert!(diag.partial_batch);
        assert_eq!(diag.depth_histogram.get(&0), Some(&1));
        assert_eq!(diag.depth_histogram.get(&1), Some(&1));
    }

    #[test]
    fn build_crawl_depth_diagnostics_no_partial_flag_on_full_coverage() {
        let reports = vec![
            report_with_links("https://example.com/", &["/a"]),
            report_with_links("https://example.com/a", &[]),
        ];
        let sitemap_urls = vec![
            "https://example.com/".to_string(),
            "https://example.com/a".to_string(),
        ];
        let diag = build_crawl_depth_diagnostics(&sitemap_urls, &reports).unwrap();
        assert!(!diag.partial_batch);
    }
}
