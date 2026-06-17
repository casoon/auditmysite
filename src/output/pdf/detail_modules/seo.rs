use super::*;

pub(in crate::output::pdf) fn render_seo(
    mut builder: renderreport::engine::ReportBuilder,
    seo: &SeoPresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let indicator_note_seo = i18n.t("pdf-seo-indicator-note");
    let seo_section_title = i18n.t("section-seo-analysis");

    if !is_first {
        builder = builder.add_component(PageBreak::new());
    }

    builder = builder
        .add_component(
            ScoreCard::new(&seo_section_title, seo.score)
                .with_description(i18n.t("seo-score-card-description"))
                .with_thresholds(80, 50),
        )
        .add_component(
            Label::new(format!(
                "ℹ {}: {}",
                i18n.t("pdf-seo-indicator-title"),
                indicator_note_seo
            ))
            .with_size("10.5pt")
            .with_color("#475569"),
        )
        .add_component(
            Label::new(format!(
                "ℹ {}: {}",
                i18n.t("pdf-seo-overview-title"),
                seo.interpretation
            ))
            .with_size("10.5pt")
            .with_color("#475569"),
        )
        .add_component(module_customer_context(
            i18n,
            "seo",
            seo.score,
            &seo.interpretation,
        ));

    let mut seo_strip = Vec::new();
    if let Some((_, title)) = seo.meta_tags.iter().find(|(key, _)| key == "Titel") {
        seo_strip.push(MetricStripItem::new("Title", truncate(title, 42)).with_accent("#0f766e"));
    }
    if let Some(profile) = &seo.profile {
        seo_strip.push(
            MetricStripItem::new("Schema.org", profile.schema_count.to_string())
                .with_accent("#2563eb"),
        );
        seo_strip.push(
            MetricStripItem::new(i18n.t("pdf-seo-maturity"), &profile.maturity_level)
                .with_accent("#7c3aed"),
        );
    }
    if !seo_strip.is_empty() {
        builder = builder.add_component(MetricStrip::new(seo_strip).compact());
    }

    if !seo.meta_tags.is_empty() {
        let col_field = i18n.t("pdf-seo-field");
        let col_value = i18n.t("pdf-seo-value");
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_field).with_width("24%"),
            TableColumn::new(&col_value).with_width("76%"),
        ])
        .with_title(i18n.t("pdf-seo-meta-tags-title"));
        for (k, v) in &seo.meta_tags {
            table = table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder.add_component(table);
    }

    if !seo.meta_issues.is_empty() {
        let col_field = i18n.t("pdf-seo-field");
        let col_desc = i18n.t("pdf-seo-meta-description");
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_field),
            TableColumn::new(i18n.t("label-severity")),
            TableColumn::new(&col_desc),
        ])
        .with_title(i18n.t("pdf-seo-meta-issues-title"));
        for (field, sev, msg) in &seo.meta_issues {
            table = table.add_row(vec![field.as_str(), sev.label(), msg.as_str()]);
        }
        builder = builder.add_component(table);
    }

    if !seo.heading_summary.is_empty() || !seo.social_summary.is_empty() {
        let mut kv = KeyValueList::new();
        if !seo.heading_summary.is_empty() {
            kv = kv.add(i18n.t("pdf-seo-headings"), &seo.heading_summary);
        }
        if !seo.social_summary.is_empty() {
            kv = kv.add(i18n.t("pdf-seo-social-tags"), &seo.social_summary);
        }
        builder = builder.add_component(kv);
    }

    if !seo.tracking_summary.is_empty() {
        let mut tracking_table = AuditTable::new(vec![
            TableColumn::new("Signal").with_width("32%"),
            TableColumn::new("Status").with_width("68%"),
        ])
        .with_title(i18n.t("seo-tracking-services"));
        for (k, v) in &seo.tracking_summary {
            tracking_table = tracking_table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder
            .add_component(
                Callout::info(&seo.tracking_summary_text)
                    .with_title(i18n.t("label-classification")),
            )
            .add_component(tracking_table);
    }

    if !seo.technical_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("seo-kv-title"));
        for (k, v) in seo.technical_summary.iter().take(5) {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
        if seo.technical_summary.len() > 5 {
            let more_note = i18n.t_args(
                "pdf-seo-more-signals",
                &[("count", (seo.technical_summary.len() - 5).to_string())],
            );
            builder = builder.add_component(Callout::info(&more_note));
        }
    }

    if !seo.technical_issues.is_empty() {
        let col_issue = i18n.t("label-issue");
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_issue).with_width("75%"),
            TableColumn::new(i18n.t("label-severity")).with_width("25%"),
        ])
        .with_title(i18n.t("seo-technical-issues-title"));
        for (_, msg, sev) in &seo.technical_issues {
            table = table.add_row(vec![msg.as_str(), sev.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // SEO Content Profile
    if let Some(profile) = &seo.profile {
        builder = render_seo_profile(builder, profile, i18n);
    }

    // robots.txt
    if let Some(robots) = &seo.robots {
        builder = render_robots(builder, robots, i18n);
    }

    // SERP pass
    if let Some(serp) = &seo.serp {
        builder = render_serp(builder, serp, i18n);
    }

    // Page health
    if let Some(ph) = &seo.page_health {
        builder = render_page_health(builder, ph, i18n);
    }

    // ── Image Efficiency ──────────────────────────────────────────────
    if let Some(ref ie) = seo.image_efficiency {
        let title = i18n.t("pdf-seo-ie-title");
        let mut kv = KeyValueList::new().with_title(&title);
        kv = kv.add(i18n.t("pdf-seo-ie-total"), ie.total_images.to_string());
        kv = kv.add(
            i18n.t("pdf-seo-ie-modern"),
            format!("{:.1}% (WebP/AVIF/SVG)", ie.modern_format_pct),
        );
        if ie.legacy_count > 0 {
            kv = kv.add(
                i18n.t("pdf-seo-ie-legacy"),
                format!("{} (JPG/PNG/GIF)", ie.legacy_count),
            );
        }
        builder = builder.add_component(kv);

        if !ie.oversized.is_empty() {
            let title_tbl = i18n.t("pdf-seo-ie-oversized-title");
            let col_src = i18n.t("pdf-seo-ie-source");
            let col_nat = i18n.t("pdf-seo-ie-natural");
            let col_dis = i18n.t("pdf-seo-ie-displayed");
            let mut table = AuditTable::new(vec![
                TableColumn::new(&col_src).with_width("56%"),
                TableColumn::new(&col_nat).with_width("22%"),
                TableColumn::new(&col_dis).with_width("22%"),
            ])
            .with_title(&title_tbl);
            for row in &ie.oversized {
                table = table.add_row(vec![
                    row.src.as_str(),
                    row.natural.as_str(),
                    row.display.as_str(),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    // Strong SEO score → no significant findings; confirm it (#446 re-scope).
    if seo.score >= 80 {
        builder = builder.add_component(clean_section_note(i18n));
    }

    builder
}

pub(in crate::output::pdf) fn render_serp(
    mut builder: renderreport::engine::ReportBuilder,
    serp: &crate::output::report_model::SerpPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(Section::new(i18n.t("section-serp-analysis")).with_level(3));

    let summary = i18n.t_args(
        "pdf-serp-summary",
        &[
            ("total", serp.signals.len().to_string()),
            ("pass", serp.pass_count.to_string()),
            ("warning", serp.warning_count.to_string()),
            ("fail", serp.fail_count.to_string()),
        ],
    );
    let serp_readiness_title = i18n.t("seo-serp-readiness");
    builder = if serp.fail_count > 0 {
        builder.add_component(Callout::warning(&summary).with_title(&serp_readiness_title))
    } else if serp.warning_count > 0 {
        builder.add_component(Callout::info(&summary).with_title(&serp_readiness_title))
    } else {
        builder.add_component(Callout::success(&summary).with_title(&serp_readiness_title))
    };

    if !serp.signals.is_empty() {
        let col_category = i18n.t("pdf-serp-category");
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_category).with_width("22%"),
            TableColumn::new("Signal").with_width("28%"),
            TableColumn::new("Status").with_width("14%"),
            TableColumn::new("Detail").with_width("36%"),
        ])
        .with_title(i18n.t("seo-serp-signals"));
        for (cat, label, status, detail) in &serp.signals {
            table = table.add_row(vec![
                cat.as_str(),
                label.as_str(),
                status.as_str(),
                detail.as_str(),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !serp.rich_result_types.is_empty() {
        let text = i18n.t_args(
            "pdf-serp-rich-results-text",
            &[("types", serp.rich_result_types.join(", "))],
        );
        builder = builder.add_component(Callout::info(&text).with_title("Rich Results"));
    }

    builder
}

/// A neutral per-section line shown when a module produced no findings, so a
/// reader can distinguish "checked and clean" from "not checked" instead of the
/// section silently collapsing to nothing (#446).
fn clean_section_note(i18n: &I18n) -> Label {
    Label::new(format!("ℹ {}", i18n.t("pdf-section-clean")))
        .with_size("10.5pt")
        .with_color("#475569")
}

pub(in crate::output::pdf) fn render_page_health(
    mut builder: renderreport::engine::ReportBuilder,
    ph: &crate::output::report_model::PageHealthPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(Section::new(i18n.t("section-page-health")).with_level(3));

    // No significant health issues (no High/Critical) → state it explicitly,
    // even when minor notes or reference data (URL analysis) still follow, so
    // "checked and clean" reads differently from "not checked" (#446 re-scope).
    // Severity strings are canonical (low/medium/high/critical).
    let has_significant = ph
        .issues
        .iter()
        .any(|(_, _, sev)| matches!(sev.as_str(), "high" | "critical"));
    if !has_significant {
        builder = builder.add_component(clean_section_note(i18n));
    }

    // Issues table (if any)
    if !ph.issues.is_empty() {
        let col_issue = i18n.t("pdf-ph-issue");
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_issue).with_width("55%"),
            TableColumn::new(i18n.t("label-severity")).with_width("25%"),
        ])
        .with_title(i18n.t("seo-page-health-issues"));
        for (_, msg, sev) in &ph.issues {
            table = table.add_row(vec![msg.as_str(), sev.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // URL info KV
    if !ph.url_info.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("seo-page-health-url-analysis"));
        for (k, v) in &ph.url_info {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    if let Some((status, detail)) = &ph.html_validator {
        let validator_title = i18n.t("pdf-ph-w3c-title");
        let callout = match status.as_str() {
            "Ausgeführt" => Callout::info(detail).with_title(&validator_title),
            "Fehlgeschlagen" => Callout::warning(detail).with_title(&validator_title),
            _ => Callout::info(detail).with_title(&validator_title),
        };
        builder = builder.add_component(callout);
    }

    // www consolidation
    if let Some((www_label, non_www_label, is_ok)) = &ph.www_status {
        let icon = if *is_ok { "✓" } else { "✗" };
        builder = builder.add_component(
            Callout::info(format!(
                "www: {} | non-www: {} {}",
                www_label, non_www_label, icon
            ))
            .with_title(i18n.t("pdf-ph-www-title")),
        );
    }

    // HTML validation table
    if !ph.html_issues.is_empty() {
        let col_check = i18n.t("pdf-ph-check");
        let col_count = i18n.t("pdf-ph-count");
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_check).with_width("40%"),
            TableColumn::new(&col_count).with_width("15%"),
            TableColumn::new(i18n.t("label-severity")).with_width("20%"),
            TableColumn::new("Detail").with_width("25%"),
        ])
        .with_title(i18n.t("seo-page-html-validation"));
        for (check, count, sev, detail) in &ph.html_issues {
            table = table.add_row(vec![
                check.as_str(),
                &count.to_string(),
                sev.as_str(),
                detail.as_str(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_robots(
    mut builder: renderreport::engine::ReportBuilder,
    robots: &crate::output::report_model::RobotsPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(Section::new(i18n.t("section-robots-audit")).with_level(3));

    if let Some(ref err) = robots.error {
        return builder.add_component(
            Callout::warning(i18n.t_args("pdf-robots-error", &[("err", err.to_string())]))
                .with_title(i18n.t("pdf-robots-no-access")),
        );
    }

    // Summary callout — only warn for genuinely problematic configurations
    if robots.has_wildcard_disallow_all {
        builder = builder.add_component(
            Callout::warning(i18n.t("pdf-robots-block-all-body"))
                .with_title(i18n.t("pdf-robots-block-all-title")),
        );
    } else if robots.blocks_ai_citation {
        builder = builder.add_component(
            Callout::info(i18n.t("pdf-robots-limit-ai-body"))
                .with_title(i18n.t("pdf-robots-limit-ai-title")),
        );
    } else if !robots.blocked_ai_bots.is_empty() {
        // Training bots blocked, citation bots allowed — this is the citationFriendly default
        builder = builder.add_component(
            Callout::info(i18n.t_args(
                "pdf-robots-training-blocked-body",
                &[
                    ("policy", robots.inferred_policy.clone()),
                    ("bots", robots.blocked_ai_bots.join(", ")),
                ],
            ))
            .with_title(i18n.t("pdf-robots-training-blocked-title")),
        );
    }

    // Bot overview table
    if !robots.bot_rows.is_empty() {
        let col_category = i18n.t("pdf-serp-category");
        let col_allowed = i18n.t("pdf-robots-allowed");
        let col_blocked = i18n.t("pdf-robots-blocked");
        let mut table = AuditTable::new(vec![
            TableColumn::new("User-agent").with_width("28%"),
            TableColumn::new(&col_category).with_width("26%"),
            TableColumn::new(&col_allowed).with_width("13%"),
            TableColumn::new(&col_blocked).with_width("13%"),
            TableColumn::new("Status").with_width("20%"),
        ])
        .with_title(i18n.t("pdf-robots-crawler-rules"));

        for (ua, class, allows, disallows, fully_blocked) in &robots.bot_rows {
            let status = if *fully_blocked {
                i18n.t("pdf-robots-status-fully-blocked")
            } else if *disallows > 0 {
                i18n.t("pdf-robots-status-partially-blocked")
            } else {
                i18n.t("pdf-robots-allowed")
            };
            table = table.add_row(vec![
                ua.clone(),
                class.clone(),
                allows.to_string(),
                disallows.to_string(),
                status,
            ]);
        }

        builder = builder.add_component(table);
    }

    // Sitemaps
    if !robots.sitemaps.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("pdf-robots-sitemap-entries"));
        for (i, sitemap) in robots.sitemaps.iter().enumerate() {
            kv = kv.add(format!("Sitemap {}", i + 1), sitemap);
        }
        builder = builder.add_component(kv);
    }

    // Crawl delays
    if !robots.crawl_delays.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("pdf-robots-crawl-delay-title"));
        for (ua, delay) in &robots.crawl_delays {
            kv = kv.add(
                ua,
                i18n.t_args(
                    "pdf-robots-crawl-delay-value",
                    &[("delay", delay.to_string())],
                ),
            );
        }
        builder = builder.add_component(kv);
    }

    builder
}

pub(in crate::output::pdf) fn render_seo_profile(
    mut builder: renderreport::engine::ReportBuilder,
    profile: &SeoProfilePresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder =
        builder.add_component(Section::new(i18n.t("section-seo-content-profile")).with_level(3));

    let mut identity_table = AuditTable::new(vec![
        TableColumn::new(i18n.t("pdf-seo-profile-aspect")).with_width("24%"),
        TableColumn::new(i18n.t("pdf-seo-value")).with_width("76%"),
    ])
    .with_title(i18n.t("pdf-seo-profile-content-profile"));
    for (key, value) in &profile.identity_facts {
        identity_table = identity_table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut page_profile_table = AuditTable::new(vec![
        TableColumn::new(i18n.t("pdf-seo-profile-aspect")).with_width("24%"),
        TableColumn::new(i18n.t("pdf-seo-value")).with_width("76%"),
    ])
    .with_title(i18n.t("pdf-seo-profile-page-profile"));
    for (key, value) in &profile.page_profile_facts {
        page_profile_table = page_profile_table.add_row(vec![key.clone(), value.clone()]);
    }

    builder = builder
        .add_component(
            Callout::info(&profile.identity_summary).with_title(i18n.t("label-classification")),
        )
        .add_component(identity_table)
        .add_component(page_profile_table);

    let mut score_grid = renderreport::components::advanced::Grid::new(2);
    for (title, score, subtitle, accent) in [
        (
            i18n.t("pdf-seo-profile-content-depth"),
            profile.content_depth_score,
            score_quality_label(profile.content_depth_score),
            score_quality_color(profile.content_depth_score),
        ),
        (
            i18n.t("pdf-seo-profile-structure-quality"),
            profile.structural_richness_score,
            score_quality_label(profile.structural_richness_score),
            score_quality_color(profile.structural_richness_score),
        ),
        (
            i18n.t("pdf-seo-profile-media-balance"),
            profile.media_text_balance_score,
            score_quality_label(profile.media_text_balance_score),
            score_quality_color(profile.media_text_balance_score),
        ),
        (
            "Intent-Fit".to_string(),
            profile.intent_fit_score,
            score_quality_label(profile.intent_fit_score),
            score_quality_color(profile.intent_fit_score),
        ),
    ] {
        score_grid = score_grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": renderreport::components::MetricCard::new(title, format!("{} / 100", score))
                .with_subtitle(subtitle)
                .with_accent_color(accent)
                .to_data()
        }));
    }
    builder = builder.add_component(score_grid);

    // Content Identity
    let mut identity = KeyValueList::new().with_title(i18n.t("pdf-seo-profile-website-identity"));
    identity = identity.add("Website", &profile.site_name);
    identity = identity.add(
        i18n.t("pdf-seo-profile-content-type"),
        &profile.content_type,
    );
    identity = identity.add(i18n.t("pdf-seo-profile-language"), &profile.language);
    if !profile.category_hints.is_empty() {
        identity = identity.add(
            i18n.t("pdf-seo-profile-schema-types"),
            profile.category_hints.join(", "),
        );
    }
    builder = builder.add_component(identity);

    // Schema Inventory
    if !profile.schema_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-seo-profile-schema-type-col")),
            TableColumn::new(i18n.t("pdf-seo-profile-completeness")),
            TableColumn::new("Details"),
        ])
        .with_title(i18n.t_args(
            "pdf-seo-profile-structured-data-title",
            &[(
                "label",
                format!(
                    "{} {}",
                    profile.schema_count,
                    match (profile.schema_count == 1, i18n.locale() == "en") {
                        (true, true) => "schema",
                        (false, true) => "schemas",
                        (true, false) => "Schema",
                        (false, false) => "Schemas",
                    }
                ),
            )],
        ));
        for (typ, completeness, details) in &profile.schema_rows {
            table = table.add_row(vec![typ.as_str(), completeness.as_str(), details.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Strength Overview
    if !profile.signal_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-serp-category")),
            TableColumn::new(i18n.t("pdf-seo-profile-rating")),
            TableColumn::new(i18n.t("pdf-seo-profile-classification")),
        ])
        .with_title(i18n.t_args(
            "pdf-seo-profile-strength-title",
            &[("pct", profile.signal_overall_pct.to_string())],
        ));
        for (cat, score, rating) in &profile.signal_rows {
            table = table.add_row(vec![cat.as_str(), score.as_str(), rating.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Details per category
    for (cat_name, checks) in &profile.signal_details {
        let mut detail_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ph-check")),
            TableColumn::new("Status"),
            TableColumn::new("Detail"),
        ])
        .with_title(cat_name);
        for (label, passed, detail) in checks {
            let status = if *passed { "✓" } else { "✗" };
            detail_table = detail_table.add_row(vec![label.as_str(), status, detail.as_str()]);
        }
        builder = builder.add_component(detail_table);
    }

    // Maturity Rating
    let mut maturity = KeyValueList::new().with_title(i18n.t("pdf-seo-profile-maturity-title"));
    maturity = maturity.add("Level", &profile.maturity_level);
    maturity = maturity.add(
        i18n.t("pdf-seo-profile-rating"),
        &profile.maturity_description,
    );
    maturity = maturity.add(
        i18n.t("pdf-seo-profile-techniques"),
        i18n.t_args(
            "pdf-seo-profile-techniques-value",
            &[
                ("used", profile.maturity_techniques_used.to_string()),
                ("total", profile.maturity_techniques_total.to_string()),
            ],
        ),
    );
    builder = builder.add_component(maturity);

    builder
}
