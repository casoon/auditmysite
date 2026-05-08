//! Audit Pipeline - Orchestrates the complete audit process
//!
//! Coordinates browser management, AXTree extraction, WCAG checking,
//! and report generation. Every audit runs two viewport passes (desktop 1280×800,
//! mobile 390×844) and blends results with 70 % mobile / 30 % desktop weighting.

use std::time::{Duration, Instant};

use chromiumoxide::Page;
use tracing::{debug, info, warn};

use super::artifacts::{
    content_hash, save_artifacts, AuditArtifacts, CacheMeta, FetchArtifact, SnapshotArtifact,
};
use super::normalize;
use super::report::{
    AuditReport, DualViewportResults, PerformanceResults, ViewportAuditData, ViewportScoreSet,
    ViewportScores,
};
use crate::accessibility::{enrich_violations_with_page, extract_ax_tree, AXTree};
use crate::audit::scoring::AccessibilityScorer;
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
use crate::wcag::{self, Violation, WcagResults};

// ── Viewport helpers ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Viewport {
    Desktop,
    Mobile,
}

async fn set_viewport(page: &Page, viewport: Viewport) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;

    let params = match viewport {
        Viewport::Desktop => SetDeviceMetricsOverrideParams::builder()
            .mobile(false)
            .width(1280_i64)
            .height(800_i64)
            .device_scale_factor(1.0_f64)
            .build()
            .unwrap(),
        Viewport::Mobile => SetDeviceMetricsOverrideParams::builder()
            .mobile(true)
            .width(390_i64)
            .height(844_i64)
            .device_scale_factor(2.0_f64)
            .build()
            .unwrap(),
    };

    page.execute(params)
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "viewport-set".to_string(),
            reason: e.to_string(),
        })?;
    // Wait for layout reflow
    tokio::time::sleep(Duration::from_millis(200)).await;
    Ok(())
}

// ── Snapshot data ─────────────────────────────────────────────────────────────

/// Extracted snapshot data from a loaded page (one viewport pass).
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

