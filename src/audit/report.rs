//! Audit Report data structure
//!
//! Contains the complete results of an accessibility audit.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::scoring::{AccessibilityScorer, ViolationStatistics};
use crate::browser::ThrottleProfile;
use crate::cli::WcagLevel;
use crate::dark_mode::DarkModeAnalysis;
use crate::mobile::MobileFriendliness;
use crate::performance::{
    AnimationAnalysis, ContentWeight, CoverageAnalysis, CriticalChain, MinificationAnalysis,
    PerformanceScore, RenderBlockingAnalysis, ThirdPartyAttribution, WebVitals,
};
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::ux::UxAnalysis;
use crate::wcag::WcagResults;

/// Execution state shared by modules, subchecks and the overall audit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Completed,
    Partial,
    Failed,
    Skipped,
    NotApplicable,
    #[default]
    NotRequested,
}

/// One recorded module or sub-analysis run. `reason_code` is stable and safe
/// for public JSON; `message` is deliberately short and sanitized.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleRun {
    pub module: String,
    pub status: ExecutionStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub viewports: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subchecks: Vec<SubcheckRun>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// One sub-analysis or collector of a module run. Collectors under the
/// collection contract (#777) also carry their `state`, method and version;
/// `scoring` marks collections the module score depends on.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubcheckRun {
    pub subcheck: String,
    pub status: ExecutionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<super::collection::CollectionState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<super::collection::CollectionMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_version: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub scoring: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<String>,
}

impl SubcheckRun {
    /// A scoring collection that lost (part of) its measurement.
    pub fn is_scoring_gap(&self) -> bool {
        self.scoring && self.state.is_some_and(|state| state.is_gap())
    }
}

impl From<super::collection::CollectionRecord> for SubcheckRun {
    fn from(record: super::collection::CollectionRecord) -> Self {
        Self {
            subcheck: record.collector,
            status: record.state.execution_status(),
            reason_code: record.reason_code,
            state: Some(record.state),
            method: Some(record.method),
            method_version: Some(record.method_version),
            scoring: record.scoring,
            gaps: record.gaps,
        }
    }
}

/// A rule the run is expected to account for, and where (#778).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedRule {
    pub rule_id: String,
    pub viewports: Vec<String>,
}

/// Requested audit scope, independent of whether the measurements succeeded.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditScope {
    #[serde(default)]
    pub requested_modules: Vec<String>,
    /// Opt-in extra checks requested on top of the modules, by collector id
    /// (`performance.third_party_isolation`, `content_visibility.ssr_gap`).
    /// Each carries a subcheck run in its module's `module_runs` (#788).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_subchecks: Vec<String>,
    /// Rules this run must account for with exactly one outcome per listed
    /// viewport -- ran, or not run with a reason (#778). Derived from the
    /// rule registrations (`wcag::expectation`). Empty for reports from
    /// before the field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_rules: Vec<RequestedRule>,
    /// WCAG level the run was configured for (`A`, `AA`, `AAA`); empty for
    /// reports from before the field. A rule above it must not have run.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub wcag_level: String,
    pub full_audit: bool,
    /// Measurement profile: under which session policy the pages ran and
    /// which journeys may change page or account state (#787).
    #[serde(default)]
    pub measurement_profile: MeasurementProfile,
    pub interactive_mode: String,
    pub journey_budget_ms: u64,
    #[serde(default)]
    pub throttling_profiles: Vec<String>,
    #[serde(default)]
    pub viewports: Vec<ViewportDefinition>,
    pub dismiss_consent: bool,
    pub capture_screenshots: bool,
    pub capture_element_evidence: bool,
    /// Display mode the scores belong to (#653). Reports from before
    /// `--display` default to `site_default`, which is what they measured.
    #[serde(default)]
    pub display_mode: crate::display::AuditedDisplayMode,
}

/// Version of the measurement profile's meaning. A report of another
/// version is another contract, never the same measurement.
pub const MEASUREMENT_PROFILE_VERSION: u32 = 2;

/// How the pages of a run were measured beyond the viewport (#787).
///
/// `read_only` runs observe; a run with `state_changing_journeys` also
/// clicks controls that can submit a form or change a cart. The distinction
/// is stored with the result so it is never an invisible side effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasurementProfile {
    pub version: u32,
    /// Cookie/storage/cache sharing between the pages of the run:
    /// `isolated` or `shared`. `unknown` for reports from before the field.
    pub session_policy: String,
    /// True when no journey that can change page or account state may run.
    pub read_only: bool,
    /// Journeys permitted to change state (submit clicks, cart triggers).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub state_changing_journeys: Vec<String>,
    /// Name of the declared state the run started from (`--state-id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_id: Option<String>,
    /// The declared browser state by identity: digest and counts, never
    /// values. `None` when every page started from an empty profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_state: Option<StorageStateRef>,
    /// Opt-in extra checks the run asked for (flag names from the support
    /// matrix, e.g. `--check-ssr-content`): extra reloads, JavaScript
    /// disabled, DNS lookups. Part of the profile because they change what
    /// was measured; sorted, empty when none was requested (#788).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_checks: Vec<String>,
}

/// Identity of a declared browser state (`--storage-state`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageStateRef {
    /// SHA-256 of the state file, hex.
    pub sha256: String,
    pub cookies: usize,
    pub origins: usize,
}

