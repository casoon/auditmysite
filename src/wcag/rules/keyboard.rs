//! WCAG 2.1.1 Keyboard Accessibility
//!
//! Ensures that all functionality is operable through a keyboard interface.
//! Level A - Critical for users who cannot use a mouse.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.1.1
pub const KEYBOARD_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.1",
    name: "Keyboard",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "All functionality must be operable through a keyboard interface",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html",
};

/// Rule metadata for 2.1.2
pub const NO_KEYBOARD_TRAP_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.2",
    name: "No Keyboard Trap",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "If keyboard focus can be moved to a component, focus can be moved away",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/no-keyboard-trap.html",
};

/// Check for keyboard accessibility issues
pub fn check_keyboard(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        results.nodes_checked += 1;

        // Check for elements with positive tabindex (disrupts natural tab order)
        if let Some(tabindex) = node.get_property_int("tabindex") {
            if tabindex > 0 {
                let violation = Violation::new(
                    KEYBOARD_RULE.id,
                    KEYBOARD_RULE.name,
                    KEYBOARD_RULE.level,
                    Severity::Moderate,
                    format!("Positive tabindex ({}) disrupts natural tab order", tabindex),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Use tabindex=\"0\" for focusable elements or tabindex=\"-1\" for programmatic focus")
                .with_help_url(KEYBOARD_RULE.help_url);

                results.add_violation(violation);
            } else {
                results.passes += 1;
            }
        }

        // Check for non-interactive elements made focusable without proper role
        if is_focusable_without_interactive_role(node) {
            let violation = Violation::new(
                KEYBOARD_RULE.id,
                KEYBOARD_RULE.name,
                KEYBOARD_RULE.level,
                Severity::Minor,
                "Focusable element without interactive role",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Add an appropriate ARIA role or use a native interactive element")
            .with_help_url(KEYBOARD_RULE.help_url);

            results.add_violation(violation);
        }

        // Check for potential keyboard traps (modal dialogs)
        if is_potential_keyboard_trap(node) {
            let violation = Violation::new(
                NO_KEYBOARD_TRAP_RULE.id,
                NO_KEYBOARD_TRAP_RULE.name,
                NO_KEYBOARD_TRAP_RULE.level,
                NO_KEYBOARD_TRAP_RULE.severity,
                "Potential keyboard trap detected (modal dialog)",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Ensure focus can be moved away using standard keyboard navigation")
            .with_help_url(NO_KEYBOARD_TRAP_RULE.help_url);

            results.add_violation(violation);
        }
    }

    results
}

/// Check if element is focusable but lacks interactive role
fn is_focusable_without_interactive_role(node: &crate::accessibility::AXNode) -> bool {
    let tabindex = node.get_property_int("tabindex");
    let has_focusable_tabindex = tabindex.map(|t| t >= 0).unwrap_or(false);
    let is_focusable = node.get_property_bool("focusable").unwrap_or(false);

    let non_interactive_roles = [
        "generic", "group", "region", "article", "section",
        "paragraph", "statictext", "none", "presentation"
    ];

    (has_focusable_tabindex || is_focusable) &&
        node.role.as_deref()
            .map(|r| non_interactive_roles.contains(&r.to_lowercase().as_str()))
            .unwrap_or(true)
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

    fn create_node_with_tabindex(id: &str, role: &str, tabindex: i64) -> AXNode {
        let mut node = create_test_node(id, role, None);
        node.properties.push(AXProperty {
            name: "tabindex".to_string(),
            value: AXValue::Int(tabindex),
        });
        node
    }

    #[test]
    fn test_keyboard_rule_metadata() {
        assert_eq!(KEYBOARD_RULE.id, "2.1.1");
        assert_eq!(KEYBOARD_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_positive_tabindex_violation() {
        let tree = AXTree::from_nodes(vec![
            create_node_with_tabindex("1", "generic", 5)
        ]);

        let results = check_keyboard(&tree);
        assert!(!results.violations.is_empty());
        assert!(results.violations.iter().any(|v| v.message.contains("Positive tabindex")));
    }

    #[test]
    fn test_zero_tabindex_no_violation() {
        let tree = AXTree::from_nodes(vec![
            create_node_with_tabindex("1", "button", 0)
        ]);

        let results = check_keyboard(&tree);
        assert!(!results.violations.iter().any(|v| v.message.contains("Positive tabindex")));
    }
}
