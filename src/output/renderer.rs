use crate::audit::normalized::NormalizedReport;
use crate::audit::AuditReport;
use crate::error::Result;

/// Transforms a single-page audit result into its final string representation.
pub trait ReportRenderer: Send + Sync {
    fn render_single_report(
        &self,
        normalized: &NormalizedReport,
        raw: &AuditReport,
    ) -> Result<String>;
}

/// JSON renderer. `pretty = true` for human-readable output.
pub struct JsonRenderer {
    pub pretty: bool,
}

impl ReportRenderer for JsonRenderer {
    fn render_single_report(
        &self,
        normalized: &NormalizedReport,
        _raw: &AuditReport,
    ) -> Result<String> {
        use crate::output::json::UnifiedReport;
        // This path is reached from a deserialized/cached NormalizedReport;
        // raw module data is no longer available, so we use single_from_normalized.
        UnifiedReport::single_from_normalized(normalized).to_json(self.pretty)
    }
}

/// Summary renderer (compact one-line-per-finding text).
pub struct SummaryRenderer;

impl ReportRenderer for SummaryRenderer {
    fn render_single_report(
        &self,
        normalized: &NormalizedReport,
        _raw: &AuditReport,
    ) -> Result<String> {
        crate::output::summary::format_summary(normalized)
            .map_err(|e| crate::error::AuditError::ConfigError(e.to_string()))
    }
}