impl Default for MeasurementProfile {
    fn default() -> Self {
        Self {
            version: 0,
            session_policy: "unknown".to_string(),
            read_only: false,
            state_changing_journeys: Vec::new(),
            state_id: None,
            storage_state: None,
            extra_checks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportDefinition {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
}

/// Public, reproducibility-oriented browser information. Executable paths and
/// launch flags intentionally stay out of the report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_version: Option<String>,
    pub headless: bool,
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavigationSnapshot {
    pub requested_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_document_status: Option<u16>,
    #[serde(default)]
    pub redirect_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_state: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stability: Vec<crate::interaction::stability::StabilityProvenance>,
    /// Short meta refreshes met while loading, per viewport pass (Plan 47).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meta_refresh: Vec<ViewportMetaRefresh>,
    /// HTTP redirects the main document went through, in order, ending
    /// before `final_url` (#775). Empty when it loaded without a redirect
    /// or the chain was not captured (`redirect_count` still says how many).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redirect_chain: Vec<RedirectHop>,
}

/// One redirect response in the main document's chain (#775).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedirectHop {
    /// The URL that answered with a redirect.
    pub url: String,
    pub status: u16,
}

impl NavigationSnapshot {
    /// The URL the document was actually served from. `None` when the
    /// navigation did not report one (older reports, failed evaluation).
    pub fn served_url(&self) -> Option<&str> {
        self.final_url.as_deref().filter(|u| !u.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportMetaRefresh {
    pub viewport: String,
    #[serde(flatten)]
    pub refresh: crate::browser::MetaRefresh,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditedContentState {
    BeforeConsent,
    AfterConsent,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsentAuditState {
    pub detected: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmp: Option<String>,
    pub dismissal_requested: bool,
    pub dismissed: bool,
    pub audited_content_state: AuditedContentState,
    #[serde(default)]
    pub status: Option<crate::browser::consent::ConsentHandlingStatus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditQualityStatus {
    #[default]
    Complete,
    Partial,
    Insufficient,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQuality {
    pub status: AuditQualityStatus,
    pub qualified_results: bool,
    #[serde(default)]
    pub failed_rule_checks: usize,
    #[serde(default)]
    pub partial_or_failed_modules: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

/// Complete execution/provenance block persisted with the raw audit report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditExecution {
    #[serde(default)]
    pub scope: AuditScope,
    #[serde(default)]
    pub environment: ExecutionEnvironment,
    #[serde(default)]
    pub navigation: NavigationSnapshot,
    #[serde(default)]
    pub consent: ConsentAuditState,
    #[serde(default)]
    pub module_runs: Vec<ModuleRun>,
    #[serde(default)]
    pub quality: AuditQuality,
    /// Applied audit exclusions and what they removed (#645).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusions: Option<crate::audit::exclusion::ExclusionReport>,
    /// Display-mode convention detected on the page (#653). Absent when the
    /// page neither sets `html[data-display]` nor has a `figure[data-viz]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_modes: Option<crate::display::DisplayModesInfo>,
    /// Which iframes were audited with the element-level rules and which
    /// were skipped, and why (#715). Absent in reports from before the frame
    /// pass; present and empty for a page without frames.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<crate::audit::frames::FrameCoverage>,
}

/// Performance vitals measured under a single network throttle profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottledPerfResult {
    pub profile: ThrottleProfile,
    /// LCP in milliseconds
    pub lcp_ms: Option<f64>,
    /// TBT in milliseconds
    pub tbt_ms: Option<f64>,
    /// CLS score
    pub cls: Option<f64>,
    /// Aggregate performance score (0–100); `None` when the profile did not
    /// measure all four core metrics.
    pub score: Option<u32>,
    /// Spread over the repeated runs of this profile (`--perf-runs N`, N > 1);
    /// the fields above are then the median run. `None` for a single run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread: Option<RunSpread>,
}

/// Min / median / max of one metric over the repeated runs of a profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSpread {
    pub min: f64,
    pub median: f64,
    pub max: f64,
}

/// What repeated performance runs of one throttle profile measured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSpread {
    /// Runs that produced a measurement.
    pub runs: u32,
    /// Runs asked for (`--perf-runs`); fewer `runs` means some failed.
    pub requested_runs: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lcp_ms: Option<MetricSpread>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tbt_ms: Option<MetricSpread>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cls: Option<MetricSpread>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<MetricSpread>,
}

/// Screenshot bytes captured during the audit (desktop + mobile viewports).
/// Not serialized — only used for PDF output.
#[derive(Debug, Clone)]
pub struct PageScreenshots {
    pub desktop: Vec<u8>,
    pub mobile: Vec<u8>,
}

/// Cookie metadata snapshot around consent interaction. Cookie values are never stored.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsentPrivacySnapshot {
    pub before_interaction: Vec<ConsentCookieSignal>,
    pub after_interaction: Vec<ConsentCookieSignal>,
    pub added_after_interaction: Vec<ConsentCookieSignal>,
}

/// Cookie signal stored for consent/privacy comparison without cookie values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConsentCookieSignal {
    pub name: String,
    pub domain: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Why device-preview screenshots are or are not available in this report.
/// Used by the PDF renderer to surface the right callout when screenshots
/// are missing (failure reason vs. not-requested).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "state", content = "reason")]
pub enum ScreenshotStatus {
    /// Screenshots were captured successfully.
    Captured,
    /// Capture was attempted but failed; the string holds a short reason.
    Failed(String),
    /// Capture was not attempted for this audit (e.g. batch mode).
    #[default]
    NotRequested,
}

/// Viewport screenshot with layout metadata
#[derive(Debug, Clone)]
pub struct ViewportScreenshot {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
    pub scroll_x: f64,
    pub scroll_y: f64,
}

/// Raw audit data for a single viewport pass.
/// Not serialized — kept in-memory for output builders.
#[derive(Debug, Clone)]
pub struct ViewportAuditData {
    pub wcag_results: WcagResults,
    pub accessibility_score: f32,
    pub performance: Option<PerformanceResults>,
    pub seo: Option<SeoAnalysis>,
    pub mobile: Option<MobileFriendliness>,
    pub ux: Option<crate::ux::UxAnalysis>,
    pub journey: Option<crate::journey::JourneyAnalysis>,
    pub screenshot: Option<ViewportScreenshot>,
    pub module_runs: Vec<ModuleRun>,
}

/// Dual-viewport raw results — desktop and mobile passes.
/// Not serialized — kept in-memory only.
#[derive(Debug, Clone)]
pub struct DualViewportResults {
    pub desktop: ViewportAuditData,
    pub mobile: ViewportAuditData,
}

/// Per-viewport scores for JSON / CLI output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportScoreSet {
    pub accessibility: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<u32>,
    pub overall: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportScores {
    pub desktop: ViewportScoreSet,
    pub mobile: ViewportScoreSet,
    /// 70 % mobile + 30 % desktop (before security adjustment)
    pub weighted_overall: u32,
}

impl ViewportScores {
    /// Canonical accessibility score for a dual-viewport audit.
    ///
    /// The calculation uses the same displayed integer viewport scores that
    /// are serialized to JSON, so readers can reproduce the result exactly.
    pub fn weighted_accessibility(&self) -> u32 {
        (self.mobile.accessibility as f64 * 0.7 + self.desktop.accessibility as f64 * 0.3).round()
            as u32
    }
}

/// User-experience signals grouped together: mobile friendliness, dark-mode
/// support and performance-budget violations. First of the planned `AuditReport`
/// section structs that replace the flat list of optional module fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperienceSection {
    /// Mobile friendliness analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<MobileFriendliness>,
    /// Dark mode support and quality analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_mode: Option<DarkModeAnalysis>,
    /// Opt-in design-quality heuristics (#528, optional). Never affects
    /// accessibility/overall score, grade, or certificate.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub design_quality: Option<crate::design_quality::DesignQualityAnalysis>,
    /// Opt-in C2PA image-provenance check (EU AI Act Art. 50, optional,
    /// single-URL mode only). Never affects accessibility/overall score,
    /// grade, or certificate.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ai_transparency: Option<crate::ai_transparency::AiTransparencyAnalysis>,
    /// Opt-in DNS-configuration check (CAA, DNSSEC, SPF/MX — #545, optional).
    /// Host-scoped rather than page-scoped (see `network::dns::module`), but
    /// still attached per-page here like the other opt-in modules. Never
    /// affects accessibility/overall score, grade, or certificate.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub network_dns: Option<crate::network::dns::NetworkDnsAnalysis>,
    /// Budget violations detected for this page
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub budget_violations: Vec<crate::audit::budget::BudgetViolation>,
}

/// Core accessibility results — the WCAG check output, the resulting score/grade/
/// certificate, violation statistics and the analyzed node count. Grouped into a
/// section so the `AuditReport` top level stays stable as modules evolve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySection {
    /// WCAG check results
    pub wcag_results: WcagResults,
    /// Overall accessibility score (0-100)
    pub score: f32,
    /// Letter grade (A-F)
    pub grade: String,
    /// Certificate level (SEHR GUT, GUT, STABIL, AUSBAUFÄHIG, UNGENÜGEND)
    pub certificate: String,
    /// Detailed violation statistics
    pub statistics: ViolationStatistics,
    /// Number of AXTree nodes analyzed
    pub nodes_analyzed: usize,
    /// Audit execution scope, provenance and completeness. Kept alongside the
    /// core result so older raw-report caches can default it safely.
    #[serde(default)]
    pub execution: AuditExecution,
}

/// Discoverability signals grouped together: how findable and machine-readable
/// the page is — search engines (SEO), AI/LLM assistants, organic content
/// visibility, source quality and the detected tech stack.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoverabilitySection {
    /// SEO analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo: Option<SeoAnalysis>,
    /// AI visibility analysis (LLM-Readability, Citation, Chunks, Knowledge Graph, Policy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_visibility: Option<crate::ai_visibility::AiVisibilityAnalysis>,
    /// Content visibility analysis (organic visibility, local business, E-E-A-T, depth, topical authority)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_visibility: Option<crate::content_visibility::ContentVisibilityAnalysis>,
    /// Source quality analysis (Substanz / Konsistenz / Autorität)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_quality: Option<crate::source_quality::SourceQualityAnalysis>,
    /// Technology stack detection and stack-specific audit findings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_stack: Option<crate::tech_stack::TechStackAnalysis>,
}

/// Complete audit report for a single URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// The URL that was audited
    pub url: String,
    /// WCAG conformance level used for the audit
    pub wcag_level: WcagLevel,
    /// Timestamp when the audit was performed
    pub timestamp: DateTime<Utc>,
    /// Core accessibility results (WCAG output, score/grade/certificate, stats).
    pub accessibility: AccessibilitySection,
    /// Time taken to complete the audit (milliseconds)
    pub duration_ms: u64,
    /// Performance analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceResults>,
    /// Security analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityAnalysis>,
    /// HTML5 spec-conformance analysis (optional) — URL/page-level, not
    /// per-viewport, same placement rationale as `security`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_conform: Option<crate::html_conform::HtmlConformAnalysis>,
    /// User-experience signals grouped into one section: mobile friendliness,
    /// dark-mode support and performance-budget violations.
    #[serde(default)]
    pub experience: ExperienceSection,
    /// UX analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux: Option<UxAnalysis>,
    /// Journey analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey: Option<crate::journey::JourneyAnalysis>,
    /// Discoverability signals grouped together: SEO, AI/LLM visibility, content
    /// visibility, source quality and tech-stack detection.
    #[serde(default)]
    pub discoverability: DiscoverabilitySection,
    /// Screenshots for PDF cover page (captured during audit, not serialized).
    #[serde(skip)]
    pub page_screenshots: Option<PageScreenshots>,
    /// Raw dual-viewport data (not serialized — in-memory only for output builders).
    #[serde(skip)]
    pub dual_viewport: Option<DualViewportResults>,
    /// Per-viewport scores (serialized for JSON consumers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_scores: Option<ViewportScores>,
    /// Performance vitals measured under different network throttle profiles.
    /// Only populated for single-page audits when performance analysis is enabled.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub throttled_performance: Vec<ThrottledPerfResult>,
    /// Structural UI pattern detection results (MainNavigation, SkipLink, etc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns: Option<crate::patterns::PatternAnalysis>,
    /// Device preview screenshot capture state (issue #26).
    #[serde(default)]
    pub screenshot_status: ScreenshotStatus,
    /// Best practices analysis (console errors, vulnerable libraries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_practices: Option<crate::best_practices::BestPracticesAnalysis>,
    /// Commerce / shop schema-completeness analysis (only on product pages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commerce: Option<crate::commerce::CommerceAnalysis>,
    /// Whether a consent banner was detected during the audit
    #[serde(default)]
    pub consent_banner_detected: bool,
    /// Which CMP was identified (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_banner_cmp: Option<String>,
    /// Whether the banner was successfully dismissed
    #[serde(default)]
    pub consent_banner_dismissed: bool,
    /// Cookie snapshot before/after consent interaction; values are never stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_privacy: Option<ConsentPrivacySnapshot>,
    /// Accessibility-Journey-Layer result (populated when `--interactive != off`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility_journey: Option<crate::audit::normalized::AccessibilityJourney>,
    /// Interactive findings produced by the Accessibility-Journey-Layer
    /// evaluator. Empty when `--interactive=off`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactive_findings: Vec<crate::audit::normalized::InteractiveFinding>,
    /// Standalone screen-reader audit artifact. Written as sidecar JSON output.
    #[serde(skip)]
    pub screen_reader_audit: Option<crate::screen_reader::SrAuditReport>,
    /// What the run judges against: declared or implied page goals and the
    /// verdict profile (#780, #781). Normalization derives findings and the
    /// decision from it, so every output of this report agrees.
    #[serde(default)]
    pub run_context: RunContext,
}

/// Declared inputs of the judgement, stored with the raw report.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunContext {
    #[serde(default)]
    pub indexability: crate::audit::finding_contract::IndexabilityExpectation,
    #[serde(default)]
    pub decision_profile: crate::audit::verdict::DecisionProfile,
}

