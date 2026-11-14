//! Text export of the single report frame in `pdf/mod.rs`, slice 18 of #936:
//! the cover labels, the table of contents, the three part intros with their
//! audience titles and bodies, the closing callouts of the executive level
//! and the confidence notes, across an enumerated input space in both
//! languages, as records in `reports/pdf-frame-texts.json` (gitignored; the
//! path can be set with `PDF_FRAME_TEXT_EXPORT`). The whole report is built
//! once per fixture, level and language and its components are recorded as
//! JSON; the confidence notes are driven with a bare `ReportBuilder` over
//! every audit flag kind. Run it before and after a change of how the texts
//! are stored and compare the two files byte for byte.

use super::*;
use crate::audit::normalized::AuditFlag;
use crate::model::WcagLevel;
use crate::wcag::{Severity, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

const LEVELS: [ReportLevel; 3] = [
    ReportLevel::Executive,
    ReportLevel::Standard,
    ReportLevel::Technical,
];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

/// A clean page and a page with one violation, at a fixed time so the cover
/// date does not move between runs.
fn fixtures() -> Vec<(&'static str, AuditReport)> {
    let mut clean = AuditReport::new(
        "https://example.com".into(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    let mut results = WcagResults::new();
    results.nodes_checked = 50;
    results.passes = 10;
    results.add_violation(
        crate::wcag::Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Image missing alt.",
            "node-img",
        )
        .with_rule_id("images/alt-missing")
        .with_selector("div.main > img.hero")
        .with_html_snippet("<img class=\"hero\">")
        .with_fix("Add alt"),
    );
    let mut dirty = AuditReport::new("https://example.com".into(), WcagLevel::AA, results, 100);
    let fixed = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    clean.timestamp = fixed;
    dirty.timestamp = fixed;
    vec![("clean", clean), ("one violation", dirty)]
}

fn flag(kind: &str, source: &str) -> AuditFlag {
    AuditFlag {
        kind: kind.to_string(),
        related_rule: None,
        source: source.to_string(),
        message: format!("Message of {kind}."),
    }
}

fn export_confidence_notes(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let kinds = [
        "consent_banner",
        "bypass_blocks_untested",
        "conflicting_signal",
        "viewport_gap",
        "incomplete_audit",
        "consent_wall_artifact",
        "unknown_kind",
    ];
    records.push(json!([
        "render_audit_confidence_notes",
        "no flags",
        lang,
        components(render_audit_confidence_notes(bare(), &[], i18n))
    ]));
    for kind in kinds {
        let flags = [flag(kind, "source")];
        records.push(json!([
            "render_audit_confidence_notes",
            kind,
            lang,
            components(render_audit_confidence_notes(bare(), &flags, i18n))
        ]));
    }
    let all: Vec<AuditFlag> = kinds.iter().map(|kind| flag(kind, "source")).collect();
    records.push(json!([
        "render_audit_confidence_notes",
        "all kinds together",
        lang,
        components(render_audit_confidence_notes(bare(), &all, i18n))
    ]));
    // The side checks that got no answer: each known id, both known ids, an
    // unknown id, repeated ids, and a mix.
    for source in [
        "seo.www_redirect",
        "seo.page_health.www_consolidation",
        "seo.www_redirect,seo.page_health.www_consolidation",
        "security.certificate",
        "other.check",
        "other.check,another.check",
        "security.certificate,other.check,security.certificate",
        "seo.www_redirect,security.certificate,other.check",
        "",
    ] {
        let flags = [flag("side_check_unanswered", source)];
        records.push(json!([
            "render_audit_confidence_notes",
            format!("side_check_unanswered source={source}"),
            lang,
            components(render_audit_confidence_notes(bare(), &flags, i18n))
        ]));
    }
}

fn export_single_report(records: &mut Vec<Value>, lang: &str) {
    for (name, report) in fixtures() {
        for level in LEVELS {
            let config = ReportConfig {
                level,
                locale: lang.to_string(),
                ..ReportConfig::default()
            };
            let (_engine, built) = build_single_report(&report, &config, None)
                .expect("fixture report passes the report contract");
            records.push(json!([
                "build_single_report",
                format!("{name} level={level}"),
                lang,
                Value::Array(built.components)
            ]));
        }
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_single_report(&mut records, lang);
        export_confidence_notes(&mut records, lang, &i18n);
    }
    records
}

#[test]
#[ignore = "writes reports/pdf-frame-texts.json; run before and after a change of the text storage"]
fn export_pdf_frame_texts() {
    let records = records();
    let path = std::env::var("PDF_FRAME_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/pdf-frame-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} pdf frame text records written to {path}", records.len());
}
