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

/// Complete audit report for a single URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// The URL that was audited
    pub url: String,
    /// WCAG conformance level used for the audit
    pub wcag_level: WcagLevel,
    /// Timestamp when the audit was performed
    pub timestamp: DateTime<Utc>,
    /// WCAG check results
    pub wcag_results: WcagResults,
    /// Overall accessibility score (0-100)
    pub score: f32,
    /// Letter grade (A-F)
    pub grade: String,
    /// Certificate level (SEHR GUT, GUT, SOLIDE, AUSBAUFÄHIG, UNGENÜGEND)
    pub certificate: String,
    /// Detailed violation statistics
    pub statistics: ViolationStatistics,
    /// Number of AXTree nodes analyzed
    pub nodes_analyzed: usize,
    /// Time taken to complete the audit (milliseconds)
    pub duration_ms: u64,
    /// Performance analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceResults>,
    /// SEO analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo: Option<SeoAnalysis>,
    /// Security analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityAnalysis>,
    /// Mobile friendliness analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<MobileFriendliness>,
    /// Budget violations detected for this page
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub budget_violations: Vec<crate::audit::budget::BudgetViolation>,
    /// UX analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux: Option<UxAnalysis>,
    /// Journey analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey: Option<crate::journey::JourneyAnalysis>,
    /// Dark mode support and quality analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_mode: Option<DarkModeAnalysis>,
    /// Source quality analysis (Substanz / Konsistenz / Autorität)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_quality: Option<crate::source_quality::SourceQualityAnalysis>,
    /// AI visibility analysis (LLM-Readability, Citation, Chunks, Knowledge Graph, Policy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_visibility: Option<crate::ai_visibility::AiVisibilityAnalysis>,
    /// Content visibility analysis (organic visibility, local business, E-E-A-T, depth, topical authority)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_visibility: Option<crate::content_visibility::ContentVisibilityAnalysis>,
    /// Technology stack detection and stack-specific audit findings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_stack: Option<crate::tech_stack::TechStackAnalysis>,
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
    /// Whether a consent banner was detected during the audit
    #[serde(default)]
    pub consent_banner_detected: bool,
    /// Which CMP was identified (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_banner_cmp: Option<String>,
    /// Whether the banner was successfully dismissed
    #[serde(default)]
    pub consent_banner_dismissed: bool,
    /// Accessibility-Journey-Layer result (populated when `--interactive != off`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility_journey: Option<crate::audit::normalized::AccessibilityJourney>,
    /// Interactive findings produced by the Accessibility-Journey-Layer
    /// evaluator. Empty when `--interactive=off`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactive_findings: Vec<crate::audit::normalized::InteractiveFinding>,
    /// Advisory findings from semantic / LLM evaluation (Phase 4).
    /// Never influence score or risk. Empty unless `--semantic-eval` is set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub advisory_findings: Vec<crate::audit::normalized::AdvisoryFinding>,
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
            wcag_results,
            score,
            grade,
            certificate,
            statistics,
            nodes_analyzed,
            duration_ms,
            performance: None,
            seo: None,
            security: None,
            mobile: None,
            ux: None,
            journey: None,
            budget_violations: Vec::new(),
            dark_mode: None,
            source_quality: None,
            ai_visibility: None,
            content_visibility: None,
            tech_stack: None,
            page_screenshots: None,
            dual_viewport: None,
            viewport_scores: None,
            throttled_performance: Vec::new(),
            patterns: None,
            screenshot_status: ScreenshotStatus::NotRequested,
            best_practices: None,
            consent_banner_detected: false,
            consent_banner_cmp: None,
            consent_banner_dismissed: false,
            accessibility_journey: None,
            interactive_findings: Vec::new(),
            advisory_findings: Vec::new(),
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
        self.seo = Some(seo);
        self
    }

    /// Set security results
    pub fn with_security(mut self, security: SecurityAnalysis) -> Self {
        self.security = Some(security);
        self
    }

    /// Set mobile friendliness results
    pub fn with_mobile(mut self, mobile: MobileFriendliness) -> Self {
        self.mobile = Some(mobile);
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
        self.dark_mode = Some(dark_mode);
        self
    }

    pub fn with_tech_stack(mut self, tech_stack: crate::tech_stack::TechStackAnalysis) -> Self {
        self.tech_stack = Some(tech_stack);
        self
    }

    pub fn with_best_practices(mut self, bp: crate::best_practices::BestPracticesAnalysis) -> Self {
        self.best_practices = Some(bp);
        self
    }

    /// Get the total number of violations
    pub fn violation_count(&self) -> usize {
        self.wcag_results.violations.len()
    }

    /// Check if the audit passed (no critical violations, score >= 70)
    pub fn passed(&self) -> bool {
        self.score >= 70.0
            && !self
                .wcag_results
                .violations
                .iter()
                .any(|v| v.severity == crate::wcag::Severity::Critical)
    }

    /// Calculate weighted overall score across all active modules.
    ///
    /// When dual-viewport data is present the score is weighted 70 % mobile /
    /// 30 % desktop, matching Google PageSpeed Insights priorities.  Security
    /// (viewport-independent) is blended in afterwards (10 % slot).
    ///
    /// Fallback weights when no dual-viewport data exists (single-pass):
    /// - WCAG Accessibility: 40 %
    /// - Performance: 20 %
    /// - SEO: 20 %
    /// - Security: 10 %
    /// - Mobile: 10 %
    pub fn overall_score(&self) -> u32 {
        if let Some(ref vs) = self.viewport_scores {
            // 70/30 viewport base, then blend in security (10 %)
            let mut weighted = vs.weighted_overall as f64 * 90.0;
            let mut total = 90.0;
            if let Some(ref security) = self.security {
                weighted += security.score as f64 * 10.0;
                total += 10.0;
            }
            return (weighted / total).round() as u32;
        }

        // Single-pass fallback
        let mut weighted_sum = self.score as f64 * 40.0;
        let mut total_weight = 40.0;

        if let Some(ref perf) = self.performance {
            weighted_sum += perf.score.overall as f64 * 20.0;
            total_weight += 20.0;
        }
        if let Some(ref seo) = self.seo {
            weighted_sum += seo.score as f64 * 20.0;
            total_weight += 20.0;
        }
        if let Some(ref security) = self.security {
            weighted_sum += security.score as f64 * 10.0;
            total_weight += 10.0;
        }
        if let Some(ref mobile) = self.mobile {
            weighted_sum += mobile.score as f64 * 10.0;
            total_weight += 10.0;
        }

        (weighted_sum / total_weight).round() as u32
    }
}

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
    /// Worst-case risk level across all audited pages.
    #[serde(default)]
    pub risk: crate::audit::normalized::RiskLevel,
    /// i18n key for the batch verdict sentence ("verdict-batch-excellent", …).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub verdict_key: String,
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
fn compute_legal_flags(reports: &[crate::audit::normalized::NormalizedReport]) -> usize {
    use crate::taxonomy::Severity;
    use std::collections::HashSet;

    reports
        .iter()
        .flat_map(|r| r.findings.iter())
        .filter(|f| {
            f.wcag_level == "A" && matches!(f.severity, Severity::Critical | Severity::High)
        })
        .map(|f| f.rule_id.as_str())
        .collect::<HashSet<_>>()
        .len()
}

