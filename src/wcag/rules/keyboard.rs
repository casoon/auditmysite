//! WCAG 2.1.2 No Keyboard Trap
//!
//! Ob der Fokus einen Bereich wieder verlassen kann, zeigt nur echte
//! Tastaturbedienung. Hier bleibt der Hinweis je modalem Dialog und der
//! seitenweite `UNTESTED`-Vermerk.
//!
//! Die 2.1.1-Prüfungen laufen seit #694 im geteilten Bestand:
//! `keyboard/focusable-no-role` und `keyboard/interactive-not-focusable`
//! (siehe `wcag::shared`).

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.1.2
pub(super) const NO_KEYBOARD_TRAP_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.2",
    name: "No Keyboard Trap",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "If keyboard focus can be moved to a component, focus can be moved away",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/no-keyboard-trap.html",
    axe_id: "keyboard-trap",
    tags: &["wcag2a", "wcag212", "cat.keyboard"],
};

/// Hinweise zur Tastaturfalle (2.1.2): je modalem Dialog und seitenweit.
pub fn check_keyboard(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        results.nodes_checked += 1;

        // Check for potential keyboard traps (modal dialogs). Reported as Warning
        // (issue #569): the AX tree only confirms a modal dialog exists — it does
        // NOT confirm focus is actually trapped, only that a keyboard test is
        // needed to rule that out. A flat Violation here overclaims what was
        // measured; the severity stays High.
        if is_potential_keyboard_trap(node) {
            let violation = Violation::new(
                NO_KEYBOARD_TRAP_RULE.id,
                NO_KEYBOARD_TRAP_RULE.name,
                NO_KEYBOARD_TRAP_RULE.level,
                Severity::High,
                "Modal dialog detected — verify with keyboard that focus can be moved away (Escape, Tab, Shift+Tab)",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Ensure focus can be moved away using standard keyboard navigation")
            .with_help_url(NO_KEYBOARD_TRAP_RULE.help_url)
            .with_rule_id(NO_KEYBOARD_TRAP_RULE.axe_id)
            .as_warning();

            results.add_violation(violation);
        }
    }

    // 2.1.2 No Keyboard Trap — structural check only detects modal `modal=true` dialogs.
    // JavaScript focus management in custom widgets requires manual Tab-key verification.
    results.add_violation(
        Violation::new(
            NO_KEYBOARD_TRAP_RULE.id,
            NO_KEYBOARD_TRAP_RULE.name,
            NO_KEYBOARD_TRAP_RULE.level,
            Severity::Medium,
            "Keyboard trap behavior cannot be verified automatically. \
             Navigate the page using only the Tab key to confirm focus is never permanently trapped \
             in dialogs, carousels, or custom JavaScript widgets.",
            "page",
        )
        .with_fix(
            "Ensure every focusable region has a keyboard escape path (Escape key, visible close button reachable by Tab, or documented keyboard shortcut).",
        )
        .with_help_url(NO_KEYBOARD_TRAP_RULE.help_url)
            .with_rule_id(NO_KEYBOARD_TRAP_RULE.axe_id)
        .with_kind(Outcome::Untested),
    );

    results
}

/// Check for potential keyboard traps
fn is_potential_keyboard_trap(node: &crate::accessibility::AXNode) -> bool {
    let role = node.role.as_deref().unwrap_or("").to_lowercase();

    if role == "dialog" || role == "alertdialog" {
        if let Some(true) = node.get_property_bool("modal") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXValue};

    fn create_test_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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
    fn test_modal_dialog_keyboard_trap_is_warning() {
        // Issue #569: the AX tree only confirms a modal dialog exists, not that
        // focus is actually trapped — reclassified from Violation to Warning
        // (severity kept at High).
        let mut node = create_test_node("1", "dialog", None);
        node.properties.push(AXProperty {
            name: "modal".to_string(),
            value: AXValue::Bool(true),
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_keyboard(&tree);
        let finding = results
            .warnings
            .iter()
            .find(|v| v.rule == NO_KEYBOARD_TRAP_RULE.id && v.node_id == "1")
            .expect("expected a warning-kind finding for the modal dialog");
        assert_eq!(finding.kind, Outcome::Review);
        assert_eq!(finding.severity, Severity::High);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.rule == NO_KEYBOARD_TRAP_RULE.id && v.node_id == "1"));
    }
}
