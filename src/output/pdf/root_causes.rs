//! Root-cause analysis (letters A, B, C… for the WCAG-mandatory causes) and
//! the timeframe roadmap that tags its systemic actions with those letters.
//!
//! Extracted verbatim from `single_report.rs` (plan 66, WP3; same cut as
//! `problem_profile.rs` in plan 27). Both sections share the letter
//! assignment, so they move together. No logic changed in the move.

use renderreport::components::advanced::{List, PageBreak, RecommendationCard, SectionHeaderSplit};
use renderreport::components::charts::Chart;
use renderreport::components::text::Label;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use super::design;
use super::helpers::truncate_with_ellipsis;
use super::problem_profile::build_overall_leverage_note;
use super::single_report::cross_module_measures;
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
        .filter(|f| f.occurrence_count > 0 && f.criticality_tier == CriticalityTier::Mandatory)
        .collect();
    findings.sort_by_key(|f| std::cmp::Reverse(f.occurrence_count));
    findings
}

/// `rule_id -> (letter, occurrence_count, share_pct)` for the top
/// `ROOT_CAUSE_SHOWN` mandatory root causes. Used by `render_timeframe_roadmap`
/// to tag systemic actions with the root cause they resolve, so the two
/// sections never disagree on which letter is which.
fn root_cause_lookup(
    findings: &[&FindingGroup],
) -> std::collections::HashMap<String, (char, usize, i64)> {
    let total_occurrences: usize = findings.iter().map(|f| f.occurrence_count).sum();
    findings
        .iter()
        .enumerate()
        .take(ROOT_CAUSE_SHOWN)
        .map(|(idx, f)| {
            let letter = (b'A' + idx as u8) as char;
            let share_pct = if total_occurrences > 0 {
                (f.occurrence_count as f64 * 100.0 / total_occurrences as f64).round() as i64
            } else {
                0
            };
            (f.rule_id.clone(), (letter, f.occurrence_count, share_pct))
        })
        .collect()
}

pub(super) fn render_root_cause_analysis(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en {
        "Root Cause Analysis"
    } else {
        "Ursachenanalyse"
    };
    let subtitle = if en {
        "Many individual findings trace back to a few recurring causes — fixing those has a compounding effect."
    } else {
        "Viele Einzelbefunde gehen auf wenige wiederkehrende Ursachen zurück – deren Behebung wirkt gebündelt."
    };

    // No leading PageBreak: the preceding part divider ("Befunde nach Ursache")
    // already opened the page — an extra break left that divider page 3/4 empty.
    builder = builder.add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(if en { "ROOT CAUSE" } else { "URSACHEN" })
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
            if en {
                "No accessibility findings were detected, so no root cause analysis is necessary."
                    .to_string()
            } else {
                "Es wurden keine Barrierefreiheits-Befunde erkannt, daher ist keine Ursachenanalyse erforderlich.".to_string()
            }
        } else if en {
            format!(
                "No confirmed WCAG violations, so no root cause analysis is necessary. {} accessibility warning(s) and {} manual check(s) remain — they are documented in their own sections and are not scored.",
                evidence.warnings, evidence.manual_checks
            )
        } else {
            format!(
                "Keine bestätigten WCAG-Verstöße, daher ist keine Ursachenanalyse erforderlich. Es bleiben {} Barrierefreiheits-Warnung(en) und {} manuelle(r) Prüfhinweis(e) — sie stehen in den eigenen Abschnitten und fließen nicht in den Score ein.",
                evidence.warnings, evidence.manual_checks
            )
        };
        builder = builder.add_component(if evidence.is_empty() {
            Callout::success(&msg)
        } else {
            Callout::info(&msg)
        });
        return builder;
    }

    // Plain lead-in (Rule A): spell out what "root cause" means in practice
    // before the technical list of causes/components follows below.
    let lead_in = if en {
        "In practice: fixing the right root cause — a shared component or template — often resolves several findings at once, instead of fixing each one individually."
    } else {
        "Das bedeutet konkret: Ein Fix an der richtigen Stelle – etwa einer wiederverwendeten Komponente oder einem Template – behebt oft mehrere Befunde gleichzeitig, statt jeden einzeln reparieren zu müssen."
    };
    builder = builder.add_component(Label::new(lead_in).with_size("10.5pt"));

    // Render root causes (assigning letters A, B, C, etc.)
    let mut list = List::new().with_title(if en {
        "Systemic Root Causes"
    } else {
        "Erkannte Kernursachen"
    });
    let mut table_rows = Vec::new();
    let mut chart_data: Vec<(String, f64)> = Vec::new();
    let total_occurrences: usize = findings.iter().map(|f| f.occurrence_count).sum();

    // Rounded (not truncating) share so a small-but-real cause reads as "1 %"
    // rather than a self-contradictory "0 %" next to its own listed row.
    let share_pct = |occurrences: usize| -> i64 {
        if total_occurrences > 0 {
            (occurrences as f64 * 100.0 / total_occurrences as f64).round() as i64
        } else {
            0
        }
    };

    let mut shown_occurrences = 0usize;

    for (idx, finding) in findings.iter().enumerate().take(ROOT_CAUSE_SHOWN) {
        let letter = (b'A' + idx as u8) as char;
        let item_title = format!(
            "{} {}: {} {} — {}",
            if en { "Root Cause" } else { "Ursache" },
            letter,
            finding.occurrence_count,
            if en { "occurrences" } else { "Vorkommen" },
            finding.title
        );
        list = list.add_item(&item_title);
        shown_occurrences += finding.occurrence_count;

        // Repeat the clear-text title next to the letter — otherwise the chart
        // and table only carry "Ursache A", forcing readers to flip back to
        // the list above to remember what it refers to.
        let short_title = truncate_with_ellipsis(&finding.title, 45);
        table_rows.push(vec![
            format!("{letter} — {short_title}"),
            finding.occurrence_count.to_string(),
            format!("{} %", share_pct(finding.occurrence_count)),
        ]);
        chart_data.push((
            format!("{letter} — {short_title}"),
            finding.occurrence_count as f64,
        ));
    }

    // Findings beyond the top ROOT_CAUSE_SHOWN would otherwise inflate
    // `total_occurrences` (the true, honest denominator) with no visible row
    // to account for them — the displayed shares silently failed to sum to
    // 100 %. Disclose the remainder explicitly instead (#QA-039 report review).
    if findings.len() > ROOT_CAUSE_SHOWN {
        let remaining_causes = findings.len() - ROOT_CAUSE_SHOWN;
        let remaining_occurrences = total_occurrences - shown_occurrences;
        let label = if en {
            format!("Other ({remaining_causes} further causes)")
        } else {
            format!("Sonstige ({remaining_causes} weitere Ursachen)")
        };
        table_rows.push(vec![
            label.clone(),
            remaining_occurrences.to_string(),
            format!("{} %", share_pct(remaining_occurrences)),
        ]);
        chart_data.push((label, remaining_occurrences as f64));
    }

    builder = builder.add_component(list);

    // Occurrence distribution as a bar chart — shows at a glance where the
    // findings concentrate (#7 distribution bars).
    if chart_data.len() >= 2 {
        builder = builder.add_component(
            Chart::bar(if en {
                "Occurrence distribution by cause"
            } else {
                "Verteilung der Vorkommen nach Ursache"
            })
            .add_series("occurrences", chart_data)
            .horizontal(),
        );
    }

    // Render the table: Ursache | Vorkommen | Anteil
    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Root Cause" } else { "Ursache" }).with_width("40%"),
        TableColumn::new(if en { "Occurrences" } else { "Vorkommen" }).with_width("30%"),
        TableColumn::new(if en { "Share" } else { "Anteil" }).with_width("30%"),
    ])
    .with_title(if en {
        "Distribution of Issues by Root Cause"
    } else {
        "Verteilung der Mängel nach Ursache"
    });

    for row in table_rows {
        table = table.add_row(row);
    }
    builder = builder.add_component(table);

    builder
}

