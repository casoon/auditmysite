//! WCAG 4.1.2 / 1.1.1 — Role-specific accessible name checks
//!
//! axe-core equivalent rules for element types that require their own name
//! but are not covered (or covered under a different axe_id) by the general
//! `aria-label` check in `accessible_name.rs`:
//!
//! - `aria-command-name`:      link / button / menuitem without a name
//! - `aria-input-field-name`:  input roles (textbox, combobox, …) without a name
//! - `aria-meter-name`:        meter role without a name
//! - `aria-progressbar-name`:  progressbar role without a name
//! - `aria-toggle-field-name`: checkbox / radio / switch / menu… without a name
//! - `aria-dialog-name`:       dialog / alertdialog without a name
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
    description:
        "Interactive command elements (link, button, menuitem) must have an accessible name",
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
    id: "1.1.1",
    name: "Progressbar Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Elements with role=progressbar must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "aria-progressbar-name",
    tags: &["wcag2a", "wcag111", "cat.aria"],
};

pub const RULE_TOGGLE_FIELD_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Toggle Field Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Toggle elements (checkbox, radio, switch, menu…) must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-toggle-field-name",
    tags: &["wcag2a", "wcag412", "cat.forms"],
};

pub const RULE_DIALOG_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Dialog Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Elements with role=dialog or alertdialog must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-dialog-name",
    tags: &["wcag2a", "wcag412", "cat.aria"],
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

const COMMAND_ROLES: &[&str] = &["button", "link", "menuitem"];
const INPUT_ROLES: &[&str] = &[
    "combobox",
    "listbox",
    "searchbox",
    "slider",
    "spinbutton",
    "textbox",
];
const TOGGLE_ROLES: &[&str] = &[
    "checkbox",
    "menu",
    "menuitemcheckbox",
    "menuitemradio",
    "radio",
    "radiogroup",
    "switch",
];
const DIALOG_ROLES: &[&str] = &["dialog", "alertdialog"];

// ── Public check function ──────────────────────────────────────────────────────

