//! Batch Processing - Concurrent URL auditing with sitemap support
//!
//! Provides efficient batch processing of multiple URLs with:
//! - Concurrent execution using browser pool
//! - Sitemap XML parsing
//! - URL file processing
//! - Progress reporting

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::{redirect::Policy, Client};
use tracing::{debug, info, warn};
use url::Url;

use super::pipeline::{audit_page, PipelineConfig};
use super::report::{AuditReport, BatchError, BatchReport, SitemapDiagnostics, SitemapHttpIssue};
use crate::browser::{BrowserOptions, BrowserPool, PoolConfig};
use crate::cli::{Args, RequestMode};
use crate::error::{AuditError, Result};
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
            ..PoolConfig::default()
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
    /// The audit result (success or error)
    pub result: std::result::Result<AuditReport, BatchAuditError>,
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
    let mut url_iter = urls.into_iter().take(total_urls);

    let make_task = |url: String,
                     pool: Arc<BrowserPool>,
                     config: Arc<PipelineConfig>,
                     completed: Arc<AtomicUsize>,
                     progress: Option<ProgressCallback>,
                     total: usize| {
        async move {
            let result = audit_url_with_pool(&pool, &url, &config).await;
            let current = completed.fetch_add(1, Ordering::SeqCst) + 1;
            match &result.result {
                Ok(report) => {
                    info!(
                        "[{}/{}] Completed: {} (score: {})",
                        current, total, url, report.accessibility.score
                    );
                    if let Some(ref cb) = progress {
                        cb(current, total, &url, None);
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    warn!("[{}/{}] Failed: {} - {}", current, total, url, msg);
                    if let Some(ref cb) = progress {
                        cb(current, total, &url, Some(&msg));
                    }
                }
            }
            result
        }
    };

    // Fill up to concurrency limit before starting the drain loop
    for url in url_iter.by_ref().take(config.concurrency) {
        in_flight.push(make_task(
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

    while let Some(batch_result) = in_flight.next().await {
        match batch_result.result {
            Ok(report) => reports.push(report),
            Err(e) => errors.push((batch_result.url, e)),
        }
        if let Some(url) = url_iter.next() {
            in_flight.push(make_task(
                url,
                Arc::clone(&pool),
                Arc::clone(&pipeline_config),
                Arc::clone(&completed),
                progress.clone(),
                total_urls,
            ));
        }
    }

    // Close pool - need to unwrap Arc
    // Note: Pool will be dropped when all Arc references are dropped

    let total_duration_ms = start_time.elapsed().as_millis() as u64;

    info!(
        "Batch audit completed: {}/{} successful, {} failed in {}ms",
        reports.len(),
        total_urls,
        errors.len(),
        total_duration_ms
    );

    let batch_errors = errors
        .into_iter()
        .map(|(url, error)| BatchError {
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

    SitemapDiagnostics {
        checked_urls: sitemap_urls.len(),
        http_issues,
        orphan_sitemap_urls,
        linked_not_in_sitemap,
    }
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
            let resolved = if target.starts_with("http://") || target.starts_with("https://") {
                target.to_string()
            } else {
                base.join(target)
                    .map(|url| url.to_string())
                    .unwrap_or_else(|_| target.to_string())
            };
            if let Some(normalized) = normalize_url(&resolved) {
                targets.insert(normalized);
            }
        }
    }
    targets
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
    // Total budget per attempt: generous multiple of the navigation timeout so that
    // a hung page (browser tab unresponsive, CDP stream frozen) cannot block the
    // whole batch forever via in_flight.next().await.
    let per_attempt_timeout = Duration::from_secs(config.timeout_secs.max(30) * 4);

    for attempt in 0..2 {
        if attempt > 0 {
            warn!("Retrying audit for {} (attempt {})", url, attempt + 1);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        let result = tokio::time::timeout(per_attempt_timeout, async {
            let pooled_page = pool.acquire().await?;
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
                    result: Ok(report),
                };
            }
            Err(e @ (AuditError::AuditTimeout { .. } | AuditError::PageLoadTimeout { .. })) => {
                // Timeouts are not transient — retrying the same URL with the same
                // budget is unlikely to succeed. Bail immediately.
                return BatchResult {
                    url: url.to_string(),
                    result: Err(BatchAuditError::Audit(e)),
                };
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    BatchResult {
        url: url.to_string(),
        result: Err(match last_error {
            Some(e) => BatchAuditError::Audit(e),
            None => BatchAuditError::Other("Unknown error".to_string()),
        }),
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
    info!("Fetching sitemap from: {}", sitemap_url);

    let client = build_browser_client(15).unwrap_or_default();

    let response =
        client
            .get(sitemap_url)
            .send()
            .await
            .map_err(|e| AuditError::SitemapParseFailed {
                url: sitemap_url.to_string(),
                reason: e.to_string(),
            })?;

    let content = response
        .text()
        .await
        .map_err(|e| AuditError::SitemapParseFailed {
            url: sitemap_url.to_string(),
            reason: e.to_string(),
        })?;

    // Try to detect if this is a sitemap index
    if content.contains("<sitemapindex") {
        info!("Detected sitemap index, extracting sitemap URLs...");
        let sitemap_urls =
            extract_sitemap_urls(&content).map_err(|reason| AuditError::SitemapParseFailed {
                url: sitemap_url.to_string(),
                reason,
            })?;

        let mut all_urls = Vec::new();
        for sm_url in sitemap_urls {
            debug!("Processing nested sitemap: {}", sm_url);
            match Box::pin(parse_sitemap(&sm_url)).await {
                Ok(urls) => all_urls.extend(urls),
                Err(e) => warn!("Failed to parse nested sitemap {}: {}", sm_url, e),
            }
        }
        return Ok(all_urls);
    }

    // Regular sitemap - extract URLs
    let urls = extract_loc_urls(&content).map_err(|reason| AuditError::SitemapParseFailed {
        url: sitemap_url.to_string(),
        reason,
    })?;
    info!("Found {} URLs in sitemap", urls.len());

    Ok(urls)
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
}
