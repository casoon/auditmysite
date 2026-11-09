//! WCAG 1.4.12 Text Spacing (Level AA, WCAG 2.1)
//!
//! No loss of content or functionality occurs when a user applies the
//! following text-spacing properties: line height (line spacing) to at least
//! 1.5 times the font size, spacing following paragraphs to at least 2 times
//! the font size, letter spacing (tracking) to at least 0.12 times the font
//! size, and word spacing to at least 0.16 times the font size.
//!
//! Injects a stylesheet enforcing these minimums, then checks whether any
//! `overflow: hidden` container now clips its own content — the standard,
//! well-established heuristic for this check (matches the W3C's own "Text
//! Spacing" testing bookmarklet technique). The injected stylesheet is
//! removed before returning, so this check has no side effects for the
//! caller to clean up (unlike the viewport-based 1.4.10/1.4.4 checks).

use chromiumoxide::Page;
use tracing::warn;

use crate::wcag::types::{RuleMetadata, Severity, Violation};
use crate::wcag::WcagLevel;

pub(super) const TEXT_SPACING_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.12",
    name: "Text Spacing",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Content remains readable when text spacing is increased to WCAG minimums",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/text-spacing.html",
    axe_id: "text-spacing",
    tags: &["wcag2aa", "wcag1412", "cat.text"],
};

const TEXT_SPACING_JS: &str = include_str!("js/text_spacing.js");

pub async fn check_text_spacing_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(TEXT_SPACING_JS).await {
        Ok(r) => r,
        Err(e) => {
            warn!("text-spacing probe JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "text-spacing",
                crate::wcag::WcagLevel::AA,
                "page_evaluation_failed",
            )];
        }
    };

    // `null` is the probe's "nothing clipped"; a missing or non-string value
    // is no answer and must not pass (#777).
    let selector = match result.value() {
        Some(serde_json::Value::String(selector)) => selector.clone(),
        None if crate::wcag::types::returned_null(&result) => return vec![],
        _ => {
            return vec![crate::wcag::technical_rule_failure_for(
                "text-spacing",
                crate::wcag::WcagLevel::AA,
                "missing_evaluation_value",
            )]
        }
    };

    vec![Violation::new(
        TEXT_SPACING_RULE.id,
        TEXT_SPACING_RULE.name,
        TEXT_SPACING_RULE.level,
        TEXT_SPACING_RULE.severity,
        format!(
            "Content is clipped by '{selector}' when WCAG-minimum text spacing \
             (line-height 1.5×, letter-spacing 0.12em, word-spacing 0.16em, \
             paragraph spacing 2×) is applied."
        ),
        &selector,
    )
    .with_selector(&selector)
    .with_fix(
        "Avoid fixed heights/widths with overflow: hidden on text containers; \
         use min-height or allow containers to grow with content.",
    )
    .with_rule_id(TEXT_SPACING_RULE.axe_id)
    .with_help_url(TEXT_SPACING_RULE.help_url)]
}
