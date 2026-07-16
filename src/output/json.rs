//! JSON Output Formatter — Unified Report Envelope v2.0
//!
//! Single- and batch reports share one envelope (`UnifiedReport`):
//! `schema_version` + `report_type` discriminants, a uniform `summary`, and
//! `pages[]` (1 element for single, N for batch). Per-page module detail lives
//! under `pages[i].detail` and is omitted for batch reports.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::audit::normalized::{AuditContext, NormalizedReport};
use crate::audit::verdict::{Verdict, VerdictResult};
use crate::audit::{
    compute_recurring_rules, AccessibilityScorer, AuditReport, BatchReport, RecurringRule,
    SampleMetadata,
};
use crate::error::Result;
use crate::output::builder::{build_batch_presentation_with_normalized, build_view_model};
use crate::output::explanations::get_explanation;
use crate::output::module::ReportModule as _;
use crate::output::report_model::{ReportConfig, UrlMatrixRow};

/// Current envelope schema version.
const SCHEMA_VERSION: &str = "2.0";

mod detail;
mod helpers;

use detail::{build_batch_detail, build_page, DetailContext};
use helpers::{
    aggregate_occurrences, aggregate_severity, avg_module_score, batch_report_timestamp,
    build_accessibility_score_breakdown, build_decision_actions, build_en301549_batch_rollup,
    build_internal_comparison, build_management_risks, build_wcag_coverage_for_level,
    build_wcag_coverage_summary, normalized_module_score,
};

#[cfg(test)]
mod tests;

// ─── Public entry points ──────────────────────────────────────────────────────

/// Generate single-report JSON from a live audit context.
pub fn format_json_normalized(
    ctx: &AuditContext<'_>,
    report: &AuditReport,
    pretty: bool,
) -> Result<String> {
    UnifiedReport::single(ctx, report).to_json(pretty)
}

/// Generate single-report JSON from cached normalized data only.
///
/// Module detail is limited to whatever the cached `NormalizedReport` still
/// carries (raw module data is not persisted, so `detail.modules` is sparse).
pub fn format_json_cached(normalized: &NormalizedReport, pretty: bool) -> Result<String> {
    UnifiedReport::single_from_normalized(normalized).to_json(pretty)
}

/// Generate batch-report JSON. `pages[i].detail` is omitted.
pub fn format_json_batch(batch_report: &BatchReport, pretty: bool) -> Result<String> {
    UnifiedReport::batch(batch_report).to_json(pretty)
}

// ─── Envelope ─────────────────────────────────────────────────────────────────

/// Unified report envelope — works for both single and batch outputs.
#[derive(Debug, Serialize)]
pub struct UnifiedReport {
    pub schema_version: &'static str,
    /// `"single"` or `"batch"`.
    pub report_type: &'static str,
    /// Top-level tool version — duplicated from `metadata.tool` for ease of consumption.
    pub tool_version: &'static str,
    pub metadata: ReportMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_scope: Option<crate::audit::AuditScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_environment: Option<crate::audit::ExecutionEnvironment>,
    pub audit_quality: crate::audit::AuditQuality,
    /// Definitions for report-specific scores and ambiguous finding counters.
    /// General values such as URL counts and durations remain self-describing.
    pub metric_context: MetricContext,
    pub summary: UnifiedSummary,
    /// Batch only — how the audited URLs were discovered and sampled. Lets a
    /// consumer tell a representative sample apart from full coverage (#261).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<SampleMetadata>,
    pub pages: Vec<PageEntry>,
    /// Batch only — compact per-URL score matrix.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub url_matrix: Vec<UrlMatrixRow>,
    /// Batch only — internal comparison across audited URLs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_comparison: Option<InternalComparison>,
    /// Batch only — crawl diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crawl_diagnostics: Option<serde_json::Value>,
    /// Batch only — sitemap HTTP/indexability and link-graph diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sitemap_diagnostics: Option<serde_json::Value>,
    /// Batch-only canonical domain/portfolio analyses that are already
    /// computed for the PDF presentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_analysis: Option<serde_json::Value>,
    /// Separately written artifacts that belong to this audit but intentionally
    /// remain outside the main JSON envelope.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ArtifactDescriptor>,
    /// Batch only — per-URL audit errors.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<serde_json::Value>,
    /// Serialization errors encountered while building this report.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collection_errors: Vec<ReportError>,
    /// CI verdict derived from this report's findings.
    pub verdict: Verdict,
    /// Reasons that drove the verdict away from Pass.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verdict_reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MetricContext {
    pub score_scale: ScoreScale,
    pub score_definitions: Vec<MetricDefinition>,
    pub count_definitions: Vec<MetricDefinition>,
}

#[derive(Debug, Serialize)]
pub struct ScoreScale {
    pub minimum: u32,
    pub maximum: u32,
    pub higher_is_better: bool,
}

#[derive(Debug, Serialize)]
pub struct MetricDefinition {
    pub field: &'static str,
    pub unit: &'static str,
    pub meaning: &'static str,
}

