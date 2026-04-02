//! WCAG 2.4.1 / 1.3.6 - Landmark Regions
//!
//! Checks that pages contain the expected landmark regions (main, navigation,
//! banner, contentinfo) and that multiple same-type landmarks are distinguishable
//! by accessible names.

use std::collections::HashMap;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for landmark region checks (2.4.1)
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Landmark Regions",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Pages should use landmark regions to help users navigate content",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks.html",
    axe_id: "landmark-one-main",
    tags: &["wcag2a", "wcag241", "cat.semantics"],
};

/// Landmark roles used for grouping/disambiguation checks
const MULTI_LANDMARK_ROLES: &[&str] = &["navigation", "complementary", "region"];

/// Check landmark structure across the accessibility tree
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for missing or improperly labeled landmark regions
pub fn check_landmarks(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked += tree.len();

    // Collect counts and nodes for each landmark role
    let main_nodes: Vec<_> = tree.nodes_with_role("main");
    let nav_nodes: Vec<_> = tree.nodes_with_role("navigation");
    let banner_nodes: Vec<_> = tree.nodes_with_role("banner");
    let contentinfo_nodes: Vec<_> = tree.nodes_with_role("contentinfo");

    // --- main landmark checks (WCAG 2.4.1, Level A) ---

    if main_nodes.is_empty() {
        results.add_violation(
            Violation::new(
                RULE_META.id,
                RULE_META.name,
                WcagLevel::A,
                Severity::Medium,
                "Page has no main landmark",
                "root",
            )
            .with_fix("Add a <main> element or an element with role=\"main\"")
            .with_help_url(RULE_META.help_url),
        );
    } else if main_nodes.len() > 1 {
        results.add_violation(
            Violation::new(
                RULE_META.id,
                RULE_META.name,
                WcagLevel::A,
                Severity::High,
                format!(
                    "Page has {} main landmarks; only one is allowed",
                    main_nodes.len()
                ),
                "root",
            )
            .with_fix("Ensure the page has exactly one main landmark")
            .with_help_url(RULE_META.help_url),
        );
    } else {
        results.passes += 1;
    }

    // --- navigation landmark check (Level AA) ---
    if nav_nodes.is_empty() {
        results.add_violation(
            Violation::new(
                "2.4.1",
                "Landmark Regions",
                WcagLevel::AA,
                Severity::Low,
                "Page has no navigation landmark",
                "root",
            )
            .with_fix("Add a <nav> element or an element with role=\"navigation\"")
            .with_help_url(RULE_META.help_url),
        );
    } else {
        results.passes += 1;
    }

    // --- banner landmark check (Level AA) ---
    if banner_nodes.is_empty() {
        results.add_violation(
            Violation::new(
                "2.4.1",
                "Landmark Regions",
                WcagLevel::AA,
                Severity::Low,
                "Page has no banner landmark",
                "root",
            )
            .with_fix("Add a <header> element at the top level or role=\"banner\"")
            .with_help_url(RULE_META.help_url),
        );
    } else {
        results.passes += 1;
    }

    // --- contentinfo landmark check (Level AA) ---
    if contentinfo_nodes.is_empty() {
        results.add_violation(
            Violation::new(
                "2.4.1",
                "Landmark Regions",
                WcagLevel::AA,
                Severity::Low,
                "Page has no contentinfo landmark",
                "root",
            )
            .with_fix("Add a <footer> element at the top level or role=\"contentinfo\"")
            .with_help_url(RULE_META.help_url),
        );
    } else {
        results.passes += 1;
    }

    // --- Multiple same-type landmarks must have distinct accessible names (Level AA) ---
    // Collect all nodes per landmark role
    let mut role_to_nodes: HashMap<&str, Vec<&crate::accessibility::AXNode>> = HashMap::new();
    for role in MULTI_LANDMARK_ROLES {
        let nodes = tree.nodes_with_role(role);
        if !nodes.is_empty() {
            role_to_nodes.insert(role, nodes);
        }
    }

    for (role, nodes) in &role_to_nodes {
        if nodes.len() < 2 {
            continue;
        }

        for node in nodes {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        "1.3.6",
                        "Identify Purpose",
                        WcagLevel::AA,
                        Severity::Medium,
                        format!(
                            "Multiple '{}' landmarks exist but this one has no accessible name to distinguish it",
                            role
                        ),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix(format!(
                        "Add an aria-label or aria-labelledby to distinguish this '{}' landmark from others",
                        role
                    ))
                    .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/identify-purpose.html"),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn make_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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

    fn make_full_landmark_tree() -> AXTree {
        AXTree::from_nodes(vec![
            make_node("1", "WebArea", Some("Test Page")),
            make_node("2", "banner", Some("Site Header")),
            make_node("3", "navigation", Some("Main Nav")),
            make_node("4", "main", Some("Main Content")),
            make_node("5", "contentinfo", Some("Site Footer")),
        ])
    }

    #[test]
    fn test_full_landmark_set_passes() {
        let tree = make_full_landmark_tree();
        let results = check_landmarks(&tree);
        let landmark_violations: Vec<_> = results
            .violations
            .iter()
            .filter(|v| {
                v.message.contains("no main")
                    || v.message.contains("no navigation")
                    || v.message.contains("no banner")
                    || v.message.contains("no contentinfo")
            })
            .collect();
        assert!(
            landmark_violations.is_empty(),
            "Complete landmark set should not produce missing-landmark violations"
        );
    }

    #[test]
    fn test_missing_main_landmark_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", Some("Test Page")),
            make_node("2", "banner", Some("Header")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_landmarks(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("no main landmark")),
            "Missing main landmark should be flagged"
        );
    }

    #[test]
    fn test_duplicate_main_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", Some("Test")),
            make_node("2", "main", Some("Main 1")),
            make_node("3", "main", Some("Main 2")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_landmarks(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("main landmarks")),
            "Multiple main landmarks should be flagged"
        );
    }

    #[test]
    fn test_multiple_nav_without_labels_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", Some("Test")),
            make_node("2", "main", Some("Content")),
            make_node("3", "banner", Some("Header")),
            make_node("4", "contentinfo", Some("Footer")),
            make_node("5", "navigation", None), // no label
            make_node("6", "navigation", None), // no label
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_landmarks(&tree);
        let unlabeled_nav_violations: Vec<_> = results
            .violations
            .iter()
            .filter(|v| {
                v.message.contains("navigation") && v.message.contains("no accessible name")
            })
            .collect();
        assert!(
            unlabeled_nav_violations.len() >= 2,
            "Multiple unlabeled navigation landmarks should each be flagged"
        );
    }

    #[test]
    fn test_single_nav_without_label_passes() {
        let nodes = vec![
            make_node("1", "WebArea", Some("Test")),
            make_node("2", "main", Some("Content")),
            make_node("3", "banner", Some("Header")),
            make_node("4", "contentinfo", Some("Footer")),
            make_node("5", "navigation", None), // single nav, no label needed
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_landmarks(&tree);
        // No violation for single unlabeled navigation
        assert!(
            !results
                .violations
                .iter()
                .any(|v| v.message.contains("no accessible name") && v.node_id == "5"),
            "Single navigation without label should not be flagged for missing name"
        );
    }
}
