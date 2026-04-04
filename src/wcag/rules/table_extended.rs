//! Extended table rules — granular header/data cell relationships
//!
//! Complements `table_rules.rs` with two axe-core rules that check header-to-data
//! cell associations:
//!
//! - `td-headers-attr`:   cells that reference non-existent or empty header IDs
//! - `th-has-data-cells`: header cells that have no corresponding data cells

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

// ── Rule metadata ──────────────────────────────────────────────────────────────

pub const RULE_TD_HEADERS_ATTR: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Table Cell Headers Attribute",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Each cell that uses the headers attribute must only refer to valid header cells in the same table",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "td-headers-attr",
    tags: &["wcag2a", "wcag131", "cat.tables"],
};

pub const RULE_TH_HAS_DATA_CELLS: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Header Cell Has Data Cells",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Each table header must have associated data cells",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "th-has-data-cells",
    tags: &["wcag2a", "wcag131", "cat.tables"],
};

// ── Public check function ──────────────────────────────────────────────────────

/// Run extended table header / data-cell relationship checks.
pub fn check_table_extended(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Collect all table nodes and their subtrees
    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        let role = match node.role.as_deref() {
            Some(r) => r.to_lowercase(),
            None => continue,
        };
        if role != "grid" && role != "table" && role != "treegrid" {
            continue;
        }

        results.nodes_checked += 1;

        // Gather all cells (columnheader, rowheader, gridcell, cell) in this table
        let mut header_cells: Vec<&AXNode> = Vec::new();
        let mut data_cells: Vec<&AXNode> = Vec::new();
        collect_table_cells(tree, node, &mut header_cells, &mut data_cells, 0);

        // ── td-headers-attr ───────────────────────────────────────────────
        // Check any cell that has a "headers" property; verify each referenced
        // ID exists as a header cell within this table.
        let header_ids: std::collections::HashSet<&str> =
            header_cells.iter().map(|h| h.node_id.as_str()).collect();

        for cell in data_cells.iter().chain(header_cells.iter()) {
            let headers_val = cell
                .get_property_str("headers")
                .or_else(|| cell.get_property_str("aria-owns"));
            if let Some(hdrs) = headers_val {
                let refs: Vec<&str> = hdrs.split_whitespace().collect();
                if refs.is_empty() {
                    // empty headers attribute
                    results.add_violation(
                        Violation::new(
                            RULE_TD_HEADERS_ATTR.id,
                            RULE_TD_HEADERS_ATTR.name,
                            RULE_TD_HEADERS_ATTR.level,
                            RULE_TD_HEADERS_ATTR.severity,
                            "Table cell has an empty headers attribute",
                            &cell.node_id,
                        )
                        .with_role(cell.role.clone())
                        .with_fix("Remove the headers attribute or reference valid th/header cell IDs")
                        .with_rule_id(RULE_TD_HEADERS_ATTR.axe_id).with_help_url(RULE_TD_HEADERS_ATTR.help_url),
                    );
                } else {
                    let mut all_valid = true;
                    for ref_id in &refs {
                        if !header_ids.contains(ref_id) {
                            all_valid = false;
                            results.add_violation(
                                Violation::new(
                                    RULE_TD_HEADERS_ATTR.id,
                                    RULE_TD_HEADERS_ATTR.name,
                                    RULE_TD_HEADERS_ATTR.level,
                                    RULE_TD_HEADERS_ATTR.severity,
                                    format!(
                                        "Table cell references header ID \"{}\" which does not exist in this table",
                                        ref_id
                                    ),
                                    &cell.node_id,
                                )
                                .with_role(cell.role.clone())
                                .with_fix("Ensure the headers attribute only references IDs of <th> elements in the same table")
                                .with_rule_id(RULE_TD_HEADERS_ATTR.axe_id).with_help_url(RULE_TD_HEADERS_ATTR.help_url),
                            );
                        }
                    }
                    if all_valid {
                        results.passes += 1;
                    }
                }
            }
        }

        // ── th-has-data-cells ─────────────────────────────────────────────
        // Each header cell should have at least one data cell in the same table.
        // Heuristic: flag header cells in tables that have zero data cells.
        if !header_cells.is_empty() {
            if data_cells.is_empty() {
                for hcell in &header_cells {
                    results.add_violation(
                        Violation::new(
                            RULE_TH_HAS_DATA_CELLS.id,
                            RULE_TH_HAS_DATA_CELLS.name,
                            RULE_TH_HAS_DATA_CELLS.level,
                            RULE_TH_HAS_DATA_CELLS.severity,
                            "Table header cell has no associated data cells in the same table",
                            &hcell.node_id,
                        )
                        .with_role(hcell.role.clone())
                        .with_fix("Ensure the table contains <td> data cells that correspond to this header, or convert this header to a data cell if no data relationship exists")
                        .with_rule_id(RULE_TH_HAS_DATA_CELLS.axe_id).with_help_url(RULE_TH_HAS_DATA_CELLS.help_url),
                    );
                }
            } else {
                results.passes += header_cells.len();
            }
        }
    }

    results
}

