use super::*;

/// Renders the design-quality module (#528) — opt-in, evidence-based UX/
/// readability heuristics. Deliberately has no `ScoreCard`: this module never
/// contributes a score to the report (see module doc comment), so showing a
/// numeric card here would misleadingly suggest a measured, conformance-grade
/// value.
pub(in crate::output::pdf) fn render_design_quality(
    mut builder: renderreport::engine::ReportBuilder,
    dq: &DesignQualityPresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let title = pick(i18n, "Design-Qualität", "Design Quality");
    let is_clean = dq.findings.is_empty();
    let takeaway = if is_clean {
        pick(
            i18n,
            "Keine Design-Qualitäts-Hinweise auf dieser Seite gefunden.",
            "No design-quality findings on this page.",
        )
        .to_string()
    } else if en {
        format!(
            "{} warning(s), {} advisory(ies) on UX/readability found.",
            dq.warning_count, dq.advisory_count
        )
    } else {
        format!(
            "{} Warnung(en), {} Hinweis(e) zu UX/Lesbarkeit gefunden.",
            dq.warning_count, dq.advisory_count
        )
    };
    builder = super::module_chapter_opener(builder, title, &takeaway, is_first);

    builder = builder.add_component(
        Label::new(pick(
            i18n,
            "Heuristische, evidenzbasierte Hinweise zu UX/Lesbarkeit — kein Bestandteil des \
             Barrierefreiheitswerts oder des kombinierten technischen Werts, keine rechtliche Bewertung.",
            "Heuristic, evidence-based UX/readability findings — not part of the accessibility \
             or overall score, not a legal assessment.",
        ))
        .with_size("10.5pt")
        .with_color(crate::output::pdf::design::tokens::NEUTRAL),
    );

    if is_clean {
        return builder.add_component(clean_section_note(i18n));
    }

    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new(
                if en { "Warnings" } else { "Warnungen" },
                dq.warning_count.to_string(),
            )
            .with_status(if dq.warning_count > 0 { "warn" } else { "good" })
            .with_accent(if dq.warning_count > 0 {
                crate::output::pdf::design::tokens::WARN_DEEP
            } else {
                crate::output::pdf::design::tokens::SUCCESS
            }),
            MetricStripItem::new(
                if en { "Advisories" } else { "Hinweise" },
                dq.advisory_count.to_string(),
            )
            .with_accent(crate::output::pdf::design::tokens::INFO),
        ])
        .compact(),
    );

    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Level" } else { "Stufe" }).with_width("15%"),
        TableColumn::new(if en { "Location" } else { "Ort" }).with_width("30%"),
        TableColumn::new(if en { "Finding" } else { "Befund" }).with_width("55%"),
    ])
    .with_title(if en { "Findings" } else { "Befunde" });

    for f in &dq.findings {
        table = table.add_row(vec![
            f.level_label.clone(),
            f.selector.clone(),
            f.message.clone(),
        ]);
    }
    builder = builder.add_component(table);

    builder
}
