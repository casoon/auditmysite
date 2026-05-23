//! Module detail renderers (performance, SEO, security, mobile, dark mode, AI visibility).

use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, List, MetricStrip, MetricStripItem, PageBreak,
};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, Finding, ScoreCard, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;

use super::helpers::{map_severity, score_quality_color, score_quality_label};

pub(super) fn render_budget_violations(
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

    for v in violations {
        table = table.add_row(vec![
            v.metric.clone(),
            v.budget_label.clone(),
            v.actual_label.clone(),
            format!("+{:.0}%", v.exceeded_by_pct),
            v.severity.label().to_string(),
        ]);
    }

    builder = builder.add_component(table);
    builder
}

pub(super) fn render_performance(
    mut builder: renderreport::engine::ReportBuilder,
    perf: &PerformancePresentation,
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
    builder = builder
        .add_component(Section::new(&perf_section_title).with_level(2))
        .add_component(perf_intro)
        .add_component(
            Callout::info(&perf.interpretation).with_title(i18n.t("pdf-perf-overview-title")),
        )
        .add_component(
            ScoreCard::new(i18n.t("perf-score-card"), perf.score)
                .with_description(format!("Grade: {}", perf.grade))
                .with_thresholds(75, 50),
        );

    // ── User-perceived Performance (Core Web Vitals) ─────────────────
    builder = builder.add_component(Section::new(i18n.t("section-user-experience")).with_level(3));
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

    // ── Technical Complexity ─────────────────────────────────────────
    if !perf.additional_metrics.is_empty() || perf.has_render_blocking {
        builder = builder
            .add_component(Section::new(i18n.t("section-technical-complexity")).with_level(3));

        if !perf.additional_metrics.is_empty() {
            let mut metrics = KeyValueList::new().with_title(i18n.t("perf-technical-indicators"));
            for (k, v) in &perf.additional_metrics {
                metrics = metrics.add(k, v);
            }
            builder = builder.add_component(metrics);
        }

        if perf.has_render_blocking {
            if !perf.render_blocking_metrics.is_empty() {
                let mut kv =
                    KeyValueList::new().with_title(i18n.t("perf-render-blocking-analysis"));
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
    }

    // ── Improvement suggestions (across both layers) ─────────────────
    if !perf.recommendations.is_empty() {
        let mut rec_list = List::new().with_title(i18n.t("label-improvement-suggestions"));
        for recommendation in &perf.recommendations {
            rec_list = rec_list.add_item(recommendation);
        }
        builder = builder.add_component(rec_list);
    }

    // ── Throttled Network Performance ────────────────────────────────
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

    // ── CLS Attribution ──────────────────────────────────────────────
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

    // ── Third-Party Attribution ───────────────────────────────────────
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
                table = table.add_row(vec![
                    row.origin.as_str(),
                    &row.request_count.to_string(),
                    &format!("{:.1}", row.transfer_kb),
                    row.resource_kinds.as_str(),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    // ── Critical Request Chain ────────────────────────────────────────
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

    // ── Minification ─────────────────────────────────────────────────
    if let Some(ref min) = perf.minification {
        let title = i18n.t("pdf-perf-min-title");
        let mut kv = KeyValueList::new().with_title(&title);
        kv = kv.add(i18n.t("pdf-perf-min-files"), min.total_count.to_string());
        kv = kv.add(
            i18n.t("pdf-perf-min-savings"),
            format!("{:.1} KB", min.total_savings_kb),
        );
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
    }

    // ── Coverage (unused JS/CSS) ──────────────────────────────────────
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

    // ── Non-composited Animations ─────────────────────────────────────
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

pub(super) fn render_seo(
    mut builder: renderreport::engine::ReportBuilder,
    seo: &SeoPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let indicator_note_seo = i18n.t("pdf-seo-indicator-note");
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-seo-analysis")).with_level(2))
        .add_component(
            Callout::info(&indicator_note_seo).with_title(i18n.t("pdf-seo-indicator-title")),
        )
        .add_component(
            Callout::info(&seo.interpretation).with_title(i18n.t("pdf-seo-overview-title")),
        )
        .add_component(
            ScoreCard::new(i18n.t("seo-score-card"), seo.score)
                .with_description(i18n.t("seo-score-card-description"))
                .with_thresholds(80, 50),
        );

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

    builder
}

pub(super) fn render_serp(
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

pub(super) fn render_page_health(
    mut builder: renderreport::engine::ReportBuilder,
    ph: &crate::output::report_model::PageHealthPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(Section::new(i18n.t("section-page-health")).with_level(3));

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

pub(super) fn render_seo_profile(
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
            &[("count", profile.schema_count.to_string())],
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

pub(super) fn render_security(
    mut builder: renderreport::engine::ReportBuilder,
    sec: &SecurityPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-security")).with_level(2))
        .add_component(TextBlock::new(&sec.interpretation))
        .add_component(
            ScoreCard::new(i18n.t("security-score-card"), sec.score)
                .with_description(format!("Grade: {}", sec.grade))
                .with_thresholds(70, 50),
        );

    let header_count = sec
        .headers
        .iter()
        .filter(|(_, status, _)| status.to_lowercase().contains("vorhanden") || status == "✓")
        .count();
    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new("Header", format!("{}/9", header_count)).with_accent("#0f766e"),
            MetricStripItem::new(
                "HTTPS",
                if sec
                    .ssl_info
                    .iter()
                    .any(|(k, v)| k.contains("HTTPS") && v == "Ja")
                {
                    i18n.t("pdf-sec-https-yes")
                } else {
                    i18n.t("pdf-sec-https-unclear")
                },
            )
            .with_accent("#2563eb"),
            MetricStripItem::new("Issues", sec.issues.len().to_string()).with_accent("#dc2626"),
        ])
        .compact(),
    );

    if !sec.headers.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Header"),
            TableColumn::new("Status"),
            TableColumn::new(i18n.t("pdf-seo-value")),
        ])
        .with_title("Security Headers");
        for (name, status, val) in &sec.headers {
            table = table.add_row(vec![name.as_str(), status.as_str(), val.as_str()]);
        }
        builder = builder.add_component(table);
    }

    if !sec.ssl_info.is_empty() {
        let mut kv = KeyValueList::new().with_title("SSL/TLS");
        for (k, v) in &sec.ssl_info {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    if !sec.protection.is_empty() {
        let title = i18n.t("pdf-sec-protection-title");
        let mut kv = KeyValueList::new().with_title(&title);
        for (name, kind) in &sec.protection {
            kv = kv.add(name, kind);
        }
        builder = builder.add_component(kv);
    }

    for (title, sev, msg) in &sec.issues {
        builder = builder.add_component(Finding::new(title, map_severity(sev), msg));
    }

    if !sec.recommendations.is_empty() {
        let mut rec_list = List::new().with_title(i18n.t("label-improvement-suggestions"));
        for rec in &sec.recommendations {
            rec_list = rec_list.add_item(rec);
        }
        builder = builder.add_component(rec_list);
    }
    builder
}

pub(super) fn render_mobile(
    mut builder: renderreport::engine::ReportBuilder,
    mobile: &MobilePresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-mobile-usability")).with_level(2))
        .add_component(TextBlock::new(&mobile.interpretation))
        .add_component(
            ScoreCard::new(i18n.t("mobile-score-card"), mobile.score).with_thresholds(80, 50),
        );

    let configured_label = i18n.t("mobile-configured");
    let viewport_status = mobile
        .viewport
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("viewport"))
        .map(|(_, v)| v.as_str())
        .unwrap_or(&configured_label);
    let touch_targets = mobile
        .touch_targets
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("zu klein") || k.to_lowercase().contains("small"))
        .map(|(_, v)| v.as_str())
        .unwrap_or("n/a");
    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new("Viewport", viewport_status).with_accent("#0f766e"),
            MetricStripItem::new(i18n.t("mobile-touch-targets"), touch_targets)
                .with_accent("#d97706"),
            MetricStripItem::new("Issues", mobile.issues.len().to_string()).with_accent("#dc2626"),
        ])
        .compact(),
    );

    if !mobile.viewport.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-viewport-config"));
        for (k, v) in &mobile.viewport {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.touch_targets.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-touch-targets"));
        for (k, v) in &mobile.touch_targets {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.font_analysis.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-font-analysis"));
        for (k, v) in &mobile.font_analysis {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.content_sizing.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("mobile-content-sizing"));
        for (k, v) in &mobile.content_sizing {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    for (cat, sev, msg) in &mobile.issues {
        builder = builder.add_component(Finding::new(cat, map_severity(sev), msg));
    }
    builder
}

pub(super) fn render_ux(
    mut builder: renderreport::engine::ReportBuilder,
    ux: &UxPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-ux")).with_level(2))
        .add_component(
            Callout::info(&ux.interpretation).with_title(i18n.t("pdf-ux-overview-title")),
        )
        .add_component(
            ScoreCard::new(i18n.t("ux-score-card"), ux.score)
                .with_description(i18n.t("label-heuristic-indicator"))
                .with_thresholds(80, 50),
        );

    // Dimension scores as KeyValueList
    let mut kv = KeyValueList::new().with_title(i18n.t("ux-dimensions"));
    for dim in &ux.dimensions {
        kv = kv.add(&dim.name, format!("{}/100 — {}", dim.score, dim.summary));
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
        let desc = format!("{} — {}", issue.impact, issue.recommendation);
        builder = builder.add_component(Finding::new(&issue.dimension, sev, &desc));
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

pub(super) fn render_journey(
    mut builder: renderreport::engine::ReportBuilder,
    journey: &JourneyPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-journey")).with_level(2))
        .add_component(
            Callout::info(&journey.interpretation).with_title(i18n.t("pdf-journey-overview-title")),
        )
        .add_component(
            ScoreCard::new(i18n.t("journey-score-card"), journey.score)
                .with_description(i18n.t("label-heuristic-indicator"))
                .with_thresholds(80, 50),
        );

    // Page intent
    let mut kv = KeyValueList::new().with_title(i18n.t("journey-page-type-dimensions"));
    kv = kv.add(
        i18n.t("pdf-journey-detected-page-type"),
        &journey.page_intent,
    );
    for dim in &journey.dimensions {
        kv = kv.add(
            format!("{} ({}%)", dim.name, dim.weight_pct),
            format!("{}/100 — {}", dim.score, dim.summary),
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
        let desc = format!("[{}] {} — {}", fp.step, fp.impact, fp.recommendation);
        builder = builder.add_component(Finding::new(&fp.problem, sev, &desc));
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

pub(super) fn render_dark_mode(
    mut builder: renderreport::engine::ReportBuilder,
    dm: &DarkModePresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let support_label = if dm.supported {
        i18n.t("pdf-dm-status-supported")
    } else {
        i18n.t("pdf-dm-status-not-supported")
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-dark-mode")).with_level(2))
        .add_component(
            ScoreCard::new(i18n.t("pdf-dm-score-title"), dm.score).with_thresholds(80, 50),
        );

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

fn score_status(score: u32) -> &'static str {
    if score >= 75 {
        "good"
    } else if score >= 50 {
        "warn"
    } else {
        "bad"
    }
}

fn score_color(score: u32) -> &'static str {
    if score >= 75 {
        "#0f766e"
    } else if score >= 50 {
        "#d97706"
    } else {
        "#dc2626"
    }
}

fn vital_status(rating: &str) -> &'static str {
    match rating {
        "good" => "good",
        "needs-improvement" => "warn",
        "poor" => "bad",
        _ => "info",
    }
}

fn vital_color(rating: &str) -> &'static str {
    match rating {
        "good" => "#0f766e",
        "needs-improvement" => "#d97706",
        "poor" => "#dc2626",
        _ => "#2563eb",
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>()
        + "…"
}

pub(super) fn render_source_quality(
    mut builder: renderreport::engine::ReportBuilder,
    sq: &crate::source_quality::SourceQualityAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("pdf-sq-section-title")).with_level(2))
        .add_component(Callout::info(&sq.disclaimer).with_title(i18n.t("pdf-sq-overview-title")))
        .add_component(
            ScoreCard::new(i18n.t("pdf-sq-score-title"), sq.score)
                .with_description(i18n.t_args(
                    "pdf-sq-score-desc-format",
                    &[
                        ("grade", sq.grade.as_str()),
                        ("quality", score_quality_label(sq.score)),
                    ],
                ))
                .with_thresholds(70, 50),
        );

    if sq.score >= 80 {
        return builder.add_component(Callout::success(i18n.t("pdf-sq-success")));
    }

    for dim in [&sq.substance, &sq.consistency, &sq.authority] {
        builder = builder.add_component(Section::new(&dim.name).with_level(3));

        builder = builder.add_component(
            ScoreCard::new(score_quality_label(dim.score), dim.score)
                .with_description(&dim.label)
                .with_thresholds(70, 50),
        );

        if !dim.signals.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Signal"),
                TableColumn::new("Status"),
                TableColumn::new("Detail"),
            ]);

            for signal in &dim.signals {
                let status = if signal.present { "✓" } else { "✗" };
                table = table.add_row(vec![&signal.name, status, &signal.detail]);
            }
            builder = builder.add_component(table);
        }
    }

    builder
}

// ─── Tech Stack ─────────────────────────────────────────────────────────────

pub(super) fn render_tech_stack(
    mut builder: renderreport::engine::ReportBuilder,
    ts: &crate::tech_stack::TechStackAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::tech_stack::Confidence;

    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("pdf-ts-section-title")).with_level(2))
        .add_component(
            ScoreCard::new(i18n.t("pdf-ts-score-title"), ts.score)
                .with_description(format!("Grade: {}", ts.grade))
                .with_thresholds(80, 50),
        );

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
                format!("{:?}", tech.category),
                tech.version.clone().unwrap_or_else(|| "—".to_string()),
                confidence.to_string(),
                tech.signals.join(", "),
            ]);
        }
        builder = builder.add_component(table);
    }

    if !ts.findings.is_empty() {
        let mut findings_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ts-finding")).with_width("35%"),
            TableColumn::new(i18n.t("pdf-ts-severity")).with_width("15%"),
            TableColumn::new("Detail").with_width("50%"),
        ])
        .with_title(i18n.t("pdf-ts-findings-title"));

        for finding in &ts.findings {
            findings_table = findings_table.add_row(vec![
                finding.title.clone(),
                finding.severity.label().to_string(),
                finding.detail.clone(),
            ]);
        }
        builder = builder.add_component(findings_table);
    }

    builder
}

