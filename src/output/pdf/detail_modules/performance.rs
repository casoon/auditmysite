use super::*;

pub(in crate::output::pdf) fn render_performance(
    mut builder: renderreport::engine::ReportBuilder,
    perf: &PerformancePresentation,
    is_first: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let perf_section_title = i18n.t("pdf-perf-section-title");
    let perf_intro = KeyValueList::new()
        .add(
            i18n.t("section-user-experience"),
            i18n.t("pdf-perf-intro-user-experience"),
        )
        .add(
            i18n.t("section-technical-complexity"),
            i18n.t("pdf-perf-intro-technical-complexity"),
        );

    builder = super::module_chapter_opener(builder, &perf_section_title, is_first);

    builder = builder
        .add_component(
            ScoreCard::new(super::module_score_caption(i18n), perf.score)
                .with_description(super::score_band_label(perf.score, i18n))
                .with_thresholds(75, 40),
        )
        .add_component(perf_intro)
        .add_component(
            Label::new(format!(
                "{}: {}",
                i18n.t("pdf-perf-overview-title"),
                perf.interpretation
            ))
            .with_size("10.5pt")
            .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        )
        .add_component(module_customer_context(
            i18n,
            "performance",
            perf.score,
            &perf.interpretation,
        ));

    // ── Subsection 1: Lade-Erfahrung & Vitals ──────────────────────────
    builder = builder.add_component(Section::new(i18n.t("pdf-perf-sub-vitals")).with_level(3));
    builder = builder.add_component(
        Callout::info(i18n.t("perf-lab-data-body")).with_title(i18n.t("perf-lab-data-note")),
    );

    match (&perf.desktop, &perf.mobile) {
        (Some(desktop), Some(mobile)) => {
            // Score comparison strip
            let score_strip = vec![
                MetricStripItem::new("Desktop", desktop.score.to_string())
                    .with_status(score_status(desktop.score))
                    .with_accent(score_color(desktop.score)),
                MetricStripItem::new("Mobile", mobile.score.to_string())
                    .with_status(score_status(mobile.score))
                    .with_accent(score_color(mobile.score)),
            ];
            builder = builder.add_component(MetricStrip::new(score_strip).compact());

            // Desktop vitals
            if !desktop.vitals.is_empty() {
                builder =
                    builder.add_component(Section::new("Desktop — Core Web Vitals").with_level(4));
                let strip = desktop
                    .vitals
                    .iter()
                    .take(4)
                    .map(|(name, value, rating)| {
                        MetricStripItem::new(name, value)
                            .with_status(vital_status(rating))
                            .with_accent(vital_color(rating))
                    })
                    .collect();
                builder = builder.add_component(MetricStrip::new(strip).compact());
                let mut kv = KeyValueList::new();
                for (name, value, rating) in &desktop.vitals {
                    kv = kv.add(name, format!("{} — {}", value, rating));
                }
                builder = builder.add_component(kv);
            }

            // Mobile vitals
            if !mobile.vitals.is_empty() {
                builder =
                    builder.add_component(Section::new("Mobile — Core Web Vitals").with_level(4));
                let strip = mobile
                    .vitals
                    .iter()
                    .take(4)
                    .map(|(name, value, rating)| {
                        MetricStripItem::new(name, value)
                            .with_status(vital_status(rating))
                            .with_accent(vital_color(rating))
                    })
                    .collect();
                builder = builder.add_component(MetricStrip::new(strip).compact());
                let mut kv = KeyValueList::new();
                for (name, value, rating) in &mobile.vitals {
                    kv = kv.add(name, format!("{} — {}", value, rating));
                }
                builder = builder.add_component(kv);
            }
        }
        _ if !perf.vitals.is_empty() => {
            // Fallback: flat vitals (no desktop data)
            let strip = perf
                .vitals
                .iter()
                .take(4)
                .map(|(name, value, rating)| {
                    MetricStripItem::new(name, value)
                        .with_status(vital_status(rating))
                        .with_accent(vital_color(rating))
                })
                .collect();
            builder = builder.add_component(MetricStrip::new(strip).compact());
            let mut kv = KeyValueList::new().with_title("Core Web Vitals");
            for (name, value, rating) in &perf.vitals {
                kv = kv.add(name, format!("{} — {}", value, rating));
            }
            builder = builder.add_component(kv);
        }
        _ => {}
    }

    // Measurement Warnings
    if !perf.measurement_warnings.is_empty() {
        let title = i18n.t("perf-measurement-warnings-title");
        let mut warning_items: Vec<String> = perf
            .measurement_warnings
            .iter()
            .map(|key| match key.as_str() {
                "lcp_not_measured" => i18n.t("perf-warning-lcp-missing"),
                "tbt_zero_heavy_page" => i18n.t("perf-warning-tbt-zero"),
                "speed_index_fallback_to_lcp" => i18n.t("perf-warning-si-fallback"),
                "tti_fallback_to_lcp" => i18n.t("perf-warning-tti-fallback"),
                "inp_not_measured" => i18n.t("perf-warning-inp-missing"),
                _ => key.clone(),
            })
            .collect();
        if perf
            .measurement_warnings
            .iter()
            .any(|w| w == "lcp_not_measured")
        {
            if let Some(lh) = perf
                .throttled_profiles
                .iter()
                .find(|p| p.profile_name == "LhMobile")
            {
                let gap = perf.score.saturating_sub(lh.score);
                if gap >= 15 {
                    warning_items.push(i18n.t_args(
                        "perf-warning-lh-mobile-gap",
                        &[
                            ("desktop", perf.score.to_string()),
                            ("mobile", lh.score.to_string()),
                            ("gap", gap.to_string()),
                        ],
                    ));
                }
            }
        }
        let body = warning_items.join(" · ");
        builder = builder.add_component(Callout::warning(&body).with_title(&title));
    }

    // Throttled Network Performance table
    if !perf.throttled_profiles.is_empty() {
        let title = i18n.t("pdf-perf-throttled-title");
        let col_profile = i18n.t("pdf-perf-throttled-profile");
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_profile).with_width("28%"),
            TableColumn::new("LCP").with_width("18%"),
            TableColumn::new("TBT").with_width("18%"),
            TableColumn::new("CLS").with_width("18%"),
            TableColumn::new("Score").with_width("18%"),
        ])
        .with_title(&title);
        for entry in &perf.throttled_profiles {
            table = table.add_row(vec![
                entry.profile_name.clone(),
                entry.lcp.clone(),
                entry.tbt.clone(),
                entry.cls.clone(),
                entry.score.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // CLS Attribution table
    if !perf.cls_attribution.is_empty() {
        let title = i18n.t("pdf-perf-cls-title");
        let col_val = i18n.t("pdf-perf-cls-value");
        let col_time = i18n.t("pdf-perf-cls-time");
        let col_elem = "Element";
        let mut table = AuditTable::new(vec![
            TableColumn::new(&col_val).with_width("15%"),
            TableColumn::new(&col_time).with_width("20%"),
            TableColumn::new(col_elem).with_width("65%"),
        ])
        .with_title(&title);
        for (val, time, node) in &perf.cls_attribution {
            let display_node = if node.is_empty() {
                "—"
            } else {
                node.as_str()
            };
            table = table.add_row(vec![val.as_str(), time.as_str(), display_node]);
        }
        builder = builder.add_component(table);
    }

    // ── Subsection 2: Ressourcen & Datenmenge ──────────────────────────
    builder = builder.add_component(Section::new(i18n.t("pdf-perf-sub-resources")).with_level(3));

    // Indicator stats (excluding DOM-Knoten)
    let resource_metrics: Vec<&(String, String)> = perf
        .additional_metrics
        .iter()
        .filter(|(k, _)| k != "DOM-Knoten")
        .collect();
    if !resource_metrics.is_empty() {
        let mut metrics = KeyValueList::new().with_title(i18n.t("perf-technical-indicators"));
        for (k, v) in resource_metrics {
            metrics = metrics.add(k, v);
        }
        builder = builder.add_component(metrics);
    }

    // Third-Party Attribution
    if let Some(ref tp) = perf.third_party {
        if !tp.origins.is_empty() {
            let title = i18n.t("pdf-perf-tp-title");
            let col_origin = "Origin";
            let col_req = i18n.t("pdf-perf-tp-requests");
            let mut kv = KeyValueList::new().with_title(&title);
            kv = kv.add(
                i18n.t("pdf-perf-tp-total-origins"),
                tp.total_origins.to_string(),
            );
            kv = kv.add(
                i18n.t("pdf-perf-tp-total-transfer"),
                format!(
                    "{:.1} KB / {} {}",
                    tp.total_kb,
                    tp.total_requests,
                    i18n.t("pdf-perf-tp-requests")
                ),
            );
            if tp.is_significant {
                kv = kv.add(
                    i18n.t("pdf-perf-tp-impact"),
                    i18n.t("pdf-perf-tp-significant"),
                );
            }
            builder = builder.add_component(kv);

            let mut table = AuditTable::new(vec![
                TableColumn::new(col_origin).with_width("48%"),
                TableColumn::new(&col_req).with_width("16%"),
                TableColumn::new("KB").with_width("18%"),
                TableColumn::new(i18n.t("pdf-perf-tp-types")).with_width("18%"),
            ]);
            for row in &tp.origins {
                let origin = match (&row.provider, &row.category) {
                    (Some(provider), Some(category)) => {
                        format!("{} ({provider}, {category})", row.origin)
                    }
                    (Some(provider), None) => format!("{} ({provider})", row.origin),
                    _ => row.origin.clone(),
                };
                table = table.add_row(vec![
                    origin.as_str(),
                    &row.request_count.to_string(),
                    &format!("{:.1}", row.transfer_kb),
                    row.resource_kinds.as_str(),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    // Coverage (unused JS/CSS)
    if let Some(ref cov) = perf.coverage {
        let title = i18n.t("pdf-perf-cov-title");
        let mut kv = KeyValueList::new().with_title(&title);
        if let (Some(used_pct), Some(unused_kb)) = (cov.js_used_pct, cov.js_unused_kb) {
            let val = i18n.t_args(
                "pdf-perf-cov-js-val",
                &[
                    ("pct", format!("{:.1}", used_pct)),
                    ("unused", format!("{:.1}", unused_kb)),
                ],
            );
            kv = kv.add(i18n.t("pdf-perf-cov-js-used"), val);
        }
        if let Some(used_pct) = cov.css_used_pct {
            let rules_str = match (cov.css_used_rules, cov.css_total_rules) {
                (Some(used), Some(total)) => i18n.t_args(
                    "pdf-perf-cov-css-val",
                    &[
                        ("pct", format!("{:.1}", used_pct)),
                        ("used", used.to_string()),
                        ("total", total.to_string()),
                    ],
                ),
                _ => format!("{:.1}%", used_pct),
            };
            kv = kv.add(i18n.t("pdf-perf-cov-css-used"), rules_str);
        }
        builder = builder.add_component(kv);
    }

    // Minification
    if let Some(ref min) = perf.minification {
        let title = i18n.t("pdf-perf-min-title");
        let mut kv = KeyValueList::new().with_title(&title);
        kv = kv.add(i18n.t("pdf-perf-min-files"), min.total_count.to_string());
        kv = kv.add(
            i18n.t("pdf-perf-min-savings"),
            format!("{:.1} KB", min.total_savings_kb),
        );
        if min.legacy_count > 0 {
            kv = kv.add(
                "Legacy-/Polyfill-Skripte",
                format!("{} (~{:.1} KB)", min.legacy_count, min.legacy_wasted_kb),
            );
        }
        builder = builder.add_component(kv);

        if !min.top_assets.is_empty() {
            let col_url = "URL";
            let col_kind = i18n.t("pdf-perf-min-type");
            let col_save = i18n.t("pdf-perf-min-saving-col");
            let mut table = AuditTable::new(vec![
                TableColumn::new(col_url).with_width("62%"),
                TableColumn::new(&col_kind).with_width("16%"),
                TableColumn::new(&col_save).with_width("22%"),
            ]);
            for (url, kind, savings) in &min.top_assets {
                table = table.add_row(vec![url.as_str(), kind.as_str(), savings.as_str()]);
            }
            builder = builder.add_component(table);
        }
        if !min.legacy_assets.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("URL").with_width("62%"),
                TableColumn::new("Signatur").with_width("16%"),
                TableColumn::new("Bytes").with_width("22%"),
            ]);
            for (url, signature, wasted) in &min.legacy_assets {
                table = table.add_row(vec![url.as_str(), signature.as_str(), wasted.as_str()]);
            }
            builder = builder.add_component(table);
        }
    }

    // ── Subsection 3: Lade-Engpässe & Rendering ────────────────────────
    builder = builder.add_component(Section::new(i18n.t("pdf-perf-sub-bottlenecks")).with_level(3));

    // DOM Complexity
    let dom_metric = perf
        .additional_metrics
        .iter()
        .find(|(k, _)| k == "DOM-Knoten");
    if let Some((k, v)) = dom_metric {
        let metrics = KeyValueList::new()
            .with_title(i18n.t("pdf-perf-sub-bottlenecks"))
            .add(k, v);
        builder = builder.add_component(metrics);
    }

    // Render-blocking resources
    if perf.has_render_blocking {
        if !perf.render_blocking_metrics.is_empty() {
            let mut kv = KeyValueList::new().with_title(i18n.t("perf-render-blocking-analysis"));
            for (k, v) in &perf.render_blocking_metrics {
                kv = kv.add(k, v);
            }
            builder = builder.add_component(kv);
        }

        if !perf.render_blocking_suggestions.is_empty() {
            let mut suggestions = List::new().with_title(i18n.t("label-recommendations"));
            for s in &perf.render_blocking_suggestions {
                suggestions = suggestions.add_item(s);
            }
            builder = builder.add_component(suggestions);
        }
    }

    // Critical Request Chain
    if let Some(ref cc) = perf.critical_chain {
        let title = i18n.t("pdf-perf-cc-title");
        let mut kv = KeyValueList::new().with_title(&title);
        kv = kv.add(i18n.t("pdf-perf-cc-max-depth"), cc.max_depth.to_string());
        kv = kv.add(
            i18n.t("pdf-perf-cc-path"),
            format!("{} / {}", cc.critical_path_ms, cc.critical_path_kb),
        );
        kv = kv.add(
            i18n.t("pdf-perf-cc-total-requests"),
            cc.total_requests.to_string(),
        );
        builder = builder.add_component(kv);
    }

    // Non-composited Animations
    if let Some(ref anim) = perf.animations {
        let title = i18n.t("pdf-perf-anim-title");
        let mut kv = KeyValueList::new().with_title(&title);
        kv = kv.add(i18n.t("pdf-perf-anim-total"), anim.total_count.to_string());
        if !anim.affected_properties.is_empty() {
            kv = kv.add(
                i18n.t("pdf-perf-anim-properties"),
                anim.affected_properties.join(", "),
            );
        }
        builder = builder.add_component(kv);
    }

    // General improvement suggestions / recommendations
    if !perf.recommendations.is_empty() {
        let mut rec_list = List::new().with_title(i18n.t("label-improvement-suggestions"));
        for recommendation in &perf.recommendations {
            rec_list = rec_list.add_item(recommendation);
        }
        builder = builder.add_component(rec_list);
    }

    builder
}
