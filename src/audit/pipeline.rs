//! Audit Pipeline - Orchestrates the complete audit process
//!
//! Coordinates browser management, AXTree extraction, WCAG checking,
//! and report generation. Every audit runs two viewport passes (desktop 1280×800,
//! mobile 390×844) and blends results with 70 % mobile / 30 % desktop weighting.

use std::collections::BTreeSet;
use std::time::Instant;

use chromiumoxide::cdp::browser_protocol::page::SetBypassCspParams;
use chromiumoxide::cdp::browser_protocol::security::{
    EnableParams as SecurityEnableParams, EventVisibleSecurityStateChanged,
};
use chromiumoxide::listeners::EventStream;
use chromiumoxide::Page;
use chrono::Utc;
use futures::StreamExt;
use tracing::{debug, info, warn};

use super::artifacts::{
    content_hash, save_artifacts, AuditArtifacts, CacheMeta, FetchArtifact, SnapshotArtifact,
};
use super::normalize;
use super::report::{
    AuditReport, ConsentCookieSignal, ConsentPrivacySnapshot, DualViewportResults,
    PerformanceResults, ViewportAuditData, ViewportScoreSet, ViewportScores, ViewportScreenshot,
};
use crate::accessibility::{enrich_violations_with_page, extract_ax_tree, AXTree};
use crate::audit::scoring::AccessibilityScorer;
use crate::best_practices::{prepare_console_collection, BestPracticesAnalysis};
use crate::browser::{
    consent::{handle_post_navigation, inject_consent_cookies},
    throttle, BrowserManager, ThrottleProfile,
};
use crate::cli::{Args, WcagLevel};
use crate::dark_mode::DarkModeAnalysis;
use crate::error::Result;
use crate::interaction::stability::{settle, wait_for_page_stability};
use crate::journey::JourneyAnalysis;
use crate::mobile::MobileFriendliness;
use crate::performance::{prepare_coverage_collection, prepare_vitals_collection};
use crate::security::{analyze_security, BrowserCertificateDetails, SecurityAnalysis};
use crate::seo::SeoAnalysis;
use crate::ux::UxAnalysis;
use crate::wcag::{self, Severity, Violation, WcagResults};

// ── Viewport helpers ──────────────────────────────────────────────────────────

use super::catalog::AuditCatalog;
use super::module::{ModuleContext, ModuleData, Viewport};

async fn set_viewport(page: &Page, viewport: Viewport) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;

    let params = match viewport {
        Viewport::Desktop => SetDeviceMetricsOverrideParams::builder()
            .mobile(false)
            .width(1280_i64)
            .height(800_i64)
            .device_scale_factor(1.0_f64)
            .build()
            .expect("static desktop viewport params are valid"),
        Viewport::Mobile => SetDeviceMetricsOverrideParams::builder()
            .mobile(true)
            .width(390_i64)
            .height(844_i64)
            .device_scale_factor(2.0_f64)
            .build()
            .expect("static mobile viewport params are valid"),
    };

    page.execute(params)
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "viewport-set".to_string(),
            reason: e.to_string(),
        })?;
    settle(page).await?;
    Ok(())
}

async fn collect_certificate_details(
    events: &mut Option<EventStream<EventVisibleSecurityStateChanged>>,
) -> Option<BrowserCertificateDetails> {
    let event = tokio::time::timeout(
        std::time::Duration::from_millis(750),
        events.as_mut()?.next(),
    )
    .await
    .ok()??;
    let cert = event
        .visible_security_state
        .certificate_security_state
        .as_ref()?;
    let valid_to = *cert.valid_to.inner() as i64;
    let now = Utc::now().timestamp();
    Some(BrowserCertificateDetails {
        protocol: non_empty(&cert.protocol),
        cipher: non_empty(&cert.cipher),
        subject: non_empty(&cert.subject_name),
        issuer: non_empty(&cert.issuer),
        valid_to: Some(valid_to),
        expires_in_days: Some((valid_to - now) / 86_400),
        chain_length: Some(cert.certificate.len()),
        error: cert.certificate_network_error.clone(),
        has_weak_signature: Some(cert.certificate_has_weak_signature),
        has_sha1_signature: Some(cert.certificate_has_sha1_signature),
    })
}

fn non_empty(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_string())
}

async fn collect_consent_cookie_signals(page: &Page) -> Vec<ConsentCookieSignal> {
    let mut signals: Vec<ConsentCookieSignal> = page
        .get_cookies()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|cookie| {
            let provider =
                crate::seo::technical::classify_tracking_cookie(&cookie.name).map(|c| c.provider);
            ConsentCookieSignal {
                category: provider
                    .as_deref()
                    .and_then(crate::seo::technical::classify_tracking_provider_category),
                provider,
                name: cookie.name,
                domain: cookie.domain,
                secure: cookie.secure,
                http_only: cookie.http_only,
                same_site: cookie
                    .same_site
                    .as_ref()
                    .map(|same_site| same_site.as_ref().to_string()),
            }
        })
        .collect();
    signals.sort();
    signals.dedup();
    signals
}

fn build_consent_privacy_snapshot(
    before_interaction: Vec<ConsentCookieSignal>,
    after_interaction: Vec<ConsentCookieSignal>,
) -> ConsentPrivacySnapshot {
    let before_keys: BTreeSet<(String, String)> = before_interaction
        .iter()
        .map(|cookie| (cookie.name.clone(), cookie.domain.clone()))
        .collect();
    let added_after_interaction = after_interaction
        .iter()
        .filter(|cookie| !before_keys.contains(&(cookie.name.clone(), cookie.domain.clone())))
        .cloned()
        .collect();
    ConsentPrivacySnapshot {
        before_interaction,
        after_interaction,
        added_after_interaction,
    }
}

// ── Snapshot data ─────────────────────────────────────────────────────────────

/// Extracted snapshot data from a loaded page (one viewport pass).
/// Captured page snapshot for one audit. Returned by `audit_page` so the caller
/// can persist the cache entry after post-processing; fields stay crate-internal.
#[derive(Debug, Clone)]
pub struct SnapshotData {
    ax_tree: AXTree,
    performance: Option<PerformanceResults>,
    seo: Option<SeoAnalysis>,
    security: Option<SecurityAnalysis>,
    mobile: Option<MobileFriendliness>,
    ux: Option<UxAnalysis>,
    journey: Option<JourneyAnalysis>,
    dark_mode: Option<DarkModeAnalysis>,
    tech_stack: Option<crate::tech_stack::TechStackAnalysis>,
    best_practices: Option<BestPracticesAnalysis>,
    module_runs: Vec<crate::audit::ModuleRun>,
}

impl SnapshotData {
    /// Route a `ModuleData` payload into the matching snapshot field.
    ///
    /// Called once per collected module after `catalog.collect_all()`. Keeping
    /// the routing here means adding a new module only requires updating
    /// `SnapshotData` and this method — not the pipeline loop body.
    fn apply_module_data(&mut self, module_id: &str, viewport: Viewport, data: ModuleData) {
        let viewport = match viewport {
            Viewport::Desktop => "desktop",
            Viewport::Mobile => "mobile",
        };
        let derive_only = matches!(
            module_id,
            "source_quality" | "ai_visibility" | "content_visibility" | "commerce"
        );
        let status = match &data {
            ModuleData::Error(_) => crate::audit::ExecutionStatus::Failed,
            ModuleData::None if derive_only => crate::audit::ExecutionStatus::Skipped,
            ModuleData::None => crate::audit::ExecutionStatus::Failed,
            _ => crate::audit::ExecutionStatus::Completed,
        };
        self.module_runs.push(crate::audit::ModuleRun {
            module: module_id.to_string(),
            status,
            viewports: vec![viewport.to_string()],
            subchecks: Vec::new(),
            reason_code: match &data {
                ModuleData::Error(_) => Some("module_collection_failed".to_string()),
                ModuleData::None if derive_only => Some("derive_phase".to_string()),
                ModuleData::None => Some("no_data_returned".to_string()),
                _ => None,
            },
            message: match &data {
                ModuleData::Error(_) => Some("The module could not be collected.".to_string()),
                _ => None,
            },
        });
        match data {
            ModuleData::None => {}
            ModuleData::Performance(p) => self.performance = Some(*p),
            ModuleData::Seo(s) => self.seo = Some(*s),
            ModuleData::Security(s) => self.security = Some(*s),
            ModuleData::Mobile(m) => self.mobile = Some(*m),
            ModuleData::Ux(u) => self.ux = Some(*u),
            ModuleData::Journey(j) => self.journey = Some(*j),
            ModuleData::DarkMode(d) => self.dark_mode = Some(*d),
            ModuleData::TechStack(t) => self.tech_stack = Some(*t),
            ModuleData::BestPractices(b) => self.best_practices = Some(*b),
            ModuleData::SourceQuality(_)
            | ModuleData::AiVisibility(_)
            | ModuleData::ContentVisibility(_)
            | ModuleData::Error(_) => {}
        }
    }
}

// ── Pipeline config ───────────────────────────────────────────────────────────

