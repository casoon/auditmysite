//! WCAG 1.4.11 Non-text Contrast
//!
//! The visual presentation of UI components and graphical objects have a
//! contrast ratio of at least 3:1 against adjacent colors.
//! Level AA
//!
//! Note: Full contrast checking for UI components requires CSS inspection via CDP.
//! This rule checks for common patterns via the AX tree: form controls and
//! interactive elements that may lack sufficient visual distinction.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const NON_TEXT_CONTRAST_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.11",
    name: "Non-text Contrast",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "UI components and graphical objects have a contrast ratio of at least 3:1",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html",
};

pub fn check_non_text_contrast(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.form_controls() {
        if node.ignored {
            continue;
        }

        let role = node.role.as_deref().unwrap_or("");

        // Check for custom controls that may lack visual affordance
        // Custom checkboxes/radios/switches without proper state indicators
        if matches!(role, "checkbox" | "radio" | "switch") {
            // Check if the element has a checked/unchecked state exposed
            let has_checked_state = node.get_property_str("checked").is_some()
                || node.get_property_bool("checked").is_some();

            if !has_checked_state {
                let violation = Violation::new(
                    NON_TEXT_CONTRAST_RULE.id,
                    NON_TEXT_CONTRAST_RULE.name,
                    NON_TEXT_CONTRAST_RULE.level,
                    Severity::Medium,
                    format!(
                        "{} control has no checked state — visual state may not be distinguishable",
                        role
                    ),
                    node.node_id.clone(),
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Ensure custom controls have visible state indicators with at least 3:1 contrast ratio")
                .with_help_url(NON_TEXT_CONTRAST_RULE.help_url);

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

    fn checkbox_node(id: &str, has_checked: bool) -> AXNode {
        let mut properties = vec![];
        if has_checked {
            properties.push(AXProperty {
                name: "checked".to_string(),
                value: AXValue::String("false".to_string()),
            });
        }
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("checkbox".to_string()),
            name: Some("Accept terms".to_string()),
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
    fn test_checkbox_with_state() {
        let tree = AXTree::from_nodes(vec![checkbox_node("1", true)]);
        let results = check_non_text_contrast(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_checkbox_without_state() {
        let tree = AXTree::from_nodes(vec![checkbox_node("1", false)]);
        let results = check_non_text_contrast(&tree);
        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].rule, "1.4.11");
    }
}
