//! JSON Output Formatter
//!
//! Generates machine-readable JSON reports from the NormalizedReport model.
//! Uses the same score source as PDF output — no more inconsistencies.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::audit::normalized::NormalizedReport;
use crate::audit::{AuditReport, BatchReport};
use crate::error::Result;
use crate::output::builder::build_view_model;
use crate::output::explanations::get_explanation;
use crate::output::report_model::ReportConfig;

/// Generate JSON output from a normalized report
pub fn format_json_normalized(
    normalized: &NormalizedReport,
    report: &AuditReport,
    pretty: bool,
) -> Result<String> {
    let json_report = JsonReport::from_normalized(normalized, report);
    let output = if pretty {
        serde_json::to_string_pretty(&json_report)
    } else {
        serde_json::to_string(&json_report)
    };

    output.map_err(|e| crate::error::AuditError::OutputError {
        reason: format!("JSON serialization failed: {}", e),
    })
}

/// Generate JSON output from cached normalized data only.
pub fn format_json_cached(normalized: &NormalizedReport, pretty: bool) -> Result<String> {
    #[derive(Debug, Serialize)]
    struct CachedJsonReport<'a> {
        metadata: ReportMetadata,
        report: &'a NormalizedReport,
    }

    let payload = CachedJsonReport {
        metadata: ReportMetadata {
            tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
            timestamp: normalized.timestamp,
            wcag_level: normalized.wcag_level.to_string(),
            execution_time_ms: normalized.duration_ms,
        },
        report: normalized,
    };

    let output = if pretty {
        serde_json::to_string_pretty(&payload)
    } else {
        serde_json::to_string(&payload)
    };

    output.map_err(|e| crate::error::AuditError::OutputError {
        reason: format!("JSON serialization failed: {}", e),
    })
}

/// Generate deterministic JSON output for batch reports.
pub fn format_json_batch(batch_report: &BatchReport, pretty: bool) -> Result<String> {
    let normalized_reports: Vec<NormalizedReport> = batch_report
        .reports
        .iter()
        .map(crate::audit::normalize)
        .collect();

    let payload = BatchJsonReport {
        metadata: ReportMetadata {
            tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
            timestamp: batch_report_timestamp(batch_report),
            wcag_level: normalized_reports
                .first()
                .map(|report| report.wcag_level.to_string())
                .unwrap_or_else(|| "mixed".to_string()),
            execution_time_ms: batch_report.total_duration_ms,
        },
        summary: &batch_report.summary,
        crawl_diagnostics: batch_report.crawl_diagnostics.as_ref(),
        errors: &batch_report.errors,
        reports: normalized_reports,
    };

    let output = if pretty {
        serde_json::to_string_pretty(&payload)
    } else {
        serde_json::to_string(&payload)
    };

    output.map_err(|e| crate::error::AuditError::OutputError {
        reason: format!("JSON serialization failed: {}", e),
    })
}

/// Extended JSON report with metadata + normalized data + module details
#[derive(Debug, Serialize)]
pub struct JsonReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Normalized audit results (score, grade, certificate, findings with taxonomy)
    pub report: NormalizedReport,
    /// AI/LLM fix guidance — one entry per finding group with problem description,
    /// user impact, root cause, recommendation, and code examples
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fix_guidance: Vec<FixGuidance>,
    /// Module detail data (performance, SEO, security, mobile)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<serde_json::Value>,
    /// UX analysis (CTA clarity, visual hierarchy, trust signals, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux: Option<serde_json::Value>,
    /// Journey analysis (entry clarity, orientation, navigation, interaction, conversion)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub confidence_summary: Vec<OutputConfidenceSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<OutputCapabilitySignal>,
    /// Historical timeline for this URL (score trend, deltas, recent snapshots)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<serde_json::Value>,
}

/// AI-oriented fix guidance for a single finding group
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

/// Bad/good code example pair
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

/// Report metadata for JSON output
#[derive(Debug, Serialize)]
pub struct ReportMetadata {
    /// Tool name and version
    pub tool: String,
    /// Timestamp when report was generated
    pub timestamp: DateTime<Utc>,
    /// WCAG conformance level checked
    pub wcag_level: String,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize)]
struct BatchJsonReport<'a> {
    pub metadata: ReportMetadata,
    pub summary: &'a crate::audit::BatchSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crawl_diagnostics: Option<&'a crate::audit::CrawlDiagnostics>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: &'a Vec<crate::audit::BatchError>,
    pub reports: Vec<NormalizedReport>,
}

impl JsonReport {
    /// Create a JSON report from normalized data + raw module data
    pub fn from_normalized(normalized: &NormalizedReport, raw: &AuditReport) -> Self {
        // Build AI fix guidance from explanations + finding data
        let fix_guidance = Self::build_fix_guidance(normalized);
        let vm = build_view_model(normalized, &ReportConfig::default());

        Self {
            metadata: ReportMetadata {
                tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
                timestamp: raw.timestamp,
                wcag_level: normalized.wcag_level.to_string(),
                execution_time_ms: normalized.duration_ms,
            },
            report: normalized.clone(),
            fix_guidance,
            performance: raw
                .performance
                .as_ref()
                .and_then(|p| serde_json::to_value(p).ok()),
            seo: raw.seo.as_ref().and_then(|s| serde_json::to_value(s).ok()),
            security: raw
                .security
                .as_ref()
                .and_then(|s| serde_json::to_value(s).ok()),
            mobile: raw
                .mobile
                .as_ref()
                .and_then(|m| serde_json::to_value(m).ok()),
            ux: raw.ux.as_ref().and_then(|u| serde_json::to_value(u).ok()),
            journey: raw
                .journey
                .as_ref()
                .and_then(|j| serde_json::to_value(j).ok()),
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
            history: None,
        }
    }

    /// Build fix guidance entries from normalized findings + explanation database
    fn build_fix_guidance(normalized: &NormalizedReport) -> Vec<FixGuidance> {
        normalized
            .findings
            .iter()
            .map(|finding| {
                let expl = get_explanation(&finding.rule_id);

                // Collect unique selectors from occurrences
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

    /// Serialize to JSON string
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

    #[test]
    fn test_json_report_normalized() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let normalized = normalize(&report);
        let output = format_json_normalized(&normalized, &report, true).unwrap();

        assert!(output.contains("example.com"));
        assert!(output.contains("\"score\": 100"));
        assert!(output.contains("\"grade\": \"A\""));
        assert!(output.contains("\"certificate\": \"PLATINUM\""));
        assert!(output.contains("\"severity_counts\""));
    }

    #[test]
    fn test_json_has_taxonomy_fields() {
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
    }

    #[test]
    fn test_json_score_matches_normalized() {
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
        let json_report = JsonReport::from_normalized(&normalized, &report);

        // JSON score must equal normalized score
        assert_eq!(json_report.report.score, normalized.score);
        assert_eq!(json_report.report.grade, normalized.grade);
        assert_eq!(json_report.report.certificate, normalized.certificate);
    }
}
