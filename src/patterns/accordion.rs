//! Accordion pattern (issue #32).
//!
//! Detects accordion structures: button with `aria-expanded` that toggles a
//! controlled region. Native disclosure widgets (`<details>`/`<summary>`)
//! also count when summary has the toggle semantics.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

use super::{PatternAnalysis, PatternConfidence};

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

        let has_controls = node.get_property_str("controls").is_some();
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
        if role == "button" && !has_controls {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn trigger(id: &str, role: &str, controls: Option<&str>) -> AXNode {
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
                value: AXValue::Bool(false),
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
    fn test_button_without_controls_low_violation() {
        let tree = AXTree::from_nodes(vec![trigger("1", "button", None)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a
            .violations
            .iter()
            .any(|v| v.message.contains("aria-controls")));
    }
}
