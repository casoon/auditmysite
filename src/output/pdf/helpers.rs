//! Shared helper functions for PDF rendering.

use renderreport::components::advanced::FlowGroup;
use renderreport::components::Component;

use crate::i18n::I18n;
use crate::output::report_model::*;

pub(super) fn extract_domain(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);
    host.trim_start_matches("www.").to_string()
}

/// Create engine with proper font configuration for German text
pub(super) fn create_engine() -> anyhow::Result<renderreport::Engine> {
    use renderreport::theme::{Theme, TokenValue};
    let mut engine = renderreport::Engine::new()?;

    let mut theme = Theme::default_theme();
    theme
        .tokens
        .set("font.body", TokenValue::Font("Helvetica".into()));
    theme
        .tokens
        .set("font.heading", TokenValue::Font("Georgia".into()));
    theme
        .tokens
        .set("font.mono", TokenValue::Font("JetBrains Mono".into()));
    engine.set_default_theme(theme);

    Ok(engine)
}

/// Map our severity to renderreport severity
pub(super) fn map_severity(severity: &crate::wcag::Severity) -> renderreport::prelude::Severity {
    use renderreport::prelude::Severity;
    match severity {
        crate::wcag::Severity::Critical => Severity::Critical,
        crate::wcag::Severity::High => Severity::High,
        crate::wcag::Severity::Medium => Severity::Medium,
        crate::wcag::Severity::Low => Severity::Low,
    }
}

pub(super) fn severity_label_i18n(severity: crate::wcag::Severity, i18n: &I18n) -> String {
    match severity {
        crate::wcag::Severity::Critical => i18n.t("severity-critical"),
        crate::wcag::Severity::High => i18n.t("severity-high"),
        crate::wcag::Severity::Medium => i18n.t("severity-medium"),
        crate::wcag::Severity::Low => i18n.t("severity-low"),
    }
}

pub(super) fn priority_label_i18n(priority: Priority, i18n: &I18n) -> String {
    match priority {
        Priority::Critical => i18n.t("priority-critical"),
        Priority::High => i18n.t("priority-high"),
        Priority::Medium => i18n.t("priority-medium"),
        Priority::Low => i18n.t("priority-low"),
    }
}

pub(super) fn role_label_i18n(role: Role, i18n: &I18n) -> String {
    match role {
        Role::Development => i18n.t("role-development"),
        Role::Editorial => i18n.t("role-editorial"),
        Role::DesignUx => i18n.t("role-designux"),
        Role::ProjectManagement => i18n.t("role-projectmanagement"),
    }
}

pub(super) fn effort_label_i18n(effort: Effort, i18n: &I18n) -> String {
    match effort {
        Effort::Quick => i18n.t("effort-quick"),
        Effort::Medium => i18n.t("effort-medium"),
        Effort::Structural => i18n.t("effort-structural"),
    }
}

pub(super) fn component_json<C: Component>(component: C) -> serde_json::Value {
    serde_json::json!({
        "type": component.component_id(),
        "data": component.to_data()
    })
}

pub(super) fn soft_flow_group(threshold: &str, items: Vec<serde_json::Value>) -> FlowGroup {
    let mut group = FlowGroup::new().with_spacing("12pt");
    for item in items {
        group = group.add_item(item);
    }
    group.with_keep_together_if_under(threshold)
}

#[allow(dead_code)]
pub(super) fn execution_priority_label(priority: ExecutionPriority) -> &'static str {
    match priority {
        ExecutionPriority::Immediate => "Direkt angehen",
        ExecutionPriority::Important => "Als Nächstes einplanen",
        ExecutionPriority::Optional => "Bei der nächsten Optimierungsrunde mitnehmen",
    }
}

pub(super) fn score_quality_label(score: u32) -> &'static str {
    match score {
        85..=100 => "Stark",
        70..=84 => "Solide",
        50..=69 => "Uneinheitlich",
        _ => "Schwach",
    }
}

pub(super) fn score_quality_color(score: u32) -> &'static str {
    match score {
        85..=100 => "#16a34a",
        70..=84 => "#0f766e",
        50..=69 => "#d97706",
        _ => "#dc2626",
    }
}