fn metric_context(report_type: &'static str) -> MetricContext {
    let accessibility_meaning = if report_type == "batch" {
        "Average of the canonical page accessibility scores. Each dual-viewport page score is 70% mobile and 30% desktop; single-viewport pages use their measured accessibility score."
    } else {
        "Canonical accessibility score. With dual-viewport data it is 70% mobile and 30% desktop; otherwise it is the measured single-viewport score. It covers automatically evaluated WCAG findings only."
    };
    let overall_meaning = if report_type == "batch" {
        "Average of the page-level overall scores across the audited URLs."
    } else {
        "Weighted score across contributing measured modules. In a dual-viewport audit, viewport module results are blended first and Security contributes 10% when available."
    };

    MetricContext {
        score_scale: ScoreScale {
            minimum: 0,
            maximum: 100,
            higher_is_better: true,
        },
        score_definitions: vec![
            MetricDefinition {
                field: "summary.accessibility_score",
                unit: "points",
                meaning: accessibility_meaning,
            },
            MetricDefinition {
                field: "summary.overall_score",
                unit: "points",
                meaning: overall_meaning,
            },
            MetricDefinition {
                field: "summary.score",
                unit: "points",
                meaning: "Compatibility alias for summary.overall_score.",
            },
            MetricDefinition {
                field: "summary.grade / summary.certificate",
                unit: "classification",
                meaning: "Classification derived from summary.overall_score; risk gates can restrict the certificate without changing the numeric score.",
            },
            MetricDefinition {
                field: "pages[].module_scores[].score",
                unit: "points",
                meaning: "Module-specific 0-100 score. Scores from different modules use different measured or heuristic inputs and are not interchangeable raw measurements.",
            },
            MetricDefinition {
                field: "pages[].detail.modules.*.score and nested dimension scores",
                unit: "points",
                meaning: "Module- or dimension-specific 0-100 score; higher is better. The surrounding module and measurement_type identify whether the value is measured or heuristic.",
            },
            MetricDefinition {
                field: "pages[].detail.viewport_scores",
                unit: "points",
                meaning: "Desktop and mobile accessibility/overall scores on the 0-100 scale. weighted_overall is the 70% mobile and 30% desktop module blend before the optional Security contribution; the canonical accessibility blend is recorded in score_breakdown.",
            },
            MetricDefinition {
                field: "summary.*_score / pages[].*_score",
                unit: "points",
                meaning: "Named module score on the 0-100 scale; higher is better. Throttled and Lighthouse performance scores are lab results, not field/RUM data.",
            },
            MetricDefinition {
                field: "summary.accessibility_score_breakdown[]",
                unit: "points and percent",
                meaning: "Accessibility area score (0-100, higher is better), its contribution weight in percent, and estimated_lost_points relative to 100.",
            },
            MetricDefinition {
                field: "pages[].risk.score",
                unit: "risk points",
                meaning: "Independent 0-100 risk index; unlike quality scores, higher means more risk. The adjacent level, threshold, and driven_by fields explain its classification.",
            },
            MetricDefinition {
                field: "pages[].principle_coverage.*.ratio",
                unit: "ratio",
                meaning: "Share from 0 to 1 of automatically evaluated checks without a finding for that WCAG principle; it is coverage context and does not affect the score.",
            },
            MetricDefinition {
                field: "pages[].findings[].priority_score",
                unit: "relative priority",
                meaning: "Unbounded ranking value calculated as severity weight multiplied by occurrence reach and divided by estimated effort; higher values should be addressed earlier and are not percentages.",
            },
            MetricDefinition {
                field: "pages[].findings[].score_impact",
                unit: "penalty points",
                meaning: "Rule-specific base and maximum deductions used by accessibility scoring; scaling states how repeated occurrences are condensed.",
            },
        ],
        count_definitions: vec![
            MetricDefinition {
                field: "violation_count / occurrence_counts",
                unit: "WCAG occurrences",
                meaning: "Affected WCAG element or page occurrences; repeated instances of the same rule count separately.",
            },
            MetricDefinition {
                field: "violated_rule_count / severity_counts",
                unit: "distinct WCAG finding groups",
                meaning: "Distinct violated WCAG rule groups, not affected elements.",
            },
            MetricDefinition {
                field: "finding_count / finding_occurrence_count",
                unit: "all-category findings",
                meaning: "Finding rows and occurrences across WCAG, SEO, and every other reported category.",
            },
        ],
    }
}

/// Uniform summary — identical field names for single and batch.
#[derive(Debug, Serialize)]
pub struct UnifiedSummary {
    pub url_count: usize,
    pub accessibility_score: u32,
    pub overall_score: u32,
    /// Alias for `overall_score` — kept for consumer compatibility.
    pub score: u32,
    pub grade: String,
    pub certificate: String,
    pub risk_level: crate::audit::normalized::RiskLevel,
    /// WCAG violation occurrences only.
    pub violation_count: usize,
    /// Distinct finding rows across every category (WCAG, SEO, ...).
    pub finding_count: usize,
    /// Finding occurrences across every category (WCAG, SEO, ...).
    pub finding_occurrence_count: usize,
    /// Anzahl unterschiedlicher Findings (eine Zeile pro Regel/Severity).
    pub severity_counts: crate::audit::normalized::SeverityCounts,
    pub severity_counts_scope: String,
    /// WCAG element/page occurrences per severity.
    pub occurrence_counts: crate::audit::normalized::SeverityCounts,
    pub occurrence_counts_scope: String,
    pub passed_url_count: usize,
    pub failed_url_count: usize,
    /// Anzahl unterschiedlicher WCAG-Regeln, die irgendwo geprüfte URLs verletzt haben
    /// (über alle Pages dedupliziert).
    #[serde(default)]
    pub violated_rule_count: usize,
    /// Häufigste WCAG-Regelverstöße (max. 10 Einträge, sortiert nach Occurrences).
    /// Bei Single-Reports bezogen auf die eine Seite; bei Batch über alle Pages aggregiert.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top_recurring_rules: Vec<RecurringRule>,
    /// WCAG findings verified to share one template/component root cause
    /// across multiple pages (batch only; always empty on single reports).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub template_clusters: Vec<crate::audit::TemplateCluster>,
    /// Domain-wide EN 301 549 clause roll-up: worst status per clause across
    /// all pages plus the affected-page count (batch only; always empty on
    /// single reports, which carry the full per-page annex under
    /// `pages[0].detail.en301549_annex` instead). Never empty for a batch
    /// report with ≥1 page — all 50 clauses are always represented.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub en301549_rollup: Vec<En301549BatchClauseRollup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_throttled_avg_score: Option<u32>,
    /// LhMobile throttled score (Lighthouse mobile preset). Present only on
    /// single-page reports when the throttled pass ran (#289).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lh_mobile_score: Option<u32>,
    /// Automated WCAG coverage scope shown near the executive summary.
    pub wcag_coverage: WcagCoverageSummary,
    /// Accessibility score explanation by weighted topic.
    pub accessibility_score_breakdown: Vec<AccessibilityScoreComponent>,
    /// Management-oriented risk view derived from findings and module scores.
    pub management_risks: Vec<ManagementRisk>,
    /// Decision-oriented top actions combining risk, impact, complexity, and reach.
    pub top_actions: Vec<DecisionAction>,
    /// Cross-page duplicate content groups (batch only): identical title,
    /// meta description, or H1 shared across multiple pages (#423).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub duplicate_content: Vec<crate::output::report_model::DuplicateContentGroup>,
    /// Per-page canonical conflicts (batch only): noindex conflict or og:url
    /// mismatch (#423).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub canonical_issues: Vec<crate::output::report_model::CanonicalIssue>,
    /// Non-reciprocal hreflang relationships between audited pages (batch only,
    /// #423).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hreflang_issues: Vec<crate::output::report_model::HreflangIssue>,
    /// Sitemap entries with HTTP/indexability issues (batch only, #471).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sitemap_http_issues: Vec<crate::audit::SitemapHttpIssue>,
    /// URLs present in the sitemap but not internally linked by audited pages.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub orphan_sitemap_urls: Vec<String>,
    /// Internal targets linked by audited pages but absent from the sitemap.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub linked_not_in_sitemap: Vec<String>,
    /// Site-wide commerce roll-up (batch only): union of mandatory/trust-page
    /// links across the audited shop pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commerce: Option<crate::commerce::CommerceSiteSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactDescriptor {
    pub kind: &'static str,
    pub media_type: &'static str,
    pub delivery: &'static str,
    pub path_template: &'static str,
    pub status: crate::audit::ExecutionStatus,
}

