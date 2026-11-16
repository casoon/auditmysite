//! Guards of `locales/{de,en}/pdf-sections.ftl` (#936): the same keys in both
//! languages, every key defined once, used by `en301549.rs`, `bik_guide.rs`,
//! `detail_modules/overview.rs`, `helpers.rs`, `cover.rs` or `appendix.rs` and
//! resolving without errors, no German in the English file, and no text of
//! the export scenarios that is a key or still holds a placeholder. None of
//! the messages agrees a noun with a count (the counts are a `(n)` suffix and
//! `finding(s)` stays fixed), so there is no plural test.

use super::annex_text_export::records;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use serde_json::Value;
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../locales/en/pdf-sections.ftl");
const DE: &str = include_str!("../../../locales/de/pdf-sections.ftl");
const SOURCES: [&str; 6] = [
    include_str!("en301549.rs"),
    include_str!("bik_guide.rs"),
    include_str!("detail_modules/overview.rs"),
    include_str!("helpers.rs"),
    include_str!("cover.rs"),
    include_str!("appendix.rs"),
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
    assert_eq!(en.len(), 28);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCES
                .iter()
                .any(|source| source.contains(&format!("\"{id}\""))),
            "{id} is not used in en301549.rs, bik_guide.rs, detail_modules/overview.rs, helpers.rs, cover.rs or appendix.rs"
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
        for name in ["count", "source", "chapters"] {
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

/// "BIK für Alle" is the name of the guide, not German wording.
#[test]
fn english_messages_have_no_german_characters() {
    let english = EN.replace("BIK für Alle", "BIK");
    assert!(!english.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
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
                !text.contains("pdf-sections-")
                    && !text.contains("batch-report-en301549")
                    && !text.contains("diagnosis-col-")
                    && !text.contains("pdf-sec-score-")
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}

/// The four self-check instructions: one per criterion in both languages, none
/// for any other criterion, and the English ones hold no German.
#[test]
fn self_check_instructions_follow_the_run_language() {
    use super::helpers::manual_recheck_instruction;
    for criterion in ["1.3.6", "1.3.1", "3.3.2", "4.1.3"] {
        let en = manual_recheck_instruction(criterion, true).expect("English instruction");
        let de = manual_recheck_instruction(criterion, false).expect("German instruction");
        assert!(en.starts_with("Self-check: "), "{criterion}: {en}");
        assert!(de.starts_with("Selbsttest: "), "{criterion}: {de}");
        assert!(!en.contains(['ä', 'ö', 'ü', 'ß']), "{criterion}: {en}");
    }
    assert!(manual_recheck_instruction("1.1.1", true).is_none());
    assert!(manual_recheck_instruction("", false).is_none());
}

/// A failed run says so on the cover, a partial one that it is provisional,
/// each in the run language.
#[test]
fn the_cover_phrase_follows_the_run_language() {
    use super::cover::cover_band_phrase;
    use crate::audit::verdict::Verdict;
    assert_eq!(
        cover_band_phrase(80, Verdict::Fail, true, true),
        "Not passed in the automated audit scope — action required · provisional"
    );
    assert_eq!(
        cover_band_phrase(80, Verdict::Fail, false, false),
        "Im automatischen Prüfumfang nicht bestanden — Handlungsbedarf"
    );
    assert!(cover_band_phrase(80, Verdict::Pass, true, false).ends_with(" · vorläufig"));
}
