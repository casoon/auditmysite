use super::*;

/// Human-readable label for a throttle profile shown in the throttled-network
/// table. `ThrottledPerfEntry::profile_name` is `format!("{:?}", ThrottleProfile)`
/// (see `output::builder::single::module_details`) — "Slow3G"/"Fast3G" already
/// read fine, but "LhMobile" is an internal preset name, not something a report
/// reader recognizes. Presentation-only: the underlying enum/JSON value is
/// untouched.
fn display_profile_label(profile_name: &str) -> &str {
    match profile_name {
        "LhMobile" => "Lighthouse Mobile",
        other => other,
    }
}

/// Finds the throttled-profile row with the worst (highest) LCP, parsed back
/// out of its formatted `"{ms} ms"` display string. Used to cross-reference
/// the overview prose's worst-case LCP claim to the actual table row it comes
/// from, without hardcoding which profile that is (issue #polish).
fn worst_throttled_lcp(profiles: &[ThrottledPerfEntry]) -> Option<(&str, f64)> {
    profiles
        .iter()
        .filter_map(|p| {
            p.lcp
                .strip_suffix(" ms")
                .and_then(|ms| ms.parse::<f64>().ok())
                .map(|ms| (p.profile_name.as_str(), ms))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Desktop/Mobile score comparison as two circular gauges side by side
/// (PageSpeed-style) instead of a flat metric strip.
fn score_gauge_grid(desktop_score: u32, mobile_score: u32) -> Grid {
    let mut desktop_gauge = Gauge::new("Desktop", desktop_score as f64);
    desktop_gauge.thresholds = score_gauge_thresholds();
    let mut mobile_gauge = Gauge::new("Mobile", mobile_score as f64);
    mobile_gauge.thresholds = score_gauge_thresholds();
    Grid::new(2)
        .add_item(serde_json::json!({"type": "gauge", "data": desktop_gauge.to_data()}))
        .add_item(serde_json::json!({"type": "gauge", "data": mobile_gauge.to_data()}))
}

/// Table for the vitals beyond the first 4 already shown in the compact
/// strip (TBT, TTI, Speed Index) — was a plain `KeyValueList` repeating
/// *all* vitals (including the first 4 a second time) as bare
/// "1100ms — good" text, with the rating printed as an untranslated
/// English token even in German reports.
fn additional_vitals_table(vitals: &[(String, String, String)], i18n: &I18n) -> Option<AuditTable> {
    let extra: Vec<&(String, String, String)> = vitals.iter().skip(4).collect();
    if extra.is_empty() {
        return None;
    }
    let en = is_english(i18n);
    let mut table = AuditTable::new(vec![
        TableColumn::new(if en { "Metric" } else { "Metrik" }).with_width("34%"),
        TableColumn::new(if en { "Value" } else { "Wert" }).with_width("33%"),
        TableColumn::new("Status").with_width("33%"),
    ]);
    for (name, value, rating) in extra {
        table = table.add_row(vec![
            name.clone(),
            value.clone(),
            vital_rating_label(rating, en).to_string(),
        ]);
    }
    Some(table)
}

fn localized_decimal(value: f64, en: bool) -> String {
    let value = format!("{value:.1}");
    if en {
        value
    } else {
        value.replace('.', ",")
    }
}

fn resource_focus_callout(perf: &PerformancePresentation, i18n: &I18n) -> Option<Callout> {
    let min = perf.minification.as_ref()?;
    if min.total_count == 0 || min.total_savings_kb <= 0.0 {
        return None;
    }
    let body = i18n.t_args(
        "pdf-perf-resource-focus",
        &[
            ("count", min.total_count.to_string()),
            (
                "savings",
                localized_decimal(min.total_savings_kb, is_english(i18n)),
            ),
        ],
    );
    let callout = if min.total_savings_kb >= 100.0 {
        Callout::warning(body)
    } else {
        Callout::info(body)
    };
    Some(callout.with_title(i18n.t("pdf-perf-resource-focus-title")))
}

fn coverage_callout(cov: &CoveragePresentation, i18n: &I18n) -> Option<Callout> {
    let unused_kb = cov.js_unused_kb?;
    if unused_kb <= 1.0 {
        return Some(
            Callout::success(i18n.t("pdf-perf-coverage-clean"))
                .with_title(i18n.t("pdf-perf-coverage-clean-title")),
        );
    }
    Some(
        Callout::warning(i18n.t_args(
            "pdf-perf-coverage-focus",
            &[("unused", localized_decimal(unused_kb, is_english(i18n)))],
        ))
        .with_title(i18n.t("pdf-perf-coverage-focus-title")),
    )
}

fn bottleneck_callout(perf: &PerformancePresentation, i18n: &I18n) -> Option<Callout> {
    let rating = if perf
        .resource_ratings
        .iter()
        .any(|(_, _, rating, _)| rating == "poor")
    {
        "poor"
    } else if perf
        .resource_ratings
        .iter()
        .any(|(_, _, rating, _)| rating == "needs-improvement")
    {
        "needs-improvement"
    } else {
        return None;
    };
    let metrics = perf
        .resource_ratings
        .iter()
        .filter(|(_, _, metric_rating, _)| metric_rating == rating)
        .map(|(name, value, _, target)| format!("{name} {value} ({target})"))
        .collect::<Vec<_>>()
        .join("; ");
    let has_dom_issue = perf
        .resource_ratings
        .iter()
        .any(|(name, _, metric_rating, _)| {
            metric_rating != "good" && (name == "DOM-Knoten" || name == "DOM nodes")
        });
    let action = i18n.t(if has_dom_issue {
        "pdf-perf-bottleneck-dom-action"
    } else {
        "pdf-perf-bottleneck-load-action"
    });
    let (title_key, body_key) = if rating == "poor" {
        (
            "pdf-perf-bottleneck-critical-title",
            "pdf-perf-bottleneck-critical",
        )
    } else {
        (
            "pdf-perf-bottleneck-warning-title",
            "pdf-perf-bottleneck-warning",
        )
    };
    Some(
        Callout::warning(i18n.t_args(body_key, &[("metrics", metrics), ("action", action)]))
            .with_title(i18n.t(title_key)),
    )
}

fn render_minification(
    mut builder: renderreport::engine::ReportBuilder,
    min: &MinificationPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let mut kv = KeyValueList::new().with_title(i18n.t("pdf-perf-min-title"));
    kv = kv.add(i18n.t("pdf-perf-min-files"), min.total_count.to_string());
    kv = kv.add(
        i18n.t("pdf-perf-min-savings"),
        format!(
            "{} KB",
            localized_decimal(min.total_savings_kb, is_english(i18n))
        ),
    );
    if min.legacy_count > 0 {
        kv = kv.add(
            "Legacy-/Polyfill-Skripte",
            format!("{} (~{:.1} KB)", min.legacy_count, min.legacy_wasted_kb),
        );
    }
    builder = builder.add_component(kv);

    if !min.top_assets.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("URL").with_width("62%"),
            TableColumn::new(i18n.t("pdf-perf-min-type")).with_width("16%"),
            TableColumn::new(i18n.t("pdf-perf-min-saving-col")).with_width("22%"),
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
    builder
}

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

    let perf_takeaway = super::first_sentence(&perf.interpretation);
    builder = super::module_chapter_opener(builder, &perf_section_title, &perf_takeaway, is_first);

    // If the interpretation prose names a worst-case throttled LCP (see
    // `audit::performance_interpretation::LATE_THROTTLED_LCP_THRESHOLD_MS`),
    // cross-reference which profile/table row that number actually comes
    // from — the throttled table appears a page later with no link back
    // otherwise.
    let mut perf_overview_text = perf.interpretation.clone();
    if let Some((profile_name, ms)) = worst_throttled_lcp(&perf.throttled_profiles) {
        if ms > crate::audit::performance_interpretation::LATE_THROTTLED_LCP_THRESHOLD_MS {
            perf_overview_text.push(' ');
            perf_overview_text.push_str(&i18n.t_args(
                "perf-worst-case-xref",
                &[("profile", display_profile_label(profile_name).to_string())],
            ));
        }
    }

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
                perf_overview_text
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
            // Score comparison as circular gauges (PageSpeed-style) — a
            // flat "Desktop 85 · Mobile 67" strip undersold the headline
            // number this section leads with.
            builder = builder.add_component(
                Label::new(if is_english(i18n) {
                    "Performance score per viewport: 0–100, higher is better."
                } else {
                    "Performance-Score je Ansicht: 0–100, höher ist besser."
                })
                .with_size("9pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
            );
            builder = builder.add_component(score_gauge_grid(desktop.score, mobile.score));

            builder = builder.add_component(
                Label::new(if is_english(i18n) {
                    "LCP: largest visible content (good ≤ 2.5 s); FCP: first visible content (good ≤ 1.8 s); CLS: layout stability (good ≤ 0.1); TTFB: server response (good ≤ 0.8 s). Lower is better for time and shift values."
                } else {
                    "LCP: größter sichtbarer Inhalt (gut ≤ 2,5 s); FCP: erster sichtbarer Inhalt (gut ≤ 1,8 s); CLS: Layoutstabilität (gut ≤ 0,1); TTFB: Serverantwort (gut ≤ 0,8 s). Bei Zeit- und Verschiebungswerten ist niedriger besser."
                })
                .with_size("9pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
            );

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
                if let Some(table) = additional_vitals_table(&desktop.vitals, i18n) {
                    builder = builder.add_component(table);
                }
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
                if let Some(table) = additional_vitals_table(&mobile.vitals, i18n) {
                    builder = builder.add_component(table);
                }
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
            if let Some(table) = additional_vitals_table(&perf.vitals, i18n) {
                builder = builder.add_component(table);
            }
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
            TableColumn::new(&col_profile).with_width("20%"),
            TableColumn::new("LCP").with_width("14%"),
            TableColumn::new("TBT").with_width("13%"),
            TableColumn::new("CLS").with_width("13%"),
            TableColumn::new("Score / 100").with_width("13%"),
            TableColumn::new("Status").with_width("27%"),
        ])
        .with_title(&title);
        for entry in &perf.throttled_profiles {
            table = table.add_row(vec![
                display_profile_label(&entry.profile_name).to_string(),
                entry.lcp.clone(),
                entry.tbt.clone(),
                entry.cls.clone(),
                entry.score.to_string(),
                super::score_band_label(entry.score, i18n).to_string(),
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

    if let Some(callout) = resource_focus_callout(perf, i18n) {
        builder = builder.add_component(callout);
    }
    if let Some(min) = perf.minification.as_ref() {
        builder = render_minification(builder, min, i18n);
    }

    // Indicator stats (excluding DOM-Knoten)
    let resource_metrics: Vec<&(String, String)> = perf
        .additional_metrics
        .iter()
        .filter(|(k, _)| k != "DOM-Knoten")
        .collect();
    if !resource_metrics.is_empty() {
        let has_heap = resource_metrics.iter().any(|(key, _)| key == "JS Heap");
        let has_carbon = resource_metrics
            .iter()
            .any(|(key, _)| key == "CO2e pro View" || key == "CO2e per view");
        let mut metrics = KeyValueList::new().with_title(i18n.t("perf-technical-indicators"));
        for (k, v) in resource_metrics {
            metrics = metrics.add(k, v);
        }
        let note_key = match (has_heap, has_carbon) {
            (true, true) => "pdf-perf-resource-note",
            (true, false) => "pdf-perf-resource-note-heap",
            (false, true) => "pdf-perf-resource-note-carbon",
            (false, false) => "pdf-perf-resource-note",
        };
        builder = builder.add_component(metrics).add_component(
            Label::new(i18n.t(note_key))
                .with_size("9pt")
                .with_color(crate::output::pdf::design::tokens::NEUTRAL),
        );
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
                    ("pct", localized_decimal(used_pct, is_english(i18n))),
                    ("unused", localized_decimal(unused_kb, is_english(i18n))),
                ],
            );
            kv = kv.add(i18n.t("pdf-perf-cov-js-used"), val);
        }
        if let Some(used_pct) = cov.css_used_pct {
            let rules_str = match (cov.css_used_rules, cov.css_total_rules) {
                (Some(used), Some(total)) => i18n.t_args(
                    "pdf-perf-cov-css-val",
                    &[
                        ("pct", localized_decimal(used_pct, is_english(i18n))),
                        ("used", used.to_string()),
                        ("total", total.to_string()),
                    ],
                ),
                _ => format!("{}%", localized_decimal(used_pct, is_english(i18n))),
            };
            kv = kv.add(i18n.t("pdf-perf-cov-css-used"), rules_str);
        }
        builder = builder.add_component(kv);
        if let Some(callout) = coverage_callout(cov, i18n) {
            builder = builder.add_component(callout);
        }
    }

    // ── Subsection 3: Lade-Engpässe & Rendering ────────────────────────
    builder = builder.add_component(Section::new(i18n.t("pdf-perf-sub-bottlenecks")).with_level(3));

    if let Some(callout) = bottleneck_callout(perf, i18n) {
        builder = builder.add_component(callout);
    }

    // DOM/load-time metrics with an established best-practice threshold —
    // rated the same "good"/"needs-improvement"/"poor" way as the Core Web
    // Vitals strip above, instead of sitting as plain, unrated text
    // indistinguishable from benign values (#perf-resource-ratings).
    if !perf.resource_ratings.is_empty() {
        let strip = perf
            .resource_ratings
            .iter()
            .map(|(name, value, rating, target)| {
                MetricStripItem::new(name, value)
                    .with_unit(target)
                    .with_status(vital_status(rating))
                    .with_accent(vital_color(rating))
            })
            .collect();
        builder = builder.add_component(MetricStrip::new(strip).compact());
    }

    if !perf.recommendations.is_empty() {
        let mut rec_list = List::new().with_title(i18n.t("pdf-perf-priority-actions"));
        for recommendation in &perf.recommendations {
            rec_list = rec_list.add_item(recommendation);
        }
        builder = builder.add_component(rec_list);
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

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_profile_label_relabels_only_lh_mobile() {
        assert_eq!(display_profile_label("LhMobile"), "Lighthouse Mobile");
        assert_eq!(display_profile_label("Slow3G"), "Slow3G");
        assert_eq!(display_profile_label("Fast3G"), "Fast3G");
        assert_eq!(display_profile_label("Unthrottled"), "Unthrottled");
    }

    fn entry(profile_name: &str, lcp: &str) -> ThrottledPerfEntry {
        ThrottledPerfEntry {
            profile_name: profile_name.to_string(),
            lcp: lcp.to_string(),
            tbt: "0 ms".to_string(),
            cls: "0.000".to_string(),
            score: 50,
        }
    }

    #[test]
    fn worst_throttled_lcp_picks_the_highest_value_regardless_of_order() {
        let profiles = vec![
            entry("Fast3G", "1224 ms"),
            entry("Slow3G", "6000 ms"),
            entry("LhMobile", "1500 ms"),
        ];
        let (profile, ms) = worst_throttled_lcp(&profiles).expect("worst entry");
        assert_eq!(profile, "Slow3G");
        assert_eq!(ms, 6000.0);
    }

    #[test]
    fn worst_throttled_lcp_skips_unparseable_rows() {
        let profiles = vec![entry("Slow3G", "\u{2014}"), entry("Fast3G", "1224 ms")];
        let (profile, ms) = worst_throttled_lcp(&profiles).expect("worst entry");
        assert_eq!(profile, "Fast3G");
        assert_eq!(ms, 1224.0);
    }

    #[test]
    fn worst_throttled_lcp_none_when_empty() {
        assert!(worst_throttled_lcp(&[]).is_none());
    }

    #[test]
    fn localized_decimal_uses_report_locale_separator() {
        assert_eq!(localized_decimal(495.24, false), "495,2");
        assert_eq!(localized_decimal(495.24, true), "495.2");
    }
}
