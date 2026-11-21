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

/// "1 audited pages have no H1" / "1 occurrences on 1 URLs" (#1039): the
/// counts of the consistency and key-point messages reach them as numbers, so
/// one selects the singular and the verb agrees with it.
#[test]
fn consistency_counts_agree_in_number() {
    use fluent_bundle::FluentValue;
    for (lang, key, one, many) in [
        (
            "en",
            "batch-consistency-missing-h1-detail",
            "1 audited page has no H1.",
            "3 audited pages have no H1.",
        ),
        (
            "en",
            "batch-consistency-multiple-h1-detail",
            "1 audited page has multiple H1 headings.",
            "3 audited pages have multiple H1 headings.",
        ),
        (
            "en",
            "batch-consistency-blocker-detail",
            "Required condition missing on 1 audited page",
            "Required condition missing on 3 audited pages",
        ),
        (
            "en",
            "batch-consistency-parity-detail",
            "Visible content differs on 1 audited page",
            "Visible content differs on 3 audited pages",
        ),
        (
            "de",
            "batch-consistency-missing-h1-detail",
            "1 geprüfte Seite hat keine H1.",
            "3 geprüfte Seiten haben keine H1.",
        ),
        (
            "de",
            "batch-consistency-multiple-h1-detail",
            "1 geprüfte Seite hat mehrere H1-Überschriften.",
            "3 geprüfte Seiten haben mehrere H1-Überschriften.",
        ),
        (
            "de",
            "batch-consistency-blocker-detail",
            "Pflichtbedingung fehlt auf 1 geprüften Seite",
            "Pflichtbedingung fehlt auf 3 geprüften Seiten",
        ),
        (
            "de",
            "batch-consistency-parity-detail",
            "Sichtbarer Inhalt weicht auf 1 geprüften Seite ab",
            "Sichtbarer Inhalt weicht auf 3 geprüften Seiten ab",
        ),
    ] {
        let i18n = crate::i18n::I18n::new(lang).unwrap();
        let text = |count: i64| i18n.t_args(key, &[("count", FluentValue::from(count))]);
        assert_eq!(text(1), one, "{lang} {key}");
        assert_eq!(text(3), many, "{lang} {key}");
    }

    let hosts = |lang: &str, www: i64, non_www: i64| {
        crate::i18n::I18n::new(lang).unwrap().t_args(
            "batch-consistency-mixed-hosts-detail",
            &[
                ("www", FluentValue::from(www)),
                ("non_www", FluentValue::from(non_www)),
            ],
        )
    };
    assert_eq!(
        hosts("en", 1, 1),
        "1 page uses www and 1 use non-www canonicals."
    );
    assert_eq!(
        hosts("en", 3, 2),
        "3 pages use www and 2 use non-www canonicals."
    );
    assert_eq!(
        hosts("de", 1, 1),
        "1 Seite verwendet www- und 1 Seite nicht-www-Canonicals."
    );
    assert_eq!(
        hosts("de", 3, 2),
        "3 Seiten verwenden www- und 2 Seiten nicht-www-Canonicals."
    );

    let headers = |lang: &str, checked: i64| {
        crate::i18n::I18n::new(lang).unwrap().t_args(
            "batch-consistency-headers-title",
            &[("checked", FluentValue::from(checked))],
        )
    };
    assert!(headers("en", 1).contains("(1 page with a security check)"));
    assert!(headers("en", 5).contains("(5 pages with a security check)"));
    assert!(headers("de", 1).contains("(1 Seite mit Sicherheitsprüfung)"));
    assert!(headers("de", 5).contains("(5 Seiten mit Sicherheitsprüfung)"));
}

/// The thin-content insight carries its consequence in both languages (the
/// English report matched a German word and printed it bare, #1039), and the
/// page type is German in the German report.
#[test]
fn thin_content_insight_names_its_consequence_in_both_languages() {
    use crate::output::report_model::{DistributionInsight, DistributionInsightKind};
    for (lang, consequence, page_type) in [
        ("en", "weaker rankings", "Thin / Minimal Content"),
        ("de", "schwächere Rankings", "Dünner / minimaler Inhalt"),
    ] {
        let i18n = crate::i18n::I18n::new(lang).unwrap();
        let mut pres = crate::output::builder::build_batch_presentation_with_locale(
            &super::text_export::batch(),
            &i18n,
        );
        pres.portfolio_summary.distribution_insights = vec![DistributionInsight {
            kind: DistributionInsightKind::HighThinContentShare,
            text: i18n.t("output-batch-distribution-thin-content"),
        }];
        let rendered = super::text_export::components(
            super::seo::render_seo_distribution_insights(super::text_export::bare(), &pres, &i18n),
        );
        let texts = all_strings(&rendered);
        assert!(
            texts.iter().any(|text| text.contains(consequence)),
            "{lang}: {texts:?}"
        );
        assert_eq!(i18n.t("seo-profile-page-type-thin-content"), page_type);
    }
}

/// The first key point names the URLs that carry critical/high findings, not
/// all audited URLs ("on 5 URLs" for two failing pages of five, #1039), and
/// both key points agree with a count of one.
#[test]
fn key_points_name_the_affected_urls_and_agree_in_number() {
    for (lang, severe, main) in [
        (
            "en",
            "5 critical/high WCAG occurrences on 1 URL (2 findings, grouped per page)",
            "(1 occurrence on 1 URL)",
        ),
        (
            "de",
            "5 kritische/hohe WCAG-Vorkommen auf 1 URL (2 Befunde, je Seite gruppiert)",
            "(1 Vorkommen auf 1 URL)",
        ),
    ] {
        let i18n = crate::i18n::I18n::new(lang).unwrap();
        let mut pres = crate::output::builder::build_batch_presentation_with_locale(
            &super::text_export::batch(),
            &i18n,
        );
        assert_eq!(pres.url_ranking.len(), 3);
        for (i, url) in pres.url_ranking.iter_mut().enumerate() {
            url.critical_violations = if i == 1 { 2 } else { 0 };
        }
        pres.portfolio_summary.total_urls = 3;
        pres.portfolio_summary.occurrence_distribution = super::text_export::dist(2, 3);
        pres.top_issues[0].finding.occurrence_count = 1;
        pres.top_issues[0].affected_urls.truncate(1);
        let points = super::management::build_batch_key_points(
            &pres,
            &super::text_export::dist(1, 1),
            0,
            &i18n,
        );
        assert_eq!(points[0], severe, "{lang}");
        assert!(points[1].ends_with(main), "{lang}: {}", points[1]);
    }
}
