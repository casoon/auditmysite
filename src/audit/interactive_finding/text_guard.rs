//! Guards of `locales/{de,en}/interactive.ftl` (#936): the same keys in both
//! languages, every key defined once, used and resolving without errors, and
//! no German in the English file.

use super::text_export::{values, KINDS, WAITED_MS};
use super::{finding_uncertainty_text, interactive_finding_text, FindingUncertainty};
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../locales/en/interactive.ftl");
const DE: &str = include_str!("../../../locales/de/interactive.ftl");
const SOURCE: &str = include_str!("../interactive_finding.rs");

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
    // A message and a fix message per finding kind, plus the uncertainty note.
    assert_eq!(en.len(), 2 * KINDS.len() + 1);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCE.contains(&format!("\"{id}\"")),
            "{id} is not used in interactive_finding.rs"
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
            "selector",
            "examples",
            "suffix",
            "title",
            "role",
            "waited_ms",
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
/// export contain `{$var}` on purpose; it is taken out first).
fn leaks(text: &str) -> bool {
    let text = text.replace("{$var}", "");
    text.starts_with("interactive-") || text.contains("{$") || text.contains("{ $")
}

#[test]
fn no_input_leaves_a_key_or_a_placeholder_in_a_message() {
    for en in [true, false] {
        for kind in KINDS {
            for v in values() {
                let (message, fix) = interactive_finding_text(kind, &v, en);
                assert!(!leaks(&message), "{kind:?} {v:?} (en={en}): {message}");
                let fix = fix.expect("every kind has a fix suggestion");
                assert!(!leaks(&fix), "{kind:?} {v:?} (en={en}): {fix}");
            }
        }
        for waited_ms in WAITED_MS {
            let text = finding_uncertainty_text(
                &FindingUncertainty::LateReactionPossible { waited_ms },
                en,
            );
            assert!(!leaks(&text), "{waited_ms} (en={en}): {text}");
        }
    }
}
