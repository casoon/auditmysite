//! Root-cause analysis (letters A, B, C… for the WCAG-mandatory causes) and
//! the timeframe roadmap that tags its systemic actions with those letters.
//!
//! Extracted verbatim from `single_report.rs` (plan 66, WP3; same cut as
//! `problem_profile.rs` in plan 27). Both sections share the letter
//! assignment, so they move together. No logic changed in the move.

use crate::output::localized::is_english;
use renderreport::components::advanced::{PageBreak, RecommendationCard, SectionHeaderSplit};
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use super::design;
use super::problem_profile::build_overall_leverage_note;
use super::single_report::{cross_module_measures, top_measures};
use crate::i18n::I18n;
use crate::output::report_model::*;

/// Max number of root causes assigned a letter (A, B, C…) in the root-cause
/// analysis section. Shared with `render_timeframe_roadmap` so both sections
/// agree on which letter identifies which cause.
const ROOT_CAUSE_SHOWN: usize = 6;

/// WCAG-Mandatory-tier findings sorted by occurrence count (descending) — the
/// exact pool `render_root_cause_analysis` assigns letters A, B, C… from.
fn mandatory_root_causes(vm: &ReportViewModel) -> Vec<&FindingGroup> {
    let mut findings: Vec<&FindingGroup> = vm
        .findings
        .all_findings
        .iter()
        .filter(|f| {
            f.finding.occurrence_count > 0 && f.criticality_tier == CriticalityTier::Mandatory
        })
        .collect();
    findings.sort_by_key(|f| std::cmp::Reverse(f.finding.occurrence_count));
    findings
}

/// `rule_id -> (letter, occurrence_count, share_pct)` for the top
/// `ROOT_CAUSE_SHOWN` mandatory root causes. Used by `render_timeframe_roadmap`
/// to tag systemic actions with the root cause they resolve, so the two
/// sections never disagree on which letter is which.
fn root_cause_lookup(
    findings: &[&FindingGroup],
) -> std::collections::HashMap<String, (char, usize, i64)> {
    let total_occurrences: usize = findings.iter().map(|f| f.finding.occurrence_count).sum();
    findings
        .iter()
        .enumerate()
        .take(ROOT_CAUSE_SHOWN)
        .map(|(idx, f)| {
            let letter = (b'A' + idx as u8) as char;
            let share_pct = if total_occurrences > 0 {
                (f.finding.occurrence_count as f64 * 100.0 / total_occurrences as f64).round()
                    as i64
            } else {
                0
            };
            (
                f.finding.rule_id.clone(),
                (letter, f.finding.occurrence_count, share_pct),
            )
        })
        .collect()
}

pub(super) fn render_root_cause_analysis(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let title = i18n.t("pdf-analysis-causes-title");
    let subtitle = i18n.t("pdf-analysis-causes-subtitle");

    // No leading PageBreak: the preceding part divider ("Befunde nach Ursache")
    // already opened the page — an extra break left that divider page 3/4 empty.
    builder = builder.add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(i18n.t("pdf-analysis-causes-eyebrow"))
            .with_level(2),
    );

    // Only WCAG/Mandatory findings — scope matches the header "N Accessibility-Befunde" count.
    // SEO findings (Optimization tier) appear in their own section below.
    let findings = mandatory_root_causes(vm);

    if findings.is_empty() {
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
        return builder;
    }

    // Plain lead-in (Rule A). One page shows where findings concentrate,
    // not that they share a cause — a shared component is a possibility for
    // the developer to check (plan 75).
    let lead_in = i18n.t("pdf-analysis-causes-lead-in");
    builder = builder.add_component(Label::new(lead_in).with_size("10.5pt"));

    // One table with the full titles. A list, a bar chart and this table
    // used to repeat the same numbers three times (plan 75).
    let total_occurrences: usize = findings.iter().map(|f| f.finding.occurrence_count).sum();
    // Rounded (not truncating) share so a small-but-real cause reads as "1 %"
    // rather than a self-contradictory "0 %" next to its own listed row.
    let share_pct = |occurrences: usize| -> i64 {
        if total_occurrences > 0 {
            (occurrences as f64 * 100.0 / total_occurrences as f64).round() as i64
        } else {
            0
        }
    };
    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("single-report-matrix-col-finding")).with_width("64%"),
        TableColumn::new(i18n.t("batch-col-occurrences")).with_width("18%"),
        TableColumn::new(i18n.t("batch-col-share")).with_width("18%"),
    ])
    .with_title(i18n.t("pdf-analysis-causes-table-title"));
    let mut shown_occurrences = 0usize;
    for finding in findings.iter().take(ROOT_CAUSE_SHOWN) {
        shown_occurrences += finding.finding.occurrence_count;
        table = table.add_row(vec![
            finding.title.clone(),
            finding.finding.occurrence_count.to_string(),
            format!("{} %", share_pct(finding.finding.occurrence_count)),
        ]);
    }
    // Findings beyond the top ROOT_CAUSE_SHOWN still count in the
    // denominator; disclose them so the shares add up to 100 % (#QA-039).
    if findings.len() > ROOT_CAUSE_SHOWN {
        let remaining_findings = findings.len() - ROOT_CAUSE_SHOWN;
        let remaining_occurrences = total_occurrences - shown_occurrences;
        table = table.add_row(vec![
            i18n.t_args(
                "pdf-analysis-causes-other",
                &[
                    ("count", remaining_findings.to_string()),
                    (
                        "more",
                        i18n.count("count-more-findings", remaining_findings),
                    ),
                ],
            ),
            remaining_occurrences.to_string(),
            format!("{} %", share_pct(remaining_occurrences)),
        ]);
    }
    builder = builder.add_component(table);

    builder
}

