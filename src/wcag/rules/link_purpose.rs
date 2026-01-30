//! WCAG 2.4.4 Link Purpose (In Context)
//!
//! Ensures the purpose of each link can be determined from the link text alone
//! or from the link text together with its context.
//! Level A

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.4.4
pub const LINK_PURPOSE_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.4",
    name: "Link Purpose (In Context)",
    level: WcagLevel::A,
    severity: Severity::Moderate,
    description: "The purpose of each link can be determined from the link text or context",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context.html",
};

/// Check for link purpose issues
pub fn check_link_purpose(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored || node.role.as_deref() != Some("link") {
            continue;
        }

        results.nodes_checked += 1;
        let link_text = node.name.as_deref().unwrap_or("").trim();

        // Check for empty link text
        if link_text.is_empty() {
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                Severity::Critical,
                "Link has no accessible text",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix("Add meaningful link text, aria-label, or aria-labelledby")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
            continue;
        }

        // Check for generic/ambiguous link text
        if is_generic_link_text(link_text) {
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                LINK_PURPOSE_RULE.severity,
                format!("Link has generic text: '{}'", link_text),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Use descriptive link text that explains where the link goes")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
        } else if looks_like_url(link_text) {
            // Check for URL-only link text
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                Severity::Minor,
                "Link text appears to be a raw URL",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Replace URL with descriptive text")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
        } else if link_text.len() == 1 && !link_text.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            // Check for single character links
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                LINK_PURPOSE_RULE.severity,
                format!("Link has single character text: '{}'", link_text),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Expand single character links to be more descriptive")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }

        // Check for links that open in new window without warning
        if opens_new_window(node) && !indicates_new_window(link_text) {
            let violation = Violation::new(
                LINK_PURPOSE_RULE.id,
                LINK_PURPOSE_RULE.name,
                LINK_PURPOSE_RULE.level,
                Severity::Minor,
                "Link opens in new window without indication",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Add '(opens in new window)' to link text")
            .with_help_url(LINK_PURPOSE_RULE.help_url);

            results.add_violation(violation);
        }
    }

    results
}

/// Check if link text is generic/ambiguous
fn is_generic_link_text(text: &str) -> bool {
    let generic_phrases = [
        "click here", "click", "here", "read more", "more", "learn more",
        "info", "information", "details", "link", "this link", "go",
        "continue", "download", "view", "see more", "see all", "read",
        "start", "begin", "submit", "next", "previous", "...", ">", ">>", "→",
    ];

    let text_lower = text.to_lowercase();
    generic_phrases.iter().any(|&phrase| text_lower == phrase)
}

/// Check if text looks like a URL
fn looks_like_url(text: &str) -> bool {
    text.starts_with("http://") ||
    text.starts_with("https://") ||
    text.starts_with("www.") ||
    (text.contains(".com") && !text.contains(' ')) ||
    (text.contains(".org") && !text.contains(' ')) ||
    (text.contains(".net") && !text.contains(' '))
}

/// Check if link opens in new window
fn opens_new_window(node: &AXNode) -> bool {
    node.properties.iter().any(|p| {
        p.name.to_lowercase() == "haspopup" && p.value.as_bool().unwrap_or(false)
    })
}

/// Check if link text indicates it opens in new window
fn indicates_new_window(text: &str) -> bool {
    let indicators = [
        "new window", "new tab", "opens in", "(external)", "external link", "[external]",
    ];
    let text_lower = text.to_lowercase();
    indicators.iter().any(|&ind| text_lower.contains(ind))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_link(id: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("link".to_string()),
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
    fn test_link_purpose_rule_metadata() {
        assert_eq!(LINK_PURPOSE_RULE.id, "2.4.4");
        assert_eq!(LINK_PURPOSE_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_generic_link_text() {
        assert!(is_generic_link_text("click here"));
        assert!(is_generic_link_text("Read more"));
        assert!(is_generic_link_text("HERE"));
        assert!(!is_generic_link_text("View product specifications"));
    }

    #[test]
    fn test_looks_like_url() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("www.example.com"));
        assert!(!looks_like_url("Visit our website"));
    }

    #[test]
    fn test_empty_link() {
        let tree = AXTree::from_nodes(vec![create_link("1", None)]);
        let results = check_link_purpose(&tree);
        assert!(results.violations.iter().any(|v| v.message.contains("no accessible text")));
    }

    #[test]
    fn test_generic_link() {
        let tree = AXTree::from_nodes(vec![create_link("1", Some("click here"))]);
        let results = check_link_purpose(&tree);
        assert!(results.violations.iter().any(|v| v.message.contains("generic text")));
    }

    #[test]
    fn test_good_link_text() {
        let tree = AXTree::from_nodes(vec![create_link("1", Some("View our accessibility statement"))]);
        let results = check_link_purpose(&tree);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_url_as_link_text() {
        let tree = AXTree::from_nodes(vec![create_link("1", Some("https://example.com/page"))]);
        let results = check_link_purpose(&tree);
        assert!(results.violations.iter().any(|v| v.message.contains("raw URL")));
    }
}
