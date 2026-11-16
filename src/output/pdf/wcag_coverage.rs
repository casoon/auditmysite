//! WCAG coverage section for PDF reports (issue #37).

use crate::output::localized::is_english;
use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, SectionHeaderSplit,
};
use renderreport::components::text::Label;
use renderreport::components::TagCloud;

use crate::audit::{AccessibilityScorer, AuditReport, CoverageRatio};
use crate::i18n::I18n;

/// Criteria with at least one violation in this run.
pub(super) fn violated_criteria(
    results: &crate::wcag::WcagResults,
) -> std::collections::BTreeSet<String> {
    results
        .violations
        .iter()
        .filter_map(|v| crate::taxonomy::criterion_for_rule(&v.rule))
        .collect()
}

/// Tag colour of an automatically checked criterion, derived from the same
/// execution states as the principle coverage ratios: only a criterion that
/// ran and found nothing is shown as passed (#902-1).
pub(super) fn criterion_tag_status(
    criterion: &str,
    states: &std::collections::BTreeMap<String, Option<bool>>,
    violated: &std::collections::BTreeSet<String>,
) -> &'static str {
    if violated.contains(criterion) {
        return "bad";
    }
    match states.get(criterion) {
        Some(Some(true)) => "good",
        Some(Some(false)) => "warn",
        _ => "neutral",
    }
}

