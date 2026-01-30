//! WCAG 3.3.2 Labels or Instructions
//!
//! Labels or instructions are provided when content requires user input.
//! Level A

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 3.3.2
pub const INSTRUCTIONS_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.2",
    name: "Labels or Instructions",
    level: WcagLevel::A,
    severity: Severity::Serious,
    description: "Labels or instructions are provided when content requires user input",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/labels-or-instructions.html",
};

/// Check for labels and instructions on form controls
pub fn check_instructions(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        results.nodes_checked += 1;
        let role_lower = node.role.as_deref().unwrap_or("").to_lowercase();

        // Check form inputs
        if is_form_input(&role_lower) {
            let has_label = has_accessible_label(node);
            let has_placeholder_text = has_placeholder(node);
            let has_instructions = has_instructions_or_hint(node);

            if !has_label {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Critical,
                    format!("Form control '{}' has no accessible label", role_lower),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Add a <label> element, aria-label, or aria-labelledby attribute")
                .with_help_url(INSTRUCTIONS_RULE.help_url);

                results.add_violation(violation);
                continue;
            }

            // Check if placeholder is used as the only label
            if !has_label && has_placeholder_text && !has_instructions {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Moderate,
                    "Placeholder used as only label",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Add a visible <label> element. Placeholder should supplement, not replace, labels")
                .with_help_url(INSTRUCTIONS_RULE.help_url);

                results.add_violation(violation);
            }

            // Check for required fields without indication
            if is_required(node) && !indicates_required(node) {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Moderate,
                    "Required field not clearly indicated",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Add visual indicator (e.g., asterisk *) and screen reader text for required fields")
                .with_help_url(INSTRUCTIONS_RULE.help_url);

                results.add_violation(violation);
            }

            // Check for inputs with format requirements
            if needs_format_instructions(&role_lower, node) && !has_format_hint(node) {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Minor,
                    format!("Input '{}' may require format instructions", role_lower),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Consider adding format instructions (e.g., 'DD/MM/YYYY' for dates)")
                .with_help_url(INSTRUCTIONS_RULE.help_url);

                results.add_violation(violation);
            }

            // If no violations found for this input, count as pass
            if has_label {
                results.passes += 1;
            }
        }

        // Check fieldsets without legends
        if role_lower == "group" || role_lower == "radiogroup" {
            if !has_group_label(node) {
                let violation = Violation::new(
                    INSTRUCTIONS_RULE.id,
                    INSTRUCTIONS_RULE.name,
                    INSTRUCTIONS_RULE.level,
                    Severity::Moderate,
                    "Form group has no legend or label",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix("Use <fieldset> with <legend>, or add aria-labelledby to the group")
                .with_help_url(INSTRUCTIONS_RULE.help_url);

                results.add_violation(violation);
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

/// Check if role is a form input
fn is_form_input(role: &str) -> bool {
    matches!(
        role,
        "textbox" | "searchbox" | "combobox" | "listbox" |
        "spinbutton" | "slider" | "checkbox" | "radio" |
        "switch" | "textarea"
    )
}

/// Check if node has an accessible label
fn has_accessible_label(node: &AXNode) -> bool {
    if let Some(name) = &node.name {
        if !name.trim().is_empty() {
            return true;
        }
    }
    false
}

/// Check if node has placeholder
fn has_placeholder(node: &AXNode) -> bool {
    node.get_property_str("placeholder")
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

/// Check if node has instructions or hint text
fn has_instructions_or_hint(node: &AXNode) -> bool {
    if let Some(desc) = &node.description {
        if !desc.trim().is_empty() {
            return true;
        }
    }
    false
}

/// Check if field is marked as required
fn is_required(node: &AXNode) -> bool {
    node.get_property_bool("required").unwrap_or(false)
}

/// Check if required status is indicated
fn indicates_required(node: &AXNode) -> bool {
    if let Some(name) = &node.name {
        let name_lower = name.to_lowercase();
        if name_lower.contains("required") || name_lower.contains("*") {
            return true;
        }
    }

    if let Some(desc) = &node.description {
        let desc_lower = desc.to_lowercase();
        if desc_lower.contains("required") {
            return true;
        }
    }

    false
}

/// Check if input type typically needs format instructions
fn needs_format_instructions(role: &str, node: &AXNode) -> bool {
    let name = node.name.as_deref().unwrap_or("").to_lowercase();

    let format_sensitive = [
        "date", "phone", "tel", "zip", "postal",
        "credit card", "ssn", "social security",
        "passport", "account", "routing"
    ];

    format_sensitive.iter().any(|&term| name.contains(term)) ||
    role == "spinbutton"
}

/// Check if format hint is provided
fn has_format_hint(node: &AXNode) -> bool {
    let format_patterns = ["format:", "example:", "e.g.", "(", "mm/dd", "yyyy"];

    if let Some(name) = &node.name {
        let name_lower = name.to_lowercase();
        if format_patterns.iter().any(|p| name_lower.contains(p)) {
            return true;
        }
    }

    if let Some(desc) = &node.description {
        let desc_lower = desc.to_lowercase();
        if format_patterns.iter().any(|p| desc_lower.contains(p)) {
            return true;
        }
    }

    has_placeholder(node)
}

/// Check if group has a label
fn has_group_label(node: &AXNode) -> bool {
    if let Some(name) = &node.name {
        if !name.trim().is_empty() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXProperty, AXValue};

    fn create_input(id: &str, role: &str, name: Option<&str>) -> AXNode {
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

    fn create_input_with_required(id: &str, role: &str, name: Option<&str>, required: bool) -> AXNode {
        let mut node = create_input(id, role, name);
        if required {
            node.properties.push(AXProperty {
                name: "required".to_string(),
                value: AXValue::Bool(true),
            });
        }
        node
    }

    #[test]
    fn test_instructions_rule_metadata() {
        assert_eq!(INSTRUCTIONS_RULE.id, "3.3.2");
        assert_eq!(INSTRUCTIONS_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_is_form_input() {
        assert!(is_form_input("textbox"));
        assert!(is_form_input("checkbox"));
        assert!(is_form_input("combobox"));
        assert!(!is_form_input("button"));
        assert!(!is_form_input("link"));
    }

    #[test]
    fn test_input_without_label() {
        let tree = AXTree::from_nodes(vec![create_input("1", "textbox", None)]);
        let results = check_instructions(&tree);
        assert!(results.violations.iter().any(|v| v.message.contains("no accessible label")));
    }

    #[test]
    fn test_input_with_label() {
        let tree = AXTree::from_nodes(vec![create_input("1", "textbox", Some("Email address"))]);
        let results = check_instructions(&tree);
        assert!(!results.violations.iter().any(|v| v.message.contains("no accessible label")));
    }

    #[test]
    fn test_required_without_indication() {
        let tree = AXTree::from_nodes(vec![
            create_input_with_required("1", "textbox", Some("Name"), true)
        ]);
        let results = check_instructions(&tree);
        assert!(results.violations.iter().any(|v| v.message.contains("Required field")));
    }

    #[test]
    fn test_required_with_indication() {
        let tree = AXTree::from_nodes(vec![
            create_input_with_required("1", "textbox", Some("Name (required)"), true)
        ]);
        let results = check_instructions(&tree);
        assert!(!results.violations.iter().any(|v| v.message.contains("Required field not clearly indicated")));
    }
}
