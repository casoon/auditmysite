//! Text export of the batch PDF sections of slice 19 of #936: everything
//! `interactive.rs`, `portfolio.rs`, `appendix.rs` and `overview.rs` put into
//! the report, across an enumerated input space in both languages, as records
//! in `reports/batch-section-texts.json` (gitignored; the path can be set with
//! `BATCH_SECTION_TEXT_EXPORT`). The render functions are driven with a bare
//! `ReportBuilder` and their components are recorded as JSON, like the export
//! of slice 7 (`text_export.rs`, whose scenario builders it reuses). Run it
//! before and after a change of how the texts are stored and compare the two
//! files byte for byte.

use super::appendix::{render_batch_appendix, render_batch_crawl_links};
use super::causes::{render_batch_causes, render_batch_next_step};
use super::compare::render_batch_page_comparison;
use super::glance::render_batch_glance;
use super::interactive::{render_batch_interactive_summary, translate_interactive_category};
use super::overview::{render_batch_cover, render_batch_en301549_rollup};
use super::portfolio::{
    localized_module_name, render_batch_module_portfolio, render_batch_tech_url_matrix,
    render_batch_url_ranking,
};
use super::technical::render_batch_method;
use super::text_export::{bare, batch, compact, components};
use crate::audit::{BatchError, SampleMetadata, SitemapDiagnostics};
use crate::i18n::I18n;
use crate::model::ReportLevel;
use crate::output::builder::build_batch_presentation_with_locale;
use crate::output::report_model::*;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// The interactive categories the batch table translates, an unknown one and
/// an empty one.
const CATEGORIES: [&str; 17] = [
    "TabOrder",
    "FocusTrap",
    "StateTransition",
    "FocusRestoration",
    "FormError",
    "SpaNavigation",
    "HiddenFocusable",
    "SkipLink",
    "FocusIndicator",
    "MenuJourney",
    "TabsJourney",
    "LinkText",
    "Landmark",
    "MediaControls",
    "HeadingOutline",
    "SomethingNew",
    "",
];

/// Every module name `localized_module_name` knows, the ones it does not
/// (an unscored extra module, an unknown name) and an empty one.
const MODULES: [&str; 18] = [
    "Accessibility",
    "Best Practices",
    "Mobile",
    "Performance",
    "Security",
    "HTML Conformance",
    "SEO",
    "UX",
    "Journey",
    "AI Visibility",
    "Content Visibility",
    "Source Quality",
    "Tech Stack",
    "Commerce",
    "Privacy",
    "Brand New Module",
    "accessibility",
    "",
];

fn push(records: &mut Vec<Value>, name: &str, input: String, lang: &str, output: Value) {
    records.push(json!([name, input, lang, output]));
}

fn export_interactive(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for category in CATEGORIES {
        push(
            records,
            "translate_interactive_category",
            format!("{category:?}"),
            lang,
            json!(translate_interactive_category(category, i18n)),
        );
    }
    let severities = [
        crate::wcag::Severity::Critical,
        crate::wcag::Severity::High,
        crate::wcag::Severity::Medium,
        crate::wcag::Severity::Low,
    ];
    for (tested, with_issues, has_critical) in [
        (3, 0, false),
        (3, 1, false),
        (3, 2, true),
        (1_234_567, 1_234_566, false),
        (0, 0, false),
    ] {
        for rotation in [None, Some(0), Some(1), Some(2), Some(3)] {
            let categories = match rotation {
                None => vec![],
                Some(r) => CATEGORIES
                    .iter()
                    .enumerate()
                    .map(|(i, category)| InteractiveCategoryRow {
                        category: category.to_string(),
                        affected_urls: i + 1,
                        max_severity: severities[(i + r) % 4],
                    })
                    .collect(),
            };
            let summary = InteractiveJourneySummary {
                total_pages_tested: tested,
                pages_with_issues: with_issues,
                categories,
                has_critical,
            };
            for total_urls in [0, 3, 1_234_567] {
                push(
                    records,
                    "render_batch_interactive_summary",
                    format!(
                        "tested={tested} issues={with_issues} critical={has_critical} rotation={rotation:?} urls={total_urls}"
                    ),
                    lang,
                    components(render_batch_interactive_summary(
                        bare(),
                        &summary,
                        total_urls,
                        i18n,
                    )),
                );
            }
        }
    }
}

fn export_module_names(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for name in MODULES {
        push(
            records,
            "localized_module_name",
            format!("{name:?}"),
            lang,
            json!(localized_module_name(name, i18n)),
        );
    }
}

