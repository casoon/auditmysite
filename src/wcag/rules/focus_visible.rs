//! WCAG 2.4.7 Focus Visible
//!
//! Any keyboard operable user interface has a mode of operation where the
//! keyboard focus indicator is visible.
//! Level AA
//!
//! Note: Full focus-visible checking requires CSS inspection via CDP.
//! This rule checks for common AX tree patterns that indicate focus issues.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const FOCUS_VISIBLE_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.7",
    name: "Focus Visible",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Any keyboard operable user interface has a visible focus indicator",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/focus-visible.html",
    axe_id: "focus-visible",
    tags: &["wcag2aa", "wcag247", "cat.keyboard"],
};

pub fn check_focus_visible(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    let mut focusable_count = 0u32;

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        // Check focusable interactive elements
        if !node.is_interactive() && !node.is_focusable() {
            continue;
        }

        focusable_count += 1;

        // Check for tabindex=-1 on normally-focusable interactive elements
        // which removes them from tab order (potential focus visibility issue)
        let role = node.role.as_deref().unwrap_or("");
        let is_natively_focusable = matches!(
            role,
            "button"
                | "link"
                | "textbox"
                | "searchbox"
                | "combobox"
                | "listbox"
                | "checkbox"
                | "radio"
                | "switch"
                | "slider"
                | "menuitem"
        );

        if is_natively_focusable {
            if let Some(tabindex) = node.get_property_int("tabindex") {
                if tabindex == -1 {
                    let violation = Violation::new(
                        FOCUS_VISIBLE_RULE.id,
                        FOCUS_VISIBLE_RULE.name,
                        FOCUS_VISIBLE_RULE.level,
                        Severity::Medium,
                        format!(
                            "Interactive {} element removed from tab order (tabindex=-1)",
                            role
                        ),
                        node.node_id.clone(),
                    )
                    .with_role(node.role.clone())
                    .with_name(node.name.clone())
                    .with_fix("Ensure interactive elements remain keyboard accessible unless there's a valid reason to remove them")
                    .with_help_url(FOCUS_VISIBLE_RULE.help_url);

                    results.add_violation(violation);
                }
            }
        }
    }

    // If there are no focusable elements at all on the page, that's a problem
    if focusable_count == 0 && tree.len() > 5 {
        let violation = Violation::new(
            FOCUS_VISIBLE_RULE.id,
            FOCUS_VISIBLE_RULE.name,
            FOCUS_VISIBLE_RULE.level,
            Severity::High,
            "Page has no focusable interactive elements",
            "root",
        )
        .with_fix("Ensure interactive elements are keyboard focusable")
        .with_help_url(FOCUS_VISIBLE_RULE.help_url);

        results.add_violation(violation);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn interactive_node(id: &str, role: &str, tabindex: Option<i64>) -> AXNode {
        let mut properties = vec![AXProperty {
            name: "focusable".to_string(),
            value: AXValue::Bool(tabindex != Some(-1)),
        }];
        if let Some(ti) = tabindex {
            properties.push(AXProperty {
                name: "tabindex".to_string(),
                value: AXValue::Int(ti),
            });
        }
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some("Test".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties,
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_focusable_buttons_pass() {
        let tree = AXTree::from_nodes(vec![
            interactive_node("1", "button", Some(0)),
            interactive_node("2", "link", None),
        ]);
        let results = check_focus_visible(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_button_removed_from_tab_order() {
        let tree = AXTree::from_nodes(vec![interactive_node("1", "button", Some(-1))]);
        let results = check_focus_visible(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("tabindex=-1"));
    }
}
