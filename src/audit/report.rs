//! Audit Report data structure
//!
//! Contains the complete results of an accessibility audit.

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
    /// Aggregate performance score (0–100)
    pub score: u32,
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
    /// Keys: "tbt_zero_heavy_page", "speed_index_fallback_to_lcp",
    /// "tti_fallback_to_lcp", "inp_not_measured".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub measurement_warnings: Vec<String>,
}

impl AuditReport {
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
            },
            duration_ms,
            performance: None,
            security: None,
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

    /// Check if the audit passed (no critical violations, score >= 70)
    pub fn passed(&self) -> bool {
        self.accessibility.score >= 70.0
            && !self
                .accessibility
                .wcag_results
                .violations
                .iter()
                .any(|v| v.severity == crate::wcag::Severity::Critical)
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
    /// representative sample apart from full domain coverage (issue #261).
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
    /// How the audited subset was chosen: `"first_n"` (sitemap/discovery order)
    /// or `"all"` when every discovered URL was audited.
    pub selection: String,
    /// True when fewer URLs were audited than discovered — i.e. this is a sample,
    /// not full coverage.
    pub is_sample: bool,
}

/// Sitemap URL validation result for sitemap-driven batch reports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SitemapDiagnostics {
    /// Number of sitemap URLs checked via HTTP.
    pub checked_urls: usize,
    /// Sitemap entries that do not resolve to a direct canonical 200 response,
    /// or that are marked noindex.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub http_issues: Vec<SitemapHttpIssue>,
    /// URLs present in the sitemap but not linked by any audited page.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub orphan_sitemap_urls: Vec<String>,
    /// Internal targets linked by audited pages but absent from the sitemap.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub linked_not_in_sitemap: Vec<String>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub broken_internal_links: Vec<BrokenLink>,
    /// Number of unique external links that were status-checked
    pub checked_external_links: usize,
    /// Broken external links (4xx/5xx or fetch failure)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub broken_external_links: Vec<BrokenLink>,
    /// Links with more than 1 redirect hop
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
}

/// One WCAG rule that recurred across multiple pages in a batch audit.
/// Domain-level type; used by BatchSummary and the JSON output formatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringRule {
    pub rule_id: String,
    pub title: String,
    pub wcag_criterion: String,
    pub wcag_level: String,
    pub severity: crate::taxonomy::Severity,
    /// Number of pages where this rule fired.
    pub affected_pages: usize,
    /// Sum of `occurrence_count` over all affected pages.
    pub total_occurrences: usize,
}

/// Summary statistics for a batch audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Total URLs audited
    pub total_urls: usize,
    /// URLs that passed
    pub passed: usize,
    /// URLs that failed
    pub failed: usize,
    /// Average score across all URLs
    pub average_score: f64,
    /// Total violations found
    pub total_violations: usize,
    /// Top-10 WCAG rules by page frequency, computed across all audited pages.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_recurring_rules: Vec<RecurringRule>,
    /// Number of distinct WCAG + SEO rule IDs that fired anywhere across all pages.
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
    /// i18n key for the batch verdict sentence ("verdict-batch-excellent", …).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub verdict_key: String,
    /// WCAG findings verified to share one template/component root cause
    /// across multiple pages — see `audit::template_dedup`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub template_clusters: Vec<crate::audit::template_dedup::TemplateCluster>,
}

