//! Text export of the accessibility detail sections of `accessibility.rs`,
//! slice 13 of #936: everything the module puts into the report, across an
//! enumerated input space in both languages, as records in
//! `reports/accessibility-details-texts.json` (gitignored; the path can be set
//! with `ACCESSIBILITY_DETAILS_TEXT_EXPORT`). The render functions are driven
//! with a bare `ReportBuilder` and their components are recorded as JSON, so
//! the export covers the texts built inside them without a full report. Run
//! it before and after a change of how the texts are stored and compare the
//! two files byte for byte.

use super::*;
use crate::audit::normalized::{
    AccessibilityJourney, FindingUncertainty, InteractiveFinding, InteractiveFindingKind,
    InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::screen_reader::{
    build_sr_audit_report, AnnouncedReadingItem, BfsgVerdict, ReadingItem, SrAuditReport,
};
use crate::taxonomy::Severity;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

const VERDICTS: [BfsgVerdict; 5] = [
    BfsgVerdict::NotEvaluated,
    BfsgVerdict::ViolationsFound,
    BfsgVerdict::NeedsReview,
    BfsgVerdict::Incomplete,
    BfsgVerdict::NoViolationsFound,
];

/// Every journey id the label table knows, with and without an index, plus
/// one it does not know.
const JOURNEYS: [&str; 13] = [
    "tab_walk",
    "spa_navigation",
    "skip_link",
    "disclosure_0",
    "menu",
    "modal_2",
    "tabs",
    "form_error",
    "add_to_cart",
    "quantity_stepper",
    "quantity_stepper_11",
    "something_new_7",
    "custom",
];

fn components(builder: renderreport::engine::ReportBuilder) -> Value {
    Value::Array(builder.build().components)
}

fn bare() -> renderreport::engine::ReportBuilder {
    renderreport::engine::ReportBuilder::new("export")
}

/// A journey finding; `journey` keeps findings apart (or identical ones
/// together) in the deduplication.
fn finding(
    severity: Severity,
    kind: InteractiveFindingKind,
    journey: &str,
    uncertainty: Option<FindingUncertainty>,
) -> InteractiveFinding {
    let mut finding = InteractiveFinding::new(
        "FocusTrap",
        kind,
        None,
        severity,
        journey.to_string(),
        None,
        None,
        InteractiveFindingValues::default(),
    );
    finding.uncertainty = uncertainty;
    finding
}

fn distinct(severity: Severity, n: usize, offset: usize) -> Vec<InteractiveFinding> {
    (0..n)
        .map(|i| {
            finding(
                severity,
                InteractiveFindingKind::FocusTrapEscaped,
                &format!("modal_{}", offset + i),
                None,
            )
        })
        .collect()
}

fn step() -> JourneyStep {
    JourneyStep {
        action: "tab".to_string(),
        target: None,
        focus: None,
        result: None,
        snapshot_label: None,
    }
}

fn journey(steps: usize) -> AccessibilityJourney {
    AccessibilityJourney {
        traces: JOURNEYS
            .iter()
            .map(|journey| JourneyTrace {
                journey: journey.to_string(),
                steps: (0..steps).map(|_| step()).collect(),
            })
            .collect(),
        ..AccessibilityJourney::default()
    }
}

fn export_journey_findings(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    let mut sets: Vec<(String, Vec<InteractiveFinding>)> = Vec::new();
    for critical in [1, 2] {
        for high in [0, 1, 2] {
            let mut findings = distinct(Severity::Critical, critical, 0);
            findings.extend(distinct(Severity::High, high, 100));
            sets.push((format!("critical={critical} high={high}"), findings));
        }
    }
    for high in [1, 2, 11] {
        let mut findings = distinct(Severity::High, high, 0);
        findings.extend(distinct(Severity::Medium, 1, 100));
        sets.push((format!("high={high} medium=1"), findings));
    }
    for minor in [1, 2, 10, 11, 12, 1_234] {
        sets.push((format!("minor={minor}"), distinct(Severity::Low, minor, 0)));
    }
    // Identical findings collapse into one row with a count; a fix
    // suggestion, an uncertainty note and every severity.
    let mut mixed = Vec::new();
    for _ in 0..3 {
        mixed.push(finding(
            Severity::Medium,
            InteractiveFindingKind::FocusTrapEscaped,
            "modal_0",
            None,
        ));
    }
    mixed.push(finding(
        Severity::Low,
        InteractiveFindingKind::FocusIndicatorNotDetected,
        "tab_walk",
        Some(FindingUncertainty::LateReactionPossible { waited_ms: 1_500 }),
    ));
    mixed.push(finding(
        Severity::High,
        InteractiveFindingKind::TabOrderBackwardJumps,
        "tab_walk",
        None,
    ));
    sets.push(("duplicates, uncertainty, kinds".to_string(), mixed));

    for (input, findings) in &sets {
        for (journey_input, journey) in [
            ("none", None),
            ("no traces", Some(AccessibilityJourney::default())),
            ("1 step", Some(journey(1))),
            ("2 steps", Some(journey(2))),
        ] {
            records.push(json!([
                "render_a11y_journey_findings",
                format!("{input} journey={journey_input}"),
                lang,
                components(render_a11y_journey_findings(
                    bare(),
                    findings,
                    journey.as_ref(),
                    i18n
                ))
            ]));
        }
    }
}

fn item(
    seq: usize,
    role: &str,
    name: Option<&str>,
    tab_stop: bool,
    states: &[&str],
) -> ReadingItem {
    ReadingItem {
        seq,
        role: Some(role.to_string()),
        name: name.map(String::from),
        description: None,
        value: None,
        states: states.iter().map(|s| s.to_string()).collect(),
        tab_stop,
        depth: 0,
        node_id: format!("{}", seq + 1),
    }
}

/// A screen-reader report whose reading sequence yields issues of every
/// severity, two identical ones among them, and criteria with and without a
/// manual recheck instruction.
fn screen_reader(with_issues: bool) -> SrAuditReport {
    let mut sr = build_sr_audit_report(
        "https://example.com/",
        chrono::Utc::now(),
        &crate::accessibility::AXTree::new(),
        "en",
        None,
    );
    sr.summary.heading_quality_score = 35;
    sr.summary.landmark_quality_score = 72;
    sr.summary.name_quality_score = 100;
    sr.summary.total_announced_nodes = 1_234;
    sr.summary.tab_stops = 87;
    if with_issues {
        let items = vec![
            item(0, "heading", Some("Title"), false, &["level=1"]),
            item(1, "heading", Some("Deep"), false, &["level=4"]),
            item(2, "link", Some("here"), true, &[]),
            item(3, "link", Some("here"), true, &[]),
            item(4, "link", Some("more"), true, &[]),
            item(5, "button", None, true, &[]),
            item(6, "textbox", Some(""), true, &[]),
        ];
        sr.reading_sequence = items
            .into_iter()
            .map(|item| AnnouncedReadingItem {
                item,
                announcement: String::new(),
            })
            .collect();
        sr.navigation_views = crate::screen_reader::navigation_views(
            &sr.reading_sequence
                .iter()
                .map(|a| a.item.clone())
                .collect::<Vec<_>>(),
        );
    }
    sr
}

fn export_screen_reader(records: &mut Vec<Value>, lang: &str, i18n: &I18n) {
    for with_issues in [false, true] {
        for verdict in VERDICTS {
            for wcag_findings in [0, 1] {
                let mut sr = screen_reader(with_issues);
                sr.bfsg_compliance.verdict = verdict;
                records.push(json!([
                    "render_screen_reader_section",
                    format!(
                        "issues={with_issues} verdict={verdict:?} wcag_findings={wcag_findings}"
                    ),
                    lang,
                    components(render_screen_reader_section(
                        bare(),
                        &sr,
                        None,
                        wcag_findings,
                        i18n
                    ))
                ]));
            }
        }
    }
}

fn export_bfsg_callout(records: &mut Vec<Value>, lang: &str) {
    for verdict in VERDICTS {
        records.push(json!([
            "bfsg_callout",
            format!("{verdict:?}"),
            lang,
            components(bare().add_component(bfsg_callout(verdict, lang == "en")))
        ]));
    }
}

fn export_journey_label(records: &mut Vec<Value>, lang: &str) {
    for journey in JOURNEYS {
        records.push(json!([
            "journey_label",
            journey,
            lang,
            journey_label(journey, lang == "en")
        ]));
    }
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        export_journey_findings(&mut records, lang, &i18n);
        export_screen_reader(&mut records, lang, &i18n);
        export_bfsg_callout(&mut records, lang);
        export_journey_label(&mut records, lang);
    }
    records
}

#[test]
#[ignore = "writes reports/accessibility-details-texts.json; run before and after a change of the text storage"]
fn export_accessibility_details_texts() {
    let records = records();
    let path = std::env::var("ACCESSIBILITY_DETAILS_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/accessibility-details-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} accessibility detail text records written to {path}",
        records.len()
    );
}
