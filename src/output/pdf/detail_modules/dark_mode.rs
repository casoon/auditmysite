use super::*;

pub(in crate::output::pdf) fn render_dark_mode(
    mut builder: renderreport::engine::ReportBuilder,
    dm: &DarkModePresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let support_label = if dm.supported {
        i18n.t("pdf-dm-status-supported")
    } else {
        i18n.t("pdf-dm-status-not-supported")
    };
    let dm_title = i18n.t("section-dark-mode");
    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }
    builder = builder.add_component(ScoreCard::new(&dm_title, dm.score).with_thresholds(80, 50));

    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new("Status", &support_label)
                .with_status(if dm.supported { "good" } else { "warn" })
                .with_accent(if dm.supported { "#0f766e" } else { "#d97706" }),
            MetricStripItem::new(
                i18n.t("pdf-dm-methods"),
                dm.detection_methods.len().to_string(),
            )
            .with_accent("#2563eb"),
            MetricStripItem::new(
                i18n.t("pdf-dm-css-variables"),
                dm.css_custom_properties.to_string(),
            )
            .with_accent("#7c3aed"),
            MetricStripItem::new(
                i18n.t("pdf-dm-print"),
                if dm.print_stylesheet_detected {
                    i18n.t("pdf-dm-yes")
                } else {
                    i18n.t("pdf-dm-no")
                },
            )
            .with_status(if dm.print_stylesheet_detected {
                "good"
            } else {
                "warn"
            })
            .with_accent(if dm.print_stylesheet_detected {
                "#0f766e"
            } else {
                "#d97706"
            }),
            MetricStripItem::new(
                i18n.t("pdf-dm-forced-colors"),
                if dm.forced_colors_detected {
                    i18n.t("pdf-dm-yes")
                } else {
                    i18n.t("pdf-dm-no")
                },
            )
            .with_status(if dm.forced_colors_detected {
                "good"
            } else {
                "warn"
            })
            .with_accent(if dm.forced_colors_detected {
                "#0f766e"
            } else {
                "#d97706"
            }),
        ])
        .compact(),
    );

    let mut kv = KeyValueList::new().with_title(i18n.t("pdf-dm-overview-title"));
    kv = kv.add(i18n.t("pdf-dm-support"), &support_label);
    if !dm.detection_methods.is_empty() {
        kv = kv.add(
            i18n.t("pdf-dm-methods-impl"),
            dm.detection_methods.join(", "),
        );
    }
    kv = kv.add(
        "color-scheme CSS",
        if dm.color_scheme_css {
            i18n.t("pdf-dm-yes")
        } else {
            i18n.t("pdf-dm-no")
        },
    );
    if let Some(ref meta) = dm.meta_color_scheme {
        kv = kv.add("<meta name=\"color-scheme\">", meta.as_str());
    }
    if dm.css_custom_properties > 0 {
        kv = kv.add(
            i18n.t("pdf-dm-css-custom-props"),
            dm.css_custom_properties.to_string(),
        );
    }
    kv = kv.add(
        i18n.t("pdf-dm-print-stylesheet"),
        if dm.print_stylesheet_detected {
            i18n.t("pdf-dm-yes")
        } else {
            i18n.t("pdf-dm-no")
        },
    );
    if dm.print_stylesheet_detected {
        kv = kv
            .add(
                i18n.t("pdf-dm-print-chrome-hidden"),
                if dm.print_interactive_chrome_hidden {
                    i18n.t("pdf-dm-yes")
                } else {
                    i18n.t("pdf-dm-no")
                },
            )
            .add(
                i18n.t("pdf-dm-print-clipped-elements"),
                dm.print_clipped_elements.to_string(),
            );
    }
    kv = kv.add(
        i18n.t("pdf-dm-forced-colors"),
        if dm.forced_colors_detected {
            i18n.t("pdf-dm-yes")
        } else {
            i18n.t("pdf-dm-no")
        },
    );
    if dm.forced_colors_detected {
        kv = kv
            .add(
                i18n.t("pdf-dm-forced-colors-active"),
                if dm.forced_colors_active_matches {
                    i18n.t("pdf-dm-yes")
                } else {
                    i18n.t("pdf-dm-no")
                },
            )
            .add(
                i18n.t("pdf-dm-forced-color-adjust"),
                dm.forced_color_adjust_count.to_string(),
            )
            .add(
                i18n.t("pdf-dm-forced-focus-visible"),
                if dm.forced_colors_focus_visible {
                    i18n.t("pdf-dm-yes")
                } else {
                    i18n.t("pdf-dm-no")
                },
            );
    }
    if dm.supported {
        kv = kv.add(
            i18n.t("pdf-dm-contrast-violations"),
            dm.dark_contrast_violations.to_string(),
        );
        if dm.dark_only_violations > 0 {
            kv = kv.add(
                i18n.t("pdf-dm-only-issues"),
                i18n.t_args(
                    "pdf-dm-only-issues-val",
                    &[("count", dm.dark_only_violations.to_string())],
                ),
            );
        }
        if dm.light_only_violations > 0 {
            kv = kv.add(
                i18n.t("pdf-dm-resolved-issues"),
                i18n.t_args(
                    "pdf-dm-resolved-issues-val",
                    &[("count", dm.light_only_violations.to_string())],
                ),
            );
        }
    }
    builder = builder.add_component(kv);

    if !dm.vision_deficiency_modes.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-dm-vision-mode")),
            TableColumn::new(i18n.t("pdf-dm-vision-contrast")),
            TableColumn::new(i18n.t("pdf-dm-vision-new")),
            TableColumn::new(i18n.t("pdf-dm-vision-use-color")),
        ])
        .with_title(i18n.t("pdf-dm-vision-title"));

        for mode in &dm.vision_deficiency_modes {
            table = table.add_row(vec![
                mode.mode.clone(),
                mode.contrast_violations.to_string(),
                mode.new_contrast_violations.to_string(),
                mode.use_of_color_violations.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !dm.issues.is_empty() {
        for (severity, description) in &dm.issues {
            builder = builder.add_component(match severity.as_str() {
                "high" => Callout::warning(description).with_title(i18n.t("pdf-dm-issue-title")),
                _ => Callout::info(description).with_title(i18n.t("pdf-dm-note-title")),
            });
        }
    }

    builder
}
