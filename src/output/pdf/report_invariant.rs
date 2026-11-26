//! Report invariant of the three report levels (#1171): decision, analysis,
//! evidence. A finding is told once in full outside the evidence level.
//!
//! The test renders a report, walks its components in order and records, per
//! rule id, the sections that carry the rule's problem, impact or
//! recommendation text. Outside the evidence level at most one section may.
//!
//! The evidence level of the single report is its appendix; the developer
//! part (part 2) holds the evidence entries, which carry the rule, the
//! criterion, the occurrences and the measurement, but not the texts of the
//! analysis card (#1209). A finding id is therefore rendered in full, with its
//! problem, impact or fix, in at most one component outside the appendix.

use super::*;
use crate::audit::{AuditReport, BatchReport};
use crate::wcag::{Severity, Violation, WcagResults};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A needle is the start of a text, so a text a table cuts short still counts.
const NEEDLE_CHARS: usize = 60;
/// Shorter texts are labels, not prose.
const MIN_TEXT_CHARS: usize = 25;

fn squash_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn needle(text: &str) -> Option<String> {
    let text = squash_whitespace(text);
    (text.chars().count() >= MIN_TEXT_CHARS).then(|| text.chars().take(NEEDLE_CHARS).collect())
}

fn strings(value: &Value, out: &mut String) {
    match value {
        Value::String(s) => {
            out.push(' ');
            out.push_str(s);
        }
        Value::Array(items) => items.iter().for_each(|v| strings(v, out)),
        Value::Object(map) => map.values().for_each(|v| strings(v, out)),
        _ => {}
    }
}

/// One section of the report: its heading, whether it belongs to the evidence
/// level, and all its text.
struct Section {
    title: String,
    evidence: bool,
    text: String,
    /// The text of each component of the section, in order: a card is one.
    components: Vec<String>,
}

/// Split the components into sections at every heading of level 1 or 2.
/// The evidence level starts at the heading titled `evidence_start`.
fn sections(components: &[Value], evidence_start: &[String]) -> Vec<Section> {
    let mut sections = vec![Section {
        title: "(before the first heading)".to_string(),
        evidence: false,
        text: String::new(),
        components: Vec::new(),
    }];
    let mut evidence = false;
    for component in components {
        let kind = component["type"].as_str().unwrap_or_default();
        let data = &component["data"];
        let level = data["level"].as_u64().unwrap_or(0);
        if matches!(kind, "section" | "section-header-split") && (1..=2).contains(&level) {
            let title = data["title"].as_str().unwrap_or_default().to_string();
            if evidence_start.contains(&title) {
                evidence = true;
            }
            sections.push(Section {
                title,
                evidence,
                text: String::new(),
                components: Vec::new(),
            });
        }
        let current = sections.last_mut().expect("a first section exists");
        let mut text = String::new();
        strings(component, &mut text);
        current.components.push(squash_whitespace(&text));
        current.text.push(' ');
        current.text.push_str(&text);
    }
    for section in &mut sections {
        section.text = squash_whitespace(&section.text);
    }
    sections
}

/// Per rule id, the sections that carry one of its texts.
fn carriers<'a>(
    sections: &'a [Section],
    rules: &[(String, Vec<String>)],
) -> BTreeMap<String, Vec<&'a Section>> {
    rules
        .iter()
        .map(|(rule, texts)| {
            let needles: Vec<String> = texts.iter().filter_map(|t| needle(t)).collect();
            let found = sections
                .iter()
                .filter(|s| needles.iter().any(|n| s.text.contains(n.as_str())))
                .collect();
            (rule.clone(), found)
        })
        .collect()
}

fn rule_texts(groups: &[FindingGroup]) -> Vec<(String, Vec<String>)> {
    groups
        .iter()
        .map(|g| {
            (
                g.finding.rule_id.clone(),
                vec![
                    g.customer_description.clone(),
                    g.user_impact.clone(),
                    g.recommendation.clone(),
                ],
            )
        })
        .collect()
}

