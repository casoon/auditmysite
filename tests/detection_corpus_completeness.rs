//! Completeness check: coverage gap between the canonical rule-id inventory
//! (#552) and the detection corpus (#553/#556) (#554).
//!
//! Browser-free, no Chrome needed — turns "systematic" from a one-time
//! intention into a standing, re-checkable number. Deliberately does *not*
//! fail on low coverage: the corpus starts empty and is populated
//! incrementally (#556), so a near-100% gap today is the expected backlog,
//! not a regression. What it *does* fail on is a real authoring bug: a
//! corpus case or `structurally_deferred.json` entry referencing a rule_id
//! that doesn't exist in the canonical inventory at all (a typo, or a rule
//! that was renamed/removed without updating the fixture).

mod common;

use common::detection_corpus::{
    detection_corpus_dir, load_corpus_dir, load_structurally_deferred, Verdict,
};
use common::rule_inventory::canonical_rule_ids;
use std::collections::BTreeSet;

/// A diagnostic coverage number is not a release gate. Deferred cases are
/// unresolved tests, not evidence that the corresponding rule works.
#[test]
#[ignore = "strict release gate: every active rule needs positive and negative evidence"]
fn release_rule_coverage_is_complete() {
    let canonical = canonical_rule_ids();
    let cases = load_corpus_dir(&detection_corpus_dir());
    assert!(!cases.is_empty(), "no detection corpus");
    let mut gaps = Vec::new();
    for id in canonical {
        let expectations: Vec<_> = cases
            .iter()
            .flat_map(|case| &case.expectations)
            .filter(|e| e.rule_id == id)
            .collect();
        let positive = expectations
            .iter()
            .any(|e| matches!(e.verdict, Verdict::Violation | Verdict::NeedsReview));
        let negative = expectations
            .iter()
            .any(|e| matches!(e.verdict, Verdict::Pass | Verdict::Absent));
        if !positive || !negative {
            gaps.push(format!("{id}: positive={positive}, negative={negative}"));
        }
    }
    assert!(
        gaps.is_empty(),
        "Unverified rule contracts block release:\n{}",
        gaps.join("\n")
    );
}

#[test]
fn detection_corpus_completeness_gap_report() {
    let canonical = canonical_rule_ids();
    let corpus_dir = detection_corpus_dir();
    let cases = load_corpus_dir(&corpus_dir);
    let deferred = load_structurally_deferred(&corpus_dir);

    let covered: BTreeSet<String> = cases
        .iter()
        .flat_map(|c| c.expectations.iter().map(|e| e.rule_id.clone()))
        .collect();
    let deferred_ids: BTreeSet<String> = deferred.iter().map(|d| d.rule_id.clone()).collect();

    // Authoring-bug guards: a rule_id in the corpus or deferred list that
    // isn't in the canonical inventory is a typo or a stale reference, not
    // a legitimate coverage gap.
    let unknown_in_corpus: Vec<&String> = covered.difference(&canonical).collect();
    assert!(
        unknown_in_corpus.is_empty(),
        "detection corpus references rule_id(s) not in the canonical inventory \
         (typo, or a rule that was renamed/removed?): {unknown_in_corpus:?}"
    );
    let unknown_in_deferred: Vec<&String> = deferred_ids.difference(&canonical).collect();
    assert!(
        unknown_in_deferred.is_empty(),
        "structurally_deferred.json references rule_id(s) not in the canonical inventory: \
         {unknown_in_deferred:?}"
    );

    // A rule can't legitimately be both covered by a real fixture and
    // deferred as untestable at the same time.
    let both: Vec<&String> = covered.intersection(&deferred_ids).collect();
    assert!(
        both.is_empty(),
        "rule_id(s) are both covered by a corpus case AND marked structurally_deferred \
         — pick one: {both:?}"
    );

    let gap: BTreeSet<&String> = canonical
        .iter()
        .filter(|id| !covered.contains(*id) && !deferred_ids.contains(*id))
        .collect();

    println!(
        "detection corpus completeness: {} covered / {} structurally_deferred / {} gap (of {} total rule_ids)",
        covered.len(),
        deferred_ids.len(),
        gap.len(),
        canonical.len()
    );
    if !gap.is_empty() {
        println!("uncovered rule_ids:");
        for id in &gap {
            println!("  {id}");
        }
    }
}
