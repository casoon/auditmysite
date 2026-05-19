//! JSON Output Formatter — Unified Report Envelope v2.0
//!
//! Single- and batch reports share one envelope (`UnifiedReport`):
//! `schema_version` + `report_type` discriminants, a uniform `summary`, and
//! `pages[]` (1 element for single, N for batch). Per-page module detail lives
//! under `pages[i].detail` and is omitted for batch reports.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::audit::normalized::NormalizedReport;
use crate::audit::{AccessibilityScorer, AuditReport, BatchReport};
use crate::error::Result;
use crate::output::builder::{build_batch_presentation, build_view_model};
use crate::output::explanations::get_explanation;
use crate::output::module::ReportModule as _;
use crate::output::report_model::{ReportConfig, UrlMatrixRow};

/// Current envelope schema version.
const SCHEMA_VERSION: &str = "2.0";

// ─── Public entry points ──────────────────────────────────────────────────────

/// Generate single-report JSON from a normalized report.
pub fn format_json_normalized(
    normalized: &NormalizedReport,
    report: &AuditReport,
    pretty: bool,
) -> Result<String> {
    UnifiedReport::single(normalized, report).to_json(pretty)
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
    pub metadata: ReportMetadata,
    pub summary: UnifiedSummary,
    pub pages: Vec<PageEntry>,
    /// Batch only — compact per-URL score matrix.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub url_matrix: Vec<UrlMatrixRow>,
    /// Batch only — crawl diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crawl_diagnostics: Option<serde_json::Value>,
    /// Batch only — per-URL audit errors.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<serde_json::Value>,
    /// Serialization errors encountered while building this report.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collection_errors: Vec<ReportError>,
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
    pub severity_counts: crate::audit::normalized::SeverityCounts,
    pub passed_url_count: usize,
    pub failed_url_count: usize,
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
    pub nodes_analyzed: usize,
    pub duration_ms: u64,
    pub module_scores: Vec<crate::audit::normalized::ModuleScoreEntry>,
    /// How `overall_score` was computed: `"module_weighted"` or `"viewport_weighted"`.
    pub score_calculation_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_breakdown: Option<crate::audit::normalized::ScoreBreakdown>,
    pub risk: crate::audit::normalized::RiskAssessment,
    pub principle_coverage: crate::audit::PrincipleCoverage,
    pub findings: Vec<crate::audit::normalized::NormalizedFinding>,
    pub audit_flags: Vec<crate::audit::normalized::AuditFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<PageDetail>,
}

/// Per-page module detail blob — single reports only.
#[derive(Debug, Serialize)]
pub struct PageDetail {
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<serde_json::Value>,
    /// Serialization errors encountered while building this page's detail.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collection_errors: Vec<ReportError>,
}

/// Module detail data, grouped under `detail.modules`.
#[derive(Debug, Serialize)]
pub struct ModuleBlob {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility: Option<serde_json::Value>,
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
    pub occurrence_count: usize,
    pub problem: String,
    pub user_impact: String,
    pub typical_cause: String,
    pub recommendation: String,
    pub technical_note: String,
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
    pub fn single(normalized: &NormalizedReport, raw: &AuditReport) -> Self {
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

        let ctx = DetailContext {
            budget_violations,
            screenshot_status,
            collection_errors,
        };
        let page = build_page(normalized, Some(ctx));
        Self::wrap_single(normalized, page)
    }

    /// Build a single-page report from cached normalized data only.
    pub fn single_from_normalized(normalized: &NormalizedReport) -> Self {
        let page = build_page(normalized, Some(DetailContext::default()));
        Self::wrap_single(normalized, page)
    }

    /// Build a batch report — one summary page per URL, no `detail`.
    pub fn batch(batch_report: &BatchReport) -> Self {
        let normalized_reports: Vec<NormalizedReport> = batch_report
            .reports
            .iter()
            .map(crate::audit::normalize)
            .collect();
        let presentation = build_batch_presentation(batch_report);

        let pages: Vec<PageEntry> = normalized_reports
            .iter()
            .map(|n| build_page(n, None))
            .collect();

        let severity_counts = aggregate_severity(&pages);
        let accessibility_score = batch_report.summary.average_score.round() as u32;

        let worst_risk = {
            let page_count = normalized_reports.len().max(1);
            let mut counts = std::collections::HashMap::new();
            for r in normalized_reports.iter() {
                *counts.entry(r.risk.level).or_insert(0usize) += 1;
            }
            compute_worst_risk(&counts, page_count)
        };
        let summary = UnifiedSummary {
            url_count: batch_report.summary.total_urls,
            accessibility_score,
            overall_score: presentation.portfolio_summary.average_overall_score,
            grade: AccessibilityScorer::calculate_grade(accessibility_score as f32).to_string(),
            certificate: AccessibilityScorer::calculate_certificate(accessibility_score as f32)
                .to_string(),
            risk_level: worst_risk,
            violation_count: batch_report.summary.total_violations,
            severity_counts,
            passed_url_count: batch_report.summary.passed,
            failed_url_count: batch_report.summary.failed,
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
            pages,
            url_matrix: presentation.url_matrix,
            crawl_diagnostics,
            errors,
            collection_errors,
        }
    }