/// Audit pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// WCAG conformance level to check
    pub wcag_level: WcagLevel,
    /// Page load timeout in seconds
    pub timeout_secs: u64,
    /// Bounded late-render stabilization budget after navigation.
    pub stability_budget_ms: u64,
    /// Whether to be verbose
    pub verbose: bool,
    /// Whether the user selected the full-audit preset.
    pub full_audit: bool,
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
    /// Run tech stack detection and stack-specific audits
    pub check_stack: bool,
    /// Persist audit artifacts under ~/.auditmysite/cache
    pub persist_artifacts: bool,
    /// Capture desktop + mobile screenshots for PDF cover page
    pub capture_screenshots: bool,
    /// Capture cropped element-evidence screenshots for confirmed WCAG
    /// violations (single-URL PDF only — see evidence-grade findings plan).
    pub capture_element_evidence: bool,
    /// Attempt to dismiss cookie consent banners before auditing
    pub dismiss_consent: bool,
    /// Accessibility-Journey-Layer mode (off/basic/full).
    pub interactive: crate::cli::InteractiveMode,
    /// Wall-clock budget for the interactive phase per URL (milliseconds).
    pub journey_budget_ms: u64,
    /// Report locale ("de" / "en") — used for i18n stopword loading.
    pub lang: String,
}

impl PipelineConfig {
    pub fn active_module_labels(&self) -> Vec<&'static str> {
        let mut labels = vec!["Accessibility"];
        if !matches!(self.interactive, crate::cli::InteractiveMode::Off) {
            labels.push("Accessibility Journey");
        }
        let catalog = AuditCatalog::standard();
        if let Ok(mut ordered) = catalog.topo_sorted() {
            ordered.retain(|m| m.is_enabled(self));
            for module in &ordered {
                labels.push(module.label());
            }
        }
        labels
    }

    /// Deterministic fingerprint of the audit-relevant configuration.
    ///
    /// Captures every option that changes the audit's findings, scores, or which
    /// modules run, so a cached result can be rejected when the current run
    /// requests a different scope. Excludes options that do not affect content
    /// (timeout, verbosity, persistence, screenshot capture).
    pub fn audit_signature(&self) -> String {
        // `fmt` is bumped whenever the cached `AuditReport` struct shape changes.
        // `AuditReport` has no `deny_unknown_fields`, so an old-shape cache would
        // otherwise deserialize *successfully* (unknown keys ignored, new fields
        // defaulted) and silently drop data within the same tool version. Bumped
        // Bumped to 2 for the ExperienceSection move, 3 for AccessibilitySection,
        // 4 for DiscoverabilitySection, 5 for the new commerce module field,
        // 6 for the commerce trust-pages restructure, 7 for commerce page_kind,
        // 8 for commerce conversion signals, 9 for structured-data rule and
        // page-fit assessments, 10 for the report quality model, 11 for
        // page-stability provenance.
        const CACHE_FMT: u8 = 11;
        format!(
            "v={};fmt={};level={};perf={};seo={};sec={};mobile={};dark={};stack={};consent={};interactive={:?};journey_budget_ms={};lang={}",
            env!("CARGO_PKG_VERSION"),
            CACHE_FMT,
            self.wcag_level,
            self.check_performance as u8,
            self.check_seo as u8,
            self.check_security as u8,
            self.check_mobile as u8,
            self.check_dark_mode as u8,
            self.check_stack as u8,
            self.dismiss_consent as u8,
            self.interactive,
            self.journey_budget_ms,
            self.lang,
        )
    }
}

impl From<&Args> for PipelineConfig {
    fn from(args: &Args) -> Self {
        let toml_cfg = crate::cli::config::Config::load();
        Self::from_args_and_config(args, toml_cfg.as_ref())
    }
}

impl PipelineConfig {
    /// Return a viewport-specific copy with the correct module on/off pattern.
    ///
    /// Desktop: SEO, security, mobile and stack detection are off (run on mobile pass).
    ///          Dark-mode analysis keeps the configured value.
    /// Mobile:  Security and dark-mode are off.  All other user flags are respected.
    pub fn for_viewport(&self, viewport: Viewport) -> Self {
        match viewport {
            Viewport::Desktop => Self {
                check_seo: false,
                check_security: false,
                check_mobile: false,
                check_stack: false,
                ..self.clone()
            },
            Viewport::Mobile => Self {
                check_security: false,
                check_dark_mode: false,
                ..self.clone()
            },
        }
    }

