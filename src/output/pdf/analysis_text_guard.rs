//! Guards of `locales/{de,en}/pdf-analysis.ftl` (#936): the same keys in both
//! languages, every key defined once, used by `search_experience.rs`,
//! `root_causes.rs`, `analysis_card.rs`, `problem_profile.rs` or `wcag_coverage.rs` and resolving
//! without errors, no German in the English file, the plural agreement of the
//! structural findings tail, and no text of the export scenarios that is a
//! key or still holds a placeholder.

use super::analysis_text_export::records;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use serde_json::Value;
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../locales/en/pdf-analysis.ftl");
const DE: &str = include_str!("../../../locales/de/pdf-analysis.ftl");
const SOURCES: [&str; 5] = [
    include_str!("../search_experience.rs"),
    include_str!("root_causes.rs"),
    include_str!("analysis_card.rs"),
    include_str!("problem_profile.rs"),
    include_str!("wcag_coverage.rs"),
];

/// The messages of other files the modules reuse, with their text in both
/// languages: the label "Technical SEO".
const REUSED: [(&str, &str, &str); 1] = [("seo-score-card", "Technical SEO", "Technisches SEO")];

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
    assert_eq!(en.len(), 86);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN)
        .into_iter()
        .chain(REUSED.map(|(id, _, _)| id.to_string()))
    {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used in search_experience.rs, root_causes.rs, analysis_card.rs, problem_profile.rs or wcag_coverage.rs"
        );
    }
}

#[test]
fn reused_messages_say_the_same_in_both_languages() {
    for (id, en, de) in REUSED {
        assert_eq!(crate::i18n::text(id, true), en, "{id}");
        assert_eq!(crate::i18n::text(id, false), de, "{id}");
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
            "score",
            "words",
            "images",
            "names",
            "kind",
            "first",
            "second",
            "initial",
            "last",
            "tail",
            "several",
            "found",
            "total",
            "pct",
            "top",
            "title",
            "warnings",
            "manual",
            "warnings_text",
            "manual_text",
            "more",
            "leverage",
            "effort",
            "letter",
            "share",
            "recommendations",
            "automated",
            "list",
            "listed",
            "aaa",
            "selector",
            "priority",
            "role",
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

/// The tail of the "isolated" problem profile agrees with the number of
/// findings it names: no structural finding, one, several.
#[test]
fn structural_tail_agrees_with_its_count() {
    for (lang, none, one, several) in [
        (
            "en",
            "Isolated local optimizations without structural problems.",
            "Isolated local optimizations; 1 finding with wide reach or a structural fix is named under key risks.",
            "Isolated local optimizations; 2 findings with wide reach or a structural fix are named under key risks.",
        ),
        (
            "de",
            "Einzelne lokale Optimierungen ohne strukturelles Problem.",
            "Einzelne lokale Optimierungen; 1 Befund mit großer Reichweite oder struktureller Änderung wird unter den wichtigsten Risiken genannt.",
            "Einzelne lokale Optimierungen; 2 Befunde mit großer Reichweite oder struktureller Änderung werden unter den wichtigsten Risiken genannt.",
        ),
    ] {
        let i18n = crate::i18n::I18n::new(lang).unwrap();
        assert_eq!(i18n.count("pdf-analysis-profile-structural", 0), none);
        assert_eq!(i18n.count("pdf-analysis-profile-structural", 1), one);
        assert_eq!(i18n.count("pdf-analysis-profile-structural", 2), several);
    }
}

/// The verb of the profile sentences follows the number of critical modules:
/// one module "determines", several "determine".
#[test]
fn profile_verb_follows_the_module_count() {
    for (lang, key, one, several) in [
        (
            "en",
            "pdf-analysis-profile-concentrated-stable",
            "A determines the need for action while other technical areas remain stable.",
            "A determine the need for action while other technical areas remain stable.",
        ),
        (
            "de",
            "pdf-analysis-profile-concentrated-stable",
            "A bestimmt den Handlungsbedarf bei ansonsten stabilen technischen Bereichen.",
            "A bestimmen den Handlungsbedarf bei ansonsten stabilen technischen Bereichen.",
        ),
        (
            "en",
            "pdf-analysis-profile-concentrated-systemic",
            "A determines the need for action. A large portion of the accessibility findings is concentrated in a few recurring patterns.",
            "A determine the need for action. A large portion of the accessibility findings is concentrated in a few recurring patterns.",
        ),
        (
            "de",
            "pdf-analysis-profile-concentrated-systemic",
            "A bestimmt den Handlungsbedarf. Ein großer Teil der Accessibility-Befunde konzentriert sich auf wenige wiederkehrende Muster.",
            "A bestimmen den Handlungsbedarf. Ein großer Teil der Accessibility-Befunde konzentriert sich auf wenige wiederkehrende Muster.",
        ),
    ] {
        let i18n = crate::i18n::I18n::new(lang).unwrap();
        let text = |several: &str| {
            i18n.t_args(
                key,
                &[
                    ("names", "A".to_string()),
                    ("several", several.to_string()),
                ],
            )
        };
        assert_eq!(text("no"), one, "{lang} {key}");
        assert_eq!(text("yes"), several, "{lang} {key}");
    }
}

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
                !text.contains("pdf-analysis-")
                    && !text.contains("seo-score-card")
                    && !text.contains("single-report-matrix-col-finding")
                    && !text.contains("batch-col-")
                    && !text.contains("count-")
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}