/// Performance analysis results wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResults {
    /// Core Web Vitals
    pub vitals: WebVitals,
    /// Performance score
    pub score: PerformanceScore,
    /// Render-blocking resource analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_blocking: Option<RenderBlockingAnalysis>,
    /// Page content weight / resource breakdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_weight: Option<ContentWeight>,
    /// Third-party resource attribution per origin (#138)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party: Option<ThirdPartyAttribution>,
    /// Critical request chain / network dependency tree (#132)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_chain: Option<CriticalChain>,
    /// Unminified JS/CSS asset detection (#111)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minification: Option<MinificationAnalysis>,
    /// Non-composited animation detection (#105)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<AnimationAnalysis>,
    /// Unused JS (#106) and CSS (#107) via CDP Coverage API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageAnalysis>,
    /// Implausible or unmeasurable metrics detected during analysis (#291).
    /// Keys: "lcp_not_measured", "speed_index_fallback_to_lcp",
    /// "tti_fallback_to_lcp", "inp_not_measured", "css_coverage_unavailable",
    /// "css_coverage_partial", "js_coverage_unavailable".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub measurement_warnings: Vec<String>,
}

impl AuditReport {
    /// The URL the audited document was served from: the browser's final URL
    /// after redirects, else the requested one (reports without a navigation
    /// snapshot). Modules judge transport and origin from this, never from
    /// the requested URL (#775).
    pub fn document_url(&self) -> &str {
        self.accessibility
            .execution
            .navigation
            .served_url()
            .unwrap_or(&self.url)
    }

    /// Whether the document was served over TLS — the final URL's scheme,
    /// so an `http://` entry that redirects to `https://` counts as secure
    /// and an `https://` entry that ends on `http://` does not.
    pub fn document_is_https(&self) -> bool {
        self.document_url().starts_with("https://")
    }

    /// Create a new audit report
    pub fn new(
        url: String,
        wcag_level: WcagLevel,
        wcag_results: WcagResults,
        duration_ms: u64,
    ) -> Self {
        let score = AccessibilityScorer::calculate_score(&wcag_results.violations);
        let grade = AccessibilityScorer::calculate_grade(score).to_string();
        let certificate = AccessibilityScorer::calculate_certificate(score).to_string();
        let statistics = AccessibilityScorer::calculate_statistics(&wcag_results.violations);
        let nodes_analyzed = wcag_results.nodes_checked;

        Self {
            url,
            wcag_level,
            timestamp: Utc::now(),
            accessibility: AccessibilitySection {
                wcag_results,
                score,
                grade,
                certificate,
                statistics,
                nodes_analyzed,
                execution: AuditExecution::default(),
            },
            duration_ms,
            performance: None,
            security: None,
            html_conform: None,
            experience: ExperienceSection::default(),
            ux: None,
            journey: None,
            discoverability: DiscoverabilitySection {
                seo: None,
                ai_visibility: None,
                content_visibility: None,
                source_quality: None,
                tech_stack: None,
            },
            page_screenshots: None,
            dual_viewport: None,
            viewport_scores: None,
            throttled_performance: Vec::new(),
            patterns: None,
            screenshot_status: ScreenshotStatus::NotRequested,
            best_practices: None,
            commerce: None,
            consent_banner_detected: false,
            consent_banner_cmp: None,
            consent_banner_dismissed: false,
            consent_privacy: None,
            accessibility_journey: None,
            interactive_findings: Vec::new(),
            screen_reader_audit: None,
            run_context: RunContext::default(),
        }
    }

    /// Set performance results
    pub fn with_performance(mut self, performance: PerformanceResults) -> Self {
        self.performance = Some(performance);
        self
    }

    /// Attach pattern detection results.
    pub fn with_patterns(mut self, patterns: crate::patterns::PatternAnalysis) -> Self {
        self.patterns = Some(patterns);
        self
    }

    /// Set SEO results
    pub fn with_seo(mut self, seo: SeoAnalysis) -> Self {
        self.discoverability.seo = Some(seo);
        self
    }

    /// Set security results
    pub fn with_security(mut self, security: SecurityAnalysis) -> Self {
        self.security = Some(security);
        self
    }

    /// Set HTML5 spec-conformance results
    pub fn with_html_conform(
        mut self,
        html_conform: crate::html_conform::HtmlConformAnalysis,
    ) -> Self {
        self.html_conform = Some(html_conform);
        self
    }

    /// Set mobile friendliness results
    pub fn with_mobile(mut self, mobile: MobileFriendliness) -> Self {
        self.experience.mobile = Some(mobile);
        self
    }

    /// Set UX analysis results
    pub fn with_ux(mut self, ux: UxAnalysis) -> Self {
        self.ux = Some(ux);
        self
    }

    /// Set journey analysis results
    pub fn with_journey(mut self, journey: crate::journey::JourneyAnalysis) -> Self {
        self.journey = Some(journey);
        self
    }

    /// Set dark mode analysis results
    pub fn with_dark_mode(mut self, dark_mode: DarkModeAnalysis) -> Self {
        self.experience.dark_mode = Some(dark_mode);
        self
    }

