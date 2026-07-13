//! WCAG 1.4.4 - Meta Viewport Large Scale
//!
//! axe-core rule: `meta-viewport-large`
//! Users should be able to zoom to at least 500%. If the viewport meta tag
//! sets `maximum-scale` < 5 (and >= 2, otherwise `resize_text.rs`'s stricter
//! `meta-viewport` rule already fires) or `user-scalable=no`, this rule fires.
//!
//! This is a DOM-level rule: it reads the actual `<meta name="viewport">`
//! tag's `content` attribute via CDP. The AX tree does not expose a
//! `viewport` property — an earlier tree-based implementation of this check
//! read a non-existent AX property and never fired in production (#QA-030).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const RULE_META_VIEWPORT_LARGE: RuleMetadata = RuleMetadata {
    id: "1.4.4",
    name: "Meta Viewport Large Scale",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "The viewport meta tag must allow users to scale the page to at least 500%",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/resize-text.html",
    axe_id: "meta-viewport-large",
    tags: &["wcag2aa", "wcag144", "cat.sensory-and-visual-cues"],
};

const VIEWPORT_CONTENT_JS: &str =
    "document.querySelector('meta[name=\"viewport\"]')?.getAttribute('content') || null";

/// Check that the viewport meta tag allows zoom to at least 500%
/// (`maximum-scale` >= 5, `user-scalable` not disabled). Does not re-flag
/// `user-scalable=no`/`maximum-scale` < 2 — those are stricter WCAG-required
/// violations already raised by `check_resize_text_with_page`.
pub async fn check_meta_viewport_large_with_page(page: &Page) -> Vec<Violation> {
    let content = match read_viewport_content(page).await {
        Some(c) => c,
        None => return vec![],
    };

    if !is_viewport_restricted(&content, 5.0) || is_viewport_restricted(&content, 2.0) {
        // Either not restricted at all, or already caught by the stricter
        // 200% rule — avoid double-reporting the same meta tag.
        return vec![];
    }

    vec![Violation::new(
        RULE_META_VIEWPORT_LARGE.id,
        RULE_META_VIEWPORT_LARGE.name,
        RULE_META_VIEWPORT_LARGE.level,
        RULE_META_VIEWPORT_LARGE.severity,
        "Viewport meta tag restricts zoom below 500% (maximum-scale < 5)",
        "meta[name=viewport]",
    )
    .with_selector("meta[name=viewport]")
    .with_fix("Set maximum-scale to at least 5 or remove the maximum-scale restriction")
    .with_rule_id(RULE_META_VIEWPORT_LARGE.axe_id)
    .with_help_url(RULE_META_VIEWPORT_LARGE.help_url)]
}

pub(super) async fn read_viewport_content(page: &Page) -> Option<String> {
    page.evaluate(VIEWPORT_CONTENT_JS)
        .await
        .ok()
        .and_then(|r| r.value().and_then(|v| v.as_str().map(str::to_owned)))
}

/// Returns true if the viewport content string restricts zoom below `threshold`x.
pub(super) fn is_viewport_restricted(content: &str, threshold: f64) -> bool {
    let content = content.to_lowercase();

    // user-scalable=no or user-scalable=0 always restricts, regardless of threshold.
    if content.contains("user-scalable=no") || content.contains("user-scalable=0") {
        return true;
    }

    if let Some(pos) = content.find("maximum-scale=") {
        let after = &content[pos + "maximum-scale=".len()..];
        let value_str: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(val) = value_str.parse::<f64>() {
            return val < threshold;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_viewport_restricted_at_5() {
        assert!(is_viewport_restricted("maximum-scale=1", 5.0));
        assert!(is_viewport_restricted("maximum-scale=4.9", 5.0));
        assert!(is_viewport_restricted("user-scalable=no", 5.0));
        assert!(is_viewport_restricted("user-scalable=0", 5.0));
        assert!(!is_viewport_restricted(
            "width=device-width, initial-scale=1",
            5.0
        ));
        assert!(!is_viewport_restricted("maximum-scale=5", 5.0));
        assert!(!is_viewport_restricted("maximum-scale=10", 5.0));
    }

    #[test]
    fn test_is_viewport_restricted_at_2() {
        assert!(is_viewport_restricted("maximum-scale=1", 2.0));
        assert!(!is_viewport_restricted("maximum-scale=3", 2.0));
    }
}