fn module_scores(score: u32) -> Vec<(&'static str, u32)> {
    MODULES.iter().map(|name| (*name, score)).collect()
}

fn export_portfolio(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let batch = batch();
    let mut pres = build_batch_presentation_with_locale(&batch, i18n);
    let normalized: Vec<_> = batch
        .reports
        .iter()
        .map(|r| {
            crate::audit::validation::normalize_checked(r)
                .expect("fixture report passes the report contract")
                .normalized
        })
        .collect();

    // No module averages: the section is left out.
    pres.portfolio_summary.module_averages = vec![];
    push(
        records,
        "render_batch_module_portfolio",
        "no module averages".to_string(),
        lang,
        components(render_batch_module_portfolio(
            bare(),
            &pres,
            &normalized,
            i18n,
        )),
    );

    for score in [100, 95, 90, 89, 75, 74, 60, 59, 0] {
        let scores = module_scores(score);
        pres.portfolio_summary.module_averages = scores
            .iter()
            .map(|(name, score)| (name.to_string(), *score))
            .collect();
        let weaker: Vec<(&str, u32)> = scores
            .iter()
            .map(|(name, score)| (*name, score.saturating_sub(40)))
            .collect();
        for (spread, details) in [
            ("no page measured", vec![]),
            (
                "constant",
                vec![
                    compact("https://example.com/a", &scores),
                    compact("https://example.com/b", &scores),
                ],
            ),
            (
                "one page lower",
                vec![
                    compact("https://example.com/a", &scores),
                    compact("https://example.com/b", &weaker),
                ],
            ),
        ] {
            pres.url_details = details;
            for (pages, reports) in [
                ("no findings", &normalized[..0]),
                ("3 pages", &normalized[..]),
            ] {
                push(
                    records,
                    "render_batch_module_portfolio",
                    format!("score={score} spread={spread} {pages}"),
                    lang,
                    components(render_batch_module_portfolio(bare(), &pres, reports, i18n)),
                );
            }
        }
    }
}

fn crawl_links(kind: &str) -> CrawlLinkSummary {
    let broken = |source: &str, hops: u8, external: bool| BrokenLinkRow {
        source_url: format!("https://example.com/{source}"),
        target_url: "https://example.com/a-very-long-path/that-keeps-going/and-going/file"
            .to_string(),
        status: "404".to_string(),
        is_external: external,
        severity: "high".to_string(),
        redirect_hops: hops,
    };
    let chain = |hops: u8| RedirectChainRow {
        source_url: "https://example.com/old".to_string(),
        target_url: "https://example.com/mid".to_string(),
        final_url: "https://example.com/new".to_string(),
        hops,
        is_external: false,
    };
    match kind {
        "everything" => CrawlLinkSummary {
            seed_url: "https://example.com/".to_string(),
            checked_internal_links: 1_234_567,
            broken_internal_links: vec![broken("a", 0, false), broken("b", 2, false)],
            checked_external_links: 2,
            broken_external_links: vec![broken("c", 0, true), broken("d", 1, true)],
            redirect_chains: vec![chain(2), chain(3)],
        },
        "external clean" => CrawlLinkSummary {
            seed_url: "https://example.com/".to_string(),
            checked_internal_links: 1,
            broken_internal_links: vec![],
            checked_external_links: 5,
            broken_external_links: vec![],
            redirect_chains: vec![],
        },
        _ => CrawlLinkSummary {
            seed_url: "https://example.com/".to_string(),
            checked_internal_links: 2,
            broken_internal_links: vec![],
            checked_external_links: 0,
            broken_external_links: vec![],
            redirect_chains: vec![],
        },
    }
}