    /// Set design-quality analysis results (#528)
    pub fn with_design_quality(
        mut self,
        design_quality: crate::design_quality::DesignQualityAnalysis,
    ) -> Self {
        self.experience.design_quality = Some(design_quality);
        self
    }

    /// Set ai-transparency analysis results (EU AI Act Art. 50)
    pub fn with_ai_transparency(
        mut self,
        ai_transparency: crate::ai_transparency::AiTransparencyAnalysis,
    ) -> Self {
        self.experience.ai_transparency = Some(ai_transparency);
        self
    }

    /// Set DNS-configuration check results (#545)
    pub fn with_network_dns(
        mut self,
        network_dns: crate::network::dns::NetworkDnsAnalysis,
    ) -> Self {
        self.experience.network_dns = Some(network_dns);
        self
    }

    pub fn with_tech_stack(mut self, tech_stack: crate::tech_stack::TechStackAnalysis) -> Self {
        self.discoverability.tech_stack = Some(tech_stack);
        self
    }

    pub fn with_best_practices(mut self, bp: crate::best_practices::BestPracticesAnalysis) -> Self {
        self.best_practices = Some(bp);
        self
    }

    /// Get the total number of violations
    pub fn violation_count(&self) -> usize {
        self.accessibility.wcag_results.violations.len()
    }

    /// Whether the page passes under the decision profile stored with it —
    /// the same decision every output publishes (#781).
    pub fn passed(&self) -> bool {
        super::normalized::normalize(self)
            .normalized
            .decision
            .verdict
            == super::Verdict::Pass
    }
}

// The weighted overall score is computed canonically in `audit::normalized`
// (see `NormalizedReport.overall_score` and `build_module_scores`). A second,
// divergent `AuditReport::overall_score()` previously lived here but was only
// reachable from tests; it was removed to keep a single source of truth (#447).

/// Batch audit report for multiple URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReport {
    /// Individual reports for each URL
    pub reports: Vec<AuditReport>,
    /// URLs that failed to audit (with error messages)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BatchError>,
    /// Summary statistics
    pub summary: BatchSummary,
    /// Optional crawl/link diagnostics if the batch originated from crawler discovery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crawl_diagnostics: Option<CrawlDiagnostics>,
    /// Optional sitemap HTTP/indexability diagnostics when the batch originated
    /// from an XML sitemap.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sitemap_diagnostics: Option<SitemapDiagnostics>,
    /// Cross-page consistency analysis (issues #44/#45/#46). None when the
    /// batch contains fewer than 2 pages.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub consistency: Option<crate::audit::batch_consistency::BatchConsistencyAnalysis>,
    /// How the audited URLs were discovered and sampled. Lets consumers tell a
    /// partial selection apart from full domain coverage (issues #261, #791).
    /// A selection is no evidence of representativeness: statements hold for
    /// the successfully audited pages only.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sample: Option<SampleMetadata>,
    /// Total execution time
    pub total_duration_ms: u64,
}

/// Where the audited URL set came from and how it was sampled.
///
/// Without this, a 20-of-500 sample is indistinguishable from a complete audit
/// in the report. Surfacing the population size, the applied limit and the
/// selection method makes the scope of a batch explicit (issue #261).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetadata {
    /// Origin of the candidate URLs: `"sitemap"`, `"crawl"` or `"url_file"`.
    pub source: String,
    /// Candidate URLs discovered before any limit/sampling was applied.
    pub total_discovered: usize,
    /// URLs actually selected and audited.
    pub audited: usize,
    /// The `--max-pages` limit, when one capped the audited set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_limit: Option<usize>,
    /// How the audited subset was chosen: `"first_n"` (sitemap/discovery order),
    /// `"all"` when every discovered URL was audited, `"path_filter"` when
    /// `--include-path`/`--exclude-path` narrowed the set (then possibly also
    /// capped by `sample_limit`, in discovery order), or `"stratified"` when
    /// `--sample-strategy stratified` drew across structural strata (after any
    /// path filter), listed in `strata`.
    pub selection: String,
    /// True when fewer URLs were audited than discovered — i.e. this is a sample,
    /// not full coverage.
    pub is_sample: bool,
    /// Strata of a `"stratified"` selection with candidates and picks per
    /// stratum; empty for every other method.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strata: Vec<SampleStratum>,
}

/// One structural stratum of a stratified selection (`[lang/]section[.ext]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleStratum {
    pub key: String,
    /// Candidate URLs in this stratum, before the budget was applied.
    pub discovered: usize,
    /// URLs the draw took from this stratum (0 when the budget did not reach it).
    pub selected: usize,
}

/// Sitemap URL validation result for sitemap-driven batch reports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SitemapDiagnostics {
    /// HTTP outcome of every checked sitemap URL by class (`2xx`, `3xx`,
    /// `4xx`, `5xx`, `other`, `fetch_error`) — measured for each URL, not
    /// inferred from the issue list (plan 87). Redirects are not followed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub status_counts: BTreeMap<String, usize>,
    /// Number of sitemap URLs checked via HTTP.
    pub checked_urls: usize,
    /// Sitemap entries that do not resolve to a direct canonical 200 response,
    /// or that are marked noindex.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub http_issues: Vec<SitemapHttpIssue>,
    /// URLs present in the sitemap but not linked by any audited page.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orphan_sitemap_urls: Vec<String>,
    /// Internal targets linked by audited pages but absent from the sitemap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_not_in_sitemap: Vec<String>,
    /// Sitemap entries blocked by a `Disallow` rule in `robots.txt` under
    /// `User-agent: *` — a URL with declared indexing intent (sitemap
    /// membership) that crawlers following `robots.txt` will not fetch (#549).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub robots_conflicts: Vec<RobotsSitemapConflict>,
    /// Crawl-depth (BFS click-distance from a heuristic start page) through
    /// this batch run's internal link graph (#548). `None` when no
    /// reasonable start-page candidate was audited.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub crawl_depths: Option<CrawlDepthDiagnostics>,
    /// Why `crawl_depths` is absent when it is: `"homepage_unavailable"`
    /// when the homepage could not be fetched — the depths are then not
    /// determined rather than measured from some other page (plan 81).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crawl_depths_unavailable: Option<String>,
}

/// Crawl-depth diagnostics for a sitemap-driven batch (#548): BFS distance
/// (in clicks) from a heuristically-chosen start page to every other
/// audited page, following each page's already-extracted internal link
/// targets.
///
/// Deliberately scoped to *this batch run's* link graph, not the true
/// site-wide graph: a link to a page outside the audited set can't be
/// followed further (that page's own outbound links are unknown), so depth
/// here is a lower bound within the audited pages, not a guarantee about
/// the real site. `partial_batch` flags when the audited set is smaller
/// than the full discovered sitemap (a `--max-pages` sample, or pages that
/// failed to audit), which makes that gap more likely to matter — the
/// report surfaces this as an explicit caveat rather than silently
/// presenting a partial graph as complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlDepthDiagnostics {
    /// The site's homepage BFS started from, after redirects (plan 81).
    pub start_url: String,
    /// Where the homepage's outbound links came from: `"audited_homepage"`
    /// (it was one of the audited pages) or `"fetched_homepage"` (fetched
    /// once over HTTP for its links only — not an audit, not in any score).
    /// Empty in reports before plan 81, whose start was the shortest audited
    /// sitemap URL, not necessarily the homepage.
    #[serde(default)]
    pub start_source: String,
    /// Depth (in clicks) -> number of audited pages at that depth. 0 is the
    /// start page itself.
    pub depth_histogram: BTreeMap<usize, usize>,
    /// The most-clicks-away pages, capped and sorted deepest-first —
    /// intentionally not a full per-page dump (see #548).
    pub deepest_pages: Vec<CrawlDepthEntry>,
    /// Audited pages never reached by following internal links from
    /// `start_url` within this batch, capped — distinct from "very deep",
    /// not a default/fallback depth.
    pub unreachable_pages: Vec<String>,
    /// True when the audited set is smaller than the full discovered
    /// sitemap (sampling via `--max-pages`, or per-page audit failures) —
    /// depth and reachability are then only a partial view of the real
    /// site-wide link graph.
    pub partial_batch: bool,
}

/// One page's BFS click-distance from the crawl-depth start page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlDepthEntry {
    pub url: String,
    pub depth: usize,
}

