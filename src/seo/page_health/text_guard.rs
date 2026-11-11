//! Guards of `locales/{de,en}/page-health.ftl` (#936): the same keys in both
//! languages, every key defined once, used and resolving without errors, and
//! no German in the English file.

use super::collect_issues;
use super::text_export::scenarios;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../locales/en/page-health.ftl");
const DE: &str = include_str!("../../../locales/de/page-health.ftl");
const SOURCE: &str = include_str!("issues.rs");

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
    assert!(en.len() > 50);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCE.contains(&format!("\"{id}\"")),
            "{id} is not used in issues.rs"
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
            "count",
            "total",
            "nodes",
            "depth",
            "delay",
            "length",
            "inefficient",
        ] {
            args.set(name, FluentValue::from(2_i64));
        }
        for name in [
            "sample", "url", "value", "header", "declared", "status", "list", "reason", "expected",
            "actual",
        ] {
            args.set(name, FluentValue::from("x"));
        }
        for id in ids(EN) {
            // The German text refers to a message of `report.ftl`; the bundle
            // of the crate resolves it (below), this one holds only this file.
            if id == "page-health-oversized-images" && locale == "de" {
                continue;
            }
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

#[test]
fn no_scenario_leaves_a_key_or_a_placeholder_in_a_message() {
    for (label, analysis) in scenarios() {
        for en in [true, false] {
            for issue in collect_issues(&analysis, en) {
                let message = issue.message.replace("{$var}", "");
                assert!(
                    !message.starts_with("page-health-") && !message.contains("{$"),
                    "{label} (en={en}): {}",
                    issue.message
                );
            }
        }
    }
}