fn artifacts_for(normalized: &NormalizedReport) -> Vec<ArtifactDescriptor> {
    if normalized.screen_reader.is_some() {
        vec![ArtifactDescriptor {
            kind: "screen_reader_audit",
            media_type: "application/json",
            delivery: "sidecar",
            path_template: "<primary-output-stem>.screen-reader.json",
            status: crate::audit::ExecutionStatus::Completed,
        }]
    } else {
        Vec::new()
    }
}

#[derive(Debug, Serialize)]
pub struct WcagCoverageSummary {
    pub level: String,
    pub automated_criteria: usize,
    pub manual_review_criteria: usize,
    pub total_wcag_aa_criteria: usize,
    pub note: String,
}

/// EN 301 549 (chapter 9, "Web") clause annex — technical evidence only, not
/// a Barrierefreiheitserklärung. Canonical English (#406); always present on
/// single-report `detail` (never gated behind a flag — cheap, derived,
/// additive). The PDF section built from the same data IS flag-gated (see
/// `--annex en301549`).
#[derive(Debug, Serialize)]
pub struct En301549Annex {
    pub standard_version: &'static str,
    pub mapping_version: u32,
    /// Fixed scope label for this annex: the automatically testable subset
    /// of EN 301 549 chapter 9 (Web).
    pub scope: &'static str,
    pub clauses: Vec<En301549ClauseEntry>,
    /// Count of WCAG findings whose criterion is outside the 50-clause WCAG
    /// 2.1 A/AA table (AAA criteria, WCAG-2.2-only criteria) — excluded from
    /// `clauses` but not silently dropped.
    pub out_of_standard_findings: usize,
    pub out_of_scope_chapters: Vec<OutOfScopeChapterEntry>,
    /// Cautious, non-legal-conclusion disclaimer. Needs lawyer review before
    /// customer-facing use — see the module-level note in `wcag::en301549`.
    pub disclaimer: String,
}

