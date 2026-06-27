//! Form pattern detector — identifies forms with required fields that are
//! candidates for the form-error-announcement journey.
//!
//! The journey needs a submit button backend-node-id. We look for the
//! first button with role="button" inside or adjacent to a form landmark,
//! preferring elements with name/description containing "submit" or "send".

use crate::accessibility::AXTree;

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

/// Submit-related name fragments (case-insensitive).
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
    "bestellen",
    "order",
    "buchen",
    "book",
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

    // Find the best submit button candidate.
    // Prefer one with a submit-hinting name; fall back to the first button.
    let buttons: Vec<_> = tree
        .iter()
        .filter(|n| matches!(n.role.as_deref(), Some("button")))
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
        .or_else(|| buttons.first())
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
