//! One decision per best-practice rule (#919). Browser-free, runs in every
//! `cargo test`.
//!
//! `taxonomy::BEST_PRACTICE_DECISIONS` holds one row per rule that carries the
//! `best-practice` keyword (plus `names/title-only`, a best practice by
//! decision without the keyword): the class it reports today, where its
//! detection lives and whether it is a convention rule (`is_convention_rule`).
//! These tests keep that table in step with the rules that carry the keyword
//! and generate `docs/reference/best-practice-rules.md` from it.
//!
//! After changing the table:
//! `UPDATE_BEST_PRACTICE_DOCS=1 cargo test --test best_practice_decisions`

mod common;

use auditmysite::taxonomy::{
    criterion_for_rule, is_convention_rule, RuleLookup, Source, BEST_PRACTICE_DECISIONS,
};
use auditmysite::wcag::shared::SHARED_RULES;
use common::rule_docs::repo_root;
use common::rule_inventory::{canonical_rule_ids, own_best_practice_rule_ids};
use std::collections::BTreeSet;

const DOC: &str = "docs/reference/best-practice-rules.md";

/// Rule ids of the shared library that carry the `best-practice` keyword: its
/// own list of recommendations plus the display-mode convention (`display/*`,
/// `viz/*`), which `a11y-rules` tags by keyword and anchors to a criterion.
fn library_tagged_ids() -> BTreeSet<String> {
    let mut ids: BTreeSet<String> = a11y_rules::BEST_PRACTICE_RULES
        .iter()
        .map(|r| r.id.to_string())
        .collect();
    ids.extend(
        SHARED_RULES
            .iter()
            .map(|r| r.id)
            .filter(|id| id.starts_with("display/") || id.starts_with("viz/"))
            .map(String::from),
    );
    ids
}

fn tagged_ids() -> BTreeSet<String> {
    let mut ids = library_tagged_ids();
    ids.extend(own_best_practice_rule_ids());
    ids
}

#[test]
fn every_best_practice_tagged_rule_has_a_decision_row() {
    let missing: Vec<String> = tagged_ids()
        .into_iter()
        .filter(|id| !BEST_PRACTICE_DECISIONS.iter().any(|d| d.id == id))
        .collect();
    assert!(
        missing.is_empty(),
        "best-practice tagged rules without a row in src/taxonomy/best_practice.rs: {missing:#?}"
    );
}

#[test]
fn every_decision_row_names_a_tagged_rule() {
    let tagged = tagged_ids();
    let rows: BTreeSet<&str> = BEST_PRACTICE_DECISIONS.iter().map(|d| d.id).collect();
    assert_eq!(rows.len(), BEST_PRACTICE_DECISIONS.len(), "duplicate row");
    let sorted: Vec<&str> = BEST_PRACTICE_DECISIONS.iter().map(|d| d.id).collect();
    assert_eq!(sorted, rows.iter().copied().collect::<Vec<_>>(), "order");
    for row in BEST_PRACTICE_DECISIONS {
        assert_eq!(
            row.keyword,
            tagged.contains(row.id),
            "{}: `keyword` says {}, but the rule {} the best-practice tag",
            row.id,
            row.keyword,
            if tagged.contains(row.id) {
                "carries"
            } else {
                "does not carry"
            }
        );
        assert!(
            RuleLookup::by_id(row.id).is_some(),
            "{} has no taxonomy entry",
            row.id
        );
    }
}

#[test]
fn source_column_follows_where_the_detection_lives() {
    let shared: BTreeSet<&str> = SHARED_RULES.iter().map(|r| r.id).collect();
    for row in BEST_PRACTICE_DECISIONS {
        let expected = if shared.contains(row.id) {
            Source::Barrierlab
        } else {
            Source::Local
        };
        assert_eq!(row.source, expected, "{}", row.id);
    }
}

#[test]
fn only_the_librarys_own_list_takes_its_reason_from_the_library() {
    for row in BEST_PRACTICE_DECISIONS {
        let listed = a11y_rules::best_practice_rule(row.id).is_some();
        assert_eq!(
            row.reason.is_none(),
            listed,
            "{}: a reason in the table exactly for rules outside the library's list",
            row.id
        );
    }
}

/// #919 changes no class: the convention column reproduces the former
/// definition (`display/*`, `viz/*` and three named ids) for every id the
/// engine can report and for every row.
#[test]
fn convention_rule_is_unchanged_for_every_rule_id() {
    fn before(id: &str) -> bool {
        id.starts_with("display/")
            || id.starts_with("viz/")
            || [
                "names/title-only",
                "description-duplicates-name",
                "zoom/viewport-scale-limited",
            ]
            .contains(&id)
    }
    let mut ids = canonical_rule_ids();
    ids.extend(BEST_PRACTICE_DECISIONS.iter().map(|d| d.id.to_string()));
    for id in &ids {
        assert_eq!(is_convention_rule(id), before(id), "{id}");
    }
}

/// The criterion the finding reports: the shared rule's own, else the
/// taxonomy entry's.
fn criterion(id: &str) -> String {
    SHARED_RULES
        .iter()
        .find(|r| r.id == id)
        .map(|r| r.criterion.to_string())
        .or_else(|| criterion_for_rule(id))
        .unwrap_or_else(|| "-".into())
}

fn render() -> String {
    let mut out = String::from(
        "---\ntitle: \"Best-practice rules\"\ndescription: \"Every best-practice rule with its outcome class, source and reason.\"\norder: 5\n---\n\n\
         <!-- Generated by tests/best_practice_decisions.rs from src/taxonomy/best_practice.rs. \
         Do not edit; regenerate with \
         `UPDATE_BEST_PRACTICE_DOCS=1 cargo test --test best_practice_decisions`. -->\n\n\
         A best-practice rule is anchored to the nearest WCAG success criterion without proving \
         a breach of it. This page lists every such rule with what it reports today. It is the \
         single source behind `is_convention_rule` (#919).\n\n\
         - **Outcome:** `fail` is a confirmed violation and counts in the score; `review` is a \
         hint that needs a person. `fail + review` means both occur depending on the case.\n\
         - **Source:** `barrierlab` rules live in the shared library (`a11y-rules`) and change \
         there first; `local` rules need the running browser or the AX tree and live in this \
         repository.\n\
         - **Convention:** the finding is never BFSG-relevant and raises no legal flag (#704).\n\
         - **Keyword:** the finding carries the `best-practice` tag. `names/title-only` is the one \
         row without it.\n\n\
         This page records the current classification; it does not change one.\n\n\
         | Rule | Criterion | Outcome | Source | Convention | Keyword | Reason |\n\
         | --- | --- | --- | --- | --- | --- | --- |\n",
    );
    for row in BEST_PRACTICE_DECISIONS {
        let reason = row
            .reason
            .or_else(|| a11y_rules::best_practice_rule(row.id).map(|r| r.reason))
            .expect("reason");
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |\n",
            row.id,
            criterion(row.id),
            row.outcome.label(),
            row.source.label(),
            if row.convention { "yes" } else { "no" },
            if row.keyword { "yes" } else { "no" },
            reason.replace('|', "\\|"),
        ));
    }
    out
}

#[test]
fn generated_best_practice_page_is_up_to_date() {
    let path = repo_root().join(DOC);
    let expected = render();
    if std::env::var_os("UPDATE_BEST_PRACTICE_DOCS").is_some() {
        std::fs::write(&path, &expected).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == expected,
        "{DOC} does not match src/taxonomy/best_practice.rs; regenerate with \
         `UPDATE_BEST_PRACTICE_DOCS=1 cargo test --test best_practice_decisions`"
    );
}
