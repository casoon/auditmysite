//! ModalDialog pattern (issue #29).
//!
//! Detects modal dialogs and flags missing accessible names or focusable
//! descendants. Note: actual focus-trap behavior cannot be verified from the
//! AXTree alone — that remains a manual-review concern.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    // Der Auslöser wird unabhängig davon angeboten, ob gerade ein Dialog im
    // Baum steht. Ein geschlossener `<dialog>` ist nicht gerendert und hat
    // damit keine Rolle — das ist der Normalfall beim Laden der Seite. Die
    // vorherige Fassung stieg hier aus und hat deshalb nie einen Kandidaten
    // erzeugt: über 171 gelaufene Seiten lief keine einzige Modal-Journey.
    emit_trigger_candidates(tree, out);

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
        let is_modal = dialog.get_property_bool("modal").unwrap_or(false);
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
            "{} {} detected, {} structurally well-formed (accessible name, aria-modal, focusable descendants).",
            dialogs.len(),
            if dialogs.len() == 1 { "dialog" } else { "dialogs" },
            well_formed
        ),
        confidence,
    );
}

/// Journey-Kandidaten aus Auslösern, die `aria-haspopup="dialog"` führen.
fn emit_trigger_candidates(tree: &AXTree, out: &mut PatternAnalysis) {
    for node in tree.iter() {
        if !matches!(node.role.as_deref(), Some("button") | Some("link")) {
            continue;
        }
        if !matches!(node.haspopup(), Some("dialog")) {
            continue;
        }
        if let Some(bid) = node.backend_dom_node_id {
            out.journey_candidates.push(JourneyCandidate {
                pattern_kind: PatternKind::Modal,
                trigger_backend_id: Some(bid),
                controlled_backend_id: None,
                confidence: 0.85,
                required_journey: JourneyKind::ModalOpen,
            });
        }
    }
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

    /// Ein geschlossener `<dialog>` ist nicht gerendert und steht nicht im
    /// Baum. Der Auslöser muss trotzdem angeboten werden — sonst wird die
    /// Modal-Journey nie ausgeführt, und genau das war der Fall.
    #[test]
    fn ausloeser_wird_auch_ohne_offenen_dialog_angeboten() {
        let mut trigger = AXNode {
            node_id: "1".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("button".into()),
            name: Some("Hinweis oeffnen".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "hasPopup".into(),
                value: AXValue::String("dialog".into()),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: Some(7),
        };
        trigger.properties.shrink_to_fit();
        let tree = AXTree::from_nodes(vec![trigger]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(
            a.recognized.is_empty(),
            "ohne Dialog im Baum gibt es nichts zu erkennen"
        );
        assert_eq!(a.journey_candidates.len(), 1);
        assert_eq!(a.journey_candidates[0].trigger_backend_id, Some(7));
        assert_eq!(
            a.journey_candidates[0].required_journey,
            JourneyKind::ModalOpen
        );
    }

    /// Die CDP-Kennung heißt `hasPopup`, nicht `haspopup`.
    #[test]
    fn kleingeschriebenes_haspopup_trifft_nicht() {
        let trigger = AXNode {
            node_id: "1".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("button".into()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "haspopup".into(),
                value: AXValue::String("dialog".into()),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: Some(7),
        };
        assert_eq!(trigger.haspopup(), None);
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
