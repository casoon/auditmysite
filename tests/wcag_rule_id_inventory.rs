//! Canonical inventory of every `rule_id` the accessibility audit can put on
//! a `Violation` (#552). See `tests/common/rule_inventory.rs` for the full
//! rationale and scan implementation — this file only holds the assertions.

mod common;

use common::rule_inventory::canonical_rule_ids;

#[test]
fn wcag_rule_id_inventory_is_non_trivial() {
    let ids = canonical_rule_ids();

    // Sanity floor, not an exact pin: catches the scan silently returning
    // ~0 after a source layout change, without needing an update every time
    // a rule is added or removed. A manual audit in #552 found 116 distinct
    // ids.
    assert!(
        ids.len() > 100,
        "canonical rule-id inventory found only {} ids — scan is likely broken \
         (expected 100+, see #552)",
        ids.len()
    );

    println!("wcag_rule_id_inventory: {} distinct rule_ids", ids.len());
    for id in &ids {
        println!("  {id}");
    }
}

/// Spot-checks one id from each of the registration shapes, so a
/// scan regression that happens to still clear the sanity floor above
/// (e.g. one extractor silently breaking while another compensates) is
/// still caught.
#[test]
fn wcag_rule_id_inventory_contains_a_known_id_from_each_shape() {
    let ids = canonical_rule_ids();

    // `RuleMetadata`-driven — the common case, works regardless of whether
    // the rule is invoked as a tree rule, a DOM page rule, or (like
    // contrast.rs) wired directly into the pipeline.
    assert!(
        ids.contains("color-contrast"),
        "contrast.rs's CONTRAST_RULE"
    );
    assert!(ids.contains("image-alt"), "text_alternatives.rs");

    // The `SHARED_RULES` table in `wcag/shared.rs`.
    assert!(ids.contains("aria/role-invalid"), "shared.rs");
    assert!(ids.contains("aria/tabpanel-missing"), "shared.rs");

    // Pattern-detection modules (`src/patterns/`) built `Violation`s with a
    // literal rule id until #694; their checks are shared rules now.
    assert!(
        ids.contains("patterns/accordion-controls-missing"),
        "shared.rs (formerly patterns/accordion.rs)"
    );
    assert!(
        ids.contains("dialog/focusable-missing"),
        "shared.rs (formerly patterns/modal_dialog.rs)"
    );
}
