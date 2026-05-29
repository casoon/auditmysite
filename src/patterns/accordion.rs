//! Accordion pattern (issue #32).
//!
//! Detects accordion structures: button with `aria-expanded` that toggles a
//! controlled region. Native disclosure widgets (`<details>`/`<summary>`)
//! also count when summary has the toggle semantics.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let mut triggers = 0usize;
    let mut with_controls = 0usize;
    let mut non_button_triggers = 0usize;

    for node in tree.iter() {
        let expanded = node.get_property_bool("expanded");
        if expanded.is_none() {
            continue;
        }
        // Skip dialog/menu/combobox — those have their own patterns.
        let role = node.role.as_deref().unwrap_or("");
        if matches!(
            role,
            "dialog" | "alertdialog" | "menu" | "combobox" | "listbox"
        ) {
            continue;
        }

        triggers += 1;

        // aria-controls is a node reference in the AX tree (idrefList), not a plain
        // string — use has_property() which matches regardless of value type.
        let has_controls = node.has_property("controls");
        if has_controls {
            with_controls += 1;
        }

        // Accordion triggers must be buttons.
        if role != "button" {
            non_button_triggers += 1;
            out.violations.push(
                Violation::new(
                    "4.1.2",
                    "Name, Role, Value",
                    WcagLevel::A,
                    Severity::Medium,
                    format!(
                        "Accordion trigger has aria-expanded but role is \"{role}\" — should be a button so keyboard users can activate it with Enter/Space."
                    ),
                    &node.node_id,
                )
                .with_fix(
                    "Use a native <button> as the accordion trigger, or set role=\"button\" with tabindex=\"0\" and a keydown handler.",
                )
                .with_rule_id("accordion-trigger-not-button")
                .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/accordion/"),
            );
        }

        // Trigger without aria-controls is a warning (not strictly required
        // but strongly recommended for screen readers).
        // Only check when the trigger is currently expanded: Chrome CDP does not
        // resolve the `controls` AX property when the target element is hidden
        // (display:none / aria-hidden), so the check is unreliable for collapsed
        // triggers even when aria-controls is correctly set in the DOM.
        // Nav/banner exception remains for disclosure menus that never expand
        // into a visible AX node.
        if role == "button"
            && expanded == Some(true)
            && !has_controls
            && !in_nav_or_banner(node, tree)
        {
            out.violations.push(
                Violation::new(
                    "4.1.2",
                    "Name, Role, Value",
                    WcagLevel::A,
                    Severity::Low,
                    "Accordion trigger has aria-expanded but no aria-controls — screen readers cannot identify the controlled region.",
                    &node.node_id,
                )
                .with_fix(
                    "Add aria-controls=\"<id>\" pointing to the collapsible region.",
                )
                .with_rule_id("accordion-no-controls")
                .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/accordion/"),
            );
        }
    }

    if triggers == 0 {
        return;
    }

    let confidence = if non_button_triggers == 0 && with_controls == triggers {
        PatternConfidence::Strong
    } else {
        PatternConfidence::Partial
    };
    out.add_recognized(
        "Accordion",
        format!(
            "{} accordion trigger(s); {} with aria-controls; {} non-button triggers.",
            triggers, with_controls, non_button_triggers
        ),
        confidence,
    );

    // Emit journey candidates for interactive accordion verification.
    // Only button-role triggers, skip nav/banner contexts (handled by DisclosureMenu).
    for node in tree.iter() {
        if node.get_property_bool("expanded").is_none() {
            continue;
        }
        let role = node.role.as_deref().unwrap_or("");
        if matches!(
            role,
            "dialog" | "alertdialog" | "menu" | "combobox" | "listbox"
        ) {
            continue;
        }
        if role != "button" {
            continue;
        }
        if in_nav_or_banner(node, tree) {
            continue;
        }
        let has_controls = node.has_property("controls");
        if let Some(bid) = node.backend_dom_node_id {
            out.journey_candidates.push(JourneyCandidate {
                pattern_kind: PatternKind::Accordion,
                trigger_backend_id: Some(bid),
                controlled_backend_id: None,
                confidence: if has_controls { 0.85 } else { 0.7 },
                required_journey: JourneyKind::AccordionToggle,
            });
        }
    }
}

fn in_nav_or_banner(node: &crate::accessibility::AXNode, tree: &AXTree) -> bool {
    let mut current = node.parent_id.as_deref();
    while let Some(id) = current {
        if let Some(parent) = tree.get_node(id) {
            match parent.role.as_deref() {
                Some("navigation") | Some("banner") => return true,
                _ => {}
            }
            current = parent.parent_id.as_deref();
        } else {
            break;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn trigger(id: &str, role: &str, controls: Option<&str>) -> AXNode {
        trigger_with_expanded(id, role, controls, false)
    }

    fn trigger_with_expanded(
        id: &str,
        role: &str,
        controls: Option<&str>,
        expanded: bool,
    ) -> AXNode {
        let mut n = AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: Some("Toggle".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "expanded".into(),
                value: AXValue::Bool(expanded),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        };
        if let Some(target) = controls {
            n.properties.push(AXProperty {
                name: "controls".into(),
                value: AXValue::String(target.into()),
            });
        }
        n
    }

    #[test]
    fn test_button_with_controls_strong() {
        let tree = AXTree::from_nodes(vec![trigger("1", "button", Some("panel-1"))]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
        assert!(a.violations.is_empty());
    }

    #[test]
    fn test_non_button_trigger_violation() {
        let tree = AXTree::from_nodes(vec![trigger("1", "generic", Some("panel-1"))]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a
            .violations
            .iter()
            .any(|v| v.message.contains("should be a button")));
    }

    #[test]
    fn test_collapsed_button_without_controls_no_violation() {
        // When collapsed (expanded=false), Chrome CDP doesn't resolve the `controls`
        // property for hidden targets — the check is unreliable, so no violation is emitted.
        let tree = AXTree::from_nodes(vec![trigger("1", "button", None)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(
            a.violations
                .iter()
                .all(|v| !v.message.contains("aria-controls")),
            "collapsed trigger should not emit aria-controls violation"
        );
    }

    #[test]
    fn test_expanded_button_without_controls_low_violation() {
        // When expanded (expanded=true), the controlled panel should be in the AX tree.
        // A missing `controls` property then means aria-controls is truly absent.
        let tree = AXTree::from_nodes(vec![trigger_with_expanded("1", "button", None, true)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(
            a.violations
                .iter()
                .any(|v| v.message.contains("aria-controls")),
            "expanded trigger without controls should emit aria-controls violation"
        );
    }
}
