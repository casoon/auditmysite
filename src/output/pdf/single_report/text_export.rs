//! Text export of the single PDF sections of `single_report.rs`, slice 8 of
//! #936: everything the module puts into the report, across an enumerated
//! input space in both languages, as records in
//! `reports/single-report-texts.json` (gitignored; the path can be set with
//! `SINGLE_REPORT_TEXT_EXPORT`). The render functions are driven with a bare
//! `ReportBuilder` and their components are recorded as JSON, so the export
//! covers the texts built inside them without a full report. Run it before
//! and after a change of how the texts are stored and compare the two files
//! byte for byte.

use super::*;
use crate::audit::baseline::BaselineGate;
use crate::audit::management_risk::ManagementRiskKind;
use crate::audit::normalized::{AccessibilityJourney, JourneyExecution};
use crate::audit::verdict::{Verdict, VerdictReasonKind};
use crate::audit::{
    BaselineStatus, DualViewportResults, PageScreenshots, ScreenshotStatus, ViewportAuditData,
};
use crate::model::WcagLevel;
use crate::wcag::{Severity, Violation, WcagResults};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// Counts: one, two and one that Fluent could group.
const COUNTS: [usize; 3] = [1, 2, 1_234_567];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

/// A page with violations across severities, criteria and levels, plus
/// manual-review and heuristic entries with and without an explanation.
fn report() -> AuditReport {
    let mut results = WcagResults::new();
    results.nodes_checked = 320;
    results.passes = 48;
    for (criterion, name, level, severity, rule_id) in [
        (
            "1.4.3",
            "Contrast (Minimum)",
            WcagLevel::AA,
            Severity::Critical,
            "contrast/text-insufficient",
        ),
        (
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::Critical,
            "images/alt-missing",
        ),
        (
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            Severity::High,
            "buttons/name-missing",
        ),
        (
            "2.4.4",
            "Link Purpose",
            WcagLevel::A,
            Severity::High,
            "links/generic-name",
        ),
        (
            "2.4.6",
            "Headings and Labels",
            WcagLevel::AA,
            Severity::Medium,
            "headings/empty",
        ),
        (
            "3.1.1",
            "Language of Page",
            WcagLevel::A,
            Severity::Low,
            "document/lang-missing",
        ),
    ] {
        results.add_violation(
            Violation::new(criterion, name, level, severity, "message", "node-1")
                .with_rule_id(rule_id)
                .with_selector("main > div.card a")
                .with_fix(format!("Fix {name}")),
        );
        results.add_violation(
            Violation::new(criterion, name, level, severity, "message", "node-2")
                .with_rule_id(rule_id)
                .with_selector("img"),
        );
    }
    for (criterion, rule_id) in [
        ("1.3.1", Some("contrast/text-insufficient")),
        ("1.3.6", None),
        ("9.9.9", Some("unknown/rule")),
        ("9.9.9", Some("unknown/rule")),
    ] {
        let mut v = Violation::new(
            criterion,
            "Name",
            WcagLevel::AA,
            Severity::Medium,
            "check",
            "node-3",
        );
        if let Some(id) = rule_id {
            v = v.with_rule_id(id);
        }
        results.add_not_testable(v.clone());
        results.add_warning(v);
    }
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        1_200,
    )
}

fn view_model(report: &AuditReport, level: ReportLevel, lang: &str) -> ReportViewModel {
    let normalized = crate::audit::normalized::normalize(report);
    let config = ReportConfig {
        level,
        locale: lang.to_string(),
        ..ReportConfig::default()
    };
    crate::output::builder::build_view_model(&normalized, &config)
}

fn module(name: &str, score: u32, measurement_type: &str) -> ModuleScore {
    ModuleScore {
        name: name.to_string(),
        score,
        measurement_type: measurement_type.to_string(),
        interpretation: String::new(),
        card_context: String::new(),
        score_context: String::new(),
        key_lever: String::new(),
        good_threshold: 75,
        warn_threshold: 50,
    }
}

