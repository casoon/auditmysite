//! Extended table rules — granular header/data cell relationships
//!
//! Complements `table_rules.rs` with two axe-core rules that check header-to-data
//! cell associations:
//!
//! - `td-headers-attr`:   cells that reference non-existent or empty header IDs
//! - `th-has-data-cells`: header cells that have no corresponding data cells
//!
//! `td-headers-attr` is a DOM-level rule (see `check_table_headers_attr_with_page`
//! below): the AX tree exposes no `headers` property (an earlier tree-based
//! implementation matched it against `aria-owns` instead, which is a
//! different relationship attribute entirely, and never fired correctly in
//! production — #QA-030). `th-has-data-cells` is a genuine tree-structure
//! question (do header/data cells coexist under one table) and stays
//! AX-tree-based.

use chromiumoxide::Page;

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

const TD_HEADERS_ATTR_CAP: usize = 250;

const TD_HEADERS_ATTR_BODY: &str = r#"
  var issues = [];
  var cells = document.querySelectorAll('[headers]');
  for (var i = 0; i < cells.length && issues.length < CAP; i++) {
    var cell = cells[i];
    var table = cell.closest('table, [role="grid"], [role="table"], [role="treegrid"]');
    var refs = (cell.getAttribute('headers') || '').trim().split(/\s+/).filter(Boolean);

    if (refs.length === 0) {
      issues.push({ selector: __amsCssSelector(cell), kind: 'empty' });
      continue;
    }

    var invalidRef = null;
    for (var r = 0; r < refs.length; r++) {
      var ref = document.getElementById(refs[r]);
      var tag = ref ? ref.tagName : null;
      var role = ref ? (ref.getAttribute('role') || '').toLowerCase() : '';
      var isHeaderCell = tag === 'TH' || role === 'columnheader' || role === 'rowheader';
      if (!ref || (table && !table.contains(ref)) || !isHeaderCell) {
        invalidRef = refs[r];
        break;
      }
    }
    if (invalidRef) {
      issues.push({ selector: __amsCssSelector(cell), kind: 'invalid', ref: invalidRef });
    }
  }
  return { issues: issues };
"#;

/// Check that cells using the `headers` attribute only reference valid
/// header cells (`<th>` or role=columnheader/rowheader) within the same table.
pub async fn check_table_headers_attr_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &TD_HEADERS_ATTR_BODY.replace("CAP", &TD_HEADERS_ATTR_CAP.to_string()),
        "})()",
    ]
    .concat();

    let val = match crate::wcag::types::evaluate_or_fail_for(
        page,
        "td-headers-attr",
        crate::cli::WcagLevel::A,
        js.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let kind = issue.get("kind")?.as_str()?;

            let (message, fix) = if kind == "empty" {
                (
                    "Table cell has an empty headers attribute".to_string(),
                    "Remove the headers attribute or reference valid th/header cell IDs",
                )
            } else {
                let r#ref = issue.get("ref").and_then(|v| v.as_str()).unwrap_or("");
                (
                    format!(
                        "Table cell references header ID \"{}\" which does not exist in this table",
                        r#ref
                    ),
                    "Ensure the headers attribute only references IDs of <th> elements in the same table",
                )
            };

            Some(
                Violation::new(
                    RULE_TD_HEADERS_ATTR.id,
                    RULE_TD_HEADERS_ATTR.name,
                    RULE_TD_HEADERS_ATTR.level,
                    RULE_TD_HEADERS_ATTR.severity,
                    message,
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix(fix)
                .with_rule_id(RULE_TD_HEADERS_ATTR.axe_id)
                .with_help_url(RULE_TD_HEADERS_ATTR.help_url),
            )
        })
        .collect()
}

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
    for node in tree.iter() {
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

        // td-headers-attr now runs as a DOM page rule
        // (check_table_headers_attr_with_page) — see module docs.

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
            r.violations
                .iter()
                .any(|v| v.rule_id.as_deref() == Some("th-has-data-cells")),
            "header without data cells should be flagged"
        );
    }

    // td-headers-attr moved to the DOM-based check_table_headers_attr_with_page
    // (#QA-030 — `headers` is not an AX property) and needs a live Page, so
    // it isn't unit-tested here; covered by live verification instead.
}
