//! One rule catalogue (#918): the engine id a rule puts on its `Violation`
//! (`images/alt-missing`) is the key of its taxonomy entry. There is no second
//! id, no map between the two and no severity-based fallback score.

mod common;

use auditmysite::taxonomy::{criteria_with_impact, RuleLookup};
use common::rule_inventory::{canonical_rule_criteria, canonical_rule_ids};
use std::collections::BTreeSet;

/// Taxonomy entries that deliberately have no engine rule behind them. A
/// criterion that only a person can check would be listed here; today every
/// entry is reported by a rule (the manual-review hints are `manual/*` rules).
const MANUAL: &[&str] = &[];

#[test]
fn every_emitted_rule_id_has_taxonomy_metadata() {
    let missing: Vec<String> = canonical_rule_ids()
        .into_iter()
        .filter(|id| RuleLookup::by_id(id).is_none())
        .collect();
    assert!(
        missing.is_empty(),
        "engine rule ids without a taxonomy entry (add one to src/taxonomy/rules.rs): {missing:#?}"
    );
}

#[test]
fn every_taxonomy_rule_has_an_emitter() {
    let engine = canonical_rule_ids();
    let orphans: Vec<&str> = auditmysite::taxonomy::rules::RULES
        .iter()
        .map(|r| r.id)
        .filter(|id| !engine.contains(*id) && !MANUAL.contains(id))
        .collect();
    assert!(
        orphans.is_empty(),
        "taxonomy entries no engine rule reports (delete them, or list them in MANUAL): {orphans:#?}"
    );
}

/// The accessibility score charges per success criterion
/// (`taxonomy::criterion_impact`). Every criterion a rule reports needs a
/// cost row, and no row may outlive its criterion.
#[test]
fn criterion_cost_rows_match_the_criteria_rules_report() {
    let reported: BTreeSet<String> = canonical_rule_criteria().into_values().collect();
    let rows: BTreeSet<String> = criteria_with_impact().map(String::from).collect();
    let uncosted: Vec<&String> = reported.difference(&rows).collect();
    assert!(
        uncosted.is_empty(),
        "criteria reported by a rule but without a score cost row: {uncosted:?}"
    );
    let stale: Vec<&String> = rows.difference(&reported).collect();
    assert!(
        stale.is_empty(),
        "score cost rows for criteria no rule reports: {stale:?}"
    );
}