// ── Private helpers ───────────────────────────────────────────────────────────

const MAX_DEPTH: usize = 12;

fn collect_table_cells<'a>(
    tree: &'a AXTree,
    node: &'a AXNode,
    header_cells: &mut Vec<&'a AXNode>,
    data_cells: &mut Vec<&'a AXNode>,
    depth: usize,
) {
    if depth > MAX_DEPTH {
        return;
    }
    for child_id in &node.child_ids {
        if let Some(child) = tree.get_node(child_id) {
            if child.ignored {
                continue;
            }
            match child.role.as_deref() {
                Some("columnheader") | Some("rowheader") => header_cells.push(child),
                Some("gridcell") | Some("cell") => data_cells.push(child),
                _ => {}
            }
            collect_table_cells(tree, child, header_cells, data_cells, depth + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node(id: &str, role: &str, parent: Option<&str>, props: Vec<(&str, &str)>) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: props
                .into_iter()
                .map(|(k, v)| AXProperty {
                    name: k.into(),
                    value: AXValue::String(v.into()),
                })
                .collect(),
            child_ids: vec![],
            parent_id: parent.map(String::from),
            backend_dom_node_id: None,
        }
    }

    fn table_with_header_and_data() -> AXTree {
        let mut table = node("t", "table", None, vec![]);
        let header = node("h1", "columnheader", Some("t"), vec![]);
        let data = node("d1", "gridcell", Some("t"), vec![]);
        table.child_ids = vec!["h1".into(), "d1".into()];
        AXTree::from_nodes(vec![table, header, data])
    }

    #[test]
    fn test_header_with_data_cells_passes() {
        let tree = table_with_header_and_data();
        let r = check_table_extended(&tree);
        assert!(r.violations.is_empty(), "header + data should pass");
    }

    #[test]
    fn test_header_without_data_cells_flagged() {
        let mut table = node("t", "table", None, vec![]);
        let header = node("h1", "columnheader", Some("t"), vec![]);
        table.child_ids = vec!["h1".into()];
        let tree = AXTree::from_nodes(vec![table, header]);
        let r = check_table_extended(&tree);
        assert!(
            r.violations.iter().any(|v| v.rule_id.as_deref() == Some("th-has-data-cells")),
            "header without data cells should be flagged"
        );
    }

    #[test]
    fn test_dangling_headers_ref_flagged() {
        let mut table = node("t", "table", None, vec![]);
        let header = node("hdr", "columnheader", Some("t"), vec![]);
        let data = node("d1", "gridcell", Some("t"), vec![("headers", "nonexistent-id")]);
        table.child_ids = vec!["hdr".into(), "d1".into()];
        let tree = AXTree::from_nodes(vec![table, header, data]);
        let r = check_table_extended(&tree);
        assert!(
            r.violations.iter().any(|v| v.rule_id.as_deref() == Some("td-headers-attr")),
            "dangling headers reference should be flagged"
        );
    }
}
