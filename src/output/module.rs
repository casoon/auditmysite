//! Module trait — compile-time interface contract for all optional audit modules.
//!
//! Every type attached as an optional module to `AuditReport` must implement
//! `ReportModule`. This enforces that each module declares a stable JSON key
//! and knows how to serialize itself.
//!
//! # Adding a new module
//! 1. Implement `ReportModule` for the new type (add an `impl` block below).
//! 2. Add a branch in `active_modules()` for the new `AuditReport` field.
//! 3. Add a rendering function in `src/output/pdf/detail_modules.rs`.
//!    The test `test_all_active_modules_present_in_json` will catch any JSON gap.

use serde_json::Value;

use crate::ai_visibility::AiVisibilityAnalysis;
use crate::audit::{AuditReport, PerformanceResults};
use crate::dark_mode::DarkModeAnalysis;
use crate::journey::JourneyAnalysis;
use crate::mobile::MobileFriendliness;
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::source_quality::SourceQualityAnalysis;
use crate::ux::UxAnalysis;

/// Interface contract for optional audit modules.
pub trait ReportModule {
    /// Stable key used as the JSON field name and PDF section identifier.
    fn module_key(&self) -> &'static str;

    /// Serialize the module to a JSON value.
    fn to_json(&self) -> Value;
}

impl ReportModule for PerformanceResults {
    fn module_key(&self) -> &'static str {
        "performance"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for SeoAnalysis {
    fn module_key(&self) -> &'static str {
        "seo"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for SecurityAnalysis {
    fn module_key(&self) -> &'static str {
        "security"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for MobileFriendliness {
    fn module_key(&self) -> &'static str {
        "mobile"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for UxAnalysis {
    fn module_key(&self) -> &'static str {
        "ux"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for JourneyAnalysis {
    fn module_key(&self) -> &'static str {
        "journey"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for DarkModeAnalysis {
    fn module_key(&self) -> &'static str {
        "dark_mode"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for SourceQualityAnalysis {
    fn module_key(&self) -> &'static str {
        "source_quality"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl ReportModule for AiVisibilityAnalysis {
    fn module_key(&self) -> &'static str {
        "ai_visibility"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// Returns all active (Some) modules for a report as (key, value) pairs.
///
/// This is the canonical module registry. Both JSON and PDF output must cover
/// every module returned here. The parity test in `output::json` guards this.
///
/// The closure forces each branch to go through the `ReportModule` trait —
/// a new field that skips the trait cannot compile here.
pub fn active_modules(report: &AuditReport) -> Vec<(&'static str, Value)> {
    let mut out: Vec<(&'static str, Value)> = Vec::new();
    let mut push = |m: &dyn ReportModule| out.push((m.module_key(), m.to_json()));

    if let Some(ref m) = report.performance {
        push(m);
    }
    if let Some(ref m) = report.seo {
        push(m);
    }
    if let Some(ref m) = report.security {
        push(m);
    }
    if let Some(ref m) = report.mobile {
        push(m);
    }
    if let Some(ref m) = report.ux {
        push(m);
    }
    if let Some(ref m) = report.journey {
        push(m);
    }
    if let Some(ref m) = report.dark_mode {
        push(m);
    }
    if let Some(ref m) = report.source_quality {
        push(m);
    }
    if let Some(ref m) = report.ai_visibility {
        push(m);
    }

    out
}
