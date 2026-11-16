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
    assert_eq!(en.len(), 47);
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
                    && !text.contains("severity-")
                    && !text.starts_with("module-")
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}

fn all_strings(value: &Value) -> Vec<&str> {
    let mut out = Vec::new();
    strings(value, &mut out);
    out
}

/// Three batch texts that were literals of one language: the overview card of
/// the broken internal links ("Broken Links" in German reports), the status of
/// a broken crawl link without code and error ("Unbekannt" in both
/// languages) and the module names "Privacy" and "Commerce" in the batch
/// tables.
#[test]
fn batch_texts_follow_the_run_language() {
    use crate::audit::{BrokenLink, BrokenLinkSeverity, CrawlDiagnostics};

    // The card, as the export records it.
    for (lang, card, leaked) in [
        ("en", "Broken internal links", None),
        ("de", "Kaputte interne Links", Some("Broken Links")),
    ] {
        let record = records()
            .into_iter()
            .find(|r| {
                r[0] == "build_batch_overview_grid"
                    && r[2] == lang
                    && r[1].as_str().is_some_and(|i| i.ends_with("broken=Some(3)"))
            })
            .expect("the export has the card");
        let texts = all_strings(&record);
        assert!(texts.contains(&card), "{lang}: {card} missing in {record}");
        if let Some(leaked) = leaked {
            assert!(
                !texts.contains(&leaked),
                "{lang}: {leaked} left in {record}"
            );
        }
    }

    // The status of a link that has neither a code nor an error.
    let status = |lang: &str| {
        let i18n = crate::i18n::I18n::new(lang).unwrap();
        let link = |external: bool| BrokenLink {
            source_url: "https://example.com/".to_string(),
            target_url: "https://example.com/x".to_string(),
            status_code: None,
            error: None,
            is_external: external,
            redirect_hops: 0,
            severity: BrokenLinkSeverity::Medium,
        };
        let batch = super::text_export::batch().with_crawl_diagnostics(CrawlDiagnostics {
            seed_url: "https://example.com/".to_string(),
            discovered_urls: 3,
            checked_internal_links: 1,
            broken_internal_links: vec![link(false)],
            checked_external_links: 1,
            broken_external_links: vec![link(true)],
            redirect_chains: vec![],
        });
        let crawl = crate::output::builder::build_batch_presentation_with_locale(&batch, &i18n)
            .portfolio_summary
            .crawl_links
            .expect("crawl summary");
        (
            crawl.broken_internal_links[0].status.clone(),
            crawl.broken_external_links[0].status.clone(),
        )
    };
    assert_eq!(status("en"), ("Unknown".to_string(), "Unknown".to_string()));
    assert_eq!(
        status("de"),
        ("Unbekannt".to_string(), "Unbekannt".to_string())
    );

    // The module names of the batch tables.
    let de = crate::i18n::I18n::new("de").unwrap();
    assert_eq!(
        super::portfolio::localized_module_name("Privacy", &de),
        "Datenschutz"
    );
    assert_eq!(
        super::portfolio::localized_module_name("Commerce", &de),
        "Shop-Prüfung"
    );
}

/// "1 URLs audited" / "1 URLs geprüft": the cover counts its URLs with a
/// number the message can choose the singular for.
#[test]
fn batch_cover_counts_one_url_in_the_singular() {
    let one = || {
        crate::audit::BatchReport::from_reports(
            vec![super::text_export::contrast_page("https://example.com/a")],
            vec![],
            100,
        )
    };
    for (lang, expected) in [("en", "1 URL audited"), ("de", "1 URL geprüft")] {
        let i18n = crate::i18n::I18n::new(lang).unwrap();
        let cover =
            crate::output::builder::build_batch_presentation_with_locale(&one(), &i18n).cover;
        assert_eq!(cover.url, expected, "{lang}");
    }
    let i18n = crate::i18n::I18n::new("en").unwrap();
    let three = crate::output::builder::build_batch_presentation_with_locale(
        &super::text_export::batch(),
        &i18n,
    );
    assert_eq!(three.cover.url, "3 URLs audited");
}