/// Fails with the section list of every rule that breaks the invariant.
fn assert_told_once(
    label: &str,
    carriers: &BTreeMap<String, Vec<&Section>>,
    require_carrier: bool,
) {
    let mut broken = Vec::new();
    for (rule, found) in carriers {
        // `cargo test report_invariant -- --nocapture` lists them per rule.
        println!(
            "{label}: {rule} -> {:?}",
            found
                .iter()
                .map(|s| format!("{}{}", s.title, if s.evidence { " [evidence]" } else { "" }))
                .collect::<Vec<_>>()
        );
        let outside: BTreeSet<&str> = found
            .iter()
            .filter(|s| !s.evidence)
            .map(|s| s.title.as_str())
            .collect();
        if outside.len() > 1 {
            broken.push(format!(
                "{rule}: told outside the evidence level in {outside:?}"
            ));
        }
        if require_carrier && found.is_empty() {
            broken.push(format!("{rule}: no section carries its text at all"));
        }
    }
    assert!(broken.is_empty(), "{label}:\n{}", broken.join("\n"));
}

/// A finding id stands in full in exactly one analysis component: its card.
/// Other components may name the id, as the evidence entry does as a
/// reference, but may not carry the texts the card carries.
fn assert_one_card_per_finding_id(
    label: &str,
    sections: &[Section],
    findings: &[(String, Vec<String>)],
) {
    let mut broken = Vec::new();
    for (id, texts) in findings {
        let needles: Vec<String> = texts.iter().filter_map(|t| needle(t)).collect();
        let cards: Vec<&str> = sections
            .iter()
            .filter(|s| !s.evidence)
            .flat_map(|s| s.components.iter().map(move |c| (s, c)))
            .filter(|(_, c)| c.contains(id.as_str()))
            .filter(|(_, c)| needles.iter().any(|n| c.contains(n.as_str())))
            .map(|(s, _)| s.title.as_str())
            .collect();
        if cards.len() != 1 {
            broken.push(format!(
                "{id}: rendered in full in {} analysis components: {cards:?}",
                cards.len()
            ));
        }
    }
    assert!(broken.is_empty(), "{label}:\n{}", broken.join("\n"));
}

fn finding_texts(groups: &[FindingGroup]) -> Vec<(String, Vec<String>)> {
    groups
        .iter()
        .map(|g| {
            (
                g.finding.id.clone(),
                vec![
                    g.customer_description.clone(),
                    g.user_impact.clone(),
                    g.recommendation.clone(),
                ],
            )
        })
        .collect()
}

/// Each text of a rule stands in at most one component outside the evidence
/// level: a card carries it, a second card or a list entry may not repeat it.
fn assert_each_text_in_one_component(
    label: &str,
    sections: &[Section],
    rules: &[(String, Vec<String>)],
) {
    let mut broken = Vec::new();
    for (rule, texts) in rules {
        for needle in texts.iter().filter_map(|t| needle(t)) {
            let holders: Vec<&str> = sections
                .iter()
                .filter(|s| !s.evidence)
                .flat_map(|s| s.components.iter().map(move |c| (s, c)))
                .filter(|(_, c)| c.contains(needle.as_str()))
                .map(|(s, _)| s.title.as_str())
                .collect();
            if holders.len() > 1 {
                broken.push(format!(
                    "{rule}: \"{needle}\" is in {} components: {holders:?}",
                    holders.len()
                ));
            }
        }
    }
    assert!(broken.is_empty(), "{label}:\n{}", broken.join("\n"));
}

fn page(url: &str, pages: usize) -> AuditReport {
    let mut results = WcagResults::new();
    results.nodes_checked = 200;
    for (criterion, name, level, severity, rule_id) in [
        (
            "1.4.3",
            "Contrast (Minimum)",
            crate::model::WcagLevel::AA,
            Severity::High,
            "contrast/text-insufficient",
        ),
        (
            "1.1.1",
            "Non-text Content",
            crate::model::WcagLevel::A,
            Severity::Critical,
            "images/alt-missing",
        ),
        (
            "4.1.2",
            "Name, Role, Value",
            crate::model::WcagLevel::A,
            Severity::High,
            "buttons/name-missing",
        ),
        (
            "2.4.4",
            "Link Purpose",
            crate::model::WcagLevel::A,
            Severity::Medium,
            "links/generic-name",
        ),
        (
            "3.1.1",
            "Language of Page",
            crate::model::WcagLevel::A,
            Severity::Low,
            "document/lang-missing",
        ),
    ] {
        for i in 0..pages {
            results.add_violation(
                Violation::new(criterion, name, level, severity, "message", format!("n{i}"))
                    .with_rule_id(rule_id)
                    .with_selector(format!("main > .item-{i} a"))
                    .with_fix(format!("Fix {name}")),
            );
        }
    }
    AuditReport::new(url.to_string(), crate::model::WcagLevel::AA, results, 1_000)
}

