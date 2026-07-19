//! WCAG 4.1.2, 2.1.1 - Widget Rules
//!
//! Checks complex ARIA widget patterns: tabs, comboboxes, sliders, tree items.

use chromiumoxide::Page;

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
            // "tab" selected-state validation moved to
            // check_tab_selected_state_with_page (DOM-based) — Chrome
            // synthesizes a default `selected: false` AX property for tabs
            // even when the author never set `aria-selected`, so AX-tree
            // presence-checking can never observe the missing-attribute case
            // (#QA-031, confirmed via live fixture).
            "combobox" => check_combobox_has_options(node, tree, &mut results),
            "slider" => check_slider_has_value(node, &mut results),
            // "treeitem" required-ancestor validation lives exclusively in
            // aria_required_parent.rs — this file used to duplicate it via
            // check_treeitem_in_tree (#QA-009 cleanup, same class as #QA-033).
            _ => {}
        }
    }

    results
}

/// Custom, stable identifiers (not real axe-core rule ids) so these checks get
/// their own taxonomy entry instead of collapsing into the shared 4.1.2 bucket.
const TABLIST_TABPANEL_AXE_ID: &str = "aria-tablist-tabpanel";
const COMBOBOX_OPTIONS_AXE_ID: &str = "aria-combobox-options";
const TAB_SELECTED_STATE_AXE_ID: &str = "aria-tab-selected-state";

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
        .with_rule_id(TABLIST_TABPANEL_AXE_ID)
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/tabs/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

const TAB_SELECTED_STATE_CAP: usize = 250;

const TAB_SELECTED_STATE_BODY: &str = r#"
  var issues = [];
  var elems = document.querySelectorAll('[role="tab"]');
  for (var i = 0; i < elems.length && issues.length < CAP; i++) {
    var el = elems[i];
    if (!el.hasAttribute('aria-selected')) {
      issues.push({ selector: __amsCssSelector(el) });
    }
  }
  return { issues: issues };
"#;

/// Each tab must expose an `aria-selected` state. DOM-level: Chrome
/// synthesizes a default `selected: false` AX property for `role="tab"`
/// elements even when the author never set `aria-selected`, so AX-tree
/// presence-checking can't distinguish "not set" from "explicitly false"
/// (#QA-031, confirmed via live fixture — same class as
/// `check_checked_state_with_page`).
pub async fn check_tab_selected_state_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &TAB_SELECTED_STATE_BODY.replace("CAP", &TAB_SELECTED_STATE_CAP.to_string()),
        "})()",
    ]
    .concat();

    let val = match crate::wcag::types::evaluate_or_fail_for(
        page,
        "tab-selected-state",
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

            Some(
                Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    Severity::Low,
                    "Tab is missing selected state indication",
                    selector.clone(),
                )
                .with_selector(selector)
                .with_rule_id(TAB_SELECTED_STATE_AXE_ID)
                .with_fix(
                    "Add aria-selected=\"true\" or aria-selected=\"false\" to each tab element",
                )
                .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/tabs/"),
            )
        })
        .collect()
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
        .with_rule_id(COMBOBOX_OPTIONS_AXE_ID)
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/combobox/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// slider must have an accessible value via value property or aria-valuenow
fn check_slider_has_value(node: &AXNode, results: &mut WcagResults) {
    let has_value =
        node.value.as_ref().is_some_and(|v| !v.trim().is_empty()) || node.has_property("valuenow");

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

    // check_tab_selected_state_with_page (tab aria-selected) is DOM-based
    // and needs a live Page — not unit-tested here; covered by live
    // verification instead (see plans/quality-audit-backlog.md, #QA-031).

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
            name: "valuenow".to_string(),
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
