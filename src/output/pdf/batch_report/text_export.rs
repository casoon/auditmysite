//! Text export of the batch PDF consistency and management sections of
//! slice 7 of #936: everything `consistency.rs` and `management.rs` put into
//! the report, across an enumerated input space in both languages, as records
//! in `reports/batch-consistency-texts.json` (gitignored; the path can be set
//! with `BATCH_CONSISTENCY_TEXT_EXPORT`). The render functions are driven with
//! a bare `ReportBuilder` and their components are recorded as JSON, so the
//! export covers the texts built inside them without a full report. Run it
//! before and after a change of how the texts are stored and compare the two
//! files byte for byte.

use super::consistency::render_batch_consistency;
use super::management::{
    audit_flag_batch_title, build_batch_assessment, build_batch_key_points,
    render_batch_audit_flags, render_batch_decision_actions, render_batch_internal_comparison,
    render_batch_management_risks,
};
use crate::audit::batch_consistency::{
    BatchConsistencyAnalysis, CanonicalConsistency, CrossPageCriterionAssessment,
    HeadingConsistency, HelpConsistency, HelpDeviation, NavigationConsistency, OrphanPageAnalysis,
    RecurringSchemaFinding, SchemaEntityConflict, SchemaGraphAnalysis, SchemaPageMatrixRow,
    SchemaTypeCount, StructuredDataConsistency,
};
use crate::audit::{AuditReport, BatchReport, ViewportScoreSet, ViewportScores};
use crate::i18n::I18n;
use crate::model::WcagLevel;
use crate::output::builder::build_batch_presentation_with_locale;
use crate::output::report_model::*;
use crate::patterns::help_mechanisms::HelpRegion;
use crate::wcag::WcagResults;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// Counts: one, two and one that Fluent could group.
const COUNTS: [usize; 3] = [1, 2, 1_234_567];

/// Free texts the sections pass through from the data.
const TEXTS: [&str; 4] = [
    "",
    " padded value ",
    "{braces} and {$var}",
    "äöüß — „quotes“ \"x\" <b>; semi",
];

/// A page with one high-severity Level AA contrast violation.
fn contrast_page(url: &str) -> AuditReport {
    let mut results = WcagResults::new();
    results.add_violation(
        crate::wcag::Violation::new(
            "1.4.3",
            "Contrast (Minimum)",
            WcagLevel::AA,
            crate::wcag::Severity::High,
            "Insufficient contrast",
            "node-1",
        )
        .with_rule_id("contrast/text-insufficient")
        .with_selector("span.separator"),
    );
    AuditReport::new(url.to_string(), WcagLevel::AA, results, 100)
}

fn batch() -> BatchReport {
    BatchReport::from_reports(
        ["a", "b", "c"]
            .iter()
            .map(|p| contrast_page(&format!("https://example.com/{p}")))
            .collect(),
        vec![],
        100,
    )
}

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

fn criteria(basis: &str) -> Vec<CrossPageCriterionAssessment> {
    [
        ("3.2.3 Consistent Navigation", "no_inconsistency_detected"),
        ("3.2.4 Consistent Identification", "manual_review"),
        ("3.2.6 Consistent Help", "warning"),
        ("2.4.5 Multiple Ways", "something_new"),
        ("9.9.9 Unknown", "warning"),
    ]
    .into_iter()
    .map(|(criterion, status)| CrossPageCriterionAssessment {
        criterion: criterion.to_string(),
        status: status.to_string(),
        basis: basis.to_string(),
        affected_pages: 2,
    })
    .collect()
}

fn help_deviations(n: usize) -> Vec<HelpDeviation> {
    (0..n)
        .map(|i| {
            if i % 2 == 0 {
                HelpDeviation::Region {
                    url: format!("https://example.com/r{i}"),
                    mechanism: "contact".to_string(),
                    expected: vec![HelpRegion::Footer],
                    found: vec![HelpRegion::Header],
                }
            } else {
                HelpDeviation::Order {
                    url: format!("https://example.com/o{i}"),
                    region: HelpRegion::Footer,
                    first: "contact".to_string(),
                    second: "faq".to_string(),
                }
            }
        })
        .collect()
}