fn export_measures(records: &mut Vec<Value>, lang: &str) {
    let en = lang == "en";
    records.push(json!(["audit_scope_line", "", lang, audit_scope_line(en)]));
    for title in ["", "Content-Security-Policy"] {
        let measure = TopMeasure {
            module: "Security".to_string(),
            title: title.to_string(),
            detail: "unsafe-inline".to_string(),
            from_findings: false,
        };
        for extra in [0, 1, 2, 1_234_567] {
            let input = format!("title={title:?} extra={extra}");
            records.push(json!([
                "TopMeasure::render",
                input,
                lang,
                measure.render(3, extra, en)
            ]));
            records.push(json!([
                "TopMeasure::card_parts",
                input,
                lang,
                measure.card_parts(extra, en)
            ]));
        }
    }
}

fn export_management(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let report = report();
    let base = view_model(&report, ReportLevel::Standard, lang);
    let clean = view_model(
        &AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        ),
        ReportLevel::Standard,
        lang,
    );

    for part in [1, 2, 3] {
        records.push(json!([
            "render_part_divider",
            format!("part={part}"),
            lang,
            components(render_part_divider(
                bare(),
                part,
                "Title",
                "Intro",
                "Audience",
                "Body",
                i18n
            ))
        ]));
    }

    records.push(json!([
        "build_severity_counter_strip",
        "",
        lang,
        build_severity_counter_strip(&base, i18n)
    ]));

    // Module split dashboards: none, strong only (pass and fail), both.
    let mut vm = view_model(&report, ReportLevel::Standard, lang);
    for (input, dashboard, verdict) in [
        ("no indicator modules", vec![], Verdict::Pass),
        (
            "measured only",
            vec![module("Accessibility", 40, "measured")],
            Verdict::Pass,
        ),
        (
            "strong only, pass",
            vec![module("UX", 90, "composite")],
            Verdict::Pass,
        ),
        (
            "strong only, warn",
            vec![module("UX", 90, "composite")],
            Verdict::Warn,
        ),
        (
            "strong only, fail",
            vec![module("UX", 90, "composite")],
            Verdict::Fail,
        ),
        (
            "strong and weak",
            vec![
                module("UX", 90, "heuristic"),
                module("Journey", 40, "composite"),
            ],
            Verdict::Fail,
        ),
    ] {
        vm.modules.dashboard = dashboard;
        vm.summary.ci_verdict = verdict;
        records.push(json!([
            "render_module_split_dashboards",
            input,
            lang,
            components(render_module_split_dashboards(bare(), &vm, i18n))
        ]));
    }

    // Overall rating: every verdict, with and without reasons, baseline and
    // a partial run.
    let mut vm = view_model(&report, ReportLevel::Standard, lang);
    for verdict in [Verdict::Pass, Verdict::Warn, Verdict::Fail] {
        for reasons in [
            vec![],
            vec![VerdictReasonKind::AuditQualityNotComplete],
            vec![
                VerdictReasonKind::LegalFlags(2),
                VerdictReasonKind::AuditQualityNotComplete,
            ],
        ] {
            for partial in [false, true] {
                for baseline in [false, true] {
                    vm.summary.ci_verdict = verdict;
                    vm.summary.ci_verdict_reasons = reasons.clone();
                    vm.summary.run_is_partial = partial;
                    vm.summary.baseline = baseline.then(|| BaselineStatus {
                        digest: "d".to_string(),
                        page_listed: true,
                        known: 1,
                        new: 2,
                        fixed: 3,
                        fixed_ids: vec![],
                        gate: BaselineGate::default(),
                    });
                    records.push(json!([
                        "render_overall_rating",
                        format!(
                            "verdict={verdict:?} reasons={} partial={partial} baseline={baseline}",
                            reasons.len()
                        ),
                        lang,
                        components(render_overall_rating(bare(), &vm, i18n))
                    ]));
                }
            }
        }
    }

    // Risks and strengths: none, all good, risks and strengths.
    let mut vm = view_model(&report, ReportLevel::Standard, lang);
    let good = vec![
        ManagementRiskKind::Seo { score: Some(95) },
        ManagementRiskKind::PerformanceMobile {
            performance: Some(95),
            mobile: Some(95),
            high_findings: 0,
        },
    ];
    let mut mixed = good.clone();
    mixed.push(ManagementRiskKind::Legal {
        legal_flags: 2,
        bfsg_relevant: 2,
        critical: 2,
        high: 1,
        blocking_issues: 0,
        keyboard_blocking: 0,
    });
    mixed.push(ManagementRiskKind::Seo { score: Some(50) });
    let mut risks_only = vec![ManagementRiskKind::Seo { score: Some(50) }];
    for (input, kinds) in [
        ("none", vec![]),
        ("all good", good),
        ("mixed", mixed),
        ("one risk", std::mem::take(&mut risks_only)),
    ] {
        vm.management_risks = kinds;
        records.push(json!([
            "render_risks_and_strengths",
            input,
            lang,
            components(render_risks_and_strengths(bare(), &vm, i18n))
        ]));
    }

    for (input, vm) in [("findings", &base), ("clean", &clean)] {
        records.push(json!([
            "build_top_measures_list",
            input,
            lang,
            build_top_measures_list(vm, i18n)
        ]));
    }

    // The whole management page: evidence counts, audit quality notes,
    // clean run and the radar.
    let mut vm = view_model(&report, ReportLevel::Standard, lang);
    vm.modules.dashboard = vec![
        module("Accessibility", 60, "measured"),
        module("Performance & Speed", 80, "measured"),
        module("SEO", 70, "measured"),
    ];
    for (warnings, manual) in [(0, 0), (1, 0), (0, 1), (2, 2), (1_234_567, 1)] {
        for (note, severe) in [(None, false), (Some("note"), false), (Some("note"), true)] {
            vm.findings.evidence.warnings = warnings;
            vm.findings.evidence.manual_checks = manual;
            vm.summary.audit_quality_note = note.map(String::from);
            vm.summary.audit_quality_severe = severe;
            records.push(json!([
                "render_management_page",
                format!("warnings={warnings} manual={manual} note={note:?} severe={severe}"),
                lang,
                components(render_management_page(bare(), &vm, i18n))
            ]));
        }
    }
    records.push(json!([
        "render_management_page",
        "clean run",
        lang,
        components(render_management_page(bare(), &clean, i18n))
    ]));
}

