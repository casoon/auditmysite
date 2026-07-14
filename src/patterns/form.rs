//! Form pattern detector — identifies forms with required fields that are
//! candidates for the form-error-announcement journey.
//!
//! The journey needs a submit button backend-node-id. We look for buttons
//! with role="button" whose name/description contains a submit-hinting
//! fragment ("submit", "send", …), then group required fields by the
//! structurally nearest such button — the button sharing the deepest common
//! ancestor with a given field — so a multi-form page (e.g. search + login +
//! newsletter) yields one candidate per form instead of one arbitrary
//! page-wide candidate. Up to 3 candidates are emitted per page.

use std::collections::HashMap;

use crate::accessibility::{AXNode, AXTree};

use super::{
    JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind,
    PURCHASE_FINAL_HINTS,
};

/// Cap on how many distinct form clusters become journey candidates per page
/// — keeps the interactive-journey budget bounded on pages with many forms.
const MAX_FORM_CANDIDATES: usize = 3;

/// Submit-related name fragments (case-insensitive).
///
/// Purchase-final and booking triggers are deliberately excluded here (see
/// `PURCHASE_FINAL_HINTS`) — the form-error journey submits with required
/// fields left empty, but a synthetic click on a real "place order"/"book"
/// button on a live site with session/autofill state can still complete a
/// real transaction.
const SUBMIT_HINTS: &[&str] = &[
    "submit",
    "send",
    "senden",
    "absenden",
    "anmelden",
    "registrieren",
    "register",
    "login",
    "sign in",
    "log in",
    "suchen",
    "search",
    "apply",
    "bewerben",
    "kontakt",
    "contact",
];

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    // Collect required form controls.
    let required_controls: Vec<_> = tree
        .iter()
        .filter(|n| {
            matches!(
                n.role.as_deref(),
                Some("textbox")
                    | Some("searchbox")
                    | Some("combobox")
                    | Some("listbox")
                    | Some("radio")
                    | Some("checkbox")
                    | Some("spinbutton")
            ) && n.get_property_bool("required").unwrap_or(false)
        })
        .collect();

    if required_controls.is_empty() {
        return;
    }

    // Eligible submit buttons: must carry a submit-hinting name, and must
    // NOT be a purchase-final/booking button (excluded entirely — never a
    // synthetic-click trigger, see `PURCHASE_FINAL_HINTS` — checked before
    // hint-matching so such a button can never be selected via any path).
    // No hint match anywhere on the page means no candidate at all, not a
    // guess (there is deliberately no "first button" fallback).
    let mut buttons: Vec<&AXNode> = tree
        .iter()
        .filter(|n| matches!(n.role.as_deref(), Some("button")))
        .filter(|n| {
            let name = n.name.as_deref().unwrap_or("").to_lowercase();
            !PURCHASE_FINAL_HINTS.iter().any(|h| name.contains(h))
        })
        .filter(|n| {
            let name = n.name.as_deref().unwrap_or("").to_lowercase();
            SUBMIT_HINTS.iter().any(|h| name.contains(h))
        })
        .collect();

    if buttons.is_empty() {
        return;
    }
    // Deterministic order — HashMap iteration order is otherwise unspecified,
    // and tie-breaking below depends on encountering candidates in a stable
    // order.
    buttons.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    // Group required controls by the structurally nearest eligible button —
    // the one sharing the deepest common ancestor with the control — instead
    // of picking one button for the whole page. A multi-form page (search +
    // login + newsletter) yields one cluster per form; a single-form page
    // still yields exactly one cluster, identical to the prior behavior.
    let button_chains: Vec<(&AXNode, Vec<&str>)> = buttons
        .iter()
        .map(|b| (*b, ancestor_chain_from_root(tree, b)))
        .collect();

    let mut clusters: HashMap<&str, (&AXNode, usize)> = HashMap::new();
    for control in &required_controls {
        let control_chain = ancestor_chain_from_root(tree, control);
        let nearest = button_chains
            .iter()
            .max_by_key(|(_, chain)| common_prefix_len(&control_chain, chain));
        if let Some((button, _)) = nearest {
            clusters
                .entry(button.node_id.as_str())
                .or_insert((*button, 0))
                .1 += 1;
        }
    }

    if clusters.is_empty() {
        return;
    }

    // Busiest form first (most required fields), then by node id for
    // deterministic ordering; cap so a page with many forms doesn't exhaust
    // the interactive-journey budget.
    let mut ranked: Vec<(&AXNode, usize)> = clusters.into_values().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.node_id.cmp(&b.0.node_id)));
    ranked.truncate(MAX_FORM_CANDIDATES);

    for (trigger, field_count) in ranked {
        out.add_recognized(
            "Form",
            format!(
                "{field_count} required {} detected; submit trigger: {}",
                if field_count == 1 { "field" } else { "fields" },
                trigger.name.as_deref().unwrap_or("(unnamed button)")
            ),
            PatternConfidence::Partial,
        );

        out.journey_candidates.push(JourneyCandidate {
            pattern_kind: PatternKind::Form,
            trigger_backend_id: trigger.backend_dom_node_id,
            controlled_backend_id: None,
            confidence: 0.75,
            required_journey: JourneyKind::FormErrorSubmit,
        });
    }
}

