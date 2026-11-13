//! Text export of the methodology section of the single report
//! (`methodology.rs`), slice 10 of #936: every text the module builds,
//! across an enumerated input space in both languages, as records in
//! `reports/methodology-texts.json` (gitignored; the path can be set with
//! `METHODOLOGY_TEXT_EXPORT`). Each scenario changes one input of a checked
//! clean report (or a few together) and records the score methodology of the
//! appendix, the methodology block with its audit facts, the confidence
//! summary and the capability matrix. Run it before and after a change of how
//! the texts are stored and compare the two files byte for byte.

use super::*;
use crate::audit::normalized::{AuditFlag, ModuleScoreEntry};
use crate::audit::{
    AuditQualityStatus, AuditReport, ExecutionStatus, ModuleRun, ViewportScoreSet, ViewportScores,
};
use crate::i18n::I18n;
use crate::model::WcagLevel;
use crate::wcag::WcagResults;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// A clean page, through the report contract.
fn base() -> NormalizedReport {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        1_234,
    );
    crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract")
        .normalized
}

fn module(
    name: &str,
    weight_pct: u32,
    contributes: bool,
    measurement_type: &str,
) -> ModuleScoreEntry {
    ModuleScoreEntry {
        name: name.to_string(),
        score: 80,
        grade: None,
        band: String::new(),
        weight_pct,
        contributes_to_overall: contributes,
        derived_from: Vec::new(),
        measurement_type: measurement_type.to_string(),
    }
}

fn run(name: &str, status: ExecutionStatus) -> ModuleRun {
    ModuleRun {
        module: name.to_string(),
        status,
        ..ModuleRun::default()
    }
}

fn flag(n: usize) -> AuditFlag {
    AuditFlag {
        kind: format!("kind-{n}"),
        related_rule: None,
        source: "export".to_string(),
        message: "message".to_string(),
    }
}

fn viewport_scores() -> ViewportScores {
    ViewportScores {
        desktop: ViewportScoreSet {
            accessibility: 91,
            performance: None,
            overall: 90,
        },
        mobile: ViewportScoreSet {
            accessibility: 77,
            performance: None,
            overall: 76,
        },
        weighted_overall: 81,
    }
}

/// The scenarios: a name and the report it describes.
fn scenarios() -> Vec<(String, NormalizedReport)> {
    let mut out: Vec<(String, NormalizedReport)> = Vec::new();
    let mut push = |name: &str, change: &dyn Fn(&mut NormalizedReport)| {
        let mut report = base();
        change(&mut report);
        out.push((name.to_string(), report));
    };

    push("base", &|_| {});
    push("qualified_results", &|r| {
        r.execution.quality.qualified_results = true;
    });
    push("screenshots", &|r| r.has_screenshots = true);
    push("viewport_scores", &|r| {
        r.viewport_scores = Some(viewport_scores())
    });
    for status in [
        AuditQualityStatus::Complete,
        AuditQualityStatus::Partial,
        AuditQualityStatus::Insufficient,
    ] {
        push(&format!("quality={status:?}"), &move |r| {
            r.execution.quality.status = status;
        });
    }
    push("modules=none", &|r| r.module_scores.clear());
    push("modules=contributing", &|r| {
        r.module_scores = vec![
            module("Accessibility", 60, true, "measured"),
            module("Performance", 40, true, "measured"),
        ];
    });
    push("modules=indicators", &|r| {
        r.module_scores = vec![
            module("Accessibility", 50, true, "measured"),
            module("Mobile", 25, true, "heuristic"),
            module("Security", 25, true, "measured"),
            module("UX", 0, false, "heuristic"),
            module("AI Visibility", 0, false, "heuristic"),
        ];
    });
    push("modules=indicators-only", &|r| {
        r.module_scores = vec![module("UX", 0, false, "heuristic")];
    });
    push("modules=three", &|r| {
        r.module_scores = vec![
            module("Accessibility", 70, true, "measured"),
            module("SEO", 20, true, "measured"),
            module("Security", 10, true, "measured"),
        ];
    });
    push("modules=five", &|r| {
        r.module_scores = vec![
            module("Accessibility", 40, true, "measured"),
            module("Performance", 20, true, "measured"),
            module("SEO", 20, true, "measured"),
            module("Security", 10, true, "measured"),
            module("Mobile", 10, true, "measured"),
        ];
    });
    push("runs=mixed", &|r| {
        r.execution.scope.requested_modules = vec![
            "accessibility".to_string(),
            "performance".to_string(),
            "seo".to_string(),
        ];
        r.execution.module_runs = vec![
            run("accessibility", ExecutionStatus::Completed),
            run("performance", ExecutionStatus::Partial),
            run("seo", ExecutionStatus::Failed),
            run("security", ExecutionStatus::Failed),
        ];
    });
    push("runs=none", &|r| {
        r.execution.scope.requested_modules.clear();
        r.execution.module_runs.clear();
    });
    for count in [1, 2, 3] {
        push(&format!("flags={count}"), &move |r| {
            r.audit_flags = (0..count).map(flag).collect();
        });
    }
    for nodes in [0, 499, 500, 1_999, 2_000, 1_234_567] {
        push(&format!("nodes={nodes}"), &move |r| {
            r.nodes_analyzed = nodes
        });
    }
    push("counts=large", &|r| {
        r.severity_counts.total = 1_234_567;
        r.occurrence_counts.total = 7_654_321;
        r.overall_score = 100;
        r.score = 0;
        r.duration_ms = 9_876_543;
    });
    push("combined", &|r| {
        r.execution.quality.qualified_results = true;
        r.execution.quality.status = AuditQualityStatus::Insufficient;
        r.has_screenshots = true;
        r.viewport_scores = Some(viewport_scores());
        r.audit_flags = (0..2).map(flag).collect();
        r.nodes_analyzed = 2_500;
        r.module_scores = vec![
            module("Accessibility", 45, true, "measured"),
            module("Performance", 20, true, "measured"),
            module("SEO", 20, true, "measured"),
            module("Security", 15, true, "measured"),
            module("UX", 0, false, "heuristic"),
        ];
    });
    out
}

fn methodology_record(block: &MethodologyBlock) -> Value {
    json!({
        "scope": block.scope,
        "method": block.method,
        "limitations": block.limitations,
        "disclaimer": block.disclaimer,
        "audit_facts": block.audit_facts,
        "confidence_summary": block.confidence_summary,
        "capabilities": block
            .capabilities
            .iter()
            .map(|cap| json!({
                "signal": cap.signal,
                "source": cap.source,
                "confidence": cap.confidence,
                "surfaces": cap.surfaces,
                "note": cap.note,
            }))
            .collect::<Vec<_>>(),
    })
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        let appendix = build_appendix_block_from_normalized(lang, &base());
        records.push(json!([
            "build_appendix_block_from_normalized",
            "base",
            lang,
            appendix.score_methodology
        ]));
        for (name, report) in scenarios() {
            records.push(json!([
                "build_methodology",
                name,
                lang,
                methodology_record(&build_methodology(&i18n, &report))
            ]));
        }
    }
    records
}

#[test]
#[ignore = "writes reports/methodology-texts.json; run before and after a change of the text storage"]
fn export_methodology_texts() {
    let records = records();
    let path = std::env::var("METHODOLOGY_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/methodology-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} methodology text records written to {path}",
        records.len()
    );
}
