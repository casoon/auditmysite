//! WCAG 3.2.2 On Input
//!
//! Changing the setting of any user interface component does not automatically
//! cause a change of context unless the user has been advised of the behavior
//! before using the component.
//! Level A
//!
//! Note: Full on-input testing requires behavioral analysis via CDP.
//! This rule checks for common patterns: select elements and radio buttons
//! that may trigger form submission or navigation without explicit submit.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const ON_INPUT_RULE: RuleMetadata = RuleMetadata {
    id: "3.2.2",
    name: "On Input",
    level: WcagLevel::A,
    severity: Severity::Moderate,
    description: "Changing a setting does not automatically cause a change of context",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/on-input.html",
};

pub fn check_on_input(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Track if the page has forms with submit buttons
    let has_submit_button = tree.iter().any(|n| {
        let role = n.role.as_deref().unwrap_or("");
        let name_lower = n.name.as_deref().unwrap_or("").to_lowercase();
        role == "button"
            && (name_lower.contains("submit")
                || name_lower.contains("send")
                || name_lower.contains("go")
                || name_lower.contains("search")
                || name_lower.contains("absenden"))
    });

    for node in tree.form_controls() {
        if node.ignored {
            continue;
        }

        let role = node.role.as_deref().unwrap_or("");

        // Check for onchange handlers on select elements
        let has_change_handler = node.get_property_str("onchange").is_some();

        if has_change_handler && matches!(role, "combobox" | "listbox") {
            let violation = Violation::new(
                ON_INPUT_RULE.id,
                ON_INPUT_RULE.name,
                ON_INPUT_RULE.level,
                Severity::Moderate,
                format!(
                    "{} element has onchange handler — may cause unexpected context change",
                    role
                ),
                node.node_id.clone(),
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Use a submit button instead of auto-submitting on selection change")
            .with_help_url(ON_INPUT_RULE.help_url);

            results.add_violation(violation);
        }

        // Check for select/radio elements in forms without submit buttons
        if matches!(role, "combobox" | "listbox" | "radio") && !has_submit_button {
            // This is a heuristic: forms without submit buttons may auto-submit on change
            // Only flag if the control appears to be in a navigation/filter context
            let name_lower = node.name.as_deref().unwrap_or("").to_lowercase();
            let navigation_hints = [
                "sort", "filter", "language", "country", "region", "navigate", "redirect", "go to",
            ];

            if navigation_hints.iter().any(|h| name_lower.contains(h)) {
                let violation = Violation::new(
                    ON_INPUT_RULE.id,
                    ON_INPUT_RULE.name,
                    ON_INPUT_RULE.level,
                    Severity::Minor,
                    format!(
                        "{} '{}' may trigger navigation on change without explicit submit",
                        role, name_lower
                    ),
                    node.node_id.clone(),
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Add a submit button or notify users that selection will change context")
                .with_help_url(ON_INPUT_RULE.help_url);

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

    fn select_node(id: &str, name: &str, has_onchange: bool) -> AXNode {
        let mut properties = vec![];
        if has_onchange {
            properties.push(AXProperty {
                name: "onchange".to_string(),
                value: AXValue::String("submit()".to_string()),
            });
        }
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("combobox".to_string()),
            name: Some(name.to_string()),
            name_source: None,
            description: None,
            value: None,
            properties,
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_select_without_onchange() {
        let tree = AXTree::from_nodes(vec![select_node("1", "Category", false)]);
        let results = check_on_input(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_select_with_onchange() {
        let tree = AXTree::from_nodes(vec![select_node("1", "Category", true)]);
        let results = check_on_input(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("onchange"));
    }

    #[test]
    fn test_navigation_select_without_submit() {
        let tree = AXTree::from_nodes(vec![select_node("1", "Sort by", false)]);
        let results = check_on_input(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("sort by"));
    }
}