#[derive(Debug, Serialize)]
pub struct En301549ClauseEntry {
    pub en_clause: &'static str,
    pub wcag: &'static str,
    pub level: &'static str,
    pub title: &'static str,
    pub status: En301549ClauseStatusKind,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<En301549FindingRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum En301549ClauseStatusKind {
    ViolationsFound,
    NoViolationsAutomated,
    ManualReviewRequired,
    PartialAutomatedEvaluation,
    NotEvaluated,
}

#[derive(Debug, Serialize)]
pub struct En301549FindingRef {
    pub rule_id: String,
    pub occurrences: usize,
}

#[derive(Debug, Serialize)]
pub struct OutOfScopeChapterEntry {
    pub chapter: &'static str,
    pub title: &'static str,
}

/// Batch-only, domain-wide roll-up for one EN 301 549 clause: the worst
/// status observed across all audited pages, plus how many pages actually
/// have a confirmed violation for it.
#[derive(Debug, Serialize)]
pub struct En301549BatchClauseRollup {
    pub en_clause: &'static str,
    pub wcag: &'static str,
    pub level: &'static str,
    pub title: &'static str,
    pub status: En301549ClauseStatusKind,
    pub affected_pages: usize,
}

#[derive(Debug, Serialize)]
pub struct AccessibilityScoreComponent {
    pub area: String,
    pub score: u32,
    pub weight_pct: u32,
    pub estimated_lost_points: u32,
    pub main_driver: String,
}

#[derive(Debug, Serialize)]
pub struct ManagementRisk {
    pub dimension: String,
    pub level: String,
    pub rationale: String,
}

#[derive(Debug, Serialize)]
pub struct DecisionAction {
    pub title: String,
    pub risk: String,
    pub priority: String,
    pub complexity: String,
    pub occurrence_count: usize,
    pub root_cause: String,
    pub expected_impact: String,
}

#[derive(Debug, Serialize)]
pub struct InternalComparison {
    pub module_extremes: Vec<ModuleExtreme>,
    pub outlier_urls: Vec<UrlOutlier>,
    pub root_causes: Vec<RootCauseSummary>,
}

#[derive(Debug, Serialize)]
pub struct ModuleExtreme {
    pub module: String,
    pub best_url: String,
    pub best_score: u32,
    pub worst_url: String,
    pub worst_score: u32,
}

#[derive(Debug, Serialize)]
pub struct UrlOutlier {
    pub url: String,
    pub accessibility_score: u32,
    pub batch_average: u32,
    pub delta_points: i32,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RootCauseSummary {
    pub title: String,
    pub occurrence_count: usize,
    pub affected_urls: usize,
    pub classification: String,
}

/// One audited page. `detail` is present for single reports, omitted for batch.
#[derive(Debug, Serialize)]
pub struct PageEntry {
    pub url: String,
    pub accessibility_score: u32,
    pub overall_score: u32,
    pub grade: String,
    pub certificate: String,
    /// WCAG violation occurrences only.
    pub violation_count: usize,
    /// Distinct finding rows across every category (WCAG, SEO, ...).
    pub finding_count: usize,
    /// Finding occurrences across every category (WCAG, SEO, ...).
    pub finding_occurrence_count: usize,
    /// Number of distinct WCAG rules that fired — `findings[].length` for wcag-category entries.
    pub violated_rule_count: usize,
    pub severity_counts: crate::audit::normalized::SeverityCounts,
    pub severity_counts_scope: String,
    /// Element-Occurrences je Severity (Summe `occurrence_count` über alle WCAG-Findings).
    pub occurrence_counts: crate::audit::normalized::SeverityCounts,
    pub occurrence_counts_scope: String,
    /// AX-tree node count (accessibility tree, not DOM). Can exceed `dom_nodes`
    /// because the browser's accessibility tree includes virtual/internal nodes
    /// and roles not present in the HTML DOM. This is expected behavior.
    pub nodes_analyzed: usize,
    pub duration_ms: u64,
    pub module_scores: Vec<crate::audit::normalized::ModuleScoreEntry>,
    /// Shortcut scores derived from `module_scores`. Present only when the module ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey_score: Option<u32>,
    /// How `overall_score` was computed: `"module_weighted"` or `"viewport_weighted"`.
    pub score_calculation_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_breakdown: Option<crate::audit::normalized::ScoreBreakdown>,
    pub risk: crate::audit::normalized::RiskAssessment,
    pub principle_coverage: crate::audit::PrincipleCoverage,
    pub findings: Vec<crate::audit::normalized::NormalizedFinding>,
    pub audit_flags: Vec<crate::audit::normalized::AuditFlag>,
    pub audit_quality: crate::audit::AuditQuality,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_runs: Vec<crate::audit::ModuleRun>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rule_outcomes: Vec<crate::wcag::RuleOutcome>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accessibility_assessments: Vec<crate::audit::normalized::AccessibilityAssessment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation: Option<crate::audit::NavigationSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent: Option<crate::audit::ConsentAuditState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_privacy: Option<crate::audit::ConsentPrivacySnapshot>,
    /// Findings produced by the Accessibility-Journey-Layer (phase 2+).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactive_findings: Vec<crate::audit::normalized::InteractiveFinding>,
    /// Reproducible journey traces. Present only when `--interactive != off`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessibility_journey: Option<crate::audit::normalized::AccessibilityJourney>,
    /// Compact screen-reader audit (reading-order quality, issues, BFSG verdict).
    /// The full reading sequence stays in the sidecar JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen_reader: Option<crate::screen_reader::ScreenReaderSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_profile: Option<BatchContentProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<PageDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchContentProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_score: Option<u32>,
    pub biggest_lever: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub topic_terms: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top_issues: Vec<String>,
}

/// Per-page module detail blob — single reports only.
#[derive(Debug, Serialize)]
pub struct PageDetail {
    /// Fix guidance entries — always present (may be an empty array when there
    /// are no findings). See issue #253.
    pub fix_guidance: Vec<FixGuidance>,
    /// EN 301 549 (chapter 9, "Web") clause annex — always present, in single
    /// AND batch reports (same "always present" rule as `fix_guidance`,
    /// issue #256). See `En301549Annex` doc comment.
    pub en301549_annex: En301549Annex,
    pub modules: ModuleBlob,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub confidence_summary: Vec<OutputConfidenceSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<OutputCapabilitySignal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_scores: Option<crate::audit::ViewportScores>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_comparison: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub budget_violations: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub throttled_performance: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_status: Option<serde_json::Value>,
    /// Serialization errors encountered while building this page's detail.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collection_errors: Vec<ReportError>,
}

/// Module detail data, grouped under `detail.modules`.
#[derive(Debug, Default, Serialize)]
pub struct ModuleBlob {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dual_viewport: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_experience: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_mode: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_quality: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_visibility: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_visibility: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_stack: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_practices: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commerce: Option<serde_json::Value>,
}

/// AI-oriented fix guidance for a single finding group.
#[derive(Debug, Serialize)]
pub struct FixGuidance {
    pub rule_id: String,
    pub title: String,
    pub wcag_criterion: String,
    pub severity: String,
    pub risk: String,
    pub remediation_priority: String,
    pub complexity: String,
    pub complexity_reason: String,
    pub confidence: String,
    pub false_positive_risk: String,
    pub verification: String,
    pub expected_impact: String,
    pub bfsg_relevance: String,
    pub occurrence_count: usize,
    pub problem: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typical_cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_example: Option<CodeExample>,
    pub affected_selectors: Vec<String>,
}

/// Bad/good code example pair.
#[derive(Debug, Serialize)]
pub struct CodeExample {
    pub bad: String,
    pub good: String,
}

#[derive(Debug, Serialize)]
pub struct OutputConfidenceSignal {
    pub signal: String,
    pub assessment: String,
}

#[derive(Debug, Serialize)]
pub struct OutputCapabilitySignal {
    pub signal: String,
    pub source: String,
    pub confidence: String,
    pub surfaces: Vec<String>,
    pub note: String,
}

/// A non-fatal error that occurred during report collection or serialization.
#[derive(Debug, Serialize)]
pub struct ReportError {
    pub module: &'static str,
    pub error_type: &'static str,
    pub reason: String,
}

/// Report metadata block.
#[derive(Debug, Serialize)]
pub struct ReportMetadata {
    pub tool: String,
    pub timestamp: DateTime<Utc>,
    pub wcag_level: String,
    pub execution_time_ms: u64,
}

fn aggregate_audit_quality(reports: &[NormalizedReport]) -> crate::audit::AuditQuality {
    let failed_rule_checks = reports
        .iter()
        .map(|report| report.execution.quality.failed_rule_checks)
        .sum();
    let partial_or_failed_modules = reports
        .iter()
        .map(|report| report.execution.quality.partial_or_failed_modules)
        .sum();
    let status = if reports.iter().any(|report| {
        report.execution.quality.status == crate::audit::AuditQualityStatus::Insufficient
    }) {
        crate::audit::AuditQualityStatus::Insufficient
    } else if reports
        .iter()
        .any(|report| report.execution.quality.status == crate::audit::AuditQualityStatus::Partial)
    {
        crate::audit::AuditQualityStatus::Partial
    } else {
        crate::audit::AuditQualityStatus::Complete
    };
    crate::audit::AuditQuality {
        status,
        qualified_results: status != crate::audit::AuditQualityStatus::Complete,
        failed_rule_checks,
        partial_or_failed_modules,
        reasons: reports
            .iter()
            .filter(|report| report.execution.quality.qualified_results)
            .map(|report| format!("qualified_page:{}", report.url))
            .collect(),
    }
}

fn build_site_analysis(
    batch: &BatchReport,
    presentation: &crate::output::report_model::BatchPresentation,
    normalized_reports: &[NormalizedReport],
) -> serde_json::Value {
    let portfolio = &presentation.portfolio_summary;
    let consistency = batch.consistency.as_ref().map(|value| {
        serde_json::json!({
            "navigation": {
                "pages_with_main_navigation": value.navigation.pages_with_main_nav,
                "pages_with_skip_link": value.navigation.pages_with_skip_link,
                "total_pages": value.navigation.total_pages,
            },
            "headings": {
                "pages_with_single_h1": value.headings.pages_with_single_h1,
                "pages_with_no_h1": value.headings.pages_with_no_h1,
                "pages_with_multiple_h1": value.headings.pages_with_multiple_h1,
                "total_pages": value.headings.total_pages,
            },
            "canonical": {
                "www_count": value.canonical.www_count,
                "non_www_count": value.canonical.non_www_count,
                "missing_count": value.canonical.missing_count,
                "total_pages": value.canonical.total_pages,
            },
            "orphan_pages": {
                "urls": value.orphan_pages.orphan_urls,
                "total_pages": value.orphan_pages.total_pages,
                "scope": "audited_urls_only",
            },
            "schema_graph": {
                "conflicts": value.schema_graph.conflicts,
            },
        })
    });
    let interactive = presentation.interactive_summary.as_ref().map(|value| {
        serde_json::json!({
            "pages_tested": value.total_pages_tested,
            "pages_with_issues": value.pages_with_issues,
            "has_critical": value.has_critical,
            "categories": value.categories.iter().map(|category| serde_json::json!({
                "category": category.category,
                "affected_urls": category.affected_urls,
                "max_severity": category.max_severity,
            })).collect::<Vec<_>>(),
        })
    });
    let mut assessment_groups: std::collections::BTreeMap<
        (String, String, String),
        (usize, std::collections::BTreeSet<String>),
    > = std::collections::BTreeMap::new();
    for report in normalized_reports {
        for assessment in &report.accessibility_assessments {
            let entry = assessment_groups
                .entry((
                    assessment.kind.clone(),
                    assessment.rule_id.clone(),
                    assessment.wcag_criterion.clone(),
                ))
                .or_default();
            entry.0 += 1;
            entry.1.insert(report.url.clone());
        }
    }

    serde_json::json!({
        "module_averages": portfolio.module_averages.iter().map(|(module, score)| serde_json::json!({
            "module": module,
            "score": score,
        })).collect::<Vec<_>>(),
        "active_modules": portfolio.active_modules,
        "consistency": consistency,
        "page_types": portfolio.page_type_distribution.iter().map(|(page_type, count, percentage)| serde_json::json!({
            "page_type": page_type,
            "count": count,
            "percentage": percentage,
        })).collect::<Vec<_>>(),
        "content_quality": {
            "strongest_pages": portfolio.strongest_content_pages.iter().map(|(url, page_type, score)| serde_json::json!({
                "url": url, "page_type": page_type, "score": score,
            })).collect::<Vec<_>>(),
            "weakest_pages": portfolio.weakest_content_pages.iter().map(|(url, page_type, score)| serde_json::json!({
                "url": url, "page_type": page_type, "score": score,
            })).collect::<Vec<_>>(),
        },
        "content_topics": {
            "top_terms": portfolio.top_topics.iter().map(|(term, pages)| serde_json::json!({
                "term": term, "page_count": pages,
            })).collect::<Vec<_>>(),
            "overlap_pairs": portfolio.overlap_pairs.iter().map(|(url_a, url_b, shared_terms)| serde_json::json!({
                "url_a": url_a, "url_b": url_b, "shared_terms": shared_terms,
            })).collect::<Vec<_>>(),
        },
        "duplicates": {
            "exact": portfolio.duplicate_content,
            "near": portfolio.near_duplicates.iter().map(|(url_a, url_b, similarity)| serde_json::json!({
                "url_a": url_a, "url_b": url_b, "similarity_percent": similarity,
            })).collect::<Vec<_>>(),
        },
        "performance": {
            "budget_summary": portfolio.budget_summary.iter().map(|(metric, budget, affected_urls, severity)| serde_json::json!({
                "metric": metric, "budget": budget, "affected_urls": affected_urls, "severity": severity,
            })).collect::<Vec<_>>(),
            "render_blocking_summary": portfolio.render_blocking_summary.iter().map(|(metric, value)| serde_json::json!({
                "metric": metric, "value": value,
            })).collect::<Vec<_>>(),
        },
        "structured_data": {
            "type_distribution": portfolio.schema_distribution.iter().map(|(schema_type, pages)| serde_json::json!({
                "schema_type": schema_type, "page_count": pages,
            })).collect::<Vec<_>>(),
            "pages_without_schema": portfolio.pages_without_schema,
            "entity_conflicts": batch.consistency.as_ref().map(|c| c.schema_graph.conflicts.clone()).unwrap_or_default(),
        },
        "interactive_summary": interactive,
        "accessibility_assessments": assessment_groups.into_iter().map(|((kind, rule_id, wcag_criterion), (occurrences, urls))| serde_json::json!({
            "kind": kind,
            "rule_id": rule_id,
            "wcag_criterion": wcag_criterion,
            "occurrences": occurrences,
            "affected_url_count": urls.len(),
            "affected_urls": urls,
        })).collect::<Vec<_>>(),
    })
}

// ─── Construction ─────────────────────────────────────────────────────────────

impl UnifiedReport {
    /// Build a single-page report with full module detail.
    pub fn single(ctx: &AuditContext<'_>, raw: &AuditReport) -> Self {
        let mut collection_errors: Vec<ReportError> = Vec::new();

        let budget_violations = raw
            .experience
            .budget_violations
            .iter()
            .filter_map(|v| {
                serde_json::to_value(v)
                    .map_err(|e| {
                        collection_errors.push(ReportError {
                            module: "budget_violations",
                            error_type: "serialization_failed",
                            reason: e.to_string(),
                        })
                    })
                    .ok()
            })
            .collect();

        let screenshot_status = match &raw.screenshot_status {
            crate::audit::ScreenshotStatus::NotRequested => None,
            s => serde_json::to_value(s)
                .map_err(|e| {
                    collection_errors.push(ReportError {
                        module: "screenshot_status",
                        error_type: "serialization_failed",
                        reason: e.to_string(),
                    })
                })
                .ok(),
        };

        let detail_ctx = DetailContext {
            budget_violations,
            screenshot_status,
            collection_errors,
        };
        let page = build_page(&ctx.normalized, Some(ctx), Some(detail_ctx));
        Self::wrap_single(ctx, page)
    }