fn export_findings(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let report = report();
    let base = view_model(&report, ReportLevel::Standard, lang);
    let mut render = |input: &str, vm: &ReportViewModel| {
        let (mut seq, mut expanded) = (0, false);
        records.push(json!([
            "render_findings_section",
            input,
            lang,
            components(render_findings_section(
                bare(),
                vm,
                i18n,
                0,
                &mut seq,
                &mut expanded
            ))
        ]));
    };
    render("as built", &base);

    // Every bucket, with and without criterion and level.
    let mut vm = view_model(&report, ReportLevel::Standard, lang);
    for (i, group) in vm.findings.all_findings.iter_mut().enumerate() {
        group.is_component_issue = i % 2 == 0;
        group.criticality_tier = if i % 4 < 2 {
            CriticalityTier::Mandatory
        } else {
            CriticalityTier::Optimization
        };
        match i % 3 {
            0 => group.finding.wcag_criterion = String::new(),
            1 => group.finding.wcag_level = String::new(),
            _ => {}
        }
    }
    render("all buckets", &vm);

    // More rows than the matrix shows.
    for extra in [1, 2] {
        let mut many = vm.findings.all_findings.clone();
        while many.len() < 30 + extra {
            let next = many[many.len() % vm.findings.all_findings.len()].clone();
            many.push(next);
        }
        many.truncate(30 + extra);
        let mut vm_many = view_model(&report, ReportLevel::Standard, lang);
        vm_many.findings.all_findings = many;
        render(&format!("{} rows", 30 + extra), &vm_many);
    }

    let mut clean = view_model(&report, ReportLevel::Standard, lang);
    clean.severity.has_issues = false;
    render("no issues", &clean);
}

fn journey(budget_exhausted: bool, count: usize) -> AccessibilityJourney {
    AccessibilityJourney {
        execution: JourneyExecution {
            mode: "standard".to_string(),
            budget_ms: 1_000,
            candidates_detected: count,
            attempted: count,
            completed: count,
            failed: count,
            skipped: count,
            budget_exhausted,
            runs: vec![],
        },
        ..AccessibilityJourney::default()
    }
}

