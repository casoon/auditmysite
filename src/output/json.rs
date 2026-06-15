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

// ─── Public entry points ──────────────────────────────────────────────────────

/// Generate single-report JSON from a live audit context.
pub fn format_json_normalized(
    ctx: &AuditContext,
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

/// Uniform summary — identical field names for single and batch.
#[derive(Debug, Serialize)]
pub struct UnifiedSummary {
    pub url_count: usize,
    pub accessibility_score: u32,
    pub overall_score: u32,
    pub grade: String,
    pub certificate: String,
    pub risk_level: crate::audit::normalized::RiskLevel,
    pub violation_count: usize,
    /// Anzahl unterschiedlicher Findings (eine Zeile pro Regel/Severity).
    pub severity_counts: crate::audit::normalized::SeverityCounts,
    pub severity_counts_scope: String,
    /// Element-Occurrences je Severity (Summe über alle Findings).
    pub occurrence_counts: crate::audit::normalized::SeverityCounts,
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
}

#[derive(Debug, Serialize)]
pub struct WcagCoverageSummary {
    pub level: String,
    pub automated_criteria: usize,
    pub manual_review_criteria: usize,
    pub total_wcag_aa_criteria: usize,
    pub note: String,
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
    pub violation_count: usize,
    /// Number of distinct WCAG rules that fired — `findings[].length` for wcag-category entries.
    pub violated_rule_count: usize,
    pub severity_counts: crate::audit::normalized::SeverityCounts,
    pub severity_counts_scope: String,
    /// Element-Occurrences je Severity (Summe `occurrence_count` über alle WCAG-Findings).
    pub occurrence_counts: crate::audit::normalized::SeverityCounts,
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
    pub detail: Option<PageDetail>,
}

/// Per-page module detail blob — single reports only.
#[derive(Debug, Serialize)]
pub struct PageDetail {
    /// Fix guidance entries — always present (may be an empty array when there
    /// are no findings). See issue #253.
    pub fix_guidance: Vec<FixGuidance>,
    pub modules: ModuleBlob,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub confidence_summary: Vec<OutputConfidenceSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<OutputCapabilitySignal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport_scores: Option<crate::audit::ViewportScores>,
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

// ─── Construction ─────────────────────────────────────────────────────────────

impl UnifiedReport {
    /// Build a single-page report with full module detail.
    pub fn single(ctx: &AuditContext, raw: &AuditReport) -> Self {
        let mut collection_errors: Vec<ReportError> = Vec::new();

        let budget_violations = raw
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
        let page = build_page(ctx, Some(ctx), Some(detail_ctx));
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
        let i18n = crate::i18n::I18n::new("de").expect("default locale must always load");
        let presentation =
            build_batch_presentation_with_normalized(batch_report, &i18n, &normalized_reports);

        let pages: Vec<PageEntry> = normalized_reports
            .iter()
            .map(|n| {
                let mut page = build_page(n, None, None);
                page.detail = Some(build_batch_detail(n));
                page
            })
            .collect();

        let severity_counts = aggregate_severity(&pages);
        let occurrence_counts = aggregate_occurrences(&pages);
        let accessibility_score = batch_report.summary.average_score.round() as u32;
        let summary = UnifiedSummary {
            url_count: batch_report.summary.total_urls,
            accessibility_score,
            overall_score: presentation.portfolio_summary.average_overall_score,
            grade: AccessibilityScorer::calculate_grade(accessibility_score as f32).to_string(),
            certificate: AccessibilityScorer::calculate_certificate(accessibility_score as f32)
                .to_string(),
            risk_level: batch_report.summary.risk,
            violation_count: pages.iter().map(|p| p.violation_count).sum(),
            severity_counts,
            severity_counts_scope: "wcag_only".to_string(),
            occurrence_counts,
            passed_url_count: batch_report.summary.passed,
            failed_url_count: batch_report.summary.failed,
            violated_rule_count: batch_report.summary.violated_rule_count,
            top_recurring_rules: batch_report.summary.top_recurring_rules.clone(),
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
            summary,
            sample: batch_report.sample.clone(),
            pages,
            url_matrix: presentation.url_matrix,
            internal_comparison: Some(build_internal_comparison(&normalized_reports)),
            crawl_diagnostics,
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

    fn wrap_single(ctx: &AuditContext, page: PageEntry) -> Self {
        let no_legal_flags = !page.findings.iter().any(|f| {
            f.wcag_level == "A"
                && matches!(
                    f.severity,
                    crate::taxonomy::Severity::Critical | crate::taxonomy::Severity::High,
                )
        });
        let passed = usize::from(
            page.overall_score >= 80 && page.severity_counts.critical == 0 && no_legal_flags,
        );
        let violated_rule_count = page.violated_rule_count;
        let (top_recurring_rules, _) =
            compute_recurring_rules(std::slice::from_ref(&ctx.normalized));
        let summary = UnifiedSummary {
            url_count: 1,
            accessibility_score: page.accessibility_score,
            overall_score: page.overall_score,
            grade: page.grade.clone(),
            certificate: page.certificate.clone(),
            risk_level: page.risk.level,
            violation_count: page.violation_count,
            severity_counts: page.severity_counts.clone(),
            severity_counts_scope: "wcag_only".to_string(),
            occurrence_counts: page.occurrence_counts.clone(),
            passed_url_count: passed,
            failed_url_count: 1 - passed,
            violated_rule_count,
            top_recurring_rules,
            performance_score: normalized_module_score(ctx, "Performance"),
            seo_score: normalized_module_score(ctx, "SEO"),
            security_score: normalized_module_score(ctx, "Security"),
            mobile_score: normalized_module_score(ctx, "Mobile"),
            ux_score: normalized_module_score(ctx, "UX"),
            journey_score: normalized_module_score(ctx, "Journey"),
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
            wcag_coverage: build_wcag_coverage_summary(ctx),
            accessibility_score_breakdown: build_accessibility_score_breakdown(
                std::slice::from_ref(&ctx.normalized),
            ),
            management_risks: build_management_risks(std::slice::from_ref(&ctx.normalized)),
            top_actions: build_decision_actions(std::slice::from_ref(&ctx.normalized)),
        };

        UnifiedReport {
            schema_version: SCHEMA_VERSION,
            report_type: "single",
            tool_version: env!("CARGO_PKG_VERSION"),
            metadata: ReportMetadata {
                tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
                timestamp: ctx.timestamp,
                wcag_level: ctx.wcag_level.to_string(),
                execution_time_ms: ctx.duration_ms,
            },
            summary,
            sample: None,
            pages: vec![page],
            url_matrix: Vec::new(),
            internal_comparison: None,
            crawl_diagnostics: None,
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
        let passed = usize::from(
            page.overall_score >= 80 && page.severity_counts.critical == 0 && no_legal_flags,
        );
        let violated_rule_count = page.violated_rule_count;
        let (top_recurring_rules, _) = compute_recurring_rules(std::slice::from_ref(normalized));
        let summary = UnifiedSummary {
            url_count: 1,
            accessibility_score: page.accessibility_score,
            overall_score: page.overall_score,
            grade: page.grade.clone(),
            certificate: page.certificate.clone(),
            risk_level: page.risk.level,
            violation_count: page.violation_count,
            severity_counts: page.severity_counts.clone(),
            severity_counts_scope: "wcag_only".to_string(),
            occurrence_counts: page.occurrence_counts.clone(),
            passed_url_count: passed,
            failed_url_count: 1 - passed,
            violated_rule_count,
            top_recurring_rules,
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
            summary,
            sample: None,
            pages: vec![page],
            url_matrix: Vec::new(),
            internal_comparison: None,
            crawl_diagnostics: None,
            errors: Vec::new(),
            collection_errors: Vec::new(),
            verdict: Verdict::Pass,
            verdict_reasons: Vec::new(),
        }
    }
}

/// Report-level inputs for `detail` that the `NormalizedReport` does not carry.
#[derive(Default)]
struct DetailContext {
    budget_violations: Vec<serde_json::Value>,
    screenshot_status: Option<serde_json::Value>,
    collection_errors: Vec<ReportError>,
}

/// Build a [`PageEntry`]. `audit_ctx` `Some` builds the full single-report `detail`
/// using raw module data; `None` builds a minimal cached detail.
/// `detail_ctx` `None` leaves `detail` unset (batch callers attach a compact detail).
fn build_page(
    normalized: &NormalizedReport,
    audit_ctx: Option<&AuditContext>,
    detail_ctx: Option<DetailContext>,
) -> PageEntry {
    let detail = detail_ctx.map(|d| {
        if let Some(ctx) = audit_ctx {
            build_detail(ctx, d)
        } else {
            build_detail_cached(normalized, d)
        }
    });

    PageEntry {
        url: normalized.url.clone(),
        accessibility_score: normalized.score,
        overall_score: normalized.overall_score,
        grade: normalized.grade.clone(),
        certificate: normalized.certificate.clone(),
        // Counts cover all finding categories (WCAG + SEO), matching the
        // contents of `findings` and `detail.fix_guidance` (issues #254, #255).
        // `severity_counts` stays WCAG-only (legal/risk semantics, see spec).
        violation_count: normalized.findings.iter().map(|f| f.occurrence_count).sum(),
        violated_rule_count: distinct_rule_count(&normalized.findings),
        severity_counts: normalized.severity_counts.clone(),
        severity_counts_scope: "wcag_only".to_string(),
        occurrence_counts: all_category_occurrence_counts(&normalized.findings),
        nodes_analyzed: normalized.nodes_analyzed,
        duration_ms: normalized.duration_ms,
        module_scores: normalized.module_scores.clone(),
        performance_score: normalized_module_score(normalized, "Performance"),
        seo_score: normalized_module_score(normalized, "SEO"),
        security_score: normalized_module_score(normalized, "Security"),
        mobile_score: normalized_module_score(normalized, "Mobile"),
        ux_score: normalized_module_score(normalized, "UX"),
        journey_score: normalized_module_score(normalized, "Journey"),
        score_calculation_method: normalized.score_calculation_method.clone(),
        score_breakdown: normalized.score_breakdown.clone(),
        risk: normalized.risk.clone(),
        principle_coverage: normalized.principle_coverage.clone(),
        findings: normalized.findings.clone(),
        audit_flags: normalized.audit_flags.clone(),
        interactive_findings: normalized.interactive_findings.clone(),
        accessibility_journey: normalized.accessibility_journey.clone(),
        screen_reader: normalized.screen_reader.clone(),
        detail,
    }
}

/// Compact per-page detail for batch reports: actionable `fix_guidance` only,
/// without the heavy module blob. Keeps batch reports from devolving into a
/// stack of single-page reports (see CLAUDE.md "Report Intent") while honouring
/// the contract that `detail.fix_guidance` is always present (issue #256).
fn build_batch_detail(normalized: &NormalizedReport) -> PageDetail {
    PageDetail {
        fix_guidance: build_fix_guidance(normalized),
        modules: ModuleBlob::default(),
        confidence_summary: Vec::new(),
        capabilities: Vec::new(),
        viewport_scores: None,
        budget_violations: Vec::new(),
        throttled_performance: Vec::new(),
        screenshot_status: None,
        collection_errors: Vec::new(),
    }
}

/// Minimal detail for the cached/from-normalized path — no raw module data available.
fn build_detail_cached(normalized: &NormalizedReport, detail_ctx: DetailContext) -> PageDetail {
    PageDetail {
        fix_guidance: build_fix_guidance(normalized),
        modules: ModuleBlob::default(),
        confidence_summary: Vec::new(),
        capabilities: Vec::new(),
        viewport_scores: normalized.viewport_scores.clone(),
        budget_violations: detail_ctx.budget_violations,
        throttled_performance: Vec::new(),
        screenshot_status: detail_ctx.screenshot_status,
        collection_errors: detail_ctx.collection_errors,
    }
}

fn build_detail(ctx: &AuditContext, detail_ctx: DetailContext) -> PageDetail {
    let vm = build_view_model(ctx, &ReportConfig::default());
    let normalized: &NormalizedReport = ctx;
    let mut errors = detail_ctx.collection_errors;

    let wcag_findings: Vec<_> = normalized
        .findings
        .iter()
        .filter(|f| f.category == "wcag")
        .collect();
    let seo_findings: Vec<_> = normalized
        .findings
        .iter()
        .filter(|f| f.category == "seo")
        .collect();

    let tech_stack = ctx.raw_tech_stack.as_ref().and_then(|m| {
        serde_json::to_value(m)
            .map_err(|e| {
                errors.push(ReportError {
                    module: "tech_stack",
                    error_type: "serialization_failed",
                    reason: e.to_string(),
                })
            })
            .ok()
    });

    let search_experience = vm.module_details.search_experience.as_ref().map(|sx| {
        serde_json::json!({
            "score": sx.score,
            "label": sx.label.clone(),
            "interpretation": sx.interpretation.clone(),
            "components": sx.components.iter().map(|component| {
                serde_json::json!({
                    "label": component.label.clone(),
                    "score": component.score,
                    "weight_pct": component.weight_pct,
                    "explanation": component.explanation.clone(),
                })
            }).collect::<Vec<_>>(),
            "warnings": sx.warnings.clone(),
            "measurement_type": "composite",
        })
    });

    let dual_viewport = ctx.raw_dual_viewport.as_ref().map(|dual| {
        serde_json::json!({
            "desktop": viewport_detail_summary(&dual.desktop),
            "mobile": viewport_detail_summary(&dual.mobile),
        })
    });

    let patterns = ctx.raw_patterns.as_ref().map(|m| {
        let total = m.recognized.len() + m.violations.len();
        let pattern_score: u32 = if total > 0 {
            (m.recognized.len() as u32 * 100) / total as u32
        } else {
            75
        };
        match serde_json::to_value(m) {
            Ok(mut v) => {
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("score".to_string(), serde_json::json!(pattern_score));
                    obj.insert("grade".to_string(), serde_json::json!(
                        crate::audit::AccessibilityScorer::calculate_grade(pattern_score as f32)
                    ));
                }
                v
            }
            Err(e) => {
                errors.push(ReportError {
                    module: "patterns",
                    error_type: "serialization_failed",
                    reason: e.to_string(),
                });
                serde_json::json!({
                    "score": pattern_score,
                    "grade": crate::audit::AccessibilityScorer::calculate_grade(pattern_score as f32)
                })
            }
        }
    });

    let throttled_performance: Vec<serde_json::Value> = {
        let mut acc = Vec::new();
        for v in ctx.raw_throttled_performance.iter() {
            match serde_json::to_value(v) {
                Ok(json) => acc.push(json),
                Err(e) => errors.push(ReportError {
                    module: "throttled_performance",
                    error_type: "serialization_failed",
                    reason: e.to_string(),
                }),
            }
        }
        acc
    };

    let seo = ctx.raw_seo.as_ref().map(|m| {
        let mut v = with_normalized_score(m.to_json(), normalized, "SEO");
        let findings_value = match serde_json::to_value(&seo_findings) {
            Ok(json) => json,
            Err(e) => {
                errors.push(ReportError {
                    module: "seo",
                    error_type: "findings_serialization_failed",
                    reason: e.to_string(),
                });
                serde_json::json!([])
            }
        };
        if let Some(obj) = v.as_object_mut() {
            obj.insert("findings".to_string(), findings_value);
        }
        v
    });

    let modules = ModuleBlob {
        accessibility: Some(serde_json::json!({
            "score": normalized.score,
            "grade": normalized.grade,
            "severity_counts": normalized.severity_counts,
            "principle_coverage": normalized.principle_coverage,
            "findings": wcag_findings,
        })),
        dual_viewport,
        search_experience,
        performance: ctx.raw_performance.as_ref().map(|m| {
            inject_unused_js_bytes(
                with_normalized_score(m.to_json(), normalized, "Performance"),
                m,
            )
        }),
        seo,
        security: ctx
            .raw_security
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Security")),
        mobile: ctx
            .raw_mobile
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Mobile")),
        ux: ctx
            .raw_ux
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "UX")),
        journey: ctx
            .raw_journey
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Journey")),
        dark_mode: ctx
            .raw_dark_mode
            .as_ref()
            .map(|m| inject_grade(m.to_json(), m.score)),
        source_quality: ctx
            .raw_source_quality
            .as_ref()
            .map(|m| with_measurement_type(m.to_json(), "heuristic")),
        ai_visibility: ctx
            .raw_ai_visibility
            .as_ref()
            .map(|m| with_measurement_type(m.to_json(), "heuristic")),
        content_visibility: ctx.raw_content_visibility.as_ref().and_then(|m| {
            // Only emit when SEO data was available — all signal sections are empty
            // without --full, which is misleading.
            if m.signal_count == 0 {
                return None;
            }
            let cv_score = (m.signal_count.saturating_sub(m.problem_count) as u32 * 100)
                / m.signal_count as u32;
            let mut v = with_measurement_type(m.to_json(), "heuristic");
            if let Some(obj) = v.as_object_mut() {
                obj.insert("score".to_string(), serde_json::json!(cv_score));
                obj.insert(
                    "grade".to_string(),
                    serde_json::json!(crate::audit::AccessibilityScorer::calculate_grade(
                        cv_score as f32
                    )),
                );
            }
            Some(v)
        }),
        tech_stack,
        patterns,
        best_practices: ctx
            .raw_best_practices
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Best Practices")),
    };

    PageDetail {
        fix_guidance: build_fix_guidance(normalized),
        modules,
        confidence_summary: vm
            .methodology
            .confidence_summary
            .iter()
            .map(|(signal, assessment)| OutputConfidenceSignal {
                signal: signal.clone(),
                assessment: assessment.clone(),
            })
            .collect(),
        capabilities: vm
            .methodology
            .capabilities
            .iter()
            .map(|cap| OutputCapabilitySignal {
                signal: cap.signal.clone(),
                source: cap.source.clone(),
                confidence: cap.confidence.clone(),
                surfaces: cap.surfaces.clone(),
                note: cap.note.clone(),
            })
            .collect(),
        viewport_scores: normalized.viewport_scores.clone(),
        budget_violations: detail_ctx.budget_violations,
        throttled_performance,
        screenshot_status: detail_ctx.screenshot_status,
        collection_errors: errors,
    }
}

fn viewport_detail_summary(data: &crate::audit::ViewportAuditData) -> serde_json::Value {
    serde_json::json!({
        "accessibility_score": data.accessibility_score.round().max(0.0) as u32,
        "wcag": {
            "violations": data.wcag_results.violations.len(),
            "warnings": data.wcag_results.warnings.len(),
            "positives": data.wcag_results.positives.len(),
            "not_testables": data.wcag_results.not_testables.len(),
            "nodes_checked": data.wcag_results.nodes_checked,
        },
        "modules": {
            "performance": data.performance.as_ref().map(|p| p.score.overall),
            "seo": data.seo.as_ref().map(|s| s.score),
            "mobile": data.mobile.as_ref().map(|m| m.score),
            "ux": data.ux.as_ref().map(|u| u.score),
            "journey": data.journey.as_ref().map(|j| j.score),
        },
        "has_screenshot": data.screenshot.is_some(),
    })
}

/// Build fix guidance entries from normalized findings + explanation database.
fn build_fix_guidance(normalized: &NormalizedReport) -> Vec<FixGuidance> {
    normalized
        .findings
        .iter()
        .map(|finding| {
            let expl = get_explanation(&finding.rule_id);

            let mut seen = std::collections::HashSet::new();
            // Accept any non-empty selector that isn't a raw numeric node ID.
            // The old CSS-character filter was too strict and dropped plain tag
            // selectors (e.g. "a") from rules like color_link_indicator.
            let affected_selectors: Vec<String> = finding
                .occurrences
                .iter()
                .filter_map(|o| o.selector.clone())
                .filter(|s| {
                    !s.is_empty()
                        && !s.chars().all(|c| c.is_ascii_digit())
                        && seen.insert(s.clone())
                })
                .take(10)
                .collect();

            let code_example = expl.and_then(|e| match (e.example_bad, e.example_good) {
                (Some(bad), Some(good)) => Some(CodeExample {
                    bad: bad.to_string(),
                    good: good.to_string(),
                }),
                _ => None,
            });

            FixGuidance {
                rule_id: finding.rule_id.clone(),
                title: expl
                    .map(|e| e.customer_title.to_string())
                    .unwrap_or_else(|| finding.title.clone()),
                wcag_criterion: finding.wcag_criterion.clone(),
                severity: format!("{:?}", finding.severity).to_lowercase(),
                risk: format!("{:?}", finding.severity).to_lowercase(),
                remediation_priority: finding.remediation_priority.clone(),
                complexity: finding.complexity.clone(),
                complexity_reason: finding.complexity_reason.clone(),
                confidence: finding.confidence.clone(),
                false_positive_risk: finding.false_positive_risk.clone(),
                verification: finding.verification.clone(),
                expected_impact: finding.expected_impact.clone(),
                bfsg_relevance: finding.bfsg_relevance.clone(),
                occurrence_count: finding.occurrence_count,
                problem: expl
                    .map(|e| e.customer_description.to_string())
                    .unwrap_or_else(|| finding.description.clone()),
                user_impact: expl.map(|e| e.user_impact.to_string()).or_else(|| {
                    if finding.user_impact.is_empty() {
                        None
                    } else {
                        Some(finding.user_impact.clone())
                    }
                }),
                typical_cause: expl
                    .map(|e| e.typical_cause.to_string())
                    .filter(|s| !s.is_empty()),
                recommendation: expl
                    .map(|e| e.recommendation.to_string())
                    .filter(|s| !s.is_empty()),
                technical_note: expl
                    .map(|e| e.technical_note.to_string())
                    .filter(|s| !s.is_empty()),
                code_example,
                affected_selectors,
            }
        })
        .collect()
}

fn build_wcag_coverage_summary(normalized: &NormalizedReport) -> WcagCoverageSummary {
    build_wcag_coverage_for_level(&normalized.wcag_level.to_string())
}

fn build_wcag_coverage_for_level(level: &str) -> WcagCoverageSummary {
    let (automated, total) = crate::wcag::coverage::coverage_stats();
    WcagCoverageSummary {
        level: format!("WCAG 2.1 {level}"),
        automated_criteria: automated,
        manual_review_criteria: crate::wcag::coverage::MANUAL_REVIEW_CRITERIA.len(),
        total_wcag_aa_criteria: total,
        note: "Automated score covers detectable criteria only; context-dependent WCAG criteria require manual review.".to_string(),
    }
}

fn build_accessibility_score_breakdown(
    reports: &[NormalizedReport],
) -> Vec<AccessibilityScoreComponent> {
    const AREAS: [(&str, u32); 8] = [
        ("Semantics", 15),
        ("Forms", 15),
        ("Keyboard", 15),
        ("Focus management", 10),
        ("Images / alternative text", 15),
        ("ARIA", 15),
        ("Heading structure", 8),
        ("Landmarks / page structure", 7),
    ];

    AREAS
        .iter()
        .map(|(area, weight_pct)| {
            let mut penalty = 0u32;
            let mut driver: Option<(&str, usize)> = None;

            for finding in reports.iter().flat_map(|report| report.findings.iter()) {
                if score_area_for_finding(finding) != *area {
                    continue;
                }
                let severity_weight = match finding.severity {
                    crate::taxonomy::Severity::Critical => 20,
                    crate::taxonomy::Severity::High => 14,
                    crate::taxonomy::Severity::Medium => 8,
                    crate::taxonomy::Severity::Low => 4,
                };
                penalty = penalty.saturating_add(severity_weight * finding.occurrence_count as u32);
                if driver
                    .map(|(_, count)| finding.occurrence_count > count)
                    .unwrap_or(true)
                {
                    driver = Some((&finding.title, finding.occurrence_count));
                }
            }

            let estimated_lost_points = penalty.min(100);
            AccessibilityScoreComponent {
                area: (*area).to_string(),
                score: 100u32.saturating_sub(estimated_lost_points),
                weight_pct: *weight_pct,
                estimated_lost_points,
                main_driver: driver
                    .map(|(title, count)| format!("{title} ({count} occurrences)"))
                    .unwrap_or_else(|| "No detected driver".to_string()),
            }
        })
        .collect()
}

fn score_area_for_finding(finding: &crate::audit::normalized::NormalizedFinding) -> &'static str {
    let key = format!(
        "{} {} {} {}",
        finding.rule_id.to_ascii_lowercase(),
        finding.subcategory.to_ascii_lowercase(),
        finding.title.to_ascii_lowercase(),
        finding.description.to_ascii_lowercase()
    );
    if key.contains("form") || key.contains("label") || key.contains("input") {
        "Forms"
    } else if key.contains("keyboard") || key.contains("tastatur") {
        "Keyboard"
    } else if key.contains("focus") || key.contains("fokus") {
        "Focus management"
    } else if key.contains("alt") || key.contains("image") || key.contains("bild") {
        "Images / alternative text"
    } else if key.contains("aria") || key.contains("role") {
        "ARIA"
    } else if key.contains("heading") || key.contains("überschrift") || key.contains("h1") {
        "Heading structure"
    } else if key.contains("landmark") || key.contains("main") || key.contains("navigation") {
        "Landmarks / page structure"
    } else {
        "Semantics"
    }
}

fn build_management_risks(reports: &[NormalizedReport]) -> Vec<ManagementRisk> {
    let legal_flags: usize = reports.iter().map(|r| r.risk.legal_flags).sum();
    let critical: usize = reports.iter().map(|r| r.severity_counts.critical).sum();
    let high: usize = reports.iter().map(|r| r.severity_counts.high).sum();
    let avg = average_accessibility_score(reports);
    let seo = average_module_score_from_reports(reports, "SEO");
    let perf = average_module_score_from_reports(reports, "Performance");
    let mobile = average_module_score_from_reports(reports, "Mobile");
    let component_findings = reports
        .iter()
        .flat_map(|r| r.findings.iter())
        .filter(|f| f.occurrence_count >= 10 || f.complexity == "high")
        .count();

    vec![
        ManagementRisk {
            dimension: "Legal / BFSG-EAA".to_string(),
            level: if legal_flags > 0 || critical > 0 {
                "high"
            } else if high > 0 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!(
                "{legal_flags} legal flags, {critical} critical and {high} high WCAG findings detected automatically."
            ),
        },
        ManagementRisk {
            dimension: "Conversion / usability".to_string(),
            level: if avg < 60 || critical > 0 || perf.is_some_and(|s| s < 50) {
                "high"
            } else if avg < 80 || high > 0 || mobile.is_some_and(|s| s < 75) {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!(
                "Average accessibility score is {avg}/100; performance {:?}, mobile {:?}.",
                perf, mobile
            ),
        },
        ManagementRisk {
            dimension: "SEO / visibility".to_string(),
            level: risk_level_from_optional_score(seo),
            rationale: seo
                .map(|score| format!("Average SEO score is {score}/100."))
                .unwrap_or_else(|| "SEO module was not run.".to_string()),
        },
        ManagementRisk {
            dimension: "Trust / brand".to_string(),
            level: if critical > 0 || avg < 50 {
                "high"
            } else if high > 0 || avg < 75 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: "Accessibility barriers can reduce perceived reliability and inclusiveness.".to_string(),
        },
        ManagementRisk {
            dimension: "Project risk".to_string(),
            level: if component_findings >= 3 {
                "high"
            } else if component_findings > 0 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
            rationale: format!("{component_findings} likely component or template issue(s) need coordinated remediation."),
        },
    ]
}

fn build_decision_actions(reports: &[NormalizedReport]) -> Vec<DecisionAction> {
    let mut findings: Vec<_> = reports.iter().flat_map(|r| r.findings.iter()).collect();
    findings.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
    });
    findings
        .into_iter()
        .take(8)
        .map(|finding| DecisionAction {
            title: finding.title.clone(),
            risk: format!("{:?}", finding.severity).to_lowercase(),
            priority: finding.remediation_priority.clone(),
            complexity: finding.complexity.clone(),
            occurrence_count: finding.occurrence_count,
            root_cause: if finding.occurrence_count >= 10 {
                "Likely shared component or template".to_string()
            } else {
                finding.subcategory.clone()
            },
            expected_impact: finding.expected_impact.clone(),
        })
        .collect()
}

