//! Form pattern detector — identifies forms with required fields that are
//! candidates for the form-error-announcement journey.
//!
//! The journey needs a submit button backend-node-id. We look for the
//! first button with role="button" inside or adjacent to a form landmark,
//! preferring elements with name/description containing "submit" or "send".

use crate::accessibility::AXTree;

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

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

/// Purchase-final / booking button names — never used as a synthetic-click
/// trigger, even as a fallback. Checked against every candidate button, not
/// just the chosen one, so these can never be selected by any path.
const PURCHASE_FINAL_HINTS: &[&str] = &[
    "zahlungspflichtig bestellen",
    "jetzt kaufen",
    "kaufen",
    "bestellen",
    "buy now",
    "place order",
    "complete purchase",
    "complete order",
    "pay",
    "checkout",
    "buchen",
    "book now",
    "reservieren",
    "reserve",
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

    // Find the best submit button candidate. Purchase-final/booking buttons
    // are excluded from consideration entirely (never a synthetic-click
    // trigger, see `PURCHASE_FINAL_HINTS`) — no fallback to "the first
    // button" either, since that button could be exactly such a trigger.
    // No hint match means no candidate for this page, not a guess.
    let buttons: Vec<_> = tree
        .iter()
        .filter(|n| matches!(n.role.as_deref(), Some("button")))
        .filter(|n| {
            let name = n.name.as_deref().unwrap_or("").to_lowercase();
            !PURCHASE_FINAL_HINTS.iter().any(|h| name.contains(h))
        })
        .collect();

    if buttons.is_empty() {
        return;
    }

    let best = buttons
        .iter()
        .find(|b| {
            let name = b.name.as_deref().unwrap_or("").to_lowercase();
            SUBMIT_HINTS.iter().any(|h| name.contains(h))
        })
        .copied();

    let Some(trigger) = best else {
        return;
    };

    let backend_id = trigger.backend_dom_node_id;

    out.add_recognized(
        "Form",
        format!(
            "{} required {} detected; submit trigger: {}",
            required_controls.len(),
            if required_controls.len() == 1 {
                "field"
            } else {
                "fields"
            },
            trigger.name.as_deref().unwrap_or("(unnamed button)")
        ),
        PatternConfidence::Partial,
    );

    out.journey_candidates.push(JourneyCandidate {
        pattern_kind: PatternKind::Form,
        trigger_backend_id: backend_id,
        controlled_backend_id: None,
        confidence: 0.75,
        required_journey: JourneyKind::FormErrorSubmit,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn required_textbox(id: &str) -> AXNode {
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
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    fn button(id: &str, name: &str) -> AXNode {
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
            parent_id: None,
            backend_dom_node_id: Some(42),
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
}