/// Run all role-specific accessible name checks.
pub fn check_aria_naming_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
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
                    .with_rule_id(RULE_COMMAND_NAME.axe_id)
                    .with_help_url(RULE_COMMAND_NAME.help_url),
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
                    .with_rule_id(RULE_INPUT_FIELD_NAME.axe_id)
                    .with_help_url(RULE_INPUT_FIELD_NAME.help_url),
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
                    .with_fix(
                        "Add aria-label or aria-labelledby to describe what the meter measures",
                    )
                    .with_rule_id(RULE_METER_NAME.axe_id)
                    .with_help_url(RULE_METER_NAME.help_url),
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
                    .with_rule_id(RULE_PROGRESSBAR_NAME.axe_id)
                    .with_help_url(RULE_PROGRESSBAR_NAME.help_url),
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
                        format!(
                            "Toggle element with role=\"{}\" has no accessible name",
                            role
                        ),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add a <label> element or aria-label / aria-labelledby")
                    .with_rule_id(RULE_TOGGLE_FIELD_NAME.axe_id)
                    .with_help_url(RULE_TOGGLE_FIELD_NAME.help_url),
                );
            } else {
                results.passes += 1;
            }
        }

        // ── aria-dialog-name ───────────────────────────────────────────────
        if DIALOG_ROLES.contains(&role) {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_DIALOG_NAME.id,
                        RULE_DIALOG_NAME.name,
                        RULE_DIALOG_NAME.level,
                        RULE_DIALOG_NAME.severity,
                        format!("Element with role=\"{}\" has no accessible name", role),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix(
                        "Add aria-label or aria-labelledby to give the dialog an accessible name",
                    )
                    .with_rule_id(RULE_DIALOG_NAME.axe_id)
                    .with_help_url(RULE_DIALOG_NAME.help_url),
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
                    .with_rule_id(RULE_TREEITEM_NAME.axe_id)
                    .with_help_url(RULE_TREEITEM_NAME.help_url),
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

    // ── aria-command-name ──────────────────────────────────────────────────

    #[test]
    fn command_name_violation_button() {
        let tree = AXTree::from_nodes(vec![node("1", "button", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-command-name")));
    }

    #[test]
    fn command_name_pass_button() {
        let tree = AXTree::from_nodes(vec![node("1", "button", Some("Submit"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-command-name")));
        assert!(r.passes >= 1);
    }

    #[test]
    fn command_name_violation_link() {
        let tree = AXTree::from_nodes(vec![node("1", "link", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-command-name")));
    }

    #[test]
    fn command_name_violation_menuitem() {
        let tree = AXTree::from_nodes(vec![node("1", "menuitem", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-command-name")));
    }

    // ── aria-input-field-name ─────────────────────────────────────────────

    #[test]
    fn input_field_name_violation_textbox() {
        let tree = AXTree::from_nodes(vec![node("1", "textbox", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-input-field-name")));
    }

    #[test]
    fn input_field_name_pass_textbox() {
        let tree = AXTree::from_nodes(vec![node("1", "textbox", Some("Search"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-input-field-name")));
        assert!(r.passes >= 1);
    }

    #[test]
    fn input_field_name_violation_slider() {
        let tree = AXTree::from_nodes(vec![node("1", "slider", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-input-field-name")));
    }

    // ── aria-meter-name ───────────────────────────────────────────────────

    #[test]
    fn meter_name_violation() {
        let tree = AXTree::from_nodes(vec![node("1", "meter", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-meter-name")));
    }

    #[test]
    fn meter_name_pass() {
        let tree = AXTree::from_nodes(vec![node("1", "meter", Some("Disk usage"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-meter-name")));
        assert!(r.passes >= 1);
    }

    // ── aria-progressbar-name ─────────────────────────────────────────────

    #[test]
    fn progressbar_name_violation() {
        let tree = AXTree::from_nodes(vec![node("1", "progressbar", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-progressbar-name")));
    }

    #[test]
    fn progressbar_name_pass() {
        let tree = AXTree::from_nodes(vec![node("1", "progressbar", Some("Loading"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-progressbar-name")));
        assert!(r.passes >= 1);
    }

    // ── aria-toggle-field-name ────────────────────────────────────────────

    #[test]
    fn toggle_field_name_violation_checkbox() {
        let tree = AXTree::from_nodes(vec![node("1", "checkbox", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-toggle-field-name")));
    }

    #[test]
    fn toggle_field_name_pass_checkbox() {
        let tree = AXTree::from_nodes(vec![node("1", "checkbox", Some("Accept terms"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-toggle-field-name")));
        assert!(r.passes >= 1);
    }

    #[test]
    fn toggle_field_name_violation_radiogroup() {
        let tree = AXTree::from_nodes(vec![node("1", "radiogroup", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-toggle-field-name")));
    }

    #[test]
    fn toggle_field_name_violation_menuitemcheckbox() {
        let tree = AXTree::from_nodes(vec![node("1", "menuitemcheckbox", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-toggle-field-name")));
    }

    // ── aria-dialog-name ──────────────────────────────────────────────────

    #[test]
    fn dialog_name_violation_dialog() {
        let tree = AXTree::from_nodes(vec![node("1", "dialog", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-dialog-name")));
    }

    #[test]
    fn dialog_name_pass_dialog() {
        let tree = AXTree::from_nodes(vec![node("1", "dialog", Some("Confirm action"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-dialog-name")));
        assert!(r.passes >= 1);
    }

    #[test]
    fn dialog_name_violation_alertdialog() {
        let tree = AXTree::from_nodes(vec![node("1", "alertdialog", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-dialog-name")));
    }

    #[test]
    fn dialog_name_pass_alertdialog() {
        let tree = AXTree::from_nodes(vec![node("1", "alertdialog", Some("Delete confirmation"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-dialog-name")));
        assert!(r.passes >= 1);
    }

    // ── aria-treeitem-name ────────────────────────────────────────────────

    #[test]
    fn treeitem_name_violation() {
        let tree = AXTree::from_nodes(vec![node("1", "treeitem", None)]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-treeitem-name")));
    }

    #[test]
    fn treeitem_name_pass() {
        let tree = AXTree::from_nodes(vec![node("1", "treeitem", Some("Documents"))]);
        let r = check_aria_naming_rules(&tree);
        assert!(!r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-treeitem-name")));
        assert!(r.passes >= 1);
    }

    // ── Combined test ─────────────────────────────────────────────────────

    #[test]
    fn all_named_elements_pass() {
        let tree = AXTree::from_nodes(vec![
            node("1", "button", Some("Close")),
            node("2", "textbox", Some("Search")),
            node("3", "meter", Some("Disk usage")),
            node("4", "progressbar", Some("Loading")),
            node("5", "checkbox", Some("Accept terms")),
            node("6", "dialog", Some("Settings")),
            node("7", "alertdialog", Some("Warning")),
            node("8", "treeitem", Some("Documents")),
            node("9", "slider", Some("Volume")),
            node("10", "radiogroup", Some("Options")),
        ]);
        let r = check_aria_naming_rules(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 10);
    }

    #[test]
    fn whitespace_only_name_counts_as_missing() {
        let tree = AXTree::from_nodes(vec![node("1", "button", Some("   "))]);
        let r = check_aria_naming_rules(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("aria-command-name")));
    }
}
