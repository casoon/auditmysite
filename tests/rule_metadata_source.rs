//! The static facts of a shared rule agree across their three homes (#1119):
//! `a11y_rules::Meta` (barrierlab), `wcag::shared::SHARED_RULES` and the
//! taxonomy entry. `Meta` carries a set of related criteria and no level, so
//! the criterion a finding is reported under and its level stay local; this
//! test makes sure they cannot drift from `Meta` or from each other unnoticed.
//!
//! A shared rule is reported under a criterion `Meta` lists. The exceptions
//! pinned below are the rules whose `Meta.wcag` is empty or whose taxonomy
//! entry carries no criterion on purpose.

use auditmysite::taxonomy::RuleLookup;
use auditmysite::wcag::shared::SHARED_RULES;

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
        if META_WITHOUT_CRITERIA.contains(&rule.id) {
            assert!(
                meta.wcag.is_empty(),
                "{}: Meta.wcag now lists {:?}, drop it from META_WITHOUT_CRITERIA and compare",
                rule.id,
                meta.wcag
            );
        } else {
            assert!(
                meta.wcag.contains(&rule.criterion),
                "{}: reported under {} but Meta.wcag is {:?}",
                rule.id,
                rule.criterion,
                meta.wcag
            );
        }
    }
}

#[test]
fn the_pinned_exceptions_name_shared_ids() {
    for id in META_WITHOUT_CRITERIA
        .iter()
        .chain(TAXONOMY_WITHOUT_CRITERION)
        .copied()
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
