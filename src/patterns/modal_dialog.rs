//! ModalDialog pattern (issue #29).
//!
//! Detects modal dialogs and flags missing accessible names or focusable
//! descendants. Note: actual focus-trap behavior cannot be verified from the
//! AXTree alone — that remains a manual-review concern.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

use super::{PatternAnalysis, PatternConfidence};

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let dialogs: Vec<_> = tree
        .iter()
        .filter(|n| matches!(n.role.as_deref(), Some("dialog") | Some("alertdialog")))
        .collect();

    if dialogs.is_empty() {
        return;
    }

    let mut well_formed = 0usize;
    for dialog in &dialogs {
        let has_name = dialog.name.as_deref().is_some_and(|n| !n.trim().is_empty());
        let is_modal = dialog
            .get_property_bool("modal")
            .or_else(|| dialog.get_property_bool("aria-modal"))
            .unwrap_or(false);
        let has_focusable_descendant = dialog
            .child_ids
            .iter()
            .filter_map(|id| tree.get_node(id))
            .any(|c| c.get_property_bool("focusable").unwrap_or(false));

        if !has_name {
            out.violations.push(
                Violation::new(
                    "4.1.2",
                    "Name, Role, Value",
                    WcagLevel::A,
                    Severity::High,
                    "Dialog has no accessible name — screen readers cannot announce its purpose.",
                    &dialog.node_id,
                )
                .with_fix(
                    "Add aria-labelledby pointing to the dialog title, or aria-label with a short description.",
                )
                .with_rule_id("aria-dialog-name")
                .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/"),
            );
        }

        if !has_focusable_descendant {
            out.violations.push(
                Violation::new(
                    "2.4.3",
                    "Focus Order",
                    WcagLevel::A,
                    Severity::Medium,
                    "Dialog contains no focusable elements — keyboard users cannot interact with it.",
                    &dialog.node_id,
                )
                .with_fix(
                    "Ensure the dialog contains at least one focusable element (close button, form field, action) and move initial focus there when opened.",
                )
                .with_rule_id("dialog-no-focusable")
                .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/"),
            );
        }

        if has_name && is_modal && has_focusable_descendant {
            well_formed += 1;
        }
    }

    let confidence = if well_formed == dialogs.len() {
        PatternConfidence::Strong
    } else {
        PatternConfidence::Partial
    };
    out.add_recognized(
        "ModalDialog",
        format!(
            "{} dialog(s) detected, {} structurally well-formed (accessible name, aria-modal, focusable descendants).",
            dialogs.len(),
            well_formed
        ),
        confidence,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn build_dialog_tree(name: Option<&str>, modal: bool, focusable_child: bool) -> AXTree {
        let mut dialog = AXNode {
            node_id: "1".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("dialog".into()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec!["2".into()],
            parent_id: None,
            backend_dom_node_id: None,
        };
        if modal {
            dialog.properties.push(AXProperty {
                name: "modal".into(),
                value: AXValue::Bool(true),
            });
        }
        let mut child = AXNode {
            node_id: "2".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("button".into()),
            name: Some("Close".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("1".into()),
            backend_dom_node_id: None,
        };
        if focusable_child {
            child.properties.push(AXProperty {
                name: "focusable".into(),
                value: AXValue::Bool(true),
            });
        }
        AXTree::from_nodes(vec![dialog, child])
    }

    #[test]
    fn test_well_formed_dialog_strong() {
        let tree = build_dialog_tree(Some("Confirm"), true, true);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
        assert!(a.violations.is_empty());
    }

    #[test]
    fn test_unnamed_dialog_violation() {
        let tree = build_dialog_tree(None, true, true);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.violations.iter().any(|v| v.rule == "4.1.2"));
    }

    #[test]
    fn test_dialog_without_focusable_violation() {
        let tree = build_dialog_tree(Some("X"), true, false);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.violations.iter().any(|v| v.rule == "2.4.3"));
    }
}
