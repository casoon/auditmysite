//! WCAG 2.1.1 Keyboard Accessibility
//!
//! Ensures that all functionality is operable through a keyboard interface.
//! Level A - Critical for users who cannot use a mouse.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.1.1
pub const KEYBOARD_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.1",
    name: "Keyboard",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "All functionality must be operable through a keyboard interface",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html",
    axe_id: "keyboard",
    tags: &["wcag2a", "wcag211", "cat.keyboard"],
};

/// Rule metadata for the "focusable without interactive role" sub-check.
/// Same WCAG criterion as `KEYBOARD_RULE` but a distinct `axe_id` — this lets
/// `output::explanations::get_explanation` resolve a customer-facing
/// explanation specific to this case instead of falling back to
/// `KEYBOARD_RULE`'s generic 2.1.1 text, which is written around "add
/// tabindex + a keydown handler" and does not fit here: the element is
/// already focusable, what it lacks is a role (#571).
pub const FOCUSABLE_NO_ROLE_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.1",
    name: "Keyboard",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "All functionality must be operable through a keyboard interface",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html",
    axe_id: "focusable-no-role",
    tags: &["wcag2a", "wcag211", "cat.keyboard"],
};

/// Rule metadata for 2.1.2
pub const NO_KEYBOARD_TRAP_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.2",
    name: "No Keyboard Trap",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "If keyboard focus can be moved to a component, focus can be moved away",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/no-keyboard-trap.html",
    axe_id: "keyboard-trap",
    tags: &["wcag2a", "wcag212", "cat.keyboard"],
};

