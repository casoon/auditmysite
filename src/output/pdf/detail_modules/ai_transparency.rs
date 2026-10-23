use super::*;

/// Renders the ai-transparency module (EU AI Act Art. 50) — opt-in C2PA
/// image-provenance check. Deliberately has no `ScoreCard`: this module never
/// contributes a score to the report, and every finding is a manual-review
/// advisory, not a confirmed violation (see module doc comment).
pub(in crate::output::pdf) fn render_ai_transparency(
    mut builder: renderreport::engine::ReportBuilder,
    at: &AiTransparencyPresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let title = pick(i18n, "KI-Transparenz", "AI Transparency");
    let is_clean = at.findings.is_empty();
    let takeaway = if is_clean {
        pick(
            i18n,
            "Keine C2PA-KI-Herkunftsnachweise in den geprüften Bildern gefunden.",
            "No C2PA AI-provenance manifests found in the checked images.",
        )
        .to_string()
    } else if en {
        format!(
            "{} of {} checked image(s) carry a C2PA manifest asserting AI/algorithmic generation.",
            at.findings.len(),
            at.images_checked
        )
    } else {
        format!(
            "{} von {} geprüften Bild(ern) tragen ein C2PA-Manifest mit KI-/algorithmischem Erzeugungs-Nachweis.",
            at.findings.len(),
            at.images_checked
        )
    };
    builder = super::module_chapter_opener(builder, title, &takeaway, is_first);

    builder = builder.add_component(
        Label::new(pick(
            i18n,
            "Prüft eingebettete C2PA-\"Content Credentials\"-Manifeste auf Bildern — kein \
             Bestandteil des Barrierefreiheitswerts oder des kombinierten technischen Werts, keine rechtliche Bewertung. Jeder \
             Fund ist ein Hinweis zur manuellen Prüfung, ob für dieses Bild eine für Menschen \
             wahrnehmbare Kennzeichnung existiert (EU-AI-Act, Art. 50).",
            "Checks embedded C2PA \"Content Credentials\" manifests on images — not part of the \
             accessibility or overall score, not a legal assessment. Every finding is a \
             manual-review prompt to verify a human-visible disclosure exists for that image \
             (EU AI Act, Art. 50).",
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
                if en {
                    "Images checked"
                } else {
                    "Geprüfte Bilder"
                },
                at.images_checked.to_string(),
            )
            .with_accent(crate::output::pdf::design::tokens::NEUTRAL),
            MetricStripItem::new(
                if en {
                    "AI provenance found"
                } else {
                    "KI-Herkunft gefunden"
                },
                at.findings.len().to_string(),
            )
            .with_status("warn")
            .with_accent(crate::output::pdf::design::tokens::WARN_DEEP),
        ])
        .compact(),
    );

    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Image" } else { "Bild" }).with_width("30%"),
        TableColumn::new(if en { "Provenance" } else { "Herkunft" }).with_width("25%"),
        TableColumn::new(if en { "Validation" } else { "Validierung" }).with_width("15%"),
        TableColumn::new(if en { "Finding" } else { "Befund" }).with_width("30%"),
    ])
    .with_title(if en { "Findings" } else { "Befunde" });

    for f in &at.findings {
        let location = match &f.generator {
            Some(g) => format!("{} ({g})", f.image_url),
            None => f.image_url.clone(),
        };
        table = table.add_row(vec![
            location,
            f.provenance_label.clone(),
            f.validation_label.clone(),
            f.message.clone(),
        ]);
    }
    builder = builder.add_component(table);

    builder
}