/// Render a WCAG coverage section (issue #37).
///
/// Lists the criteria this tool automatically checks, plus the criteria
/// that fundamentally require manual review. Communicates audit scope
/// transparently so users avoid a false sense of security.
pub(super) fn render_wcag_coverage_section(
    mut builder: renderreport::engine::ReportBuilder,
    report: &AuditReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::wcag::coverage::{automated_criteria, coverage_stats, manual_review_criteria};

    let en = is_english(i18n);

    // Principle coverage — informative secondary indicator (#99).
    let coverage =
        AccessibilityScorer::calculate_execution_coverage(&report.accessibility.wcag_results);
    let cov_title = i18n.t("pdf-analysis-coverage-principles-title");
    let fmt_ratio =
        |r: &CoverageRatio| format!("{}/{} ({:.0} %)", r.passed, r.total, r.ratio * 100.0);
    builder = builder.add_component(
        KeyValueList::new()
            .with_title(cov_title)
            .add(
                i18n.t("pdf-analysis-coverage-perceivable"),
                fmt_ratio(&coverage.perceivable),
            )
            .add(
                i18n.t("pdf-analysis-coverage-operable"),
                fmt_ratio(&coverage.operable),
            )
            .add(
                i18n.t("pdf-analysis-coverage-understandable"),
                fmt_ratio(&coverage.understandable),
            )
            .add(
                i18n.t("pdf-analysis-coverage-robust"),
                fmt_ratio(&coverage.robust),
            ),
    );
    let (automated, total) = coverage_stats();
    // Criteria this tool checks automatically that WCAG 2.2 added (they didn't
    // exist in WCAG 2.1). They count toward `automated`/`total` like any other
    // A/AA criterion (plan 54 §1), but stay marked as new in WCAG 2.2: they
    // are in EN 301 549 V4.1.1 (mapped in this report's own appendix) but not
    // in V3.2.1 (#572).
    //
    // Nur A/AA: `automated` zählt keine AAA-Kriterien, und 2.4.12 ist sowohl
    // AAA als auch neu in 2.2 — es als "N davon" in die A/AA-Quote zu zählen
    // ergäbe wieder eine Zahl, die sich mit ihrer Bezugsgröße nicht deckt
    // (dieselbe Klasse Fehler wie in Plan 36 §1).
    let wcag22_criteria: Vec<&str> = automated_criteria()
        .iter()
        .filter(|(id, level)| *level != "AAA" && crate::wcag::coverage::is_wcag22_only(id))
        .map(|(id, _)| *id)
        .collect();
    let title = i18n.t("pdf-analysis-coverage-scope-title");
    let intro = if wcag22_criteria.is_empty() {
        i18n.t_args(
            "pdf-analysis-coverage-intro",
            &[
                ("automated", automated.to_string()),
                ("total", total.to_string()),
            ],
        )
    } else {
        i18n.t_args(
            "pdf-analysis-coverage-intro-new",
            &[
                ("automated", automated.to_string()),
                ("total", total.to_string()),
                ("count", wcag22_criteria.len().to_string()),
                ("list", wcag22_criteria.join(", ")),
            ],
        )
    };

    builder = builder.add_component(SectionHeaderSplit::new(title, &intro).with_level(2));

    // The cloud lists every criterion this tool checks; `automated` counts
    // only the A/AA ones the ratio above is scoped to. Titling the cloud with
    // `automated` put "Automatisch geprüft (36)" above 56 entries, with
    // nothing reconciling the two numbers (plan 36 §1).
    let listed = automated_criteria().len();
    // Two disjoint groups, so they add up to `listed`.
    let aaa_listed = automated_criteria()
        .iter()
        .filter(|(_, level)| *level == "AAA")
        .count();
    debug_assert_eq!(
        automated + aaa_listed,
        listed,
        "the two groups must partition the listed criteria"
    );
    let automated_title = i18n.t_args(
        "pdf-analysis-coverage-automated-title",
        &[("listed", listed.to_string())],
    );
    let listed_note = i18n.t_args(
        "pdf-analysis-coverage-listed-note",
        &[
            ("listed", listed.to_string()),
            ("automated", automated.to_string()),
            ("aaa", aaa_listed.to_string()),
        ],
    );
    let mut tag_cloud = TagCloud::new().with_title(&automated_title).with_gap("5pt");
    let states =
        AccessibilityScorer::criterion_execution_states(&report.accessibility.wcag_results);
    let violated = violated_criteria(&report.accessibility.wcag_results);
    for (c, l) in automated_criteria().iter() {
        let tag = if crate::wcag::coverage::is_wcag22_only(c) {
            format!("WCAG {} ({}, WCAG 2.2)", c, l)
        } else {
            format!("WCAG {} ({})", c, l)
        };
        tag_cloud = tag_cloud.add(tag, criterion_tag_status(c, &states, &violated));
    }
    builder = builder.add_component(tag_cloud);
    let listed_note = format!("{listed_note} {}", i18n.t("pdf-analysis-coverage-legend"));
    builder = builder.add_component(
        Label::new(listed_note)
            .with_size("8.8pt")
            .with_color(crate::output::pdf::design::tokens::MUTED),
    );

    let manual_title = i18n.t_args(
        "pdf-analysis-coverage-manual-title",
        &[("count", manual_review_criteria().len().to_string())],
    );
    let mut manual_cloud = TagCloud::new().with_title(&manual_title).with_gap("5pt");
    for (c, l, name) in manual_review_criteria().iter() {
        let localized_name = if en {
            *name
        } else {
            crate::wcag::coverage::manual_review_criterion_name_de(c, name)
        };
        manual_cloud = manual_cloud.add(format!("{c} ({l}) – {localized_name}"), "info");
    }
    builder = builder.add_component(manual_cloud);

    // Practical testing guide — how to test the manual criteria above
    let how_title = i18n.t("pdf-analysis-coverage-how-title");
    let rows: Vec<ChecklistRow> = [
        (
            "pdf-analysis-coverage-test-keyboard-title",
            "pdf-analysis-coverage-test-keyboard-text",
        ),
        (
            "pdf-analysis-coverage-test-screen-reader-title",
            "pdf-analysis-coverage-test-screen-reader-text",
        ),
        (
            "pdf-analysis-coverage-test-zoom-title",
            "pdf-analysis-coverage-test-zoom-text",
        ),
        (
            "pdf-analysis-coverage-test-motion-title",
            "pdf-analysis-coverage-test-motion-text",
        ),
        (
            "pdf-analysis-coverage-test-modal-title",
            "pdf-analysis-coverage-test-modal-text",
        ),
        (
            "pdf-analysis-coverage-test-color-title",
            "pdf-analysis-coverage-test-color-text",
        ),
    ]
    .iter()
    .map(|(title, text)| ChecklistRow::new(i18n.t(title), i18n.t(text)).with_status("info"))
    .collect();
    builder.add_component(ChecklistPanel::new(rows).with_title(how_title))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn tag_status_marks_only_passed_criteria_as_good() {
        let mut states = BTreeMap::new();
        states.insert("1.1.1".to_string(), Some(true));
        states.insert("1.4.3".to_string(), Some(false));
        states.insert("2.5.3".to_string(), Some(false));
        states.insert("1.2.1".to_string(), None);
        let violated: BTreeSet<String> = ["2.5.3".to_string()].into_iter().collect();

        assert_eq!(criterion_tag_status("1.1.1", &states, &violated), "good");
        // Not passed without a confirmed violation: needs review, not green.
        assert_eq!(criterion_tag_status("1.4.3", &states, &violated), "warn");
        // A violated criterion is never green, whatever its state says.
        assert_eq!(criterion_tag_status("2.5.3", &states, &violated), "bad");
        assert_eq!(criterion_tag_status("1.2.1", &states, &violated), "neutral");
        assert_eq!(criterion_tag_status("9.9.9", &states, &violated), "neutral");

        let mut contradicting = states.clone();
        contradicting.insert("2.5.3".to_string(), Some(true));
        assert_eq!(
            criterion_tag_status("2.5.3", &contradicting, &violated),
            "bad"
        );
    }
}
