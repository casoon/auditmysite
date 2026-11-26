//! Guards of `locales/{de,en}/batch-overview.ftl` (#1212): the same
//! keys in both languages, every key defined once, used by `glance.rs`,
//! `compare.rs`, `causes.rs`, `technical.rs`, `seo.rs`, `consistency.rs` or
//! `batch_report.rs` and resolving without errors, and no
//! German in the English file.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../../locales/en/batch-overview.ftl");
const DE: &str = include_str!("../../../../locales/de/batch-overview.ftl");
const SOURCES: [&str; 7] = [
    include_str!("glance.rs"),
    include_str!("compare.rs"),
    include_str!("causes.rs"),
    include_str!("technical.rs"),
    include_str!("seo.rs"),
    include_str!("consistency.rs"),
    include_str!("../batch_report.rs"),
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
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used by the batch report sections that read this file"
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
            "min",
            "max",
            "passed",
            "total",
            "severe",
            "types",
            "pages",
            "shared",
            "all",
            "specific",
            "batch_types",
            "shared_on",
        ] {
            args.set(name, FluentValue::from(2_i64));
        }
        for name in [
            "page", "reason", "module", "title", "severity", "text", "list", "reach", "command",
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
