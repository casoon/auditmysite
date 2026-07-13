//! WCAG 1.4.4 Resize Text
//!
//! Except for captions and images of text, text can be resized without
//! assistive technology up to 200 percent without loss of content or functionality.
//! Level AA
//!
//! This is a DOM-level rule: it reads the actual `<meta name="viewport">`
//! tag's `content` attribute via CDP (shared with `meta_viewport_large.rs`,
//! which covers the same tag at a stricter 500% best-practice threshold).
//! The AX tree does not expose a `viewport` property — an earlier tree-based
//! implementation of this check read a non-existent AX property and never
//! fired in production (#QA-030).

use chromiumoxide::Page;

use super::meta_viewport_large::{is_viewport_restricted, read_viewport_content};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const RESIZE_TEXT_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.4",
    name: "Resize Text",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Text can be resized up to 200% without loss of content or functionality",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/resize-text.html",
    axe_id: "meta-viewport",
    tags: &["wcag2aa", "wcag144", "cat.sensory-and-visual-cues"],
};

/// Check that the viewport meta tag allows zoom to at least 200%
/// (`maximum-scale` >= 2, `user-scalable` not disabled).
pub async fn check_resize_text_with_page(page: &Page) -> Vec<Violation> {
    let content = match read_viewport_content(page).await {
        Some(c) => c,
        None => return vec![],
    };

    let content_lower = content.to_lowercase();
    let mut violations = Vec::new();

    if content_lower.contains("user-scalable=no") || content_lower.contains("user-scalable=0") {
        violations.push(
            Violation::new(
                RESIZE_TEXT_RULE.id,
                RESIZE_TEXT_RULE.name,
                RESIZE_TEXT_RULE.level,
                Severity::High,
                "Viewport meta tag prevents user scaling (user-scalable=no)",
                "meta[name=viewport]",
            )
            .with_selector("meta[name=viewport]")
            .with_fix("Remove user-scalable=no from the viewport meta tag to allow text resizing")
            .with_rule_id(RESIZE_TEXT_RULE.axe_id)
            .with_help_url(RESIZE_TEXT_RULE.help_url),
        );
    } else if is_viewport_restricted(&content, 2.0) {
        violations.push(
            Violation::new(
                RESIZE_TEXT_RULE.id,
                RESIZE_TEXT_RULE.name,
                RESIZE_TEXT_RULE.level,
                Severity::Medium,
                "Viewport maximum-scale is less than 2.0, limiting text resize",
                "meta[name=viewport]",
            )
            .with_selector("meta[name=viewport]")
            .with_fix("Set maximum-scale to at least 2.0 or remove it entirely")
            .with_rule_id(RESIZE_TEXT_RULE.axe_id)
            .with_help_url(RESIZE_TEXT_RULE.help_url),
        );
    }

    violations
}
