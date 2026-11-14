//! Guards of `locales/{de,en}/single-report.ftl` (#936): the same keys in
//! both languages, every key defined once, used by `single_report.rs` and
//! resolving without errors, no German in the English file, and no text of
//! the export scenarios that is a key or still holds a placeholder.

use super::text_export::records;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use serde_json::Value;
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../../locales/en/single-report.ftl");
const DE: &str = include_str!("../../../../locales/de/single-report.ftl");
const SOURCE: &str = include_str!("../single_report.rs");

/// The `report.ftl` messages `single_report.rs` reuses.
const REUSED: [&str; 14] = [
    "batch-rating-title",
    "batch-rating-verdict-",
    "batch-rating-reasons",
    "batch-col-priority",
    "batch-col-occurrences",
    "label-effort",
    "module-accessibility",
    "module-security",
    "severity-critical",
    "severity-high",
    "severity-medium",
    "count-heuristic-warnings",
    "count-manual-points",
    "count-more-findings",
];

/// The message ids of a file (column-0 `id =` lines), each only once.
fn ids(ftl: &str) -> Vec<String> {
    let ids: Vec<String> = ftl
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with([' ', '#']))
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
    assert_eq!(en.len(), 97);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCE.contains(&format!("\"{id}\"")),
            "{id} is not used in single_report.rs"
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
        let mut args = FluentArgs::new();
        for name in [
            "part",
            "count",
            "warnings",
            "manual",
            "manual_only",
            "automated",
            "total",
            "completed",
            "attempted",
            "failed",
            "skipped",
            "desktop",
            "mobile",
            "combined",
        ] {
            args.set(name, FluentValue::from(2_i64));
        }
        for name in [
            "rating",
            "verdict",
            "warnings_text",
            "manual_text",
            "more",
            "budget",
            "reason",
            "occurrence_context",
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

/// Every string in a record, component JSON included.
fn strings<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::String(text) => out.push(text),
        Value::Array(items) => items.iter().for_each(|item| strings(item, out)),
        Value::Object(map) => map.values().for_each(|item| strings(item, out)),
        _ => {}
    }
}

#[test]
fn no_scenario_leaves_a_key_or_a_placeholder_in_a_text() {
    for record in records() {
        let mut texts = Vec::new();
        strings(&record, &mut texts);
        for text in texts {
            assert!(
                !text.contains("single-report-")
                    && !REUSED.iter().any(|key| text.contains(key))
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}
