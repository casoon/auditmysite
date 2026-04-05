//! WCAG 1.3.1 - Region (Landmark)
//!
//! Validates that all meaningful page content is contained within landmark regions.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for region/landmark check
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Region",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "All page content should be contained within landmark regions",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "region",
    tags: &["wcag2a", "wcag131", "cat.aria", "best-practice"],
};

/// ARIA landmark roles
const LANDMARK_ROLES: &[&str] = &[
    "banner",
    "complementary",
    "contentinfo",
    "form",
    "main",
    "navigation",
    "region",
    "search",
];

/// Roles/elements to skip (structural, non-content)
const SKIP_ROLES: &[&str] = &[
    "WebArea",
    "RootWebArea",
    "Iframe",
    "InlineTextBox",
    "LineBreak",
    "SVGRoot",
    "SvgRoot",
    "Canvas",
    "EmbeddedObject",
    "LayoutTable",
    "LayoutTableRow",
    "LayoutTableCell",
    "Unknown",
    "none",
    "presentation",
    "generic",
    "document",
];

/// Check that all meaningful content is within landmark regions
pub fn check_region(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Build a set of node IDs that are inside a landmark
    let mut inside_landmark: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Walk from root, marking nodes inside landmarks
    if let Some(root_id) = &tree.root_id {
        mark_landmark_descendants(root_id, false, tree, &mut inside_landmark);
    }

    // Now check each node
    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        // Skip structural/root nodes
        if SKIP_ROLES.contains(&role) {
            continue;
        }

        // Skip landmarks themselves
        if LANDMARK_ROLES.contains(&role) {
            continue;
        }

        // Skip hidden nodes
        if node.get_property_bool("hidden").unwrap_or(false) {
            continue;
        }

        // Only flag nodes that have visible text content (StaticText) or are
        // interactive, meaning they carry meaningful content
        let is_static_text = role == "StaticText";
        let has_text_name = node.name.as_ref().is_some_and(|n| !n.trim().is_empty());
        let is_meaningful = is_static_text
            || (has_text_name
                && matches!(
                    role,
                    "heading"
                        | "paragraph"
                        | "link"
                        | "button"
                        | "textbox"
                        | "checkbox"
                        | "radio"
                        | "img"
                        | "image"
                        | "listitem"
                        | "list"
                        | "table"
                        | "cell"
                        | "row"
                ));

        if !is_meaningful {
            continue;
        }

        // Check if inside a landmark
        if !inside_landmark.contains(&node.node_id) {
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                format!(
                    "Element with role '{}' is not contained within a landmark region",
                    role
                ),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_rule_id(RULE_META.axe_id)
            .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
            .with_fix(
                "Wrap this content in a landmark region (e.g., <main>, <nav>, <aside>, or an element with an appropriate ARIA landmark role)",
            )
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        }
    }

    results
}

/// Recursively mark all descendants of landmark nodes
fn mark_landmark_descendants(
    node_id: &str,
    in_landmark: bool,
    tree: &AXTree,
    marked: &mut std::collections::HashSet<String>,
) {
    let node = match tree.nodes.get(node_id) {
        Some(n) => n,
        None => return,
    };

    let role = node.role.as_deref().unwrap_or("");
    let is_landmark = LANDMARK_ROLES.contains(&role);
    let currently_in_landmark = in_landmark || is_landmark;

    if currently_in_landmark {
        marked.insert(node_id.to_string());
    }

    for child_id in &node.child_ids {
        mark_landmark_descendants(child_id, currently_in_landmark, tree, marked);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn make_node(id: &str, role: &str, parent_id: Option<&str>, child_ids: Vec<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some(format!("Node {}", id)),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: child_ids.into_iter().map(String::from).collect(),
            parent_id: parent_id.map(String::from),
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_content_inside_landmark_passes() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "main", Some("1"), vec!["3"]),
            make_node("3", "heading", Some("2"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_region(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_content_outside_landmark_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "heading", Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_region(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0]
            .message
            .contains("not contained within a landmark"));
    }

    #[test]
    fn test_landmark_itself_not_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "navigation", Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_region(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
