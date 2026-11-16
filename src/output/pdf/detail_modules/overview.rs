use super::*;

pub(in crate::output::pdf) fn render_search_experience(
    mut builder: renderreport::engine::ReportBuilder,
    sx: &SearchExperiencePresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let title = i18n.t("pdf-sections-search-title");
    let body = i18n.t("pdf-sections-search-body");

    let sx_takeaway = super::first_sentence(&sx.interpretation);
    builder = super::module_chapter_opener(builder, &title, &sx_takeaway, is_first);

    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), sx.score)
                .with_description(&sx.interpretation)
                .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(&body)
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        );
    if sx.score < 75 {
        builder = builder.add_component(module_customer_context(
            i18n,
            "search_experience",
            Some(sx.score),
            &sx.interpretation,
        ));
    }

    // Warnings before the breakdown table: `module_customer_context` above just
    // made a claim ("shows visible improvement potential") with no specifics —
    // the reader needs the concrete reason right there, not several paragraphs
    // and a methodology table later. The breakdown table still follows as the
    // full supporting detail, not as the reader's only path to the "why".
    if !sx.warnings.is_empty() {
        let mut list = List::new().with_title(i18n.t("pdf-sections-search-warnings"));
        for warning in &sx.warnings {
            list = list.add_item(warning);
        }
        builder = builder.add_component(list);
    }

    if !sx.components.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-sec-score-component")).with_width("28%"),
            TableColumn::new(i18n.t("pdf-sections-search-col-weight")).with_width("14%"),
            TableColumn::new("Score").with_width("14%"),
            TableColumn::new(i18n.t("pdf-sections-search-col-meaning")).with_width("44%"),
        ])
        .with_title(i18n.t("pdf-sections-search-table-title"));
        for component in &sx.components {
            table = table.add_row(vec![
                component.label.clone(),
                format!("{}%", component.weight_pct),
                format!("{}/100", component.score),
                component.explanation.clone(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

pub(in crate::output::pdf) fn render_budget_violations(
    mut builder: renderreport::engine::ReportBuilder,
    violations: &[crate::audit::BudgetViolation],
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::audit::BudgetSeverity;

    builder =
        builder.add_component(Section::new(i18n.t("section-perf-budget-violations")).with_level(2));

    let error_count = violations
        .iter()
        .filter(|v| v.severity == BudgetSeverity::Error)
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| v.severity == BudgetSeverity::Warning)
        .count();
    let summary_text = i18n.t_args(
        "pdf-budget-violations-summary",
        &[
            ("total", violations.len().to_string()),
            ("error_count", error_count.to_string()),
            ("warning_count", warning_count.to_string()),
        ],
    );

    builder = if error_count > 0 {
        builder.add_component(
            Callout::warning(&summary_text).with_title(i18n.t("budget-callout-exceeded")),
        )
    } else {
        builder.add_component(
            Callout::info(&summary_text).with_title(i18n.t("budget-callout-warnings")),
        )
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("budget-table-metric")),
        TableColumn::new("Budget"),
        TableColumn::new(i18n.t("budget-table-actual")),
        TableColumn::new(i18n.t("budget-table-overage")),
        TableColumn::new(i18n.t("label-severity")),
    ])
    .with_title(i18n.t("budget-table-title"));

    // Metric names are stored in canonical English; re-localize the few
    // translatable ones for the PDF (#406). Acronyms (LCP, TBT, …) pass through.
    let localize_metric = |metric: &str| -> String {
        match metric {
            "JS size" => i18n.t("pdf-sections-budget-js-size"),
            "CSS size" => i18n.t("pdf-sections-budget-css-size"),
            "Page size" => i18n.t("pdf-sections-budget-page-size"),
            other => other.to_string(),
        }
    };

    for v in violations {
        table = table.add_row(vec![
            localize_metric(&v.metric),
            v.budget_label.clone(),
            v.actual_label.clone(),
            format!("+{:.0}%", v.exceeded_by_pct),
            match v.severity {
                crate::audit::budget::BudgetSeverity::Error => i18n.t("severity-error"),
                crate::audit::budget::BudgetSeverity::Warning => i18n.t("severity-warning"),
            },
        ]);
    }

    builder = builder.add_component(table);
    builder
}
