//! Text export of `score_drivers.rs`, slice 15 of #936: everything the module
//! puts into the report, across an enumerated input space in both languages,
//! as records in `reports/score-drivers-texts.json` (gitignored; the path can
//! be set with `SCORE_DRIVERS_TEXT_EXPORT`). The render functions are driven
//! with a bare `ReportBuilder` and their components are recorded as JSON, so
//! the export covers the texts built inside them without a full report. Run
//! it before and after a change of how the texts are stored and compare the
//! two files byte for byte.

use super::*;
use crate::audit::normalized::{ModuleScoreEntry, ScoreBreakdown};
use crate::audit::AuditReport;
use crate::model::{ReportLevel, WcagLevel};
use crate::wcag::WcagResults;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

/// A view model with a security presentation, so the security row's
/// corrected band can be exercised.
fn view_model(lang: &str) -> ReportViewModel {
    let ssl = crate::security::SslInfo {
        https: true,
        valid_certificate: true,
        ..Default::default()
    };
    let headers = crate::security::SecurityHeaders::default();
    let issues = crate::security::generate_security_issues(&headers, true);
    let score = crate::security::calculate_security_score(&headers, &ssl, &issues);
    let security = crate::security::SecurityAnalysis {
        score,
        grade: String::new(),
        headers,
        ssl,
        issues,
        recommendations: vec![],
        protection: Default::default(),
        sourcemap_leaks: Default::default(),
        fetch: Default::default(),
    };
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    )
    .with_security(security);
    let normalized = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    let config = ReportConfig {
        level: ReportLevel::Standard,
        locale: lang.to_string(),
        ..ReportConfig::default()
    };
    crate::output::builder::build_view_model(&normalized, &config)
}

fn entry(name: &str, score: u32, contributes: bool) -> ModuleScoreEntry {
    ModuleScoreEntry::new(name, score, "measured", contributes)
}

/// Module sets: none, one, all six with a weakest module, other drags, a
/// stabilizing module and a non-contributing one, all six above "Good", a
/// subset (renormalised, modules missing), all six with a lowered weight
/// (renormalised, none missing) and a module without a localized name.
fn module_sets() -> Vec<(&'static str, Vec<ModuleScoreEntry>)> {
    let mut lowered = vec![
        entry("Accessibility", 82, true),
        entry("Performance", 77, true),
        entry("SEO", 90, true),
        entry("Security", 80, true),
        entry("Mobile", 95, true),
        entry("HTML Conformance", 75, true),
    ];
    lowered[1].weight_pct = 10;
    let mut custom = entry("Custom Module", 30, true);
    custom.weight_pct = 5;
    vec![
        ("none", vec![]),
        ("one", vec![entry("Accessibility", 40, true)]),
        (
            "all six, mixed",
            vec![
                entry("Accessibility", 40, true),
                entry("Performance", 60, true),
                entry("SEO", 90, true),
                entry("Security", 80, true),
                entry("Mobile", 100, true),
                entry("HTML Conformance", 74, true),
                entry("UX", 50, false),
            ],
        ),
        (
            "all six, good",
            vec![
                entry("Accessibility", 92, true),
                entry("Performance", 75, true),
                entry("SEO", 88, true),
                entry("Security", 90, true),
                entry("Mobile", 100, true),
                entry("HTML Conformance", 99, true),
            ],
        ),
        (
            "subset",
            vec![entry("Accessibility", 70, true), entry("SEO", 80, true)],
        ),
        ("all six, lowered weight", lowered),
        (
            "unknown module",
            vec![entry("Accessibility", 76, true), custom],
        ),
    ]
}

