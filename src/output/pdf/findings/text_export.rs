//! Text export of `findings.rs` and `detail_modules.rs`, slice 17 of #936:
//! everything the two modules put into the report, across an enumerated input
//! space in both languages, as records in `reports/pdf-findings-texts.json`
//! (gitignored; the path can be set with `PDF_FINDINGS_TEXT_EXPORT`). The
//! render function is driven with a bare `ReportBuilder` and its components
//! are recorded as JSON, so the export covers the texts built inside it
//! without a full report. Run it before and after a change of how the texts
//! are stored and compare the two files byte for byte.

use super::*;
use crate::audit::AuditReport;
use crate::model::{ReportLevel, WcagLevel};
use crate::output::report_model::ReportConfig;
use crate::wcag::{Severity, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// Every code `report_code_label` knows, plus one it does not know.
const CODES: [&str; 11] = [
    "very_high",
    "high",
    "medium",
    "low",
    "very_low",
    "immediate",
    "quick_win",
    "normal",
    "manual_review_recommended",
    "automatically_confirmed",
    "unknown",
];

const SEVERITIES: [Severity; 4] = [
    Severity::Critical,
    Severity::High,
    Severity::Medium,
    Severity::Low,
];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

/// The first finding group of a report with one violation, in `lang`.
fn group(lang: &str) -> FindingGroup {
    let mut results = WcagResults::new();
    results.nodes_checked = 50;
    results.passes = 10;
    results.add_violation(
        crate::wcag::Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Image missing alt. Second sentence.",
            "node-img",
        )
        .with_rule_id("images/alt-missing")
        .with_selector("div.main > img.hero")
        .with_html_snippet("<img class=\"hero\">")
        .with_fix("Add alt"),
    );
    let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, results, 100);
    let normalized = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    let config = ReportConfig {
        level: ReportLevel::Standard,
        locale: lang.to_string(),
        ..ReportConfig::default()
    };
    crate::output::builder::build_view_model(&normalized, &config)
        .findings
        .all_findings
        .remove(0)
}

/// An occurrence with every evidence row `render_finding_technical` knows:
/// DOM path, contrast (with or without the required ratio), dark color
/// scheme, a screenshot from `viewport` and a suggested fix.
fn occurrence(
    index: usize,
    required: bool,
    viewport: Option<&'static str>,
) -> RepresentativeOccurrence {
    let mut evidence = vec![
        Evidence::new("ax_tree")
            .with_field("dom_path")
            .with_value("html > body > div.main > img"),
        Evidence::computed("contrast_ratio", "2.56"),
        Evidence::computed(
            crate::wcag::types::COLOR_SCHEME_EVIDENCE_FIELD,
            crate::wcag::types::COLOR_SCHEME_DARK,
        ),
    ];
    if required {
        evidence.push(Evidence::computed("required_ratio", "4.5"));
    }
    RepresentativeOccurrence {
        selector: format!("div.main > img.hero-{index}"),
        node_id: format!("img-{index}"),
        message: "Image missing alt. Second sentence.".to_string(),
        html_snippet: Some("<img class=\"hero\">".to_string()),
        suggested_code: Some("<img class=\"hero\" alt=\"\">".to_string()),
        evidence,
        evidence_screenshot: Some(vec![0u8; 4]),
        evidence_viewport: viewport,
    }
}

