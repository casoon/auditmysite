//! Audit Pipeline - Orchestrates the complete audit process
//!
//! Coordinates browser management, AXTree extraction, WCAG checking,
//! and report generation.

use std::time::Instant;

use chromiumoxide::Page;
use tracing::{debug, info, warn};

use super::report::AuditReport;
use crate::accessibility::extract_ax_tree;
use crate::browser::{BrowserManager, BrowserOptions};
use crate::cli::{Args, WcagLevel};
use crate::error::Result;
use crate::wcag;

/// Audit pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// WCAG conformance level to check
    pub wcag_level: WcagLevel,
    /// Page load timeout in seconds
    pub timeout_secs: u64,
    /// Whether to be verbose
    pub verbose: bool,
}

impl From<&Args> for PipelineConfig {
    fn from(args: &Args) -> Self {
        Self {
            wcag_level: args.level,
            timeout_secs: args.timeout,
            verbose: args.verbose,
        }
    }
}

/// Run a single-page audit
///
/// # Arguments
/// * `url` - The URL to audit
/// * `browser` - The browser manager to use
/// * `config` - Pipeline configuration
///
/// # Returns
/// * `Ok(AuditReport)` - The audit results
/// * `Err(AuditError)` - If the audit fails
pub async fn run_single_audit(
    url: &str,
    browser: &BrowserManager,
    config: &PipelineConfig,
) -> Result<AuditReport> {
    let start_time = Instant::now();
    info!("Starting audit for: {}", url);

    // Create a new page
    let page = browser.new_page().await?;
    debug!("Created new page");

    // Navigate to URL
    info!("Navigating to {}...", url);
    browser.navigate(&page, url).await?;

    // Run the audit on this page
    let report = audit_page(&page, url, config).await?;

    let duration = start_time.elapsed();
    info!(
        "Audit completed for {} in {:?} (score: {})",
        url, duration, report.score
    );

    Ok(report)
}

/// Audit a single page that's already loaded
///
/// # Arguments
/// * `page` - The chromiumoxide Page to audit
/// * `url` - The URL (for reporting)
/// * `config` - Pipeline configuration
///
/// # Returns
/// * `Ok(AuditReport)` - The audit results
pub async fn audit_page(page: &Page, url: &str, config: &PipelineConfig) -> Result<AuditReport> {
    let start_time = Instant::now();

    // Extract Accessibility Tree
    debug!("Extracting Accessibility Tree...");
    let ax_tree = extract_ax_tree(page).await?;
    info!("Extracted {} nodes from AXTree", ax_tree.len());

    // Run WCAG checks
    debug!("Running WCAG checks at level {}...", config.wcag_level);
    let mut wcag_results = wcag::check_all(&ax_tree, config.wcag_level);

    // Run contrast check with page access (Level AA and AAA only)
    if matches!(config.wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        info!("Running contrast check with CDP...");
        let contrast_violations =
            wcag::rules::ContrastRule::check_with_page(page, &ax_tree, config.wcag_level).await;
        info!("Found {} contrast violations", contrast_violations.len());
        wcag_results.violations.extend(contrast_violations);
    }

    // Calculate duration
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Create report
    let report = AuditReport::new(url.to_string(), wcag_results, duration_ms);

    Ok(report)
}

/// Run audits on multiple URLs
///
/// # Arguments
/// * `urls` - The URLs to audit
/// * `args` - CLI arguments for configuration
///
/// # Returns
/// * `Ok(Vec<AuditReport>)` - Reports for each URL
pub async fn run_batch_audit(urls: Vec<String>, args: &Args) -> Result<Vec<AuditReport>> {
    let config = PipelineConfig::from(args);

    // Build browser options
    let browser_options = BrowserOptions {
        chrome_path: args.chrome_path.clone(),
        headless: true,
        disable_gpu: true,
        no_sandbox: args.no_sandbox,
        disable_images: args.disable_images,
        window_size: (1920, 1080),
        timeout_secs: args.timeout,
        verbose: args.verbose,
    };

    // Launch browser
    info!("Launching browser...");
    let browser = BrowserManager::with_options(browser_options).await?;

    let mut reports = Vec::with_capacity(urls.len());
    let max_pages = if args.max_pages == 0 {
        urls.len()
    } else {
        args.max_pages.min(urls.len())
    };

    // Process URLs
    for (i, url) in urls.iter().take(max_pages).enumerate() {
        info!("Auditing URL {}/{}: {}", i + 1, max_pages, url);

        match run_single_audit(url, &browser, &config).await {
            Ok(report) => {
                reports.push(report);
            }
            Err(e) => {
                warn!("Failed to audit {}: {}", url, e);
                // Continue with other URLs
            }
        }
    }

    // Close browser
    browser.close().await?;

    Ok(reports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_from_args() {
        let args = Args {
            url: Some("https://example.com".to_string()),
            sitemap: None,
            url_file: None,
            level: WcagLevel::AA,
            format: crate::cli::OutputFormat::Table,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            concurrency: 3,
            timeout: 30,
            no_sandbox: false,
            disable_images: false,
            verbose: true,
            quiet: false,
            detect_chrome: false,
        };

        let config = PipelineConfig::from(&args);
        assert_eq!(config.wcag_level, WcagLevel::AA);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.verbose);
    }
}