fn build_internal_comparison(reports: &[NormalizedReport]) -> InternalComparison {
    let module_names = {
        let mut names = std::collections::BTreeSet::new();
        for report in reports {
            for module in &report.module_scores {
                names.insert(module.name.clone());
            }
        }
        names
    };

    let module_extremes = module_names
        .into_iter()
        .filter_map(|module| {
            let mut scored: Vec<(&NormalizedReport, u32)> = reports
                .iter()
                .filter_map(|report| {
                    report
                        .module_scores
                        .iter()
                        .find(|m| m.name == module)
                        .map(|m| (report, m.score))
                })
                .collect();
            if scored.is_empty() {
                return None;
            }
            scored.sort_by_key(|(_, score)| *score);
            let (worst_report, worst_score) = scored.first().copied()?;
            let (best_report, best_score) = scored.last().copied()?;
            Some(ModuleExtreme {
                module,
                best_url: best_report.url.clone(),
                best_score,
                worst_url: worst_report.url.clone(),
                worst_score,
            })
        })
        .collect();

    let avg = average_accessibility_score(reports);
    let outlier_urls = reports
        .iter()
        .filter_map(|report| {
            let delta = report.score as i32 - avg as i32;
            (delta <= -15).then(|| UrlOutlier {
                url: report.url.clone(),
                accessibility_score: report.score,
                batch_average: avg,
                delta_points: delta,
                reason: "Accessibility score is at least 15 points below the batch average."
                    .to_string(),
            })
        })
        .collect();

    let mut root_map: std::collections::BTreeMap<
        String,
        (usize, std::collections::BTreeSet<String>),
    > = std::collections::BTreeMap::new();
    for report in reports {
        for finding in &report.findings {
            let entry = root_map
                .entry(finding.title.clone())
                .or_insert_with(|| (0, std::collections::BTreeSet::new()));
            entry.0 += finding.occurrence_count;
            entry.1.insert(report.url.clone());
        }
    }
    let mut root_causes: Vec<_> = root_map
        .into_iter()
        .map(|(title, (occurrence_count, urls))| RootCauseSummary {
            title,
            occurrence_count,
            affected_urls: urls.len(),
            classification: if urls.len() >= 2 || occurrence_count >= 10 {
                "likely_template_or_component".to_string()
            } else {
                "page_specific".to_string()
            },
        })
        .collect();
    root_causes.sort_by(|a, b| {
        b.affected_urls
            .cmp(&a.affected_urls)
            .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
    });
    root_causes.truncate(10);

    InternalComparison {
        module_extremes,
        outlier_urls,
        root_causes,
    }
}

