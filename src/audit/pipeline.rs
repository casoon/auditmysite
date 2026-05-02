//! Audit Pipeline - Orchestrates the complete audit process
//!
//! Coordinates browser management, AXTree extraction, WCAG checking,
//! and report generation.

use std::time::{Duration, Instant};

use chromiumoxide::Page;
use tracing::{debug, info, warn};

use super::artifacts::{
    content_hash, save_artifacts, AuditArtifacts, CacheMeta, FetchArtifact, SnapshotArtifact,
};
use super::normalize;
use super::report::{AuditReport, PerformanceResults};
use crate::accessibility::{enrich_violations_with_page, extract_ax_tree, AXTree};
use crate::browser::BrowserManager;
use crate::cli::{Args, WcagLevel};
use crate::dark_mode::{analyze_dark_mode, DarkModeAnalysis};
use crate::error::Result;
use crate::journey::{analyze_journey, JourneyAnalysis};
use crate::mobile::{analyze_mobile_friendliness, MobileFriendliness};
use crate::performance::{
    analyze_content_weight, analyze_render_blocking, calculate_performance_score,
    extract_web_vitals,
};
use crate::security::{analyze_security, SecurityAnalysis};
use crate::seo::{analyze_seo, SeoAnalysis};
use crate::ux::{analyze_ux, UxAnalysis};
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
    ux: Option<UxAnalysis>,
    journey: Option<JourneyAnalysis>,
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
    /// Capture desktop + mobile screenshots for PDF cover page
    pub capture_screenshots: bool,
}

impl From<&Args> for PipelineConfig {
    fn from(args: &Args) -> Self {
        let full_audit = args.full_audit_enabled();
        Self {
            wcag_level: args.level,
            timeout_secs: args.effective_timeout(),
            verbose: args.verbose,
            check_performance: (full_audit || args.performance) && !args.skip_performance,
            check_seo: full_audit || args.seo,
            check_security: full_audit || args.security,
            check_mobile: (full_audit || args.mobile) && !args.skip_mobile,
            persist_artifacts: true,
            capture_screenshots: args.url.is_some()
                && matches!(args.format, None | Some(crate::cli::OutputFormat::Pdf)),
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
    let mut report = audit_page(&page, url, config).await?;

    if config.capture_screenshots {
        match capture_page_screenshots(&page).await {
            Ok(shots) => {
                report.page_screenshots = Some(shots);
            }
            Err(e) => {
                warn!("Screenshot capture failed (continuing without): {}", e);
            }
        }
    }

    let duration = start_time.elapsed();
    info!(
        "Audit completed for {} in {:?} (score: {})",
        url, duration, report.score
    );

    Ok(report)
}

/// Capture desktop and mobile viewport screenshots of the current page.
async fn capture_page_screenshots(page: &Page) -> crate::error::Result<crate::audit::report::PageScreenshots> {
    use chromiumoxide::cdp::browser_protocol::emulation::{
        ClearDeviceMetricsOverrideParams, SetDeviceMetricsOverrideParams,
    };
    use chromiumoxide::page::ScreenshotParams;

    // Scroll to top before desktop shot
    let _ = page.evaluate("window.scrollTo(0, 0)").await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Desktop at 1280×960 (4:3 ratio — fills more vertical space in the PDF box)
    page.execute(
        SetDeviceMetricsOverrideParams::builder()
            .mobile(false)
            .width(1280_i64)
            .height(960_i64)
            .device_scale_factor(1.0_f64)
            .build()
            .unwrap(),
    )
    .await
    .map_err(|e| crate::error::AuditError::NavigationFailed {
        url: "viewport-desktop".to_string(),
        reason: e.to_string(),
    })?;
    tokio::time::sleep(Duration::from_millis(300)).await;

    let desktop = page
        .screenshot(ScreenshotParams::default())
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "screenshot-desktop".to_string(),
            reason: e.to_string(),
        })?;