/// The consistency analysis with every section filled, its counts set to
/// `count` (the totals one higher, so the "missing" rows appear).
fn full_consistency(count: usize, text: &str) -> BatchConsistencyAnalysis {
    let total = count + 1;
    BatchConsistencyAnalysis {
        navigation: NavigationConsistency {
            pages_with_main_nav: count,
            pages_with_skip_link: count,
            total_pages: total,
            findings: vec![text.to_string()],
        },
        help: HelpConsistency {
            pages_with_help: count,
            total_pages: total,
            repeated_mechanisms: count,
            deviations: help_deviations(12),
        },
        headings: HeadingConsistency {
            pages_with_single_h1: count,
            pages_with_no_h1: count,
            pages_with_multiple_h1: count,
            total_pages: total,
            findings: vec![text.to_string()],
        },
        canonical: CanonicalConsistency {
            www_count: count,
            non_www_count: count,
            missing_count: count,
            total_pages: total,
            findings: vec![text.to_string()],
        },
        orphan_pages: OrphanPageAnalysis {
            orphan_urls: vec!["https://example.com/orphan".to_string(), text.to_string()],
            total_pages: total,
            findings: vec![],
        },
        schema_graph: SchemaGraphAnalysis {
            conflicts: vec![SchemaEntityConflict {
                entity_id: "https://example.com/#org".to_string(),
                conflicts: vec!["name differs".to_string(), text.to_string()],
            }],
            findings: vec![],
        },
        structured_data: StructuredDataConsistency {
            type_distribution: vec![SchemaTypeCount {
                schema_type: "Organization".to_string(),
                pages: count,
            }],
            recurring_blockers: vec![RecurringSchemaFinding {
                key: "Product.offers".to_string(),
                affected_pages: count,
                urls: vec![],
            }],
            parity_mismatches: vec![RecurringSchemaFinding {
                key: "Product.offers.price".to_string(),
                affected_pages: count,
                urls: vec![],
            }],
            page_type_matrix: [
                ("corporate_page", "complete", vec!["Organization"], vec![]),
                ("faq_page", "opportunity", vec![], vec!["FAQPage", "WebPage"]),
                ("indeterminate", "manual_review", vec!["A", "B"], vec!["C"]),
                ("hub_page", "not_applicable", vec![], vec![]),
                ("something_new", "something_new", vec!["X"], vec!["Y"]),
            ]
            .into_iter()
            .map(|(kind, status, expected, detected)| SchemaPageMatrixRow {
                url: "https://example.com/a-very-long-path/that-keeps-going/and-going/file"
                    .to_string(),
                page_kind: kind.to_string(),
                coverage_status: status.to_string(),
                expected_types: expected.into_iter().map(String::from).collect(),
                detected_types: detected.into_iter().map(String::from).collect(),
            })
            .collect(),
            identity_findings: vec![
                format!(
                    "Organization identity uses {count} different @id values across the audited pages."
                ),
                format!(
                    "{count} of {total} organization nodes have no stable @id for cross-page identity validation."
                ),
                text.to_string(),
            ],
        },
        wcag_cross_page: criteria(text),
    }
}

fn header(
    name: &str,
    checked: usize,
    present: usize,
    distinct: usize,
) -> SecurityHeaderConsistency {
    SecurityHeaderConsistency {
        header: name.to_string(),
        pages_checked: checked,
        pages_present: present,
        distinct_values: distinct,
        missing_on: vec![
            "https://example.com/a".to_string(),
            "https://example.com/b".to_string(),
        ],
    }
}

/// Every row kind of the security header panel, with `count` pages checked.
fn security_headers(count: usize) -> Vec<SecurityHeaderConsistency> {
    let checked = count + 1;
    vec![
        header("Content-Security-Policy", checked, count, 1),
        header("Strict-Transport-Security", checked, checked, 3),
        header("X-Frame-Options", checked, checked, 1),
        header("Referrer-Policy", checked, checked, 1),
        header("Permissions-Policy", checked, 0, 0),
        header("X-Content-Type-Options", checked, 0, 0),
    ]
}