/// Check for keyboard accessibility issues
///
/// Positive-tabindex detection used to live here too, but `tabindex` is not
/// an AX property (dead code, #QA-030) and the concern is really about focus
/// *order*, not keyboard operability — it is now the sole responsibility of
/// `focus_order.rs`'s DOM-based `check_positive_tabindex_with_page` (2.4.3).
pub fn check_keyboard(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        results.nodes_checked += 1;

        // Check for non-interactive elements made focusable without proper role.
        // Reported as Warning (issue #569): the AX tree only shows a structural
        // mismatch (focusable, but no interactive role) — whether the element is
        // actually operable via keyboard depends on JS event handling that this
        // static check cannot observe, so this is a review hint, not a confirmed
        // violation.
        if is_focusable_without_interactive_role(node) {
            let violation = Violation::new(
                KEYBOARD_RULE.id,
                KEYBOARD_RULE.name,
                KEYBOARD_RULE.level,
                Severity::Low,
                "Focusable element without interactive role — may not be operable via keyboard; verify manually",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Add an appropriate ARIA role or use a native interactive element")
            .with_help_url(KEYBOARD_RULE.help_url)
            .with_rule_id(FOCUSABLE_NO_ROLE_RULE.axe_id)
            .as_warning();

            results.add_violation(violation);
        }

        // Check for interactive ARIA role on non-focusable element (issue #34).
        // Reported as Warning: the AX tree `focusable` attribute may not
        // reflect JS-added tab handling, so a definitive violation requires
        // interactive testing.
        if has_interactive_role_but_not_focusable(node) {
            let role = node.role.as_deref().unwrap_or("interactive");
            let warning = Violation::new(
                KEYBOARD_RULE.id,
                KEYBOARD_RULE.name,
                KEYBOARD_RULE.level,
                Severity::High,
                format!(
                    "Element has role=\"{role}\" but appears not keyboard-focusable — verify that tabindex or JS event handling is present"
                ),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Add tabindex=\"0\" to make the element focusable, or use the native HTML element (e.g. <button>, <a>).",
            )
            .with_help_url(KEYBOARD_RULE.help_url)
            .with_rule_id(KEYBOARD_RULE.axe_id)
            .as_warning();
            results.add_violation(warning);
        }

        // Check for potential keyboard traps (modal dialogs). Reported as Warning
        // (issue #569): the AX tree only confirms a modal dialog exists — it does
        // NOT confirm focus is actually trapped, only that a keyboard test is
        // needed to rule that out. A flat Violation here overclaims what was
        // measured (compare the sibling `has_interactive_role_but_not_focusable`
        // check above, which uses the same Warning + reduced-but-not-zeroed
        // severity treatment for the same reason).
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

/// Check if element is focusable but lacks interactive role
fn is_focusable_without_interactive_role(node: &crate::accessibility::AXNode) -> bool {
    // `tabindex` is not an AX property (#QA-030); `focusable` is the real,
    // Chrome-computed property that already reflects tabindex's effect on
    // focusability, so it fully covers what the removed tabindex read
    // attempted (see the equivalent fix in `focus_visible.rs`).
    let is_focusable = node.get_property_bool("focusable").unwrap_or(false);

    let non_interactive_roles = [
        "generic",
        "group",
        "region",
        "article",
        "section",
        "paragraph",
        "statictext",
        "none",
        "presentation",
    ];

    is_focusable
        && node
            .role
            .as_deref()
            .map(|r| non_interactive_roles.contains(&r.to_lowercase().as_str()))
            .unwrap_or(true)
}

/// Check if element has an interactive ARIA role but is not focusable.
/// Catches `<div role="button">` without `tabindex` — keyboard-unreachable widgets.
/// Native interactive elements (`<button>`, `<a>`, `<input>`) are auto-focusable
/// even without tabindex, so this only flags ARIA-roled non-native elements.
fn has_interactive_role_but_not_focusable(node: &crate::accessibility::AXNode) -> bool {
    let role = match node.role.as_deref() {
        Some(r) => r.to_lowercase(),
        None => return false,
    };

    let interactive_roles = [
        "button",
        "link",
        "checkbox",
        "radio",
        "switch",
        "menuitem",
        "menuitemcheckbox",
        "menuitemradio",
        "tab",
        "option",
        "treeitem",
    ];
    if !interactive_roles.contains(&role.as_str()) {
        return false;
    }

    // Native interactive elements are focusable=true automatically.
    // Only flag when role is interactive but element is not focusable.
    let is_focusable = node.get_property_bool("focusable").unwrap_or(false);
    !is_focusable
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
    fn test_keyboard_rule_metadata() {
        assert_eq!(KEYBOARD_RULE.id, "2.1.1");
        assert_eq!(KEYBOARD_RULE.level, WcagLevel::A);
    }

    // Positive-tabindex detection moved to focus_order.rs's DOM-based
    // check_positive_tabindex_with_page (2.4.3) — see that module's tests.

    fn create_node_with_focusable(id: &str, role: &str, focusable: bool) -> AXNode {
        let mut node = create_test_node(id, role, None);
        node.properties.push(AXProperty {
            name: "focusable".to_string(),
            value: AXValue::Bool(focusable),
        });
        node
    }

    #[test]
    fn test_role_button_without_focusable_flagged() {
        // <div role="button"> without tabindex → focusable=false → warning (heuristic)
        let tree = AXTree::from_nodes(vec![create_node_with_focusable("1", "button", false)]);
        let results = check_keyboard(&tree);
        assert!(
            results
                .warnings
                .iter()
                .any(|v| v.message.contains("not keyboard-focusable")),
            "Expected warning for role=button without focusable; got warnings: {:?}",
            results
                .warnings
                .iter()
                .map(|v| &v.message)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_native_button_with_focusable_passes() {
        // Native <button> is focusable=true → no violation
        let tree = AXTree::from_nodes(vec![create_node_with_focusable("1", "button", true)]);
        let results = check_keyboard(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("not keyboard-focusable")));
    }

    #[test]
    fn test_non_interactive_role_not_flagged() {
        // role="generic" without focusable is not a #34 violation (different rule covers that)
        let tree = AXTree::from_nodes(vec![create_node_with_focusable("1", "generic", false)]);
        let results = check_keyboard(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("not keyboard-focusable")));
    }

    #[test]
    fn test_focusable_without_interactive_role_is_warning() {
        // Issue #569: structural inference only (focusable + non-interactive
        // role) — whether the element is actually unusable via keyboard needs
        // manual verification, so this must be a Warning, not a Violation.
        let tree = AXTree::from_nodes(vec![create_node_with_focusable("1", "generic", true)]);
        let results = check_keyboard(&tree);
        let finding = results
            .warnings
            .iter()
            .find(|v| {
                v.message
                    .contains("Focusable element without interactive role")
            })
            .expect("expected a warning-kind finding");
        assert_eq!(finding.kind, Outcome::Review);
        // Regression for #571: this finding must carry the distinct
        // "focusable-no-role" rule_id, not the shared "keyboard" id, so
        // `output::explanations::get_explanation` resolves the role-specific
        // explanation instead of KEYBOARD_RULE's generic tabindex+keydown text.
        assert_eq!(
            finding.rule_id.as_deref(),
            Some(FOCUSABLE_NO_ROLE_RULE.axe_id)
        );
        assert!(!results.violations.iter().any(|v| v
            .message
            .contains("Focusable element without interactive role")));
    }

    #[test]
    fn test_modal_dialog_keyboard_trap_is_warning() {
        // Issue #569: the AX tree only confirms a modal dialog exists, not that
        // focus is actually trapped — reclassified from Violation to Warning
        // (severity kept at High, following the sibling
        // has_interactive_role_but_not_focusable precedent of not zeroing out
        // severity just because a finding is a Warning).
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
