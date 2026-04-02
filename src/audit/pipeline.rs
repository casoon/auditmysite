//! Audit Pipeline - Orchestrates the complete audit process
//!
//! Coordinates browser management, AXTree extraction, WCAG checking,
//! and report generation.

use std::time::Instant;

use chromiumoxide::Page;
use tracing::{debug, info, warn};

use super::artifacts::{
    content_hash, save_artifacts, AuditArtifacts, FetchArtifact, SnapshotArtifact,
};
use super::normalize;
use super::report::{AuditReport, PerformanceResults};
use crate::accessibility::{enrich_violations_with_page, extract_ax_tree, AXTree};
use crate::browser::BrowserManager;
use crate::cli::{Args, WcagLevel};
use crate::dark_mode::{analyze_dark_mode, DarkModeAnalysis};
use crate::error::Result;
use crate::mobile::{analyze_mobile_friendliness, MobileFriendliness};
use crate::performance::{
    analyze_content_weight, analyze_render_blocking, calculate_performance_score,
    extract_web_vitals,
};
use crate::security::{analyze_security, SecurityAnalysis};
use crate::seo::{analyze_seo, SeoAnalysis};
use crate::wcag::{self, WcagResults};

/// Extracted snapshot data from a loaded page.
///
/// This boundary is the basis for artifact persistence and future cache reuse.
#[derive(Debug, Clone)]
struct SnapshotData {
    ax_tree: AXTree,
    performance: Option<PerformanceResults>,
    seo: Option<SeoAnalysis>,
    security: Option<SecurityAnalysis>,
    mobile: Option<MobileFriendliness>,
    dark_mode: Option<DarkModeAnalysis>,
}

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
    /// Persist audit artifacts under ~/.auditmysite/cache
    pub persist_artifacts: bool,
}

