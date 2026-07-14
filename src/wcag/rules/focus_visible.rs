//! WCAG 2.4.7 Focus Visible
//!
//! Any keyboard operable user interface has a mode of operation where the
//! keyboard focus indicator is visible.
//! Level AA
//!
//! Note: Full focus-visible checking requires CSS inspection via CDP — see
//! `focus_visible_css.rs` for the check that inspects `:focus` stylesheet
//! rules for suppressed outlines. This AX-tree-only rule is limited to the
//! degenerate case of a page with no focusable elements at all.
//!
//! An earlier version of this rule also flagged natively-focusable roles
//! (button/link/radio/…) carrying `tabindex="-1"` as removed from the tab
//! order. That check never fired against real CDP data — Chrome's
//! Accessibility tree does not expose a `tabindex` AX property (`focusable`
//! only reflects whether the element can receive focus at all, not whether
//! it is in the sequential tab order) — and, had it worked, it would have
//! misflagged the widely-recommended "roving tabindex" ARIA pattern (e.g.
//! WAI-ARIA APG radio groups and menus, where every non-active item
//! legitimately carries `tabindex="-1"`). Reachability via tabindex is
//! already covered under 2.1.1 by `keyboard.rs`/`click_handlers.rs`, so the
//! dead branch was removed rather than reimplemented here.

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
        .with_selector("root")
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

    /// Regression guard: `tabindex="-1"` on a normally-focusable role must
    /// NOT be flagged. This used to raise a "removed from tab order"
    /// violation, but that check read an AX `tabindex` property Chrome's
    /// CDP Accessibility tree never actually populates (so it was dead code
    /// in production), and — had it worked — it would have misflagged the
    /// widely-recommended "roving tabindex" ARIA pattern (e.g. radio groups
    /// and menus, where every non-active item legitimately carries
    /// `tabindex="-1"`). See module docs.
    #[test]
    fn test_tabindex_minus_one_is_not_flagged() {
        let tree = AXTree::from_nodes(vec![
            interactive_node("1", "radio", Some(0)),
            interactive_node("2", "radio", Some(-1)),
            interactive_node("3", "radio", Some(-1)),
        ]);
        let results = check_focus_visible(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
