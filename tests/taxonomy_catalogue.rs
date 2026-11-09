//! Mapping of the two rule catalogues (#918): engine id (`Violation::rule_id`,
//! e.g. `images/alt-missing`) against the taxonomy entry it resolves to
//! (`a11y.alt_text.missing`).
//!
//! This is the analysis that precedes the re-key. It records, per engine id,
//! how the taxonomy finds its metadata today — by the id itself, through the
//! WCAG criterion bucket, or not at all (`default_impact` fallback) — and the
//! taxonomy entries that no engine id reaches. Run it with
//! `cargo test --test taxonomy_catalogue -- --nocapture` to see the table.

mod common;

use auditmysite::taxonomy::RuleLookup;
use common::rule_inventory::canonical_rule_ids;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn mapping_report() {
    let engine = canonical_rule_ids();

    // engine id -> taxonomy entry id, `None` when the id has no entry of its
    // own and falls back to its criterion bucket (or to nothing).
    let mut own: BTreeMap<&str, &str> = BTreeMap::new();
    let mut without_entry: Vec<&str> = Vec::new();
    for id in &engine {
        match RuleLookup::by_legacy_wcag_id(id) {
            Some(rule) => {
                own.insert(id.as_str(), rule.id);
            }
            None => without_entry.push(id.as_str()),
        }
    }

    let mut shared_entry: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (engine_id, rule_id) in &own {
        shared_entry.entry(rule_id).or_default().push(engine_id);
    }
    shared_entry.retain(|_, ids| ids.len() > 1);

    let reached: BTreeSet<&str> = own.values().copied().collect();
    let unreached: Vec<&str> = auditmysite::taxonomy::rules::RULES
        .iter()
        .map(|r| r.id)
        .filter(|id| !reached.contains(id))
        .collect();

    println!("engine ids: {}", engine.len());
    println!("  with an entry of their own id: {}", own.len());
    println!(
        "  without (criterion bucket / default_impact): {}",
        without_entry.len()
    );
    for id in &without_entry {
        println!("    {id}");
    }
    println!(
        "taxonomy entries shared by several engine ids: {}",
        shared_entry.len()
    );
    for (rule, ids) in &shared_entry {
        println!("    {rule} <- {}", ids.join(", "));
    }
    println!(
        "taxonomy entries no engine id resolves to: {}",
        unreached.len()
    );
    for id in &unreached {
        println!("    {id}");
    }
}