fn config(locale: &str, level: ReportLevel) -> ReportConfig {
    ReportConfig {
        level,
        locale: locale.to_string(),
        ..ReportConfig::default()
    }
}

#[test]
fn single_report_tells_each_finding_once_outside_the_evidence_level() {
    for locale in ["en", "de"] {
        for level in [ReportLevel::Standard, ReportLevel::Technical] {
            let report = page("https://example.com", 3);
            let config = config(locale, level);
            let normalized = crate::audit::validation::normalize_checked(&report).unwrap();
            let vm = build_view_model(&normalized, &config);
            let i18n = I18n::new(locale).unwrap();
            let (_, request) = build_single_report(&report, &config, None).unwrap();
            let sections = sections(
                &request.components,
                &[i18n.t("single-report-appendix-title")],
            );
            let rules = rule_texts(&vm.findings.all_findings);
            assert!(rules.len() >= 5, "{locale}: fixture lost its rules");
            let carriers = carriers(&sections, &rules);
            assert_told_once(&format!("single {locale} {level:?}"), &carriers, true);
            assert_each_text_in_one_component(
                &format!("single {locale} {level:?}"),
                &sections,
                &rules,
            );
            assert_one_card_per_finding_id(
                &format!("single {locale} {level:?}"),
                &sections,
                &finding_texts(&vm.findings.all_findings),
            );
        }
    }
}

#[test]
fn batch_report_tells_each_finding_once_outside_the_evidence_level() {
    for locale in ["en", "de"] {
        for level in [ReportLevel::Standard, ReportLevel::Technical] {
            let batch = BatchReport::from_reports(
                ["a", "b", "c"]
                    .iter()
                    .map(|p| page(&format!("https://example.com/{p}"), 2))
                    .collect(),
                vec![],
                100,
            );
            let config = config(locale, level);
            let i18n = I18n::new(locale).unwrap();
            let pres = crate::output::builder::build_batch_presentation_with_locale(&batch, &i18n);
            let (_, request) =
                super::batch_report::build_batch_report(&batch, &config, None).unwrap();
            let sections = sections(&request.components, &[i18n.t("section-appendix")]);
            let rules = rule_texts(&pres.top_issues);
            assert!(rules.len() >= 5, "{locale}: fixture lost its rules");
            let carriers = carriers(&sections, &rules);
            assert_told_once(&format!("batch {locale} {level:?}"), &carriers, true);
            assert_each_text_in_one_component(
                &format!("batch {locale} {level:?}"),
                &sections,
                &rules,
            );
        }
    }
}

/// #1211: the management page states the result and the first step in one
/// sentence, from the measured counts of its own strip and the first key
/// measure, in both languages and without German characters in English.
#[test]
fn management_page_names_result_and_first_step() {
    for locale in ["en", "de"] {
        let report = page("https://example.com", 3);
        let config = config(locale, ReportLevel::Standard);
        let normalized = crate::audit::validation::normalize_checked(&report).unwrap();
        let vm = build_view_model(&normalized, &config);
        let i18n = I18n::new(locale).unwrap();
        let builder = renderreport::engine::ReportBuilder::new("management");
        let built = super::single_report::render_management_page(builder, &vm, &i18n).build();
        let text = squash_whitespace(&serde_json::to_string(&built.components).unwrap());
        let (title, step, total) = if locale == "en" {
            (
                "Result and first step",
                "First step: ",
                format!(
                    "{} confirmed WCAG occurrences on this page",
                    vm.severity.total
                ),
            )
        } else {
            (
                "Ergebnis und erster Schritt",
                "Erster Schritt: ",
                format!(
                    "{} bestätigte WCAG-Vorkommen auf dieser Seite",
                    vm.severity.total
                ),
            )
        };
        assert!(text.contains(title), "{locale}: {text}");
        assert!(text.contains(&total), "{locale}: {total}");
        assert!(text.contains(step), "{locale}");
        assert_eq!(text.matches(step).count(), 1, "{locale}: said once");
    }
}
