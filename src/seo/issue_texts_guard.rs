//! Guards of `locales/{de,en}/seo-issues.ftl` (#936): the same keys in both
//! languages, every key defined once, used and resolving without errors, and
//! no German in the English file. The file also holds the structured-data and
//! robots.txt table labels.

use super::issue_texts_export::{meta_values, schema_issues, technical_scenarios, META_KINDS};
use super::label_texts_export::{robots_audits, BOT_CLASSES, OTHER_POLICIES};
use super::meta::meta_issue_text;
use super::robots::{infer_robots_policy, localize_policy_label};
use super::schema::{schema_issue_label, schema_issue_text};
use super::technical::collect_technical_issues;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../locales/en/seo-issues.ftl");
const DE: &str = include_str!("../../locales/de/seo-issues.ftl");
const SOURCES: [&str; 4] = [
    include_str!("technical.rs"),
    include_str!("meta.rs"),
    include_str!("schema.rs"),
    include_str!("robots.rs"),
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
    assert_eq!(en.len(), 71);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used in technical.rs, meta.rs, schema.rs or robots.rs"
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
        for name in ["count", "words", "blockable", "length"] {
            args.set(name, FluentValue::from(2_i64));
        }
        for name in [
            "list",
            "samples",
            "primary",
            "alternative",
            "note",
            "value",
            "allowed",
            "type",
            "property",
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

/// A text that is a key or still holds a placeholder (the inputs of the
/// export contain `{$var}` and `{$y}` on purpose; they are taken out first).
fn leaks(text: &str) -> bool {
    let text = text.replace("{$var}", "").replace("{$y}", "");
    text.starts_with("seo-") || text.contains("{$")
}

#[test]
fn no_input_leaves_a_key_or_a_placeholder_in_a_message() {
    for en in [true, false] {
        for (label, technical) in technical_scenarios() {
            for issue in collect_technical_issues(&technical, en) {
                assert!(
                    !leaks(&issue.message),
                    "{label} (en={en}): {}",
                    issue.message
                );
            }
        }
        for kind in META_KINDS {
            for values in meta_values() {
                let text = meta_issue_text(kind, &values, en);
                assert!(!leaks(&text), "{kind:?} {values:?} (en={en}): {text}");
            }
        }
        for issue in schema_issues() {
            let text = schema_issue_text(&issue, en);
            assert!(!leaks(&text), "{issue:?} (en={en}): {text}");
            let label = schema_issue_label(&issue, en);
            assert!(!leaks(&label), "{issue:?} (en={en}): {label}");
        }
        for class in BOT_CLASSES {
            let label = class.label(en);
            assert!(!leaks(&label), "{class:?} (en={en}): {label}");
        }
        for (label, audit) in robots_audits() {
            let policy = infer_robots_policy(&audit, en);
            assert!(!leaks(&policy), "{label} (en={en}): {policy}");
            let stored = infer_robots_policy(&audit, true);
            let localized = localize_policy_label(&stored, en);
            assert!(!leaks(&localized), "{stored} (en={en}): {localized}");
            assert_eq!(localized, policy, "{label} (en={en})");
        }
        for stored in OTHER_POLICIES {
            assert_eq!(localize_policy_label(stored, en), stored);
        }
    }
}