    pub fn from_args_and_config(
        args: &Args,
        toml_cfg: Option<&crate::cli::config::Config>,
    ) -> Self {
        let full_audit = args.full_audit_enabled();
        let journey_budget_ms = toml_cfg
            .and_then(|c| c.interactive.journey_budget_ms)
            .unwrap_or(crate::a11y_journey::DEFAULT_BUDGET_MS);
        Self {
            wcag_level: args.level,
            timeout_secs: args.effective_timeout(),
            stability_budget_ms: args.stability_budget_ms,
            verbose: args.verbose,
            full_audit,
            check_performance: (full_audit || args.performance) && !args.skip_performance,
            check_seo: full_audit || args.seo,
            check_security: full_audit || args.security,
            check_mobile: (full_audit || args.mobile) && !args.skip_mobile,
            check_dark_mode: true,
            check_stack: full_audit || args.stack,
            persist_artifacts: true,
            capture_screenshots: args.url.is_some()
                && matches!(args.format, None | Some(crate::cli::OutputFormat::Pdf)),
            capture_element_evidence: args.url.is_some()
                && matches!(args.format, None | Some(crate::cli::OutputFormat::Pdf)),
            dismiss_consent: args.dismiss_consent,
            interactive: args.interactive,
            journey_budget_ms,
            lang: args.lang.clone(),
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

    let (mut report, snapshot) = audit_page(&page, url, config, browser).await?;

    if config.check_performance {
        let content_weight = report
            .performance
            .as_ref()
            .and_then(|p| p.content_weight.clone());
        let (throttled, canonical) =
            collect_throttled_performance(&page, url, browser, config, content_weight.as_ref())
                .await;
        attach_throttled_profile_subchecks(&mut report, &throttled);
        report.throttled_performance = throttled;
        if let Some((vitals, score)) = canonical {
            apply_canonical_perf(&mut report, vitals, score);
        }
    }

    // Persist now that the report is final (post canonical-performance), so a
    // cache hit renders identically to this fresh run (#404).
    if config.persist_artifacts {
        persist_artifacts(url, config, &snapshot, &report);
    }

    let duration = start_time.elapsed();
    info!(
        "Audit completed for {} in {:?} (score: {})",
        url, duration, report.accessibility.score
    );

    Ok(report)
}

fn attach_throttled_profile_subchecks(
    report: &mut AuditReport,
    results: &[crate::audit::report::ThrottledPerfResult],
) {
    let subchecks: Vec<crate::audit::SubcheckRun> = ThrottleProfile::AUTO_PROFILES
        .iter()
        .map(|profile| {
            let completed = results.iter().any(|result| result.profile == *profile);
            crate::audit::SubcheckRun {
                subcheck: format!("throttled_profile:{}", profile.label()),
                status: if completed {
                    crate::audit::ExecutionStatus::Completed
                } else {
                    crate::audit::ExecutionStatus::Failed
                },
                reason_code: (!completed).then(|| "profile_measurement_failed".to_string()),
            }
        })
        .collect();

    if let Some(performance) = report
        .accessibility
        .execution
        .module_runs
        .iter_mut()
        .find(|run| run.module == "performance")
    {
        if subchecks
            .iter()
            .any(|check| check.status == crate::audit::ExecutionStatus::Failed)
            && performance.status == crate::audit::ExecutionStatus::Completed
        {
            performance.status = crate::audit::ExecutionStatus::Partial;
            performance.reason_code = Some("throttled_profile_incomplete".to_string());
        }
        performance.subchecks.extend(subchecks);
    }

    update_audit_quality(report);
}

/// Audit a single page — dual-pass (desktop then mobile).
///
/// Handles its own viewport switching and URL navigation internally.
/// Callers must supply `browser` for re-navigation between passes.
///
/// Returns the report together with the captured `SnapshotData` so the caller
/// can persist the cache entry *after* any post-processing (e.g. the canonical
/// performance pass), keeping the cached report identical to a fresh render.
pub async fn audit_page(
    page: &Page,
    url: &str,
    config: &PipelineConfig,
    browser: &BrowserManager,
) -> Result<(AuditReport, SnapshotData)> {
    let start_time = Instant::now();
    // Threaded across both viewport passes: caps element-evidence crops at
    // MAX_ELEMENT_CROPS report-wide and captures each rule at most once
    // (desktop pass runs first, so it wins when a violation is confirmed in
    // both viewports — see `merge_wcag_violations` below).
    let mut evidence_budget = crate::accessibility::ElementEvidenceBudget::new();
    let mut security_listener_failed = false;

    // Enable bypassing CSP on the page
    let _ = page.execute(SetBypassCspParams::new(true)).await;
    let mut security_events = if config.check_security {
        match page
            .event_listener::<EventVisibleSecurityStateChanged>()
            .await
        {
            Ok(events) => {
                let _ = page.execute(SecurityEnableParams::default()).await;
                Some(events)
            }
            Err(e) => {
                warn!("Security CDP listener setup failed: {}", e);
                security_listener_failed = true;
                None
            }
        }
    } else {
        None
    };

    // ── Security: viewport-independent, fetch once ────────────────────────────
    let mut security: Option<SecurityAnalysis> = if config.check_security {
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

    // ── Pre-navigation: inject consent cookies ────────────────────────────────
    if config.dismiss_consent {
        inject_consent_cookies(page, url).await;
    }

    // ── Desktop pass ──────────────────────────────────────────────────────────
    info!("Desktop pass starting for {}", url);
    set_viewport(page, Viewport::Desktop).await?;
    if config.check_performance {
        if let Err(e) = prepare_vitals_collection(page).await {
            warn!("Vitals observer injection failed (desktop): {}", e);
        }
        if let Err(e) = prepare_coverage_collection(page).await {
            warn!("Coverage collection setup failed (desktop): {}", e);
        }
        if let Err(e) = prepare_console_collection(page).await {
            warn!("Console collection setup failed (desktop): {}", e);
        }
    }
    browser.navigate(page, url).await?;
    if let (Some(sec), Some(details)) = (
        &mut security,
        collect_certificate_details(&mut security_events).await,
    ) {
        sec.ssl.apply_certificate_details(details);
    }

    let consent_cookies_before = collect_consent_cookie_signals(page).await;

    // ── Post-navigation: detect and optionally dismiss consent banner ─────────
    let mut consent_result = handle_post_navigation(page, config.dismiss_consent).await;
    let desktop_stability =
        wait_for_page_stability(page, "desktop", config.stability_budget_ms).await;
    let consent_privacy = Some(build_consent_privacy_snapshot(
        consent_cookies_before,
        collect_consent_cookie_signals(page).await,
    ));

    // Capture desktop screenshot
    let desktop_screenshot = match capture_screenshot_with_metadata(page).await {
        Ok(s) => Some(s),
        Err(e) => {
            warn!("Desktop screenshot capture failed: {}", e);
            None
        }
    };

    let desktop_config = config.for_viewport(Viewport::Desktop);
    let desktop_snap = extract_snapshot(page, url, Viewport::Desktop, &desktop_config).await?;
    let desktop_wcag = run_rules(
        page,
        &desktop_snap,
        config,
        desktop_screenshot.as_ref(),
        "desktop",
        &mut evidence_budget,
    )
    .await;

    // ── Mobile pass ───────────────────────────────────────────────────────────
    info!("Mobile pass starting for {}", url);
    set_viewport(page, Viewport::Mobile).await?;
    if config.check_performance {
        if let Err(e) = prepare_vitals_collection(page).await {
            warn!("Vitals observer injection failed (mobile): {}", e);
        }
        if let Err(e) = prepare_coverage_collection(page).await {
            warn!("Coverage collection setup failed (mobile): {}", e);
        }
        if let Err(e) = prepare_console_collection(page).await {
            warn!("Console collection setup failed (mobile): {}", e);
        }
    }
    browser.navigate(page, url).await?;

    // ── Post-navigation: consent banner may reappear after mobile reload ──────
    let mobile_consent = handle_post_navigation(page, config.dismiss_consent).await;
    let mobile_stability =
        wait_for_page_stability(page, "mobile", config.stability_budget_ms).await;
    consent_result.merge(mobile_consent);

    // Capture mobile screenshot
    let mobile_screenshot = match capture_screenshot_with_metadata(page).await {
        Ok(s) => Some(s),
        Err(e) => {
            warn!("Mobile screenshot capture failed: {}", e);
            None
        }
    };

    let mobile_config = config.for_viewport(Viewport::Mobile);
    let mobile_snap = extract_snapshot(page, url, Viewport::Mobile, &mobile_config).await?;
    let mut mobile_wcag = run_rules(
        page,
        &mobile_snap,
        config,
        mobile_screenshot.as_ref(),
        "mobile",
        &mut evidence_budget,
    )
    .await;

    // 1.4.10 Reflow — temporarily sets viewport to 320×256, then restores mobile
    if matches!(config.wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        info!("Running reflow check at 320 CSS px...");
        let raw_reflow_findings = wcag::check_reflow_with_page(page).await;
        let (reflow_outcome, reflow_findings) =
            page_rule_outcome("reflow", Some("1.4.10"), "mobile", raw_reflow_findings);
        if !reflow_findings.is_empty() {
            info!("Found reflow violation at 320px");
        }
        mobile_wcag.rule_outcomes.push(reflow_outcome);
        mobile_wcag.extend_findings(reflow_findings);
        // check_reflow_with_page leaves the viewport at 320px — restore mobile
        if let Err(e) = set_viewport(page, Viewport::Mobile).await {
            warn!(
                "Failed to restore mobile viewport after reflow check: {}",
                e
            );
        }
    }

    // ── Merge ─────────────────────────────────────────────────────────────────
    let mut merged_wcag = merge_wcag_violations(&desktop_wcag, &mobile_wcag);

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
            accessibility: desktop_acc.round().max(1.0) as u32,
            performance: desktop_perf_score,
            overall: desktop_overall,
        },
        mobile: ViewportScoreSet {
            accessibility: mobile_acc.round().max(1.0) as u32,
            performance: mobile_perf_score,
            overall: mobile_overall,
        },
        weighted_overall,
    };
    let weighted_accessibility = viewport_scores.weighted_accessibility();

    let dual_viewport = DualViewportResults {
        desktop: ViewportAuditData {
            wcag_results: desktop_wcag,
            accessibility_score: desktop_acc,
            performance: desktop_snap.performance.clone(),
            seo: None,
            mobile: None,
            ux: None,
            journey: None,
            screenshot: desktop_screenshot.clone(),
            module_runs: consolidate_module_runs(&desktop_snap.module_runs),
        },
        mobile: ViewportAuditData {
            wcag_results: mobile_wcag,
            accessibility_score: mobile_acc,
            performance: mobile_snap.performance.clone(),
            seo: mobile_snap.seo.clone(),
            mobile: mobile_snap.mobile.clone(),
            ux: mobile_snap.ux.clone(),
            journey: mobile_snap.journey.clone(),
            screenshot: mobile_screenshot.clone(),
            module_runs: consolidate_module_runs(&mobile_snap.module_runs),
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
        tech_stack: mobile_snap.tech_stack.clone(),
        best_practices: mobile_snap.best_practices.clone(),
        module_runs: desktop_snap
            .module_runs
            .iter()
            .chain(&mobile_snap.module_runs)
            .cloned()
            .collect(),
    };
    let mut primary_snap = primary_snap;
    if config.check_security {
        primary_snap.module_runs.push(crate::audit::ModuleRun {
            module: "security".to_string(),
            status: match (primary_snap.security.is_some(), security_listener_failed) {
                (true, true) => crate::audit::ExecutionStatus::Partial,
                (true, false) => crate::audit::ExecutionStatus::Completed,
                (false, _) => crate::audit::ExecutionStatus::Failed,
            },
            subchecks: vec![crate::audit::SubcheckRun {
                subcheck: "certificate_listener".to_string(),
                status: if security_listener_failed {
                    crate::audit::ExecutionStatus::Failed
                } else {
                    crate::audit::ExecutionStatus::Completed
                },
                reason_code: security_listener_failed
                    .then(|| "security_listener_setup_failed".to_string()),
            }],
            reason_code: primary_snap
                .security
                .is_none()
                .then(|| "security_analysis_failed".to_string()),
            ..Default::default()
        });
    }

    // ── Pattern analysis — dedicated enrichment pass ──────────────────────────
    // Pattern violations come from the AXTree and reference real DOM nodes, so
    // they can be enriched with selectors just like WCAG violations. Running this
    // here (rather than inside aggregate_report) gives us access to the live page.
    let pattern_analysis = crate::patterns::analyze(&primary_snap.ax_tree);
    let mut pattern_violations = pattern_analysis.violations.clone();
    enrich_violations_with_page(page, &mut pattern_violations, &primary_snap.ax_tree).await;
    let (kept_patterns, demoted_patterns): (Vec<_>, Vec<_>) = pattern_violations
        .into_iter()
        .partition(|v| v.kind == crate::wcag::types::FindingKind::Violation);
    merged_wcag.violations.extend(kept_patterns);
    merged_wcag.warnings.extend(demoted_patterns);

    let mut report = aggregate_report(
        url,
        config,
        &primary_snap,
        merged_wcag,
        pattern_analysis,
        start_time.elapsed().as_millis() as u64,
    );
    report.consent_banner_detected = consent_result.banner_detected;
    report.consent_banner_cmp = consent_result.cmp_name;
    report.consent_banner_dismissed = consent_result.dismissed;
    report.consent_privacy = consent_privacy;
    report.accessibility.execution.environment = crate::audit::ExecutionEnvironment {
        browser_version: browser.chrome_version().map(str::to_string),
        headless: browser.is_headless(),
        source: "live".to_string(),
    };
    report.accessibility.execution.navigation = collect_navigation_snapshot(page, url).await;
    report.accessibility.execution.navigation.stability = vec![desktop_stability, mobile_stability];
    report.accessibility.execution.consent = crate::audit::ConsentAuditState {
        detected: report.consent_banner_detected,
        cmp: report.consent_banner_cmp.clone(),
        dismissal_requested: config.dismiss_consent,
        dismissed: report.consent_banner_dismissed,
        audited_content_state: if report.consent_banner_dismissed {
            crate::audit::AuditedContentState::AfterConsent
        } else if report.consent_banner_detected {
            crate::audit::AuditedContentState::BeforeConsent
        } else {
            crate::audit::AuditedContentState::Unknown
        },
        status: Some(consent_result.status),
        evidence: consent_result.evidence,
    };
    report.dual_viewport = Some(dual_viewport);
    report.viewport_scores = Some(viewport_scores);
    // The report-wide accessibility score is the reproducible 70/30 blend of
    // the two viewport scores shown in JSON/PDF. The merged finding union is
    // still used for evidence, counts and remediation, but must not silently
    // create a third, lower score beside the two viewport values.
    report.accessibility.score = weighted_accessibility as f32;
    report.accessibility.grade =
        AccessibilityScorer::calculate_grade(report.accessibility.score).to_string();
    report.accessibility.certificate =
        AccessibilityScorer::calculate_certificate(report.accessibility.score).to_string();

    if config.capture_screenshots {
        if let (Some(desktop), Some(mobile)) = (&desktop_screenshot, &mobile_screenshot) {
            report.page_screenshots = Some(crate::audit::report::PageScreenshots {
                desktop: desktop.bytes.clone(),
                mobile: mobile.bytes.clone(),
            });
            report.screenshot_status = crate::audit::ScreenshotStatus::Captured;
        } else {
            report.screenshot_status = crate::audit::ScreenshotStatus::Failed(
                "Failed to capture pass screenshots".to_string(),
            );
        }
    } else {
        report.screenshot_status = crate::audit::ScreenshotStatus::NotRequested;
    }

    // ── Accessibility-Journey-Layer (Phase 1 hook — no-op when `off`) ─────────
    // Single call site. Phases 2–5 extend the run() body, not this hook.
    let journey_ctx = crate::a11y_journey::RunContext {
        page,
        mode: config.interactive,
        patterns: report.patterns.as_ref(),
        ax_tree: &primary_snap.ax_tree,
        initial_url: url,
        locale: &config.lang,
        budget_ms: config.journey_budget_ms,
        commerce: report.commerce.as_ref(),
    };
    match crate::a11y_journey::run(journey_ctx).await {
        Ok(Some(out)) => {
            let journey_partial =
                out.journey.execution.failed > 0 || out.journey.execution.budget_exhausted;
            report.accessibility_journey = Some(out.journey);
            report.interactive_findings = out.findings;
            report
                .accessibility
                .execution
                .module_runs
                .push(crate::audit::ModuleRun {
                    module: "accessibility_journey".to_string(),
                    status: if journey_partial {
                        crate::audit::ExecutionStatus::Partial
                    } else {
                        crate::audit::ExecutionStatus::Completed
                    },
                    reason_code: journey_partial.then(|| "journey_coverage_partial".to_string()),
                    ..Default::default()
                });
        }
        Ok(None) => {}
        Err(e) => {
            warn!("Accessibility-Journey-Layer failed: {}", e);
            report
                .accessibility
                .execution
                .module_runs
                .push(crate::audit::ModuleRun {
                    module: "accessibility_journey".to_string(),
                    status: crate::audit::ExecutionStatus::Failed,
                    reason_code: Some("journey_layer_failed".to_string()),
                    message: Some("The interactive audit could not be completed.".to_string()),
                    ..Default::default()
                });
        }
    }
    ensure_requested_module_runs(&mut report);
    report.accessibility.execution.module_runs =
        consolidate_module_runs(&report.accessibility.execution.module_runs);
    update_audit_quality(&mut report);

    // Persistence is deferred to the caller: the single-page path applies the
    // canonical (LhMobile) performance pass *after* audit_page returns, so
    // persisting here would cache a pre-canonical report that differs from a
    // fresh render (#404). The caller persists once the report is final.
    Ok((report, primary_snap))
}

async fn collect_navigation_snapshot(
    page: &Page,
    requested_url: &str,
) -> crate::audit::NavigationSnapshot {
    let value = page
        .evaluate(
            "(() => { const n = performance.getEntriesByType('navigation')[0]; return { finalUrl: location.href, status: n && Number.isFinite(n.responseStatus) ? n.responseStatus : null, redirects: n ? n.redirectCount : 0, readyState: document.readyState }; })()",
        )
        .await
        .ok()
        .and_then(|result| result.value().cloned());
    crate::audit::NavigationSnapshot {
        requested_url: requested_url.to_string(),
        final_url: value
            .as_ref()
            .and_then(|v| v.get("finalUrl"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        main_document_status: value
            .as_ref()
            .and_then(|v| v.get("status"))
            .and_then(|v| v.as_u64())
            .and_then(|v| u16::try_from(v).ok()),
        redirect_count: value
            .as_ref()
            .and_then(|v| v.get("redirects"))
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        ready_state: value
            .as_ref()
            .and_then(|v| v.get("readyState"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        stability: Vec::new(),
    }
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

    fn merge_aux(desktop: &[Violation], mobile: &[Violation]) -> Vec<Violation> {
        let mut merged = Vec::new();
        for finding in mobile {
            let mut finding = finding.clone();
            finding.tags.push("mobile-only".to_string());
            merged.push(finding);
        }
        for finding in desktop {
            if let Some(existing) = merged.iter_mut().find(|candidate| {
                candidate.rule == finding.rule && candidate.message == finding.message
            }) {
                existing
                    .tags
                    .retain(|tag| tag != "mobile-only" && tag != "desktop-only");
                existing.tags.push("both-viewports".to_string());
            } else {
                let mut finding = finding.clone();
                finding.tags.push("desktop-only".to_string());
                merged.push(finding);
            }
        }
        merged
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
            // Evidence-grade findings: prefer the desktop crop when a
            // violation is confirmed in both viewports (owner decision —
            // desktop crops are larger/more legible; the mobile pass skips
            // capturing a rule already captured on desktop, so this is
            // usually a no-op restoring what the mobile clone already lacks).
            if shared.evidence_screenshot.is_none() {
                shared.evidence_screenshot = desktop.violations[idx].evidence_screenshot.clone();
                shared.evidence_viewport = desktop.violations[idx].evidence_viewport;
            }
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

    // Merge warnings, positives, and not_testables without deduplication
    // (heuristic/structural signals — viewport tagging not meaningful).
    let warnings = merge_aux(&desktop.warnings, &mobile.warnings);
    let positives = merge_aux(&desktop.positives, &mobile.positives);
    let not_testables = merge_aux(&desktop.not_testables, &mobile.not_testables);

    WcagResults {
        violations: merged,
        warnings,
        positives,
        not_testables,
        passes: mobile.passes.max(desktop.passes),
        incomplete: mobile.incomplete.max(desktop.incomplete),
        nodes_checked: mobile.nodes_checked.max(desktop.nodes_checked),
        rule_outcomes: desktop
            .rule_outcomes
            .iter()
            .chain(&mobile.rule_outcomes)
            .cloned()
            .collect(),
    }
}

// ── Screenshot capture ────────────────────────────────────────────────────────

/// Capture viewport screenshot with layout metadata.
async fn capture_screenshot_with_metadata(page: &Page) -> crate::error::Result<ViewportScreenshot> {
    use chromiumoxide::page::ScreenshotParams;

    // Reset scroll to (0, 0)
    let _ = page
        .evaluate(
            "window.scrollTo(0,0);\
             document.documentElement.scrollTop=0;\
             document.body.scrollTop=0;\
             if(document.scrollingElement)document.scrollingElement.scrollTop=0;",
        )
        .await;

    settle(page).await?;

    // Query viewport metadata
    let eval_res = page
        .evaluate(
            "(() => {
                return {
                    dpr: window.devicePixelRatio || 1.0,
                    width: window.innerWidth || 1280,
                    height: window.innerHeight || 800,
                    scrollX: window.scrollX || window.pageXOffset || 0,
                    scrollY: window.scrollY || window.pageYOffset || 0
                };
             })()",
        )
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "viewport-metadata".to_string(),
            reason: e.to_string(),
        })?;

    let val = eval_res
        .value()
        .ok_or_else(|| crate::error::AuditError::NavigationFailed {
            url: "viewport-metadata".to_string(),
            reason: "No value returned from metadata evaluation".to_string(),
        })?;

    let dpr = val.get("dpr").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let width = val.get("width").and_then(|v| v.as_u64()).unwrap_or(1280) as u32;
    let height = val.get("height").and_then(|v| v.as_u64()).unwrap_or(800) as u32;
    let scroll_x = val.get("scrollX").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let scroll_y = val.get("scrollY").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let bytes = page
        .screenshot(ScreenshotParams::default())
        .await
        .map_err(|e| crate::error::AuditError::NavigationFailed {
            url: "screenshot-capture".to_string(),
            reason: e.to_string(),
        })?;

    Ok(ViewportScreenshot {
        bytes,
        width,
        height,
        device_scale_factor: dpr,
        scroll_x,
        scroll_y,
    })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

async fn extract_snapshot(
    page: &Page,
    url: &str,
    viewport: Viewport,
    config: &PipelineConfig,
) -> Result<SnapshotData> {
    debug!("Extracting Accessibility Tree...");
    let ax_tree = extract_ax_tree(page).await?;
    info!("Extracted {} nodes from AXTree", ax_tree.len());

    // ── Catalog-driven collection (A3 / #332) ─────────────────────────────────
    // Eight collection modules run through the AuditCatalog. Per-module
    // failure semantics (warn + None) live inside each `impl AuditModule`.
    // Security is registered in the catalog but `is_enabled` returns false
    // here — per-pass configs set `check_security = false` so it stays None
    // in the snapshot and the top-level fetch in `audit_page` provides the
    // actual SecurityAnalysis.
    let collected = {
        let catalog = AuditCatalog::standard();
        let ctx = ModuleContext {
            page,
            url,
            viewport,
            ax_tree: &ax_tree,
            pipeline_config: config,
        };
        catalog.collect_all(&ctx).await?
    };

    let mut snapshot = SnapshotData {
        ax_tree,
        performance: None,
        seo: None,
        security: None,
        mobile: None,
        ux: None,
        journey: None,
        dark_mode: None,
        tech_stack: None,
        best_practices: None,
        module_runs: Vec::new(),
    };

    for (id, data) in collected {
        snapshot.apply_module_data(id, viewport, data);
    }

    Ok(snapshot)
}

async fn run_rules(
    page: &Page,
    snapshot: &SnapshotData,
    config: &PipelineConfig,
    screenshot: Option<&ViewportScreenshot>,
    viewport_label: &'static str,
    evidence_budget: &mut crate::accessibility::ElementEvidenceBudget,
) -> WcagResults {
    debug!("Running WCAG checks at level {}...", config.wcag_level);
    let mut wcag_results = wcag::check_all(&snapshot.ax_tree, config.wcag_level);
    for outcome in &mut wcag_results.rule_outcomes {
        outcome.viewport = Some(viewport_label.to_string());
    }
    let mut lang_outcome = apply_lang_attribute_check(page, &mut wcag_results).await;
    lang_outcome.viewport = Some(viewport_label.to_string());
    wcag_results.rule_outcomes.push(lang_outcome);

    // Contrast carries extra args (ax tree, level, screenshot) and stays inline.
    if matches!(config.wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        info!("Running contrast check with CDP...");
        let contrast_violations = wcag::rules::ContrastRule::check_with_page(
            page,
            &snapshot.ax_tree,
            config.wcag_level,
            screenshot,
        )
        .await;
        let (outcome, findings) = page_rule_outcome(
            "color-contrast",
            Some("1.4.3"),
            viewport_label,
            contrast_violations,
        );
        info!("Found {} contrast findings", findings.len());
        wcag_results.rule_outcomes.push(outcome);
        wcag_results.extend_findings(findings);
    }

    // Table-driven page rules (#334). min_level gates each entry.
    for rule in wcag::rules::PAGE_RULES {
        if config.wcag_level < rule.min_level {
            continue;
        }
        let raw_findings = (rule.check_fn)(page).await;
        let criterion = rule
            .rule_id
            .split('/')
            .next()
            .filter(|id| id.chars().all(|c| c.is_ascii_digit() || c == '.'));
        let (outcome, findings) =
            page_rule_outcome(rule.rule_id, criterion, viewport_label, raw_findings);
        if !findings.is_empty() {
            info!("Found {} {} violations", findings.len(), rule.name);
        }
        wcag_results.rule_outcomes.push(outcome);
        wcag_results.extend_findings(findings);
    }

    enrich_violations_with_page(page, &mut wcag_results.violations, &snapshot.ax_tree).await;

    if config.capture_element_evidence {
        crate::accessibility::capture_element_evidence(
            page,
            &mut wcag_results.violations,
            &snapshot.ax_tree,
            viewport_label,
            evidence_budget,
        )
        .await;
    }

    move_demoted_violations_to_warnings(&mut wcag_results);
    wcag_results.incomplete = wcag_results
        .rule_outcomes
        .iter()
        .filter(|outcome| outcome.status == crate::wcag::RuleOutcomeStatus::Failed)
        .count();
    wcag_results
}

fn page_rule_outcome(
    rule_id: &str,
    criterion: Option<&str>,
    viewport: &str,
    findings: Vec<Violation>,
) -> (crate::wcag::RuleOutcome, Vec<Violation>) {
    let technical_failure = findings
        .iter()
        .find_map(crate::wcag::technical_failure_reason)
        .map(str::to_string);
    let visible_findings: Vec<Violation> = findings
        .into_iter()
        .filter(|finding| crate::wcag::technical_failure_reason(finding).is_none())
        .collect();

    let violation_count = visible_findings
        .iter()
        .filter(|finding| finding.kind == crate::wcag::FindingKind::Violation)
        .count();
    let status = if technical_failure.is_some() {
        crate::wcag::RuleOutcomeStatus::Failed
    } else if violation_count > 0 {
        crate::wcag::RuleOutcomeStatus::ViolationsFound
    } else if visible_findings
        .iter()
        .any(|finding| finding.kind == crate::wcag::FindingKind::Warning)
    {
        crate::wcag::RuleOutcomeStatus::Warning
    } else if visible_findings
        .iter()
        .any(|finding| finding.kind == crate::wcag::FindingKind::NotTestable)
    {
        crate::wcag::RuleOutcomeStatus::ManualReviewRequired
    } else {
        crate::wcag::RuleOutcomeStatus::NoViolationDetected
    };

    (
        crate::wcag::RuleOutcome {
            rule_id: rule_id.to_string(),
            status,
            wcag_criterion: criterion.map(str::to_string),
            viewport: Some(viewport.to_string()),
            reason_code: technical_failure,
            finding_count: violation_count,
        },
        visible_findings,
    )
}

/// 3.1.1 verifying-subtraction: the AX tree does not expose `html[lang]`,
/// so if check_all emitted a 3.1.1 violation, query the DOM and remove it
/// when a valid lang attribute is present.
async fn apply_lang_attribute_check(
    page: &Page,
    results: &mut WcagResults,
) -> crate::wcag::RuleOutcome {
    // The AX-tree-based check_language (language.rs) reads the AX `language`
    // property, which Chrome can synthesize from locale/context even when the
    // author never set a `lang` attribute — making the tree-based check blind
    // to the most common 3.1.1 violation (#QA-001). The DOM `lang`/`xml:lang`
    // attribute is authoritative, so it can both clear a false-positive AND
    // add a violation the tree-based check missed.
    let evaluated = page
        .evaluate(
            "document.documentElement.getAttribute('lang') || \
             document.documentElement.getAttribute('xml:lang') || ''",
        )
        .await;
    let has_lang = evaluated
        .as_ref()
        .ok()
        .and_then(|r| r.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .map(|lang| {
            let l = lang.trim().to_lowercase();
            l.len() >= 2 && l.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
        })
        .unwrap_or(false);

    let has_violation = results.violations.iter().any(|v| v.rule == "3.1.1");

    if evaluated.is_err() {
        return crate::wcag::RuleOutcome {
            rule_id: crate::wcag::rules::LANGUAGE_RULE.axe_id.to_string(),
            status: crate::wcag::RuleOutcomeStatus::Failed,
            wcag_criterion: Some("3.1.1".to_string()),
            viewport: None,
            reason_code: Some("page_evaluation_failed".to_string()),
            finding_count: usize::from(has_violation),
        };
    }

    if has_lang {
        if has_violation {
            results.violations.retain(|v| v.rule != "3.1.1");
            results.passes += 1;
        }
    } else if !has_violation {
        results.violations.push(
            Violation::new(
                crate::wcag::rules::LANGUAGE_RULE.id,
                crate::wcag::rules::LANGUAGE_RULE.name,
                crate::wcag::rules::LANGUAGE_RULE.level,
                Severity::High,
                "Page is missing a valid lang attribute on the html element",
                "document",
            )
            .with_fix("Add a valid lang attribute to the <html> element, e.g., <html lang=\"en\">")
            .with_rule_id(crate::wcag::rules::LANGUAGE_RULE.axe_id)
            // "document" isn't a real AX node id, so enrich_violations_with_page
            // can't resolve a backend_dom_node_id for it — without an explicit
            // selector it demotes the violation to a Warning (ghost element),
            // silently dropping it from findings[] despite this fix (#QA-001).
            .with_selector("html")
            .with_help_url(crate::wcag::rules::LANGUAGE_RULE.help_url),
        );
    }

    crate::wcag::RuleOutcome {
        rule_id: crate::wcag::rules::LANGUAGE_RULE.axe_id.to_string(),
        status: if results.violations.iter().any(|v| v.rule == "3.1.1") {
            crate::wcag::RuleOutcomeStatus::ViolationsFound
        } else {
            crate::wcag::RuleOutcomeStatus::NoViolationDetected
        },
        wcag_criterion: Some("3.1.1".to_string()),
        viewport: None,
        reason_code: None,
        finding_count: usize::from(results.violations.iter().any(|v| v.rule == "3.1.1")),
    }
}

/// After enrichment, violations whose kind was demoted to Warning (e.g.
/// because no DOM element could be located) must be moved to the warnings
/// Vec so they don't inflate violation counts or affect scoring.
fn move_demoted_violations_to_warnings(results: &mut WcagResults) {
    let (kept, demoted): (Vec<_>, Vec<_>) = results
        .violations
        .drain(..)
        .partition(|v| v.kind == crate::wcag::types::FindingKind::Violation);
    results.violations = kept;
    results.warnings.extend(demoted);
}

fn aggregate_report(
    url: &str,
    config: &PipelineConfig,
    snapshot: &SnapshotData,
    wcag_results: WcagResults,
    pattern_analysis: crate::patterns::PatternAnalysis,
    duration_ms: u64,
) -> AuditReport {
    // Pattern violations were already enriched and merged into wcag_results by
    // the caller (audit_page), which has access to the live page for CDP lookups.
    let mut report = AuditReport::new(
        url.to_string(),
        config.wcag_level,
        wcag_results,
        duration_ms,
    );
    report.accessibility.execution.scope = audit_scope_from_config(config);
    report.accessibility.execution.navigation.requested_url = url.to_string();
    report.accessibility.execution.environment.source = "live".to_string();
    report.accessibility.execution.module_runs = consolidate_module_runs(&snapshot.module_runs);
    let sr_audit = crate::screen_reader::build_sr_audit_report(
        url,
        report.timestamp,
        &snapshot.ax_tree,
        &config.lang,
        Some(&pattern_analysis),
    );
    report = report.with_patterns(pattern_analysis);
    report.screen_reader_audit = Some(sr_audit);

    if let Some(performance) = snapshot.performance.clone() {
        report = report.with_performance(performance);
        attach_performance_subchecks(&mut report);
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

    if let Some(tech_stack) = snapshot.tech_stack.clone() {
        report = report.with_tech_stack(tech_stack);
    }

    if let Some(bp) = snapshot.best_practices.clone() {
        report = report.with_best_practices(bp);
    }

    // Post-processing modules (source_quality, ai_visibility, content_visibility)
    // run via the catalog's derive phase. Topo order ensures content_visibility
    // sees source_quality + ai_visibility populated.
    if let Err(e) = AuditCatalog::standard().derive_all(&mut report, config) {
        warn!("Post-processing derive_all failed: {}", e);
    }
    report.accessibility.execution.module_runs =
        consolidate_module_runs(&report.accessibility.execution.module_runs);

    report
}

fn attach_performance_subchecks(report: &mut AuditReport) {
    let Some(performance) = report.performance.as_ref() else {
        return;
    };
    let checks = [
        ("content_weight", performance.content_weight.is_some()),
        ("render_blocking", performance.render_blocking.is_some()),
        ("third_party", performance.third_party.is_some()),
        ("critical_chain", performance.critical_chain.is_some()),
        ("minification", performance.minification.is_some()),
        ("animations", performance.animations.is_some()),
        ("coverage", performance.coverage.is_some()),
    ];
    if let Some(run) = report
        .accessibility
        .execution
        .module_runs
        .iter_mut()
        .find(|run| run.module == "performance")
    {
        run.subchecks = checks
            .iter()
            .map(|(name, available)| crate::audit::SubcheckRun {
                subcheck: (*name).to_string(),
                status: if *available {
                    crate::audit::ExecutionStatus::Completed
                } else {
                    crate::audit::ExecutionStatus::Failed
                },
                reason_code: (!available).then(|| "subanalysis_unavailable".to_string()),
            })
            .collect();
        if checks.iter().any(|(_, available)| !available) {
            run.status = crate::audit::ExecutionStatus::Partial;
            run.reason_code = Some("one_or_more_subanalyses_unavailable".to_string());
        }
    }
}

fn audit_scope_from_config(config: &PipelineConfig) -> crate::audit::AuditScope {
    let catalog = AuditCatalog::standard();
    let mut requested_modules = vec!["accessibility".to_string()];
    if !matches!(config.interactive, crate::cli::InteractiveMode::Off) {
        requested_modules.push("accessibility_journey".to_string());
    }
    requested_modules.extend(
        catalog
            .enabled(config)
            .map(|module| module.id().to_string()),
    );
    requested_modules.sort();
    requested_modules.dedup();

    crate::audit::AuditScope {
        requested_modules,
        full_audit: config.full_audit,
        interactive_mode: format!("{:?}", config.interactive).to_lowercase(),
        journey_budget_ms: config.journey_budget_ms,
        throttling_profiles: if config.check_performance {
            ThrottleProfile::AUTO_PROFILES
                .iter()
                .map(|profile| profile.label().to_string())
                .collect()
        } else {
            Vec::new()
        },
        viewports: vec![
            crate::audit::ViewportDefinition {
                name: "desktop".to_string(),
                width: 1280,
                height: 800,
                device_scale_factor: 1.0,
            },
            crate::audit::ViewportDefinition {
                name: "mobile".to_string(),
                width: 390,
                height: 844,
                device_scale_factor: 2.0,
            },
        ],
        dismiss_consent: config.dismiss_consent,
        capture_screenshots: config.capture_screenshots,
        capture_element_evidence: config.capture_element_evidence,
    }
}

fn ensure_requested_module_runs(report: &mut AuditReport) {
    let outcomes = &report.accessibility.wcag_results.rule_outcomes;
    let failed = outcomes
        .iter()
        .filter(|outcome| outcome.status == crate::wcag::RuleOutcomeStatus::Failed)
        .count();
    let accessibility_status = if outcomes.is_empty() || failed == outcomes.len() {
        crate::audit::ExecutionStatus::Failed
    } else if failed > 0 {
        crate::audit::ExecutionStatus::Partial
    } else {
        crate::audit::ExecutionStatus::Completed
    };
    if !report
        .accessibility
        .execution
        .module_runs
        .iter()
        .any(|run| run.module == "accessibility")
    {
        report
            .accessibility
            .execution
            .module_runs
            .push(crate::audit::ModuleRun {
                module: "accessibility".to_string(),
                status: accessibility_status,
                reason_code: (accessibility_status != crate::audit::ExecutionStatus::Completed)
                    .then(|| "one_or_more_rule_checks_failed".to_string()),
                ..Default::default()
            });
    }

    let requested = report
        .accessibility
        .execution
        .scope
        .requested_modules
        .clone();
    for module in requested {
        if !report
            .accessibility
            .execution
            .module_runs
            .iter()
            .any(|run| run.module == module)
        {
            report
                .accessibility
                .execution
                .module_runs
                .push(crate::audit::ModuleRun {
                    module,
                    status: crate::audit::ExecutionStatus::Failed,
                    reason_code: Some("requested_module_has_no_run_record".to_string()),
                    ..Default::default()
                });
        }
    }
}

fn consolidate_module_runs(runs: &[crate::audit::ModuleRun]) -> Vec<crate::audit::ModuleRun> {
    use std::collections::BTreeMap;

    let mut grouped: BTreeMap<String, Vec<&crate::audit::ModuleRun>> = BTreeMap::new();
    for run in runs {
        grouped.entry(run.module.clone()).or_default().push(run);
    }

    grouped
        .into_iter()
        .map(|(module, items)| {
            let has_completed = items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::Completed);
            let has_failed = items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::Failed);
            let has_partial = items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::Partial);
            let status = if has_partial || (has_completed && has_failed) {
                crate::audit::ExecutionStatus::Partial
            } else if has_failed {
                crate::audit::ExecutionStatus::Failed
            } else if has_completed {
                crate::audit::ExecutionStatus::Completed
            } else if items
                .iter()
                .any(|run| run.status == crate::audit::ExecutionStatus::NotApplicable)
            {
                crate::audit::ExecutionStatus::NotApplicable
            } else {
                crate::audit::ExecutionStatus::Skipped
            };
            let mut viewports: Vec<String> = items
                .iter()
                .flat_map(|run| run.viewports.iter().cloned())
                .collect();
            viewports.sort();
            viewports.dedup();
            crate::audit::ModuleRun {
                module,
                status,
                viewports,
                subchecks: items
                    .iter()
                    .flat_map(|run| run.subchecks.iter().cloned())
                    .collect(),
                reason_code: items.iter().find_map(|run| run.reason_code.clone()),
                message: items.iter().find_map(|run| run.message.clone()),
            }
        })
        .collect()
}

fn update_audit_quality(report: &mut AuditReport) {
    let stability_budget_exhausted = report
        .accessibility
        .execution
        .navigation
        .stability
        .iter()
        .filter(|entry| {
            entry.status == crate::interaction::stability::StabilityStatus::BudgetExhausted
        })
        .count();
    let failed_rule_checks = report
        .accessibility
        .wcag_results
        .rule_outcomes
        .iter()
        .filter(|outcome| outcome.status == crate::wcag::RuleOutcomeStatus::Failed)
        .count();
    let partial_or_failed_modules = report
        .accessibility
        .execution
        .module_runs
        .iter()
        .filter(|run| {
            matches!(
                run.status,
                crate::audit::ExecutionStatus::Partial | crate::audit::ExecutionStatus::Failed
            )
        })
        .count();
    let status = if failed_rule_checks > 0 {
        crate::audit::AuditQualityStatus::Insufficient
    } else if partial_or_failed_modules > 0 || stability_budget_exhausted > 0 {
        crate::audit::AuditQualityStatus::Partial
    } else {
        crate::audit::AuditQualityStatus::Complete
    };
    let mut reasons = Vec::new();
    if failed_rule_checks > 0 {
        reasons.push(format!("failed_rule_checks:{failed_rule_checks}"));
    }
    if partial_or_failed_modules > 0 {
        reasons.push(format!(
            "partial_or_failed_modules:{partial_or_failed_modules}"
        ));
    }
    if stability_budget_exhausted > 0 {
        reasons.push(format!(
            "page_stability_budget_exhausted:{stability_budget_exhausted}"
        ));
    }
    report.accessibility.execution.quality = crate::audit::AuditQuality {
        status,
        qualified_results: status != crate::audit::AuditQualityStatus::Complete,
        failed_rule_checks,
        partial_or_failed_modules,
        reasons,
    };
}

/// Run one performance-only page load per throttle profile and return the results.
///
/// Uses the mobile viewport (most relevant for throttling scenarios).
/// Runs sequentially; errors in individual profiles are logged and skipped.
///
/// Returns the per-profile summary plus the LhMobile vitals/score as the canonical
/// throttled measurement (issue #236). LhMobile matches Lighthouse's mobile preset
/// and is used as the reported Performance score; the unthrottled desktop/mobile
/// passes remain available via `dual_viewport` for diagnostics.
async fn collect_throttled_performance(
    page: &Page,
    url: &str,
    browser: &BrowserManager,
    _config: &PipelineConfig,
    content_weight: Option<&crate::performance::ContentWeight>,
) -> (
    Vec<crate::audit::report::ThrottledPerfResult>,
    Option<(
        crate::performance::WebVitals,
        crate::performance::PerformanceScore,
    )>,
) {
    use crate::audit::report::ThrottledPerfResult;
    use crate::performance::calculate_performance_score;

    let mut results = Vec::new();
    let mut canonical: Option<(
        crate::performance::WebVitals,
        crate::performance::PerformanceScore,
    )> = None;

    for &profile in ThrottleProfile::AUTO_PROFILES {
        info!("Throttled perf pass: {:?}", profile);

        if let Err(e) = throttle::apply_throttling(page, profile).await {
            warn!("Throttle apply failed for {:?}: {}", profile, e);
            continue;
        }

        if let Err(e) = throttle::apply_cpu_throttling(page, profile).await {
            warn!("CPU throttle apply failed for {:?}: {}", profile, e);
        }

        if let Err(e) = throttle::disable_cache(page).await {
            warn!("Cache disable failed for {:?}: {}", profile, e);
        }

        if let Err(e) = prepare_vitals_collection(page).await {
            warn!("Vitals injection failed for {:?}: {}", profile, e);
            let _ = throttle::enable_cache(page).await;
            let _ = throttle::disable_throttling(page).await;
            let _ = throttle::disable_cpu_throttling(page).await;
            continue;
        }

        if let Err(e) = browser.navigate(page, url).await {
            warn!("Navigation failed for {:?}: {}", profile, e);
            let _ = throttle::enable_cache(page).await;
            let _ = throttle::disable_throttling(page).await;
            let _ = throttle::disable_cpu_throttling(page).await;
            continue;
        }

        match crate::performance::extract_web_vitals(page).await {
            Ok(vitals) => {
                // Pass the headline content_weight so the size/JS/request caps
                // apply to throttled profiles too — otherwise a throttled
                // profile can out-score the headline (and Slow3G out-score
                // Fast3G) purely because its caps were skipped (#456).
                let score = calculate_performance_score(&vitals, content_weight);
                // If LCP could not be measured under throttling (timeout or
                // navigation pre-completion), the most important navigation
                // metric is missing — do not let CLS/TBT alone push the score
                // to 100. Cap to "AUSBAUFÄHIG" tier so the profile reflects
                // that the measurement was incomplete.
                let final_score = if vitals.lcp.is_none() {
                    score.overall.min(50)
                } else {
                    score.overall
                };
                results.push(ThrottledPerfResult {
                    profile,
                    lcp_ms: vitals.lcp.as_ref().map(|v| v.value),
                    tbt_ms: vitals.tbt.as_ref().map(|v| v.value),
                    cls: vitals.cls.as_ref().map(|v| v.value),
                    score: final_score,
                });
                // LhMobile = Lighthouse mobile preset → canonical perf measurement.
                // Only adopt when LCP could actually be measured; otherwise fall
                // back to the unthrottled mobile pass so we don't report a
                // capped-to-50 score that reflects measurement failure.
                if profile == ThrottleProfile::LhMobile && vitals.lcp.is_some() {
                    let mut adopted_score = score.clone();
                    adopted_score.overall = final_score;
                    adopted_score.grade =
                        crate::performance::PerformanceGrade::from_score(final_score);
                    // The canonical report vitals come from this throttled pass;
                    // tag the direct metrics so the JSON reflects that (#406).
                    let mut throttled_vitals = vitals.clone();
                    crate::performance::mark_throttled_mobile(&mut throttled_vitals);
                    canonical = Some((throttled_vitals, adopted_score));
                }
                let _ = throttle::enable_cache(page).await;
            }
            Err(e) => {
                warn!("Vitals collection failed for {:?}: {}", profile, e);
                let _ = throttle::enable_cache(page).await;
            }
        }

        if let Err(e) = throttle::disable_throttling(page).await {
            warn!("Throttle disable failed for {:?}: {}", profile, e);
        }
        if let Err(e) = throttle::disable_cpu_throttling(page).await {
            warn!("CPU throttle disable failed for {:?}: {}", profile, e);
        }

        if let Err(e) = settle(page).await {
            warn!("Browser settle failed after {:?}: {}", profile, e);
        }
    }

    // Restore mobile viewport for screenshot capture that follows.
    if let Err(e) = set_viewport(page, Viewport::Mobile).await {
        warn!(
            "Failed to restore mobile viewport after throttled pass: {}",
            e
        );
    }

    // Restore unthrottled state for any subsequent operations.
    if !results.is_empty() {
        let _ = throttle::disable_throttling(page).await;
        let _ = throttle::disable_cpu_throttling(page).await;
    }

    (results, canonical)
}

/// Replace the report's Performance vitals/score with the LhMobile (throttled)
/// measurement and recompute viewport scores accordingly (issue #236).
///
/// Auxiliary structural data (render_blocking, content_weight, third_party,
/// critical_chain, minification, animations, coverage) stays from the unthrottled
/// pass since these describe page composition rather than timing.
fn apply_canonical_perf(
    report: &mut AuditReport,
    vitals: crate::performance::WebVitals,
    score: crate::performance::PerformanceScore,
) {
    let measurement_warnings = crate::performance::validate_metrics(&vitals);
    if let Some(ref mut perf) = report.performance {
        perf.measurement_warnings = measurement_warnings;
        perf.vitals = vitals;
        perf.score = score;
    } else {
        report.performance = Some(PerformanceResults {
            vitals,
            score,
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings,
        });
    }

    let new_perf = report.performance.as_ref().map(|p| p.score.overall);
    let mobile_seo = report.discoverability.seo.as_ref().map(|s| s.score);
    let mobile_mf = report.experience.mobile.as_ref().map(|m| m.score);
    if let Some(ref mut vps) = report.viewport_scores {
        vps.mobile.performance = new_perf;
        let mobile_overall = compute_viewport_overall(
            vps.mobile.accessibility as f32,
            vps.mobile.performance,
            mobile_seo,
            mobile_mf,
        );
        let desktop_overall = vps.desktop.overall;
        vps.mobile.overall = mobile_overall;
        vps.weighted_overall =
            (mobile_overall as f64 * 0.7 + desktop_overall as f64 * 0.3).round() as u32;
    }
}

pub(crate) fn persist_artifacts(
    url: &str,
    config: &PipelineConfig,
    snapshot: &SnapshotData,
    report: &AuditReport,
) {
    let snapshot_artifact = SnapshotArtifact {
        ax_tree: snapshot.ax_tree.clone(),
        performance: snapshot.performance.clone(),
        seo: snapshot.seo.clone(),
        security: snapshot.security.clone(),
        mobile: snapshot.mobile.clone(),
    };
    let hash = content_hash(&snapshot_artifact);
    let normalized = normalize(report).normalized;
    let wcag_level = report.wcag_level.to_string();
    let meta = CacheMeta {
        auditmysite_version: env!("CARGO_PKG_VERSION").to_string(),
        wcag_level: wcag_level.clone(),
        cached_at: report.timestamp,
        content_hash: hash.clone(),
        audit_signature: config.audit_signature(),
    };
    // Persist the full report so cache hits render every module faithfully.
    // Screenshots reference temp files that won't exist on reload, so strip them.
    let mut report_for_cache = report.clone();
    report_for_cache.page_screenshots = None;

    let artifacts = AuditArtifacts {
        fetch: FetchArtifact {
            requested_url: url.to_string(),
            final_url: report
                .accessibility
                .execution
                .navigation
                .final_url
                .clone()
                .unwrap_or_else(|| report.url.clone()),
            status_code: report
                .accessibility
                .execution
                .navigation
                .main_document_status,
            fetched_at: report.timestamp,
            duration_ms: report.duration_ms,
        },
        snapshot: snapshot_artifact,
        audit: normalized,
        report: Some(report_for_cache),
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
    use clap::Parser;

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
            stability_budget_ms: 1500,
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
            stack: false,
            reuse_cache: false,
            force_refresh: false,
            no_sitemap_suggest: false,
            prefer_sitemap: false,
            per_page_reports: false,
            dismiss_consent: false,
            interactive: crate::cli::InteractiveMode::Off,
            report_level: crate::cli::ReportLevel::Standard,
            lang: "de".to_string(),
            also_json: false,
            logo: None,
            debug_typ: false,
            export_snapshot: None,
            annex: None,
            request_mode: crate::cli::RequestMode::Browser,
            report_mode: false,
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
    fn active_module_labels_follow_pipeline_flags() {
        let config =
            PipelineConfig::from(&Args::parse_from(["auditmysite", "https://example.com"]));
        let expected = vec![
            "Accessibility",
            "Accessibility Journey",
            "Best Practices",
            "Dark Mode",
            "Journey",
            "Mobile",
            "Performance",
            "Security",
            "SEO",
            "AI Visibility",
            "Tech Stack",
            "Commerce",
            "UX",
            "Source Quality",
            "Content Visibility",
        ];
        assert_eq!(config.active_module_labels(), expected);
    }

    #[test]
    fn active_module_labels_respect_disabled_full_modules() {
        let config = PipelineConfig::from(&Args::parse_from([
            "auditmysite",
            "https://example.com",
            "--skip-performance",
            "--skip-mobile",
        ]));
        let expected = vec![
            "Accessibility",
            "Accessibility Journey",
            "Dark Mode",
            "AI Visibility",
            "Source Quality",
        ];
        assert_eq!(config.active_module_labels(), expected);
    }

    fn test_pipeline_config() -> PipelineConfig {
        PipelineConfig {
            wcag_level: WcagLevel::AA,
            timeout_secs: 30,
            stability_budget_ms: 1500,
            verbose: false,
            full_audit: false,
            check_performance: true,
            check_seo: true,
            check_security: false,
            check_mobile: true,
            check_dark_mode: true,
            check_stack: false,
            persist_artifacts: true,
            capture_screenshots: false,
            capture_element_evidence: false,
            dismiss_consent: false,
            interactive: crate::cli::InteractiveMode::Off,
            journey_budget_ms: crate::a11y_journey::DEFAULT_BUDGET_MS,
            lang: "de".to_string(),
        }
    }

    #[test]
    fn audit_signature_changes_with_audit_relevant_options() {
        let base = test_pipeline_config();
        let base_sig = base.audit_signature();

        // WCAG level changes the signature.
        let mut other = test_pipeline_config();
        other.wcag_level = WcagLevel::AAA;
        assert_ne!(base_sig, other.audit_signature());

        // Toggling a module changes the signature.
        let mut other = test_pipeline_config();
        other.check_security = true;
        assert_ne!(base_sig, other.audit_signature());

        // Consent handling changes the signature.
        let mut other = test_pipeline_config();
        other.dismiss_consent = true;
        assert_ne!(base_sig, other.audit_signature());

        // Interactive budget changes the possible journey findings.
        let mut other = test_pipeline_config();
        other.journey_budget_ms += 1;
        assert_ne!(base_sig, other.audit_signature());

        // Output language drives locale-dependent detection/text in the cached
        // report, so it must invalidate the cache (#405).
        let mut other = test_pipeline_config();
        other.lang = "en".to_string();
        assert_ne!(base_sig, other.audit_signature());
    }

    #[test]
    fn audit_signature_ignores_non_content_options() {
        let base = test_pipeline_config();
        let base_sig = base.audit_signature();

        // Timeout, verbosity, persistence and screenshots do not affect findings.
        let mut other = test_pipeline_config();
        other.timeout_secs = 120;
        other.verbose = true;
        other.persist_artifacts = false;
        other.capture_screenshots = true;
        other.capture_element_evidence = true;
        assert_eq!(base_sig, other.audit_signature());
    }

    #[test]
    fn pipeline_config_uses_interactive_budget_from_config() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let config: crate::cli::config::Config = toml::from_str(
            r#"
[interactive]
journey_budget_ms = 1234
"#,
        )
        .unwrap();

        let pipeline = PipelineConfig::from_args_and_config(&args, Some(&config));

        assert_eq!(pipeline.journey_budget_ms, 1234);
    }

    #[test]
    fn for_viewport_desktop_disables_seo_security_mobile_stack() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "--full"]);
        let config = PipelineConfig::from_args_and_config(&args, None);
        let desktop = config.for_viewport(Viewport::Desktop);

        assert!(!desktop.check_seo);
        assert!(!desktop.check_security);
        assert!(!desktop.check_mobile);
        assert!(!desktop.check_stack);
        assert_eq!(desktop.check_dark_mode, config.check_dark_mode);
        assert_eq!(desktop.check_performance, config.check_performance);
    }