/// Ordered ancestor chain from the tree root down to (but excluding) `node`.
/// Used to find the structurally nearest button to a given control via
/// shared-prefix length (deeper shared prefix = closer common ancestor).
fn ancestor_chain_from_root<'a>(tree: &'a AXTree, node: &'a AXNode) -> Vec<&'a str> {
    let mut chain: Vec<&'a str> = Vec::new();
    let mut current = node.parent_id.as_deref();
    while let Some(id) = current {
        chain.push(id);
        current = tree.nodes.get(id).and_then(|p| p.parent_id.as_deref());
    }
    chain.reverse();
    chain
}

fn common_prefix_len(a: &[&str], b: &[&str]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn required_textbox(id: &str) -> AXNode {
        required_textbox_in(id, None)
    }

    fn required_textbox_in(id: &str, parent_id: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("textbox".into()),
            name: Some("Email".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "required".into(),
                value: AXValue::Bool(true),
            }],
            child_ids: vec![],
            parent_id: parent_id.map(String::from),
            backend_dom_node_id: None,
        }
    }

    fn button(id: &str, name: &str) -> AXNode {
        button_full(id, name, None, 42)
    }

    fn button_full(id: &str, name: &str, parent_id: Option<&str>, backend_id: i64) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("button".into()),
            name: Some(name.into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: parent_id.map(String::from),
            backend_dom_node_id: Some(backend_id),
        }
    }

    /// A generic, unnamed container — stands in for a `<form>`/wrapping `<div>`.
    fn container(id: &str) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("generic".into()),
            name: None,
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
    fn picks_hinted_submit_button() {
        let tree = AXTree::from_nodes(vec![required_textbox("1"), button("2", "Absenden")]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
        assert_eq!(
            a.journey_candidates[0].required_journey,
            JourneyKind::FormErrorSubmit
        );
    }

    #[test]
    fn no_candidate_without_hint_match_no_first_button_fallback() {
        // A button with no submit-hinting name must not be picked by any
        // fallback — the removed `buttons.first()` fallback is the exact
        // hazard this guards against (an arbitrary button, e.g. a nav
        // burger, must never become a synthetic-click target).
        let tree = AXTree::from_nodes(vec![required_textbox("1"), button("2", "Menu")]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.journey_candidates.is_empty());
    }

    #[test]
    fn purchase_final_button_never_selected_even_as_only_button() {
        for name in [
            "Jetzt kaufen",
            "Zahlungspflichtig bestellen",
            "Buy now",
            "Place order",
            "Book now",
        ] {
            let tree = AXTree::from_nodes(vec![required_textbox("1"), button("2", name)]);
            let mut a = PatternAnalysis::default();
            detect(&tree, &mut a);
            assert!(
                a.journey_candidates.is_empty(),
                "purchase-final button {name:?} must never be selected as a trigger"
            );
        }
    }

    #[test]
    fn purchase_final_button_excluded_even_when_a_safe_button_also_present() {
        // The deny-list must disqualify the purchase button from the
        // candidate pool entirely, not just from being *preferred* — so a
        // safe hinted button elsewhere on the page is still correctly
        // picked instead.
        let tree = AXTree::from_nodes(vec![
            required_textbox("1"),
            button("2", "Jetzt kaufen"),
            button("3", "Suchen"),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
    }

    #[test]
    fn two_separate_forms_yield_two_distinct_candidates() {
        // Two unrelated forms (e.g. newsletter signup + site search) on the
        // same page must each get their own candidate, with the button that
        // is structurally nearest to each form's own required field — not
        // one arbitrary page-wide candidate.
        let tree = AXTree::from_nodes(vec![
            container("form-a"),
            required_textbox_in("f1", Some("form-a")),
            button_full("b1", "Absenden", Some("form-a"), 1),
            container("form-b"),
            required_textbox_in("f2", Some("form-b")),
            button_full("b2", "Suchen", Some("form-b"), 2),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 2);
        let backend_ids: std::collections::BTreeSet<_> = a
            .journey_candidates
            .iter()
            .map(|c| c.trigger_backend_id)
            .collect();
        assert_eq!(
            backend_ids,
            std::collections::BTreeSet::from([Some(1), Some(2)]),
            "each form must resolve to its own nearby button, not a shared/wrong one"
        );
    }

    #[test]
    fn candidates_capped_at_three_even_with_more_forms() {
        let mut nodes = Vec::new();
        for i in 1..=4 {
            let form_id = format!("form-{i}");
            let field_id = format!("f{i}");
            let button_id = format!("b{i}");
            nodes.push(container(&form_id));
            nodes.push(required_textbox_in(&field_id, Some(&form_id)));
            nodes.push(button_full(&button_id, "Absenden", Some(&form_id), i));
        }
        let tree = AXTree::from_nodes(nodes);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(
            a.journey_candidates.len(),
            MAX_FORM_CANDIDATES,
            "must cap at MAX_FORM_CANDIDATES even when more distinct forms exist"
        );
    }
}