    /// Build a single-page report from cached normalized data only.
    pub fn single_from_normalized(normalized: &NormalizedReport) -> Self {
        let page = build_page(normalized, None, Some(DetailContext::default()));
        Self::wrap_single_from_normalized(normalized, page)
    }

    /// Build a batch report — one summary page per URL, no `detail`.
    pub fn batch(batch_report: &BatchReport) -> Self {
        let normalized_reports: Vec<NormalizedReport> = batch_report
            .reports
            .iter()
            .map(|r| crate::audit::normalize(r).normalized)
            .collect();
        let i18n = crate::i18n::I18n::new("en").expect("canonical JSON locale must load");
        let presentation =
            build_batch_presentation_with_normalized(batch_report, &i18n, &normalized_reports);

        let pages: Vec<PageEntry> = normalized_reports
            .iter()
            .map(|n| {
                let mut page = build_page(n, None, None);
                page.detail = Some(build_batch_detail(n));
                page.content_profile = presentation
                    .url_details
                    .iter()
                    .find(|detail| detail.url == n.url)
                    .map(|detail| BatchContentProfile {
                        page_type: detail.page_type.clone(),
                        attributes: detail.page_attributes.clone(),
                        semantic_score: detail.page_semantic_score,
                        biggest_lever: detail.biggest_lever.clone(),
                        topic_terms: detail.topic_terms.clone(),
                        top_issues: detail.top_issues.clone(),
                    });
                page
            })
            .collect();

        let severity_counts = aggregate_severity(&pages);
        let occurrence_counts = aggregate_occurrences(&pages);
        let accessibility_score = batch_report.summary.average_score.round() as u32;
        let overall_score_batch = presentation.portfolio_summary.average_overall_score;
        let summary = UnifiedSummary {
            url_count: batch_report.summary.total_urls,
            accessibility_score,
            overall_score: overall_score_batch,
            score: overall_score_batch,
            // Grade/certificate derive from the same field as `score`/`overall_score`
            // (overall_score_batch), consistent with single-report semantics where
            // grade is derived from overall_score, not accessibility_score alone.
            grade: AccessibilityScorer::calculate_grade(overall_score_batch as f32).to_string(),
            certificate: AccessibilityScorer::calculate_certificate(overall_score_batch as f32)
                .to_string(),
            risk_level: batch_report.summary.risk,
            violation_count: pages.iter().map(|p| p.violation_count).sum(),
            finding_count: pages.iter().map(|p| p.finding_count).sum(),
            finding_occurrence_count: pages.iter().map(|p| p.finding_occurrence_count).sum(),
            severity_counts,
            severity_counts_scope: "wcag_only".to_string(),
            occurrence_counts,
            occurrence_counts_scope: "wcag_only".to_string(),
            passed_url_count: batch_report.summary.passed,
            failed_url_count: batch_report.summary.failed,
            violated_rule_count: batch_report.summary.violated_rule_count,
            top_recurring_rules: batch_report.summary.top_recurring_rules.clone(),
            template_clusters: batch_report.summary.template_clusters.clone(),
            en301549_rollup: build_en301549_batch_rollup(&normalized_reports),
            performance_score: avg_module_score(&pages, "Performance"),
            seo_score: avg_module_score(&pages, "SEO"),
            security_score: avg_module_score(&pages, "Security"),
            mobile_score: avg_module_score(&pages, "Mobile"),
            ux_score: avg_module_score(&pages, "UX"),
            journey_score: avg_module_score(&pages, "Journey"),
            performance_throttled_avg_score: None,
            lh_mobile_score: None,
            wcag_coverage: normalized_reports
                .first()
                .map(build_wcag_coverage_summary)
                .unwrap_or_else(|| build_wcag_coverage_for_level("mixed")),
            accessibility_score_breakdown: build_accessibility_score_breakdown(&normalized_reports),
            management_risks: build_management_risks(&normalized_reports),
            top_actions: build_decision_actions(&normalized_reports),
            duplicate_content: presentation.portfolio_summary.duplicate_content.clone(),
            canonical_issues: presentation.portfolio_summary.canonical_issues.clone(),
            hreflang_issues: presentation.portfolio_summary.hreflang_issues.clone(),
            sitemap_http_issues: presentation.portfolio_summary.sitemap_http_issues.clone(),
            orphan_sitemap_urls: presentation.portfolio_summary.orphan_sitemap_urls.clone(),
            linked_not_in_sitemap: presentation.portfolio_summary.linked_not_in_sitemap.clone(),
            commerce: crate::commerce::aggregate_site_commerce(
                batch_report
                    .reports
                    .iter()
                    .filter_map(|r| r.commerce.as_ref()),
            ),
        };

        let mut collection_errors: Vec<ReportError> = Vec::new();

        let crawl_diagnostics = batch_report.crawl_diagnostics.as_ref().and_then(|c| {
            serde_json::to_value(c)
                .map_err(|e| {
                    collection_errors.push(ReportError {
                        module: "crawl_diagnostics",
                        error_type: "serialization_failed",
                        reason: e.to_string(),
                    })
                })
                .ok()
        });

        let sitemap_diagnostics = batch_report.sitemap_diagnostics.as_ref().and_then(|s| {
            serde_json::to_value(s)
                .map_err(|e| {
                    collection_errors.push(ReportError {
                        module: "sitemap_diagnostics",
                        error_type: "serialization_failed",
                        reason: e.to_string(),
                    })
                })
                .ok()
        });

        let errors: Vec<serde_json::Value> = batch_report
            .errors
            .iter()
            .filter_map(|e| {
                serde_json::to_value(e)
                    .map_err(|err| {
                        collection_errors.push(ReportError {
                            module: "errors",
                            error_type: "serialization_failed",
                            reason: err.to_string(),
                        })
                    })
                    .ok()
            })
            .collect();
        let site_analysis = build_site_analysis(batch_report, &presentation, &normalized_reports);

        UnifiedReport {
            schema_version: SCHEMA_VERSION,
            report_type: "batch",
            tool_version: env!("CARGO_PKG_VERSION"),
            metadata: ReportMetadata {
                tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
                timestamp: batch_report_timestamp(batch_report),
                wcag_level: normalized_reports
                    .first()
                    .map(|r| r.wcag_level.to_string())
                    .unwrap_or_else(|| "mixed".to_string()),
                execution_time_ms: batch_report.total_duration_ms,
            },
            audit_scope: normalized_reports
                .first()
                .map(|report| report.execution.scope.clone()),
            execution_environment: normalized_reports
                .first()
                .map(|report| report.execution.environment.clone()),
            audit_quality: aggregate_audit_quality(&normalized_reports),
            metric_context: metric_context("batch"),
            summary,
            sample: batch_report.sample.clone(),
            pages,
            url_matrix: presentation.url_matrix,
            internal_comparison: Some(build_internal_comparison(&normalized_reports)),
            crawl_diagnostics,
            sitemap_diagnostics,
            site_analysis: Some(site_analysis),
            artifacts: Vec::new(),
            errors,
            collection_errors,
            verdict: Verdict::Pass,
            verdict_reasons: Vec::new(),
        }
    }

