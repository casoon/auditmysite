//! WCAG 1.3.1 - List Structure
//!
//! Checks that list items are inside list containers, that lists are non-empty,
//! and that definition terms have corresponding definitions.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for list structure rules
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships - Lists",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description:
        "List items must be inside list containers; definition terms must have definitions",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "list",
    tags: &["wcag2a", "wcag131", "cat.structure"],
};

/// Run all list-structure WCAG checks
pub fn check_list_structure(tree: &AXTree) -> WcagResults {
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

        match role {
            "listitem" => check_listitem_in_list(node, tree, &mut results),
            "list" => check_list_not_empty(node, tree, &mut results),
            "term" => check_term_has_definition(node, tree, &mut results),
            _ => {}
        }
    }

    results
}

/// listitem must have a list or group ancestor
fn check_listitem_in_list(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let has_list_parent = has_ancestor_with_role(node, &["list", "group"], tree);

    if !has_list_parent {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "List item is not contained within a list element",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix("Ensure list items (role=\"listitem\") are inside a list (role=\"list\") element")
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// list must have at least one child node
fn check_list_not_empty(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    // Filter out ignored children
    let visible_children = node
        .child_ids
        .iter()
        .filter(|cid| tree.get_node(cid).map(|c| !c.ignored).unwrap_or(false))
        .count();

    if visible_children == 0 {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Low,
            "List element has no list items",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix("Either remove the empty list or add list item elements inside it")
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// term must have at least one sibling definition under the same parent
fn check_term_has_definition(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let parent = match node.parent_id.as_deref().and_then(|pid| tree.get_node(pid)) {
        Some(p) => p,
        None => {
            // No parent — cannot verify structure, flag as violation
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::Medium,
                "Definition term has no corresponding definition",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Each <dt> (role=\"term\") must be paired with at least one <dd> (role=\"definition\") under the same parent",
            )
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
            return;
        }
    };

    let has_sibling_definition = parent.child_ids.iter().any(|sid| {
        if sid == &node.node_id {
            return false;
        }
        tree.get_node(sid)
            .and_then(|s| s.role.as_deref())
            .map(|r| r == "definition")
            .unwrap_or(false)
    });

    if !has_sibling_definition {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "Definition term has no corresponding definition",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix(
            "Each <dt> (role=\"term\") must be paired with at least one <dd> (role=\"definition\") under the same parent",
        )
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Walk up the ancestor chain and check for a matching role
fn has_ancestor_with_role(node: &AXNode, roles: &[&str], tree: &AXTree) -> bool {
    let mut current_parent_id = node.parent_id.as_deref();

    while let Some(parent_id) = current_parent_id {
        if let Some(parent) = tree.get_node(parent_id) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn make_node(
        id: &str,
        role: &str,
        name: Option<&str>,
        parent_id: Option<&str>,
        child_ids: Vec<&str>,
    ) -> AXNode {
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
            child_ids: child_ids.into_iter().map(String::from).collect(),
            parent_id: parent_id.map(String::from),
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_listitem_outside_list_flagged() {
        let nodes = vec![make_node("1", "listitem", Some("Item"), None, vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_list_structure(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("not contained within a list")));
    }

    #[test]
    fn test_listitem_inside_list_passes() {
        let nodes = vec![
            make_node("L", "list", None, None, vec!["1"]),
            make_node("1", "listitem", Some("Item"), Some("L"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_list_structure(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("not contained within a list")));
    }

    #[test]
    fn test_empty_list_flagged() {
        let nodes = vec![make_node("1", "list", None, None, vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_list_structure(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no list items")));
    }

    #[test]
    fn test_term_without_definition_flagged() {
        let nodes = vec![
            make_node("P", "generic", None, None, vec!["T"]),
            make_node("T", "term", Some("Word"), Some("P"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_list_structure(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no corresponding definition")));
    }

    #[test]
    fn test_term_with_definition_passes() {
        let nodes = vec![
            make_node("P", "generic", None, None, vec!["T", "D"]),
            make_node("T", "term", Some("Word"), Some("P"), vec![]),
            make_node("D", "definition", Some("Explanation"), Some("P"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_list_structure(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("no corresponding definition")));
    }
}