fn export_render_finding_technical(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let mut evidence_seq = 0usize;
    let mut record = |records: &mut Vec<Value>, input: String, group: &FindingGroup, expanded| {
        let mut acronyms_expanded = expanded;
        let rendered = render_finding_technical(
            bare(),
            group,
            i18n,
            0,
            &mut evidence_seq,
            &mut acronyms_expanded,
        );
        records.push(json!([
            "render_finding_technical",
            input,
            lang,
            components(rendered)
        ]));
    };

    // Severity prefix, the code labels in every metadata row, the BFSG/EAA
    // label spelled out and abbreviated.
    for severity in SEVERITIES {
        for code in CODES {
            for expanded in [false, true] {
                let mut g = group(lang);
                g.finding.severity = severity;
                g.finding.remediation_priority = code.to_string();
                g.finding.confidence = code.to_string();
                g.finding.false_positive_risk = code.to_string();
                g.finding.bfsg_relevance = code.to_string();
                record(
                    records,
                    format!("severity={severity:?} code={code} acronyms_expanded={expanded}"),
                    &g,
                    expanded,
                );
            }
        }
    }

    // Action assessment: verification, self-test (1.3.1 and the content-model
    // exception) and none.
    for (criterion, rule_id) in [
        ("1.3.1", "structure/heading-order"),
        ("1.3.1", "html-content-model/list"),
        ("1.1.1", "images/alt-missing"),
    ] {
        for verification in ["manual_review_recommended", "automatically_confirmed"] {
            let mut g = group(lang);
            g.finding.wcag_criterion = criterion.to_string();
            g.finding.rule_id = rule_id.to_string();
            g.finding.verification = verification.to_string();
            record(
                records,
                format!("criterion={criterion} rule={rule_id} verification={verification}"),
                &g,
                true,
            );
        }
    }

    // Occurrences: evidence rows, both screenshot captions and the hidden
    // occurrence note (none, one, several).
    for count in [1usize, 4, 5, 15] {
        for required in [true, false] {
            for viewport in [Some("mobile"), Some("desktop"), None] {
                let mut g = group(lang);
                g.representative_occurrences = (0..count)
                    .map(|i| occurrence(i, required, viewport))
                    .collect();
                record(
                    records,
                    format!("occurrences={count} required={required} viewport={viewport:?}"),
                    &g,
                    true,
                );
            }
        }
    }

    // Structural cause: likely root cause (component) and structural cause.
    for component in [true, false] {
        let mut g = group(lang);
        g.structural_cause = Some("Shared template.".to_string());
        g.is_component_issue = component;
        record(
            records,
            format!("structural_cause component={component}"),
            &g,
            true,
        );
    }
}

