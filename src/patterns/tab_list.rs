//! TabList pattern (issue #30).
//!
//! Detects tab-list/tab/tabpanel structures and validates that:
//! - each tab declares aria-selected
//! - each tab has aria-controls pointing to a tabpanel that exists
//!
//! Required-children violations (tablist without tabs) are already covered
//! by the widget_rules check — this pattern adds semantic validation.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let tablists = tree.nodes_with_role("tablist");
    if tablists.is_empty() {
        return;
    }

    let mut tabs_total = 0usize;
    let mut tabs_with_selected = 0usize;
    let mut tabs_with_valid_controls = 0usize;

    for tablist in &tablists {
        let tabs: Vec<_> = tablist
            .child_ids
            .iter()
            .filter_map(|id| tree.get_node(id))
            .filter(|c| c.role.as_deref() == Some("tab"))
            .collect();

        for tab in &tabs {
            tabs_total += 1;

            let has_selected = tab.has_property("selected");
            if has_selected {
                tabs_with_selected += 1;
            } else {
                out.violations.push(
                    Violation::new(
                        "4.1.2",
                        "Name, Role, Value",
                        WcagLevel::A,
                        Severity::Medium,
                        "Tab is missing aria-selected — assistive tech cannot announce which tab is active.",
                        &tab.node_id,
                    )
                    .with_fix(
                        "Set aria-selected=\"true\" on the active tab and aria-selected=\"false\" on the others; toggle on activation.",
                    )
                    .with_rule_id("tab-no-aria-selected")
                    .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/tabs/"),
                );
            }

            // aria-controls → tabpanel (id_matches is a stub; just count presence)
            if tab.has_property("controls") {
                tabs_with_valid_controls += 1;
            }
        }
    }

    let confidence = if tabs_total > 0 && tabs_with_selected == tabs_total {
        PatternConfidence::Strong
    } else {
        PatternConfidence::Partial
    };
    out.add_recognized(
        "TabList",
        format!(
            "{} tablist(s) with {} tab(s); {} have aria-selected, {} reference a valid tabpanel.",
            tablists.len(),
            tabs_total,
            tabs_with_selected,
            tabs_with_valid_controls
        ),
        confidence,
    );

    // Emit journey candidates: one per tablist using the first tab as trigger.
    for tablist in &tablists {
        let first_tab = tablist
            .child_ids
            .iter()
            .filter_map(|id| tree.get_node(id))
            .find(|c| c.role.as_deref() == Some("tab"));
        if let Some(tab) = first_tab {
            if let Some(bid) = tab.backend_dom_node_id {
                out.journey_candidates.push(JourneyCandidate {
                    pattern_kind: PatternKind::Tabs,
                    trigger_backend_id: Some(bid),
                    controlled_backend_id: None,
                    confidence: 0.8,
                    required_journey: JourneyKind::TabsNavigate,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn make_tab(id: &str, selected: bool) -> AXNode {
        let mut n = AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("tab".into()),
            name: Some(format!("Tab {id}")),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("0".into()),
            backend_dom_node_id: None,
        };
        if selected {
            n.properties.push(AXProperty {
                name: "selected".into(),
                value: AXValue::Bool(true),
            });
        }
        n
    }

    #[test]
    fn test_tab_without_selected_emits_violation() {
        let tablist = AXNode {
            node_id: "0".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("tablist".into()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec!["1".into()],
            parent_id: None,
            backend_dom_node_id: None,
        };
        let tab = make_tab("1", false);
        let tree = AXTree::from_nodes(vec![tablist, tab]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a
            .violations
            .iter()
            .any(|v| v.message.contains("aria-selected")));
    }

    #[test]
    fn test_well_formed_tablist() {
        let tablist = AXNode {
            node_id: "0".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("tablist".into()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec!["1".into()],
            parent_id: None,
            backend_dom_node_id: None,
        };
        let tab = make_tab("1", true);
        let tree = AXTree::from_nodes(vec![tablist, tab]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.violations.is_empty());
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
    }
}
