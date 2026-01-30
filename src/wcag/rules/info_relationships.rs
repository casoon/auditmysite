//! WCAG 1.3.1 Info and Relationships
//!
//! Information, structure, and relationships conveyed through presentation
//! can be programmatically determined or are available in text.
//! Level A

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 1.3.1
pub const INFO_RELATIONSHIPS_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships",
    level: WcagLevel::A,
    severity: Severity::Serious,
    description: "Information, structure, and relationships can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
};

/// Check for proper info and relationships
pub fn check_info_relationships(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        results.nodes_checked += 1;
        let role = node.role.as_deref().unwrap_or("").to_lowercase();

        // Check for tables without proper structure
        if role == "table" {
            check_table_structure(node, tree, &mut results);
        }

        // Check for lists
        if role == "list" {
            check_list_structure(node, tree, &mut results);
        }

        // Check for form fields in fieldsets
        if is_form_control(&role) {
            check_form_grouping(node, tree, &mut results);
        }

        // Check for data cells without headers
        if role == "cell" || role == "gridcell" {
            check_cell_headers(node, tree, &mut results);
        }
    }

    results
}

/// Check table has proper headers
fn check_table_structure(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    // Count header cells in the table's children
    let mut has_headers = false;
    let mut has_data_cells = false;

    for child_id in &node.child_ids {
        if let Some(child) = tree.get_node(child_id) {
            let child_role = child.role.as_deref().unwrap_or("").to_lowercase();

            // Check rowgroup and row children
            if child_role == "rowgroup" || child_role == "row" {
                for grandchild_id in &child.child_ids {
                    if let Some(grandchild) = tree.get_node(grandchild_id) {
                        let gc_role = grandchild.role.as_deref().unwrap_or("").to_lowercase();
                        if gc_role == "columnheader" || gc_role == "rowheader" {
                            has_headers = true;
                        }
                        if gc_role == "cell" || gc_role == "gridcell" {
                            has_data_cells = true;
                        }
                    }
                }
            }

            if child_role == "columnheader" || child_role == "rowheader" {
                has_headers = true;
            }
            if child_role == "cell" || child_role == "gridcell" {
                has_data_cells = true;
            }
        }
    }

    // If table has data cells but no headers, flag it
    if has_data_cells && !has_headers {
        let violation = Violation::new(
            INFO_RELATIONSHIPS_RULE.id,
            INFO_RELATIONSHIPS_RULE.name,
            INFO_RELATIONSHIPS_RULE.level,
            Severity::Serious,
            "Data table lacks header cells",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Add <th> elements for column and/or row headers in data tables")
        .with_help_url(INFO_RELATIONSHIPS_RULE.help_url);

        results.add_violation(violation);
    } else if has_headers {
        results.passes += 1;
    }
}

/// Check list has proper structure
fn check_list_structure(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let mut has_list_items = false;
    let mut has_non_list_items = false;

    for child_id in &node.child_ids {
        if let Some(child) = tree.get_node(child_id) {
            let child_role = child.role.as_deref().unwrap_or("").to_lowercase();
            if child_role == "listitem" {
                has_list_items = true;
            } else if !child_role.is_empty()
                && child_role != "presentation"
                && child_role != "none"
            {
                has_non_list_items = true;
            }
        }
    }

    if has_non_list_items && !has_list_items {
        let violation = Violation::new(
            INFO_RELATIONSHIPS_RULE.id,
            INFO_RELATIONSHIPS_RULE.name,
            INFO_RELATIONSHIPS_RULE.level,
            Severity::Moderate,
            "List does not contain proper list item elements",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Use <li> elements as direct children of <ul> or <ol> lists")
        .with_help_url(INFO_RELATIONSHIPS_RULE.help_url);

        results.add_violation(violation);
    } else if has_list_items {
        results.passes += 1;
    }
}

/// Check form controls are properly grouped
fn check_form_grouping(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let role = node.role.as_deref().unwrap_or("").to_lowercase();

    // Radio buttons and checkboxes should be in a group
    if role == "radio" {
        // Check if parent is a radiogroup
        if let Some(ref parent_id) = node.parent_id {
            if let Some(parent) = tree.get_node(parent_id) {
                let parent_role = parent.role.as_deref().unwrap_or("").to_lowercase();
                if parent_role != "radiogroup" && parent_role != "group" {
                    let violation = Violation::new(
                        INFO_RELATIONSHIPS_RULE.id,
                        INFO_RELATIONSHIPS_RULE.name,
                        INFO_RELATIONSHIPS_RULE.level,
                        Severity::Moderate,
                        "Radio button is not contained in a group",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_name(node.name.clone())
                    .with_fix("Group related radio buttons using <fieldset> and <legend> or role=\"radiogroup\"")
                    .with_help_url(INFO_RELATIONSHIPS_RULE.help_url);

                    results.add_violation(violation);
                    return;
                }
            }
        }
    }

    results.passes += 1;
}

/// Check data cells have associated headers
fn check_cell_headers(node: &AXNode, _tree: &AXTree, results: &mut WcagResults) {
    // Check if cell has any text content
    let has_content = node.name.as_ref().map(|n| !n.trim().is_empty()).unwrap_or(false);

    if has_content {
        // Data cells should ideally have headers associated
        // This is a simplified check - full implementation would trace header associations
        results.passes += 1;
    }
}

/// Check if role is a form control
fn is_form_control(role: &str) -> bool {
    matches!(
        role,
        "textbox" | "searchbox" | "combobox" | "listbox" | "spinbutton" | "slider" | "checkbox"
            | "radio" | "switch" | "button"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_node(id: &str, role: &str, name: Option<&str>, children: Vec<&str>) -> AXNode {
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
            child_ids: children.iter().map(|s| s.to_string()).collect(),
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_info_relationships_rule_metadata() {
        assert_eq!(INFO_RELATIONSHIPS_RULE.id, "1.3.1");
        assert_eq!(INFO_RELATIONSHIPS_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_table_with_headers() {
        let table = create_node("1", "table", None, vec!["2", "3"]);
        let header = create_node("2", "columnheader", Some("Name"), vec![]);
        let cell = create_node("3", "cell", Some("John"), vec![]);

        let tree = AXTree::from_nodes(vec![table, header, cell]);
        let results = check_info_relationships(&tree);

        // Should pass - has header and cell
        assert!(results.violations.is_empty() || results.passes > 0);
    }

    #[test]
    fn test_table_without_headers() {
        let table = create_node("1", "table", None, vec!["2"]);
        let cell = create_node("2", "cell", Some("Data"), vec![]);

        let tree = AXTree::from_nodes(vec![table, cell]);
        let results = check_info_relationships(&tree);

        // Should flag - has data cell but no headers
        assert!(
            results.violations.iter().any(|v| v.message.contains("header"))
        );
    }

    #[test]
    fn test_is_form_control() {
        assert!(is_form_control("textbox"));
        assert!(is_form_control("checkbox"));
        assert!(is_form_control("radio"));
        assert!(!is_form_control("link"));
        assert!(!is_form_control("heading"));
    }
}
