//! WCAG 4.1.2 - Name, Role, Value
//!
//! For all user interface components, the name and role can be programmatically determined.
//! This rule checks that form controls and interactive elements have accessible names.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 4.1.2
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value",
    level: WcagLevel::A,
    severity: Severity::Serious,
    description: "For all user interface components, the name and role can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
};

/// Check that form controls and interactive elements have accessible names
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for elements missing accessible names
pub fn check_labels(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Check form controls
    let form_controls = tree.form_controls();
    results.nodes_checked += form_controls.len();

    for control in form_controls {
        if control.ignored {
            continue;
        }

        check_form_control(control, &mut results);
    }

    // Check links
    let links = tree.links();
    results.nodes_checked += links.len();

    for link in links {
        if link.ignored {
            continue;
        }

        check_link(link, &mut results);
    }

    // Check buttons
    let buttons = tree.nodes_with_role("button");
    results.nodes_checked += buttons.len();

    for button in buttons {
        if button.ignored {
            continue;
        }

        check_button(button, &mut results);
    }

    results
}

/// Check a single form control
fn check_form_control(
    node: &crate::accessibility::AXNode,
    results: &mut WcagResults,
) {
    let role = node.role.as_deref().unwrap_or("unknown");

    // Check for accessible name
    if !node.has_name() {
        let message = match role {
            "textbox" => "Text input field is missing a label",
            "checkbox" => "Checkbox is missing a label",
            "radio" => "Radio button is missing a label",
            "combobox" => "Dropdown/select is missing a label",
            "listbox" => "List box is missing a label",
            "spinbutton" => "Spin button is missing a label",
            "slider" => "Slider is missing a label",
            "searchbox" => "Search field is missing a label",
            _ => "Form control is missing a label",
        };

        let fix = match role {
            "textbox" | "searchbox" => {
                "Add a <label> element with 'for' attribute, or use aria-label/aria-labelledby"
            }
            "checkbox" | "radio" => {
                "Wrap in a <label> element, or use aria-label"
            }
            _ => "Add aria-label or aria-labelledby attribute",
        };

        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            RULE_META.severity,
            message,
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix(fix)
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Check a link element
fn check_link(node: &crate::accessibility::AXNode, results: &mut WcagResults) {
    if !node.has_name() {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Serious,
            "Link is missing accessible text",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Add text content inside the link, or use aria-label")
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        // Check for generic link text
        let name = node.name.as_ref().unwrap().to_lowercase();
        let generic_texts = [
            "click here",
            "here",
            "read more",
            "more",
            "learn more",
            "link",
            "click",
        ];

        if generic_texts.iter().any(|&t| name == t) {
            let violation = Violation::new(
                "2.4.4", // Link Purpose (In Context)
                "Link Purpose",
                WcagLevel::A,
                Severity::Moderate,
                format!("Link text '{}' is not descriptive", name),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Use descriptive link text that explains the destination")
            .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context.html");

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }
}

/// Check a button element
fn check_button(
    node: &crate::accessibility::AXNode,
    results: &mut WcagResults,
) {
    if !node.has_name() {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Serious,
            "Button is missing accessible text",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Add text content inside the button, or use aria-label")
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

    fn create_control_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_labeled_textbox() {
        let nodes = vec![create_control_node("1", "textbox", Some("Email Address"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_labels(&tree);

        assert_eq!(results.violations.len(), 0);
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_unlabeled_textbox() {
        let nodes = vec![create_control_node("1", "textbox", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_labels(&tree);

        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].rule, "4.1.2");
        assert!(results.violations[0].message.contains("label"));
    }

    #[test]
    fn test_button_without_label() {
        let nodes = vec![create_control_node("1", "button", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_labels(&tree);

        assert_eq!(results.violations.len(), 1);
    }

    #[test]
    fn test_link_with_generic_text() {
        let nodes = vec![create_control_node("1", "link", Some("click here"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_labels(&tree);

        // Should flag generic link text
        assert!(results.violations.iter().any(|v| v.rule == "2.4.4"));
    }

    #[test]
    fn test_link_without_name() {
        let nodes = vec![create_control_node("1", "link", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_labels(&tree);

        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("Link"));
    }
}