impl From<&Args> for PipelineConfig {
    fn from(args: &Args) -> Self {
        let full_audit = args.full_audit_enabled();
        Self {
            wcag_level: args.level,
            timeout_secs: args.timeout,
            verbose: args.verbose,
            check_performance: (full_audit || args.performance) && !args.skip_performance,
            check_seo: full_audit || args.seo,
            check_security: full_audit || args.security,
            check_mobile: (full_audit || args.mobile) && !args.skip_mobile,
            persist_artifacts: true,
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

    let snapshot = extract_snapshot(page, url, config).await?;
    let wcag_results = run_rules(page, &snapshot, config).await;
    let report = aggregate_report(
        url,
        config,
        &snapshot,
        wcag_results,
        start_time.elapsed().as_millis() as u64,
    );

    if config.persist_artifacts {
        persist_artifacts(url, &snapshot, &report);
    }

    Ok(report)
}

async fn extract_snapshot(page: &Page, url: &str, config: &PipelineConfig) -> Result<SnapshotData> {
    debug!("Extracting Accessibility Tree...");
    let ax_tree = extract_ax_tree(page).await?;
    info!("Extracted {} nodes from AXTree", ax_tree.len());

    let performance = if config.check_performance {
        match extract_web_vitals(page).await {
            Ok(vitals) => {
                let score = calculate_performance_score(&vitals);
                let render_blocking = match analyze_render_blocking(page, url).await {
                    Ok(rb) => Some(rb),
                    Err(e) => {
                        warn!("Render-blocking analysis failed: {}", e);
                        None
                    }
                };
                let content_weight = match analyze_content_weight(page).await {
                    Ok(cw) => Some(cw),
                    Err(e) => {
                        warn!("Content-weight analysis failed: {}", e);
                        None
                    }
                };
                Some(PerformanceResults {
                    vitals,
                    score,
                    render_blocking,
                    content_weight,
                })
            }
            Err(e) => {
                warn!("Performance analysis failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let seo = if config.check_seo {
        match analyze_seo(page, url).await {
            Ok(seo) => Some(seo),
            Err(e) => {
                warn!("SEO analysis failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let security = if config.check_security {
        match analyze_security(url).await {
            Ok(sec) => Some(sec),
            Err(e) => {
                warn!("Security analysis failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let mobile = if config.check_mobile {
        match analyze_mobile_friendliness(page).await {
            Ok(mobile) => Some(mobile),
            Err(e) => {
                warn!("Mobile analysis failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let dark_mode = match analyze_dark_mode(page, config.wcag_level).await {
        Ok(dm) => Some(dm),
        Err(e) => {
            warn!("Dark mode analysis failed: {}", e);
            None
        }
    };

    Ok(SnapshotData {
        ax_tree,
        performance,
        seo,
        security,
        mobile,
        dark_mode,
    })
}

async fn run_rules(page: &Page, snapshot: &SnapshotData, config: &PipelineConfig) -> WcagResults {
    debug!("Running WCAG checks at level {}...", config.wcag_level);
    let mut wcag_results = wcag::check_all(&snapshot.ax_tree, config.wcag_level);

    if matches!(config.wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        info!("Running contrast check with CDP...");
        let contrast_violations =
            wcag::rules::ContrastRule::check_with_page(page, &snapshot.ax_tree, config.wcag_level)
                .await;
        info!("Found {} contrast violations", contrast_violations.len());
        wcag_results.violations.extend(contrast_violations);
    }

    // Enrich violation selectors with actual DOM locations (img src, form ids, etc.)
    enrich_violations_with_page(page, &mut wcag_results.violations, &snapshot.ax_tree).await;

    wcag_results
}

fn aggregate_report(
    url: &str,
    config: &PipelineConfig,
    snapshot: &SnapshotData,
    wcag_results: WcagResults,
    duration_ms: u64,
) -> AuditReport {
    let mut report = AuditReport::new(
        url.to_string(),
        config.wcag_level,
        wcag_results,
        duration_ms,
    );

    if let Some(performance) = snapshot.performance.clone() {
        report = report.with_performance(performance);
    }
    if let Some(seo) = snapshot.seo.clone() {
        report = report.with_seo(seo);
    }
    if let Some(security) = snapshot.security.clone() {
        report = report.with_security(security);
    }
    if let Some(mobile) = snapshot.mobile.clone() {
        report = report.with_mobile(mobile);
    }
    if let Some(dark_mode) = snapshot.dark_mode.clone() {
        report = report.with_dark_mode(dark_mode);
    }

    report
}

fn persist_artifacts(url: &str, snapshot: &SnapshotData, report: &AuditReport) {
    let snapshot_artifact = SnapshotArtifact {
        ax_tree: snapshot.ax_tree.clone(),
        performance: snapshot.performance.clone(),
        seo: snapshot.seo.clone(),
        security: snapshot.security.clone(),
        mobile: snapshot.mobile.clone(),
    };
    let normalized = normalize(report);
    let artifacts = AuditArtifacts {
        fetch: FetchArtifact {
            requested_url: url.to_string(),
            final_url: report.url.clone(),
            status_code: None,
            fetched_at: report.timestamp,
            duration_ms: report.duration_ms,
        },
        snapshot: snapshot_artifact.clone(),
        audit: normalized,
        content_hash: content_hash(&snapshot_artifact),
    };

    if let Err(e) = save_artifacts(url, &artifacts) {
        warn!("Artifact persistence failed for {}: {}", url, e);
    }
}

/// Run audits on multiple URLs.
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
            crawl: false,
            level: WcagLevel::AA,
            format: None,
            output: None,
            chrome_path: None,
            remote_debugging_port: None,
            max_pages: 0,
            crawl_depth: 2,
            concurrency: 3,
            timeout: 30,
            no_sandbox: false,
            disable_images: false,
            verbose: true,
            quiet: false,
            detect_chrome: false,
            full: false,
            performance: false,
            skip_performance: false,
            seo: false,
            security: false,
            mobile: false,
            skip_mobile: false,
            reuse_cache: false,
            force_refresh: false,
            no_sitemap_suggest: false,
            prefer_sitemap: false,
            per_page_reports: false,
            report_level: crate::cli::ReportLevel::Standard,
            lang: "de".to_string(),
            company_name: None,
            logo: None,
            compare: vec![],
        };

        let config = PipelineConfig::from(&args);
        assert_eq!(config.wcag_level, WcagLevel::AA);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.verbose);
        assert!(config.check_performance);
        assert!(config.check_mobile);
        assert!(config.check_seo);
        assert!(config.check_security);
    }
}