fn export_tables(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for kind in ["everything", "external clean", "nothing"] {
        push(
            records,
            "render_batch_crawl_links",
            kind.to_string(),
            lang,
            components(render_batch_crawl_links(bare(), &crawl_links(kind), i18n)),
        );
    }

    let batch = batch();
    let mut pres = build_batch_presentation_with_locale(&batch, i18n);

    // The ranking: its extra columns come from the module scores.
    pres.url_details = vec![compact(
        "https://example.com/a",
        &[
            ("SEO", 70),
            ("Performance", 60),
            ("Security", 50),
            ("HTML Conformance", 40),
        ],
    )];
    push(
        records,
        "render_batch_url_ranking",
        "3 ranked URLs".to_string(),
        lang,
        components(render_batch_url_ranking(bare(), &pres, false, i18n)),
    );
    // More than ten pages: the lowest and the highest five, the full list in
    // the appendix or the JSON report.
    pres.url_ranking = (0..12)
        .map(|i| UrlSummary {
            url: format!("https://example.com/{i}"),
            score: 50.0 + i as f32,
            site_quality_score: 50 + i as u32,
            grade: "C".to_string(),
            critical_violations: 0,
            total_violations: 0,
            passed: true,
            priority: Priority::Medium,
        })
        .collect();
    for has_appendix in [false, true] {
        push(
            records,
            "render_batch_url_ranking",
            format!("12 ranked URLs, appendix={has_appendix}"),
            lang,
            components(render_batch_url_ranking(bare(), &pres, has_appendix, i18n)),
        );
    }

    // The technical URL matrix: word counts and titles, the page-type
    // table (all pages, or the weakest ten), and the crawl links.
    let words = [0, 1234, 1_234_567];
    pres.url_matrix = (0..3)
        .map(|i| UrlMatrixRow {
            rank: i + 1,
            url: format!("https://example.com/{i}"),
            title: [
                None,
                Some("A title".to_string()),
                Some("A very long page title that keeps going and going and going".to_string()),
            ][i]
                .clone(),
            inbound_links: i,
            outbound_links: 10 * i as u32,
            word_count: words[i],
        })
        .collect();
    for pages in [3, 11] {
        pres.url_details = (0..pages)
            .map(|i| {
                let mut detail = compact(
                    &format!("https://example.com/{i}"),
                    &[("Accessibility", 90 - i as u32)],
                );
                detail.score = 90.0 - i as f32;
                if i % 2 == 0 {
                    detail.page_type = Some("article".to_string());
                    detail.page_attributes =
                        vec!["long-form".to_string(), "structured".to_string()];
                    detail.top_issues = vec![
                        "Insufficient contrast".to_string(),
                        "frame-ancestors directive missing".to_string(),
                        "Third issue".to_string(),
                        "Fourth issue".to_string(),
                    ];
                } else {
                    detail.top_issues = vec!["Only issue".to_string()];
                }
                detail
            })
            .collect();
        for level in [
            ReportLevel::Executive,
            ReportLevel::Standard,
            ReportLevel::Technical,
        ] {
            for crawl in [None, Some("everything"), Some("nothing")] {
                pres.portfolio_summary.crawl_links = crawl.map(crawl_links);
                let config = ReportConfig {
                    level,
                    ..ReportConfig::default()
                };
                push(
                    records,
                    "render_batch_tech_url_matrix",
                    format!("pages={pages} level={level:?} crawl={crawl:?}"),
                    lang,
                    components(render_batch_tech_url_matrix(bare(), &pres, &config, i18n)),
                );
            }
        }
    }

    // The appendix: a URL without violations is skipped.
    let violation = |elements: usize| AppendixViolation {
        rule: "1.4.3".to_string(),
        rule_name: "Contrast (Minimum)".to_string(),
        severity: crate::wcag::Severity::High,
        message: "Insufficient contrast".to_string(),
        fix_suggestion: None,
        affected_elements: (0..elements)
            .map(|i| AffectedElement {
                selector: format!("span.separator-{i}"),
                node_id: format!("node-{i}"),
            })
            .collect(),
    };
    pres.appendix = BatchAppendixData {
        per_url: vec![
            UrlAppendix {
                url: "https://example.com/clean".to_string(),
                violations: vec![],
            },
            UrlAppendix {
                url: "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/file".to_string(),
                violations: vec![violation(0), violation(1), violation(3)],
            },
        ],
    };
    push(
        records,
        "render_batch_appendix",
        "one clean URL, one with violations".to_string(),
        lang,
        components(render_batch_appendix(bare(), &pres, i18n)),
    );
}

fn sample(source: &str, selection: &str, audited: usize, total: usize) -> SampleMetadata {
    SampleMetadata {
        source: source.to_string(),
        total_discovered: total,
        audited,
        sample_limit: None,
        selection: selection.to_string(),
        is_sample: audited < total,
        strata: vec![],
    }
}