/// One sitemap URL blocked by a `robots.txt` `Disallow` rule under
/// `User-agent: *` (#549) — a contradiction between declared indexing intent
/// (sitemap membership) and actual crawler access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsSitemapConflict {
    pub url: String,
    /// The specific `Disallow` path prefix that blocks this URL.
    pub rule: String,
}

/// One sitemap URL whose HTTP/indexability state contradicts sitemap guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapHttpIssue {
    /// Canonical issue kind: `"status"`, `"redirect"`, `"noindex"` or `"fetch_error"`.
    pub kind: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    pub detail: String,
}

/// Severity of a broken or problematic link finding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrokenLinkSeverity {
    High,
    Medium,
    Low,
}

/// A redirect chain with more than 1 hop detected during link checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectChain {
    /// Page that contains the link
    pub source_url: String,
    /// Original link target
    pub target_url: String,
    /// Final resolved URL after all redirects
    pub final_url: String,
    /// Number of redirect hops
    pub hops: u8,
    /// Whether the link points to an external domain
    pub is_external: bool,
}

/// Optional crawl/link diagnostics attached to crawler-driven batch reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlDiagnostics {
    /// Seed URL that started discovery
    pub seed_url: String,
    /// Number of discovered pages in the crawl set
    pub discovered_urls: usize,
    /// Number of unique internal links that were status-checked
    pub checked_internal_links: usize,
    /// Broken internal links (4xx/5xx or fetch failure)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub broken_internal_links: Vec<BrokenLink>,
    /// Number of unique external links that were status-checked
    pub checked_external_links: usize,
    /// Broken external links (4xx/5xx or fetch failure)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub broken_external_links: Vec<BrokenLink>,
    /// Links with more than 1 redirect hop
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redirect_chains: Vec<RedirectChain>,
}

/// A broken internal link found during crawl-based link checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenLink {
    /// Page that contains the broken link
    pub source_url: String,
    /// Link target that failed
    pub target_url: String,
    /// HTTP status code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    /// Error reason for network/content failures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether the link points to an external domain
    pub is_external: bool,
    /// Number of redirect hops before reaching the final status (0 = direct)
    #[serde(default)]
    pub redirect_hops: u8,
    /// Severity derived from link type and status
    pub severity: BrokenLinkSeverity,
}

/// A failed URL audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// The URL that failed
    pub url: String,
    /// Error message
    pub error: String,
    /// Set when the page was a bot wall, challenge or access denial rather
    /// than the site's content (`AuditError::AccessBlocked`): why it counts as
    /// blocked. Lets per-page consumers tell "blocked" from "failed" without
    /// parsing `error` (plan 67). Not part of the batch JSON contract.
    #[serde(skip)]
    pub blocked_reason: Option<String>,
}

/// URL lifecycle of a batch, counted separately so a partial or failed run is
/// never read as a complete one (#768). `discovered >= selected >= attempted`;
/// `attempted = succeeded + blocked + failed`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchCoverage {
    /// Candidate URLs found by the source (sitemap, crawl, file).
    pub discovered: usize,
    /// URLs chosen for auditing after path filters and `--max-pages`.
    pub selected: usize,
    /// URLs an audit was started for.
    pub attempted: usize,
    /// URLs that produced a report; the only ones the averages are over.
    pub succeeded: usize,
    /// URLs answered with a bot wall, challenge or access denial.
    pub blocked: usize,
    /// URLs whose audit failed for another reason.
    pub failed: usize,
}

impl BatchCoverage {
    pub fn from_outcomes(succeeded: usize, errors: &[BatchError]) -> Self {
        let blocked = errors.iter().filter(|e| e.blocked_reason.is_some()).count();
        let attempted = succeeded + errors.len();
        Self {
            discovered: attempted,
            selected: attempted,
            attempted,
            succeeded,
            blocked,
            failed: errors.len() - blocked,
        }
    }

    /// True when every selected URL was attempted and produced a report.
    pub fn is_complete(&self) -> bool {
        self.blocked == 0
            && self.failed == 0
            && self.succeeded == self.attempted
            && self.selected <= self.attempted
    }

    /// URLs selected but never started (budget, cancellation).
    pub fn not_attempted(&self) -> usize {
        self.selected.saturating_sub(self.attempted)
    }
}

/// Summary statistics for a batch audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// URLs that produced a report (the denominator of `average_score`).
    /// Failed and blocked URLs are counted in `coverage`, not here.
    pub total_urls: usize,
    /// URL lifecycle counters, including failed and blocked URLs.
    #[serde(default)]
    pub coverage: BatchCoverage,
    /// URLs that passed
    pub passed: usize,
    /// URLs that failed
    pub failed: usize,
    /// Average score over the URLs that produced a report. `None` when no
    /// URL did: nothing was measured, which is not a score of 0 (#768).
    pub average_score: Option<f64>,
    /// Denominator of `average_score`: the URLs that produced a report
    /// (`coverage.succeeded`), out of `coverage.attempted`.
    #[serde(default)]
    pub average_over: usize,
    /// WCAG occurrences across all pages — the batch sum of the pages'
    /// `occurrence_counts.total`. Findings of other domains are not
    /// violations.
    pub total_violations: usize,
    /// Number of distinct WCAG rule IDs that fired anywhere across all pages.
    #[serde(default)]
    pub violated_rule_count: usize,
    /// Number of distinct WCAG Level-A rules with High/Critical severity across
    /// all pages (legal exposure indicator).
    #[serde(default)]
    pub legal_flags: usize,
    /// Sum of blocking-interaction-issue occurrences (unlabeled interactive
    /// elements, 4.1.2/2.1.1) across all pages. Mirrors the single-page
    /// `RiskAssessment.blocking_issues` signal so the batch CI verdict can
    /// fail on it too (#QA-028).
    #[serde(default)]
    pub blocking_issues: usize,
    /// Worst-case risk level across all audited pages.
    #[serde(default)]
    pub risk: crate::audit::normalized::RiskLevel,
    /// i18n key of the narrative score band sentence
    /// ("verdict-batch-excellent", …). A band of the average score, not the
    /// decision: that is `decision`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub score_band_key: String,
    /// Pages with a gate finding (technical blocker) summed over the batch.
    #[serde(default)]
    pub technical_blockers: usize,
    /// The batch decision under the run's profile (#781). `passed`/`failed`
    /// count the pages' own decisions under the same profile.
    pub decision: crate::audit::verdict::VerdictResult,
    /// Finding patterns that recur on two or more pages, by pattern id —
    /// see `audit::template_dedup::detect_root_causes` (2.0 finding model).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_causes: Vec<crate::audit::template_dedup::RootCause>,
    #[serde(default)]
    pub audit_quality: AuditQuality,
    /// Spread and worst cases of the page scores next to the mean (#791).
    /// `None` when no page produced a report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_spread: Option<ScoreSpread>,
    /// What the successfully audited pages are: page types and languages
    /// (#791). Describes the selection; it does not make it representative.
    #[serde(default)]
    pub selection_profile: SelectionProfile,
}

/// Distribution of the page scores behind `average_score`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreSpread {
    pub min: u32,
    pub median: f64,
    pub max: u32,
    /// Population standard deviation of the page scores.
    pub std_dev: f64,
    /// Lowest-scoring pages, worst first (at most [`ScoreSpread::WORST_CASES`]).
    pub worst_pages: Vec<WorstPage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorstPage {
    pub url: String,
    pub score: u32,
}

impl ScoreSpread {
    pub const WORST_CASES: usize = 5;

    fn from_pages(pages: &[(String, u32)]) -> Option<Self> {
        if pages.is_empty() {
            return None;
        }
        let mut sorted: Vec<&(String, u32)> = pages.iter().collect();
        sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        let n = sorted.len();
        let mean = sorted.iter().map(|p| p.1 as f64).sum::<f64>() / n as f64;
        let variance = sorted
            .iter()
            .map(|p| (p.1 as f64 - mean).powi(2))
            .sum::<f64>()
            / n as f64;
        let median = if n % 2 == 1 {
            sorted[n / 2].1 as f64
        } else {
            (sorted[n / 2 - 1].1 + sorted[n / 2].1) as f64 / 2.0
        };
        Some(Self {
            min: sorted[0].1,
            median,
            max: sorted[n - 1].1,
            std_dev: variance.sqrt(),
            worst_pages: sorted
                .iter()
                .take(Self::WORST_CASES)
                .map(|p| WorstPage {
                    url: p.0.clone(),
                    score: p.1,
                })
                .collect(),
        })
    }
}

