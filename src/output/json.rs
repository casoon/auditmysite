//! JSON Output Formatter
//!
//! Generates machine-readable JSON reports.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::audit::AuditReport;
use crate::error::Result;

/// Generate JSON output from an audit report
pub fn format_json(report: &AuditReport, pretty: bool) -> Result<String> {
    let output = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    };

    output.map_err(|e| crate::error::AuditError::OutputError {
        reason: format!("JSON serialization failed: {}", e),
    })
}

/// Extended JSON report with additional metadata
#[derive(Debug, Serialize)]
pub struct JsonReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// The audit results
    pub report: AuditReport,
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
    /// Create a new JSON report with metadata
    pub fn new(
        report: AuditReport,
        wcag_level: &str,
        execution_time_ms: u64,
    ) -> Self {
        Self {
            metadata: ReportMetadata {
                tool: format!("auditmysit v{}", env!("CARGO_PKG_VERSION")),
                timestamp: Utc::now(),
                wcag_level: wcag_level.to_string(),
                execution_time_ms,
            },
            report,
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
    use crate::audit::AuditReport;
    use crate::wcag::WcagResults;

    #[test]
    fn test_format_json() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagResults::new(),
            500,
        );

        let json = format_json(&report, true).unwrap();
        assert!(json.contains("example.com"));
        assert!(json.contains("\"score\": 100"));
    }

    #[test]
    fn test_json_report_with_metadata() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagResults::new(),
            1200,
        );

        let json_report = JsonReport::new(report, "AA", 1200);
        let output = json_report.to_json(true).unwrap();

        assert!(output.contains("auditmysit"));
        assert!(output.contains("\"wcag_level\": \"AA\""));
    }
}