/// The optional blocks of `render_finding_technical` one by one and together:
/// header variants, lead-in, reference row, assessment rows, element types,
/// recommendation, examples, affected URLs, location-less findings, the
/// html-conform location, occurrence snapshots without optional rows, pattern
/// clusters and the empty finding.
fn export_render_finding_blocks(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    use crate::audit::normalized::{ComplexityKind, ExpectedImpactKind};
    let mut evidence_seq = 0usize;
    let mut record = |records: &mut Vec<Value>, input: String, group: &FindingGroup| {
        let mut acronyms_expanded = false;
        let rendered = render_finding_technical(
            bare(),
            group,
            i18n,
            7,
            &mut evidence_seq,
            &mut acronyms_expanded,
        );
        records.push(json!([
            "render_finding_technical blocks",
            input,
            lang,
            components(rendered)
        ]));
    };
    let plain = |selector: &str, node_id: &str| RepresentativeOccurrence {
        selector: selector.to_string(),
        node_id: node_id.to_string(),
        message: "Image missing alt. Second sentence.".to_string(),
        html_snippet: None,
        suggested_code: None,
        evidence: vec![],
        evidence_screenshot: None,
        evidence_viewport: None,
    };

    // Header: criterion with and without the WCAG 2.2 suffix, no criterion
    // with and without a title.
    for (criterion, title) in [
        ("1.1.1", "Title"),
        ("2.5.8", "Title"),
        ("", "Title"),
        ("", ""),
    ] {
        let mut g = group(lang);
        g.finding.wcag_criterion = criterion.to_string();
        g.title = title.to_string();
        record(
            records,
            format!("header criterion={criterion:?} title={title:?}"),
            &g,
        );
    }

    // Lead-in: description and impact, each empty, equal, different.
    for (description, impact) in [
        ("", ""),
        ("Description.", ""),
        ("", "Impact."),
        ("Same.", "Same."),
        ("Same.", " Same. "),
        ("Description.", "Impact."),
    ] {
        let mut g = group(lang);
        g.customer_description = description.to_string();
        g.user_impact = impact.to_string();
        record(
            records,
            format!("lead-in description={description:?} impact={impact:?}"),
            &g,
        );
    }

    // Reference row.
    for url in [None, Some("  "), Some(" https://example.com/ref ")] {
        let mut g = group(lang);
        g.finding.understanding_url = url.map(str::to_string);
        record(records, format!("understanding_url={url:?}"), &g);
    }

    // Action assessment rows: complexity and expected impact variants, with
    // and without the self-test.
    for (complexity, impact) in [
        (ComplexityKind::LowScope, ExpectedImpactKind::default()),
        (
            ComplexityKind::HighOccurrence {
                occurrence_count: 30,
            },
            ExpectedImpactKind::default(),
        ),
        (
            ComplexityKind::TechnicalPattern,
            ExpectedImpactKind::Wcag {
                occurrence_count: 3,
                score_effect: Default::default(),
                wcag_level: "AA".to_string(),
                score_gain: None,
            },
        ),
        (
            ComplexityKind::ModerateOccurrence {
                occurrence_count: 6,
            },
            ExpectedImpactKind::Other {
                occurrence_count: 6,
                score_effect: Default::default(),
            },
        ),
    ] {
        for criterion in ["1.1.1", "1.3.1"] {
            let mut g = group(lang);
            g.finding.wcag_criterion = criterion.to_string();
            g.finding.complexity_kind = complexity;
            g.finding.expected_impact_kind = impact.clone();
            record(
                records,
                format!("assessment {complexity:?} {impact:?} criterion={criterion}"),
                &g,
            );
        }
    }

    // Recommendation, examples (with and without decorative note), affected
    // URLs (none, one, ten, eleven), pattern clusters.
    for recommendation in ["", "Add an alt text."] {
        let mut g = group(lang);
        g.recommendation = recommendation.to_string();
        record(records, format!("recommendation={recommendation:?}"), &g);
    }
    for decorative in [None, Some("Decorative image.".to_string())] {
        let mut g = group(lang);
        g.examples = vec![
            ExampleBlock {
                bad: "<img src=a>".to_string(),
                good: "<img src=a alt=b>".to_string(),
                decorative: decorative.clone(),
            },
            ExampleBlock {
                bad: "<img src=c>".to_string(),
                good: "<img src=c alt=\"\">".to_string(),
                decorative: None,
            },
        ];
        record(records, format!("examples decorative={decorative:?}"), &g);
    }
    for count in [0usize, 1, 10, 11] {
        let mut g = group(lang);
        g.affected_urls = (0..count)
            .map(|i| format!("https://example.com/page-{i}/{}", "x".repeat(60)))
            .collect();
        record(records, format!("affected_urls={count}"), &g);
    }
    for clusters in [0usize, 2] {
        let mut g = group(lang);
        g.pattern_clusters = (0..clusters)
            .map(|i| FindingPatternCluster {
                label: format!("img.hero-{i} {}", "y".repeat(80)),
                occurrences: i + 2,
            })
            .collect();
        record(records, format!("pattern_clusters={clusters}"), &g);
    }

    // Locations: none real (selector equals node id), no occurrences at all,
    // node-id handles, html-conform positions.
    let mut g = group(lang);
    g.representative_occurrences = vec![plain("multiple_h1", "multiple_h1")];
    g.pattern_clusters = vec![FindingPatternCluster {
        label: "multiple".to_string(),
        occurrences: 2,
    }];
    g.structural_cause = Some("Shared template.".to_string());
    record(records, "no real location".to_string(), &g);

    let mut g = group(lang);
    g.representative_occurrences = Vec::new();
    record(records, "no occurrences".to_string(), &g);

    for node_id in ["", "document", "12345", "ax-12", "rule_key"] {
        let mut g = group(lang);
        g.representative_occurrences = vec![
            plain("div > img.a", node_id),
            plain("<section class=\"b\">", "x-1"),
            plain("a.link", "x-2"),
            plain("button", "x-3"),
        ];
        record(records, format!("node_id={node_id:?}"), &g);
    }

    for (selector, message) in [
        ("190:12271", "Element \"div\" not allowed here. More."),
        ("not a position", "No element named."),
        ("7:3", ""),
    ] {
        let mut g = group(lang);
        g.finding.rule_id = "html_conform.content-model".to_string();
        g.representative_occurrences = vec![RepresentativeOccurrence {
            html_snippet: Some("<div>".to_string()),
            message: message.to_string(),
            ..plain(selector, "html_conform.content-model")
        }];
        record(
            records,
            format!("html_conform selector={selector:?} message={message:?}"),
            &g,
        );
    }

    // Everything at once.
    let mut g = group(lang);
    g.structural_cause = Some("Shared template.".to_string());
    g.is_component_issue = true;
    g.recommendation = "Add an alt text.".to_string();
    g.affected_urls = vec!["https://example.com/a".to_string()];
    g.pattern_clusters = vec![FindingPatternCluster {
        label: "img.hero".to_string(),
        occurrences: 3,
    }];
    g.representative_occurrences = (0..5)
        .map(|i| occurrence(i, true, Some("mobile")))
        .collect();
    record(records, "all blocks".to_string(), &g);
}

