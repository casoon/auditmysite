//! Accordion pattern (issue #32).
//!
//! Detects accordion structures: button with `aria-expanded` that toggles a
//! controlled region. Native disclosure widgets (`<details>`/`<summary>`)
//! also count when summary has the toggle semantics.
//!
//! Befunde meldet die Erkennung seit #694 nicht mehr: Der aufgeklappte
//! Button ohne `aria-controls` ist `patterns/accordion-controls-missing` im
//! geteilten Bestand. `accordion-trigger-not-button` entfällt ersatzlos --
//! an Rollen ohne `aria-expanded` meldet das `aria/attribute-not-allowed`,
//! an `link`, `tab` oder `treeitem` erlaubt WAI-ARIA den Zustand, und
//! fehlender Fokus ist `keyboard/interactive-not-focusable`.

use crate::accessibility::AXTree;

use super::disclosure_menu::NATIVE_DISCLOSURE_ROLES;
use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

pub(super) fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let mut triggers = 0usize;
    let mut with_controls = 0usize;
    let mut non_button_triggers = 0usize;

    for node in tree.iter() {
        let expanded = node.get_property_bool("expanded");
        if expanded.is_none() {
            continue;
        }
        // Skip dialog/menu/combobox — those have their own patterns.
        // Also skip Details (the <details> container): it carries expanded state
        // but is not itself the clickable trigger — DisclosureTriangle (<summary>) is.
        let role = node.role.as_deref().unwrap_or("");
        if matches!(
            role,
            "dialog" | "alertdialog" | "menu" | "combobox" | "listbox" | "Details"
        ) {
            continue;
        }
        if is_expandable_composite_item(node, tree) {
            continue;
        }

        triggers += 1;

        // aria-controls is a node reference in the AX tree (idrefList), not a plain
        // string — use has_property() which matches regardless of value type.
        let has_controls = node.has_property("controls");
        if has_controls {
            with_controls += 1;
        }

        // Accordion triggers should be buttons (APG). DisclosureTriangle is
        // Chrome's AX-tree role for <summary>, which has implicit ARIA role
        // "button" per the HTML-ARIA spec; DisclosureTriangleGrouped is the
        // same <summary> inside <details name>.
        let is_button_like = role == "button" || NATIVE_DISCLOSURE_ROLES.contains(&role);
        if !is_button_like {
            non_button_triggers += 1;
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
            "{} accordion {}; {} with aria-controls; {} non-button triggers.",
            triggers,
            if triggers == 1 { "trigger" } else { "triggers" },
            with_controls,
            non_button_triggers
        ),
        confidence,
    );

    // Emit journey candidates for interactive accordion verification.
    // Include both button and DisclosureTriangle (<summary>) triggers.
    // Skip nav/banner contexts (handled by DisclosureMenu).
    //
    // AccordionToggle and DisclosureToggle run the same journey. A trigger that
    // DisclosureMenu already offered would otherwise be clicked through twice
    // and report every finding twice. A menu button (`aria-haspopup`) that
    // DisclosureMenu offered as MenuOpen is skipped as well: ARIA APG treats it
    // as the Menu Button pattern, and the disclosure journey would judge it by
    // expectations it isn't meant to meet (plan 53, 2026-09-27).
    let already_offered: Vec<i64> = out
        .journey_candidates
        .iter()
        .filter(|c| {
            matches!(
                c.required_journey,
                JourneyKind::DisclosureToggle | JourneyKind::MenuOpen
            )
        })
        .filter_map(|c| c.trigger_backend_id)
        .collect();
    for node in tree.iter() {
        if node.get_property_bool("expanded").is_none() {
            continue;
        }
        let role = node.role.as_deref().unwrap_or("");
        if matches!(
            role,
            "dialog" | "alertdialog" | "menu" | "combobox" | "listbox" | "Details"
        ) {
            continue;
        }
        if role != "button" && !NATIVE_DISCLOSURE_ROLES.contains(&role) {
            continue;
        }
        if in_nav_or_banner(node, tree) {
            continue;
        }
        let has_controls = node.has_property("controls");
        if let Some(bid) = node.backend_dom_node_id {
            if already_offered.contains(&bid) {
                continue;
            }
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

/// A tree item, or a row of a grid/treegrid, carries `aria-expanded` as part
/// of its own composite widget: the tree or grid owns the keyboard contract
/// (arrow keys, Enter), so the item is no accordion trigger (#655).
fn is_expandable_composite_item(node: &crate::accessibility::AXNode, tree: &AXTree) -> bool {
    match node.role.as_deref() {
        Some("treeitem") => true,
        Some("row") => {
            let mut current = node.parent_id.as_deref();
            while let Some(parent) = current.and_then(|id| tree.get_node(id)) {
                if matches!(parent.role.as_deref(), Some("grid" | "treegrid")) {
                    return true;
                }
                current = parent.parent_id.as_deref();
            }
            false
        }
        _ => false,
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
    fn test_non_button_trigger_is_partial_without_finding() {
        let tree = AXTree::from_nodes(vec![trigger("1", "generic", Some("panel-1"))]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.violations.is_empty());
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Partial);
        assert!(a.recognized[0].message.contains("1 non-button triggers"));
    }

    #[test]
    fn test_disclosure_triangle_no_false_positive() {
        // Chrome exposes <summary> as DisclosureTriangle in the AX tree.
        // It has an implicit ARIA role of "button" per the HTML-ARIA spec
        // and must not be reported as a non-button accordion trigger.
        let tree = AXTree::from_nodes(vec![trigger("1", "DisclosureTriangle", None)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(
            a.recognized[0].message.contains("0 non-button triggers"),
            "DisclosureTriangle (<summary>) must not count as non-button trigger"
        );
    }

    /// `<summary>` in `<details name>` has its own role. It was reported as a
    /// non-button trigger (Medium, 4.1.2) on every exclusive accordion.
    #[test]
    fn grouped_summary_is_no_false_non_button_trigger() {
        let tree = AXTree::from_nodes(vec![trigger("1", "DisclosureTriangleGrouped", None)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(
            a.recognized[0].message.contains("0 non-button triggers"),
            "{:?}",
            a.recognized
        );
    }

    /// AccordionToggle and DisclosureToggle run the same journey: a trigger
    /// DisclosureMenu already offered gets no second candidate.
    #[test]
    fn trigger_already_offered_by_disclosure_menu_is_not_offered_twice() {
        for role in ["button", "DisclosureTriangle", "DisclosureTriangleGrouped"] {
            let mut n = trigger("1", role, None);
            n.backend_dom_node_id = Some(7);
            let tree = AXTree::from_nodes(vec![n]);
            let analysis = crate::patterns::analyze(&tree);
            let offered: Vec<_> = analysis
                .journey_candidates
                .iter()
                .filter(|c| c.trigger_backend_id == Some(7))
                .collect();
            assert_eq!(offered.len(), 1, "{role}: {offered:?}");
            assert_eq!(offered[0].required_journey, JourneyKind::DisclosureToggle);
        }
    }

    /// A menu button is tested by the menu journey only — no accordion
    /// candidate next to its MenuOpen one.
    #[test]
    fn menu_button_is_offered_to_the_menu_journey_only() {
        let mut n = trigger("1", "button", None);
        n.backend_dom_node_id = Some(7);
        n.properties.push(AXProperty {
            name: "hasPopup".into(),
            value: AXValue::String("menu".into()),
        });
        let tree = AXTree::from_nodes(vec![n]);
        let analysis = crate::patterns::analyze(&tree);
        let offered: Vec<_> = analysis
            .journey_candidates
            .iter()
            .filter(|c| c.trigger_backend_id == Some(7))
            .map(|c| c.required_journey)
            .collect();
        assert_eq!(offered, vec![JourneyKind::MenuOpen]);
    }

    /// Without a DisclosureToggle candidate for the trigger, the accordion
    /// still offers its own.
    #[test]
    fn accordion_still_offers_triggers_nobody_else_offered() {
        let mut n = trigger("1", "DisclosureTriangle", None);
        n.backend_dom_node_id = Some(7);
        let tree = AXTree::from_nodes(vec![n]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
        assert_eq!(
            a.journey_candidates[0].required_journey,
            JourneyKind::AccordionToggle
        );
    }

    fn with_parent(mut n: AXNode, parent: &str) -> AXNode {
        n.parent_id = Some(parent.into());
        n
    }

    fn container(id: &str, role: &str, parent: Option<&str>) -> AXNode {
        let mut n = trigger(id, role, None);
        n.properties.clear();
        n.parent_id = parent.map(Into::into);
        n
    }

    /// A treegrid parent row marks its expanded state with aria-expanded,
    /// exactly as the WAI-ARIA treegrid pattern prescribes (#655).
    #[test]
    fn expandable_row_in_treegrid_or_grid_is_no_accordion_trigger() {
        for grid_role in ["treegrid", "grid"] {
            let tree = AXTree::from_nodes(vec![
                container("g", grid_role, None),
                container("rg", "rowgroup", Some("g")),
                with_parent(trigger("r", "row", None), "rg"),
            ]);
            let mut a = PatternAnalysis::default();
            detect(&tree, &mut a);
            assert!(a.recognized.is_empty(), "{grid_role}: {:?}", a.recognized);
        }
    }

    #[test]
    fn expandable_treeitem_is_no_accordion_trigger() {
        let tree = AXTree::from_nodes(vec![
            container("t", "tree", None),
            with_parent(trigger("i", "treeitem", None), "t"),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.recognized.is_empty(), "{:?}", a.recognized);
    }

    /// Outside a grid, an expandable row keeps counting as a trigger.
    #[test]
    fn expandable_row_outside_grid_is_still_counted() {
        let tree = AXTree::from_nodes(vec![
            container("t", "table", None),
            with_parent(trigger("r", "row", None), "t"),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.recognized[0].message.contains("1 non-button triggers"));
    }

    #[test]
    fn test_details_container_skipped() {
        // The <details> element itself carries expanded state but is the container,
        // not the trigger — it must not be counted as an accordion trigger.
        let tree = AXTree::from_nodes(vec![trigger("1", "Details", None)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(
            a.recognized.is_empty(),
            "Details role (the <details> container) must not count as a trigger"
        );
    }
}
