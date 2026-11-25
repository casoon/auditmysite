//! The analysis level of the single report (#1171): each finding told once in
//! full, grouped by the cause the audit observed. It replaces the root-cause
//! table and the action-plan cards, which told the same findings twice.
//!
//! A cause is a recurring element pattern: the most frequent pattern of a
//! finding (`NormalizedFinding::patterns`, the same normalized selectors as
//! the root-cause ids of the JSON) when it has at least two elements.
//! Findings that share such a pattern stand together; a finding without one
//! has no observed cause and goes to the final group "other findings".

use renderreport::components::advanced::SectionHeaderSplit;
use renderreport::components::text::Label;
use renderreport::prelude::*;

use super::analysis_card::{analysis_card, AnalysisCard};
use super::design;
use super::problem_profile::build_overall_leverage_note;
use super::single_report::{cross_module_measures, top_measures};
use crate::i18n::I18n;
use crate::output::report_model::*;

/// Longest selector a cause heading prints.
const CAUSE_SELECTOR_CHARS: usize = 70;

/// A pattern of fewer elements is no recurrence.
const MIN_PATTERN_ELEMENTS: usize = 2;

/// The findings that share one observed cause.
struct Cause<'a> {
    /// Normalized selector of the shared pattern; `None` for the findings
    /// without an observed cause.
    selector: Option<String>,
    /// Elements of the largest pattern among the findings.
    elements: usize,
    findings: Vec<&'a FindingGroup>,
}

/// The recurring pattern of a finding: its normalized selector and its
/// element count.
fn observed_cause(group: &FindingGroup) -> Option<(String, usize)> {
    group
        .finding
        .patterns
        .first()
        .filter(|pattern| pattern.occurrence_count >= MIN_PATTERN_ELEMENTS)
        .map(|pattern| {
            (
                a11y_report::normalize_selector(&pattern.selector),
                pattern.occurrence_count,
            )
        })
}

/// The findings of the page grouped by cause: the causes with the most
/// elements first, the findings without a cause last.
fn causes(vm: &ReportViewModel) -> Vec<Cause<'_>> {
    let mut causes: Vec<Cause> = Vec::new();
    let mut other: Vec<&FindingGroup> = Vec::new();
    for group in vm
        .findings
        .all_findings
        .iter()
        .filter(|f| f.finding.occurrence_count > 0)
    {
        match observed_cause(group) {
            Some((selector, elements)) => {
                match causes
                    .iter_mut()
                    .find(|c| c.selector.as_deref() == Some(selector.as_str()))
                {
                    Some(cause) => {
                        cause.elements = cause.elements.max(elements);
                        cause.findings.push(group);
                    }
                    None => causes.push(Cause {
                        selector: Some(selector),
                        elements,
                        findings: vec![group],
                    }),
                }
            }
            None => other.push(group),
        }
    }
    causes.sort_by_key(|c| std::cmp::Reverse(c.elements));
    if !other.is_empty() {
        causes.push(Cause {
            selector: None,
            elements: 0,
            findings: other,
        });
    }
    causes
}

/// The cause row of a card and how sure the report is about it. A recurring
/// pattern is observed; a shared component behind it is not.
fn cause_text(cause: &Cause, i18n: &I18n) -> String {
    match &cause.selector {
        Some(selector) => i18n.t_args(
            "pdf-analysis-cause-markup",
            &[
                (
                    "selector",
                    fluent_bundle::FluentValue::from(crate::util::truncate_url(
                        selector,
                        CAUSE_SELECTOR_CHARS,
                    )),
                ),
                (
                    "count",
                    fluent_bundle::FluentValue::from(cause.elements as i64),
                ),
            ],
        ),
        None => i18n.t("pdf-analysis-cause-none"),
    }
}

/// One card per key measure that is not a finding group of its own: module
/// recommendations (security, mobile, performance, SEO). The key-measures list
/// names them, so the analysis must carry them too (#902-8).
fn render_module_measure_cards(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = crate::output::localized::is_english(i18n);
    let measures: Vec<_> = top_measures(vm, i18n)
        .into_iter()
        .filter(|(measure, _)| !measure.from_findings)
        .collect();
    if measures.is_empty() {
        return builder;
    }
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("pdf-analysis-roadmap-modules-title"),
            i18n.t("pdf-analysis-roadmap-modules-subtitle"),
        )
        .with_level(3),
    );
    // These measures carry no impact or effort of their own, so they are a
    // compact list, not cards with only a title and a sentence (#952).
    let mut list = renderreport::components::advanced::List::new();
    for (measure, extra) in &measures {
        let (title, body) = measure.card_parts(*extra, en);
        list = list.add_item(format!("{title}: {body}"));
    }
    builder.add_component(list)
}

pub(super) fn render_analysis(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // No leading PageBreak: the preceding part divider already opened the page.
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("pdf-analysis-title"),
            i18n.t("pdf-analysis-subtitle"),
        )
        .with_eyebrow(i18n.t("pdf-analysis-eyebrow"))
        .with_level(2),
    );

    let causes = causes(vm);
    if causes.is_empty() {
        // "No accessibility findings were detected" was false whenever the
        // journey or screen-reader modules had produced findings — they are
        // just not confirmed WCAG violations and therefore out of scope here.
        // Name the other two evidence classes instead of denying them.
        let evidence = &vm.findings.evidence;
        let msg = if evidence.is_empty() {
            i18n.t("pdf-analysis-causes-none")
        } else {
            i18n.t_args(
                "pdf-analysis-causes-none-evidence",
                &[
                    ("warnings", evidence.warnings.to_string()),
                    ("manual", evidence.manual_checks.to_string()),
                    (
                        "warnings_text",
                        i18n.count("count-a11y-warnings", evidence.warnings as usize),
                    ),
                    (
                        "manual_text",
                        i18n.count("count-manual-hints", evidence.manual_checks as usize),
                    ),
                ],
            )
        };
        builder = builder.add_component(if evidence.is_empty() {
            Callout::success(&msg)
        } else {
            Callout::info(&msg)
        });
        // The cards below carry only findings; the other modules' measures
        // are named instead of claiming there is none to do.
        let module_measure_count = cross_module_measures(vm, i18n).len();
        if module_measure_count > 0 {
            builder = builder.add_component(
                Label::new(i18n.t_args(
                    "pdf-analysis-roadmap-empty-modules",
                    &[
                        ("count", module_measure_count.to_string()),
                        (
                            "recommendations",
                            i18n.count("count-recommendations", module_measure_count),
                        ),
                    ],
                ))
                .with_color(design::tokens::NEUTRAL),
            );
        }
        return render_module_measure_cards(builder, vm, i18n);
    }

    // Plain lead-in (Rule A). One page shows where findings concentrate, not
    // that they share a cause — a shared component is a possibility for the
    // developer to check.
    builder = builder
        .add_component(Label::new(i18n.t("pdf-analysis-causes-lead-in")).with_size("10.5pt"));
    if let Some(note) = build_overall_leverage_note(vm, i18n) {
        builder = builder.add_component(Label::new(note).with_size("10.5pt"));
    }

    // The cards of one cause stand together; each names its cause.
    for cause in &causes {
        let cause_text = cause_text(cause, i18n);
        for group in &cause.findings {
            builder = builder.add_component(analysis_card(
                group,
                AnalysisCard {
                    reach: i18n.count(
                        "single-report-decision-reach",
                        group.finding.occurrence_count,
                    ),
                    cause: Some(cause_text.clone()),
                    context_rows: Vec::new(),
                    score_effect: None,
                },
                i18n,
            ));
        }
    }

    render_module_measure_cards(builder, vm, i18n)
}
