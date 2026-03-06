//! Audit Pipeline - Orchestrates the complete audit process
//!
//! Coordinates browser management, AXTree extraction, WCAG checking,
//! and report generation.

use std::time::Instant;

use chromiumoxide::Page;
use tracing::{debug, info, warn};

use super::report::{AuditReport, PerformanceResults};
use crate::accessibility::extract_ax_tree;
use crate::browser::BrowserManager;
use crate::cli::{Args, WcagLevel};
use crate::error::Result;
use crate::mobile::analyze_mobile_friendliness;
use crate::performance::{calculate_performance_score, extract_web_vitals};
use crate::security::analyze_security;
use crate::seo::analyze_seo;
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
    /// Run performance analysis
    pub check_performance: bool,
    /// Run SEO analysis
    pub check_seo: bool,
    /// Run security analysis
    pub check_security: bool,
    /// Run mobile friendliness analysis
    pub check_mobile: bool,
}

impl From<&Args> for PipelineConfig {
    fn from(args: &Args) -> Self {
        Self {
            wcag_level: args.level,
            timeout_secs: args.timeout,
            verbose: args.verbose,
            check_performance: args.full || args.performance,
            check_seo: args.full || args.seo,
            check_security: args.full || args.security,
            check_mobile: args.full || args.mobile,
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

    // Create report with WCAG results
    let mut report = AuditReport::new(url.to_string(), config.wcag_level, wcag_results, 0);

    // Run optional module checks
    if config.check_performance {
        match extract_web_vitals(page).await {
            Ok(vitals) => {
                let score = calculate_performance_score(&vitals);
                report = report.with_performance(PerformanceResults { vitals, score });
            }
            Err(e) => warn!("Performance analysis failed: {}", e),
        }
    }

    if config.check_seo {
        match analyze_seo(page, url).await {
            Ok(seo) => report = report.with_seo(seo),
            Err(e) => warn!("SEO analysis failed: {}", e),
        }
    }

    if config.check_security {
        match analyze_security(url).await {
            Ok(sec) => report = report.with_security(sec),
            Err(e) => warn!("Security analysis failed: {}", e),
        }
    }

    if config.check_mobile {
        match analyze_mobile_friendliness(page).await {
            Ok(mobile) => report = report.with_mobile(mobile),
            Err(e) => warn!("Mobile analysis failed: {}", e),
        }
    }

    // Set final duration
    report.duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(report)
}

/// Run audits on multiple URLs
///

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_from_args() {
        let args = Args {
            command: None,
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
            full: false,
            performance: false,
            seo: false,
            security: false,
            mobile: false,
        };

        let config = PipelineConfig::from(&args);
        assert_eq!(config.wcag_level, WcagLevel::AA);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.verbose);
        assert!(!config.check_performance);
    }
}
