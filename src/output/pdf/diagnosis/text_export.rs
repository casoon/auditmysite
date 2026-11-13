//! Text export of the system diagnosis section of the single PDF
//! (`diagnosis.rs`), slice 12 of #936: everything `render_diagnosis_section`
//! puts into the report, across an enumerated input space in both languages,
//! as records in `reports/diagnosis-pdf-texts.json` (gitignored; the path can
//! be set with `DIAGNOSIS_PDF_TEXT_EXPORT`). The section is rendered into a
//! bare `ReportBuilder` from the diagnosis block of a real view model, with
//! and without a dominant issue, category table and cluster table, and with
//! fewer clusters than categories. Run it before and after a change of how
//! the texts are stored and compare the two files byte for byte.

use super::*;
use crate::audit::AuditReport;
use crate::model::{ReportLevel, WcagLevel};
use crate::wcag::{Severity, Violation, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// A change to the diagnosis block of a scenario.
type Change = fn(&mut DiagnosisBlock);

/// A page with two accessibility rules, two occurrences each.
fn report() -> AuditReport {
    let mut results = WcagResults::new();
    for (criterion, name, severity, rule_id) in [
        (
            "1.1.1",
            "Non-text Content",
            Severity::Critical,
            "images/alt-missing",
        ),
        (
            "4.1.2",
            "Name, Role, Value",
            Severity::High,
            "buttons/name-missing",
        ),
    ] {
        for node in ["node-1", "node-2"] {
            results.add_violation(
                Violation::new(criterion, name, WcagLevel::A, severity, "message", node)
                    .with_rule_id(rule_id),
            );
        }
    }
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        1_200,
    )
}

fn diagnosis(lang: &str) -> DiagnosisBlock {
    let report = report();
    let checked = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    let config = ReportConfig {
        level: ReportLevel::Standard,
        locale: lang.to_string(),
        ..ReportConfig::default()
    };
    crate::output::builder::build_view_model(&checked, &config).diagnosis
}

/// The scenarios: a name and the change to the diagnosis block.
fn scenarios() -> Vec<(&'static str, Change)> {
    vec![
        ("as-built", |_| {}),
        ("dominant=none", |d| d.dominant_issue = None),
        ("dominant=quoted", |d| {
            d.dominant_issue = Some("Images without \"alt\" text".to_string())
        }),
        ("categories=none clusters=none", |d| {
            d.category_breakdown.clear();
            d.clusters.clear();
        }),
        ("clusters=none", |d| d.clusters.clear()),
        ("clusters<categories", |d| {
            d.category_breakdown
                .push(("SEO".to_string(), 1, "Low".to_string()))
        }),
    ]
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for (name, change) in scenarios() {
            let mut block = diagnosis(lang);
            change(&mut block);
            let builder = render_diagnosis_section(
                renderreport::engine::ReportBuilder::new("export"),
                &block,
                &i18n,
            );
            records.push(json!([
                "render_diagnosis_section",
                name,
                lang,
                Value::Array(builder.build().components)
            ]));
        }
    }
    records
}

#[test]
#[ignore = "writes reports/diagnosis-pdf-texts.json; run before and after a change of the text storage"]
fn export_diagnosis_pdf_texts() {
    let records = records();
    let path = std::env::var("DIAGNOSIS_PDF_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/diagnosis-pdf-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} diagnosis PDF text records written to {path}",
        records.len()
    );
}
