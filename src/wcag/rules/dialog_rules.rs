//! WCAG 4.1.2, 2.4.3 - Dialog Rules
//!
//! Checks that dialogs and alerts have accessible names and correct modal indication.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for dialog accessibility (4.1.2)
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value - Dialogs",
    level: WcagLevel::A,
    severity: Severity::High,
    description:
        "Dialogs and alert regions must have accessible names and be properly marked as modal",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "dialog-name",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Run all dialog-related WCAG checks
pub fn check_dialog_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        match role {
            "dialog" | "alertdialog" => {
                check_dialog_has_name(node, &mut results);
                if role == "dialog" {
                    check_dialog_modal_property(node, &mut results);
                }
            }
            "alert" | "status" => {
                check_alert_has_name(node, &mut results);
            }
            _ => {}
        }
    }

    results
}

/// Dialogs and alertdialogs must have an accessible name
fn check_dialog_has_name(node: &AXNode, results: &mut WcagResults) {
    if !node.has_name() {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::High,
            "Dialog has no accessible name",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Add aria-labelledby pointing to the dialog title, or use aria-label")
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Dialogs should indicate modal status via aria-modal
fn check_dialog_modal_property(node: &AXNode, results: &mut WcagResults) {
    let is_modal = node
        .get_property_bool("modal")
        .or_else(|| node.get_property_bool("aria-modal"))
        .unwrap_or(false);

    if !is_modal {
        let violation = Violation::new(
            "4.1.2",
            "Name, Role, Value - Dialog Modal",
            WcagLevel::A,
            Severity::Medium,
            "Dialog may not be marked as modal",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix("Add aria-modal=\"true\" to the dialog element so assistive technologies know to restrict focus")
        .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Alert and status regions benefit from having an accessible name
fn check_alert_has_name(node: &AXNode, results: &mut WcagResults) {
    if !node.has_name() {
        let violation = Violation::new(
            "4.1.2",
            "Name, Role, Value - Alert Region",
            WcagLevel::A,
            Severity::Medium,
            "Alert/status region has no accessible name",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Add aria-label or aria-labelledby to identify the alert or status region")
        .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn make_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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
    fn test_dialog_without_name_flagged() {
        let nodes = vec![make_node("1", "dialog", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_dialog_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Dialog has no accessible name")));
    }

    #[test]
    fn test_dialog_with_name_passes_name_check() {
        let nodes = vec![make_node("1", "dialog", Some("Settings"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_dialog_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("Dialog has no accessible name")));
    }

    #[test]
    fn test_dialog_without_modal_flagged() {
        let nodes = vec![make_node("1", "dialog", Some("Settings"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_dialog_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("marked as modal")));
    }

    #[test]
    fn test_dialog_with_modal_passes() {
        let mut node = make_node("1", "dialog", Some("Settings"));
        node.properties.push(AXProperty {
            name: "modal".to_string(),
            value: AXValue::Bool(true),
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_dialog_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("marked as modal")));
    }

    #[test]
    fn test_alert_without_name_flagged() {
        let nodes = vec![make_node("1", "alert", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_dialog_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible name")));
    }

    #[test]
    fn test_alertdialog_without_name_flagged() {
        let nodes = vec![make_node("1", "alertdialog", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_dialog_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Dialog has no accessible name")));
    }
}
