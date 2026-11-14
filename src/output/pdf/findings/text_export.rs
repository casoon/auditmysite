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
