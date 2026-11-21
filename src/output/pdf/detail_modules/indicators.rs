use super::*;

// ─── Tech Stack ─────────────────────────────────────────────────────────────

pub(in crate::output::pdf) fn render_tech_stack(
    mut builder: renderreport::engine::ReportBuilder,
    ts: &crate::tech_stack::TechStackAnalysis,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::tech_stack::Confidence;

    let ts_title = i18n.t("pdf-ts-section-title");
    let ts_takeaway = if ts.detected.is_empty() {
        i18n.t("output-ts-takeaway-none")
    } else {
        i18n.t_args(
            "output-ts-takeaway-count",
            &[("count", ts.detected.len() as i64)],
        )
    };
    builder = super::module_chapter_opener(builder, &ts_title, &ts_takeaway, is_first);
    if !ts.detected.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ts-technology")).with_width("28%"),
            TableColumn::new(i18n.t("pdf-serp-category")).with_width("22%"),
            TableColumn::new("Version").with_width("18%"),
            TableColumn::new(i18n.t("pdf-ts-confidence")).with_width("14%"),
            TableColumn::new("Detail").with_width("18%"),
        ])
        .with_title(i18n.t("pdf-ts-detected-title"));

        for tech in &ts.detected {
            let confidence = match tech.confidence {
                Confidence::High => i18n.t("pdf-ts-confidence-high"),
                Confidence::Medium => i18n.t("pdf-ts-confidence-medium"),
                Confidence::Low => i18n.t("pdf-ts-confidence-low"),
            };
            table = table.add_row(vec![
                tech.name.clone(),
                tech.category.label().to_string(),
                tech.version.clone().unwrap_or_else(|| "—".to_string()),
                confidence.to_string(),
                tech.signals.join(", "),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

pub(in crate::output::pdf) fn render_best_practices(
    mut builder: renderreport::engine::ReportBuilder,
    bp: &crate::best_practices::BestPracticesAnalysis,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let collected = bp.console_errors.collected;
    let checked = bp.vulnerable_libraries.checked;
    if bp.is_unmeasured() {
        let takeaway = i18n.t("output-bp-takeaway-not-measured");
        builder = super::module_chapter_opener(builder, "Best Practices", &takeaway, is_first);
        return builder.add_component(TextBlock::new(i18n.t("pdf-bp-not-measured")));
    }
    let bp_clean =
        bp.console_errors.error_count == 0 && !bp.vulnerable_libraries.has_vulnerabilities;
    let bp_takeaway = i18n.t(if bp_clean {
        "output-bp-takeaway-clean"
    } else {
        "output-bp-takeaway-issues"
    });
    builder = super::module_chapter_opener(builder, "Best Practices", &bp_takeaway, is_first);
    builder = builder.add_component(
        ScoreCard::new(super::module_score_caption(i18n), bp.score)
            .with_description(super::score_band_label(bp.score, i18n))
            .with_thresholds(75, 40),
    );
    // What "nothing detected" covers — a clean result is no general
    // all-clear.
    builder = builder.add_component(TextBlock::new(i18n.t_args(
        "pdf-bp-scope",
        &[("libs", crate::best_practices::CHECKED_LIBRARIES.join(", "))],
    )));
    if !collected {
        builder = builder.add_component(TextBlock::new(i18n.t("pdf-bp-console-not-collected")));
    }
    if !checked {
        builder = builder.add_component(TextBlock::new(i18n.t("pdf-bp-libs-not-checked")));
    }

    if bp.score >= 90
        && bp.console_errors.error_count == 0
        && !bp.vulnerable_libraries.has_vulnerabilities
        && collected
        && checked
    {
        return builder.add_component(TextBlock::new(i18n.t("pdf-bp-success")));
    }

    // Console errors
    if bp.console_errors.error_count > 0 {
        let title = i18n.t("pdf-bp-console-errors-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new("Level").with_width("15%"),
            TableColumn::new(i18n.t("pdf-bp-message-col")).with_width("85%"),
        ])
        .with_title(&title);
        for error in &bp.console_errors.errors {
            table = table.add_row(vec![error.level.clone(), error.message.clone()]);
        }
        builder = builder.add_component(table);
    }

    // Vulnerable libraries
    if bp.vulnerable_libraries.has_vulnerabilities {
        let title = i18n.t("pdf-bp-vuln-libs-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-bp-lib-col")).with_width("20%"),
            TableColumn::new("Version").with_width("15%"),
            TableColumn::new(i18n.t("pdf-bp-severity-col")).with_width("15%"),
            TableColumn::new(i18n.t("pdf-ph-issue")).with_width("35%"),
            TableColumn::new(i18n.t("pdf-bp-fix-col")).with_width("15%"),
        ])
        .with_title(&title);
        for lib in &bp.vulnerable_libraries.vulnerable {
            // A range not yet checked against its source is a lead, not an
            // established statement (#773).
            let unreviewed = lib
                .advisories
                .iter()
                .any(|a| a.review_status == crate::best_practices::ReviewStatus::Unreviewed);
            let description = if unreviewed {
                format!(
                    "{} {}",
                    lib.description,
                    i18n.t("pdf-bp-vuln-libs-unreviewed")
                )
            } else {
                lib.description.clone()
            };
            table = table.add_row(vec![
                lib.name.clone(),
                lib.version.clone(),
                super::severity_text(&lib.severity, i18n),
                description,
                lib.safe_version.clone(),
            ]);
        }
        builder = builder.add_component(table);
    } else if !bp.vulnerable_libraries.detected.is_empty() {
        builder = builder.add_component(TextBlock::new(i18n.t("pdf-bp-libs-up-to-date")));
    }

    // Duplicate library versions (#547)
    if !bp.vulnerable_libraries.duplicate_libraries.is_empty() {
        builder = builder
            .add_component(Section::new(i18n.t("pdf-bp-duplicate-libs-title")).with_level(3));
        let mut list = List::new();
        for dup in &bp.vulnerable_libraries.duplicate_libraries {
            list = list.add_item(i18n.t_args(
                "pdf-bp-duplicate-libs-item",
                &[
                    ("name", dup.name.clone()),
                    ("versions", dup.versions.join(", ")),
                ],
            ));
        }
        builder = builder.add_component(list);
    }

    builder
}
