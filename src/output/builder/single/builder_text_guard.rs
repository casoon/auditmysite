//! Guards of `locales/{de,en}/report-builder.ftl` (#936): the same keys in
//! both languages, every key defined once, used by the builder files or the
//! terminal table and resolving without errors, no German in the English
//! file, and no text of the export scenarios that is a key or still holds a
//! placeholder. None of the messages carries a count that agrees with its
//! noun ("finding(s)" is written out in both languages), so there is no plural
//! guard.

use super::builder_text_export::records;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use serde_json::Value;
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../../locales/en/report-builder.ftl");
const DE: &str = include_str!("../../../../locales/de/report-builder.ftl");

/// The files whose code calls the messages.
const SOURCES: [&str; 8] = [
    include_str!("executive.rs"),
    include_str!("positive.rs"),
    include_str!("serp.rs"),
    include_str!("mod.rs"),
    include_str!("modules_block.rs"),
    include_str!("module_details/performance.rs"),
    include_str!("module_details/platform.rs"),
    include_str!("../../terminal.rs"),
];

/// The `report.ftl` messages `positive.rs` reuses.
const REUSED: [&str; 2] = ["module-accessibility", "module-security"];

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
    assert_eq!(en.len(), 116);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN).iter().map(String::as_str).chain(REUSED) {
        let literal = format!("\"{id}\"");
        assert!(
            SOURCES.iter().any(|source| source.contains(&literal)),
            "{id} is not used by the report builder or the terminal table"
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
            "title",
            "share",
            "automated",
            "total",
            "areas",
            "view",
            "ms",
            "count",
            "affected",
            "modules",
            "indicators",
            "weights",
            "name",
            "pct",
            "days",
            "score",
            "numeric",
            "base",
            "context",
            "scope",
            "area",
            "label",
            "problems",
            "signals",
            "findings",
            "domain",
            "audited",
            "source",
            "failed",
            "url",
            "error",
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
    for id in ids(EN).iter().map(String::as_str).chain(REUSED) {
        for en in [true, false] {
            assert_ne!(crate::i18n::text(id, en), id, "{id} falls back to its key");
        }
    }
}

#[test]
fn english_messages_have_no_german_characters() {
    assert!(!EN.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
}

/// The batch cover date (`output::builder::batch`) takes its format from the
/// message the single report's date uses: `%Y-%m-%d` in English and
/// `%d.%m.%Y` in German, as the two branches it replaced.
#[test]
fn the_date_format_message_keeps_the_notation_of_each_language() {
    assert_eq!(crate::i18n::text("date-format-str", true), "%Y-%m-%d");
    assert_eq!(crate::i18n::text("date-format-str", false), "%d.%m.%Y");
}

/// Every string in a record.
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
    let all = records()
        .into_iter()
        .chain(crate::output::terminal::text_export::records());
    for record in all {
        let mut texts = Vec::new();
        strings(&record, &mut texts);
        for text in texts {
            assert!(
                !text.contains("report-builder-")
                    && !REUSED.iter().any(|key| text.contains(key))
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}