fn export_appendix(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for level in [
        ReportLevel::Executive,
        ReportLevel::Standard,
        ReportLevel::Technical,
    ] {
        for journey_state in [None, Some(false), Some(true)] {
            let mut report = report();
            report.accessibility_journey = journey_state.map(|b| journey(b, 2));
            let normalized = crate::audit::normalized::normalize(&report);
            let vm = view_model(&report, level, lang);
            let config = ReportConfig {
                level,
                locale: lang.to_string(),
                ..ReportConfig::default()
            };
            records.push(json!([
                "render_appendix_full",
                format!("level={level:?} journey_budget_exhausted={journey_state:?}"),
                lang,
                components(render_appendix_full(
                    bare(),
                    &vm,
                    &report,
                    &normalized.normalized.findings,
                    &config,
                    i18n
                ))
            ]));
        }
    }
    for count in COUNTS {
        let mut report = report();
        report.accessibility_journey = Some(journey(false, count));
        records.push(json!([
            "render_assessment_and_execution_notes",
            format!("journey count={count}"),
            lang,
            components(render_assessment_and_execution_notes(bare(), &report, i18n))
        ]));
    }
    let mut report = report();
    report.accessibility.wcag_results.warnings.clear();
    report.accessibility.wcag_results.not_testables.clear();
    records.push(json!([
        "render_assessment_and_execution_notes",
        "nothing to note",
        lang,
        components(render_assessment_and_execution_notes(bare(), &report, i18n))
    ]));

    // The appendix list without violations.
    let mut vm = view_model(&report, ReportLevel::Technical, lang);
    vm.appendix.has_violations = false;
    records.push(json!([
        "render_appendix_section",
        "no violations",
        lang,
        components(render_appendix_section(bare(), &vm, i18n))
    ]));
}

fn export_signals(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let signals = [
        PositiveSignal {
            title: "Landmarks".to_string(),
            description: "main, nav".to_string(),
            strong: true,
        },
        PositiveSignal {
            title: "Skip link".to_string(),
            description: "present".to_string(),
            strong: false,
        },
    ];
    for (input, signals) in [
        ("none", &signals[..0]),
        ("strong and present", &signals[..]),
    ] {
        records.push(json!([
            "render_positive_signals_section",
            input,
            lang,
            components(render_positive_signals_section(bare(), signals, i18n))
        ]));
    }
}

fn viewport(results: WcagResults) -> ViewportAuditData {
    ViewportAuditData {
        wcag_results: results,
        accessibility_score: 80.0,
        performance: None,
        seo: None,
        mobile: None,
        ux: None,
        journey: None,
        screenshot: None,
        module_runs: vec![],
    }
}

fn export_dual_viewport(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for scores in [false, true] {
        for status in [
            ScreenshotStatus::NotRequested,
            ScreenshotStatus::Captured,
            ScreenshotStatus::Failed("timeout {x}".to_string()),
        ] {
            for screenshots in [false, true] {
                for dual in [false, true] {
                    for is_first in [true, false] {
                        let mut report = report();
                        report.screenshot_status = status.clone();
                        report.page_screenshots = screenshots.then(|| PageScreenshots {
                            desktop: vec![],
                            mobile: vec![],
                        });
                        report.dual_viewport = dual.then(|| DualViewportResults {
                            desktop: viewport(report.accessibility.wcag_results.clone()),
                            mobile: viewport(WcagResults::new()),
                        });
                        let mut vm = view_model(&report, ReportLevel::Standard, lang);
                        vm.cover.desktop_score = scores.then_some(95);
                        vm.cover.mobile_score = scores.then_some(38);
                        records.push(json!([
                            "render_dual_viewport_summary_section",
                            format!(
                                "scores={scores} status={status:?} screenshots={screenshots} dual={dual} is_first={is_first}"
                            ),
                            lang,
                            components(render_dual_viewport_summary_section(
                                bare(),
                                &vm,
                                &report,
                                is_first,
                                i18n
                            ))
                        ]));
                    }
                }
            }
        }
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_measures(&mut records, lang);
        export_management(&mut records, lang, &i18n);
        export_findings(&mut records, lang, &i18n);
        export_appendix(&mut records, lang, &i18n);
        export_signals(&mut records, lang, &i18n);
        export_dual_viewport(&mut records, lang, &i18n);
    }
    records
}

#[test]
#[ignore = "writes reports/single-report-texts.json; run before and after a change of the text storage"]
fn export_single_report_texts() {
    let records = records();
    let path = std::env::var("SINGLE_REPORT_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/single-report-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} single report text records written to {path}",
        records.len()
    );
}