fn average_accessibility_score(reports: &[NormalizedReport]) -> u32 {
    if reports.is_empty() {
        0
    } else {
        reports.iter().map(|r| r.score).sum::<u32>() / reports.len() as u32
    }
}

fn average_module_score_from_reports(
    reports: &[NormalizedReport],
    module_name: &str,
) -> Option<u32> {
    let scores: Vec<u32> = reports
        .iter()
        .filter_map(|report| {
            report
                .module_scores
                .iter()
                .find(|module| module.name == module_name)
                .map(|module| module.score)
        })
        .collect();
    (!scores.is_empty()).then(|| scores.iter().sum::<u32>() / scores.len() as u32)
}

fn risk_level_from_optional_score(score: Option<u32>) -> String {
    match score {
        Some(score) if score < 60 => "high",
        Some(score) if score < 80 => "medium",
        Some(_) => "low",
        None => "unknown",
    }
    .to_string()
}

/// Number of distinct violated rules across all finding categories (issue #254).
fn distinct_rule_count(findings: &[crate::audit::normalized::NormalizedFinding]) -> usize {
    findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

/// Occurrence counts across ALL finding categories (WCAG + SEO), by severity
/// (issue #255). Distinct from `NormalizedReport.occurrence_counts`, which stays
/// WCAG-only because it drives risk classification (`SiteState`).
fn all_category_occurrence_counts(
    findings: &[crate::audit::normalized::NormalizedFinding],
) -> crate::audit::normalized::SeverityCounts {
    use crate::taxonomy::Severity;
    let occ = |sev: Severity| -> usize {
        findings
            .iter()
            .filter(|f| f.severity == sev)
            .map(|f| f.occurrence_count)
            .sum()
    };
    crate::audit::normalized::SeverityCounts {
        critical: occ(Severity::Critical),
        high: occ(Severity::High),
        medium: occ(Severity::Medium),
        low: occ(Severity::Low),
        total: findings.iter().map(|f| f.occurrence_count).sum(),
    }
}

fn aggregate_severity(pages: &[PageEntry]) -> crate::audit::normalized::SeverityCounts {
    crate::audit::normalized::SeverityCounts {
        critical: pages.iter().map(|p| p.severity_counts.critical).sum(),
        high: pages.iter().map(|p| p.severity_counts.high).sum(),
        medium: pages.iter().map(|p| p.severity_counts.medium).sum(),
        low: pages.iter().map(|p| p.severity_counts.low).sum(),
        total: pages.iter().map(|p| p.severity_counts.total).sum(),
    }
}

fn aggregate_occurrences(pages: &[PageEntry]) -> crate::audit::normalized::SeverityCounts {
    crate::audit::normalized::SeverityCounts {
        critical: pages.iter().map(|p| p.occurrence_counts.critical).sum(),
        high: pages.iter().map(|p| p.occurrence_counts.high).sum(),
        medium: pages.iter().map(|p| p.occurrence_counts.medium).sum(),
        low: pages.iter().map(|p| p.occurrence_counts.low).sum(),
        total: pages.iter().map(|p| p.occurrence_counts.total).sum(),
    }
}

fn normalized_module_score(normalized: &NormalizedReport, module_name: &str) -> Option<u32> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.score)
}

