//! WCAG 4.1.2 - ARIA Required Parent
//!
//! Validates that ARIA roles which must be nested inside a specific parent role
//! actually have a matching ancestor.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA required parent
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Required Parent",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "Certain ARIA roles must be contained in specific parent roles",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-required-parent",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles and the parent roles they must be nested within
const REQUIRED_PARENTS: &[(&str, &[&str])] = &[
    ("listitem", &["list"]),
    ("menuitem", &["menu", "menubar", "group"]),
    ("menuitemcheckbox", &["menu", "menubar", "group"]),
    ("menuitemradio", &["menu", "menubar", "group"]),
    ("option", &["listbox", "group"]),
    ("tab", &["tablist"]),
    ("treeitem", &["tree", "group"]),
    ("gridcell", &["row"]),
    ("row", &["grid", "treegrid", "table", "rowgroup"]),
    ("rowgroup", &["grid", "treegrid", "table"]),
    ("columnheader", &["row"]),
    ("rowheader", &["row"]),
];

/// Walk up the tree to check if any ancestor has one of the specified roles
fn has_ancestor_with_role(
    node: &crate::accessibility::AXNode,
    roles: &[&str],
    tree: &AXTree,
) -> bool {
    let mut current_parent_id = node.parent_id.as_deref();

    while let Some(parent_id) = current_parent_id {
        if let Some(parent) = tree.nodes.get(parent_id) {
            if let Some(parent_role) = parent.role.as_deref() {
                if roles.contains(&parent_role) {
                    return true;
                }
            }
            current_parent_id = parent.parent_id.as_deref();
        } else {
            break;
        }
    }

    false
}

/// Check that elements with roles requiring a parent context have one
pub fn check_aria_required_parent(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        let required_parents = match REQUIRED_PARENTS.iter().find(|(r, _)| *r == role) {
            Some((_, parents)) => parents,
            None => continue,
        };

        if !has_ancestor_with_role(node, required_parents, tree) {
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                format!(
                    "Element with role '{}' must be contained in a parent with role: {}",
                    role,
                    required_parents.join(", ")
                ),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_rule_id(RULE_META.axe_id)
            .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
            .with_fix(format!(
                "Place this element inside a parent with role: {}",
                required_parents.join(", ")
            ))
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        }
    }

    results
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
    fn test_tab_inside_tablist_passes() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "tablist", Some("1"), vec!["3"]),
            make_node("3", "tab", Some("2"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_parent(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_tab_without_tablist_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "tab", Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_parent(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("tablist"));
    }

    #[test]
    fn test_listitem_in_list_passes() {
        let nodes = vec![
            make_node("1", "list", None, vec!["2"]),
            make_node("2", "listitem", Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_parent(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
