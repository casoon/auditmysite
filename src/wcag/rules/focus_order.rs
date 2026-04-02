//! WCAG 2.4.3 Focus Order
//!
//! If a Web page can be navigated sequentially and the navigation sequences
//! affect meaning or operation, focusable components receive focus in an order
//! that preserves meaning and operability.
//! Level A

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const FOCUS_ORDER_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.3",
    name: "Focus Order",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Focusable components receive focus in an order that preserves meaning",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/focus-order.html",
    axe_id: "focus-order-semantics",
    tags: &["wcag2a", "wcag243", "cat.keyboard"],
};

pub fn check_focus_order(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    let mut positive_tabindexes: Vec<(i64, String, Option<String>)> = Vec::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        // Detect positive tabindex values which disrupt natural focus order
        if let Some(tabindex) = node.get_property_int("tabindex") {
            if tabindex > 0 {
                positive_tabindexes.push((tabindex, node.node_id.clone(), node.role.clone()));
            }
        }
    }

    // Flag all elements with positive tabindex
    for (tabindex, node_id, role) in &positive_tabindexes {
        let violation = Violation::new(
            FOCUS_ORDER_RULE.id,
            FOCUS_ORDER_RULE.name,
            FOCUS_ORDER_RULE.level,
            Severity::High,
            format!(
                "Element has positive tabindex={} which disrupts natural focus order",
                tabindex
            ),
            node_id.clone(),
        )
        .with_role(role.clone())
        .with_fix("Remove positive tabindex values. Use tabindex=\"0\" for natural order or tabindex=\"-1\" for programmatic focus only")
        .with_help_url(FOCUS_ORDER_RULE.help_url);

        results.add_violation(violation);
    }

    // Check for focusable elements inside aria-hidden containers
    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        let is_aria_hidden = node
            .get_property_str("aria-hidden")
            .is_some_and(|v| v == "true");

        if is_aria_hidden && node.is_focusable() {
            let violation = Violation::new(
                FOCUS_ORDER_RULE.id,
                FOCUS_ORDER_RULE.name,
                FOCUS_ORDER_RULE.level,
                Severity::Critical,
                "Focusable element inside aria-hidden context",
                node.node_id.clone(),
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Either remove aria-hidden or make the element not focusable with tabindex=\"-1\"",
            )
            .with_help_url(FOCUS_ORDER_RULE.help_url);

            results.add_violation(violation);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_tabindex(id: &str, role: &str, tabindex: i64) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some("Test".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "tabindex".to_string(),
                value: AXValue::Int(tabindex),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_no_positive_tabindex() {
        let tree = AXTree::from_nodes(vec![
            node_with_tabindex("1", "button", 0),
            node_with_tabindex("2", "link", 0),
        ]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_positive_tabindex_violation() {
        let tree = AXTree::from_nodes(vec![
            node_with_tabindex("1", "button", 5),
            node_with_tabindex("2", "link", 3),
        ]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 2);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("tabindex=5")));
    }

    #[test]
    fn test_negative_tabindex_ok() {
        let tree = AXTree::from_nodes(vec![node_with_tabindex("1", "div", -1)]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
