//! DOM checks for modern interaction attributes (`popover`, `inert`).
//!
//! Das Popover-Ziel (`popover/target-missing`, `popover/target-invalid`) und
//! der inerte Dialog (`inert/dialog-inert`) laufen seit #691 im geteilten
//! Bestand (siehe `wcag::shared`), der fehlende Name offener Dialoge seit
//! #692 als `dialog/name-missing`. Hier bleibt, was dort keine Entsprechung
//! hat: der fehlende Name offener Menüs und Popover (nur Prüfhinweis -- weder
//! HTML noch WAI-ARIA 1.2 verlangen dort einen Namen) und das offene Popover
//! mit `inert`. Fokus in einem inerten Teilbaum gibt es nicht: HTMLs Focus
//! Fixup nimmt ihn heraus, sobald der Teilbaum inert wird (#783).

use chromiumoxide::Page;
use tracing::warn;

use crate::wcag::types::{Evidence, Outcome, RuleMetadata, Severity, Violation};
use crate::wcag::WcagLevel;

pub(super) const MODERN_ATTRIBUTES_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value - Modern Interaction Attributes",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Modern interaction attributes such as popover and inert must expose valid targets, names, and states",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "modern-attribute-misuse",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Check common misuse patterns for `popover` and `inert`.
pub async fn check_modern_attributes_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        include_str!("js/modern_attributes.js"),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("modern attributes DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "modern-attributes",
                crate::wcag::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "modern-attributes",
            crate::wcag::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let issue_type = issue
                .get("issue_type")
                .and_then(|v| v.as_str())
                .unwrap_or("modern_attribute_misuse");
            let message = issue
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or(MODERN_ATTRIBUTES_RULE.description);
            let attr = issue.get("attr").and_then(|v| v.as_str()).unwrap_or("");
            let value = issue
                .get("value")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let outcome = if issue.get("outcome").and_then(|v| v.as_str()) == Some("review") {
                Outcome::Review
            } else {
                Outcome::Fail
            };

            let mut violation = Violation::new(
                MODERN_ATTRIBUTES_RULE.id,
                MODERN_ATTRIBUTES_RULE.name,
                MODERN_ATTRIBUTES_RULE.level,
                MODERN_ATTRIBUTES_RULE.severity,
                message,
                &selector,
            )
            .with_selector(&selector)
            .with_kind(outcome)
            .with_rule_id(MODERN_ATTRIBUTES_RULE.axe_id)
            .with_tags(
                MODERN_ATTRIBUTES_RULE
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix("Fix the popover/inert attributes so the active surface has a valid target, accessible name, and operable state.")
            .with_help_url(MODERN_ATTRIBUTES_RULE.help_url)
            .with_evidence_item(Evidence::dom_attribute(attr, value));
            violation.message = format!("{message} ({issue_type})");

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}
