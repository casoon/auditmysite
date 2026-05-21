//! WCAG 1.3.1 / 4.1.2 - Table Rules
//!
//! Checks that data tables have captions/accessible names, header cells,
//! and that presentational tables do not incorrectly contain header cells.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for table structure rules
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships - Tables",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Data tables must have captions and header cells; presentational tables must not contain header cells",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "table-duplicate-name",
    tags: &["wcag2a", "wcag131", "cat.tables"],
};

/// Maximum recursion depth for subtree traversal
const MAX_DEPTH: usize = 10;

/// Check whether any of the listed roles exist anywhere in the subtree (depth-limited)
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

/// Run all table-related WCAG checks
pub fn check_table_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

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
            "table" | "grid" => {
                check_table_accessible_name(node, &mut results);
                if role == "table" {
                    check_table_has_headers(node, tree, &mut results);
                }
            }
            "presentation" | "none" => {
                check_presentational_table_headers(node, tree, &mut results);
            }
            "cell" | "gridcell" => {
                check_cell_has_row_parent(node, tree, &mut results);
            }
            _ => {}
        }
    }

    results
}

/// Table (or grid) must have an accessible name (caption or aria-label)
fn check_table_accessible_name(node: &AXNode, results: &mut WcagResults) {
    if !node.has_name() {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "Data table has no caption or accessible name",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Add a <caption> element inside the table, or use aria-label/aria-labelledby")
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Data tables must have at least one column or row header somewhere in their subtree
fn check_table_has_headers(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let has_headers =
        has_any_role_in_subtree(tree, node, &["columnheader", "rowheader"], MAX_DEPTH);

    if !has_headers {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::High,
            "Data table has no header cells",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix(
            "Add <th> elements (or role=\"columnheader\"/\"rowheader\") to identify table headers",
        )
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Presentational elements must not contain header cells
fn check_presentational_table_headers(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let has_headers =
        has_any_role_in_subtree(tree, node, &["columnheader", "rowheader"], MAX_DEPTH);

    if has_headers {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "Presentational table incorrectly contains header cells",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix(
            "Remove role=\"presentation\"/\"none\" if this is a data table, or remove header cells if purely for layout",
        )
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Cells must have a row parent for correct table semantics
fn check_cell_has_row_parent(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let parent_is_row = node
        .parent_id
        .as_deref()
        .and_then(|pid| tree.get_node(pid))
        .and_then(|p| p.role.as_deref())
        .map(|r| r == "row")
        .unwrap_or(false);

    if !parent_is_row {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "Table cell is not contained within a row element",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Ensure each table cell is inside a <tr> element (role=\"row\")")
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
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
    fn test_table_without_name_flagged() {
        let nodes = vec![make_node("1", "table", None, None, vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_table_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no caption")));
    }

    #[test]
    fn test_table_with_name_and_headers_passes() {
        let nodes = vec![
            make_node("1", "table", Some("Sales Data"), None, vec!["2"]),
            make_node("2", "columnheader", Some("Q1"), Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_table_rules(&tree);
        // No violations for missing name or missing headers
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("no caption") || v.message.contains("no header")));
    }

    #[test]
    fn test_table_without_headers_flagged() {
        let nodes = vec![
            make_node("1", "table", Some("Data"), None, vec!["2"]),
            make_node("2", "cell", Some("Value"), Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_table_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no header")));
    }

    #[test]
    fn test_presentational_table_with_header_flagged() {
        let nodes = vec![
            make_node("1", "presentation", None, None, vec!["2"]),
            make_node("2", "columnheader", Some("Col"), Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_table_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Presentational table")));
    }

    #[test]
    fn test_cell_without_row_parent_flagged() {
        let nodes = vec![
            make_node("1", "table", Some("T"), None, vec!["2"]),
            make_node("2", "cell", Some("V"), Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_table_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("not contained within a row")));
    }
}