// ─── AI Visibility ──────────────────────────────────────────────────────────

pub(super) fn render_ai_visibility(
    mut builder: renderreport::engine::ReportBuilder,
    av: &crate::ai_visibility::AiVisibilityAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let indicator_note_ai = i18n.t("pdf-ai-indicator-note");
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("pdf-ai-section-title")).with_level(2))
        .add_component(
            Callout::info(&indicator_note_ai).with_title(i18n.t("pdf-seo-indicator-title")),
        )
        .add_component(Callout::info(&av.disclaimer).with_title(i18n.t("pdf-ai-overview-title")))
        .add_component(
            ScoreCard::new(i18n.t("pdf-ai-score-title"), av.score)
                .with_description(i18n.t_args(
                    "pdf-ai-score-desc-format",
                    &[
                        ("grade", av.grade.as_str()),
                        ("quality", score_quality_label(av.score)),
                    ],
                ))
                .with_thresholds(70, 50),
        );

    if av.score >= 80 {
        return builder.add_component(Callout::success(i18n.t("pdf-ai-success")));
    }

    // Render each dimension
    for (dim, title) in [
        (&av.readability.dimension, i18n.t("pdf-ai-readability")),
        (&av.citation.dimension, i18n.t("pdf-ai-citability")),
        (&av.chunks.dimension, i18n.t("pdf-ai-tech-readability")),
        (
            &av.knowledge_graph.dimension,
            i18n.t("pdf-seo-profile-structured-data"),
        ),
        (&av.policy.dimension, i18n.t("pdf-ai-policy")),
    ] {
        builder = builder.add_component(Section::new(title).with_level(3));
        let mut dim_kv = KeyValueList::new().add(
            i18n.t("label-heuristic-indicator"),
            format!("~{}/100 — {}", dim.score, score_quality_label(dim.score)),
        );
        if !dim.label.is_empty() {
            dim_kv = dim_kv.add("Basis", &dim.label);
        }
        builder = builder.add_component(dim_kv);

        if !dim.signals.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Signal"),
                TableColumn::new("Status"),
                TableColumn::new("Detail"),
            ]);

            for signal in dim.signals.iter().take(5) {
                let status = if signal.present { "✓" } else { "✗" };
                table = table.add_row(vec![&signal.name, status, &signal.detail]);
            }
            builder = builder.add_component(table);
            if dim.signals.len() > 5 {
                let more_note = i18n.t_args(
                    "pdf-ai-more-signals",
                    &[("count", (dim.signals.len() - 5).to_string())],
                );
                builder = builder.add_component(Callout::info(&more_note));
            }
        }
    }

    // Chunk sections summary
    if !av.chunks.sections.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-content-sections")).with_level(3));

        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ai-section-col")).with_width("70%"),
            TableColumn::new("Level").with_width("15%"),
            TableColumn::new(i18n.t("pdf-ai-words-col")).with_width("15%"),
        ])
        .with_title(i18n.t("pdf-ai-sections-title"));

        for section in &av.chunks.sections {
            table = table.add_row(vec![
                &section.heading,
                &format!("H{}", section.level),
                &section.word_count.to_string(),
            ]);
        }
        builder = builder.add_component(table);
        builder = builder.add_component(
            Callout::info(&av.chunks.recommendation).with_title(i18n.t("pdf-ai-rec-title")),
        );
    }

    // Knowledge graph entities
    if !av.knowledge_graph.entities.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-detected-entities")).with_level(3));

        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ai-entity-col")),
            TableColumn::new(i18n.t("pdf-perf-min-type")),
            TableColumn::new(i18n.t("pdf-seo-ie-source")),
        ])
        .with_title(i18n.t("pdf-ai-entities-title"));

        for entity in &av.knowledge_graph.entities {
            table = table.add_row(vec![
                &entity.name,
                &entity.entity_type,
                &entity.source.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Knowledge graph relationships
    if !av.knowledge_graph.relationships.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("pdf-ai-subject-col")),
            TableColumn::new(i18n.t("pdf-ai-relation-col")),
            TableColumn::new(i18n.t("pdf-ai-object-col")),
        ])
        .with_title(i18n.t("pdf-ai-relations-title"));

        for rel in &av.knowledge_graph.relationships {
            table = table.add_row(vec![&rel.subject, &rel.predicate, &rel.object]);
        }
        builder = builder.add_component(table);
    }

    // Link suggestions
    if !av.knowledge_graph.link_suggestions.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-link-suggestions")).with_level(3));

        let mut list = List::new();
        for suggestion in &av.knowledge_graph.link_suggestions {
            list = list.add_item(format!("{}: {}", suggestion.entity, suggestion.reason));
        }
        builder = builder.add_component(list);
    }

    // AI Policy details
    if av.policy.blocks_all {
        builder = builder.add_component(
            Callout::warning(i18n.t("pdf-ai-policy-blocks-all-body"))
                .with_title(i18n.t("pdf-ai-policy-blocks-all-title")),
        );
    } else if av.policy.blocks_ai_citation {
        let mut kv = KeyValueList::new().with_title(i18n.t("pdf-ai-policy-limited-title"));
        kv = kv.add("Policy", &av.policy.inferred_policy);
        kv = kv.add("Status", i18n.t("pdf-ai-policy-limited-body"));
        builder = builder.add_component(kv);
    } else if av.policy.blocks_ai_training {
        let mut kv = KeyValueList::new().with_title(i18n.t("pdf-ai-policy"));
        kv = kv.add("Policy", &av.policy.inferred_policy);
        kv = kv.add("Status", i18n.t("pdf-ai-policy-training-body"));
        builder = builder.add_component(kv);
    }

    builder
}

