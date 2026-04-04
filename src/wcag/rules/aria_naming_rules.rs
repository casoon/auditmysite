//! WCAG 4.1.2 — Role-specific accessible name checks
//!
//! axe-core equivalent rules for element types that require their own name
//! but are not covered (or covered under a different axe_id) by the general
//! `aria-label` check in `accessible_name.rs`:
//!
//! - `aria-command-name`:      link / button / menuitem without a name
//! - `aria-input-field-name`:  input roles (textbox, combobox, …) without a name
//! - `aria-meter-name`:        meter role without a name
//! - `aria-progressbar-name`:  progressbar role without a name
//! - `aria-toggle-field-name`: checkbox / radio / switch without a name
//! - `aria-treeitem-name`:     treeitem without a name

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

// ── Rule metadata ──────────────────────────────────────────────────────────────

pub const RULE_COMMAND_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Command Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Interactive command elements (link, button, menuitem) must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-command-name",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

pub const RULE_INPUT_FIELD_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Input Field Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Input-type roles must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-input-field-name",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

pub const RULE_METER_NAME: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Meter Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Elements with role=meter must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "aria-meter-name",
    tags: &["wcag2a", "wcag111", "cat.aria"],
};

pub const RULE_PROGRESSBAR_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Progressbar Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Elements with role=progressbar must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-progressbar-name",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

pub const RULE_TOGGLE_FIELD_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Toggle Field Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Toggle elements (checkbox, radio, switch) must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-toggle-field-name",
    tags: &["wcag2a", "wcag412", "cat.forms"],
};

pub const RULE_TREEITEM_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Treeitem Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Elements with role=treeitem must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-treeitem-name",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

// ── Role sets ─────────────────────────────────────────────────────────────────

const COMMAND_ROLES: &[&str] = &["link", "button", "menuitem", "menuitemcheckbox", "menuitemradio"];
const INPUT_ROLES: &[&str] = &["textbox", "combobox", "listbox", "searchbox", "spinbutton"];
const TOGGLE_ROLES: &[&str] = &["checkbox", "radio", "switch"];

// ── Public check function ──────────────────────────────────────────────────────

/// Run all role-specific accessible name checks.
pub fn check_aria_naming_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r.to_lowercase(),
            None => continue,
        };
        let role = role.as_str();

        // ── aria-command-name ──────────────────────────────────────────────
        if COMMAND_ROLES.contains(&role) {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_COMMAND_NAME.id,
                        RULE_COMMAND_NAME.name,
                        RULE_COMMAND_NAME.level,
                        RULE_COMMAND_NAME.severity,
                        format!("Element with role=\"{}\" has no accessible name", role),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix(format!(
                        "Add an aria-label, aria-labelledby, or visible text to the {} element",
                        role
                    ))
                    .with_rule_id(RULE_COMMAND_NAME.axe_id).with_help_url(RULE_COMMAND_NAME.help_url),
                );
            } else {
                results.passes += 1;
            }
        }

        // ── aria-input-field-name ──────────────────────────────────────────
        if INPUT_ROLES.contains(&role) {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_INPUT_FIELD_NAME.id,
                        RULE_INPUT_FIELD_NAME.name,
                        RULE_INPUT_FIELD_NAME.level,
                        RULE_INPUT_FIELD_NAME.severity,
                        format!("Input field with role=\"{}\" has no accessible name", role),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Associate a <label> element or add aria-label / aria-labelledby")
                    .with_rule_id(RULE_INPUT_FIELD_NAME.axe_id).with_help_url(RULE_INPUT_FIELD_NAME.help_url),
                );
            } else {
                results.passes += 1;
            }
        }

        // ── aria-meter-name ────────────────────────────────────────────────
        if role == "meter" {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_METER_NAME.id,
                        RULE_METER_NAME.name,
                        RULE_METER_NAME.level,
                        RULE_METER_NAME.severity,
                        "Element with role=\"meter\" has no accessible name",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add aria-label or aria-labelledby to describe what the meter measures")
                    .with_rule_id(RULE_METER_NAME.axe_id).with_help_url(RULE_METER_NAME.help_url),
                );
            } else {
                results.passes += 1;
            }
        }

        // ── aria-progressbar-name ──────────────────────────────────────────
        if role == "progressbar" {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_PROGRESSBAR_NAME.id,
                        RULE_PROGRESSBAR_NAME.name,
                        RULE_PROGRESSBAR_NAME.level,
                        RULE_PROGRESSBAR_NAME.severity,
                        "Element with role=\"progressbar\" has no accessible name",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add aria-label or aria-labelledby to describe what is loading")
                    .with_rule_id(RULE_PROGRESSBAR_NAME.axe_id).with_help_url(RULE_PROGRESSBAR_NAME.help_url),
                );
            } else {
                results.passes += 1;
            }
        }

        // ── aria-toggle-field-name ─────────────────────────────────────────
        if TOGGLE_ROLES.contains(&role) {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_TOGGLE_FIELD_NAME.id,
                        RULE_TOGGLE_FIELD_NAME.name,
                        RULE_TOGGLE_FIELD_NAME.level,
                        RULE_TOGGLE_FIELD_NAME.severity,
                        format!("Toggle element with role=\"{}\" has no accessible name", role),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add a <label> element or aria-label / aria-labelledby")
                    .with_rule_id(RULE_TOGGLE_FIELD_NAME.axe_id).with_help_url(RULE_TOGGLE_FIELD_NAME.help_url),
                );
            } else {
                results.passes += 1;
            }
        }

        // ── aria-treeitem-name ─────────────────────────────────────────────
        if role == "treeitem" {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_TREEITEM_NAME.id,
                        RULE_TREEITEM_NAME.name,
                        RULE_TREEITEM_NAME.level,
                        RULE_TREEITEM_NAME.severity,
                        "Element with role=\"treeitem\" has no accessible name",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add visible text content or aria-label to the treeitem")
                    .with_rule_id(RULE_TREEITEM_NAME.axe_id).with_help_url(RULE_TREEITEM_NAME.help_url),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
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
    fn test_button_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node("1", "button", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("aria-command-name")));
    }

    #[test]
    fn test_button_with_name_passes() {
        let tree = AXTree::from_nodes(vec![node("1", "button", Some("Submit"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_textbox_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node("1", "textbox", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("aria-input-field-name")));
    }

    #[test]
    fn test_meter_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node("1", "meter", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("aria-meter-name")));
    }

    #[test]
    fn test_progressbar_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node("1", "progressbar", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("aria-progressbar-name")));
    }

    #[test]
    fn test_checkbox_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node("1", "checkbox", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("aria-toggle-field-name")));
    }

    #[test]
    fn test_treeitem_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node("1", "treeitem", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("aria-treeitem-name")));
    }

    #[test]
    fn test_named_elements_all_pass() {
        let tree = AXTree::from_nodes(vec![
            node("1", "button", Some("Close")),
            node("2", "textbox", Some("Search")),
            node("3", "meter", Some("Disk usage")),
            node("4", "progressbar", Some("Loading")),
            node("5", "checkbox", Some("Accept terms")),
            node("6", "treeitem", Some("Documents")),
        ]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.is_empty());
    }
}
