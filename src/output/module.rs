//! Module trait — compile-time interface contract for all optional audit modules.
//!
//! Every type attached as an optional module to `AuditReport` must implement
//! `ReportModule`. This enforces that each module declares a stable JSON key
//! and knows how to serialize itself.
//!
//! # Adding a new module
//! 1. Implement `ReportModule` for the new type (add an `impl` block below).
//! 2. Add a branch in `active_modules()` for the new `AuditReport` field.
//! 3. Add/update the PDF dispatch in `src/output/pdf/single_report.rs`.
//!    The test `test_all_active_modules_present_in_json` will catch any JSON gap.

use serde_json::Value;

use crate::ai_visibility::AiVisibilityAnalysis;
use crate::audit::{AuditReport, PerformanceResults};
use crate::best_practices::BestPracticesAnalysis;
use crate::content_visibility::ContentVisibilityAnalysis;
use crate::dark_mode::DarkModeAnalysis;
use crate::i18n::I18n;
use crate::journey::JourneyAnalysis;
use crate::mobile::MobileFriendliness;
use crate::patterns::PatternAnalysis;
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::source_quality::SourceQualityAnalysis;
use crate::tech_stack::TechStackAnalysis;
use crate::ux::UxAnalysis;

pub type PdfComponents = Vec<Box<dyn renderreport::components::Component>>;

/// Interface contract for optional audit modules.
pub trait ReportModule {
    /// Stable key used as the JSON field name and PDF section identifier.
    fn module_key(&self) -> &'static str;

    /// Serialize the module to a JSON value.
    fn to_json(&self) -> Value;

    /// Render a structural PDF hook for this module.
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents;
}

fn pdf_marker(module_key: &str, i18n: &I18n) -> PdfComponents {
    vec![Box::new(renderreport::components::text::TextBlock::new(
        format!("pdf-module:{module_key}:{}", i18n.locale()),
    ))]
}

/// Non-module report areas that still need explicit JSON/PDF coverage.
pub const REPORT_AREAS: &[&str] = &[
    "wcag_findings",
    "a11y_journey",
    "audit_flags",
    "search_experience",
    "screen_reader",
    "advisory_findings",
    "score_breakdown",
];

impl ReportModule for PerformanceResults {
    fn module_key(&self) -> &'static str {
        "performance"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for SeoAnalysis {
    fn module_key(&self) -> &'static str {
        "seo"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for SecurityAnalysis {
    fn module_key(&self) -> &'static str {
        "security"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for MobileFriendliness {
    fn module_key(&self) -> &'static str {
        "mobile"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for UxAnalysis {
    fn module_key(&self) -> &'static str {
        "ux"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for JourneyAnalysis {
    fn module_key(&self) -> &'static str {
        "journey"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for DarkModeAnalysis {
    fn module_key(&self) -> &'static str {
        "dark_mode"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for SourceQualityAnalysis {
    fn module_key(&self) -> &'static str {
        "source_quality"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for AiVisibilityAnalysis {
    fn module_key(&self) -> &'static str {
        "ai_visibility"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for ContentVisibilityAnalysis {
    fn module_key(&self) -> &'static str {
        "content_visibility"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for BestPracticesAnalysis {
    fn module_key(&self) -> &'static str {
        "best_practices"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for TechStackAnalysis {
    fn module_key(&self) -> &'static str {
        "tech_stack"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
    }
}

impl ReportModule for PatternAnalysis {
    fn module_key(&self) -> &'static str {
        "patterns"
    }
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
    fn render_pdf(&self, i18n: &I18n) -> PdfComponents {
        pdf_marker(self.module_key(), i18n)
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
    if let Some(ref m) = report.content_visibility {
        push(m);
    }
    if let Some(ref m) = report.best_practices {
        push(m);
    }
    if let Some(ref m) = report.tech_stack {
        push(m);
    }
    if let Some(ref m) = report.patterns {
        push(m);
    }

    out
}

/// Returns all active modules as trait objects so JSON/PDF coverage tests use
/// the same compile-time `ReportModule` contract.
pub fn active_report_modules(report: &AuditReport) -> Vec<&dyn ReportModule> {
    let mut out: Vec<&dyn ReportModule> = Vec::new();

    macro_rules! push_module {
        ($field:expr) => {
            if let Some(ref m) = $field {
                out.push(m);
            }
        };
    }

    push_module!(report.performance);
    push_module!(report.seo);
    push_module!(report.security);
    push_module!(report.mobile);
    push_module!(report.ux);
    push_module!(report.journey);
    push_module!(report.dark_mode);
    push_module!(report.source_quality);
    push_module!(report.ai_visibility);
    push_module!(report.content_visibility);
    push_module!(report.best_practices);
    push_module!(report.tech_stack);
    push_module!(report.patterns);

    out
}