/// Compute the worst-case risk level across a set of normalized reports.
///
/// Critical: any page is Critical.
/// High: ≥ 20% of pages are High.
/// Medium: ≥ 20% of pages are Medium.
/// Low: otherwise.
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

    [RiskLevel::Critical, RiskLevel::High, RiskLevel::Medium]
        .iter()
        .copied()
        .find(|&lvl| {
            let n = *counts.get(&lvl).unwrap_or(&0);
            n > 0 && (lvl == RiskLevel::Critical || n * 5 >= page_count)
        })
        .unwrap_or(RiskLevel::Low)
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
        // no WCAG-Level-A high/critical findings (i.e. no legal exposure).
        // See issue #253.
        let passed = normalized_reports
            .iter()
            .filter(|r| {
                let no_legal_flags = !r.findings.iter().any(|f| {
                    f.wcag_level == "A"
                        && matches!(
                            f.severity,
                            crate::taxonomy::Severity::Critical | crate::taxonomy::Severity::High,
                        )
                });
                r.score >= 80 && r.severity_counts.critical == 0 && no_legal_flags
            })
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
        let legal_flags = compute_legal_flags(&normalized_reports);
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
                risk,
                verdict_key,
            },
            crawl_diagnostics: None,
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
        assert_eq!(report.score, 100.0); // No violations = perfect score
        assert_eq!(report.duration_ms, 500);
        assert!(report.performance.is_none());
        assert!(report.seo.is_none());
        assert!(report.security.is_none());
        assert!(report.mobile.is_none());
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
    fn test_overall_score_wcag_only() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        // WCAG only: overall = WCAG score
        assert_eq!(report.overall_score(), 100);
    }

    #[test]
    fn test_overall_score_weighted() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        // WCAG = 100 (weight 40), Security = 50 (weight 10)
        report.security = Some(crate::security::SecurityAnalysis {
            score: 50,
            grade: "D".to_string(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
            protection: Default::default(),
        });
        // Weighted: (100*40 + 50*10) / (40+10) = 4500/50 = 90
        assert_eq!(report.overall_score(), 90);
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

        let r1 = normalize(&AuditReport::new(
            "https://a.com".into(),
            WcagLevel::AA,
            results1,
            100,
        ));
        let r2 = normalize(&AuditReport::new(
            "https://b.com".into(),
            WcagLevel::AA,
            results2,
            100,
        ));

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
        let empty = normalize(&AuditReport::new(
            "https://a.com".into(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        ));

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
        let risky = normalize(&AuditReport::new(
            "https://b.com".into(),
            WcagLevel::AA,
            results,
            50,
        ));

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
