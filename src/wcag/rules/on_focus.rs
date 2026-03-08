//! WCAG 3.2.1 On Focus
//!
//! When any user interface component receives focus, it does not initiate
//! a change of context.
//! Level A
//!
//! Note: Full on-focus testing requires behavioral analysis via CDP.
//! This rule checks for common patterns in the AX tree that may indicate
//! focus-triggered context changes.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const ON_FOCUS_RULE: RuleMetadata = RuleMetadata {
    id: "3.2.1",
    name: "On Focus",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Receiving focus does not initiate a change of context",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/on-focus.html",
};

pub fn check_on_focus(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        let role = node.role.as_deref().unwrap_or("");

        // Check for onfocus handlers on non-interactive elements
        // In AX tree, this may appear as an event handler property
        let has_focus_handler = node.get_property_str("onfocus").is_some();

        if has_focus_handler && !node.is_interactive() {
            let violation = Violation::new(
                ON_FOCUS_RULE.id,
                ON_FOCUS_RULE.name,
                ON_FOCUS_RULE.level,
                Severity::High,
                format!(
                    "Non-interactive {} element has onfocus handler which may cause context change",
                    role
                ),
                node.node_id.clone(),
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Remove onfocus handlers that cause context changes, or use interactive elements",
            )
            .with_help_url(ON_FOCUS_RULE.help_url);

            results.add_violation(violation);
        }

        // Check for autofocus on elements other than the first form field
        // Autofocus can unexpectedly move focus context
        if node.get_property_bool("autofocus") == Some(true) {
            // Autofocus is acceptable on the first form field but problematic elsewhere
            // We flag it as a warning for manual review
            if !matches!(role, "textbox" | "searchbox" | "combobox") {
                let violation = Violation::new(
                    ON_FOCUS_RULE.id,
                    ON_FOCUS_RULE.name,
                    ON_FOCUS_RULE.level,
                    Severity::Medium,
                    format!(
                        "{} element has autofocus which may cause unexpected context change",
                        role
                    ),
                    node.node_id.clone(),
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Avoid autofocus on non-input elements as it can disorient users")
                .with_help_url(ON_FOCUS_RULE.help_url);

                results.add_violation(violation);
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_autofocus(id: &str, role: &str) -> AXNode {
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
                name: "autofocus".to_string(),
                value: AXValue::Bool(true),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_no_autofocus() {
        let tree = AXTree::from_nodes(vec![AXNode {
            node_id: "1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("button".to_string()),
            name: Some("Submit".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }]);
        let results = check_on_focus(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_autofocus_on_textbox_ok() {
        let tree = AXTree::from_nodes(vec![node_with_autofocus("1", "textbox")]);
        let results = check_on_focus(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_autofocus_on_button() {
        let tree = AXTree::from_nodes(vec![node_with_autofocus("1", "button")]);
        let results = check_on_focus(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("autofocus"));
    }
}
