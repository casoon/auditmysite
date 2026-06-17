use super::*;

pub(in crate::output::pdf) fn render_ux(
    mut builder: renderreport::engine::ReportBuilder,
    ux: &UxPresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let ux_title = i18n.t("section-ux");
    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }
    builder = builder
        .add_component(
            ScoreCard::new(&ux_title, ux.score)
                .with_description(i18n.t("label-heuristic-indicator"))
                .with_thresholds(80, 50),
        )
        .add_component(
            Label::new(format!(
                "ℹ {}: {}",
                i18n.t("pdf-ux-overview-title"),
                ux.interpretation
            ))
            .with_size("10.5pt")
            .with_color("#475569"),
        )
        .add_component(module_customer_context(
            i18n,
            "ux",
            ux.score,
            &ux.interpretation,
        ));

    // The struct carries canonical English; re-derive everything in the run language.
    let en = i18n.locale() == "en";

    // Dimension scores as KeyValueList
    let mut kv = KeyValueList::new().with_title(i18n.t("ux-dimensions"));
    for dim in &ux.dimensions {
        let name = crate::ux::ux_dimension_name(dim.kind, en);
        let summary = crate::ux::ux_dimension_summary(dim.kind, dim.score, en);
        kv = kv.add(name, format!("{}/100 — {}", dim.score, summary));
    }
    builder = builder.add_component(kv);

    // Issues as findings (top 3 only)
    for issue in ux.issues.iter().take(3) {
        let sev = map_severity(&match issue.severity.as_str() {
            "high" => crate::taxonomy::Severity::High,
            "medium" => crate::taxonomy::Severity::Medium,
            "low" => crate::taxonomy::Severity::Low,
            _ => crate::taxonomy::Severity::Medium,
        });
        let dimension = crate::ux::ux_dimension_name(issue.kind.dimension(), en);
        let (_problem, impact, recommendation) =
            crate::ux::ux_issue_text(issue.kind, &issue.values, en);
        let desc = format!("{} — {}", impact, recommendation);
        builder = builder.add_component(Finding::new(dimension, sev, &desc));
    }
    if ux.issues.len() > 3 {
        let more_note = i18n.t_args(
            "pdf-ux-more-issues",
            &[("count", (ux.issues.len() - 3).to_string())],
        );
        builder = builder.add_component(Callout::info(&more_note));
    }
    builder
}

pub(in crate::output::pdf) fn render_journey(
    mut builder: renderreport::engine::ReportBuilder,
    journey: &JourneyPresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let journey_title = i18n.t("section-journey");
    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }
    builder = builder
        .add_component(
            ScoreCard::new(&journey_title, journey.score)
                .with_description(i18n.t("label-heuristic-indicator"))
                .with_thresholds(80, 50),
        )
        .add_component(
            Label::new(format!(
                "ℹ {}: {}",
                i18n.t("pdf-journey-overview-title"),
                journey.interpretation
            ))
            .with_size("10.5pt")
            .with_color("#475569"),
        )
        .add_component(module_customer_context(
            i18n,
            "journey",
            journey.score,
            &journey.interpretation,
        ));

    // The struct carries canonical English; re-derive everything in the run language.
    let en = i18n.locale() == "en";

    // Page intent
    let mut kv = KeyValueList::new().with_title(i18n.t("journey-page-type-dimensions"));
    kv = kv.add(
        i18n.t("pdf-journey-detected-page-type"),
        &journey.page_intent,
    );
    for dim in &journey.dimensions {
        let name = crate::journey::journey_dimension_name(dim.kind, en);
        let summary = crate::journey::journey_dimension_summary(dim.kind, dim.score, en);
        kv = kv.add(
            format!("{} ({}%)", name, dim.weight_pct),
            format!("{}/100 — {}", dim.score, summary),
        );
    }
    builder = builder.add_component(kv);

    // Friction points as findings (top 3 only)
    for fp in journey.friction_points.iter().take(3) {
        let sev = map_severity(&match fp.severity.as_str() {
            "high" => crate::taxonomy::Severity::High,
            "medium" => crate::taxonomy::Severity::Medium,
            "low" => crate::taxonomy::Severity::Low,
            _ => crate::taxonomy::Severity::Medium,
        });
        let (problem, impact, recommendation) =
            crate::journey::journey_friction_text(fp.kind, &fp.values, en);
        let desc = format!("[{}] {} — {}", fp.step, impact, recommendation);
        builder = builder.add_component(Finding::new(&problem, sev, &desc));
    }
    if journey.friction_points.len() > 3 {
        let more_note = i18n.t_args(
            "pdf-journey-more-issues",
            &[("count", (journey.friction_points.len() - 3).to_string())],
        );
        builder = builder.add_component(Callout::info(&more_note));
    }
    builder
}
