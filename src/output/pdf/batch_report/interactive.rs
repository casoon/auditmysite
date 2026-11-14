use renderreport::components::advanced::SectionHeaderSplit;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;

use super::super::helpers::severity_label_i18n;

// ─── Interactive Journey Summary ────────────────────────────────────────────

pub(super) fn translate_interactive_category(category: &str, i18n: &I18n) -> String {
    i18n.t(match category {
        "TabOrder" => "batch-report-interactive-cat-tab-order",
        "FocusTrap" => "batch-report-interactive-cat-focus-trap",
        "StateTransition" => "batch-report-interactive-cat-state-transition",
        "FocusRestoration" => "batch-report-interactive-cat-focus-restoration",
        "FormError" => "batch-report-interactive-cat-form-error",
        "SpaNavigation" => "batch-report-interactive-cat-spa-navigation",
        "HiddenFocusable" => "batch-report-interactive-cat-hidden-focusable",
        "SkipLink" => "batch-report-interactive-cat-skip-link",
        "FocusIndicator" => "batch-report-interactive-cat-focus-indicator",
        "MenuJourney" => "batch-report-interactive-cat-menu",
        "TabsJourney" => "batch-report-interactive-cat-tabs",
        "LinkText" => "batch-report-interactive-cat-link-text",
        "Landmark" => "batch-report-interactive-cat-landmark",
        "MediaControls" => "batch-report-interactive-cat-media-controls",
        "HeadingOutline" => "batch-report-interactive-cat-heading-outline",
        _ => return category.to_string(),
    })
}

/// The summary sentence; verb and noun agree with the counts (#1039).
pub(super) fn interactive_callout_text(
    summary: &crate::output::report_model::InteractiveJourneySummary,
    i18n: &I18n,
) -> String {
    let total = summary.total_pages_tested as i64;
    if summary.pages_with_issues == 0 {
        return i18n.t_args("batch-interactive-none", &[("total", total)]);
    }
    let with_issues = summary.pages_with_issues as i64;
    i18n.t_args(
        "batch-interactive-some",
        &[
            ("count", with_issues),
            ("total", total),
            ("clean", total - with_issues),
        ],
    )
}

pub(super) fn render_batch_interactive_summary(
    mut builder: renderreport::engine::ReportBuilder,
    summary: &crate::output::report_model::InteractiveJourneySummary,
    total_urls: usize,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("a11y-details-journey-title"),
            i18n.t("batch-report-interactive-intro"),
        )
        .with_level(1),
    );

    let callout_text = interactive_callout_text(summary, i18n);
    builder = builder.add_component(if summary.has_critical {
        Callout::warning(&callout_text)
    } else if summary.pages_with_issues > 0 {
        Callout::info(&callout_text)
    } else {
        Callout::success(&callout_text)
    });

    if !summary.categories.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-report-interactive-col-category")),
            TableColumn::new(i18n.t("batch-report-interactive-col-affected")),
            TableColumn::new(i18n.t("batch-col-share")),
            TableColumn::new(i18n.t("batch-report-interactive-col-severity")),
        ])
        .with_title(i18n.t("batch-report-interactive-table-title"));
        for row in &summary.categories {
            let pct = (row.affected_urls * 100)
                .checked_div(total_urls)
                .unwrap_or(0);
            let sev_label = severity_label_i18n(row.max_severity, i18n);
            let category_label = translate_interactive_category(&row.category, i18n);
            table = table.add_row(vec![
                category_label,
                row.affected_urls.to_string(),
                format!("{pct}%"),
                sev_label,
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}
