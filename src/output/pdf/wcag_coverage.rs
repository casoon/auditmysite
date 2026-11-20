//! WCAG coverage section for PDF reports (issue #37).

use crate::output::localized::is_english;
use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, SectionHeaderSplit,
};
use renderreport::components::text::Label;
use renderreport::components::TagCloud;

use crate::audit::{AccessibilityScorer, AuditReport, CoverageRatio, CriterionState};
use crate::i18n::I18n;

/// Tag colour of an automatically checked criterion, derived from the same
/// criterion states as the principle coverage counts: only a criterion that
/// ran and found nothing is shown as passed, and a violated one is never
/// green (#902-1, #1135).
pub(super) fn criterion_tag_status(
    criterion: &str,
    states: &std::collections::BTreeMap<String, CriterionState>,
) -> &'static str {
    match states.get(criterion) {
        Some(CriterionState::Violation) => "bad",
        Some(CriterionState::Passed) => "good",
        Some(CriterionState::Review | CriterionState::NotTestable) => "warn",
        Some(CriterionState::NotApplicable) | None => "neutral",
    }
}

/// One principle's coverage as the report words it: the passed share, then
/// the criteria that are not passed, by state, so "11/12" explains itself
/// (#1135). Zero counts are left out.
fn format_coverage(ratio: &CoverageRatio, i18n: &I18n) -> String {
    let base = format!(
        "{}/{} ({:.0} %)",
        ratio.passed,
        ratio.total,
        ratio.ratio * 100.0
    );
    let parts: Vec<String> = [
        ("pdf-analysis-coverage-state-violation", ratio.violation),
        ("pdf-analysis-coverage-state-review", ratio.review),
        (
            "pdf-analysis-coverage-state-not-testable",
            ratio.not_testable,
        ),
        (
            "pdf-analysis-coverage-state-not-applicable",
            ratio.not_applicable,
        ),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(key, count)| i18n.t_args(key, &[("count", count.to_string())]))
    .collect();
    if parts.is_empty() {
        base
    } else {
        format!("{base} — {}", parts.join(", "))
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
    let fmt_ratio = |r: &CoverageRatio| format_coverage(r, i18n);
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
    let states = AccessibilityScorer::criterion_states(&report.accessibility.wcag_results);
    for (c, l) in automated_criteria().iter() {
        let tag = if crate::wcag::coverage::is_wcag22_only(c) {
            format!("WCAG {} ({}, WCAG 2.2)", c, l)
        } else {
            format!("WCAG {} ({})", c, l)
        };
        tag_cloud = tag_cloud.add(tag, criterion_tag_status(c, &states));
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
    use std::collections::BTreeMap;

    #[test]
    fn tag_status_marks_only_passed_criteria_as_good() {
        let states = BTreeMap::from([
            ("1.1.1".to_string(), CriterionState::Passed),
            ("1.4.3".to_string(), CriterionState::Review),
            ("1.4.4".to_string(), CriterionState::NotTestable),
            ("2.5.3".to_string(), CriterionState::Violation),
            ("1.2.1".to_string(), CriterionState::NotApplicable),
        ]);
        assert_eq!(criterion_tag_status("1.1.1", &states), "good");
        // Not passed without a confirmed violation: needs review, not green.
        assert_eq!(criterion_tag_status("1.4.3", &states), "warn");
        assert_eq!(criterion_tag_status("1.4.4", &states), "warn");
        // A violated criterion is never green.
        assert_eq!(criterion_tag_status("2.5.3", &states), "bad");
        assert_eq!(criterion_tag_status("1.2.1", &states), "neutral");
        assert_eq!(criterion_tag_status("9.9.9", &states), "neutral");
    }

    /// #1135: "11/12" explains itself: the criterion that is not passed is
    /// named by its state, in both languages, and zero counts stay out.
    #[test]
    fn coverage_names_the_states_of_the_criteria_that_are_not_passed() {
        let mut ratio = CoverageRatio {
            passed: 11,
            review: 1,
            total: 12,
            ratio: 0.9167,
            ..Default::default()
        };
        let de = format_coverage(&ratio, &I18n::new("de").unwrap());
        assert_eq!(de, "11/12 (92 %) — 1 zur manuellen Prüfung");
        let en = format_coverage(&ratio, &I18n::new("en").unwrap());
        assert_eq!(en, "11/12 (92 %) — 1 need manual review");

        ratio.violation = 2;
        ratio.not_testable = 1;
        ratio.not_applicable = 3;
        let en = format_coverage(&ratio, &I18n::new("en").unwrap());
        assert_eq!(
            en,
            "11/12 (92 %) — 2 with a violation, 1 need manual review, 1 not testable or not run, 3 not applicable"
        );

        let clean = CoverageRatio {
            passed: 4,
            total: 4,
            ratio: 1.0,
            ..Default::default()
        };
        assert_eq!(
            format_coverage(&clean, &I18n::new("en").unwrap()),
            "4/4 (100 %)"
        );
    }
}
