//! Guards of `locales/{de,en}/single-report.ftl` (#936): the same keys in
//! both languages, every key defined once, used by `single_report.rs` and
//! resolving without errors, no German in the English file, and no text of
//! the export scenarios that is a key or still holds a placeholder.

use super::text_export::records;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use serde_json::Value;
use std::collections::BTreeSet;

const EN: &str = include_str!("../../../../locales/en/single-report.ftl");
const DE: &str = include_str!("../../../../locales/de/single-report.ftl");
const SOURCE: &str = include_str!("../single_report.rs");

/// The `report.ftl` messages `single_report.rs` reuses.
const REUSED: [&str; 14] = [
    "batch-rating-title",
    "batch-rating-verdict-",
    "batch-rating-reasons",
    "batch-col-priority",
    "batch-col-occurrences",
    "label-effort",
    "module-accessibility",
    "module-security",
    "severity-critical",
    "severity-high",
    "severity-medium",
    "count-heuristic-warnings",
    "count-manual-points",
    "count-more-findings",
];

/// The message ids of a file (column-0 `id =` lines), each only once.
fn ids(ftl: &str) -> Vec<String> {
    let ids: Vec<String> = ftl
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with([' ', '#']))
        .filter_map(|line| line.split_once(" =").map(|(id, _)| id.to_string()))
        .collect();
    let unique: BTreeSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len(), "a message is defined twice");
    ids
}

#[test]
fn de_and_en_define_the_same_messages() {
    let de: BTreeSet<_> = ids(DE).into_iter().collect();
    let en: BTreeSet<_> = ids(EN).into_iter().collect();
    assert_eq!(de, en);
    assert_eq!(en.len(), 120);
}

#[test]
fn every_message_is_used_by_the_code() {
    for id in ids(EN) {
        assert!(
            SOURCE.contains(&format!("\"{id}\"")),
            "{id} is not used in single_report.rs"
        );
    }
}

#[test]
fn every_key_resolves_in_both_languages_without_errors() {
    for (locale, ftl) in [("en", EN), ("de", DE)] {
        let resource = FluentResource::try_new(ftl.to_string()).expect("parses");
        let mut bundle = FluentBundle::new(vec![locale.parse().unwrap()]);
        bundle.set_use_isolating(false);
        bundle.add_resource(resource).expect("no duplicates");
        let mut args = FluentArgs::new();
        for name in [
            "part",
            "index",
            "count",
            "warnings",
            "manual",
            "manual_only",
            "automated",
            "supported",
            "total",
            "completed",
            "attempted",
            "failed",
            "skipped",
            "desktop",
            "mobile",
            "combined",
            "severe",
            "score",
            "checked",
        ] {
            args.set(name, FluentValue::from(2_i64));
        }
        for name in [
            "rating",
            "verdict",
            "warnings_text",
            "manual_text",
            "more",
            "budget",
            "reason",
            "occurrence_context",
            "cap_note",
            "weighted",
            "list",
            "id",
            "title",
            "reach",
            "priority",
            "subject",
        ] {
            args.set(name, FluentValue::from("x"));
        }
        for id in ids(EN) {
            let message = bundle.get_message(&id).expect("defined");
            let pattern = message.value().expect("has a value");
            let mut errors = vec![];
            let text = bundle.format_pattern(pattern, Some(&args), &mut errors);
            assert!(errors.is_empty(), "{locale} {id}: {errors:?}");
            assert!(!text.is_empty(), "{locale} {id} is empty");
        }
    }
    for id in ids(EN) {
        for en in [true, false] {
            assert_ne!(crate::i18n::text(&id, en), id, "{id} falls back to its key");
        }
    }
}

#[test]
fn english_messages_have_no_german_characters() {
    assert!(!EN.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']));
}

/// Every string in a record, component JSON included.
fn strings<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::String(text) => out.push(text),
        Value::Array(items) => items.iter().for_each(|item| strings(item, out)),
        Value::Object(map) => map.values().for_each(|item| strings(item, out)),
        _ => {}
    }
}

#[test]
fn no_scenario_leaves_a_key_or_a_placeholder_in_a_text() {
    for record in records() {
        let mut texts = Vec::new();
        strings(&record, &mut texts);
        for text in texts {
            assert!(
                !text.contains("single-report-")
                    && !REUSED.iter().any(|key| text.contains(key))
                    && !text.contains("{$")
                    && !text.contains("{ $"),
                "{record}: {text}"
            );
        }
    }
}

/// The two criteria counts of the report, named once and used alike (#1256):
/// "supported" is the A/AA criteria the tool can fail
/// (`coverage_stats().0`), "executed" those of them evaluated in this run.
/// The summary status block and the appendix scope text must state the same
/// two numbers. The expectation comes from the definition: a page whose only
/// outcome is one violation of 1.1.1 has evaluated exactly that one criterion.
#[test]
fn summary_and_appendix_state_the_same_supported_and_executed_counts() {
    use crate::i18n::I18n;
    use crate::model::{ReportLevel, WcagLevel};
    use crate::output::report_model::ReportConfig;
    use crate::wcag::{Severity, Violation, WcagResults};

    let (supported, total) = crate::wcag::coverage::coverage_stats();
    assert!(
        supported < total,
        "the tool does not support every criterion"
    );

    let mut results = WcagResults::new();
    results.nodes_checked = 10;
    results.add_violation(
        Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Image without alternative text.",
            "node-1",
        )
        .with_rule_id("images/alt-missing"),
    );
    let report = crate::audit::AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        100,
    );
    let executed = 1;
    assert!(executed <= supported);

    let normalized = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    for (lang, status, scope) in [
        (
            "en",
            format!("{executed} of {supported} supported A/AA criteria evaluated in this run"),
            format!("{executed} of them were evaluated in this run"),
        ),
        (
            "de",
            format!(
                "{executed} von {supported} unterstützten A/AA-Kriterien in diesem Lauf ausgeführt"
            ),
            format!("{executed} davon wurden in diesem Lauf ausgeführt"),
        ),
    ] {
        let config = ReportConfig {
            level: ReportLevel::Standard,
            locale: lang.to_string(),
            ..ReportConfig::default()
        };
        let vm = crate::output::builder::build_view_model(&normalized, &config);
        assert_eq!(vm.findings.evidence.criteria_checked, executed as u32);
        let i18n = I18n::new(lang).unwrap();
        let summary = serde_json::to_string(
            &super::render_check_status(
                renderreport::engine::ReportBuilder::new("guard"),
                &vm,
                &i18n,
            )
            .build()
            .components,
        )
        .unwrap();
        let appendix = serde_json::to_string(
            &super::super::wcag_coverage::render_wcag_coverage_section(
                renderreport::engine::ReportBuilder::new("guard"),
                &report,
                &i18n,
            )
            .build()
            .components,
        )
        .unwrap();
        assert!(summary.contains(&status), "{lang} summary: {summary}");
        assert!(
            appendix.contains(&format!("{supported} of ~{total}"))
                || appendix.contains(&format!("{supported} von ca. {total}")),
            "{lang} appendix supported count: {appendix}"
        );
        assert!(appendix.contains(&scope), "{lang} appendix: {appendix}");
    }
}