// ── Pipeline config ───────────────────────────────────────────────────────────

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
    /// Run dark-mode analysis
    pub check_dark_mode: bool,
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
            check_dark_mode: true,
            persist_artifacts: true,
            capture_screenshots: args.url.is_some()
                && matches!(args.format, None | Some(crate::cli::OutputFormat::Pdf)),
        }
    }
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Run a single-page dual-viewport audit.
pub async fn run_single_audit(
    url: &str,
    browser: &BrowserManager,
    config: &PipelineConfig,
) -> Result<AuditReport> {
    let start_time = Instant::now();
    info!("Starting audit for: {}", url);

    let page = browser.new_page().await?;
    debug!("Created new page");

    let mut report = audit_page(&page, url, config, browser).await?;

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

/// Audit a single page — dual-pass (desktop then mobile).
///
/// Handles its own viewport switching and URL navigation internally.
/// Callers must supply `browser` for re-navigation between passes.
pub async fn audit_page(
    page: &Page,
    url: &str,
    config: &PipelineConfig,
    browser: &BrowserManager,
) -> Result<AuditReport> {
    let start_time = Instant::now();

    // ── Security: viewport-independent, fetch once ────────────────────────────
    let security: Option<SecurityAnalysis> = if config.check_security {
        match analyze_security(url).await {
            Ok(s) => Some(s),
            Err(e) => {
                warn!("Security analysis failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    // ── Desktop pass ──────────────────────────────────────────────────────────
    info!("Desktop pass starting for {}", url);
    set_viewport(page, Viewport::Desktop).await?;
    browser.navigate(page, url).await?;

    let desktop_config = PipelineConfig {
        check_performance: config.check_performance,
        check_seo: false,
        check_security: false,
        check_mobile: false,
        check_dark_mode: true,
        ..config.clone()
    };
    let desktop_snap = extract_snapshot(page, url, &desktop_config).await?;
    let desktop_wcag = run_rules(page, &desktop_snap, config).await;

    // ── Mobile pass ───────────────────────────────────────────────────────────
    info!("Mobile pass starting for {}", url);
    set_viewport(page, Viewport::Mobile).await?;
    browser.navigate(page, url).await?;

    let mobile_config = PipelineConfig {
        check_performance: config.check_performance,
        check_seo: config.check_seo,
        check_security: false,
        check_mobile: config.check_mobile,
        check_dark_mode: false, // taken from desktop pass
        ..config.clone()
    };
    let mobile_snap = extract_snapshot(page, url, &mobile_config).await?;
    let mobile_wcag = run_rules(page, &mobile_snap, config).await;

    // ── Merge ─────────────────────────────────────────────────────────────────
    let merged_wcag = merge_wcag_violations(&desktop_wcag, &mobile_wcag);

    let desktop_acc = AccessibilityScorer::calculate_score(&desktop_wcag.violations);
    let mobile_acc = AccessibilityScorer::calculate_score(&mobile_wcag.violations);

    let desktop_perf_score = desktop_snap.performance.as_ref().map(|p| p.score.overall);
    let mobile_perf_score = mobile_snap.performance.as_ref().map(|p| p.score.overall);
    let mobile_seo_score = mobile_snap.seo.as_ref().map(|s| s.score);
    let mobile_mf_score = mobile_snap.mobile.as_ref().map(|m| m.score);

    let desktop_overall = compute_viewport_overall(desktop_acc, desktop_perf_score, None, None);
    let mobile_overall = compute_viewport_overall(
        mobile_acc,
        mobile_perf_score,
        mobile_seo_score,
        mobile_mf_score,
    );
    let weighted_overall =
        (mobile_overall as f64 * 0.7 + desktop_overall as f64 * 0.3).round() as u32;

    let viewport_scores = ViewportScores {
        desktop: ViewportScoreSet {
            accessibility: desktop_acc.round() as u32,
            performance: desktop_perf_score,
            overall: desktop_overall,
        },
        mobile: ViewportScoreSet {
            accessibility: mobile_acc.round() as u32,
            performance: mobile_perf_score,
            overall: mobile_overall,
        },
        weighted_overall,
    };

    let dual_viewport = DualViewportResults {
        desktop: ViewportAuditData {
            wcag_results: desktop_wcag,
            accessibility_score: desktop_acc,
            performance: desktop_snap.performance.clone(),
            seo: None,
            mobile: None,
            ux: None,
            journey: None,
        },
        mobile: ViewportAuditData {
            wcag_results: mobile_wcag,
            accessibility_score: mobile_acc,
            performance: mobile_snap.performance.clone(),
            seo: mobile_snap.seo.clone(),
            mobile: mobile_snap.mobile.clone(),
            ux: mobile_snap.ux.clone(),
            journey: mobile_snap.journey.clone(),
        },
    };

    // ── Build report ──────────────────────────────────────────────────────────
    // Use mobile pass as primary snapshot (mobile-first); desktop data lives in dual_viewport.
    let primary_snap = SnapshotData {
        ax_tree: mobile_snap.ax_tree.clone(),
        performance: mobile_snap.performance.clone(),
        seo: mobile_snap.seo.clone(),
        security,
        mobile: mobile_snap.mobile.clone(),
        ux: mobile_snap.ux.clone(),
        journey: mobile_snap.journey.clone(),
        dark_mode: desktop_snap.dark_mode.clone(), // taken from desktop pass
    };

    let mut report = aggregate_report(
        url,
        config,
        &primary_snap,
        merged_wcag,
        start_time.elapsed().as_millis() as u64,
    );
    report.dual_viewport = Some(dual_viewport);
    report.viewport_scores = Some(viewport_scores);

    if config.persist_artifacts {
        persist_artifacts(url, &primary_snap, &report);
    }

    Ok(report)
}

// ── Score helpers ─────────────────────────────────────────────────────────────

/// Compute a normalized module score for one viewport pass.
///
/// Weights: Accessibility 40 %, Performance 20 %, SEO 20 %, Mobile 10 %
/// (normalized to active modules, same as the single-pass formula).
fn compute_viewport_overall(
    acc: f32,
    perf: Option<u32>,
    seo: Option<u32>,
    mobile: Option<u32>,
) -> u32 {
    let mut weighted = acc as f64 * 40.0;
    let mut total = 40.0;

    if let Some(p) = perf {
        weighted += p as f64 * 20.0;
        total += 20.0;
    }
    if let Some(s) = seo {
        weighted += s as f64 * 20.0;
        total += 20.0;
    }
    if let Some(m) = mobile {
        weighted += m as f64 * 10.0;
        total += 10.0;
    }

    (weighted / total).round() as u32
}

// ── WCAG deduplication ────────────────────────────────────────────────────────

/// Merge violations from both passes.
///
/// Dedup key: (rule, selector-or-node_id).
/// - Violations present on both → tag "both-viewports" (reported once)
/// - Desktop-only → tag "desktop-only"
/// - Mobile-only → tag "mobile-only"
fn merge_wcag_violations(desktop: &WcagResults, mobile: &WcagResults) -> WcagResults {
    fn dedup_key(v: &Violation) -> (&str, String) {
        let id = v
            .selector
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| v.node_id.clone());
        (v.rule.as_str(), id)
    }

    let mut merged: Vec<Violation> = Vec::new();
    let mut desktop_matched = vec![false; desktop.violations.len()];

    for mv in &mobile.violations {
        let mk = dedup_key(mv);
        let match_idx = desktop.violations.iter().position(|dv| dedup_key(dv) == mk);

        if let Some(idx) = match_idx {
            desktop_matched[idx] = true;
            let mut shared = mv.clone();
            shared.tags.push("both-viewports".to_string());
            merged.push(shared);
        } else {
            let mut mobile_only = mv.clone();
            mobile_only.tags.push("mobile-only".to_string());
            merged.push(mobile_only);
        }
    }

    for (i, dv) in desktop.violations.iter().enumerate() {
        if !desktop_matched[i] {
            let mut desktop_only = dv.clone();
            desktop_only.tags.push("desktop-only".to_string());
            merged.push(desktop_only);
        }
    }

    WcagResults {
        violations: merged,
        passes: mobile.passes.max(desktop.passes),
        incomplete: mobile.incomplete.max(desktop.incomplete),
        nodes_checked: mobile.nodes_checked.max(desktop.nodes_checked),
    }
}

// ── Screenshot capture ────────────────────────────────────────────────────────

/// Capture desktop and mobile viewport screenshots of the current page.
async fn capture_page_screenshots(
    page: &Page,
) -> crate::error::Result<crate::audit::report::PageScreenshots> {
    use chromiumoxide::cdp::browser_protocol::emulation::{
        ClearDeviceMetricsOverrideParams, SetDeviceMetricsOverrideParams,
    };
    use chromiumoxide::page::ScreenshotParams;

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
    tokio::time::sleep(Duration::from_millis(350)).await;
    let _ = page
        .evaluate(
            "window.scrollTo(0,0);\
             document.documentElement.scrollTop=0;\
             document.body.scrollTop=0;\
             if(document.scrollingElement)document.scrollingElement.scrollTop=0;",
        )
        .await;
    tokio::time::sleep(Duration::from_millis(150)).await;

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
    tokio::time::sleep(Duration::from_millis(450)).await;
    let _ = page
        .evaluate(
            "window.scrollTo(0,0);\
             document.documentElement.scrollTop=0;\
             document.body.scrollTop=0;\
             if(document.scrollingElement)document.scrollingElement.scrollTop=0;",
        )
        .await;
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

// ── Internal helpers ──────────────────────────────────────────────────────────

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

    // Security is handled outside extract_snapshot (shared between passes)
    let security = None;

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

    // UX and Journey run once on the mobile pass (gated by check_mobile || check_seo)
    let ux = if config.check_mobile || config.check_seo {
        Some(analyze_ux(&ax_tree))
    } else {
        None
    };

    let journey = if config.check_mobile || config.check_seo {
        Some(analyze_journey(&ax_tree))
    } else {
        None
    };

    let dark_mode = if config.check_dark_mode {
        match analyze_dark_mode(page, config.wcag_level).await {
            Ok(dm) => Some(dm),
            Err(e) => {
                warn!("Dark mode analysis failed: {}", e);
                None
            }
        }
    } else {
        None
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

    report.source_quality = Some(crate::source_quality::analyze_source_quality(&report));
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

// ── Tests ─────────────────────────────────────────────────────────────────────

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
        assert!(config.check_dark_mode);
    }

    #[test]
    fn test_merge_wcag_violations_dedup() {
        use crate::wcag::Severity;

        fn make_v(rule: &str, selector: &str) -> Violation {
            let mut v = Violation::new(rule, rule, WcagLevel::A, Severity::High, "msg", "node-1");
            v.selector = Some(selector.to_string());
            v
        }

        let desktop = WcagResults {
            violations: vec![
                make_v("1.1.1", "#img1"),  // shared
                make_v("2.4.4", "#link1"), // desktop-only
            ],
            passes: 5,
            incomplete: 0,
            nodes_checked: 100,
        };
        let mobile = WcagResults {
            violations: vec![
                make_v("1.1.1", "#img1"),  // shared
                make_v("1.4.3", "#text1"), // mobile-only
            ],
            passes: 4,
            incomplete: 0,
            nodes_checked: 90,
        };

        let merged = merge_wcag_violations(&desktop, &mobile);
        assert_eq!(merged.violations.len(), 3); // 1 shared + 1 desktop-only + 1 mobile-only

        let shared = merged
            .violations
            .iter()
            .find(|v| v.rule == "1.1.1")
            .unwrap();
        assert!(shared.tags.contains(&"both-viewports".to_string()));

        let desktop_only = merged
            .violations
            .iter()
            .find(|v| v.rule == "2.4.4")
            .unwrap();
        assert!(desktop_only.tags.contains(&"desktop-only".to_string()));

        let mobile_only = merged
            .violations
            .iter()
            .find(|v| v.rule == "1.4.3")
            .unwrap();
        assert!(mobile_only.tags.contains(&"mobile-only".to_string()));

        assert_eq!(merged.passes, 5); // max of desktop/mobile
        assert_eq!(merged.nodes_checked, 100);
    }

    #[test]
    fn test_compute_viewport_overall_acc_only() {
        assert_eq!(compute_viewport_overall(80.0, None, None, None), 80);
    }

    #[test]
    fn test_compute_viewport_overall_with_perf() {
        // acc=80 (40%) + perf=60 (20%) → (80*40 + 60*20) / 60 = 4400/60 ≈ 73
        let result = compute_viewport_overall(80.0, Some(60), None, None);
        assert_eq!(result, 73);
    }
}
