use super::*;

pub(in crate::output::pdf) fn render_search_experience(
    mut builder: renderreport::engine::ReportBuilder,
    sx: &SearchExperiencePresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = i18n.locale() == "en";
    let title = if en {
        "Search Experience"
    } else {
        "Sichtbarkeit & Nutzerverständnis"
    };
    let body = if en {
        "This composite combines classic technical SEO with content clarity, trust signals, AI readability, semantic structure and mobile readability. Classic SEO remains visible in the next detail section."
    } else {
        "Dieser Gesamtwert verbindet klassisches technisches SEO mit Inhaltsverständlichkeit, Vertrauenssignalen, KI-Lesbarkeit, semantischer Struktur und mobiler Lesbarkeit. Das klassische SEO bleibt im nächsten Detailkapitel sichtbar."
    };

    let sx_takeaway = super::first_sentence(&sx.interpretation);
    builder = super::module_chapter_opener(builder, title, &sx_takeaway, is_first);

    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), sx.score)
                .with_description(&sx.interpretation)
                .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(body)
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(module_customer_context(
            i18n,
            "search_experience",
            sx.score,
            &sx.interpretation,
        ));

    // Warnings before the breakdown table: `module_customer_context` above just
    // made a claim ("shows visible improvement potential") with no specifics —
    // the reader needs the concrete reason right there, not several paragraphs
    // and a methodology table later. The breakdown table still follows as the
    // full supporting detail, not as the reader's only path to the "why".
    if !sx.warnings.is_empty() {
        let mut list = List::new().with_title(if en {
            "What still goes wrong"
        } else {
            "Was noch schief läuft"
        });
        for warning in &sx.warnings {
            list = list.add_item(warning);
        }
        builder = builder.add_component(list);
    }

    if !sx.components.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if en { "Component" } else { "Bestandteil" }).with_width("28%"),
            TableColumn::new(if en { "Weight" } else { "Gewicht" }).with_width("14%"),
            TableColumn::new("Score").with_width("14%"),
            TableColumn::new(if en { "Meaning" } else { "Einordnung" }).with_width("44%"),
        ])
        .with_title(if en {
            "How the score is composed"
        } else {
            "Wie sich der Wert zusammensetzt"
        });
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
        if i18n.locale() == "en" {
            return metric.to_string();
        }
        match metric {
            "JS size" => "JS-Größe".to_string(),
            "CSS size" => "CSS-Größe".to_string(),
            "Page size" => "Seitengröße".to_string(),
            other => other.to_string(),
        }
    };

    for v in violations {
        table = table.add_row(vec![
            localize_metric(&v.metric),
            v.budget_label.clone(),
            v.actual_label.clone(),
            format!("+{:.0}%", v.exceeded_by_pct),
            v.severity.label().to_string(),
        ]);
    }

    builder = builder.add_component(table);
    builder
}
