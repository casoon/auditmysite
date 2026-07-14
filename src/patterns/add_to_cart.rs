//! Add-to-cart pattern detector — identifies an "Add to Cart"-style trigger
//! as a candidate for the commerce add-to-cart feedback journey (SC 4.1.3).
//!
//! Only a button/link whose accessible name matches a specific add-to-cart
//! phrase is ever emitted — and never one that also matches
//! `PURCHASE_FINAL_HINTS` (e.g. a "Buy Now"/one-click-purchase button that
//! skips the cart). This journey only adds an item to the cart; it must
//! never risk triggering a one-click purchase.
//!
//! Whether the page is actually a shop product page is not decided here —
//! this module has no commerce context (`patterns::analyze()` runs before
//! the commerce module derives `page_kind`, see `audit/pipeline.rs`). The
//! journey layer gates the actual interaction on `ctx.commerce` +
//! `CommercePageKind::ProductDetail` at run time.

use crate::accessibility::AXTree;

use super::{
    JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind,
    PURCHASE_FINAL_HINTS,
};

/// Add-to-cart trigger name fragments (case-insensitive), DE/EN.
const ADD_TO_CART_HINTS: &[&str] = &[
    "in den warenkorb",
    "in den einkaufswagen",
    "in den korb",
    "zum warenkorb hinzufügen",
    "add to cart",
    "add to basket",
    "add to bag",
];

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let trigger = tree
        .iter()
        .filter(|n| matches!(n.role.as_deref(), Some("button") | Some("link")))
        .find(|n| {
            let name = n.name.as_deref().unwrap_or("").to_lowercase();
            ADD_TO_CART_HINTS.iter().any(|h| name.contains(h))
                && !PURCHASE_FINAL_HINTS.iter().any(|h| name.contains(h))
        });

    let Some(trigger) = trigger else {
        return;
    };

    out.add_recognized(
        "AddToCart",
        format!(
            "Add-to-cart trigger detected: {}",
            trigger.name.as_deref().unwrap_or("(unnamed button)")
        ),
        PatternConfidence::Partial,
    );

    out.journey_candidates.push(JourneyCandidate {
        pattern_kind: PatternKind::AddToCart,
        trigger_backend_id: trigger.backend_dom_node_id,
        controlled_backend_id: None,
        confidence: 0.75,
        required_journey: JourneyKind::AddToCart,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn node(id: &str, role: &str, name: &str) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: Some(name.into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: Some(7),
        }
    }

    #[test]
    fn detects_hinted_add_to_cart_button() {
        let tree = AXTree::from_nodes(vec![node("1", "button", "In den Warenkorb")]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
        assert_eq!(
            a.journey_candidates[0].required_journey,
            JourneyKind::AddToCart
        );
    }

    #[test]
    fn detects_english_add_to_cart_link() {
        let tree = AXTree::from_nodes(vec![node("1", "link", "Add to Cart")]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
    }

    #[test]
    fn no_candidate_without_hint_match() {
        let tree = AXTree::from_nodes(vec![node("1", "button", "Details ansehen")]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.journey_candidates.is_empty());
    }

    #[test]
    fn buy_now_button_never_selected_even_if_cart_adjacent() {
        // A "Buy Now" one-click-purchase button must never be picked, even
        // if a site phrases it in a way that loosely overlaps cart wording.
        for name in ["Jetzt kaufen", "Buy now", "In den Warenkorb & jetzt kaufen"] {
            let tree = AXTree::from_nodes(vec![node("1", "button", name)]);
            let mut a = PatternAnalysis::default();
            detect(&tree, &mut a);
            assert!(
                a.journey_candidates.is_empty(),
                "purchase-final button {name:?} must never be selected as an add-to-cart trigger"
            );
        }
    }
}
