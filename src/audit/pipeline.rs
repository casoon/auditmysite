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
use crate::interaction::stability::settle;
use crate::journey::JourneyAnalysis;
use crate::mobile::MobileFriendliness;
use crate::performance::{prepare_coverage_collection, prepare_vitals_collection};
use crate::security::{analyze_security, BrowserCertificateDetails, SecurityAnalysis};
use crate::seo::SeoAnalysis;
use crate::ux::UxAnalysis;
use crate::wcag::{self, Violation, WcagResults};

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
}

impl SnapshotData {
    /// Route a `ModuleData` payload into the matching snapshot field.
    ///
    /// Called once per collected module after `catalog.collect_all()`. Keeping
    /// the routing here means adding a new module only requires updating
    /// `SnapshotData` and this method — not the pipeline loop body.
    fn apply_module_data(&mut self, data: ModuleData) {
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
    /// Run tech stack detection and stack-specific audits
    pub check_stack: bool,
    /// Persist audit artifacts under ~/.auditmysite/cache
    pub persist_artifacts: bool,
    /// Capture desktop + mobile screenshots for PDF cover page
    pub capture_screenshots: bool,
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
        // 6 for the commerce trust-pages restructure (product/trust_pages split).
        const CACHE_FMT: u8 = 6;
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
            verbose: args.verbose,
            check_performance: (full_audit || args.performance) && !args.skip_performance,
            check_seo: full_audit || args.seo,
            check_security: full_audit || args.security,
            check_mobile: (full_audit || args.mobile) && !args.skip_mobile,
            check_dark_mode: true,
            check_stack: full_audit || args.stack,
            persist_artifacts: true,
            capture_screenshots: args.url.is_some()
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
    let desktop_wcag = run_rules(page, &desktop_snap, config, desktop_screenshot.as_ref()).await;

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
    consent_result.banner_detected |= mobile_consent.banner_detected;
    if consent_result.cmp_name.is_none() {
        consent_result.cmp_name = mobile_consent.cmp_name;
    }
    consent_result.dismissed |= mobile_consent.dismissed;

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
    let mut mobile_wcag = run_rules(page, &mobile_snap, config, mobile_screenshot.as_ref()).await;

    // 1.4.10 Reflow — temporarily sets viewport to 320×256, then restores mobile
    if matches!(config.wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        info!("Running reflow check at 320 CSS px...");
        let reflow_violations = wcag::check_reflow_with_page(page).await;
        if !reflow_violations.is_empty() {
            info!("Found reflow violation at 320px");
        }
        mobile_wcag.extend_findings(reflow_violations);
        // check_reflow_with_page leaves the viewport at 320px — restore mobile
        let _ = set_viewport(page, Viewport::Mobile).await;
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
    };

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
    report.dual_viewport = Some(dual_viewport);
    report.viewport_scores = Some(viewport_scores);

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
    };
    match crate::a11y_journey::run(journey_ctx).await {
        Ok(Some(out)) => {
            report.accessibility_journey = Some(out.journey);
            report.interactive_findings = out.findings;
        }
        Ok(None) => {}
        Err(e) => warn!("Accessibility-Journey-Layer failed: {}", e),
    }

    // Persistence is deferred to the caller: the single-page path applies the
    // canonical (LhMobile) performance pass *after* audit_page returns, so
    // persisting here would cache a pre-canonical report that differs from a
    // fresh render (#404). The caller persists once the report is final.
    Ok((report, primary_snap))
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

    // Merge warnings, positives, and not_testables without deduplication
    // (heuristic/structural signals — viewport tagging not meaningful).
    let mut warnings = mobile.warnings.clone();
    warnings.extend(desktop.warnings.iter().cloned());
    warnings.dedup_by(|a, b| a.message == b.message);

    let mut positives = mobile.positives.clone();
    positives.extend(desktop.positives.iter().cloned());
    positives.dedup_by(|a, b| a.message == b.message);

    let mut not_testables = mobile.not_testables.clone();
    not_testables.extend(desktop.not_testables.iter().cloned());
    not_testables.dedup_by(|a, b| a.message == b.message);

    WcagResults {
        violations: merged,
        warnings,
        positives,
        not_testables,
        passes: mobile.passes.max(desktop.passes),
        incomplete: mobile.incomplete.max(desktop.incomplete),
        nodes_checked: mobile.nodes_checked.max(desktop.nodes_checked),
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
    };

    for (_id, data) in collected {
        snapshot.apply_module_data(data);
    }

    Ok(snapshot)
}

async fn run_rules(
    page: &Page,
    snapshot: &SnapshotData,
    config: &PipelineConfig,
    screenshot: Option<&ViewportScreenshot>,
) -> WcagResults {
    debug!("Running WCAG checks at level {}...", config.wcag_level);
    let mut wcag_results = wcag::check_all(&snapshot.ax_tree, config.wcag_level);
    apply_lang_attribute_check(page, &mut wcag_results).await;

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
        info!("Found {} contrast violations", contrast_violations.len());
        wcag_results.extend_findings(contrast_violations);
    }

    // Table-driven page rules (#334). min_level gates each entry.
    for rule in wcag::rules::PAGE_RULES {
        if config.wcag_level < rule.min_level {
            continue;
        }
        let findings = (rule.check_fn)(page).await;
        if !findings.is_empty() {
            info!("Found {} {} violations", findings.len(), rule.name);
        }
        wcag_results.extend_findings(findings);
    }

    enrich_violations_with_page(page, &mut wcag_results.violations, &snapshot.ax_tree).await;
    move_demoted_violations_to_warnings(&mut wcag_results);
    wcag_results
}

/// 3.1.1 verifying-subtraction: the AX tree does not expose `html[lang]`,
/// so if check_all emitted a 3.1.1 violation, query the DOM and remove it
/// when a valid lang attribute is present.
async fn apply_lang_attribute_check(page: &Page, results: &mut WcagResults) {
    if !results.violations.iter().any(|v| v.rule == "3.1.1") {
        return;
    }
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
        results.violations.retain(|v| v.rule != "3.1.1");
        results.passes += 1;
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
    report = report.with_patterns(pattern_analysis);
    report.screen_reader_audit = Some(crate::screen_reader::build_sr_audit_report(
        url,
        report.timestamp,
        &snapshot.ax_tree,
        &config.lang,
    ));

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

    report
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
    let _ = set_viewport(page, Viewport::Mobile).await;

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
            final_url: report.url.clone(),
            status_code: None,
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
            request_mode: crate::cli::RequestMode::Browser,
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
            verbose: false,
            check_performance: true,
            check_seo: true,
            check_security: false,
            check_mobile: true,
            check_dark_mode: true,
            check_stack: false,
            persist_artifacts: true,
            capture_screenshots: false,
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
        };
        let mobile = WcagResults {
            violations: vec![make_v("1.1.1", "#img1"), make_v("1.4.3", "#text1")],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 3,
            incomplete: 0,
            nodes_checked: 50,
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
        };
        let mobile = WcagResults {
            violations: vec![],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 4,
            incomplete: 0,
            nodes_checked: 60,
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
        };
        let mobile = WcagResults {
            violations: vec![mobile_v],
            warnings: vec![],
            positives: vec![],
            not_testables: vec![],
            passes: 0,
            incomplete: 0,
            nodes_checked: 0,
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
}
