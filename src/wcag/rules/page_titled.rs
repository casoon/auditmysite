//! WCAG 2.4.2 Page Titled
//!
//! Web pages have titles that describe topic or purpose.
//! Level A

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};
use chromiumoxide::Page;
use tracing::warn;

/// Rule metadata for 2.4.2
pub const PAGE_TITLED_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.2",
    name: "Page Titled",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Web pages have titles that describe topic or purpose",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/page-titled.html",
    axe_id: "document-title",
    tags: &["wcag2a", "wcag242", "cat.text-alternatives"],
};

/// Page title check against the DOM. This is the only `document-title`
/// source: the AX tree's root name falls back to the URL when the `<title>`
/// is missing or empty, so it cannot tell those cases apart, and a second,
/// AX-based check reported every missing title twice.
pub async fn check_page_titled_with_page(page: &Page) -> Vec<Violation> {
    let result = match page
        .evaluate(
            "(function() { var title = document.querySelector('title'); return title ? title.textContent : ''; })()",
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("document-title DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for("document-title", crate::cli::WcagLevel::A, "page_evaluation_failed")];
        }
    };

    let title = result
        .value()
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();

    if !title.is_empty() && !is_generic_title(title) {
        return vec![];
    }

    vec![Violation::new(
        PAGE_TITLED_RULE.id,
        PAGE_TITLED_RULE.name,
        PAGE_TITLED_RULE.level,
        Severity::High,
        "Page has missing or non-descriptive title",
        "document",
    )
    .with_rule_id(PAGE_TITLED_RULE.axe_id)
    .with_selector("head")
    .with_tags(
        PAGE_TITLED_RULE
            .tags
            .iter()
            .map(|s| s.to_string())
            .collect(),
    )
    .with_fix("Add a descriptive <title> element that describes the page topic or purpose")
    .with_help_url(PAGE_TITLED_RULE.help_url)]
}

/// Check if a title is generic/non-descriptive
fn is_generic_title(title: &str) -> bool {
    let generic_titles = [
        "untitled",
        "untitled document",
        "new page",
        "home",
        "index",
        "page",
        "document",
        "welcome",
        "test",
        "localhost",
    ];

    let title_lower = title.to_lowercase();
    generic_titles.iter().any(|&g| title_lower == g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_titled_rule_metadata() {
        assert_eq!(PAGE_TITLED_RULE.id, "2.4.2");
        assert_eq!(PAGE_TITLED_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_is_generic_title() {
        assert!(is_generic_title("Untitled"));
        assert!(is_generic_title("home"));
        assert!(is_generic_title("Index"));
        assert!(!is_generic_title("Product Details - My Store"));
    }
}
