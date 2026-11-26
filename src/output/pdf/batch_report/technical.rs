//! The technical part of the batch PDF (#1212): the audit frame, how the
//! recommendation is calculated and what the automated audit leaves open. The
//! cross-page tables behind the overview follow it (module scores per page,
//! audit caveats, shared signals, consistency); per-page detail is in the JSON
//! report, in the single reports and, at report level technical, in the detail
//! chapters at the end.

use crate::output::localized::is_english;
use renderreport::components::advanced::{KeyValueList, PageBreak, SectionHeaderSplit};
use renderreport::components::text::TextBlock;
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::output::report_model::*;

/// The header of the technical part. It starts a page, so the header never
/// stands alone at the foot of the page before.
pub(super) fn render_batch_technical_header(
    builder: renderreport::engine::ReportBuilder,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(i18n.t("batch-tech-title"), i18n.t("batch-tech-intro"))
            .with_level(1),
    )
}

/// The audit frame (domain, date, modules, version, display mode, exclusions)
/// and the method statements: how the recommendation is calculated and what
/// the automated audit leaves open. The scope of the run is on the overview
/// page.
pub(super) fn render_batch_method(
    mut builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let domain = &pres.portfolio_summary.domain;
    let modules = pres
        .portfolio_summary
        .active_modules
        .iter()
        .map(|name| super::portfolio::localized_module_name(name, i18n))
        .collect::<Vec<_>>()
        .join(", ");
    let mut frame = KeyValueList::new()
        .add(i18n.t("batch-cover-frame-domain"), domain)
        .add(i18n.t("batch-cover-frame-date"), &pres.cover.date)
        .add(
            i18n.t("batch-cover-frame-urls"),
            format!("{}", pres.portfolio_summary.total_urls),
        )
        .add(i18n.t("batch-cover-frame-modules"), &modules)
        .add(
            i18n.t("batch-cover-frame-version"),
            format!("auditmysite v{}", pres.cover.version),
        );

    // #653: the display mode every score in this report belongs to.
    let en = is_english(i18n);
    let mode = batch
        .reports
        .first()
        .map(|r| r.accessibility.execution.scope.display_mode)
        .unwrap_or_default();
    let mut display = crate::display::audited_mode_text(mode, en);
    let offering = batch
        .reports
        .iter()
        .filter(|r| {
            r.accessibility
                .execution
                .display_modes
                .as_ref()
                .is_some_and(|d| d.offers_display_modes)
        })
        .count();
    if offering > 0 {
        display.push_str(" — ");
        display.push_str(&i18n.t_args(
            "batch-display-offered",
            &[
                ("count", offering as i64),
                ("total", batch.reports.len() as i64),
            ],
        ));
    }
    frame = frame.add(i18n.t("batch-cover-frame-display"), display);

    if let Some(note) = crate::audit::exclusion::BatchExclusionSummary::aggregate(
        batch
            .reports
            .iter()
            .filter_map(|r| r.accessibility.execution.exclusions.as_ref()),
    )
    .and_then(|summary| {
        crate::audit::exclusion::batch_exclusion_note(&summary, batch.reports.len(), i18n)
    }) {
        frame = frame.add(i18n.t("exclusion-fact-label"), note);
    }
    builder = builder.add_component(frame);

    builder = builder
        .add_component(Section::new(i18n.t("batch-tech-method-title")).with_level(2))
        .add_component(TextBlock::new(i18n.t("batch-rating-note")));
    if pres.portfolio_summary.total_urls >= 2 {
        builder = builder.add_component(TextBlock::new(i18n.t("batch-tech-pick-rule")));
    }
    // What the score effect row of a cause card means and why the values do
    // not add up.
    if pres
        .top_issues
        .iter()
        .any(|group| super::management::batch_score_effect(group, i18n).is_some())
    {
        builder = builder.add_component(TextBlock::new(i18n.t("batch-management-score-gain-note")));
    }

    // The share comes from the criteria list the JSON reports as
    // `summary.wcag_coverage`, not from a fixed "30–40%" (#1039).
    let (automated, total) = crate::wcag::coverage::coverage_stats();
    builder.add_component(
        Callout::info(i18n.t_args(
            "batch-next-steps-manual",
            &[("automated", automated as i64), ("total", total as i64)],
        ))
        .with_title(i18n.t("section-next-steps-block")),
    )
}
