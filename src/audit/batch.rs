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
use super::report::{AuditReport, BatchReport};
use crate::browser::{BrowserPool, PoolConfig};
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
        let mut pool_config = PoolConfig::default();
        pool_config.max_pages = args.concurrency;
        pool_config.browser_options.chrome_path = args.chrome_path.clone();
        pool_config.browser_options.no_sandbox = args.no_sandbox;
        pool_config.browser_options.disable_images = args.disable_images;
        pool_config.browser_options.timeout_secs = args.timeout;
        pool_config.browser_options.verbose = args.verbose;

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
            let _permit = semaphore.acquire().await.expect("Semaphore closed");

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
            Ok(batch_result) => {
                match batch_result.result {
                    Ok(report) => reports.push(report),
                    Err(e) => errors.push((batch_result.url, e)),
                }
            }
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

    Ok(BatchReport::from_reports(reports, total_duration_ms))
}

/// Audit a single URL using a page from the pool
async fn audit_url_with_pool(
    pool: &BrowserPool,
    url: &str,
    config: &PipelineConfig,
) -> BatchResult {
    let result = async {
        // Acquire a page from the pool
        let pooled_page = pool.acquire().await?;
        let page = pooled_page.page();

        // Navigate to URL
        pool.browser().navigate(page, url).await?;

        // Run audit
        let report = audit_page(page, url, config).await?;

        Ok::<AuditReport, AuditError>(report)
    }
    .await;

    BatchResult {
        url: url.to_string(),
        result: result.map_err(|e| e.to_string()),
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

/// Extract <sitemap><loc> URLs from a sitemap index
fn extract_sitemap_urls(content: &str) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    let mut in_sitemap = false;
    let mut current_loc = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.contains("<sitemap>") || line.contains("<sitemap ") {
            in_sitemap = true;
            current_loc.clear();
        } else if line.contains("</sitemap>") {
            if in_sitemap && !current_loc.is_empty() {
                urls.push(current_loc.clone());
            }
            in_sitemap = false;
        } else if in_sitemap && line.contains("<loc>") {
            if let Some(url) = extract_loc_value(line) {
                current_loc = url;
            }
        }
    }

    Ok(urls)
}

/// Extract <url><loc> URLs from a sitemap
fn extract_loc_urls(content: &str) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    let mut in_url = false;

    for line in content.lines() {
        let line = line.trim();

        if line.contains("<url>") || line.contains("<url ") {
            in_url = true;
        } else if line.contains("</url>") {
            in_url = false;
        } else if in_url && line.contains("<loc>") {
            if let Some(url) = extract_loc_value(line) {
                urls.push(url);
            }
        }
    }

    Ok(urls)
}

/// Extract URL from a <loc>...</loc> line
fn extract_loc_value(line: &str) -> Option<String> {
    let start = line.find("<loc>")? + 5;
    let end = line.find("</loc>")?;
    Some(line[start..end].trim().to_string())
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
    fn test_extract_loc_value() {
        assert_eq!(
            extract_loc_value("  <loc>https://example.com/page</loc>  "),
            Some("https://example.com/page".to_string())
        );
        assert_eq!(extract_loc_value("<loc>https://test.com</loc>"), Some("https://test.com".to_string()));
        assert_eq!(extract_loc_value("no loc here"), None);
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
}
