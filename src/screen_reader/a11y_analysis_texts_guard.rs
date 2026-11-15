//! Guards of `locales/{de,en}/a11y-analysis.ftl` (#936): the same keys in both
//! languages, every key defined once, used and resolving without errors, and
//! no German in the English file.

use super::a11y_analysis_texts_export::{a11y_analysis_texts, pattern_messages};
use crate::patterns::pattern_message_text;
use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../locales/en/a11y-analysis.ftl");
const DE: &str = include_str!("../../locales/de/a11y-analysis.ftl");
const SOURCES: [&str; 3] = [
    include_str!("analyzer.rs"),
    include_str!("../patterns/mod.rs"),
    include_str!("../patterns/easy_language.rs"),
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
    assert_eq!(en.len(), COUNT);
}

const COUNT: usize = 47;

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used in analyzer.rs, patterns/mod.rs or patterns/easy_language.rs"
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
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("count", FluentValue::from(2_i64));
        for name in [
            "name",
            "half",
            "quality",
            "start",
            "entries",
            "level",
            "seq",
            "landmarks",
            "unnamed",
            "native",
            "triggers",
            "controls",
            "dialogs",
            "well_formed",
            "tablists",
            "tabs",
            "selected",
            "non_button",
            "fields",
            "trigger",
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

/// A text that is a key or still holds a placeholder.
fn leaks(text: &str) -> bool {
    text.starts_with("a11y-analysis-") || text.contains("{$") || text.contains("{ $")
}

#[test]
fn no_input_leaves_a_key_or_a_placeholder_in_a_message() {
    for en in [true, false] {
        for record in a11y_analysis_texts(en) {
            let text = record
                .as_array()
                .and_then(|fields| fields.last())
                .and_then(|text| text.as_str())
                .expect("a record ends in its text");
            assert!(!leaks(text), "(en={en}): {text}");
        }
        for (label, message) in pattern_messages() {
            let text = pattern_message_text(&message, en);
            assert!(!leaks(&text), "{label} (en={en}): {text}");
        }
    }
}

/// EN text has no German characters, DE text is translated (every pattern
/// message and every issue message differs between the languages).
#[test]
fn the_two_languages_differ_and_english_has_no_german_characters() {
    let en = a11y_analysis_texts(true);
    let de = a11y_analysis_texts(false);
    assert_eq!(en.len(), de.len());
    for (en, de) in en.iter().zip(&de) {
        let (en, de) = (en.as_array().unwrap(), de.as_array().unwrap());
        let last = en.len() - 1;
        assert_eq!(en[..last], de[..last], "the records do not line up");
        let (en, de) = (en[last].as_str().unwrap(), de[last].as_str().unwrap());
        assert!(
            !en.chars().any(|c| "äöüÄÖÜß„“".contains(c)),
            "German character in {en}"
        );
        assert_ne!(en, de, "not translated: {en}");
    }
}
