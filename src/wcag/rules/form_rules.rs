//! WCAG 1.3.1, 3.3.1, 3.3.2 - Form Rules
//!
//! Checks grouped form controls for fieldset/legend, required field indication,
//! and error descriptions for invalid fields.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for form structure rules (1.3.1)
pub const RULE_META_STRUCTURE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships - Forms",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Grouped form controls must have a fieldset/legend; form structure must be programmatically determinable",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "form-field-multiple-labels",
    tags: &["wcag2a", "wcag131", "cat.forms"],
};

/// Rule metadata for error identification rules (3.3.1)
pub const RULE_META_ERROR: RuleMetadata = RuleMetadata {
    id: "3.3.1",
    name: "Error Identification",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "If an input error is automatically detected, the item that is in error must be identified and the error described in text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/error-identification.html",
    axe_id: "input-error-message",
    tags: &["wcag2a", "wcag331", "cat.forms"],
};

/// Rule metadata for labels/instructions (3.3.2)
pub const RULE_META_LABELS: RuleMetadata = RuleMetadata {
    id: "3.3.2",
    name: "Labels or Instructions",
    level: WcagLevel::A,
    severity: Severity::Low,
    description: "Labels or instructions are provided when content requires user input",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/labels-or-instructions.html",
    axe_id: "label",
    tags: &["wcag2a", "wcag332", "cat.forms"],
};

/// Run all form-related WCAG checks
pub fn check_form_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    check_grouped_controls(tree, &mut results);
    check_required_field_indication(tree, &mut results);
    check_invalid_field_description(tree, &mut results);

    results
}

/// Check that grouped radio/checkbox controls have a group ancestor
fn check_grouped_controls(tree: &AXTree, results: &mut WcagResults) {
    let grouped_roles = ["radio", "checkbox"];

    let grouped_nodes: Vec<&AXNode> = tree
        .nodes
        .values()
        .filter(|n| {
            !n.ignored
                && n.role
                    .as_deref()
                    .map(|r| grouped_roles.contains(&r))
                    .unwrap_or(false)
        })
        .collect();

    results.nodes_checked += grouped_nodes.len();

    if grouped_nodes.len() < 2 {
        // Single radio/checkbox doesn't require a group
        if grouped_nodes.len() == 1 {
            results.passes += 1;
        }
        return;
    }

    for node in &grouped_nodes {
        let has_group = has_ancestor_with_role(node, &["group", "radiogroup"], tree);
        if !has_group {
            let violation = Violation::new(
                RULE_META_STRUCTURE.id,
                RULE_META_STRUCTURE.name,
                RULE_META_STRUCTURE.level,
                Severity::Medium,
                "Grouped form controls may be missing a fieldset/legend",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Wrap related radio buttons or checkboxes in a <fieldset> with a <legend>, or use role=\"group\" with aria-labelledby",
            )
            .with_help_url(RULE_META_STRUCTURE.help_url);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }
}

/// Check that required fields have some indication of being required in their label
fn check_required_field_indication(tree: &AXTree, results: &mut WcagResults) {
    let input_roles = ["textbox", "combobox", "spinbutton", "listbox"];

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        let role = match node.role.as_deref() {
            Some(r) if input_roles.contains(&r) => r,
            _ => continue,
        };

        results.nodes_checked += 1;

        if !node.is_required() {
            results.passes += 1;
            continue;
        }

        let name_lower = node.name.as_deref().unwrap_or("").to_lowercase();
        let has_indication = name_lower.contains('*')
            || name_lower.contains("required")
            || name_lower.contains("pflicht")
            || name_lower.contains("obligatoire");

        if !has_indication {
            let violation = Violation::new(
                RULE_META_LABELS.id,
                RULE_META_LABELS.name,
                RULE_META_LABELS.level,
                Severity::Low,
                format!(
                    "Required {} field may not indicate required status in its label",
                    role
                ),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Include an asterisk (*) or the word \"required\" in the label, or use aria-required and explain the convention",
            )
            .with_help_url(RULE_META_LABELS.help_url);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }
}

/// Check that invalid fields have an accessible error description
fn check_invalid_field_description(tree: &AXTree, results: &mut WcagResults) {
    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        if node.role.is_none() {
            continue;
        }

        results.nodes_checked += 1;

        if !node.is_invalid() {
            results.passes += 1;
            continue;
        }

        let has_description = node
            .description
            .as_ref()
            .is_some_and(|d| !d.trim().is_empty());

        if !has_description {
            let violation = Violation::new(
                RULE_META_ERROR.id,
                RULE_META_ERROR.name,
                RULE_META_ERROR.level,
                Severity::Medium,
                "Invalid field has no accessible error description",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Add aria-describedby pointing to an element containing the error message, or use aria-errormessage",
            )
            .with_help_url(RULE_META_ERROR.help_url);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }
}

/// Walk up the ancestor chain to check whether any ancestor has one of the given roles
fn has_ancestor_with_role(node: &AXNode, roles: &[&str], tree: &AXTree) -> bool {
    let mut current_parent_id = node.parent_id.as_deref();

    while let Some(parent_id) = current_parent_id {
        if let Some(parent) = tree.get_node(parent_id) {
            if let Some(parent_role) = parent.role.as_deref() {
                if roles.contains(&parent_role) {
                    return true;
                }
            }
            current_parent_id = parent.parent_id.as_deref();
        } else {
            break;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn make_node(id: &str, role: &str, name: Option<&str>, parent_id: Option<&str>) -> AXNode {
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
            parent_id: parent_id.map(String::from),
            backend_dom_node_id: None,
        }
    }

    fn make_node_required(id: &str, role: &str, name: Option<&str>) -> AXNode {
        let mut node = make_node(id, role, name, None);
        node.properties.push(AXProperty {
            name: "required".to_string(),
            value: AXValue::Bool(true),
        });
        node
    }

    fn make_node_invalid(id: &str, role: &str) -> AXNode {
        let mut node = make_node(id, role, Some("Field"), None);
        node.properties.push(AXProperty {
            name: "invalid".to_string(),
            value: AXValue::Bool(true),
        });
        node
    }

    #[test]
    fn test_radio_without_group_flagged() {
        let nodes = vec![
            make_node("1", "radio", Some("Option A"), None),
            make_node("2", "radio", Some("Option B"), None),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("fieldset/legend")));
    }

    #[test]
    fn test_radio_with_group_passes() {
        let nodes = vec![
            make_node("g", "group", Some("Options"), None),
            make_node("1", "radio", Some("Option A"), Some("g")),
            make_node("2", "radio", Some("Option B"), Some("g")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("fieldset/legend")));
    }

    #[test]
    fn test_required_field_without_indication_flagged() {
        let nodes = vec![make_node_required("1", "textbox", Some("Email"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("required status")));
    }

    #[test]
    fn test_required_field_with_asterisk_passes() {
        let nodes = vec![make_node_required("1", "textbox", Some("Email *"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("required status")));
    }

    #[test]
    fn test_invalid_field_without_description_flagged() {
        let nodes = vec![make_node_invalid("1", "textbox")];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible error description")));
    }

    #[test]
    fn test_invalid_field_with_description_passes() {
        let mut node = make_node_invalid("1", "textbox");
        node.description = Some("Please enter a valid email address".to_string());
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_form_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible error description")));
    }
}