/// Page types and languages of the pages that produced a report. Keys are
/// canonical (snake_case page type, lower-case primary language subtag), so
/// the JSON stays language-independent. Pages without the signal count under
/// `unclassified` / `language_unknown`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionProfile {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub page_types: BTreeMap<String, usize>,
    #[serde(default)]
    pub unclassified: usize,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub languages: BTreeMap<String, usize>,
    #[serde(default)]
    pub language_unknown: usize,
}

impl SelectionProfile {
    fn from_reports(reports: &[AuditReport]) -> Self {
        let mut profile = Self::default();
        for report in reports {
            let seo = report.discoverability.seo.as_ref();
            match seo
                .and_then(|s| s.content_profile.as_ref())
                .and_then(|p| serde_json::to_value(&p.page_classification.primary_type).ok())
                .and_then(|v| v.as_str().map(str::to_string))
            {
                Some(kind) => *profile.page_types.entry(kind).or_default() += 1,
                None => profile.unclassified += 1,
            }
            match seo
                .and_then(|s| s.technical.lang.as_deref())
                .and_then(|l| l.split(['-', '_']).next())
                .map(|l| l.trim().to_ascii_lowercase())
                .filter(|l| !l.is_empty())
            {
                Some(lang) => *profile.languages.entry(lang).or_default() += 1,
                None => profile.language_unknown += 1,
            }
        }
        profile
    }
}

impl BatchSummary {
    /// Grade and certificate of the batch, by the single-report rule: both
    /// describe the average accessibility score, not the weighted overall,
    /// and the certificate is gated by the batch risk (plan 29, D1). The JSON
    /// summary and the PDF read this one function (plan 111).
    pub fn grade_and_certificate(&self) -> (String, String) {
        let accessibility = self.average_score.unwrap_or(0.0).round() as f32;
        let grade = AccessibilityScorer::calculate_grade(accessibility).to_string();
        let certificate = crate::audit::normalized::gate_certificate_by_risk(
            AccessibilityScorer::calculate_certificate(accessibility).to_string(),
            &self.risk,
            self.legal_flags,
            self.blocking_issues,
        );
        (grade, certificate)
    }
}

/// Number of distinct rule ids (WCAG and SEO) that fired anywhere across the
/// reports. Rules that recur across pages are reported per element pattern
/// in `BatchSummary::root_causes` (2.0 finding model).
pub fn count_violated_rules(reports: &[crate::audit::normalized::NormalizedReport]) -> usize {
    reports
        .iter()
        .flat_map(|r| {
            r.findings
                .iter()
                .filter(|f| f.category == "wcag")
                .map(|f| f.rule_id.as_str())
        })
        .collect::<std::collections::HashSet<_>>()
        .len()
}

/// Count the number of distinct WCAG Level-A rules with High/Critical severity
/// that appear across any of the given reports (legal exposure indicator).
///
/// Also accounts for legal exposure raised purely by the screen-reader BFSG
/// verdict (`RiskAssessment.legal_flags` can exceed the page's own WCAG
/// Level-A finding count, see `audit::normalized::compute_risk`). That signal
/// has no WCAG rule id of its own, so it is represented as a synthetic member
/// of the same distinct-signal set whenever it fires on at least one page —
/// otherwise a page could FAIL its single-page verdict on this signal while
/// contributing nothing to the batch's legal_flags summary (#QA-024).
fn compute_legal_flags(reports: &[crate::audit::normalized::NormalizedReport]) -> usize {
    use std::collections::HashSet;

    let mut rule_ids: HashSet<&str> = reports
        .iter()
        .flat_map(|r| r.findings.iter())
        .filter(|f| f.is_legal_flag())
        .map(|f| f.rule_id.as_str())
        .collect();

    let sr_only_legal_exposure = reports.iter().any(|r| {
        let findings_based = r.findings.iter().filter(|f| f.is_legal_flag()).count();
        r.risk.legal_flags > findings_based
    });
    if sr_only_legal_exposure {
        rule_ids.insert("screen_reader.bfsg_noncompliant");
    }

    rule_ids.len()
}