/// Compute top-10 recurring WCAG rules and total violated-rule count across
/// a set of normalized reports. Used by `BatchReport::from_reports` and the
/// JSON formatter for single-page summary blocks.
///
/// Returns `(top_recurring_rules, violated_rule_count)`.
pub fn compute_recurring_rules(
    reports: &[crate::audit::normalized::NormalizedReport],
) -> (Vec<RecurringRule>, usize) {
    use std::collections::{HashMap, HashSet};

    let violated_rule_count = reports
        .iter()
        .flat_map(|r| r.findings.iter().map(|f| f.rule_id.as_str()))
        .collect::<HashSet<_>>()
        .len();

    struct Acc {
        title: String,
        wcag_criterion: String,
        wcag_level: String,
        severity: crate::taxonomy::Severity,
        affected_pages: usize,
        total_occurrences: usize,
    }

    let mut by_rule: HashMap<String, Acc> = HashMap::new();
    for report in reports {
        for f in report.findings.iter().filter(|f| f.category == "wcag") {
            let entry = by_rule.entry(f.rule_id.clone()).or_insert(Acc {
                title: f.title.clone(),
                wcag_criterion: f.wcag_criterion.clone(),
                wcag_level: f.wcag_level.clone(),
                severity: f.severity,
                affected_pages: 0,
                total_occurrences: 0,
            });
            entry.affected_pages += 1;
            entry.total_occurrences += f.occurrence_count;
        }
    }

    let mut rules: Vec<RecurringRule> = by_rule
        .into_iter()
        .map(|(rule_id, a)| RecurringRule {
            rule_id,
            title: a.title,
            wcag_criterion: a.wcag_criterion,
            wcag_level: a.wcag_level,
            severity: a.severity,
            affected_pages: a.affected_pages,
            total_occurrences: a.total_occurrences,
        })
        .collect();
    rules.sort_by(|a, b| {
        b.affected_pages
            .cmp(&a.affected_pages)
            .then_with(|| b.total_occurrences.cmp(&a.total_occurrences))
            .then_with(|| b.severity.cmp(&a.severity))
    });
    rules.truncate(10);

    (rules, violated_rule_count)
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
    use crate::taxonomy::Severity;
    use std::collections::HashSet;

    let mut rule_ids: HashSet<&str> = reports
        .iter()
        .flat_map(|r| r.findings.iter())
        .filter(|f| {
            f.wcag_level == "A" && matches!(f.severity, Severity::Critical | Severity::High)
        })
        .map(|f| f.rule_id.as_str())
        .collect();

    let sr_only_legal_exposure = reports.iter().any(|r| {
        let findings_based = r
            .findings
            .iter()
            .filter(|f| {
                f.wcag_level == "A" && matches!(f.severity, Severity::Critical | Severity::High)
            })
            .count();
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
        let normalized_reports: Vec<crate::audit::normalized::NormalizedReport> = reports
            .iter()
            .map(|r| crate::audit::normalized::normalize(r).normalized)
            .collect();
        // Pass criterion: accessibility score ≥ 80, no critical findings, and
        // no legal exposure. See issue #253. Uses the page's own risk
        // assessment (`r.risk.legal_flags`) rather than recomputing from
        // findings, so screen-reader-derived legal exposure (#484) is not
        // silently dropped in batch mode while it fails the single-page
        // verdict (#QA-024).
        let passed = normalized_reports
            .iter()
            .filter(|r| r.score >= 80 && r.severity_counts.critical == 0 && r.risk.legal_flags == 0)
            .count();
        let failed = total_urls - passed;

        let average_score = if total_urls > 0 {
            normalized_reports
                .iter()
                .map(|r| r.score as f64)
                .sum::<f64>()
                / total_urls as f64
        } else {
            0.0
        };

        let total_violations = normalized_reports
            .iter()
            .flat_map(|r| r.findings.iter())
            .map(|f| f.occurrence_count)
            .sum();

        let (top_recurring_rules, violated_rule_count) =
            compute_recurring_rules(&normalized_reports);
        let template_clusters =
            crate::audit::template_dedup::detect_template_clusters(&normalized_reports);
        let legal_flags = compute_legal_flags(&normalized_reports);
        let blocking_issues = normalized_reports
            .iter()
            .map(|r| r.risk.blocking_issues)
            .sum();
        let risk = compute_worst_risk(&normalized_reports);
        let verdict_key = {
            let s = average_score.round() as u32;
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
                passed,
                failed,
                average_score,
                total_violations,
                top_recurring_rules,
                violated_rule_count,
                legal_flags,
                blocking_issues,
                risk,
                verdict_key,
                template_clusters,
            },
            crawl_diagnostics: None,
            sitemap_diagnostics: None,
            consistency: None,
            sample: None,
            total_duration_ms,
        };
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
        self.sample = Some(sample);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::WcagResults;

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
        assert_eq!(batch.summary.average_score, 100.0);
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
        // No violations: no recurring rules, no violated rules, Low risk
        assert!(batch.summary.top_recurring_rules.is_empty());
        assert_eq!(batch.summary.violated_rule_count, 0);
        assert_eq!(batch.summary.legal_flags, 0);
        assert_eq!(batch.summary.risk, crate::audit::normalized::RiskLevel::Low);
    }

    #[test]
    fn compute_recurring_rules_aggregates_across_pages() {
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

        let (rules, violated_count) = compute_recurring_rules(&[r1.normalized, r2.normalized]);
        assert_eq!(violated_count, 1, "one distinct rule fired");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].affected_pages, 2, "rule appeared on both pages");
        assert_eq!(rules[0].total_occurrences, 2);
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
}