fn export_pure_functions(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let en = lang == "en";
    for (required, evidence) in [
        (
            true,
            vec![
                Evidence::computed("contrast_ratio", "2.56"),
                Evidence::computed("required_ratio", "4.5"),
            ],
        ),
        (false, vec![Evidence::computed("contrast_ratio", "2.56:1")]),
        (false, vec![]),
    ] {
        records.push(json!([
            "contrast_measured_text",
            format!("required={required} evidence={}", evidence.len()),
            lang,
            contrast_measured_text(&evidence, en)
        ]));
    }
    let dimmed = vec![
        Evidence::computed("opacity", "0.55"),
        Evidence::computed("effective_foreground", "#8c8c8c"),
        Evidence::computed("state_dependent", "true"),
    ];
    records.push(json!([
        "opacity_text",
        "opacity=0.55",
        lang,
        opacity_text(&dimmed, en)
    ]));
    let occurrence = crate::audit::normalized::OccurrenceDetail {
        evidence: dimmed,
        ..Default::default()
    };
    records.push(json!([
        "opacity_pattern_text",
        "opacity=0.55",
        lang,
        opacity_pattern_text(&[occurrence], en)
    ]));
    for code in CODES {
        records.push(json!([
            "report_code_label",
            code,
            lang,
            report_code_label(code, i18n)
        ]));
    }
    for expanded in [false, true] {
        let mut flag = expanded;
        let label = bfsg_eaa_label(i18n, &mut flag);
        records.push(json!([
            "bfsg_eaa_label",
            format!("acronyms_expanded={expanded}"),
            lang,
            [label.to_string(), flag.to_string()]
        ]));
    }
    // One group per branch of the customer perspective.
    for (rule_id, title, dimension) in [
        ("buttons/name-missing", "Name", "Accessibility"),
        ("lists/empty", "Struktur", "Accessibility"),
        ("contrast/text-insufficient", "Kontrast", "Accessibility"),
        ("x/y", "Titel", "SEO"),
        ("x/y", "Header", "Security"),
        ("x/y", "Ladezeit", "Performance"),
        ("x/y", "Viewport", "Mobile"),
        ("x/y", "Sonstiges", "Other"),
    ] {
        let mut g = group(lang);
        g.title = title.to_string();
        g.finding.rule_id = rule_id.to_string();
        g.finding.dimension = dimension.to_string();
        g.finding.subcategory = String::new();
        records.push(json!([
            "customer_perspective_title",
            format!("{rule_id} {title} {dimension}"),
            lang,
            customer_perspective_title(&g, en)
        ]));
    }
}

/// Every record of the export, in both languages: `findings.rs` first, then
/// `detail_modules.rs`.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_render_finding_technical(&mut records, lang, &i18n);
        export_render_finding_blocks(&mut records, lang, &i18n);
        export_pure_functions(&mut records, lang, &i18n);
    }
    records.extend(crate::output::pdf::detail_modules::text_export::records());
    records
}

#[test]
#[ignore = "writes reports/pdf-findings-texts.json; run before and after a change of the text storage"]
fn export_pdf_findings_texts() {
    let records = records();
    let path = std::env::var("PDF_FINDINGS_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/pdf-findings-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} pdf findings text records written to {path}",
        records.len()
    );
}