    /// Serialize to a JSON string.
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        let output = if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        };
        output.map_err(|e| crate::error::AuditError::OutputError {
            reason: format!("JSON serialization failed: {}", e),
        })
    }

    /// Override the default (Pass) verdict with a computed result.
    pub fn with_verdict(mut self, vr: &VerdictResult) -> Self {
        self.verdict = vr.verdict;
        self.verdict_reasons = vr.reasons.clone();
        self
    }

    fn wrap_single(ctx: &AuditContext<'_>, page: PageEntry) -> Self {
        let no_legal_flags = !page.findings.iter().any(|f| {
            f.wcag_level == "A"
                && matches!(
                    f.severity,
                    crate::taxonomy::Severity::Critical | crate::taxonomy::Severity::High,
                )
        });
        // Pass criterion: accessibility score ≥ 80, no critical findings, no legal
        // exposure. Must match the batch criterion in audit/report.rs (#253).
        let passed = usize::from(
            page.accessibility_score >= 80 && page.severity_counts.critical == 0 && no_legal_flags,
        );
        let violated_rule_count = page.violated_rule_count;
        let (top_recurring_rules, _) =
            compute_recurring_rules(std::slice::from_ref(&ctx.normalized));
        let summary = UnifiedSummary {
            url_count: 1,
            accessibility_score: page.accessibility_score,
            overall_score: page.overall_score,
            score: page.overall_score,
            grade: page.grade.clone(),
            certificate: page.certificate.clone(),
            risk_level: page.risk.level,
            violation_count: page.violation_count,
            finding_count: page.finding_count,
            finding_occurrence_count: page.finding_occurrence_count,
            severity_counts: page.severity_counts.clone(),
            severity_counts_scope: "wcag_only".to_string(),
            occurrence_counts: page.occurrence_counts.clone(),
            occurrence_counts_scope: "wcag_only".to_string(),
            passed_url_count: passed,
            failed_url_count: 1 - passed,
            violated_rule_count,
            top_recurring_rules,
            template_clusters: Vec::new(),
            en301549_rollup: Vec::new(),
            performance_score: normalized_module_score(&ctx.normalized, "Performance"),
            seo_score: normalized_module_score(&ctx.normalized, "SEO"),
            security_score: normalized_module_score(&ctx.normalized, "Security"),
            mobile_score: normalized_module_score(&ctx.normalized, "Mobile"),
            ux_score: normalized_module_score(&ctx.normalized, "UX"),
            journey_score: normalized_module_score(&ctx.normalized, "Journey"),
            performance_throttled_avg_score: {
                let scores: Vec<u32> = ctx
                    .raw_throttled_performance
                    .iter()
                    .map(|t| t.score)
                    .collect();
                if scores.is_empty() {
                    None
                } else {
                    Some(scores.iter().sum::<u32>() / scores.len() as u32)
                }
            },
            lh_mobile_score: ctx
                .raw_throttled_performance
                .iter()
                .find(|t| t.profile == crate::browser::ThrottleProfile::LhMobile)
                .map(|t| t.score),
            wcag_coverage: build_wcag_coverage_summary(&ctx.normalized),
            accessibility_score_breakdown: build_accessibility_score_breakdown(
                std::slice::from_ref(&ctx.normalized),
            ),
            management_risks: build_management_risks(std::slice::from_ref(&ctx.normalized)),
            top_actions: build_decision_actions(std::slice::from_ref(&ctx.normalized)),
            duplicate_content: Vec::new(),
            canonical_issues: Vec::new(),
            hreflang_issues: Vec::new(),
            sitemap_http_issues: Vec::new(),
            orphan_sitemap_urls: Vec::new(),
            linked_not_in_sitemap: Vec::new(),
            commerce: None,
        };

        UnifiedReport {
            schema_version: SCHEMA_VERSION,
            report_type: "single",
            tool_version: env!("CARGO_PKG_VERSION"),
            metadata: ReportMetadata {
                tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
                timestamp: ctx.normalized.timestamp,
                wcag_level: ctx.normalized.wcag_level.to_string(),
                execution_time_ms: ctx.normalized.duration_ms,
            },
            audit_scope: Some(ctx.normalized.execution.scope.clone()),
            execution_environment: Some(ctx.normalized.execution.environment.clone()),
            audit_quality: ctx.normalized.execution.quality.clone(),
            metric_context: metric_context("single"),
            summary,
            sample: None,
            pages: vec![page],
            url_matrix: Vec::new(),
            internal_comparison: None,
            crawl_diagnostics: None,
            sitemap_diagnostics: None,
            site_analysis: None,
            artifacts: artifacts_for(&ctx.normalized),
            errors: Vec::new(),
            collection_errors: Vec::new(),
            verdict: Verdict::Pass,
            verdict_reasons: Vec::new(),
        }
    }