    // Mobile at 390×844 (iPhone 14)
    page.execute(
        SetDeviceMetricsOverrideParams::builder()
            .mobile(true)
            .width(390_i64)
            .height(844_i64)
            .device_scale_factor(2.0_f64)
            .build()
            .unwrap(),
    )
    .await
    .map_err(|e| crate::error::AuditError::NavigationFailed {
        url: "viewport-mobile".to_string(),
        reason: e.to_string(),
    })?;

    // Scroll to top — page may have reflowed under mobile viewport
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = page.evaluate("window.scrollTo(0, 0)").await;
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mobile = page
        .screenshot(ScreenshotParams::default())
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "screenshot-mobile".to_string(),
            reason: e.to_string(),
        })?;

    let _ = page.execute(ClearDeviceMetricsOverrideParams {}).await;

    Ok(crate::audit::report::PageScreenshots { desktop, mobile })
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

    // UX analysis runs on AXTree data — no CDP calls needed
    let ux = if config.check_mobile || config.check_seo {
        Some(analyze_ux(&ax_tree))
    } else {
        None
    };

    // Journey analysis runs on AXTree data — no CDP calls needed
    let journey = if config.check_mobile || config.check_seo {
        Some(analyze_journey(&ax_tree))
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
        ux,
        journey,
        dark_mode,
    })
}

async fn run_rules(page: &Page, snapshot: &SnapshotData, config: &PipelineConfig) -> WcagResults {
    debug!("Running WCAG checks at level {}...", config.wcag_level);
    let mut wcag_results = wcag::check_all(&snapshot.ax_tree, config.wcag_level);

    // The AX tree does not expose the html[lang] attribute — verify it via DOM.
    // If the page has a valid lang, remove the false-positive 3.1.1 violation.
    if wcag_results.violations.iter().any(|v| v.rule == "3.1.1") {
        let has_lang = page
            .evaluate(
                "document.documentElement.getAttribute('lang') || \
                 document.documentElement.getAttribute('xml:lang') || ''",
            )
            .await
            .ok()
            .and_then(|r| r.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
            .map(|lang| {
                let l = lang.trim().to_lowercase();
                l.len() >= 2 && l.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
            })
            .unwrap_or(false);

        if has_lang {
            wcag_results.violations.retain(|v| v.rule != "3.1.1");
            wcag_results.passes += 1;
        }
    }

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
    if let Some(ux) = snapshot.ux.clone() {
        report = report.with_ux(ux);
    }
    if let Some(journey) = snapshot.journey.clone() {
        report = report.with_journey(journey);
    }
    if let Some(dark_mode) = snapshot.dark_mode.clone() {
        report = report.with_dark_mode(dark_mode);
    }

    // Source quality is derived from all other modules — must run last
    report.source_quality = Some(crate::source_quality::analyze_source_quality(&report));

    // AI visibility is derived from all other modules (especially SEO)
    report.ai_visibility = Some(crate::ai_visibility::analyze_ai_visibility(&report));

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
    let hash = content_hash(&snapshot_artifact);
    let normalized = normalize(report);
    let wcag_level = report.wcag_level.to_string();
    let meta = CacheMeta {
        auditmysite_version: env!("CARGO_PKG_VERSION").to_string(),
        wcag_level: wcag_level.clone(),
        cached_at: report.timestamp,
        content_hash: hash.clone(),
    };
    let artifacts = AuditArtifacts {
        fetch: FetchArtifact {
            requested_url: url.to_string(),
            final_url: report.url.clone(),
            status_code: None,
            fetched_at: report.timestamp,
            duration_ms: report.duration_ms,
        },
        snapshot: snapshot_artifact,
        audit: normalized,
        content_hash: hash,
        meta,
    };

    if let Err(e) = save_artifacts(url, &wcag_level, &artifacts) {
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
            concurrency: None,
            timeout: None,
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
