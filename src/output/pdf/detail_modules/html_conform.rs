use super::*;

pub(in crate::output::pdf) fn render_html_conform(
    mut builder: renderreport::engine::ReportBuilder,
    hc: &HtmlConformPresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let title = i18n.t("section-html-conform");
    let takeaway = if hc.checked {
        first_sentence(&hc.interpretation)
    } else {
        i18n.t("pdf-html-conform-not-checked")
    };
    builder = super::module_chapter_opener(builder, &title, &takeaway, is_first);

    if !hc.checked {
        return builder.add_component(Label::new(i18n.t("pdf-html-conform-not-checked")));
    }

    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), hc.score)
                .with_description(super::score_band_label(hc.score, i18n))
                .with_thresholds(75, 40),
        )
        .add_component(
            Label::new(i18n.t("pdf-html-conform-dedup-note"))
                .with_size("10.5pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(module_customer_context(
            i18n,
            "html_conform",
            hc.score,
            &hc.interpretation,
        ));

    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new(
                i18n.t("pdf-html-conform-defects"),
                hc.distinct_defect_count.to_string(),
            ),
            MetricStripItem::new(
                i18n.t("pdf-html-conform-errors"),
                hc.error_count.to_string(),
            )
            .with_accent("#dc2626"),
            MetricStripItem::new(
                i18n.t("pdf-html-conform-warnings"),
                hc.warning_count.to_string(),
            )
            .with_accent("#d97706"),
            MetricStripItem::new(i18n.t("pdf-html-conform-info"), hc.info_count.to_string())
                .with_accent("#2563eb"),
        ])
        .compact(),
    );

    if hc.error_count == 0 {
        builder = builder.add_component(clean_section_note(i18n));
    }

    if !hc.findings.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-html-conform-rule")),
            TableColumn::new(i18n.t("pdf-html-conform-severity")),
            TableColumn::new(i18n.t("pdf-html-conform-message")),
            TableColumn::new(i18n.t("pdf-html-conform-location")),
            TableColumn::new(i18n.t("pdf-html-conform-occurrences")),
        ])
        .with_title(&title);
        for (rule_id, severity, message, location, occurrences) in &hc.findings {
            let occurrences = occurrences.to_string();
            table = table.add_row(vec![
                rule_id.as_str(),
                severity.as_str(),
                message.as_str(),
                location.as_str(),
                occurrences.as_str(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}
