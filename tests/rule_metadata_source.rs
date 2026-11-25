//! The static facts of a shared rule agree across their three homes (#1119):
//! `a11y_rules::Meta` (barrierlab), `wcag::shared::SHARED_RULES` and the
//! taxonomy entry. `Meta` carries a set of related criteria and no level, so
//! the criterion a finding is reported under and its level stay local; this
//! test makes sure they cannot drift from `Meta` or from each other unnoticed.
//!
//! Every deviation is pinned below together with the `Meta` value it deviates
//! from: when barrierlab changes the criteria of such a rule, the test fails
//! and the deviation has to be decided again.

use auditmysite::taxonomy::RuleLookup;
use auditmysite::wcag::shared::SHARED_RULES;

/// The criterion a finding is reported under is deliberately not one of the
/// criteria `Meta` lists: `(id, reported criterion, Meta.wcag)`. Reasons in
/// `docs/reference/shared-rule-criteria.md`.
const DEVIATES_FROM_META: &[(&str, &str, &[&str])] = &[
    ("frames/name-missing", "2.4.1", &["4.1.2"]),
    ("images/server-side-map", "1.1.1", &["2.1.1"]),
];

/// `Meta.wcag` is empty (best-practice rules), the criterion is auditmysite's
/// own assignment.
const META_WITHOUT_CRITERIA: &[&str] = &[
    "landmarks/not-top-level",
    "landmarks/banner-duplicate",
    "landmarks/contentinfo-duplicate",
    "landmarks/content-outside",
    "aria/role-redundant",
    "lists/role-redundant",
    "patterns/tooltip-unreferenced",
    "frames/cross-origin",
];

/// The taxonomy deliberately carries no criterion (best practice, #779) while
/// the finding is reported under one.
const TAXONOMY_WITHOUT_CRITERION: &[&str] = &["headings/h1-missing"];

#[test]
fn every_shared_id_is_declared_by_a11y_rules() {
    for rule in SHARED_RULES {
        assert!(
            a11y_rules::meta_for(rule.id).is_some(),
            "{} is in SHARED_RULES but not declared by a11y-rules",
            rule.id
        );
    }
}

#[test]
fn the_reported_criterion_is_one_of_the_criteria_of_meta() {
    for rule in SHARED_RULES {
        let Some(meta) = a11y_rules::meta_for(rule.id) else {
            continue;
        };
        let pinned_deviation = DEVIATES_FROM_META.iter().find(|(id, ..)| *id == rule.id);
        let pinned_empty = META_WITHOUT_CRITERIA.contains(&rule.id);
        match (pinned_deviation, pinned_empty) {
            (Some((_, criterion, wcag)), false) => {
                assert_eq!(rule.criterion, *criterion, "{}: pinned criterion", rule.id);
                assert_eq!(
                    meta.wcag, *wcag,
                    "{}: Meta.wcag changed, decide the deviation again",
                    rule.id
                );
                assert!(
                    !meta.wcag.contains(&rule.criterion),
                    "{}: no deviation any more, drop it from DEVIATES_FROM_META",
                    rule.id
                );
            }
            (None, true) => assert!(
                meta.wcag.is_empty(),
                "{}: Meta.wcag now lists {:?}, drop it from META_WITHOUT_CRITERIA and compare",
                rule.id,
                meta.wcag
            ),
            (None, false) => assert!(
                meta.wcag.contains(&rule.criterion),
                "{}: reported under {} but Meta.wcag is {:?}",
                rule.id,
                rule.criterion,
                meta.wcag
            ),
            (Some(_), true) => panic!("{}: pinned twice", rule.id),
        }
    }
}

#[test]
fn the_pinned_exceptions_name_shared_ids() {
    for id in META_WITHOUT_CRITERIA
        .iter()
        .chain(TAXONOMY_WITHOUT_CRITERION)
        .copied()
        .chain(DEVIATES_FROM_META.iter().map(|(id, ..)| *id))
    {
        assert!(
            SHARED_RULES.iter().any(|r| r.id == id),
            "{id} is pinned but not in SHARED_RULES"
        );
    }
}

#[test]
fn the_taxonomy_entry_agrees_with_the_shared_rule() {
    for rule in SHARED_RULES {
        let entry = RuleLookup::by_id(rule.id)
            .unwrap_or_else(|| panic!("{} has no taxonomy entry", rule.id));
        if TAXONOMY_WITHOUT_CRITERION.contains(&rule.id) {
            assert_eq!(entry.external_ref, None, "{}", rule.id);
            assert_eq!(entry.external_level, None, "{}", rule.id);
            continue;
        }
        assert_eq!(
            entry.external_ref,
            Some(format!("WCAG {}", rule.criterion).as_str()),
            "{}: taxonomy criterion differs from SHARED_RULES",
            rule.id
        );
        assert_eq!(
            entry.external_level,
            Some(format!("{:?}", rule.level).as_str()),
            "{}: taxonomy level differs from SHARED_RULES",
            rule.id
        );
    }
}
