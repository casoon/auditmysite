//! Guards of `locales/{de,en}/audit-interpretation.ftl` (#936, slice 27): the
//! same keys in both languages, every key defined once, used and resolving
//! without errors, no German in the English file, no key or placeholder in
//! any exported text, and counts that agree with their noun.

use super::interpretation_texts_export::interpretation_texts;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../locales/en/audit-interpretation.ftl");
const DE: &str = include_str!("../../locales/de/audit-interpretation.ftl");
const SOURCES: [&str; 9] = [
    include_str!("interpretation.rs"),
    include_str!("performance_interpretation.rs"),
    include_str!("verdict.rs"),
    include_str!("finding_derive.rs"),
    include_str!("summary.rs"),
    include_str!("baseline.rs"),
    include_str!("../seo/interpretation.rs"),
    include_str!("../seo/headings.rs"),
    include_str!("../taxonomy/score_area.rs"),
];

/// The message ids of a file (column-0 `id =` lines), each only once.
fn ids(ftl: &str) -> Vec<String> {
    let ids: Vec<String> = ftl
        .lines()
        .filter(|line| !line.starts_with([' ', '#']))
        .filter_map(|line| line.split_once(" =").map(|(id, _)| id.to_string()))
        .collect();
    let unique: BTreeSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "a message is defined twice");
    ids
}

#[test]
fn de_and_en_define_the_same_messages() {
    let de: BTreeSet<_> = ids(DE).into_iter().collect();
    let en: BTreeSet<_> = ids(EN).into_iter().collect();
    assert_eq!(de, en);
    assert_eq!(en.len(), 159);
    assert!(en.iter().all(|id| id.starts_with("audit-interpretation-")));
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used by the interpretation layer"
        );
    }
}

#[test]
fn every_key_resolves_in_both_languages_without_errors() {
    for (locale, ftl) in [("en", EN), ("de", DE)] {
        let resource = FluentResource::try_new(ftl.to_string()).expect("parses");
        let mut bundle = FluentBundle::new(vec![locale.parse().unwrap()]);
        bundle.set_use_isolating(false);
        bundle.add_resource(resource).expect("no duplicates");
        // `audit-join-and` is not part of this file; no message here uses it.
        let mut args = FluentArgs::new();
        for name in ["count", "urgent"] {
            args.set(name, FluentValue::from(2_i64));
        }
        for name in [
            "page_type",
            "depth",
            "quality",
            "traits",
            "nodes",
            "ms",
            "kb",
            "phrases",
            "base",
            "reasons",
            "automated",
            "all",
            "version",
            "rules",
            "total",
            "critical",
            "high",
            "score",
            "present",
            "dom",
            "perf",
            "threshold",
            "level",
            "title",
            "share",
            "groups",
            "occurrences",
            "detail",
            "cross",
            "length",
            "excerpt",
            "known",
            "new",
            "fixed",
        ] {
            args.set(name, FluentValue::from("x"));
        }
        for id in ids(EN) {
            let message = bundle.get_message(&id).expect("defined");
            let pattern = message.value().expect("has a value");
            let mut errors = vec![];
            let text = bundle.format_pattern(pattern, Some(&args), &mut errors);
            assert!(errors.is_empty(), "{locale} {id}: {errors:?}");
            assert!(!text.is_empty(), "{locale} {id} is empty");
        }
    }
    for id in ids(EN) {
        for en in [true, false] {
            assert_ne!(crate::i18n::text(&id, en), id, "{id} falls back to its key");
        }
    }
}

#[test]
fn english_messages_have_no_german_characters() {
    assert!(!EN.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
}

/// The English score area messages are the canonical labels of the JSON.
#[test]
fn english_score_area_labels_are_the_canonical_ones() {
    for area in crate::taxonomy::ScoreArea::ALL {
        assert_eq!(area.localized_label(true), area.label());
    }
}

/// A text that is a key or still holds a placeholder.
fn leaks(text: &str) -> bool {
    text.starts_with("audit-") || text.starts_with("explanation-") || text.contains("{$")
}

/// Every string of a JSON value, keys excluded.
fn strings(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(text) => out.push(text.clone()),
        serde_json::Value::Array(items) => items.iter().for_each(|item| strings(item, out)),
        serde_json::Value::Object(map) => map.values().for_each(|item| strings(item, out)),
        _ => {}
    }
}

#[test]
fn no_input_leaves_a_key_or_a_placeholder_in_a_text() {
    let mut texts = Vec::new();
    for record in interpretation_texts() {
        // [name, input label, language, output]: only the output is a text.
        strings(&record[3], &mut texts);
    }
    assert!(!texts.is_empty());
    for text in texts {
        assert!(!leaks(&text), "{text}");
    }
}

/// A count agrees with its noun in both languages: one, many and none.
#[test]
fn counts_agree_with_their_noun() {
    use crate::i18n::count_text;
    for (key, en, de) in [
        (
            "audit-interpretation-verdict-legal-flags",
            ["1 finding with", "2 findings with", "0 findings with"],
            ["1 Befund mit", "2 Befunde mit", "0 Befunde mit"],
        ),
        (
            "audit-interpretation-verdict-technical-blockers",
            [
                "1 technical blocker (",
                "2 technical blockers",
                "0 technical blockers",
            ],
            [
                "1 technischer Blocker (",
                "2 technische Blocker",
                "0 technische Blocker",
            ],
        ),
        (
            "audit-interpretation-verdict-findings-present",
            ["1 open finding", "2 open findings", "0 open findings"],
            ["1 offener Befund", "2 offene Befunde", "0 offene Befunde"],
        ),
        (
            "audit-interpretation-summary-groups",
            ["1 finding group", "2 finding groups", "0 finding groups"],
            ["1 Problemgruppe", "2 Problemgruppen", "0 Problemgruppen"],
        ),
    ] {
        for (n, (en_text, de_text)) in [1, 2, 0].into_iter().zip(en.into_iter().zip(de)) {
            assert!(count_text(key, n, true).starts_with(en_text), "{key} {n}");
            assert!(count_text(key, n, false).starts_with(de_text), "{key} {n}");
        }
    }
    // The noun of the finding sentences follows the count; the one-form of the
    // two sentences that print a verb keeps the wording of the source.
    assert!(
        count_text("audit-interpretation-complexity-high", 1, true).starts_with("1 occurrence ")
    );
    assert!(
        count_text("audit-interpretation-complexity-high", 2, true).starts_with("2 occurrences ")
    );
    assert!(
        count_text("audit-interpretation-expected-impact-other", 1, true)
            .starts_with("Addresses 1 occurrence;")
    );
    assert!(
        count_text("audit-interpretation-expected-impact-other", 5, true)
            .starts_with("Addresses 5 occurrences;")
    );
}
