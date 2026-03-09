//! Batch Processing - Concurrent URL auditing with sitemap support
//!
//! Provides efficient batch processing of multiple URLs with:
//! - Concurrent execution using browser pool
//! - Sitemap XML parsing
//! - URL file processing
//! - Progress reporting

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use super::pipeline::{audit_page, PipelineConfig};
use super::report::{AuditReport, BatchError, BatchReport};
use crate::browser::{BrowserOptions, BrowserPool, PoolConfig};
use crate::cli::Args;
use crate::error::{AuditError, Result};

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
            max_pages: args.concurrency,
            browser_options: BrowserOptions {
                chrome_path: args.chrome_path.clone(),
                no_sandbox: args.no_sandbox,
                disable_images: args.disable_images,
                timeout_secs: args.timeout,
                verbose: args.verbose,
                ..BrowserOptions::default()
            },
            ..PoolConfig::default()
        };

        Self {
            pipeline: PipelineConfig::from(args),
            concurrency: args.concurrency,
            max_urls: args.max_pages,
            pool_config,
        }
    }
}

/// Result of a single URL audit within a batch
#[derive(Debug)]
pub struct BatchResult {
    /// The URL that was audited
    pub url: String,
    /// The audit result (success or error)
    pub result: std::result::Result<AuditReport, String>,
}

/// Progress callback type
pub type ProgressCallback = Arc<dyn Fn(usize, usize, &str) + Send + Sync>;

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

    // Semaphore for concurrency control
    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let completed = Arc::new(AtomicUsize::new(0));

    // Spawn tasks for each URL
    let mut handles = Vec::with_capacity(total_urls);

    for url in urls.into_iter().take(total_urls) {
        let pool = Arc::clone(&pool);
        let config = Arc::clone(&pipeline_config);
        let semaphore = Arc::clone(&semaphore);
        let completed = Arc::clone(&completed);
        let progress = progress.clone();
        let total = total_urls;

        let handle = tokio::spawn(async move {
            // Acquire semaphore permit
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    return BatchResult {
                        url: url.to_string(),
                        result: Err("Semaphore closed".to_string()),
                    }
                }
            };

            let result = audit_url_with_pool(&pool, &url, &config).await;

            // Update progress
            let current = completed.fetch_add(1, Ordering::SeqCst) + 1;
            if let Some(ref cb) = progress {
                cb(current, total, &url);
            }

            match &result.result {
                Ok(report) => {
                    info!(
                        "[{}/{}] Completed: {} (score: {})",
                        current, total, url, report.score
                    );
                }
                Err(e) => {
                    warn!("[{}/{}] Failed: {} - {}", current, total, url, e);
                }
            }

            result
        });

        handles.push(handle);
    }

    // Collect results
    let mut reports = Vec::with_capacity(total_urls);
    let mut errors = Vec::new();

    for handle in handles {
        match handle.await {
            Ok(batch_result) => match batch_result.result {
                Ok(report) => reports.push(report),
                Err(e) => errors.push((batch_result.url, e)),
            },
            Err(e) => {
                warn!("Task panicked: {}", e);
            }
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
        .map(|(url, error)| BatchError { url, error })
        .collect();

    Ok(BatchReport::from_reports(
        reports,
        batch_errors,
        total_duration_ms,
    ))
}

/// Audit a single URL using a page from the pool
async fn audit_url_with_pool(
    pool: &BrowserPool,
    url: &str,
    config: &PipelineConfig,
) -> BatchResult {
    let mut last_error = None;

    for attempt in 0..2 {
        if attempt > 0 {
            warn!("Retrying audit for {} (attempt {})", url, attempt + 1);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        let result = async {
            let pooled_page = pool.acquire().await?;
            let page = pooled_page.page()?;
            pool.browser().navigate(page, url).await?;
            let report = audit_page(page, url, config).await?;
            Ok::<AuditReport, AuditError>(report)
        }
        .await;

        match result {
            Ok(report) => {
                return BatchResult {
                    url: url.to_string(),
                    result: Ok(report),
                };
            }
            Err(e) => {
                last_error = Some(e.to_string());
            }
        }
    }

    BatchResult {
        url: url.to_string(),
        result: Err(last_error.unwrap_or_else(|| "Unknown error".to_string())),
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

    let response = reqwest::get(sitemap_url)
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
        let sitemap_urls = extract_sitemap_urls(&content)?;

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
    let urls = extract_loc_urls(&content)?;
    info!("Found {} URLs in sitemap", urls.len());

    Ok(urls)
}

/// Extract all <loc> URLs from sitemap XML content.
/// Handles both `<url><loc>` (regular sitemaps) and `<sitemap><loc>` (sitemap indexes).
/// Robust against inline elements, CDATA sections, and varying whitespace.
fn extract_sitemap_urls(content: &str) -> Result<Vec<String>> {
    Ok(extract_all_loc_values(content))
}

/// Extract <url><loc> URLs from a sitemap
fn extract_loc_urls(content: &str) -> Result<Vec<String>> {
    Ok(extract_all_loc_values(content))
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
}