fn export_consistency(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let batch = batch();
    let mut pres = build_batch_presentation_with_locale(&batch, i18n);
    let mut render =
        |name: &str, input: String, c: &BatchConsistencyAnalysis, p: &BatchPresentation| {
            records.push(json!([
                name,
                input,
                lang,
                components(render_batch_consistency(bare(), c, p, i18n))
            ]));
        };

    pres.portfolio_summary.security_header_consistency = vec![];
    pres.portfolio_summary.top_topics = vec![];
    pres.portfolio_summary.overlap_pairs = vec![];
    pres.portfolio_summary.schema_distribution = vec![("Organization".to_string(), 3)];
    render(
        "render_batch_consistency",
        "empty analysis".to_string(),
        &BatchConsistencyAnalysis::default(),
        &pres,
    );

    for count in COUNTS {
        for text in TEXTS {
            pres.portfolio_summary.security_header_consistency = security_headers(count);
            pres.portfolio_summary.top_topics = vec![
                ("accessibility".to_string(), count + 1),
                ("single".to_string(), 1),
                (text.to_string(), 2),
            ];
            pres.portfolio_summary.overlap_pairs = vec![TopicOverlapPair {
                url_a: "https://example.com/a".to_string(),
                url_b: "https://example.com/b".to_string(),
                similarity_percent: count as u32,
                shared_terms: vec!["audit".to_string(), text.to_string()],
            }];
            for schema_distribution in [vec![], vec![("Organization".to_string(), count)]] {
                let shown = schema_distribution.is_empty();
                pres.portfolio_summary.schema_distribution = schema_distribution;
                render(
                    "render_batch_consistency",
                    format!("full count={count} text={text:?} structured_types={shown}"),
                    &full_consistency(count, text),
                    &pres,
                );
            }
        }
    }

    // One deviation at a time: each row of the navigation, heading,
    // canonical and security header panels on its own.
    pres.portfolio_summary.overlap_pairs = vec![];
    pres.portfolio_summary.top_topics = vec![];
    let finding = vec!["x".to_string()];
    let partial = [
        (
            "main nav missing",
            NavigationConsistency {
                pages_with_main_nav: 1,
                pages_with_skip_link: 2,
                total_pages: 2,
                findings: finding.clone(),
            },
            HeadingConsistency::default(),
            CanonicalConsistency::default(),
        ),
        (
            "skip link missing, no H1, mixed hosts",
            NavigationConsistency {
                pages_with_main_nav: 2,
                pages_with_skip_link: 1,
                total_pages: 2,
                findings: finding.clone(),
            },
            HeadingConsistency {
                pages_with_single_h1: 1,
                pages_with_no_h1: 1,
                pages_with_multiple_h1: 0,
                total_pages: 2,
                findings: finding.clone(),
            },
            CanonicalConsistency {
                www_count: 1,
                non_www_count: 1,
                missing_count: 0,
                total_pages: 2,
                findings: finding.clone(),
            },
        ),
        (
            "multiple H1, canonical missing",
            NavigationConsistency::default(),
            HeadingConsistency {
                pages_with_single_h1: 1,
                pages_with_no_h1: 0,
                pages_with_multiple_h1: 1,
                total_pages: 2,
                findings: finding.clone(),
            },
            CanonicalConsistency {
                www_count: 1,
                non_www_count: 0,
                missing_count: 1,
                total_pages: 2,
                findings: finding.clone(),
            },
        ),
    ];
    for (input, navigation, headings, canonical) in partial {
        let consistency = BatchConsistencyAnalysis {
            navigation,
            headings,
            canonical,
            ..BatchConsistencyAnalysis::default()
        };
        for headers in [
            vec![header("X-Frame-Options", 2, 2, 1)],
            vec![header("Permissions-Policy", 2, 0, 0)],
        ] {
            let kind = headers[0].header.clone();
            pres.portfolio_summary.security_header_consistency = headers;
            render(
                "render_batch_consistency",
                format!("{input}; headers={kind}"),
                &consistency,
                &pres,
            );
        }
    }
}

