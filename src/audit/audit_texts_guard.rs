//! Guards of `locales/{de,en}/audit-texts.ftl` (#936): the same keys in both
//! languages, every key defined once, used and resolving without errors, and
//! no German in the English file.

use super::audit_texts_export::{
    audit_texts, help_consistencies, help_deviations, risk_kinds, HEADING_ISSUE_TYPES, HELP_REGIONS,
};
use super::batch_consistency::{consistent_help_basis, help_deviation_text, help_region_name};
use super::normalized::seo_heading_finding_text;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../locales/en/audit-texts.ftl");
const DE: &str = include_str!("../../locales/de/audit-texts.ftl");
const SOURCES: [&str; 3] = [
    include_str!("normalized.rs"),
    include_str!("batch_consistency.rs"),
    include_str!("management_risk.rs"),
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
    assert_eq!(en.len(), 46);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used in normalized.rs, batch_consistency.rs or management_risk.rs"
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
            "mechanisms",
            "deviations",
            "pages",
            "mechanism",
            "found",
            "expected",
            "region",
            "first",
            "second",
            "critical",
            "high",
            "base",
            "controls",
            "consequence",
            "accessibility",
            "performance",
            "mobile",
            "score",
            "severe",
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
fn no_input_leaves_a_key_or_a_placeholder_in_a_message() {
    for en in [true, false] {
        let mut texts = Vec::new();
        for record in audit_texts(en) {
            strings(&record, &mut texts);
        }
        for text in texts {
            assert!(!leaks(&text), "(en={en}): {text}");
        }
        for issue_type in HEADING_ISSUE_TYPES {
            let (title, impact) = seo_heading_finding_text(issue_type, en, "fallback");
            assert!(!leaks(&title) && !leaks(&impact), "{issue_type} (en={en})");
        }
        for (label, help) in help_consistencies() {
            let text = consistent_help_basis(&help, en);
            assert!(!leaks(&text), "{label} (en={en}): {text}");
        }
        for region in HELP_REGIONS {
            assert!(
                !leaks(&help_region_name(region, en)),
                "{region:?} (en={en})"
            );
        }
        for (label, deviation) in help_deviations() {
            let text = help_deviation_text(&deviation, en);
            assert!(!leaks(&text), "{label} (en={en}): {text}");
        }
        for (label, kind) in risk_kinds() {
            for text in [kind.dimension(en), kind.rationale(en)] {
                assert!(!leaks(&text), "{label} (en={en}): {text}");
            }
        }
    }
}
