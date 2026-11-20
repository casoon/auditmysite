//! Guards of `locales/{de,en}/output-texts.ftl` (#936): the same keys in both
//! languages, every key defined once, used by the builder helpers, the batch
//! builder, the display texts or the indicator, SEO and performance detail
//! modules and resolving without errors, no German in the English file, and
//! no text of the export scenarios that is a key or still holds a
//! placeholder.

use super::module_text_export::records;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use serde_json::Value;
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../../locales/en/output-texts.ftl");
const DE: &str = include_str!("../../../../locales/de/output-texts.ftl");
const SOURCES: [&str; 6] = [
    include_str!("../../builder/helpers.rs"),
    include_str!("../../builder/batch.rs"),
    include_str!("../../../display/mod.rs"),
    include_str!("indicators.rs"),
    include_str!("performance.rs"),
    include_str!("seo.rs"),
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
    assert_eq!(en.len(), 45);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used by the builder helpers, the batch builder, the display texts \
             or the indicator, SEO and performance detail modules"
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
        args.set("count", FluentValue::from(2_i64));
        for name in [
            "pages", "total", "distinct", "loads", "mode", "modes", "active", "score", "basis",
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

/// The schema count agrees with its noun, as the four-way match it replaced.
#[test]
fn schema_count_selects_singular_and_plural() {
    let key = "output-seo-schema-count";
    let en = crate::i18n::I18n::new("en").unwrap();
    let de = crate::i18n::I18n::new("de").unwrap();
    assert_eq!(en.count(key, 0), "0 schemas");
    assert_eq!(en.count(key, 1), "1 schema");
    assert_eq!(en.count(key, 2), "2 schemas");
    assert_eq!(de.count(key, 0), "0 Schemas");
    assert_eq!(de.count(key, 1), "1 Schema");
    assert_eq!(de.count(key, 2), "2 Schemas");
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
                !text.contains("output-")
                    && !text.contains("col-metric")
                    && !text.contains("col-value")
                    && text != "yes"
                    && text != "no"
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}

/// The legacy-script rows of the performance chapter were German literals in
/// every language ("Legacy-/Polyfill-Skripte", "Signatur").
#[test]
fn legacy_script_rows_follow_the_run_language() {
    for (lang, row, column, leaked) in [
        (
            "en",
            "Legacy / polyfill scripts",
            "Signature",
            ["Legacy-/Polyfill-Skripte", "Signatur"],
        ),
        (
            "de",
            "Legacy-/Polyfill-Skripte",
            "Signatur",
            ["Legacy / polyfill scripts", "Signature"],
        ),
    ] {
        let record = records()
            .into_iter()
            .find(|r| {
                r[0] == "render_performance"
                    && r[1] == "minification and legacy scripts"
                    && r[2] == lang
            })
            .expect("the export has the legacy scenario");
        let mut texts = Vec::new();
        strings(&record, &mut texts);
        assert!(texts.contains(&row), "{lang}: {row} missing");
        assert!(texts.contains(&column), "{lang}: {column} missing");
        for leaked in leaked {
            assert!(
                !texts.contains(&leaked),
                "{lang}: {leaked} left in the table"
            );
        }
    }
}

/// "1 technologies detected." and "1 Technologien erkannt.": the count reaches
/// the message as a number, so its plural select can choose the singular.
#[test]
fn tech_stack_takeaway_agrees_with_the_count() {
    for (lang, one, three) in [
        ("en", "1 technology detected.", "3 technologies detected."),
        ("de", "1 Technologie erkannt.", "3 Technologien erkannt."),
    ] {
        for (scenario, expected) in [("one first=true", one), ("three first=true", three)] {
            let record = records()
                .into_iter()
                .find(|r| r[0] == "render_tech_stack" && r[1] == scenario && r[2] == lang)
                .expect("the export has the scenario");
            let mut texts = Vec::new();
            strings(&record, &mut texts);
            assert!(
                texts.iter().any(|text| text.contains(expected)),
                "{lang} {scenario}: {expected} missing in {record}"
            );
        }
    }
}