// ─── Content Visibility & Trust ─────────────────────────────────────────────

pub(super) fn render_content_visibility(
    mut builder: renderreport::engine::ReportBuilder,
    cv: &crate::content_visibility::ContentVisibilityAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    use crate::assessment::AssessmentLevel;

    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("pdf-cv-section-title")).with_level(2))
        .add_component(
            Callout::info(i18n.t("pdf-cv-overview-body"))
                .with_title(i18n.t("pdf-cv-overview-title")),
        )
        .add_component(TextBlock::new(i18n.t_args(
            "pdf-cv-signals-analyzed",
            &[
                ("signals", cv.signal_count.to_string()),
                ("problems", cv.problem_count.to_string()),
            ],
        )));

    let areas: Vec<(String, &[crate::assessment::ContentSignal])> = vec![
        (
            i18n.t("pdf-cv-area-organic-visibility"),
            &cv.organic_visibility,
        ),
        (i18n.t("pdf-cv-area-local-business"), &cv.local_business),
        (i18n.t("pdf-cv-area-eeat"), &cv.eeat),
        (i18n.t("pdf-cv-area-content-depth"), &cv.content_depth),
        (
            i18n.t("pdf-cv-area-topical-authority"),
            &cv.topical_authority,
        ),
    ];

    let mut not_testable_rows: Vec<ChecklistRow> = Vec::new();

    for (area_name, signals) in &areas {
        let visible: Vec<_> = signals
            .iter()
            .filter(|s| s.level != AssessmentLevel::NotTestable)
            .collect();

        // Collect NotTestable signals across all areas
        for s in signals
            .iter()
            .filter(|s| s.level == AssessmentLevel::NotTestable)
        {
            not_testable_rows.push(ChecklistRow::new(&s.title, &s.detail).with_status("info"));
        }

        if visible.is_empty() {
            continue;
        }

        builder = builder.add_component(Section::new(area_name.clone()).with_level(3));

        for signal in visible {
            let conf_prefix = match signal.confidence {
                crate::assessment::EvidenceConfidence::High => "● ",
                crate::assessment::EvidenceConfidence::Medium => "◐ ",
                crate::assessment::EvidenceConfidence::Low => "○ ",
            };
            let body = format!("{}{}", conf_prefix, signal.detail);
            let title = signal.title.clone();

            builder = builder.add_component(match signal.level {
                AssessmentLevel::Pass | AssessmentLevel::Positive => {
                    Callout::success(&body).with_title(&title)
                }
                AssessmentLevel::Warning => Callout::warning(&body).with_title(&title),
                AssessmentLevel::Violation => Callout::warning(&body).with_title(&title),
                AssessmentLevel::NotTestable => unreachable!(),
            });

            if !signal.evidence.is_empty() {
                let mut kv = KeyValueList::new();
                for ev in &signal.evidence {
                    let source_label = format!("{:?}", ev.source);
                    let mut detail = String::new();
                    if let Some(ref fp) = ev.field_path {
                        detail.push_str(fp);
                    }
                    if let Some(ref val) = ev.value_excerpt {
                        if !detail.is_empty() {
                            detail.push_str(": ");
                        }
                        detail.push_str(val);
                    }
                    if detail.is_empty() {
                        detail = source_label.clone();
                    }
                    kv = kv.add(&source_label, &detail);
                }
                builder = builder.add_component(kv);
            }
        }
    }

    if !not_testable_rows.is_empty() {
        let title = i18n.t("pdf-cv-manual-review-title");
        builder = builder
            .add_component(Section::new(&title).with_level(3))
            .add_component(ChecklistPanel::new(not_testable_rows).with_title(&title));
    }

    builder
}

pub(super) fn render_best_practices(
    mut builder: renderreport::engine::ReportBuilder,
    bp: &crate::best_practices::BestPracticesAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Best Practices").with_level(2))
        .add_component(
            ScoreCard::new(i18n.t("pdf-bp-score-title"), bp.score).with_thresholds(80, 50),
        );

    if bp.score >= 90
        && bp.console_errors.error_count == 0
        && !bp.vulnerable_libraries.has_vulnerabilities
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
            table = table.add_row(vec![
                lib.name.clone(),
                lib.version.clone(),
                lib.severity.clone(),
                lib.description.clone(),
                lib.safe_version.clone(),
            ]);
        }
        builder = builder.add_component(table);
    } else if !bp.vulnerable_libraries.detected.is_empty() {
        builder = builder.add_component(TextBlock::new(i18n.t("pdf-bp-libs-up-to-date")));
    }

    builder
}