pub(super) fn render_timeframe_roadmap(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en { "Action Plan" } else { "Maßnahmenplan" };
    // Disclose the scope mismatch with the root-cause section above: this plan
    // also includes SEO/Optimization-tier actions, which are not part of the
    // WCAG-only root-cause count (#5 fix).
    let subtitle = if en {
        "Recommended actions grouped by where the problem lives — including supplementary SEO and quality recommendations without legal relevance."
    } else {
        "Empfohlene Maßnahmen, gruppiert nach Ebene des Problems — inklusive ergänzender SEO- und Qualitätsempfehlungen ohne Rechtsbezug."
    };

    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(title, subtitle)
            .with_eyebrow(if en { "ROADMAP" } else { "MASSNAHMEN" })
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
            if en {
                format!(
                    "No prioritized accessibility actions. {module_measure_count} recommendation(s) from other modules (performance, SEO, security, mobile) are listed in their module sections and in the key measures above."
                )
            } else {
                format!(
                    "Keine priorisierten Barrierefreiheits-Maßnahmen. {module_measure_count} Empfehlung(en) aus anderen Modulen (Performance, SEO, Sicherheit, Mobile) stehen in den jeweiligen Modulabschnitten und in den wichtigsten Maßnahmen oben."
                )
            }
        } else if en {
            "No prioritized actions — no findings require remediation.".to_string()
        } else {
            "Keine priorisierten Maßnahmen — keine Befunde mit Handlungsbedarf.".to_string()
        };
        builder = builder.add_component(Label::new(empty_msg).with_color(design::tokens::NEUTRAL));
        return builder;
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
            // plan/6-remediation-leverage-metric.md: "is this worth doing
            // first" verdict — reach (occurrences), cost (effort) and risk
            // condensed into one line. "betroffen", not "behoben"/"beseitigt"
            // (plan/4-root-cause-confidence-wording.md): occurrence_count is
            // how many elements this rule affects, not a guarantee that one
            // fix resolves every one of them.
            let leverage_tag = if en {
                format!(
                    "Remediation leverage: {} — {} occurrence(s) affected · effort: {}",
                    item.leverage, item.occurrence_count, item.effort
                )
            } else {
                format!(
                    "Reparaturhebel: {} — {} Vorkommen betroffen · Aufwand: {}",
                    item.leverage, item.occurrence_count, item.effort
                )
            };
            why = format!("{why}\n\n{leverage_tag}");
            if let Some((letter, occurrence_count, share_pct)) =
                root_cause_by_rule.get(&item.rule_id)
            {
                let tag = if en {
                    format!("→ Root Cause {letter}, {occurrence_count} occurrences ({share_pct}%)")
                } else {
                    format!("→ Ursache {letter}, {occurrence_count} Vorkommen ({share_pct} %)")
                };
                why = format!("{why}\n\n{tag}");
            }
            builder = builder.add_component(RecommendationCard::new(item.action.clone(), why));
        }
    }

    builder
}
