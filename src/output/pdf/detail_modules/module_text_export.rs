//! Text export of `indicators.rs`, `seo.rs` and `performance.rs` of the PDF
//! detail modules, written together with the builder helper, batch builder
//! and display texts (`output::builder::batch::text_export`), slice 21 of
//! #936: everything the render functions put into the report, across an
//! enumerated input space in both languages, as records in
//! `reports/builder-detail-texts.json` (gitignored; the path can be set with
//! `BUILDER_DETAIL_TEXT_EXPORT`). The render functions are driven with a bare
//! `ReportBuilder` and their components are recorded as JSON. Run it before
//! and after a change of how the texts are stored and compare the two files
//! byte for byte.

use super::*;
use crate::best_practices::{
    BestPracticesAnalysis, ConsoleErrorsAnalysis, ConsoleMessage, VulnerableLibrariesAnalysis,
};
use crate::tech_stack::{Confidence, DetectedTech, TechCategory, TechStackAnalysis};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

fn tech(name: &str, confidence: Confidence) -> DetectedTech {
    DetectedTech {
        name: name.to_string(),
        category: TechCategory::Cms,
        version: Some("1.2".to_string()),
        confidence,
        signals: vec!["meta generator".to_string()],
    }
}

fn console(collected: bool, errors: usize) -> ConsoleErrorsAnalysis {
    ConsoleErrorsAnalysis {
        errors: (0..errors)
            .map(|i| ConsoleMessage {
                level: "error".to_string(),
                message: format!("Uncaught error {i}"),
            })
            .collect(),
        warnings: vec![],
        error_count: errors,
        warning_count: 0,
        collected,
    }
}

fn libraries(checked: bool, vulnerable: bool) -> VulnerableLibrariesAnalysis {
    VulnerableLibrariesAnalysis {
        detected: vec![],
        vulnerable: vec![],
        has_vulnerabilities: vulnerable,
        duplicate_libraries: vec![],
        checked,
    }
}

fn vitals(n: usize) -> Vec<(String, String, String)> {
    [
        ("LCP", "2.500s", "good"),
        ("FCP", "1.800s", "needs-improvement"),
        ("CLS", "0.150", "poor"),
        ("TTFB", "800ms", "good"),
        ("TBT", "1100ms", "needs-improvement"),
        ("TTI", "3.200s", "unknown"),
        ("Speed Index", "-0.5s", "poor"),
    ]
    .iter()
    .take(n)
    .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
    .collect()
}

fn viewport(score: u32, n: usize) -> PerformanceViewport {
    PerformanceViewport {
        score,
        grade: "Gold".to_string(),
        vitals: vitals(n),
    }
}

fn performance(
    score: u32,
    desktop: Option<PerformanceViewport>,
    mobile: Option<PerformanceViewport>,
    flat: usize,
) -> PerformancePresentation {
    PerformancePresentation {
        score,
        grade: "Gold".to_string(),
        interpretation: "Interpretation of the score.".to_string(),
        vitals: vitals(flat),
        desktop,
        mobile,
        additional_metrics: vec![],
        resource_ratings: vec![],
        recommendations: vec![],
        render_blocking_metrics: vec![],
        render_blocking_suggestions: vec![],
        has_render_blocking: false,
        throttled_profiles: vec![],
        cls_attribution: vec![],
        third_party: None,
        critical_chain: None,
        minification: None,
        coverage: None,
        animations: None,
        measurement_warnings: vec![],
    }
}

fn seo_profile(schema_count: usize) -> SeoProfilePresentation {
    SeoProfilePresentation {
        identity_summary: String::new(),
        site_name: "Example".to_string(),
        content_type: "Website".to_string(),
        language: "en".to_string(),
        category_hints: vec![],
        identity_facts: vec![],
        page_type: "Editorial".to_string(),
        page_attributes: vec![],
        content_depth_score: 50,
        structural_richness_score: 50,
        media_text_balance_score: 50,
        intent_fit_score: 50,
        page_profile_summary: String::new(),
        optimization_note: String::new(),
        page_profile_facts: vec![],
        schema_rows: (0..schema_count)
            .map(|i| (format!("Type{i}"), "80%".to_string(), "Details".to_string()))
            .collect(),
        schema_count,
        schema_status_summary: String::new(),
        schema_status_has_errors: false,
        schema_status_has_json_ld: false,
        schema_status_rows: vec![],
        schema_fit_summary: None,
        schema_fit_is_success: false,
        schema_fit_is_warning: false,
        schema_fit_facts: vec![],
        schema_rule_rows: vec![],
        schema_manual_review_rows: vec![],
        schema_parity_rows: vec![],
        signal_rows: vec![],
        signal_passed: 0,
        signal_total: 0,
        signal_details: vec![],
        maturity_level: "Basic".to_string(),
        maturity_description: String::new(),
        maturity_techniques_used: 1,
        maturity_techniques_total: 4,
    }
}

