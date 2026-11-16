//! Text export of the terminal table's experimental indicator lines
//! (`terminal.rs`), slice 26 of #936: every text `experimental_lines` and the
//! scope note of `render_single_report` print, in both languages, as records
//! in `reports/terminal-texts.json` (gitignored; the path can be set with
//! `TERMINAL_TEXT_EXPORT`). Run it before and after a change of how the texts
//! are stored and compare the two files byte for byte.

use super::*;
use crate::audit::ExperimentalIndicators;
use crate::commerce::{CommerceAnalysis, CommercePageKind};
use crate::content_visibility::ContentVisibilityAnalysis;
use crate::wcag::WcagResults;
use serde_json::{json, Value};

fn report() -> AuditReport {
    AuditReport::new(
        "https://example.com".to_string(),
        crate::model::WcagLevel::AA,
        WcagResults::new(),
        42,
    )
}

fn commerce() -> CommerceAnalysis {
    CommerceAnalysis {
        page_kind: CommercePageKind::Other,
        product: None,
        trust_pages: Default::default(),
        identification_function_detected: false,
        identification_function_bfsg_reference: None,
        findings: Vec::new(),
    }
}

/// The indicator sets of the export: each indicator alone, then all together.
fn scenarios() -> Vec<(String, ExperimentalIndicators)> {
    let tree = crate::AXTree::new();
    let mut out: Vec<(String, ExperimentalIndicators)> = Vec::new();
    let mut push = |name: &str, change: &dyn Fn(&mut ExperimentalIndicators)| {
        let mut set = ExperimentalIndicators::default();
        change(&mut set);
        out.push((name.to_string(), set));
    };
    push("none", &|_| {});
    push("ux", &|s| s.ux = Some(crate::analyze_ux(&tree)));
    push("journey", &|s| {
        s.journey = Some(crate::page_flow::analyze_journey(&tree))
    });
    push("ai_visibility", &|s| {
        s.ai_visibility = Some(crate::ai_visibility::analyze_ai_visibility(&report()))
    });
    push("source_quality", &|s| {
        s.source_quality = Some(crate::source_quality::analyze_source_quality(&report()))
    });
    for (signals, problems) in [(0, 0), (7, 2), (1_234_567, 1)] {
        push(
            &format!("content_visibility {problems}/{signals}"),
            &move |s| {
                s.content_visibility = Some(ContentVisibilityAnalysis {
                    signal_count: signals,
                    problem_count: problems,
                    ..ContentVisibilityAnalysis::default()
                })
            },
        );
    }
    push("commerce", &|s| s.commerce = Some(commerce()));
    push("all", &|s| {
        s.ux = Some(crate::analyze_ux(&tree));
        s.journey = Some(crate::page_flow::analyze_journey(&tree));
        s.ai_visibility = Some(crate::ai_visibility::analyze_ai_visibility(&report()));
        s.source_quality = Some(crate::source_quality::analyze_source_quality(&report()));
        s.content_visibility = Some(ContentVisibilityAnalysis::default());
        s.commerce = Some(commerce());
    });
    out
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records = Vec::new();
    for lang in ["en", "de"] {
        for (name, set) in scenarios() {
            let lines = experimental_lines(&set, lang != "de");
            let options = TerminalOptions {
                color: ColorPolicy::Never,
                lang: lang.to_string(),
                ..TerminalOptions::default()
            };
            let mut report = report();
            report.experimental = set;
            let table = render_single_report(&report, &options).unwrap();
            let note: Vec<&str> = table
                .lines()
                .filter(|line| line.contains("indicators") || line.contains("Indikatoren"))
                .collect();
            records.push(json!({
                "scenario": name,
                "lang": lang,
                "lines": lines,
                "scope_note": note,
            }));
        }
    }
    records
}

#[test]
#[ignore = "writes reports/terminal-texts.json; run before and after a change of the text storage"]
fn export_terminal_texts() {
    let records = records();
    let path = std::env::var("TERMINAL_TEXT_EXPORT")
        .unwrap_or_else(|_| format!("{}/reports/terminal-texts.json", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} terminal text records written to {path}", records.len());
}