/// Compute the worst-case risk level across a set of normalized reports.
///
/// Critical: any page is Critical.
/// High: ≥ 20% of pages are High (or worse).
/// Medium: ≥ 20% of pages are Medium or worse.
/// Low: otherwise.
///
/// Severity buckets are cumulative — a High-risk page also counts toward the
/// Medium threshold — so e.g. 1 High + 1 Medium page out of 10 (20% elevated)
/// correctly escalates to Medium instead of each bucket independently missing
/// the 20% cutoff and silently reporting Low (#QA-022).
pub fn compute_worst_risk(
    reports: &[crate::audit::normalized::NormalizedReport],
) -> crate::audit::normalized::RiskLevel {
    use crate::audit::normalized::RiskLevel;
    use std::collections::HashMap;

    let page_count = reports.len().max(1);
    let mut counts: HashMap<RiskLevel, usize> = HashMap::new();
    for r in reports {
        *counts.entry(r.risk.level).or_insert(0) += 1;
    }

    let critical = *counts.get(&RiskLevel::Critical).unwrap_or(&0);
    let high = *counts.get(&RiskLevel::High).unwrap_or(&0);
    let medium = *counts.get(&RiskLevel::Medium).unwrap_or(&0);

    if critical > 0 {
        RiskLevel::Critical
    } else if high * 5 >= page_count {
        RiskLevel::High
    } else if (high + medium) * 5 >= page_count {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

impl BatchReport {
    /// Create a batch report from individual reports and errors
    pub fn from_reports(
        reports: Vec<AuditReport>,
        errors: Vec<BatchError>,
        total_duration_ms: u64,
    ) -> Self {
        let total_urls = reports.len();
        let coverage = BatchCoverage::from_outcomes(total_urls, &errors);
        let normalized_reports: Vec<crate::audit::normalized::NormalizedReport> = reports
            .iter()
            .map(|r| crate::audit::normalized::normalize(r).normalized)
            .collect();
        // A page passes when its own stored decision passes (#781): the
        // batch count is a projection of the single-page policy, never a
        // second rule.
        let passed = normalized_reports
            .iter()
            .filter(|r| r.decision.verdict == super::Verdict::Pass)
            .count();
        let profile = reports
            .first()
            .map(|r| r.run_context.decision_profile.clone())
            .unwrap_or_default();
        let failed = total_urls - passed;

        let average_score = (total_urls > 0).then(|| {
            normalized_reports
                .iter()
                .map(|r| r.score as f64)
                .sum::<f64>()
                / total_urls as f64
        });

        let score_spread = ScoreSpread::from_pages(
            &normalized_reports
                .iter()
                .map(|r| (r.url.clone(), r.score))
                .collect::<Vec<_>>(),
        );
        let selection_profile = SelectionProfile::from_reports(&reports);

        let total_violations = normalized_reports
            .iter()
            .map(|r| r.occurrence_counts.total)
            .sum();
        let technical_blockers = normalized_reports
            .iter()
            .map(super::verdict::technical_blockers)
            .sum();

        let violated_rule_count = count_violated_rules(&normalized_reports);
        let root_causes = crate::audit::template_dedup::detect_root_causes(&normalized_reports);
        let legal_flags = compute_legal_flags(&normalized_reports);
        let blocking_issues = normalized_reports
            .iter()
            .map(|r| r.risk.blocking_issues)
            .sum();
        let risk = compute_worst_risk(&normalized_reports);
        let audit_quality = {
            let failed_rule_checks = normalized_reports
                .iter()
                .map(|report| report.execution.quality.failed_rule_checks)
                .sum();
            let partial_or_failed_modules = normalized_reports
                .iter()
                .map(|report| report.execution.quality.partial_or_failed_modules)
                .sum();
            let status = if normalized_reports.is_empty()
                || normalized_reports.iter().any(|report| {
                    report.execution.quality.status == AuditQualityStatus::Insufficient
                }) {
                AuditQualityStatus::Insufficient
            } else if !errors.is_empty()
                || normalized_reports
                    .iter()
                    .any(|report| report.execution.quality.status == AuditQualityStatus::Partial)
            {
                AuditQualityStatus::Partial
            } else {
                AuditQualityStatus::Complete
            };
            AuditQuality {
                status,
                qualified_results: status != AuditQualityStatus::Complete,
                failed_rule_checks,
                partial_or_failed_modules,
                reasons: {
                    let mut reasons = Vec::new();
                    if coverage.failed > 0 {
                        reasons.push(format!("failed_urls:{}", coverage.failed));
                    }
                    if coverage.blocked > 0 {
                        reasons.push(format!("blocked_urls:{}", coverage.blocked));
                    }
                    reasons
                },
            }
        };
        let score_band_key = {
            let s = average_score.unwrap_or(0.0).round() as u32;
            if s >= 90 {
                "verdict-batch-excellent"
            } else if s >= 70 {
                "verdict-batch-solid"
            } else if s >= 50 {
                "verdict-batch-deficient"
            } else {
                "verdict-batch-critical"
            }
            .to_string()
        };

        let mut result = Self {
            reports,
            errors,
            summary: BatchSummary {
                total_urls,
                coverage,
                passed,
                failed,
                average_score,
                average_over: total_urls,
                total_violations,
                violated_rule_count,
                legal_flags,
                blocking_issues,
                risk,
                score_band_key,
                technical_blockers,
                decision: super::verdict::VerdictResult::undecided(),
                root_causes,
                audit_quality,
                score_spread,
                selection_profile,
            },
            crawl_diagnostics: None,
            sitemap_diagnostics: None,
            consistency: None,
            sample: None,
            total_duration_ms,
        };
        result.summary.decision = super::verdict::compute_batch_verdict(&result.summary, &profile);
        result.consistency = crate::audit::batch_consistency::analyze(&result);
        result
    }

    pub fn with_crawl_diagnostics(mut self, diagnostics: CrawlDiagnostics) -> Self {
        self.crawl_diagnostics = Some(diagnostics);
        self
    }

    pub fn with_sitemap_diagnostics(mut self, diagnostics: SitemapDiagnostics) -> Self {
        self.sitemap_diagnostics = Some(diagnostics);
        self
    }

    pub fn with_sample(mut self, sample: SampleMetadata) -> Self {
        self.summary.coverage.discovered = sample.total_discovered;
        self.summary.coverage.selected = sample.audited;
        self.sample = Some(sample);
        // Selected URLs that were never started qualify the batch (#768); the
        // decision is derived again from the final coverage.
        let unattempted = self.summary.coverage.not_attempted();
        if unattempted > 0 {
            let quality = &mut self.summary.audit_quality;
            quality
                .reasons
                .push(format!("not_attempted_urls:{unattempted}"));
            quality.qualified_results = true;
            if quality.status == AuditQualityStatus::Complete {
                quality.status = AuditQualityStatus::Partial;
            }
        }
        let profile = self.summary.decision.profile.clone();
        self.summary.decision = super::verdict::compute_batch_verdict(&self.summary, &profile);
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn redirect_chain_serializes_only_when_captured() {
        let json = serde_json::to_value(NavigationSnapshot::default()).unwrap();
        assert!(json.get("redirect_chain").is_none());
        let nav = NavigationSnapshot {
            redirect_chain: vec![RedirectHop {
                url: "http://a.test/".into(),
                status: 301,
            }],
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&nav).unwrap()["redirect_chain"],
            serde_json::json!([{"url": "http://a.test/", "status": 301}])
        );
    }

    #[test]
    fn meta_refresh_serializes_flat_per_viewport() {
        let nav = NavigationSnapshot {
            meta_refresh: vec![ViewportMetaRefresh {
                viewport: "desktop".to_string(),
                refresh: crate::browser::MetaRefresh {
                    outcome: crate::browser::MetaRefreshOutcome::InterstitialFollowed,
                    hops: 1,
                },
            }],
            ..Default::default()
        };
        let json = serde_json::to_value(&nav).unwrap();
        assert_eq!(
            json["meta_refresh"],
            serde_json::json!([{"viewport": "desktop", "outcome": "interstitial_followed", "hops": 1}])
        );
        let empty = serde_json::to_value(NavigationSnapshot::default()).unwrap();
        assert!(empty.get("meta_refresh").is_none());
    }
    use super::*;
    use crate::wcag::WcagResults;

    #[test]
    fn score_spread_exposes_worst_cases_next_to_the_mean() {
        let pages: Vec<(String, u32)> = [("a", 100), ("b", 100), ("c", 100), ("d", 20)]
            .iter()
            .map(|(u, s)| (u.to_string(), *s))
            .collect();
        let spread = ScoreSpread::from_pages(&pages).unwrap();
        assert_eq!((spread.min, spread.max), (20, 100));
        assert_eq!(spread.median, 100.0);
        assert!(spread.std_dev > 30.0, "{}", spread.std_dev);
        assert_eq!(spread.worst_pages[0].url, "d");
        assert!(ScoreSpread::from_pages(&[]).is_none());
    }

    #[test]
    fn selection_profile_counts_unknown_instead_of_dropping_pages() {
        let mut with_lang = AuditReport::new(
            "https://e.com/a".into(),
            WcagLevel::AA,
            WcagResults::new(),
            1,
        );
        with_lang.discoverability.seo = Some(crate::seo::SeoAnalysis {
            technical: crate::seo::technical::TechnicalSeo {
                lang: Some("de-AT".into()),
                ..Default::default()
            },
            ..Default::default()
        });
        let bare = AuditReport::new(
            "https://e.com/b".into(),
            WcagLevel::AA,
            WcagResults::new(),
            1,
        );
        let profile = SelectionProfile::from_reports(&[with_lang, bare]);
        assert_eq!(profile.languages.get("de"), Some(&1));
        assert_eq!(profile.language_unknown, 1);
        assert_eq!(profile.unclassified, 2);
    }

    #[test]
    fn test_audit_report_new() {
        let results = WcagResults::new();
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            500,
        );

        assert_eq!(report.url, "https://example.com");
        assert_eq!(report.accessibility.score, 100.0); // No violations = perfect score
        assert_eq!(report.duration_ms, 500);
        assert!(report.performance.is_none());
        assert!(report.discoverability.seo.is_none());
        assert!(report.security.is_none());
        assert!(report.experience.mobile.is_none());
    }

    #[test]
    fn test_batch_report() {
        let reports = vec![
            AuditReport::new(
                "https://a.com".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ),
            AuditReport::new(
                "https://b.com".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                200,
            ),
        ];

        let batch = BatchReport::from_reports(reports, vec![], 300);

        assert_eq!(batch.summary.total_urls, 2);
        assert_eq!(batch.summary.passed, 2);
        assert_eq!(batch.summary.average_score, Some(100.0));
    }

    #[test]
    fn test_passed_with_perfect_score() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        assert!(report.passed());
    }

    #[test]
    fn test_passed_with_critical_violation() {
        let mut results = WcagResults::new();
        results.add_violation(crate::wcag::Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::wcag::Severity::Critical,
            "Missing alt",
            "node-1",
        ));
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        assert!(!report.passed());
    }

    #[test]
    fn test_with_builder_methods() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );

        let report = report.with_security(crate::security::SecurityAnalysis {
            score: 80,
            grade: "A".to_string(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
            protection: Default::default(),
            sourcemap_leaks: Default::default(),
            fetch: Default::default(),
        });

        assert!(report.security.is_some());
        assert_eq!(report.security.as_ref().unwrap().score, 80);
    }

    #[test]
    fn test_violation_count() {
        let mut results = WcagResults::new();
        results.add_violation(crate::wcag::Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "Missing alt",
            "node-1",
        ));
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        assert_eq!(report.violation_count(), 1);
    }

    #[test]
    fn batch_summary_has_recurring_rules_and_risk() {
        let reports = vec![
            AuditReport::new(
                "https://a.com".into(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ),
            AuditReport::new(
                "https://b.com".into(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 0);
        // No violations: no root causes, no violated rules, Low risk
        assert!(batch.summary.root_causes.is_empty());
        assert_eq!(batch.summary.violated_rule_count, 0);
        assert_eq!(batch.summary.legal_flags, 0);
        assert_eq!(batch.summary.risk, crate::audit::normalized::RiskLevel::Low);
    }

    #[test]
    fn count_violated_rules_counts_distinct_rules_across_pages() {
        use crate::audit::normalized::normalize;
        use crate::wcag::Violation;

        let mut results1 = WcagResults::new();
        results1.add_violation(Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "Missing alt",
            "n1",
        ));

        let mut results2 = WcagResults::new();
        results2.add_violation(Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "Missing alt",
            "n2",
        ));

        let report1 = AuditReport::new("https://a.com".into(), WcagLevel::AA, results1, 100);
        let report2 = AuditReport::new("https://b.com".into(), WcagLevel::AA, results2, 100);
        let r1 = normalize(&report1);
        let r2 = normalize(&report2);

        assert_eq!(
            count_violated_rules(&[r1.normalized, r2.normalized]),
            1,
            "one distinct rule fired on both pages"
        );
    }

    /// Batch reports and their diagnostics skip empty lists when serialized;
    /// reading them back must not fail on the missing fields.
    #[test]
    fn batch_report_with_empty_lists_round_trips_through_json() {
        let mut batch = BatchReport::from_reports(
            vec![AuditReport::new(
                "https://a.com".into(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            )],
            vec![],
            0,
        );
        batch.crawl_diagnostics = Some(CrawlDiagnostics {
            seed_url: "https://a.com".into(),
            discovered_urls: 1,
            checked_internal_links: 0,
            broken_internal_links: vec![],
            checked_external_links: 0,
            broken_external_links: vec![],
            redirect_chains: vec![],
        });
        batch.sitemap_diagnostics = Some(SitemapDiagnostics::default());

        let json = serde_json::to_value(&batch).unwrap();
        assert!(json.get("errors").is_none());
        let reparsed: BatchReport =
            serde_json::from_value(json.clone()).expect("serialized BatchReport must deserialize");
        assert_eq!(serde_json::to_value(&reparsed).unwrap(), json);
    }

    #[test]
    fn compute_worst_risk_critical_if_any_critical_page() {
        use crate::audit::normalized::{normalize, RiskLevel};
        use crate::wcag::Violation;

        // Create two pages: one with no violations (Low risk) and one with
        // many critical violations that push it to Critical risk.
        let empty_report = AuditReport::new(
            "https://a.com".into(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let empty = normalize(&empty_report);

        // Build a report likely to be Critical risk: legal_flags > 0 + critical issues
        let mut results = WcagResults::new();
        for i in 0..3 {
            results.add_violation(Violation::new(
                "1.1.1",
                "Alt",
                WcagLevel::A,
                crate::wcag::Severity::Critical,
                "missing",
                format!("n{i}"),
            ));
        }
        let risky_report = AuditReport::new("https://b.com".into(), WcagLevel::AA, results, 50);
        let risky = normalize(&risky_report);

        let risk = compute_worst_risk(&[empty.normalized, risky.normalized]);
        // Risky page should pull the batch to at least Medium or higher
        assert!(
            matches!(
                risk,
                RiskLevel::Critical | RiskLevel::High | RiskLevel::Medium
            ),
            "batch risk must be elevated when one page has critical issues, got {:?}",
            risk
        );
    }

    fn clean_report(url: &str) -> AuditReport {
        AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100)
    }

    fn batch_error(url: &str, blocked: bool) -> BatchError {
        BatchError {
            url: url.to_string(),
            error: "navigation failed".to_string(),
            blocked_reason: blocked.then(|| "bot wall".to_string()),
        }
    }

    /// #768: a clean report plus a failed URL must never read as complete.
    #[test]
    fn failed_url_is_counted_and_never_an_unqualified_pass() {
        let batch = BatchReport::from_reports(
            vec![clean_report("https://a.com")],
            vec![batch_error("https://b.com", false)],
            10,
        );
        let c = &batch.summary.coverage;
        assert_eq!(
            (c.attempted, c.succeeded, c.failed, c.blocked),
            (2, 1, 1, 0)
        );
        assert_eq!(batch.summary.total_urls, 1);
        assert_ne!(
            batch.summary.audit_quality.status,
            AuditQualityStatus::Complete
        );
        let v = crate::audit::compute_batch_verdict(&batch.summary, &Default::default());
        assert_ne!(v.verdict, crate::audit::Verdict::Pass);
        assert!(v
            .reasons
            .iter()
            .any(|r| r.starts_with("incomplete_scope: 1 of 2")));
    }

    /// The verdict reads the coverage itself, not only the derived quality.
    #[test]
    fn verdict_qualifies_failed_scope_even_with_complete_quality() {
        let mut batch = BatchReport::from_reports(vec![clean_report("https://a.com")], vec![], 10);
        batch.summary.coverage.attempted = 2;
        batch.summary.coverage.failed = 1;
        let v = crate::audit::compute_batch_verdict(&batch.summary, &Default::default());
        assert_eq!(v.verdict, crate::audit::Verdict::Warn);
    }

    #[test]
    fn blocked_and_failed_urls_are_counted_separately() {
        let batch = BatchReport::from_reports(
            vec![clean_report("https://a.com")],
            vec![
                batch_error("https://b.com", true),
                batch_error("https://c.com", false),
            ],
            10,
        );
        let c = &batch.summary.coverage;
        assert_eq!((c.succeeded, c.blocked, c.failed), (1, 1, 1));
        let reasons = &batch.summary.audit_quality.reasons;
        assert!(reasons.contains(&"failed_urls:1".to_string()));
        assert!(reasons.contains(&"blocked_urls:1".to_string()));
    }

    /// A batch where every URL failed has nothing measured: FAIL, exit 2.
    #[test]
    fn fully_failed_batch_fails() {
        let batch = BatchReport::from_reports(
            vec![],
            vec![
                batch_error("https://a.com", false),
                batch_error("https://b.com", true),
            ],
            10,
        );
        assert_eq!(batch.summary.coverage.succeeded, 0);
        assert_eq!(batch.summary.average_score, None);
        assert_eq!(batch.summary.average_over, 0);
        assert_eq!(
            batch.summary.audit_quality.status,
            AuditQualityStatus::Insufficient
        );
        let v = crate::audit::compute_batch_verdict(&batch.summary, &Default::default());
        assert_eq!(v.verdict, crate::audit::Verdict::Fail);
        assert_eq!(v.verdict.exit_code(), 2);
    }

    /// #768: URLs selected but never attempted (budget, cancel) are not a
    /// complete batch, and the denominator names what was measured.
    #[test]
    fn selected_but_never_attempted_urls_qualify_the_batch() {
        let batch = BatchReport::from_reports(vec![clean_report("https://a.com")], vec![], 10)
            .with_sample(SampleMetadata {
                source: "url_file".to_string(),
                total_discovered: 3,
                audited: 3,
                sample_limit: None,
                selection: "all".to_string(),
                is_sample: false,
                strata: vec![],
            });
        let c = &batch.summary.coverage;
        assert_eq!((c.selected, c.attempted, c.succeeded), (3, 1, 1));
        assert_eq!(c.not_attempted(), 2);
        assert!(!c.is_complete());
        assert_eq!(batch.summary.average_over, 1);
        assert_ne!(batch.summary.decision.verdict, crate::audit::Verdict::Pass);
        assert!(batch
            .summary
            .decision
            .reasons
            .iter()
            .any(|r| r.contains("2 not attempted")));
        assert!(batch.summary.audit_quality.qualified_results);
    }

    #[test]
    fn sample_metadata_sets_discovered_and_selected() {
        let batch = BatchReport::from_reports(vec![clean_report("https://a.com")], vec![], 10)
            .with_sample(SampleMetadata {
                source: "sitemap".to_string(),
                total_discovered: 50,
                audited: 1,
                sample_limit: Some(1),
                selection: "first_n".to_string(),
                is_sample: true,
                strata: vec![],
            });
        let c = &batch.summary.coverage;
        assert_eq!(
            (c.discovered, c.selected, c.attempted, c.succeeded),
            (50, 1, 1, 1)
        );
        assert!(c.is_complete());
    }
}
