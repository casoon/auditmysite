//! DisclosureMenu pattern (issue #28).
//!
//! Detects collapsible menu triggers (hamburger menus, dropdown triggers):
//! a button (or button-role element) with `aria-expanded` is the canonical
//! disclosure pattern. Flags missing `aria-expanded` when the structure
//! suggests a disclosure but the attribute is absent.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

use super::{PatternAnalysis, PatternConfidence};

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let buttons = tree.nodes_with_role("button");
    let mut disclosure_count = 0usize;
    let mut with_controls = 0usize;

    for btn in &buttons {
        let expanded = btn.get_property_bool("expanded");
        if expanded.is_none() {
            continue;
        }
        disclosure_count += 1;

        let has_controls = btn.get_property_str("controls").is_some();
        if has_controls {
            with_controls += 1;
        }
    }

    if disclosure_count == 0 {
        return;
    }

    let confidence = if with_controls == disclosure_count {
        PatternConfidence::Strong
    } else {
        PatternConfidence::Partial
    };
    let detail = if with_controls == disclosure_count {
        format!(
            "{} disclosure trigger(s) with aria-expanded and aria-controls — well-formed pattern.",
            disclosure_count
        )
    } else {
        format!(
            "{} disclosure trigger(s) with aria-expanded ({} with aria-controls). Controls relationship strengthens screen-reader announcements.",
            disclosure_count, with_controls
        )
    };
    out.add_recognized("DisclosureMenu", detail, confidence);

    // Flag toggle-like nodes that look like menus but lack aria-expanded.
    // Heuristic: a `generic` or `link` node whose name contains "menu" /
    // "menü" and has an expanded child group is a likely disclosure that
    // failed to declare `aria-expanded`.
    for node in tree.iter() {
        let role = node.role.as_deref().unwrap_or("");
        if role != "generic" && role != "link" {
            continue;
        }
        let name = node.name.as_deref().unwrap_or("").to_lowercase();
        let looks_like_menu = name.contains("menu") || name.contains("menü");
        if !looks_like_menu {
            continue;
        }
        // Skip if it already has aria-expanded
        if node.get_property_bool("expanded").is_some() {
            continue;
        }
        // Has a focusable descendant suggesting an expandable region?
        let has_focusable_descendant = node
            .child_ids
            .iter()
            .filter_map(|id| tree.get_node(id))
            .any(|c| c.get_property_bool("focusable").unwrap_or(false));
        if !has_focusable_descendant {
            continue;
        }

        out.violations.push(
            Violation::new(
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::Medium,
                format!(
                    "Likely disclosure menu trigger (\"{}\") lacks aria-expanded — screen readers cannot announce open/closed state.",
                    node.name.as_deref().unwrap_or("(menu)")
                ),
                &node.node_id,
            )
            .with_fix(
                "Use a native <button> with aria-expanded=\"true|false\" toggled by the click handler.",
            )
            .with_rule_id("aria-expanded-required")
            .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_prop(id: &str, role: &str, prop: AXProperty) -> AXNode {
        let mut n = AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        };
        n.properties.push(prop);
        n
    }

    #[test]
    fn test_button_with_expanded_recognized() {
        let tree = AXTree::from_nodes(vec![node_with_prop(
            "1",
            "button",
            AXProperty {
                name: "expanded".into(),
                value: AXValue::Bool(false),
            },
        )]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized.len(), 1);
        assert_eq!(a.recognized[0].pattern, "DisclosureMenu");
    }

    #[test]
    fn test_button_with_expanded_and_controls_strong() {
        let mut n = node_with_prop(
            "1",
            "button",
            AXProperty {
                name: "expanded".into(),
                value: AXValue::Bool(false),
            },
        );
        n.properties.push(AXProperty {
            name: "controls".into(),
            value: AXValue::String("menu-1".into()),
        });
        let tree = AXTree::from_nodes(vec![n]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
    }

    #[test]
    fn test_nothing_without_buttons() {
        let tree = AXTree::from_nodes(vec![]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.recognized.is_empty());
    }
}