    #[test]
    fn for_viewport_desktop_respects_disabled_dark_mode() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);
        let mut config = PipelineConfig::from_args_and_config(&args, None);
        config.check_dark_mode = false;

        let desktop = config.for_viewport(Viewport::Desktop);

        assert!(!desktop.check_dark_mode);
    }

    #[test]
    fn for_viewport_mobile_disables_security_and_dark_mode() {
        let args = Args::parse_from(["auditmysite", "https://example.com", "--full"]);
        let config = PipelineConfig::from_args_and_config(&args, None);
        let mobile = config.for_viewport(Viewport::Mobile);

        assert!(!mobile.check_security);
        assert!(!mobile.check_dark_mode);
        assert_eq!(mobile.check_seo, config.check_seo);
        assert_eq!(mobile.check_mobile, config.check_mobile);
        assert_eq!(mobile.check_stack, config.check_stack);
    }

    #[test]
    fn pipeline_config_uses_default_interactive_budget_without_config() {
        let args = Args::parse_from(["auditmysite", "https://example.com"]);

        let pipeline = PipelineConfig::from_args_and_config(&args, None);

        assert_eq!(
            pipeline.journey_budget_ms,
            crate::a11y_journey::DEFAULT_BUDGET_MS
        );
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
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 5,
            incomplete: 0,
            nodes_checked: 100,
            rule_outcomes: vec![],
        };
        let mobile = WcagResults {
            violations: vec![
                make_v("1.1.1", "#img1"),  // shared
                make_v("1.4.3", "#text1"), // mobile-only
            ],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 4,
            incomplete: 0,
            nodes_checked: 90,
            rule_outcomes: vec![],
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
    fn test_merge_wcag_violations_empty_desktop() {
        use crate::wcag::Severity;

        fn make_v(rule: &str, selector: &str) -> Violation {
            let mut v = Violation::new(rule, rule, WcagLevel::A, Severity::High, "msg", "node-1");
            v.selector = Some(selector.to_string());
            v
        }

        let desktop = WcagResults {
            violations: vec![],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 0,
            incomplete: 0,
            nodes_checked: 0,
            rule_outcomes: vec![],
        };
        let mobile = WcagResults {
            violations: vec![make_v("1.1.1", "#img1"), make_v("1.4.3", "#text1")],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 3,
            incomplete: 0,
            nodes_checked: 50,
            rule_outcomes: vec![],
        };

        let merged = merge_wcag_violations(&desktop, &mobile);
        assert_eq!(merged.violations.len(), 2);
        assert!(merged
            .violations
            .iter()
            .all(|v| v.tags.contains(&"mobile-only".to_string())));
        assert_eq!(merged.passes, 3);
        assert_eq!(merged.nodes_checked, 50);
    }

    #[test]
    fn test_merge_wcag_violations_empty_mobile() {
        use crate::wcag::Severity;

        fn make_v(rule: &str, selector: &str) -> Violation {
            let mut v = Violation::new(rule, rule, WcagLevel::A, Severity::High, "msg", "node-1");
            v.selector = Some(selector.to_string());
            v
        }

        let desktop = WcagResults {
            violations: vec![make_v("2.4.4", "#link1")],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 7,
            incomplete: 0,
            nodes_checked: 80,
            rule_outcomes: vec![],
        };
        let mobile = WcagResults {
            violations: vec![],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 4,
            incomplete: 0,
            nodes_checked: 60,
            rule_outcomes: vec![],
        };

        let merged = merge_wcag_violations(&desktop, &mobile);
        assert_eq!(merged.violations.len(), 1);
        assert!(merged
            .violations
            .iter()
            .all(|v| v.tags.contains(&"desktop-only".to_string())));
        assert_eq!(merged.passes, 7); // max
        assert_eq!(merged.nodes_checked, 80); // max
    }

    #[test]
    fn test_merge_wcag_violations_contradictory_severity_takes_mobile() {
        use crate::wcag::Severity;

        let mut desktop_v = Violation::new(
            "1.4.3",
            "Contrast",
            WcagLevel::AA,
            Severity::Critical,
            "msg",
            "node-1",
        );
        desktop_v.selector = Some("#text1".to_string());

        let mut mobile_v = Violation::new(
            "1.4.3",
            "Contrast",
            WcagLevel::AA,
            Severity::High,
            "msg",
            "node-1",
        );
        mobile_v.selector = Some("#text1".to_string());

        let desktop = WcagResults {
            violations: vec![desktop_v],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 0,
            incomplete: 0,
            nodes_checked: 0,
            rule_outcomes: vec![],
        };
        let mobile = WcagResults {
            violations: vec![mobile_v],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 0,
            incomplete: 0,
            nodes_checked: 0,
            rule_outcomes: vec![],
        };

        let merged = merge_wcag_violations(&desktop, &mobile);
        assert_eq!(merged.violations.len(), 1);
        let v = &merged.violations[0];
        // Shared violations take the mobile clone (mobile variant wins).
        assert_eq!(v.severity, Severity::High);
        assert!(v.tags.contains(&"both-viewports".to_string()));
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

    #[test]
    fn failed_page_rule_is_not_reported_as_clean() {
        let marker = crate::wcag::technical_rule_failure_for(
            "reflow",
            WcagLevel::AA,
            "dom_evaluation_failed",
        );
        let (outcome, visible) =
            page_rule_outcome("reflow", Some("1.4.10"), "mobile", vec![marker]);

        assert!(visible.is_empty());
        assert_eq!(outcome.status, crate::wcag::RuleOutcomeStatus::Failed);
        assert_eq!(outcome.viewport.as_deref(), Some("mobile"));
        assert_eq!(
            outcome.reason_code.as_deref(),
            Some("dom_evaluation_failed")
        );
    }

    #[test]
    fn missing_requested_module_qualifies_audit() {
        let mut results = WcagResults::new();
        results.rule_outcomes.push(crate::wcag::RuleOutcome {
            rule_id: "image_alt".to_string(),
            status: crate::wcag::RuleOutcomeStatus::NoViolationDetected,
            wcag_criterion: Some("1.1.1".to_string()),
            viewport: Some("desktop".to_string()),
            reason_code: None,
            finding_count: 0,
        });
        let mut report =
            AuditReport::new("https://example.com".to_string(), WcagLevel::AA, results, 1);
        report.accessibility.execution.scope.requested_modules =
            vec!["accessibility".to_string(), "seo".to_string()];

        ensure_requested_module_runs(&mut report);
        update_audit_quality(&mut report);

        assert_eq!(
            report
                .accessibility
                .execution
                .module_runs
                .iter()
                .find(|run| run.module == "accessibility")
                .unwrap()
                .status,
            crate::audit::ExecutionStatus::Completed
        );
        assert_eq!(
            report
                .accessibility
                .execution
                .module_runs
                .iter()
                .find(|run| run.module == "seo")
                .unwrap()
                .status,
            crate::audit::ExecutionStatus::Failed
        );
        assert!(report.accessibility.execution.quality.qualified_results);
    }

    #[test]
    fn missing_throttled_profile_marks_performance_partial() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            1,
        );
        report
            .accessibility
            .execution
            .module_runs
            .push(crate::audit::ModuleRun {
                module: "performance".to_string(),
                status: crate::audit::ExecutionStatus::Completed,
                ..Default::default()
            });

        attach_throttled_profile_subchecks(&mut report, &[]);

        let performance = report
            .accessibility
            .execution
            .module_runs
            .iter()
            .find(|run| run.module == "performance")
            .unwrap();
        assert_eq!(performance.status, crate::audit::ExecutionStatus::Partial);
        assert_eq!(
            performance.subchecks.len(),
            ThrottleProfile::AUTO_PROFILES.len()
        );
        assert!(performance
            .subchecks
            .iter()
            .all(|check| check.status == crate::audit::ExecutionStatus::Failed));
    }
}