fn breakdowns() -> Vec<(&'static str, Option<ScoreBreakdown>)> {
    let breakdown = |security: Option<u32>| ScoreBreakdown {
        calculation_note: String::new(),
        desktop_weight_pct: 30,
        mobile_weight_pct: 70,
        desktop_accessibility: 94,
        mobile_accessibility: 91,
        viewport_blended_accessibility: 92,
        desktop_overall: 94,
        mobile_overall: 91,
        viewport_blended_overall: 92,
        viewport_blend_weight_pct: 90,
        security_score: security,
        security_weight_pct: security.map(|_| 10),
    };
    vec![
        ("no breakdown", None),
        ("security mixed in", Some(breakdown(Some(95)))),
        ("security not measured", Some(breakdown(None))),
    ]
}

fn export_score_driver_table(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for (set_input, set) in module_sets() {
        let security_score = set
            .iter()
            .find(|m| m.name == "Security")
            .map_or(80, |m| m.score);
        for (security_input, band) in [
            ("no security detail", None),
            (
                "security band as score",
                Some(score_band_label(security_score, i18n)),
            ),
            ("security band lowered", Some(score_band_label(30, i18n))),
        ] {
            for method in ["weighted", "viewport_weighted"] {
                for (breakdown_input, breakdown) in breakdowns() {
                    let mut vm = view_model(lang);
                    vm.summary.overall_score = 72;
                    vm.modules.module_scores = set.clone();
                    vm.summary.score_calculation_method = method.to_string();
                    vm.summary.score_breakdown = breakdown;
                    match band {
                        None => vm.module_details.security = None,
                        Some(band) => {
                            vm.module_details
                                .security
                                .as_mut()
                                .expect("fixture has a security presentation")
                                .band_label = band.to_string();
                        }
                    }
                    records.push(json!([
                        "render_score_driver_table",
                        format!(
                            "{set_input}, {security_input}, method={method}, {breakdown_input}"
                        ),
                        lang,
                        components(render_score_driver_table(bare(), &vm, i18n))
                    ]));
                }
            }
        }
    }
}

fn export_additional_score_drivers(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let mut vm = view_model(lang);
    for len in [0, 3, 4, 6] {
        vm.summary.technical_overview = (0..len).map(|i| format!("Bullet {i}")).collect();
        records.push(json!([
            "render_additional_score_drivers",
            format!("bullets={len}"),
            lang,
            components(render_additional_score_drivers(bare(), &vm, i18n))
        ]));
    }
}

fn export_subcategory_breakdown(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let title = i18n.t("module-accessibility");
    for mask in 0..32u32 {
        let mut entries: Vec<(String, u32)> = Vec::new();
        if mask & 1 != 0 {
            entries.push(("ARIA".to_string(), 59));
            entries.push(("Landmarks".to_string(), 10));
        }
        if mask & 2 != 0 {
            entries.push(("Forms".to_string(), 60));
            entries.push(("Images".to_string(), 99));
        }
        if mask & 4 != 0 {
            entries.push(("Semantics".to_string(), 100));
            entries.push(("Media".to_string(), 100));
        }
        let mut critical: Vec<(String, String)> = Vec::new();
        if mask & 8 != 0 {
            critical.push((
                "Link without an accessible name".to_string(),
                "Technical Robustness".to_string(),
            ));
        }
        if mask & 16 != 0 {
            critical.push(("Missing alt text".to_string(), "Images".to_string()));
        }
        records.push(json!([
            "render_subcategory_breakdown",
            format!(
                "weak={} partial={} clean={} critical={}",
                mask & 1 != 0,
                mask & 2 != 0,
                mask & 4 != 0,
                critical.len()
            ),
            lang,
            components(render_subcategory_breakdown(
                bare(),
                &title,
                &entries,
                &critical,
                i18n
            ))
        ]));
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_score_driver_table(&mut records, lang, &i18n);
        export_additional_score_drivers(&mut records, lang, &i18n);
        export_subcategory_breakdown(&mut records, lang, &i18n);
    }
    records
}

#[test]
#[ignore = "writes reports/score-drivers-texts.json; run before and after a change of the text storage"]
fn export_score_drivers_texts() {
    let records = records();
    let path = std::env::var("SCORE_DRIVERS_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/score-drivers-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} score driver text records written to {path}",
        records.len()
    );
}