fn compact(url: &str, modules: &[(&str, u32)]) -> CompactUrlSummary {
    CompactUrlSummary {
        url: url.to_string(),
        score: 80.0,
        grade: "B".to_string(),
        critical_violations: 0,
        total_violations: 0,
        page_type: None,
        page_attributes: vec![],
        page_semantic_score: None,
        biggest_lever: String::new(),
        topic_terms: vec![],
        top_issues: vec![],
        module_scores: modules.iter().map(|(m, s)| (m.to_string(), *s)).collect(),
    }
}

fn ranked(url: &str, score: f32) -> UrlSummary {
    UrlSummary {
        url: url.to_string(),
        score,
        overall_score: score as u32,
        grade: "B".to_string(),
        critical_violations: 0,
        total_violations: 0,
        passed: true,
        priority: Priority::Medium,
    }
}

fn dist(critical: usize, high: usize) -> SeverityDistribution {
    SeverityDistribution {
        critical,
        high,
        medium: 0,
        low: 0,
    }
}

fn export_management(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for kind in [
        "consent_banner",
        "bypass_blocks_untested",
        "conflicting_signal",
        "viewport_gap",
        "incomplete_audit",
        "consent_wall_artifact",
        "side_check_unanswered",
        "",
    ] {
        records.push(json!([
            "audit_flag_batch_title",
            kind,
            lang,
            audit_flag_batch_title(kind, i18n)
        ]));
    }

    // Consent banner on two pages, a viewport gap on one.
    let mut pages: Vec<AuditReport> = ["a", "b", "c"]
        .iter()
        .map(|p| contrast_page(&format!("https://example.com/{p}")))
        .collect();
    pages[0].consent_banner_detected = true;
    pages[1].consent_banner_detected = true;
    pages[1].consent_banner_cmp = Some("Usercentrics".to_string());
    pages[0].viewport_scores = Some(ViewportScores {
        desktop: ViewportScoreSet {
            accessibility: 95,
            performance: None,
            overall: 95,
        },
        mobile: ViewportScoreSet {
            accessibility: 60,
            performance: None,
            overall: 60,
        },
        weighted_overall: 71,
    });
    let flagged = BatchReport::from_reports(pages, vec![], 100);
    records.push(json!([
        "render_batch_audit_flags",
        "consent banner on 2 pages, viewport gap on 1",
        lang,
        components(render_batch_audit_flags(bare(), &flagged, i18n))
    ]));

    let batch = batch();
    let mut pres = build_batch_presentation_with_locale(&batch, i18n);

    for score in [
        0.0, 39.4, 39.5, 40.0, 59.0, 60.0, 74.0, 75.0, 89.0, 90.0, 100.0,
    ] {
        pres.portfolio_summary.average_score = Some(score);
        for (critical, high) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            records.push(json!([
                "build_batch_assessment",
                format!("score={score} critical={critical} high={high}"),
                lang,
                build_batch_assessment(&pres.portfolio_summary, &dist(critical, high), i18n)
            ]));
        }
    }

    let normalized: Vec<_> = batch
        .reports
        .iter()
        .map(|r| {
            crate::audit::validation::normalize_checked(r)
                .expect("fixture page passes the report contract")
                .normalized
        })
        .collect();
    for (input, reports) in [("no pages", &normalized[..0]), ("3 pages", &normalized[..])] {
        records.push(json!([
            "render_batch_management_risks",
            input,
            lang,
            components(render_batch_management_risks(bare(), reports, i18n))
        ]));
    }

    // Internal comparison: below three URLs a note, then the module table
    // (no variation, ties, distinct best and weakest) and the outliers.
    pres.portfolio_summary.average_score = Some(80.0);
    for n in [0, 1, 2] {
        pres.url_details = (0..n)
            .map(|i| {
                compact(
                    &format!("https://example.com/{i}"),
                    &[("accessibility", 80)],
                )
            })
            .collect();
        records.push(json!([
            "render_batch_internal_comparison",
            format!("{n} URLs"),
            lang,
            components(render_batch_internal_comparison(bare(), &pres, i18n))
        ]));
    }
    pres.url_details = vec![
        compact(
            "https://example.com/a",
            &[("accessibility", 90), ("seo", 70), ("performance", 50)],
        ),
        compact(
            "https://example.com/b",
            &[("accessibility", 90), ("seo", 70), ("performance", 60)],
        ),
        compact(
            "https://example.com/c",
            &[("accessibility", 70), ("seo", 70), ("performance", 60)],
        ),
        compact(
            "https://example.com/d",
            &[("accessibility", 90), ("seo", 70)],
        ),
    ];
    for ranking in [
        vec![ranked("https://example.com/a", 80.0)],
        vec![
            ranked("https://example.com/a", 80.0),
            ranked("https://example.com/b", 65.0),
            ranked("https://example.com/c", 64.6),
            ranked("https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/file", 0.0),
        ],
    ] {
        let input = format!("4 URLs, {} ranked", ranking.len());
        pres.url_ranking = ranking;
        records.push(json!([
            "render_batch_internal_comparison",
            input,
            lang,
            components(render_batch_internal_comparison(bare(), &pres, i18n))
        ]));
    }

    // Decision actions: one top issue across every root, scope and reach.
    let rule_id = pres.top_issues[0].finding.rule_id.clone();
    for cluster in [false, true] {
        pres.template_clusters = if cluster {
            vec![TemplateClusterView {
                id: "p1".to_string(),
                rule_id: rule_id.clone(),
                selector: "nav > div.menu a.mainnav-toggle".to_string(),
                confidence: "confirmed".to_string(),
                affected_pages: 3,
                headline: "headline".to_string(),
                decision_label: "shared component".to_string(),
            }]
        } else {
            vec![]
        };
        for urls in [1, 2, 3] {
            for occurrences in COUNTS {
                for complexity in ["high", "medium", "low", "other"] {
                    for wcag_level in ["", "AA"] {
                        let group = &mut pres.top_issues[0];
                        group.affected_urls = (0..urls)
                            .map(|i| format!("https://example.com/{i}"))
                            .collect();
                        group.finding.occurrence_count = occurrences;
                        group.finding.complexity = complexity.to_string();
                        group.finding.wcag_level = wcag_level.to_string();
                        records.push(json!([
                            "render_batch_decision_actions",
                            format!(
                                "cluster={cluster} urls={urls} occurrences={occurrences} complexity={complexity} wcag={wcag_level:?}"
                            ),
                            lang,
                            components(render_batch_decision_actions(bare(), &pres, i18n))
                        ]));
                    }
                }
            }
        }
    }
    pres.template_clusters = vec![];

    // Key points: critical/high counts, the main issue, the legal status.
    for count in COUNTS {
        pres.portfolio_summary.total_urls = count;
        pres.portfolio_summary.occurrence_distribution = dist(count, count);
        pres.top_issues[0].finding.occurrence_count = count;
        for with_top in [true, false] {
            let top = if with_top {
                vec![]
            } else {
                std::mem::take(&mut pres.top_issues)
            };
            for (critical, high) in [(0, 0), (1, 0), (0, 1), (count, count)] {
                for legal_flags in [0, 1] {
                    records.push(json!([
                        "build_batch_key_points",
                        format!(
                            "count={count} top_issue={with_top} critical={critical} high={high} legal_flags={legal_flags}"
                        ),
                        lang,
                        build_batch_key_points(&pres, &dist(critical, high), legal_flags, i18n)
                    ]));
                }
            }
            if !with_top {
                pres.top_issues = top;
            }
        }
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_consistency(&mut records, lang, &i18n);
        export_management(&mut records, lang, &i18n);
    }
    records
}

#[test]
#[ignore = "writes reports/batch-consistency-texts.json; run before and after a change of the text storage"]
fn export_batch_consistency_texts() {
    let records = records();
    let path = std::env::var("BATCH_CONSISTENCY_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/batch-consistency-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} batch consistency text records written to {path}",
        records.len()
    );
}
