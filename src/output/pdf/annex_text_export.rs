//! Text export of the annex and section helper texts of slice 25 of #936:
//! everything `en301549.rs`, `bik_guide.rs`, `detail_modules/overview.rs`,
//! `helpers.rs`, `cover.rs` and the appendix table of `appendix.rs` put into
//! the report, across an enumerated input space in both languages, as records
//! in `reports/pdf-annex-texts.json` (gitignored; the path can be set with
//! `PDF_ANNEX_TEXT_EXPORT`). The render functions are driven with a bare
//! `ReportBuilder` and their components are recorded as JSON; the single
//! report is built once per fixture, level, annex and language for the
//! appendix table. Run it before and after a change of how the texts are
//! stored and compare the two files byte for byte.

use super::bik_guide::render_bik_guide_annex;
use super::cover::{build_batch_cover_score_row, cover_band_phrase};
use super::detail_modules::{render_budget_violations, render_search_experience};
use super::en301549::render_en301549_annex;
use super::helpers::{manual_recheck_instruction, module_name_with_taxonomy_suffix};
use super::*;
use crate::audit::budget::BudgetViolation;
use crate::audit::normalized::{
    AccessibilityAssessment, InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues,
    NormalizedFinding,
};
use crate::audit::verdict::Verdict;
use crate::model::{AnnexKind, WcagLevel};
use crate::screen_reader::SrAuditIssue;
use crate::seo::technical::TechnicalIssue;
use crate::taxonomy::Severity as TaxonomySeverity;
use crate::wcag::{NotRun, RuleRun, Severity, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

fn violation(
    criterion: &str,
    name: &str,
    level: WcagLevel,
    rule_id: &str,
) -> crate::wcag::Violation {
    crate::wcag::Violation::new(
        criterion,
        name,
        level,
        Severity::High,
        "Fixture violation.",
        "node-1",
    )
    .with_rule_id(rule_id)
    .with_selector("div.main > img.hero")
    .with_html_snippet("<img class=\"hero\">")
    .with_fix("Fix it")
}

/// Pages with violations of the criteria whose self-check instructions
/// `manual_recheck_instruction` knows and of other criteria, at a fixed time
/// so the cover date does not move between runs.
fn report_fixtures() -> Vec<(&'static str, AuditReport)> {
    let fixed = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    let mut clean = AuditReport::new(
        "https://example.com".into(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    let mut results = WcagResults::new();
    results.nodes_checked = 50;
    results.passes = 10;
    results.add_violation(violation(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        "images/alt-missing",
    ));
    results.add_violation(violation(
        "1.4.3",
        "Contrast (Minimum)",
        WcagLevel::AA,
        "contrast/text-insufficient",
    ));
    results.add_violation(violation(
        "1.3.1",
        "Info and Relationships",
        WcagLevel::A,
        "forms/label-missing",
    ));
    let mut dirty = AuditReport::new("https://example.com".into(), WcagLevel::AA, results, 100);
    clean.timestamp = fixed;
    dirty.timestamp = fixed;
    vec![("clean", clean), ("violations", dirty)]
}

/// The normalized findings of the fixture with violations, as the template
/// for findings of any criterion.
fn finding_template() -> NormalizedFinding {
    let (_, report) = report_fixtures().remove(1);
    crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract")
        .normalized
        .findings
        .into_iter()
        .next()
        .expect("the fixture has a finding")
}

fn finding(criterion: &str, rule_id: &str, occurrences: usize) -> NormalizedFinding {
    let mut finding = finding_template();
    finding.wcag_criterion = criterion.to_string();
    finding.rule_id = rule_id.to_string();
    finding.occurrence_count = occurrences;
    finding
}

fn unmeasured(rule_id: &str, wcag: &[&str], why: NotRun) -> RuleRun {
    let mut run = RuleRun::not_run(rule_id, why);
    run.wcag = wcag.iter().map(|c| c.to_string()).collect();
    run
}

fn export_en301549(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    // Criteria with a clause: 1.1.1 / 1.4.3 / 2.4.4 / 3.3.2 are automated,
    // 1.2.1 / 1.2.2 manual; 1.4.6 and 2.4.9 are Level AAA and have none.
    let scenarios: Vec<(&str, Vec<NormalizedFinding>, Vec<RuleRun>)> = vec![
        ("no findings", vec![], vec![]),
        (
            "one violation",
            vec![finding("1.1.1", "images/alt-missing", 1)],
            vec![],
        ),
        (
            "several violations and rules",
            vec![
                finding("1.1.1", "images/alt-missing", 3),
                finding("1.1.1", "images/alt-suspicious", 2),
                finding("1.4.3", "contrast/text-insufficient", 12),
                finding("2.4.4", "links/generic-name", 1_234_567),
                finding("3.3.2", "forms/label-missing", 4),
            ],
            vec![],
        ),
        (
            "violations and one outside the standard",
            vec![
                finding("1.1.1", "images/alt-missing", 3),
                finding("1.4.6", "contrast/text-enhanced", 5),
            ],
            vec![],
        ),
        (
            "two outside the standard",
            vec![
                finding("1.4.6", "contrast/text-enhanced", 5),
                finding("2.4.9", "links/purpose-link-only", 2),
            ],
            vec![],
        ),
        (
            "unmeasured criteria",
            vec![finding("1.1.1", "images/alt-missing", 3)],
            vec![
                unmeasured("image-alt", &["1.1.1"], NotRun::Errored),
                unmeasured("color-contrast", &["1.4.3", "1.4.11"], NotRun::Errored),
                unmeasured("link-name", &["2.4.4"], NotRun::CapabilityMissing),
                RuleRun::ran("label", 0),
            ],
        ),
        (
            "unmeasured, nothing found",
            vec![],
            vec![unmeasured("link-name", &["2.4.4"], NotRun::Errored)],
        ),
    ];
    for (name, findings, outcomes) in scenarios {
        records.push(json!([
            "render_en301549_annex",
            name,
            lang,
            components(render_en301549_annex(bare(), &findings, &outcomes, i18n))
        ]));
    }
}

fn sr_issue(criterion: Option<&str>, nodes: usize) -> SrAuditIssue {
    SrAuditIssue {
        wcag_criterion: criterion.map(str::to_string),
        severity: "high".to_string(),
        affected_node_ids: (0..nodes).map(|n| format!("n{n}")).collect(),
        message: "Fixture issue.".to_string(),
        review: false,
    }
}

fn interactive(kind: InteractiveFindingKind, count: u32) -> InteractiveFinding {
    InteractiveFinding::new(
        "Test",
        kind,
        None,
        TaxonomySeverity::Medium,
        "test".into(),
        None,
        None,
        InteractiveFindingValues {
            count: Some(count),
            ..Default::default()
        },
    )
}

fn seo_issue(issue_type: &str) -> TechnicalIssue {
    TechnicalIssue {
        issue_type: issue_type.into(),
        message: "Fixture message.".into(),
        severity: TaxonomySeverity::Low,
    }
}

fn assessment(kind: &str, criterion: &str) -> AccessibilityAssessment {
    AccessibilityAssessment {
        kind: kind.to_string(),
        rule_id: format!("assess/{criterion}"),
        wcag_criterion: criterion.to_string(),
        severity: TaxonomySeverity::Medium,
        message: "Fixture assessment.".to_string(),
        fix_suggestion: None,
        selector: None,
        viewport: None,
        evidence: vec![],
    }
}

fn export_bik(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    struct Scenario {
        name: &'static str,
        findings: Vec<NormalizedFinding>,
        interactive: Vec<InteractiveFinding>,
        sr: Vec<SrAuditIssue>,
        seo: Vec<TechnicalIssue>,
        easy_language: bool,
        assessments: Vec<AccessibilityAssessment>,
    }
    let none = |name| Scenario {
        name,
        findings: vec![],
        interactive: vec![],
        sr: vec![],
        seo: vec![],
        easy_language: false,
        assessments: vec![],
    };
    let scenarios = vec![
        none("nothing found, easy language absent"),
        Scenario {
            easy_language: true,
            ..none("easy language detected")
        },
        Scenario {
            findings: vec![finding("1.1.1", "images/alt-missing", 3)],
            ..none("image finding")
        },
        Scenario {
            seo: vec![seo_issue("og_image_missing_alt")],
            ..none("image finding from the SEO issues")
        },
        Scenario {
            interactive: vec![interactive(InteractiveFindingKind::LinkTextGeneric, 4)],
            ..none("generic link text")
        },
        Scenario {
            sr: vec![sr_issue(Some("1.3.1"), 3), sr_issue(Some("1.3.6"), 0)],
            ..none("structure issues")
        },
        Scenario {
            findings: vec![finding("1.2.2", "video/caption", 1_234_567)],
            assessments: vec![assessment("manual_review", "1.2.1")],
            interactive: vec![
                interactive(InteractiveFindingKind::MediaControlsMissingName, 2),
                interactive(InteractiveFindingKind::MediaControlsNotReachable, 1),
            ],
            ..none("video findings")
        },
        Scenario {
            findings: vec![
                finding("1.1.1", "images/alt-missing", 3),
                finding("1.2.2", "video/caption", 1),
            ],
            interactive: vec![interactive(InteractiveFindingKind::LinkTextGeneric, 1)],
            sr: vec![sr_issue(Some("1.3.1"), 2)],
            seo: vec![seo_issue("twitter_image_missing_alt")],
            easy_language: true,
            assessments: vec![assessment("warning", "1.2.3")],
            name: "every chapter with findings",
        },
    ];
    for s in scenarios {
        records.push(json!([
            "render_bik_guide_annex",
            s.name,
            lang,
            components(render_bik_guide_annex(
                bare(),
                &s.findings,
                &s.interactive,
                &s.sr,
                &s.seo,
                s.easy_language,
                &s.assessments,
                i18n,
            ))
        ]));
    }
}

fn export_search_experience(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for score in [0, 40, 74, 75, 100] {
        for (warnings, components_name, has_warnings, has_components) in [
            ("no warnings", "no components", false, false),
            ("warnings", "no components", true, false),
            ("no warnings", "components", false, true),
            ("warnings", "components", true, true),
        ] {
            for is_first in [false, true] {
                let sx = SearchExperiencePresentation {
                    score,
                    label: "Label".to_string(),
                    interpretation: "The page is found. It could be clearer.".to_string(),
                    components: if has_components {
                        vec![
                            SearchExperienceComponent {
                                label: "Technical SEO".to_string(),
                                score: 82,
                                weight_pct: 40,
                                explanation: "Meta data is present.".to_string(),
                            },
                            SearchExperienceComponent {
                                label: "Content clarity".to_string(),
                                score: 0,
                                weight_pct: 60,
                                explanation: String::new(),
                            },
                        ]
                    } else {
                        vec![]
                    },
                    warnings: if has_warnings {
                        vec!["First warning.".to_string(), "Second warning.".to_string()]
                    } else {
                        vec![]
                    },
                };
                records.push(json!([
                    "render_search_experience",
                    format!("score={score} {warnings} {components_name} is_first={is_first}"),
                    lang,
                    components(render_search_experience(bare(), &sx, is_first, i18n))
                ]));
            }
        }
    }
}

fn budget_violation(metric: &str, error: bool, pct: f64) -> BudgetViolation {
    serde_json::from_value(json!({
        "metric": metric,
        "budget_label": "budget",
        "actual_label": "actual",
        "budget_value": 1.0,
        "actual_value": 2.0,
        "exceeded_by_pct": pct,
        "severity": if error { "error" } else { "warning" },
    }))
    .expect("a budget violation")
}

fn export_budget(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let metrics = [
        "JS size",
        "CSS size",
        "Page size",
        "Image size",
        "LCP",
        "TBT",
        "JS-Größe",
        "",
    ];
    for error in [true, false] {
        let violations: Vec<BudgetViolation> = metrics
            .iter()
            .map(|metric| budget_violation(metric, error, 12.5))
            .collect();
        records.push(json!([
            "render_budget_violations",
            format!("every metric, error={error}"),
            lang,
            components(render_budget_violations(bare(), &violations, i18n))
        ]));
    }
    let mixed = vec![
        budget_violation("JS size", true, 80.0),
        budget_violation("LCP", false, 20.0),
    ];
    records.push(json!([
        "render_budget_violations",
        "one error, one warning",
        lang,
        components(render_budget_violations(bare(), &mixed, i18n))
    ]));
    let only_warning = vec![budget_violation("LCP", false, 20.0)];
    records.push(json!([
        "render_budget_violations",
        "one warning",
        lang,
        components(render_budget_violations(bare(), &only_warning, i18n))
    ]));
}

fn export_helpers(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let en = lang == "en";
    for criterion in ["1.3.6", "1.3.1", "3.3.2", "4.1.3", "1.1.1", ""] {
        records.push(json!([
            "manual_recheck_instruction",
            criterion,
            lang,
            manual_recheck_instruction(criterion, en)
        ]));
    }
    for name in [
        "UX",
        "Accessibility",
        "accessibility",
        "HTML Conformance",
        "",
    ] {
        for measurement_type in ["measured", "heuristic", "composite", "not_measured", ""] {
            records.push(json!([
                "module_name_with_taxonomy_suffix",
                format!("{name:?} {measurement_type:?}"),
                lang,
                module_name_with_taxonomy_suffix(name, measurement_type, i18n)
            ]));
        }
    }
}

fn export_cover(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let en = lang == "en";
    for verdict in [Verdict::Pass, Verdict::Warn, Verdict::Fail] {
        for partial in [false, true] {
            for score in [0, 39, 40, 59, 60, 74, 75, 89, 90, 100] {
                records.push(json!([
                    "cover_band_phrase",
                    format!("{verdict:?} partial={partial} score={score}"),
                    lang,
                    cover_band_phrase(score, verdict, partial, en)
                ]));
                let grid =
                    build_batch_cover_score_row(score, "B", verdict, partial, 1_234_567, 1, i18n)
                        .expect("the row builds");
                records.push(json!([
                    "build_batch_cover_score_row",
                    format!("{verdict:?} partial={partial} score={score}"),
                    lang,
                    components(bare().add_component(grid))
                ]));
            }
        }
    }
}

fn export_single_report(records: &mut Vec<Value>, lang: &str) {
    for (name, report) in report_fixtures() {
        for level in [ReportLevel::Standard, ReportLevel::Technical] {
            for annex in [None, Some(AnnexKind::En301549), Some(AnnexKind::Bik)] {
                let config = ReportConfig {
                    level,
                    locale: lang.to_string(),
                    annex,
                    ..ReportConfig::default()
                };
                let (_engine, built) = build_single_report(&report, &config, None)
                    .expect("fixture report passes the report contract");
                records.push(json!([
                    "build_single_report",
                    format!("{name} level={level} annex={annex:?}"),
                    lang,
                    Value::Array(built.components)
                ]));
            }
        }
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_en301549(&mut records, lang, &i18n);
        export_bik(&mut records, lang, &i18n);
        export_search_experience(&mut records, lang, &i18n);
        export_budget(&mut records, lang, &i18n);
        export_helpers(&mut records, lang, &i18n);
        export_cover(&mut records, lang, &i18n);
        export_single_report(&mut records, lang);
    }
    records
}

#[test]
#[ignore = "writes reports/pdf-annex-texts.json; run before and after a change of the text storage"]
fn export_pdf_annex_texts() {
    let records = records();
    let path = std::env::var("PDF_ANNEX_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/pdf-annex-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} pdf annex text records written to {path}", records.len());
}