fn seo(profile: Option<SeoProfilePresentation>) -> SeoPresentation {
    SeoPresentation {
        score: 70,
        interpretation: "Interpretation of the score.".to_string(),
        meta_tags: vec![("Titel".to_string(), "A page title".to_string())],
        meta_issues: vec![],
        heading_summary: String::new(),
        social_summary: String::new(),
        technical_summary: vec![],
        tracking_summary: vec![],
        tracking_summary_text: String::new(),
        profile,
        robots: None,
        page_health: None,
        serp: None,
        image_efficiency: None,
        technical_issues: vec![],
    }
}

fn push(records: &mut Vec<Value>, name: &str, input: String, lang: &str, output: Value) {
    records.push(json!([name, input, lang, output]));
}

/// Every record of the export, in both languages.
fn records() -> Vec<Value> {
    let mut records = crate::output::builder::text_export_records();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();

        for (name, ts) in [
            ("none", TechStackAnalysis { detected: vec![] }),
            (
                "one",
                TechStackAnalysis {
                    detected: vec![tech("WordPress", Confidence::High)],
                },
            ),
            (
                "three",
                TechStackAnalysis {
                    detected: vec![
                        tech("WordPress", Confidence::High),
                        tech("jQuery", Confidence::Medium),
                        tech("Nginx", Confidence::Low),
                    ],
                },
            ),
        ] {
            for is_first in [true, false] {
                push(
                    &mut records,
                    "render_tech_stack",
                    format!("{name} first={is_first}"),
                    lang,
                    components(render_tech_stack(bare(), &ts, is_first, &i18n)),
                );
            }
        }

        for (name, collected, checked, errors, vulnerable, score) in [
            ("unmeasured", false, false, 0, false, 100),
            ("clean", true, true, 0, false, 100),
            ("clean, low score", true, true, 0, false, 80),
            ("console errors", true, true, 2, false, 85),
            ("vulnerable libraries", true, true, 0, true, 60),
            ("both", true, true, 3, true, 40),
            ("console not collected", false, true, 0, false, 95),
            ("libraries not checked", true, false, 0, false, 95),
        ] {
            let bp = BestPracticesAnalysis {
                console_errors: console(collected, errors),
                vulnerable_libraries: libraries(checked, vulnerable),
                score,
            };
            for is_first in [true, false] {
                push(
                    &mut records,
                    "render_best_practices",
                    format!("{name} first={is_first}"),
                    lang,
                    components(render_best_practices(bare(), &bp, is_first, &i18n)),
                );
            }
        }

        // The per-viewport section: the score is the mobile one, the
        // desktop one or neither; with and without the additional table.
        for (name, score, desktop, mobile) in [
            (
                "score of mobile, 7 vitals",
                60,
                viewport(80, 7),
                viewport(60, 7),
            ),
            (
                "score of desktop, 7 vitals",
                80,
                viewport(80, 7),
                viewport(60, 7),
            ),
            (
                "score of neither, 4 vitals",
                70,
                viewport(80, 4),
                viewport(60, 4),
            ),
            (
                "score of both, no vitals",
                70,
                viewport(70, 0),
                viewport(70, 0),
            ),
        ] {
            let perf = performance(score, Some(desktop), Some(mobile), 0);
            push(
                &mut records,
                "render_performance",
                name.to_string(),
                lang,
                components(render_performance(bare(), &perf, true, &i18n)),
            );
        }
        for (name, flat) in [("flat, 3 vitals", 3), ("flat, 7 vitals", 7), ("none", 0)] {
            let perf = performance(70, None, None, flat);
            push(
                &mut records,
                "render_performance",
                name.to_string(),
                lang,
                components(render_performance(bare(), &perf, false, &i18n)),
            );
        }
        {
            let mut perf = performance(70, None, None, 0);
            perf.minification = Some(MinificationPresentation {
                total_count: 2,
                total_savings_kb: 120.5,
                top_assets: vec![(
                    "https://example.com/a.js".to_string(),
                    "script".to_string(),
                    "60.2".to_string(),
                )],
                legacy_count: 1,
                legacy_wasted_kb: 12.3,
                legacy_assets: vec![(
                    "https://example.com/p.js".to_string(),
                    "core-js".to_string(),
                    "12595".to_string(),
                )],
            });
            push(
                &mut records,
                "render_performance",
                "minification and legacy scripts".to_string(),
                lang,
                components(render_performance(bare(), &perf, false, &i18n)),
            );
        }

        for (name, profile) in [
            ("no profile", None),
            ("no schema", Some(seo_profile(0))),
            ("one schema", Some(seo_profile(1))),
            ("two schemas", Some(seo_profile(2))),
            ("many schemas", Some(seo_profile(30))),
        ] {
            let seo = seo(profile);
            push(
                &mut records,
                "render_seo",
                name.to_string(),
                lang,
                components(render_seo(bare(), &seo, true, &i18n)),
            );
        }
    }
    records
}

#[test]
#[ignore = "writes reports/builder-detail-texts.json; run before and after a change of the text storage"]
fn export_builder_and_detail_module_texts() {
    let records = records();
    let path = std::env::var("BUILDER_DETAIL_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/builder-detail-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} builder and detail module text records written to {path}",
        records.len()
    );
}