fn avg_module_score(pages: &[PageEntry], name: &str) -> Option<u32> {
    let scores: Vec<u32> = pages
        .iter()
        .flat_map(|p| p.module_scores.iter())
        .filter(|m| m.name == name)
        .map(|m| m.score)
        .collect();
    if scores.is_empty() {
        None
    } else {
        Some(scores.iter().sum::<u32>() / scores.len() as u32)
    }
}

fn with_normalized_score(
    mut value: serde_json::Value,
    normalized: &NormalizedReport,
    module_name: &str,
) -> serde_json::Value {
    let Some(entry) = normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
    else {
        return value;
    };

    if let Some(obj) = value.as_object_mut() {
        if module_name == "Performance" {
            if let Some(existing) = obj.remove("score") {
                obj.insert("score_details".to_string(), existing);
            }
        }
        obj.insert("score".to_string(), serde_json::json!(entry.score));
        obj.insert("grade".to_string(), serde_json::json!(entry.grade));
    }

    value
}

fn inject_grade(mut value: serde_json::Value, score: u32) -> serde_json::Value {
    let grade = crate::audit::AccessibilityScorer::calculate_grade(score as f32);
    if let Some(obj) = value.as_object_mut() {
        obj.insert("grade".to_string(), serde_json::json!(grade));
    }
    value
}

