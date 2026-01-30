//! WCAG 2.4.1 Bypass Blocks
//!
//! Provides a mechanism to bypass blocks of content that are repeated.
//! Level A - Important for keyboard users to skip navigation.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.4.1
pub const BYPASS_BLOCKS_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Bypass Blocks",
    level: WcagLevel::A,
    severity: Severity::Moderate,
    description: "A mechanism is available to bypass blocks of content that are repeated",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
};

/// Check for bypass block mechanisms (skip links, landmarks)
pub fn check_bypass_blocks(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked = tree.len();

    let has_skip_link = has_skip_navigation(tree);
    let has_main_landmark = has_landmark(tree, "main");
    let has_navigation_landmark = has_landmark(tree, "navigation");
    let landmark_count = count_landmarks(tree);

    // Check for skip navigation link or main landmark
    if !has_skip_link && !has_main_landmark {
        let violation = Violation::new(
            BYPASS_BLOCKS_RULE.id,
            BYPASS_BLOCKS_RULE.name,
            BYPASS_BLOCKS_RULE.level,
            BYPASS_BLOCKS_RULE.severity,
            "No skip navigation mechanism found",
            "page",
        )
        .with_fix("Add a skip link or use <main> landmark")
        .with_help_url(BYPASS_BLOCKS_RULE.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }

    // Check for main landmark
    if !has_main_landmark {
        let violation = Violation::new(
            BYPASS_BLOCKS_RULE.id,
            BYPASS_BLOCKS_RULE.name,
            BYPASS_BLOCKS_RULE.level,
            BYPASS_BLOCKS_RULE.severity,
            "Missing main landmark",
            "page",
        )
        .with_fix("Wrap the main content in a <main> element")
        .with_help_url(BYPASS_BLOCKS_RULE.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }

    // Check for navigation landmark
    if !has_navigation_landmark && landmark_count < 2 {
        let violation = Violation::new(
            BYPASS_BLOCKS_RULE.id,
            BYPASS_BLOCKS_RULE.name,
            BYPASS_BLOCKS_RULE.level,
            Severity::Minor,
            "Missing navigation landmark",
            "page",
        )
        .with_fix("Wrap navigation in a <nav> element")
        .with_help_url(BYPASS_BLOCKS_RULE.help_url);

        results.add_violation(violation);
    }

    // Check for heading structure
    let heading_count = count_headings(tree);
    if heading_count == 0 {
        let violation = Violation::new(
            BYPASS_BLOCKS_RULE.id,
            BYPASS_BLOCKS_RULE.name,
            BYPASS_BLOCKS_RULE.level,
            BYPASS_BLOCKS_RULE.severity,
            "No headings found for content navigation",
            "page",
        )
        .with_fix("Add headings (h1-h6) to structure your content")
        .with_help_url(BYPASS_BLOCKS_RULE.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }

    results
}

/// Check for skip navigation link
fn has_skip_navigation(tree: &AXTree) -> bool {
    let skip_patterns = [
        "skip to", "skip navigation", "skip to content", "skip to main",
        "jump to", "jump to content", "go to main", "go to content",
    ];

    tree.iter().any(|node| {
        if node.role.as_deref() == Some("link") {
            if let Some(name) = &node.name {
                let name_lower = name.to_lowercase();
                return skip_patterns.iter().any(|pattern| name_lower.contains(pattern));
            }
        }
        false
    })
}

/// Check if a specific landmark exists
fn has_landmark(tree: &AXTree, landmark_type: &str) -> bool {
    tree.iter().any(|node| {
        node.role.as_deref()
            .map(|r| r.to_lowercase() == landmark_type.to_lowercase())
            .unwrap_or(false)
    })
}

/// Count total landmarks in the page
fn count_landmarks(tree: &AXTree) -> usize {
    let landmark_roles = [
        "banner", "navigation", "main", "complementary",
        "contentinfo", "region", "search", "form"
    ];

    tree.iter().filter(|node| {
        node.role.as_deref()
            .map(|r| landmark_roles.contains(&r.to_lowercase().as_str()))
            .unwrap_or(false)
    }).count()
}

/// Count headings in the page
fn count_headings(tree: &AXTree) -> usize {
    tree.iter().filter(|node| {
        node.role.as_deref() == Some("heading")
    }).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXNode;

    fn create_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
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
    fn test_bypass_blocks_rule_metadata() {
        assert_eq!(BYPASS_BLOCKS_RULE.id, "2.4.1");
        assert_eq!(BYPASS_BLOCKS_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_has_skip_navigation() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "link", Some("Skip to main content")),
            create_node("2", "main", None),
        ]);

        assert!(has_skip_navigation(&tree));
    }

    #[test]
    fn test_no_skip_navigation() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "link", Some("Home")),
            create_node("2", "link", Some("About")),
        ]);

        assert!(!has_skip_navigation(&tree));
    }

    #[test]
    fn test_has_main_landmark() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "main", None),
        ]);

        assert!(has_landmark(&tree, "main"));
    }

    #[test]
    fn test_page_with_proper_landmarks() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "banner", None),
            create_node("2", "navigation", None),
            create_node("3", "main", None),
            create_node("4", "contentinfo", None),
            create_node("5", "heading", Some("Page Title")),
        ]);

        let results = check_bypass_blocks(&tree);
        assert!(!results.violations.iter().any(|v| v.message.contains("No skip navigation")));
        assert!(!results.violations.iter().any(|v| v.message.contains("Missing main landmark")));
    }

    #[test]
    fn test_page_without_landmarks() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "generic", None),
            create_node("2", "paragraph", Some("Some text")),
        ]);

        let results = check_bypass_blocks(&tree);
        assert!(results.violations.iter().any(|v| v.message.contains("Missing main landmark")));
    }
}
