//! JSON Output Formatter
//!
//! Generates machine-readable JSON reports from the NormalizedReport model.
//! Uses the same score source as PDF output — no more inconsistencies.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::audit::normalized::NormalizedReport;
use crate::audit::AuditReport;
use crate::error::Result;

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
            timestamp: Utc::now(),
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

/// Extended JSON report with metadata + normalized data + module details
#[derive(Debug, Serialize)]
pub struct JsonReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Normalized audit results (score, grade, certificate, findings with taxonomy)
    pub report: NormalizedReport,
    /// Module detail data (performance, SEO, security, mobile)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<serde_json::Value>,
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

impl JsonReport {
    /// Create a JSON report from normalized data + raw module data
    pub fn from_normalized(normalized: &NormalizedReport, raw: &AuditReport) -> Self {
        Self {
            metadata: ReportMetadata {
                tool: format!("auditmysite v{}", env!("CARGO_PKG_VERSION")),
                timestamp: Utc::now(),
                wcag_level: normalized.wcag_level.to_string(),
                execution_time_ms: normalized.duration_ms,
            },
            report: normalized.clone(),
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
        }
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
