//! Shared helper functions for PDF rendering.

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
    use renderreport::components::ComponentId;
    use renderreport::theme::{Theme, TokenValue};
    let mut engine = renderreport::Engine::new()?;

    engine.components_mut().register(
        ComponentId::new("section-header-split"),
        include_str!("templates/section_header_split.typ").to_string(),
    );
    engine.components_mut().register(
        ComponentId::new("metric-strip"),
        include_str!("templates/metric_strip.typ").to_string(),
    );

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

pub(super) fn score_quality_label(score: u32) -> &'static str {
    match score {
        85..=100 => "Stark",
        70..=84 => "Solide",
        50..=69 => "Uneinheitlich",
        _ => "Schwach",
    }
}

pub(super) fn score_quality_color(score: u32) -> &'static str {
    use super::design::tokens;
    match score {
        85..=100 => tokens::SUCCESS,
        70..=84 => tokens::SUCCESS,
        50..=69 => tokens::WARN_DEEP,
        _ => tokens::DANGER,
    }
}

#[cfg(test)]
mod tests {
    use super::create_engine;
    use renderreport::components::ComponentId;

    #[test]
    fn section_eyebrow_uses_light_spacious_typography() {
        let engine = create_engine().expect("PDF engine");
        let template = engine
            .components()
            .get_template(&ComponentId::new("section-header-split"))
            .expect("section header template");

        assert!(template.contains("weight: \"regular\""));
        assert!(template.contains("tracking: 0.20em"));
        assert!(template.contains("#v(spacing-3)"));
    }

    #[test]
    fn metric_value_and_context_share_a_bottom_alignment() {
        let engine = create_engine().expect("PDF engine");
        let template = engine
            .components()
            .get_template(&ComponentId::new("metric-strip"))
            .expect("metric strip template");

        assert!(template.contains("align: bottom + left"));
        assert!(!template.contains("pad(top: 3pt)"));
    }
}
