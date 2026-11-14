//! Guards of `locales/{de,en}/batch-report.ftl` (#936): the same keys in both
//! languages, every key defined once, used by `interactive.rs`,
//! `portfolio.rs`, `appendix.rs` or `overview.rs` and resolving without
//! errors, no German in the English file, and no text of the export scenarios
//! that is a key or still holds a placeholder.

use super::section_text_export::records;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use serde_json::Value;
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../../locales/en/batch-report.ftl");
const DE: &str = include_str!("../../../../locales/de/batch-report.ftl");
const SOURCES: [&str; 4] = [
    include_str!("interactive.rs"),
    include_str!("portfolio.rs"),
    include_str!("appendix.rs"),
    include_str!("overview.rs"),
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
    assert_eq!(en.len(), 59);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used in interactive.rs, portfolio.rs, appendix.rs or overview.rs"
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
        for name in ["chapter", "title"] {
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

/// The count of product pages agrees with its noun and is not grouped, as
/// the plain `{}` it replaced.
#[test]
fn product_pages_count_selects_singular_and_plural() {
    let key = "batch-report-commerce-product-pages-audited";
    let en = crate::i18n::I18n::new("en").unwrap();
    let de = crate::i18n::I18n::new("de").unwrap();
    assert_eq!(en.count(key, 1), "1 product page audited");
    assert_eq!(en.count(key, 0), "0 product pages audited");
    assert_eq!(en.count(key, 1_234_567), "1234567 product pages audited");
    assert_eq!(de.count(key, 1), "1 Produktseite geprüft");
    assert_eq!(de.count(key, 2), "2 Produktseiten geprüft");
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
                !text.contains("batch-report-")
                    && !text.contains("batch-col-")
                    && !text.contains("batch-interactive-")
                    && !text.contains("a11y-details-")
                    && !text.contains("pdf-commerce-")
                    && !text.contains("severity-")
                    && !text.starts_with("module-")
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}