/// One card per key measure that is not a finding group of its own: module
/// recommendations (security, mobile, performance, SEO). The key-measures list
/// names them, so the plan must carry them too (#902-8).
fn render_module_measure_cards(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
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

pub(super) fn render_timeframe_roadmap(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let title = i18n.t("pdf-analysis-roadmap-title");
    // Disclose the scope mismatch with the root-cause section above: this plan
    // also includes SEO/Optimization-tier actions, which are not part of the
    // WCAG-only root-cause count (#5 fix).
    let subtitle = i18n.t("pdf-analysis-roadmap-subtitle");

    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(i18n.t("pdf-analysis-roadmap-eyebrow"))
            .with_level(2),
    );

    // Overall-leverage sentence (plan/7-management-summary-consolidation.md):
    // reuses the same problem-concentration numbers shown earlier
    // (plan/3-problem-concentration-diagnosis-narrative.md) but framed
    // around leverage rather than diagnosis — not literally the same
    // sentence twice.
    if let Some(note) = build_overall_leverage_note(vm, i18n) {
        builder = builder.add_component(Label::new(note).with_size("10.5pt"));
    }

    let columns = &vm.actions.roadmap_columns;
    if columns.is_empty() {
        // The roadmap is built from WCAG/finding groups only. Saying "no
        // findings require remediation" here contradicted the report whenever
        // other modules had documented work, so name where that work is
        // instead of claiming there is none.
        let module_measure_count = cross_module_measures(vm, i18n).len();
        let empty_msg = if module_measure_count > 0 {
            i18n.t_args(
                "pdf-analysis-roadmap-empty-modules",
                &[
                    ("count", module_measure_count.to_string()),
                    (
                        "recommendations",
                        i18n.count("count-recommendations", module_measure_count),
                    ),
                ],
            )
        } else {
            i18n.t("pdf-analysis-roadmap-empty")
        };
        builder = builder.add_component(Label::new(empty_msg).with_color(design::tokens::NEUTRAL));
        return render_module_measure_cards(builder, vm, i18n);
    }

    // Same letter assignment as `render_root_cause_analysis`, so an action
    // tied to root cause "A" always points at the same finding the reader saw
    // lettered "A" above (#2 fix).
    let root_causes = mandatory_root_causes(vm);
    let root_cause_by_rule = root_cause_lookup(&root_causes);

    // One level group per column (systemic vs. local), each action as a
    // clean recommendation card — what to do, why, and (plan/6) a
    // remediation-leverage line instead of a separate visual effort badge.
    for col in columns {
        builder = builder.add_component(
            SectionHeaderSplit::new(col.title.clone(), col.description.clone()).with_level(3),
        );
        for item in &col.items {
            let mut why = if !item.benefit.is_empty() {
                item.benefit.clone()
            } else {
                item.risk_effect.clone()
            };
            if !item.hint.is_empty() {
                why = if why.is_empty() {
                    item.hint.clone()
                } else {
                    format!("{}\n\n{why}", item.hint)
                };
            }
            // plan/6-remediation-leverage-metric.md: "is this worth doing
            // first" verdict — reach (occurrences), cost (effort) and risk
            // condensed into one line. "betroffen", not "behoben"/"beseitigt"
            // (plan/4-root-cause-confidence-wording.md): occurrence_count is
            // how many elements this rule affects, not a guarantee that one
            // fix resolves every one of them.
            let leverage_tag = i18n.t_args(
                "pdf-analysis-roadmap-leverage",
                &[
                    ("leverage", item.leverage.clone()),
                    ("count", item.occurrence_count.to_string()),
                    ("effort", item.effort.clone()),
                ],
            );
            why = format!("{why}\n\n{leverage_tag}");
            if let Some((letter, occurrence_count, share_pct)) =
                root_cause_by_rule.get(&item.rule_id)
            {
                let tag = i18n.t_args(
                    "pdf-analysis-roadmap-root-cause",
                    &[
                        ("letter", letter.to_string()),
                        ("count", occurrence_count.to_string()),
                        ("share", share_pct.to_string()),
                    ],
                );
                why = format!("{why}\n\n{tag}");
            }
            builder = builder.add_component(RecommendationCard::new(item.action.clone(), why));
        }
    }

    render_module_measure_cards(builder, vm, i18n)
}
