//! WCAG 4.1.2, 2.1.1 - Widget Rules
//!
//! Checks complex ARIA widget patterns: tabs, comboboxes, sliders, tree items.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for widget rules (4.1.2)
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value - Widgets",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description:
        "Interactive widget roles must have all required states, properties, and owned elements",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-required-children",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Maximum depth for subtree traversal
const MAX_DEPTH: usize = 10;

/// Run all widget-related WCAG checks
pub fn check_widget_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Collect tree-level facts once
    let has_tabpanel = tree
        .nodes
        .values()
        .any(|n| !n.ignored && n.role.as_deref() == Some("tabpanel"));

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        match role {
            "tablist" => check_tablist_has_tabpanel(node, has_tabpanel, &mut results),
            "tab" => check_tab_selected_state(node, &mut results),
            "combobox" => check_combobox_has_options(node, tree, &mut results),
            "slider" => check_slider_has_value(node, &mut results),
            "treeitem" => check_treeitem_in_tree(node, tree, &mut results),
            _ => {}
        }
    }

    results
}

/// tablist must be accompanied by at least one tabpanel in the tree
fn check_tablist_has_tabpanel(node: &AXNode, has_tabpanel: bool, results: &mut WcagResults) {
    if !has_tabpanel {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "Tab list has no associated tab panels",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix("Add elements with role=\"tabpanel\" that correspond to each tab in the tablist")
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/tabs/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Each tab must expose a selected/aria-selected state
fn check_tab_selected_state(node: &AXNode, results: &mut WcagResults) {
    let has_selected = node.get_property_bool("selected").is_some()
        || node.get_property_bool("aria-selected").is_some()
        || node
            .properties
            .iter()
            .any(|p| p.name == "selected" || p.name == "aria-selected");

    if !has_selected {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Low,
            "Tab is missing selected state indication",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix("Add aria-selected=\"true\" or aria-selected=\"false\" to each tab element")
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/tabs/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// combobox must have a descendant listbox or option
fn check_combobox_has_options(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let has_options = has_any_role_in_subtree(tree, node, &["listbox", "option"], MAX_DEPTH);

    if !has_options {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "Combobox has no associated options list",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix(
            "Add a popup element with role=\"listbox\" containing role=\"option\" elements, or use aria-controls to reference the listbox",
        )
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/combobox/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// slider must have an accessible value via value property or aria-valuenow
fn check_slider_has_value(node: &AXNode, results: &mut WcagResults) {
    let has_value = node.value.as_ref().is_some_and(|v| !v.trim().is_empty())
        || node.get_property_str("aria-valuenow").is_some()
        || node.get_property_int("aria-valuenow").is_some()
        || node.get_property_str("valuenow").is_some()
        || node.get_property_int("valuenow").is_some();

    if !has_value {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::High,
            "Slider is missing accessible value",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix(
            "Add aria-valuenow (current), aria-valuemin (minimum), and aria-valuemax (maximum) attributes to the slider",
        )
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/slider/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// treeitem must have a tree or group ancestor
fn check_treeitem_in_tree(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let has_tree_ancestor = has_ancestor_with_role(node, &["tree", "group"], tree);

    if !has_tree_ancestor {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "Tree item is not contained within a tree element",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix("Ensure role=\"treeitem\" elements are nested inside a role=\"tree\" container")
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/treeview/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Check if any of the given roles exist in the subtree (depth-limited)
fn has_any_role_in_subtree(tree: &AXTree, node: &AXNode, roles: &[&str], depth: usize) -> bool {
    if depth == 0 {
        return false;
    }
    for child_id in &node.child_ids {
        if let Some(child) = tree.get_node(child_id) {
            if roles.iter().any(|r| child.role.as_deref() == Some(r)) {
                return true;
            }
            if has_any_role_in_subtree(tree, child, roles, depth - 1) {
                return true;
            }
        }
    }
    false
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
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

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
    fn test_tablist_without_tabpanel_flagged() {
        let nodes = vec![
            make_node("tl", "tablist", Some("Tabs"), None, vec!["t1"]),
            make_node("t1", "tab", Some("Tab 1"), Some("tl"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_widget_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no associated tab panels")));
    }

    #[test]
    fn test_tablist_with_tabpanel_passes() {
        let nodes = vec![
            make_node("tl", "tablist", Some("Tabs"), None, vec!["t1"]),
            make_node("t1", "tab", Some("Tab 1"), Some("tl"), vec![]),
            make_node("tp1", "tabpanel", Some("Panel 1"), None, vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_widget_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("no associated tab panels")));
    }

    #[test]
    fn test_tab_without_selected_state_flagged() {
        let nodes = vec![
            make_node("tl", "tablist", Some("Tabs"), None, vec!["t1"]),
            make_node("t1", "tab", Some("Tab 1"), Some("tl"), vec![]),
            make_node("tp", "tabpanel", Some("Panel"), None, vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_widget_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("missing selected state")));
    }

    #[test]
    fn test_slider_without_value_flagged() {
        let nodes = vec![make_node("s", "slider", Some("Volume"), None, vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_widget_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("missing accessible value")));
    }

    #[test]
    fn test_slider_with_value_passes() {
        let mut node = make_node("s", "slider", Some("Volume"), None, vec![]);
        node.properties.push(AXProperty {
            name: "aria-valuenow".to_string(),
            value: AXValue::Int(50),
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_widget_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("missing accessible value")));
    }
}
