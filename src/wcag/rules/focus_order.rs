//! WCAG 2.4.3 Focus Order
//!
//! If a Web page can be navigated sequentially and the navigation sequences
//! affect meaning or operation, focusable components receive focus in an
//! order that preserves meaning and operability.
//! Level A
//!
//! Die Pruefung auf positives `tabindex` laeuft seit der Umstellung auf die
//! geteilten Crates als `keyboard/positive-tabindex` im `a11y-rules`-Bestand
//! (siehe `wcag::shared`) -- die vormalige DOM-Regel ist deshalb geloescht.
//! Sie las `tabindex` per JavaScript aus dem DOM, weil es keine AX-Eigenschaft
//! ist (#QA-030); die geteilte Regel liest dasselbe Attribut aus dem
//! CDP-Abzug.
//!
//! Hier bleibt die AX-baumbasierte Pruefung: fokussierbare Elemente innerhalb
//! von `aria-hidden`. `hidden` ist -- anders als `tabindex` -- eine echte
//! AX-Eigenschaft.

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

/// Check for focusable elements inside `aria-hidden` containers.
/// Tree-based: `hidden` is a real CDP AX property (unlike `tabindex`).
pub fn check_focus_order(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        let is_aria_hidden = node.get_property_bool("hidden").unwrap_or(false);

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
            .with_help_url(FOCUS_ORDER_RULE.help_url)
            .with_rule_id(FOCUS_ORDER_RULE.axe_id);

            results.add_violation(violation);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_hidden_and_focusable(
        id: &str,
        role: &str,
        hidden: bool,
        focusable: bool,
    ) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some("Test".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![
                AXProperty {
                    name: "hidden".to_string(),
                    value: AXValue::Bool(hidden),
                },
                AXProperty {
                    name: "focusable".to_string(),
                    value: AXValue::Bool(focusable),
                },
            ],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_focusable_in_hidden_context_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_hidden_and_focusable(
            "1", "button", true, true,
        )]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 1);
    }

    #[test]
    fn test_focusable_not_hidden_passes() {
        let tree = AXTree::from_nodes(vec![node_with_hidden_and_focusable(
            "1", "button", false, true,
        )]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_hidden_not_focusable_passes() {
        let tree = AXTree::from_nodes(vec![node_with_hidden_and_focusable(
            "1", "button", true, false,
        )]);
        let results = check_focus_order(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