fn inject_unused_js_bytes(
    mut value: serde_json::Value,
    raw: &crate::audit::PerformanceResults,
) -> serde_json::Value {
    let Some(cov) = &raw.coverage else {
        return value;
    };
    if let Some(obj) = value.as_object_mut() {
        if let Some(cov_val) = obj.get_mut("coverage") {
            if let Some(cov_obj) = cov_val.as_object_mut() {
                cov_obj.insert(
                    "unused_js_bytes".to_string(),
                    serde_json::json!(cov.unused_js.unused_bytes),
                );
            }
        }
    }
    value
}

fn with_measurement_type(
    mut value: serde_json::Value,
    measurement_type: &str,
) -> serde_json::Value {
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "measurement_type".to_string(),
            serde_json::json!(measurement_type),
        );
    }
    value
}

fn batch_report_timestamp(batch_report: &BatchReport) -> DateTime<Utc> {
    batch_report
        .reports
        .iter()
        .map(|report| report.timestamp)
        .max()
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::normalize;
    use crate::cli::WcagLevel;
    use crate::wcag::WcagResults;

    fn first_page(report: &UnifiedReport) -> &PageEntry {
        &report.pages[0]
    }

    #[test]
    fn test_single_envelope_shape() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);

        assert_eq!(unified.schema_version, "2.0");
        assert_eq!(unified.report_type, "single");
        assert_eq!(unified.pages.len(), 1);
        assert_eq!(unified.summary.url_count, 1);

        let output = unified.to_json(true).unwrap();
        assert!(output.contains("\"schema_version\": \"2.0\""));
        assert!(output.contains("\"report_type\": \"single\""));
        assert!(output.contains("example.com"));
        assert!(output.contains("\"accessibility_score\": 100"));
    }

    #[test]
    fn test_single_summary_fields_present() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);

        assert_eq!(unified.summary.accessibility_score, normalized.score);
        assert_eq!(unified.summary.overall_score, normalized.overall_score);
        assert_eq!(unified.summary.violation_count, 0);
        assert_eq!(unified.summary.passed_url_count, 1);
        assert_eq!(unified.summary.failed_url_count, 0);
    }

    #[test]
    fn test_single_taxonomy_fields() {
        use crate::taxonomy::Severity;
        use crate::wcag::Violation;

        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Missing alt",
            "n1",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            500,
        );
        let normalized = normalize(&report);
        let output = format_json_normalized(&normalized, &report, true).unwrap();

        assert!(output.contains("\"dimension\""));
        assert!(output.contains("\"subcategory\""));
        assert!(output.contains("\"issue_class\""));
        assert!(output.contains("\"aggregation_key\""));
        assert!(output.contains("\"user_impact\""));
        assert!(output.contains("\"principle_coverage\""));
    }

    #[test]
    fn test_single_score_matches_normalized() {
        use crate::taxonomy::Severity;
        use crate::wcag::Violation;

        let mut results = WcagResults::new();
        results.add_violation(Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            Severity::High,
            "Missing",
            "n1",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            500,
        );
        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let page = first_page(&unified);

        assert_eq!(page.accessibility_score, normalized.score);
        assert_eq!(page.grade, normalized.grade);
        assert_eq!(page.certificate, normalized.certificate);
    }

    #[test]
    fn test_single_violations_match_severity_counts() {
        use crate::taxonomy::Severity;
        use crate::wcag::Violation;

        let mut results = WcagResults::new();
        for node in ["n1", "n2", "n3"] {
            results.add_violation(Violation::new(
                "1.1.1",
                "Alt",
                WcagLevel::A,
                Severity::High,
                "Missing alt",
                node,
            ));
        }
        results.add_violation(Violation::new(
            "1.4.3",
            "Contrast",
            WcagLevel::AA,
            Severity::High,
            "Low contrast",
            "n4",
        ));

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            500,
        );
        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let page = first_page(&unified);

        assert_eq!(
            page.violation_count,
            page.findings
                .iter()
                .map(|f| f.occurrence_count)
                .sum::<usize>()
        );
        for finding in &page.findings {
            assert!(!finding.occurrences.is_empty());
        }
    }

    #[test]
    fn test_batch_envelope_shape() {
        use crate::audit::BatchReport;

        let reports = vec![
            AuditReport::new(
                "https://example.com/a".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ),
            AuditReport::new(
                "https://example.com/b".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 200);
        let unified = UnifiedReport::batch(&batch);

        assert_eq!(unified.report_type, "batch");
        assert_eq!(unified.pages.len(), 2);
        // Batch pages carry a compact detail with fix_guidance only (#256).
        // No new data is collected — it is derived from the findings already
        // normalized for each page; with no violations fix_guidance is empty.
        assert!(unified.pages.iter().all(|p| p.detail.is_some()));
        for page in &unified.pages {
            let detail = page.detail.as_ref().expect("batch page detail present");
            assert!(detail.fix_guidance.is_empty());
        }

        let output = unified.to_json(true).unwrap();
        assert!(output.contains("\"report_type\": \"batch\""));
        assert!(output.contains("\"schema_version\": \"2.0\""));
        // No sample metadata attached → the block is omitted.
        assert!(!output.contains("\"sample\""));
    }

    #[test]
    fn test_batch_envelope_includes_sample_metadata() {
        use crate::audit::{BatchReport, SampleMetadata};

        let reports = vec![AuditReport::new(
            "https://example.com/a".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        )];
        let batch = BatchReport::from_reports(reports, vec![], 200).with_sample(SampleMetadata {
            source: "sitemap".to_string(),
            total_discovered: 487,
            audited: 20,
            sample_limit: Some(20),
            selection: "first_n".to_string(),
            is_sample: true,
        });

        let json: serde_json::Value =
            serde_json::from_str(&UnifiedReport::batch(&batch).to_json(false).unwrap()).unwrap();
        let sample = &json["sample"];
        assert_eq!(sample["source"], "sitemap");
        assert_eq!(sample["total_discovered"], 487);
        assert_eq!(sample["audited"], 20);
        assert_eq!(sample["sample_limit"], 20);
        assert_eq!(sample["selection"], "first_n");
        assert_eq!(sample["is_sample"], true);
    }

    #[test]
    fn test_worst_risk_all_low() {
        use crate::audit::compute_worst_risk;
        use crate::audit::normalized::RiskLevel;
        use crate::wcag::WcagResults;
        // No critical/high/medium pages — result must be Low
        let reports: Vec<crate::audit::normalized::NormalizedReport> = (0..3)
            .map(|_| {
                crate::audit::normalized::normalize(&AuditReport::new(
                    "https://example.com".to_string(),
                    WcagLevel::AA,
                    WcagResults::new(),
                    100,
                ))
                .normalized
            })
            .collect();
        let result = compute_worst_risk(&reports);
        assert_eq!(
            result,
            RiskLevel::Low,
            "all-low batch must report Low risk, got {:?}",
            result
        );
    }

    #[test]
    fn test_modules_under_page_detail() {
        use crate::output::module::active_modules;
        use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        )
        .with_performance(crate::audit::PerformanceResults {
            vitals: WebVitals::default(),
            score: PerformanceScore {
                overall: 80,
                grade: PerformanceGrade::Gold,
                lcp_score: Some(20),
                fcp_score: Some(20),
                cls_score: Some(20),
                interactivity_score: Some(20),
                si_score: Some(20),
                metrics_available: 5,
                size_penalty: None,
                js_penalty: None,
                request_penalty: None,
                dom_penalty: None,
                is_capped: None,
            },
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings: vec![],
        })
        .with_seo(crate::seo::SeoAnalysis::default())
        .with_security(crate::security::SecurityAnalysis {
            score: 90,
            grade: "A".to_string(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
            protection: Default::default(),
        })
        .with_ux(crate::ux::analyze_ux(&crate::AXTree::new()))
        .with_journey(crate::journey::analyze_journey(&crate::AXTree::new()));

        let active_keys: Vec<&'static str> = active_modules(&report)
            .into_iter()
            .map(|(key, _)| key)
            .collect();

        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(true).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let modules = &json_value["pages"][0]["detail"]["modules"];
        for key in &active_keys {
            assert!(
                modules.get(key).is_some(),
                "Module '{}' is active but missing from pages[0].detail.modules",
                key
            );
        }
    }

    /// Builds an AuditReport with all 11 modules registered in `active_modules()`.
    fn all_active_modules_report() -> AuditReport {
        use crate::audit::PerformanceResults;
        use crate::dark_mode::DarkModeAnalysis;
        use crate::mobile::{
            ContentSizing, FontSizeAnalysis, MobileFriendliness, TouchTargetAnalysis,
            ViewportAnalysis,
        };
        use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            0,
        )
        .with_performance(PerformanceResults {
            vitals: WebVitals::default(),
            score: PerformanceScore {
                overall: 80,
                grade: PerformanceGrade::Gold,
                lcp_score: None,
                fcp_score: None,
                cls_score: None,
                interactivity_score: None,
                si_score: None,
                metrics_available: 0,
                size_penalty: None,
                js_penalty: None,
                request_penalty: None,
                dom_penalty: None,
                is_capped: None,
            },
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings: vec![],
        })
        .with_seo(crate::seo::SeoAnalysis::default())
        .with_security(crate::security::SecurityAnalysis {
            score: 80,
            grade: "B".into(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
            protection: Default::default(),
        })
        .with_mobile(MobileFriendliness {
            score: 75,
            viewport: ViewportAnalysis::default(),
            touch_targets: TouchTargetAnalysis::default(),
            font_sizes: FontSizeAnalysis::default(),
            content_sizing: ContentSizing::default(),
            issues: vec![],
        })
        .with_ux(crate::ux::analyze_ux(&crate::AXTree::new()))
        .with_journey(crate::journey::analyze_journey(&crate::AXTree::new()))
        .with_dark_mode(DarkModeAnalysis {
            supported: false,
            class_based_dark_mode: false,
            score: 50,
            detection_methods: vec![],
            color_scheme_css: false,
            meta_color_scheme: None,
            meta_theme_color_dark: false,
            css_custom_properties: 0,
            dark_contrast_violations: 0,
            light_only_violations: 0,
            dark_only_violations: 0,
            contrast_violations: vec![],
            issues: vec![],
        })
        .with_best_practices(crate::best_practices::BestPracticesAnalysis {
            console_errors: crate::best_practices::ConsoleErrorsAnalysis {
                errors: vec![],
                warnings: vec![],
                error_count: 0,
                warning_count: 0,
            },
            vulnerable_libraries: crate::best_practices::VulnerableLibrariesAnalysis {
                detected: vec![],
                vulnerable: vec![],
                has_vulnerabilities: false,
            },
            score: 100,
        })
        .with_tech_stack(crate::tech_stack::TechStackAnalysis {
            detected: vec![],
            findings: vec![],
            score: 100,
            grade: "A".into(),
        })
        .with_patterns(crate::patterns::PatternAnalysis {
            recognized: vec![],
            violations: vec![],
            journey_candidates: vec![],
        });
        let sq = crate::source_quality::analyze_source_quality(&report);
        let av = crate::ai_visibility::analyze_ai_visibility(&report);
        report.source_quality = Some(sq);
        report.ai_visibility = Some(av);
        // content_visibility is set separately per test — its JSON emission
        // is conditional on signal_count > 0.
        report
    }

    #[test]
    fn test_json_all_active_modules_non_null() {
        use crate::output::module::active_modules;

        let report = all_active_modules_report();
        let active_keys: Vec<&'static str> = active_modules(&report)
            .into_iter()
            .map(|(key, _)| key)
            // content_visibility is intentionally skipped: the JSON emitter suppresses it
            // when signal_count == 0 (an empty fixture produces no signals). This is
            // expected behavior, not a bug — tested via the PDF ViewModel path instead.
            .filter(|k| *k != "content_visibility")
            .collect();

        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(true).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let modules = &json_value["pages"][0]["detail"]["modules"];

        for key in &active_keys {
            let value = modules.get(key);
            assert!(
                value.is_some(),
                "Module '{}' missing from pages[0].detail.modules",
                key
            );
            assert!(
                !value.unwrap().is_null(),
                "Module '{}' is null in pages[0].detail.modules",
                key
            );
        }
    }

    #[test]
    fn test_search_experience_serialized_in_single_detail_modules() {
        let report = all_active_modules_report();
        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(true).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let search_experience = &json_value["pages"][0]["detail"]["modules"]["search_experience"];

        assert!(
            search_experience.is_object(),
            "search_experience must be serialized in single report detail modules"
        );
        assert_eq!(search_experience["measurement_type"], "composite");
        assert!(search_experience["score"].as_u64().is_some());
        assert!(
            search_experience["components"]
                .as_array()
                .is_some_and(|components| !components.is_empty()),
            "search_experience must include its component inputs"
        );
    }

    #[test]
    fn test_dual_viewport_summary_serialized_in_single_detail_modules() {
        let mut report = all_active_modules_report();
        report.dual_viewport = Some(crate::audit::DualViewportResults {
            desktop: crate::audit::ViewportAuditData {
                wcag_results: WcagResults::new(),
                accessibility_score: 92.0,
                performance: None,
                seo: None,
                mobile: None,
                ux: None,
                journey: None,
                screenshot: None,
            },
            mobile: crate::audit::ViewportAuditData {
                wcag_results: WcagResults::new(),
                accessibility_score: 71.0,
                performance: None,
                seo: None,
                mobile: None,
                ux: None,
                journey: None,
                screenshot: None,
            },
        });

        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(true).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let dual = &json_value["pages"][0]["detail"]["modules"]["dual_viewport"];

        assert!(dual.is_object(), "dual_viewport summary must be serialized");
        assert_eq!(dual["desktop"]["accessibility_score"], 92);
        assert_eq!(dual["mobile"]["accessibility_score"], 71);
        assert_eq!(dual["desktop"]["wcag"]["violations"], 0);
    }

    #[test]
    fn test_score_breakdown_present_for_viewport_weighted() {
        use crate::audit::{ViewportScoreSet, ViewportScores};

        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.viewport_scores = Some(ViewportScores {
            desktop: ViewportScoreSet {
                accessibility: 100,
                performance: None,
                overall: 100,
            },
            mobile: ViewportScoreSet {
                accessibility: 100,
                performance: None,
                overall: 100,
            },
            weighted_overall: 100,
        });
        let normalized = normalize(&report);
        assert_eq!(normalized.score_calculation_method, "viewport_weighted");
        assert!(
            normalized.score_breakdown.is_some(),
            "NormalizedReport must have score_breakdown for viewport_weighted"
        );
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(true).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(
            json_value["pages"][0].get("score_breakdown").is_some()
                && !json_value["pages"][0]["score_breakdown"].is_null(),
            "score_breakdown must be present and non-null for viewport_weighted pages"
        );
    }

    #[test]
    fn test_batch_page_detail_omitted_when_none() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let normalized = normalize(&report);
        let page = super::build_page(&normalized, None, None);
        let json_str = serde_json::to_string(&page).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(
            !json_value.as_object().unwrap().contains_key("detail"),
            "batch page must not emit \"detail\" key when detail is None, got: {}",
            json_str
        );
    }

    #[test]
    fn test_collection_errors_absent_when_empty() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let normalized = normalize(&report);
        let unified = UnifiedReport::single(&normalized, &report);
        let json_str = unified.to_json(false).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert!(
            !json_value
                .as_object()
                .unwrap()
                .contains_key("collection_errors"),
            "collection_errors must be absent from JSON when there are no errors"
        );
        let detail = &json_value["pages"][0]["detail"];
        assert!(
            !detail
                .as_object()
                .unwrap()
                .contains_key("collection_errors"),
            "detail.collection_errors must be absent from JSON when there are no errors"
        );
    }

    #[test]
    fn test_collection_errors_serialized_when_present() {
        let mut unified = UnifiedReport {
            schema_version: "2.0",
            report_type: "batch",
            tool_version: env!("CARGO_PKG_VERSION"),
            metadata: ReportMetadata {
                tool: "test".to_string(),
                timestamp: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
                wcag_level: "AA".to_string(),
                execution_time_ms: 0,
            },
            summary: UnifiedSummary {
                url_count: 0,
                accessibility_score: 0,
                overall_score: 0,
                grade: "F".to_string(),
                certificate: "None".to_string(),
                risk_level: crate::audit::normalized::RiskLevel::Low,
                violation_count: 0,
                severity_counts: crate::audit::normalized::SeverityCounts {
                    critical: 0,
                    high: 0,
                    medium: 0,
                    low: 0,
                    total: 0,
                },
                severity_counts_scope: "wcag_only".to_string(),
                occurrence_counts: crate::audit::normalized::SeverityCounts::default(),
                passed_url_count: 0,
                failed_url_count: 0,
                violated_rule_count: 0,
                top_recurring_rules: vec![],
                performance_score: None,
                seo_score: None,
                security_score: None,
                mobile_score: None,
                ux_score: None,
                journey_score: None,
                performance_throttled_avg_score: None,
                lh_mobile_score: None,
                wcag_coverage: build_wcag_coverage_for_level("AA"),
                accessibility_score_breakdown: vec![],
                management_risks: vec![],
                top_actions: vec![],
            },
            sample: None,
            pages: vec![],
            url_matrix: vec![],
            internal_comparison: None,
            crawl_diagnostics: None,
            errors: vec![],
            collection_errors: vec![ReportError {
                module: "crawl_diagnostics",
                error_type: "serialization_failed",
                reason: "NaN value in field".to_string(),
            }],
            verdict: Verdict::Pass,
            verdict_reasons: Vec::new(),
        };
        let json_str = unified.to_json(false).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let errs = &json_value["collection_errors"];
        assert!(errs.is_array(), "collection_errors must be an array");
        assert_eq!(errs.as_array().unwrap().len(), 1);
        assert_eq!(errs[0]["module"], "crawl_diagnostics");
        assert_eq!(errs[0]["error_type"], "serialization_failed");
        assert!(errs[0]["reason"].as_str().unwrap().contains("NaN"));

        // Verify detail-level collection_errors work the same way
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let normalized = normalize(&report);
        let mut page_unified = UnifiedReport::single(&normalized, &report);
        if let Some(detail) = page_unified.pages[0].detail.as_mut() {
            detail.collection_errors.push(ReportError {
                module: "tech_stack",
                error_type: "serialization_failed",
                reason: "custom serializer error".to_string(),
            });
        }
        let json_str2 = page_unified.to_json(false).unwrap();
        let json_value2: serde_json::Value = serde_json::from_str(&json_str2).unwrap();
        let detail_errs = &json_value2["pages"][0]["detail"]["collection_errors"];
        assert!(detail_errs.is_array());
        assert_eq!(detail_errs[0]["module"], "tech_stack");
        // suppress unused warning from the first mut binding
        let _ = &mut unified;
    }
}