    /// Attach a history preview to the (single-report) first page detail.
    pub fn set_history(&mut self, history: serde_json::Value) {
        if let Some(detail) = self.pages.first_mut().and_then(|p| p.detail.as_mut()) {
            detail.history = Some(history);
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

    fn wrap_single(normalized: &NormalizedReport, page: PageEntry) -> Self {
        let passed = usize::from(page.overall_score >= 80 && page.severity_counts.critical == 0);
        let summary = UnifiedSummary {
            url_count: 1,
            accessibility_score: page.accessibility_score,
            overall_score: page.overall_score,
            grade: page.grade.clone(),
            certificate: page.certificate.clone(),
            risk_level: page.risk.level,
            violation_count: page.violation_count,
            severity_counts: page.severity_counts.clone(),
            passed_url_count: passed,
            failed_url_count: 1 - passed,
        };

        UnifiedReport {
            schema_version: SCHEMA_VERSION,
            report_type: "single",
            metadata: ReportMetadata {
                tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
                timestamp: normalized.timestamp,
                wcag_level: normalized.wcag_level.to_string(),
                execution_time_ms: normalized.duration_ms,
            },
            summary,
            pages: vec![page],
            url_matrix: Vec::new(),
            crawl_diagnostics: None,
            errors: Vec::new(),
            collection_errors: Vec::new(),
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

fn compute_worst_risk(
    counts: &std::collections::HashMap<crate::audit::normalized::RiskLevel, usize>,
    page_count: usize,
) -> crate::audit::normalized::RiskLevel {
    use crate::audit::normalized::RiskLevel;
    [RiskLevel::Critical, RiskLevel::High, RiskLevel::Medium]
        .iter()
        .copied()
        .find(|&lvl| {
            let n = *counts.get(&lvl).unwrap_or(&0);
            n > 0 && (lvl == RiskLevel::Critical || n * 5 >= page_count)
        })
        .unwrap_or(RiskLevel::Low)
}

/// Build a [`PageEntry`]. `detail` is built when `ctx` is `Some`
/// (single reports); `None` produces a compact batch page without `detail`.
fn build_page(normalized: &NormalizedReport, ctx: Option<DetailContext>) -> PageEntry {
    let detail = ctx.map(|ctx| build_detail(normalized, ctx));

    PageEntry {
        url: normalized.url.clone(),
        accessibility_score: normalized.score,
        overall_score: normalized.overall_score,
        grade: normalized.grade.clone(),
        certificate: normalized.certificate.clone(),
        violation_count: normalized.findings.iter().map(|f| f.occurrence_count).sum(),
        violated_rule_count: normalized
            .findings
            .iter()
            .filter(|f| f.category == "wcag")
            .count(),
        severity_counts: normalized.severity_counts.clone(),
        nodes_analyzed: normalized.nodes_analyzed,
        duration_ms: normalized.duration_ms,
        module_scores: normalized.module_scores.clone(),
        score_calculation_method: normalized.score_calculation_method.clone(),
        score_breakdown: normalized.score_breakdown.clone(),
        risk: normalized.risk.clone(),
        principle_coverage: normalized.principle_coverage.clone(),
        findings: normalized.findings.clone(),
        audit_flags: normalized.audit_flags.clone(),
        detail,
    }
}

fn build_detail(normalized: &NormalizedReport, ctx: DetailContext) -> PageDetail {
    let vm = build_view_model(normalized, &ReportConfig::default());
    let mut errors = ctx.collection_errors;

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

    let tech_stack = normalized.raw_tech_stack.as_ref().and_then(|m| {
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

    let patterns = normalized.raw_patterns.as_ref().map(|m| {
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
                }
                v
            }
            Err(e) => {
                errors.push(ReportError {
                    module: "patterns",
                    error_type: "serialization_failed",
                    reason: e.to_string(),
                });
                serde_json::json!({ "score": pattern_score })
            }
        }
    });

    let throttled_performance: Vec<serde_json::Value> = {
        let mut acc = Vec::new();
        for v in normalized.raw_throttled_performance.iter() {
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

    let seo = normalized.raw_seo.as_ref().map(|m| {
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
        performance: normalized
            .raw_performance
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Performance")),
        seo,
        security: normalized
            .raw_security
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Security")),
        mobile: normalized
            .raw_mobile
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Mobile")),
        ux: normalized
            .raw_ux
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "UX")),
        journey: normalized
            .raw_journey
            .as_ref()
            .map(|m| with_normalized_score(m.to_json(), normalized, "Journey")),
        dark_mode: normalized.raw_dark_mode.as_ref().map(|m| m.to_json()),
        source_quality: normalized
            .raw_source_quality
            .as_ref()
            .map(|m| with_measurement_type(m.to_json(), "heuristic")),
        ai_visibility: normalized
            .raw_ai_visibility
            .as_ref()
            .map(|m| with_measurement_type(m.to_json(), "heuristic")),
        content_visibility: normalized.raw_content_visibility.as_ref().and_then(|m| {
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
            }
            Some(v)
        }),
        tech_stack,
        patterns,
        best_practices: normalized.raw_best_practices.as_ref().map(|m| m.to_json()),
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
        budget_violations: ctx.budget_violations,
        throttled_performance,
        screenshot_status: ctx.screenshot_status,
        history: None,
        collection_errors: errors,
    }
}

/// Build fix guidance entries from normalized findings + explanation database.
fn build_fix_guidance(normalized: &NormalizedReport) -> Vec<FixGuidance> {
    normalized
        .findings
        .iter()
        .map(|finding| {
            let expl = get_explanation(&finding.rule_id);

            let mut seen = std::collections::HashSet::new();
            let affected_selectors: Vec<String> = finding
                .occurrences
                .iter()
                .filter_map(|o| o.selector.clone())
                .filter(|s| {
                    (s.contains('.')
                        || s.contains('#')
                        || s.contains('[')
                        || s.contains('>')
                        || s.contains(' '))
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
                occurrence_count: finding.occurrence_count,
                problem: expl
                    .map(|e| e.customer_description.to_string())
                    .unwrap_or_else(|| finding.description.clone()),
                user_impact: expl
                    .map(|e| e.user_impact.to_string())
                    .unwrap_or_else(|| finding.user_impact.clone()),
                typical_cause: expl
                    .map(|e| e.typical_cause.to_string())
                    .unwrap_or_default(),
                recommendation: expl
                    .map(|e| e.recommendation.to_string())
                    .unwrap_or_default(),
                technical_note: expl
                    .map(|e| e.technical_note.to_string())
                    .unwrap_or_default(),
                code_example,
                affected_selectors,
            }
        })
        .collect()
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

fn normalized_module_score(normalized: &NormalizedReport, module_name: &str) -> Option<u32> {
    normalized
        .module_scores
        .iter()
        .find(|m| m.name == module_name)
        .map(|m| m.score)
}

fn with_normalized_score(
    mut value: serde_json::Value,
    normalized: &NormalizedReport,
    module_name: &str,
) -> serde_json::Value {
    let Some(score) = normalized_module_score(normalized, module_name) else {
        return value;
    };

    if let Some(obj) = value.as_object_mut() {
        if module_name == "Performance" {
            // Move sub-scores to score_details; score itself becomes the scalar overall
            if let Some(existing) = obj.remove("score") {
                obj.insert("score_details".to_string(), existing);
            }
        }
        obj.insert("score".to_string(), serde_json::json!(score));
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
        assert!(unified.pages.iter().all(|p| p.detail.is_none()));

        let output = unified.to_json(true).unwrap();
        assert!(output.contains("\"report_type\": \"batch\""));
        assert!(output.contains("\"schema_version\": \"2.0\""));
    }

    #[test]
    fn test_worst_risk_all_low() {
        use crate::audit::normalized::RiskLevel;
        use std::collections::HashMap;
        // No critical/high/medium pages — result must be Low, not Critical
        let mut counts = HashMap::new();
        counts.insert(RiskLevel::Low, 3usize);
        let result = super::compute_worst_risk(&counts, 3);
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
                metrics_available: 4,
            },
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
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
                metrics_available: 0,
            },
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
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
        let page = super::build_page(&normalized, None);
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
                passed_url_count: 0,
                failed_url_count: 0,
            },
            pages: vec![],
            url_matrix: vec![],
            crawl_diagnostics: None,
            errors: vec![],
            collection_errors: vec![ReportError {
                module: "crawl_diagnostics",
                error_type: "serialization_failed",
                reason: "NaN value in field".to_string(),
            }],
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
