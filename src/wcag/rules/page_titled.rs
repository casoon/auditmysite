//! WCAG 2.4.2 Page Titled
//!
//! Web pages have titles that describe topic or purpose.
//! Level A

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};
use chromiumoxide::Page;
use tracing::warn;

/// Rule metadata for 2.4.2
pub const PAGE_TITLED_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.2",
    name: "Page Titled",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Web pages have titles that describe topic or purpose",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/page-titled.html",
    axe_id: "document-title",
    tags: &["wcag2a", "wcag242", "cat.text-alternatives"],
};

/// Check for proper page title
pub fn check_page_titled(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked += 1;

    // Look for the root document node to check title
    let has_title = tree.iter().any(|node| {
        if let Some(ref role) = node.role {
            let role_lower = role.to_lowercase();
            // Check for RootWebArea which contains page title info
            if role_lower == "rootwebarea" || role_lower == "document" {
                if let Some(ref name) = node.name {
                    let title = name.trim();
                    // Check if title exists and is meaningful
                    if !title.is_empty() && !is_generic_title(title) && !is_url_like_title(title) {
                        return true;
                    }
                }
            }
        }
        false
    });

    if !has_title {
        // Check if we found any document node
        let found_document = tree.iter().any(|node| {
            node.role.as_deref().map(|r| r.to_lowercase()) == Some("rootwebarea".to_string())
                || node.role.as_deref().map(|r| r.to_lowercase()) == Some("document".to_string())
        });

        let url_fallback = tree.iter().any(|node| {
            matches!(node.role.as_deref(), Some("RootWebArea") | Some("document"))
                && node.name.as_deref().is_some_and(is_url_like_title)
        });
        if url_fallback {
            results.add_not_testable(
                Violation::new(
                    PAGE_TITLED_RULE.id,
                    PAGE_TITLED_RULE.name,
                    PAGE_TITLED_RULE.level,
                    Severity::Low,
                    "The AXTree exposes a URL-like document name; the DOM title check is authoritative.",
                    "document",
                )
                .with_kind(crate::wcag::types::FindingKind::NotTestable)
                .with_rule_id(PAGE_TITLED_RULE.axe_id),
            );
        } else if found_document {
            let violation = Violation::new(
                PAGE_TITLED_RULE.id,
                PAGE_TITLED_RULE.name,
                PAGE_TITLED_RULE.level,
                Severity::High,
                "Page has missing or non-descriptive title",
                "document",
            )
            .with_fix("Add a descriptive <title> element that describes the page topic or purpose")
            .with_help_url(PAGE_TITLED_RULE.help_url);

            results.add_violation(violation);
        }
    } else {
        results.passes += 1;
    }

    results
}

/// DOM supplement for document-title parity. Some pages expose the URL rather
/// than the empty `<title>` state in the AX tree; `document.title` is the
/// canonical signal for axe-core's `document-title` rule.
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

fn is_url_like_title(title: &str) -> bool {
    let title = title.trim();
    url::Url::parse(title).is_ok()
        || (title.starts_with("www.") && !title.chars().any(char::is_whitespace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXNode;

    fn create_document_node(id: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("RootWebArea".to_string()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_page_titled_rule_metadata() {
        assert_eq!(PAGE_TITLED_RULE.id, "2.4.2");
        assert_eq!(PAGE_TITLED_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_page_with_good_title() {
        let tree = AXTree::from_nodes(vec![create_document_node(
            "1",
            Some("Shopping Cart - Example Store"),
        )]);
        let results = check_page_titled(&tree);
        assert!(results.violations.is_empty());
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_page_with_generic_title() {
        let tree = AXTree::from_nodes(vec![create_document_node("1", Some("Untitled"))]);
        let results = check_page_titled(&tree);
        assert!(!results.violations.is_empty());
    }

    #[test]
    fn test_page_without_title() {
        let tree = AXTree::from_nodes(vec![create_document_node("1", None)]);
        let results = check_page_titled(&tree);
        assert!(!results.violations.is_empty());
    }

    #[test]
    fn url_like_axtree_name_is_not_accepted_as_a_page_title() {
        let tree = AXTree::from_nodes(vec![create_document_node(
            "1",
            Some("https://example.com/path"),
        )]);
        let results = check_page_titled(&tree);
        assert!(results.violations.is_empty());
        assert_eq!(results.not_testables.len(), 1);
    }

    #[test]
    fn test_is_generic_title() {
        assert!(is_generic_title("Untitled"));
        assert!(is_generic_title("home"));
        assert!(is_generic_title("Index"));
        assert!(!is_generic_title("Product Details - My Store"));
    }
}