    /// Wrap a single page (cached/from-normalized path — no raw module data available).
    fn wrap_single_from_normalized(normalized: &NormalizedReport, page: PageEntry) -> Self {
        let no_legal_flags = !page.findings.iter().any(|f| {
            f.wcag_level == "A"
                && matches!(
                    f.severity,
                    crate::taxonomy::Severity::Critical | crate::taxonomy::Severity::High,
                )
        });
        // Pass criterion: accessibility score ≥ 80, no critical findings, no legal
        // exposure. Must match the batch criterion in audit/report.rs (#253).
        let passed = usize::from(
            page.accessibility_score >= 80 && page.severity_counts.critical == 0 && no_legal_flags,
        );
        let violated_rule_count = page.violated_rule_count;
        let (top_recurring_rules, _) = compute_recurring_rules(std::slice::from_ref(normalized));
        let summary = UnifiedSummary {
            url_count: 1,
            accessibility_score: page.accessibility_score,
            overall_score: page.overall_score,
            score: page.overall_score,
            grade: page.grade.clone(),
            certificate: page.certificate.clone(),
            risk_level: page.risk.level,
            violation_count: page.violation_count,
            finding_count: page.finding_count,
            finding_occurrence_count: page.finding_occurrence_count,
            severity_counts: page.severity_counts.clone(),
            severity_counts_scope: "wcag_only".to_string(),
            occurrence_counts: page.occurrence_counts.clone(),
            occurrence_counts_scope: "wcag_only".to_string(),
            passed_url_count: passed,
            failed_url_count: 1 - passed,
            violated_rule_count,
            top_recurring_rules,
            template_clusters: Vec::new(),
            en301549_rollup: Vec::new(),
            performance_score: normalized_module_score(normalized, "Performance"),
            seo_score: normalized_module_score(normalized, "SEO"),
            security_score: normalized_module_score(normalized, "Security"),
            mobile_score: normalized_module_score(normalized, "Mobile"),
            ux_score: normalized_module_score(normalized, "UX"),
            journey_score: normalized_module_score(normalized, "Journey"),
            performance_throttled_avg_score: None,
            lh_mobile_score: None,
            wcag_coverage: build_wcag_coverage_summary(normalized),
            accessibility_score_breakdown: build_accessibility_score_breakdown(
                std::slice::from_ref(normalized),
            ),
            management_risks: build_management_risks(std::slice::from_ref(normalized)),
            top_actions: build_decision_actions(std::slice::from_ref(normalized)),
            duplicate_content: Vec::new(),
            canonical_issues: Vec::new(),
            hreflang_issues: Vec::new(),
            sitemap_http_issues: Vec::new(),
            orphan_sitemap_urls: Vec::new(),
            linked_not_in_sitemap: Vec::new(),
            commerce: None,
        };

        UnifiedReport {
            schema_version: SCHEMA_VERSION,
            report_type: "single",
            tool_version: env!("CARGO_PKG_VERSION"),
            metadata: ReportMetadata {
                tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
                timestamp: normalized.timestamp,
                wcag_level: normalized.wcag_level.to_string(),
                execution_time_ms: normalized.duration_ms,
            },
            audit_scope: Some(normalized.execution.scope.clone()),
            execution_environment: Some(normalized.execution.environment.clone()),
            audit_quality: normalized.execution.quality.clone(),
            metric_context: metric_context("single"),
            summary,
            sample: None,
            pages: vec![page],
            url_matrix: Vec::new(),
            internal_comparison: None,
            crawl_diagnostics: None,
            sitemap_diagnostics: None,
            site_analysis: None,
            artifacts: artifacts_for(normalized),
            errors: Vec::new(),
            collection_errors: Vec::new(),
            verdict: Verdict::Pass,
            verdict_reasons: Vec::new(),
        }
    }
}
