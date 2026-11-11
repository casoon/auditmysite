//! Guards of `locales/{de,en}/labels.ftl` (#936): the same keys in both
//! languages, every key defined once, used and resolving without errors, and
//! no German in the English file.

use super::detail_texts_export::{evidence_sources, EVIDENCE_FIELD_PATHS};
use super::label_texts_export::{
    EFFORTS, EXECUTION_PRIORITIES, ISSUE_CLASSES, PAGE_INTENTS, PRIORITIES, ROLES, SUBCATEGORIES,
};
use fluent_bundle::{FluentBundle, FluentResource};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../locales/en/labels.ftl");
const DE: &str = include_str!("../../locales/de/labels.ftl");
const SOURCES: [&str; 6] = [
    include_str!("../page_flow/page_intent.rs"),
    include_str!("../audit/prioritization.rs"),
    include_str!("../output/report_model.rs"),
    include_str!("../taxonomy/issue_class.rs"),
    include_str!("../taxonomy/dimensions.rs"),
    include_str!("../assessment/mod.rs"),
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
    assert_eq!(en.len(), 74);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used in page_intent.rs, prioritization.rs, report_model.rs, \
             issue_class.rs, dimensions.rs or assessment/mod.rs"
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
        for id in ids(EN) {
            let message = bundle.get_message(&id).expect("defined");
            let pattern = message.value().expect("has a value");
            let mut errors = vec![];
            let text = bundle.format_pattern(pattern, None, &mut errors);
            assert!(errors.is_empty(), "{locale} {id}: {errors:?}");
            assert!(!text.is_empty(), "{locale} {id} is empty");
        }
    }
    for id in ids(EN) {
        for en in [true, false] {
            assert_ne!(super::text(&id, en), id, "{id} falls back to its key");
        }
    }
}

#[test]
fn english_messages_have_no_german_characters() {
    assert!(!EN.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
}

/// A text that is a key or still holds a placeholder.
fn leaks(text: &str) -> bool {
    text.starts_with("labels-") || text.contains("{$") || text.contains("{ $")
}

#[test]
fn no_variant_leaves_a_key_or_a_placeholder_in_a_label() {
    for en in [true, false] {
        let mut labels: Vec<(String, String)> = Vec::new();
        labels.extend(PAGE_INTENTS.map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(PRIORITIES.map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(EFFORTS.map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(EXECUTION_PRIORITIES.map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(ROLES.map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(ISSUE_CLASSES.map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(ISSUE_CLASSES.map(|v| (format!("{v:?} display"), v.to_string())));
        labels.extend(SUBCATEGORIES.map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(evidence_sources().map(|v| (format!("{v:?}"), v.label(en))));
        labels.extend(EVIDENCE_FIELD_PATHS.into_iter().filter_map(|path| {
            crate::assessment::evidence_field_label(path, en)
                .map(|label| (path.to_string(), label.replace("{$var}", "")))
        }));
        for (variant, label) in labels {
            assert!(!leaks(&label), "{variant} (en={en}): {label}");
        }
    }
}