fn export_cover(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let base = batch();
    let mut pres = build_batch_presentation_with_locale(&base, i18n);
    // The date is the day of the run.
    pres.cover.date = "2026-01-01".to_string();
    let samples = [
        ("no sample", None),
        ("first_n", Some(sample("sitemap", "first_n", 20, 40))),
        ("stratified", Some(sample("crawl", "stratified", 1, 3))),
        (
            "path_filter",
            Some(sample("url_file", "path_filter", 7, 3000)),
        ),
        ("all", Some(sample("sitemap", "all", 3, 3))),
        ("nothing discovered", Some(sample("sitemap", "all", 0, 0))),
    ];
    for level in [
        ReportLevel::Executive,
        ReportLevel::Standard,
        ReportLevel::Technical,
    ] {
        for appendix in [false, true] {
            pres.appendix.per_url = if appendix {
                vec![UrlAppendix {
                    url: "https://example.com/a".to_string(),
                    violations: vec![],
                }]
            } else {
                vec![]
            };
            for (name, sample) in &samples {
                for failures in [false, true] {
                    let mut batch = batch();
                    batch.sample = sample.clone();
                    if failures {
                        batch.errors = vec![BatchError::new("https://example.com/x", "timeout")];
                        batch.sitemap_diagnostics = Some(SitemapDiagnostics::default());
                    }
                    let config = ReportConfig {
                        level,
                        locale: lang.to_string(),
                        ..ReportConfig::default()
                    };
                    let cover = render_batch_cover(bare(), &batch, &pres, &config, 87, i18n)
                        .expect("the cover renders");
                    push(
                        records,
                        "render_batch_cover",
                        format!(
                            "level={level:?} appendix={appendix} sample={name} failures={failures}"
                        ),
                        lang,
                        components(cover),
                    );
                }
            }
        }
    }
}

fn export_status(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let batch = batch();
    let normalized: Vec<_> = batch
        .reports
        .iter()
        .map(|r| {
            crate::audit::validation::normalize_checked(r)
                .expect("fixture report passes the report contract")
                .normalized
        })
        .collect();
    let rollup = crate::wcag::en301549::derive_batch_rollup(
        normalized.iter().map(|r| r.findings.as_slice()),
    );
    let empty_rollup = crate::wcag::en301549::derive_batch_rollup(std::iter::empty());
    let all_criteria: std::collections::BTreeSet<String> =
        rollup.iter().map(|r| r.clause.wcag.to_string()).collect();
    let none = std::collections::BTreeSet::new();

    for (name, rollup) in [("with findings", &rollup), ("no findings", &empty_rollup)] {
        for (failed_name, failed) in [("none failed", &none), ("all failed", &all_criteria)] {
            push(
                records,
                "render_batch_en301549_rollup",
                format!("{name}, {failed_name}"),
                lang,
                components(render_batch_en301549_rollup(bare(), rollup, failed, i18n)),
            );
        }
    }

    let pres = build_batch_presentation_with_locale(&batch, i18n);
    for failures in [false, true] {
        let mut batch = batch.clone();
        if failures {
            batch.errors = vec![BatchError::new("https://example.com/x", "timeout")];
            batch.sitemap_diagnostics = Some(SitemapDiagnostics::default());
        }
        push(
            records,
            "render_batch_glance",
            format!("failures={failures}"),
            lang,
            components(
                render_batch_glance(bare(), &batch, &pres, 87, i18n).expect("the overview renders"),
            ),
        );
        push(
            records,
            "render_batch_method",
            format!("failures={failures}"),
            lang,
            components(render_batch_method(bare(), &batch, &pres, i18n)),
        );
    }
    push(
        records,
        "render_batch_page_comparison",
        "fixture".to_string(),
        lang,
        components(render_batch_page_comparison(bare(), &pres, i18n)),
    );
    push(
        records,
        "render_batch_causes",
        "fixture".to_string(),
        lang,
        components(render_batch_causes(bare(), &pres, i18n)),
    );
    push(
        records,
        "render_batch_next_step",
        "fixture".to_string(),
        lang,
        components(render_batch_next_step(bare(), &pres, i18n)),
    );
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_interactive(&mut records, lang, &i18n);
        export_module_names(&mut records, lang, &i18n);
        export_portfolio(&mut records, lang, &i18n);
        export_tables(&mut records, lang, &i18n);
        export_cover(&mut records, lang, &i18n);
        export_status(&mut records, lang, &i18n);
    }
    records
}

#[test]
#[ignore = "writes reports/batch-section-texts.json; run before and after a change of the text storage"]
fn export_batch_section_texts() {
    let records = records();
    let path = std::env::var("BATCH_SECTION_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/batch-section-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} batch section text records written to {path}",
        records.len()
    );
}
