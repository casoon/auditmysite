//! Quantity-stepper pattern detector — identifies a spinbutton-style
//! quantity control (PDP/cart line item) as a candidate for the
//! keyboard-operability + value-exposure journey (SC 2.1.1, 4.1.2).
//!
//! Only ARIA-exposed `role="spinbutton"` controls are detected (this
//! includes native `<input type="number">`, which Chrome maps to
//! `spinbutton`). The alternative "separate +/- button pair with a plain
//! text value display, no single focusable/valued element" implementation
//! is NOT detected here — there is no single element to focus/read a value
//! from, and building a two-button-plus-display pattern matcher was judged
//! out of scope for this slice; see `a11y_journey::quantity_stepper`'s
//! module doc for the same caveat from the journey side.
//!
//! Only the first spinbutton found is used, even on pages with several
//! quantity fields (e.g. multiple cart line items) — one representative
//! instance is tested, consistent with how this codebase's other
//! pattern-based journeys (modal, tabs, disclosure) each test one instance
//! rather than every occurrence.

use crate::accessibility::AXTree;

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let Some(stepper) = tree
        .iter()
        .find(|n| n.role.as_deref() == Some("spinbutton"))
    else {
        return;
    };

    out.add_recognized(
        "QuantityStepper",
        format!(
            "Spinbutton quantity control detected: {}",
            stepper.name.as_deref().unwrap_or("(unnamed)")
        ),
        PatternConfidence::Partial,
    );

    out.journey_candidates.push(JourneyCandidate {
        pattern_kind: PatternKind::QuantityStepper,
        trigger_backend_id: stepper.backend_dom_node_id,
        controlled_backend_id: None,
        confidence: 0.75,
        required_journey: JourneyKind::QuantityStepper,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: Some(9),
        }
    }

    #[test]
    fn detects_spinbutton() {
        let tree = AXTree::from_nodes(vec![node("1", "spinbutton", Some("Menge"))]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
        assert_eq!(
            a.journey_candidates[0].required_journey,
            JourneyKind::QuantityStepper
        );
    }

    #[test]
    fn no_candidate_without_spinbutton() {
        let tree = AXTree::from_nodes(vec![node("1", "textbox", Some("Email"))]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.journey_candidates.is_empty());
    }

    #[test]
    fn only_first_spinbutton_used_when_several_present() {
        let tree = AXTree::from_nodes(vec![
            node("1", "spinbutton", Some("Menge Artikel 1")),
            node("2", "spinbutton", Some("Menge Artikel 2")),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
    }
}
