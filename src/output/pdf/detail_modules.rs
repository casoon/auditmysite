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

#[inline]
fn is_en(i18n: &I18n) -> bool {
    i18n.locale() == "en"
}

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

    let summary_text = if is_en(i18n) {
        format!(
            "{} budget violation{} detected: {} critical (>50% exceeded), {} warning{}.",
            violations.len(),
            if violations.len() == 1 { "" } else { "s" },
            error_count,
            warning_count,
            if warning_count == 1 { "" } else { "s" },
        )
    } else {
        format!(
            "{} Budget-Verletzung{} erkannt: {} kritisch (>50% überschritten), {} Warnung{}.",
            violations.len(),
            if violations.len() == 1 { "" } else { "en" },
            error_count,
            warning_count,
            if warning_count == 1 { "" } else { "en" },
        )
    };

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
    let perf_section_title = if is_en(i18n) {
        "Performance — User Experience & Technical Complexity"
    } else {
        "Performance — Nutzererlebnis & Technische Metriken"
    };
    let perf_intro = KeyValueList::new()
        .add(
            i18n.t("section-user-experience"),
            if is_en(i18n) {
                "Core Web Vitals, render blocking — how fast the page feels to users"
            } else {
                "Core Web Vitals, Render-Blocking — wie schnell die Seite für Nutzer wirkt"
            },
        )
        .add(
            i18n.t("section-technical-complexity"),
            if is_en(i18n) {
                "DOM size, resource loading, blocking budget"
            } else {
                "DOM-Größe, Ressourcen-Loading, Blocking-Budget"
            },
        );
    builder = builder
        .add_component(Section::new(perf_section_title).with_level(2))
        .add_component(perf_intro)
        .add_component(
            Callout::info(&perf.interpretation).with_title(if is_en(i18n) {
                "Performance — Overview"
            } else {
                "Performance — Überblick"
            }),
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
        let title = if is_en(i18n) {
            "Performance under throttled conditions"
        } else {
            "Performance unter gedrosselten Bedingungen"
        };
        let col_profile = if is_en(i18n) { "Profile" } else { "Profil" };
        let mut table = AuditTable::new(vec![
            TableColumn::new(col_profile).with_width("28%"),
            TableColumn::new("LCP").with_width("18%"),
            TableColumn::new("TBT").with_width("18%"),
            TableColumn::new("CLS").with_width("18%"),
            TableColumn::new("Score").with_width("18%"),
        ])
        .with_title(title);
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
        let title = if is_en(i18n) {
            "CLS — Layout Shifts"
        } else {
            "CLS — Layout-Verschiebungen"
        };
        let col_val = if is_en(i18n) { "Shift" } else { "Wert" };
        let col_time = if is_en(i18n) { "Time" } else { "Zeitpunkt" };
        let col_elem = "Element";
        let mut table = AuditTable::new(vec![
            TableColumn::new(col_val).with_width("15%"),
            TableColumn::new(col_time).with_width("20%"),
            TableColumn::new(col_elem).with_width("65%"),
        ])
        .with_title(title);
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
            let title = if is_en(i18n) {
                "Third-Party Resources"
            } else {
                "Drittanbieter-Ressourcen"
            };
            let col_origin = "Origin";
            let col_req = if is_en(i18n) { "Requests" } else { "Anfragen" };
            let mut kv = KeyValueList::new().with_title(title);
            kv = kv.add(
                if is_en(i18n) {
                    "Total origins"
                } else {
                    "Drittanbieter gesamt"
                },
                tp.total_origins.to_string(),
            );
            kv = kv.add(
                if is_en(i18n) {
                    "Total transfer"
                } else {
                    "Übertragung gesamt"
                },
                format!("{:.1} KB / {} Anfragen", tp.total_kb, tp.total_requests),
            );
            if tp.is_significant {
                kv = kv.add(
                    if is_en(i18n) { "Impact" } else { "Einfluss" },
                    if is_en(i18n) {
                        "Significant (>20% of page bytes)"
                    } else {
                        "Signifikant (>20% der Seitengröße)"
                    },
                );
            }
            builder = builder.add_component(kv);

            let mut table = AuditTable::new(vec![
                TableColumn::new(col_origin).with_width("48%"),
                TableColumn::new(col_req).with_width("16%"),
                TableColumn::new("KB").with_width("18%"),
                TableColumn::new(if is_en(i18n) { "Types" } else { "Typen" }).with_width("18%"),
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
        let title = if is_en(i18n) {
            "Critical Request Chain"
        } else {
            "Kritische Request-Kette"
        };
        let mut kv = KeyValueList::new().with_title(title);
        kv = kv.add(
            if is_en(i18n) {
                "Max depth"
            } else {
                "Max. Tiefe"
            },
            cc.max_depth.to_string(),
        );
        kv = kv.add(
            if is_en(i18n) {
                "Critical path"
            } else {
                "Kritischer Pfad"
            },
            format!("{} / {}", cc.critical_path_ms, cc.critical_path_kb),
        );
        kv = kv.add(
            if is_en(i18n) {
                "Total requests"
            } else {
                "Anfragen gesamt"
            },
            cc.total_requests.to_string(),
        );
        builder = builder.add_component(kv);
    }

    // ── Minification ─────────────────────────────────────────────────
    if let Some(ref min) = perf.minification {
        let title = if is_en(i18n) {
            "Unminified Assets"
        } else {
            "Unminifizierte Assets"
        };
        let mut kv = KeyValueList::new().with_title(title);
        kv = kv.add(
            if is_en(i18n) {
                "Unminified files"
            } else {
                "Unminifizierte Dateien"
            },
            min.total_count.to_string(),
        );
        kv = kv.add(
            if is_en(i18n) {
                "Est. savings"
            } else {
                "Geschätzte Einsparung"
            },
            format!("{:.1} KB", min.total_savings_kb),
        );
        builder = builder.add_component(kv);

        if !min.top_assets.is_empty() {
            let col_url = "URL";
            let col_kind = if is_en(i18n) { "Type" } else { "Typ" };
            let col_save = if is_en(i18n) { "Savings" } else { "Einsparung" };
            let mut table = AuditTable::new(vec![
                TableColumn::new(col_url).with_width("62%"),
                TableColumn::new(col_kind).with_width("16%"),
                TableColumn::new(col_save).with_width("22%"),
            ]);
            for (url, kind, savings) in &min.top_assets {
                table = table.add_row(vec![url.as_str(), kind.as_str(), savings.as_str()]);
            }
            builder = builder.add_component(table);
        }
    }

    // ── Coverage (unused JS/CSS) ──────────────────────────────────────
    if let Some(ref cov) = perf.coverage {
        let title = if is_en(i18n) {
            "Code Coverage"
        } else {
            "Code-Abdeckung"
        };
        let mut kv = KeyValueList::new().with_title(title);
        if let (Some(used_pct), Some(unused_kb)) = (cov.js_used_pct, cov.js_unused_kb) {
            kv = kv.add(
                if is_en(i18n) { "JS used" } else { "JS genutzt" },
                format!("{:.1}% ({:.1} KB ungenutzt)", used_pct, unused_kb),
            );
        }
        if let Some(used_pct) = cov.css_used_pct {
            let rules_str = match (cov.css_used_rules, cov.css_total_rules) {
                (Some(used), Some(total)) => {
                    format!("{:.1}% ({}/{} Regeln genutzt)", used_pct, used, total)
                }
                _ => format!("{:.1}%", used_pct),
            };
            kv = kv.add(
                if is_en(i18n) {
                    "CSS used"
                } else {
                    "CSS genutzt"
                },
                rules_str,
            );
        }
        builder = builder.add_component(kv);
    }

    // ── Non-composited Animations ─────────────────────────────────────
    if let Some(ref anim) = perf.animations {
        let title = if is_en(i18n) {
            "Non-composited Animations"
        } else {
            "Nicht-composited Animationen"
        };
        let mut kv = KeyValueList::new().with_title(title);
        kv = kv.add(
            if is_en(i18n) { "Total" } else { "Gesamt" },
            anim.total_count.to_string(),
        );
        if !anim.affected_properties.is_empty() {
            kv = kv.add(
                if is_en(i18n) {
                    "Properties"
                } else {
                    "Eigenschaften"
                },
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
    let indicator_note_seo = if is_en(i18n) {
        "Indicators are heuristic estimates based on measurable signals — not direct ranking signals, but pointers to optimization potential."
    } else {
        "Indikatoren sind heuristische Schätzwerte auf Basis messbarer Signale — kein direktes Ranking-Signal, sondern Hinweis auf Optimierungspotenzial."
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-seo-analysis")).with_level(2))
        .add_component(
            Callout::info(indicator_note_seo).with_title(if is_en(i18n) {
                "Indicator-based metrics"
            } else {
                "Indikator-basierte Metriken"
            }),
        )
        .add_component(
            Callout::info(&seo.interpretation).with_title(if is_en(i18n) {
                "SEO — Overview"
            } else {
                "SEO — Überblick"
            }),
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
            MetricStripItem::new(
                if is_en(i18n) { "Maturity" } else { "Reifegrad" },
                &profile.maturity_level,
            )
            .with_accent("#7c3aed"),
        );
    }
    if !seo_strip.is_empty() {
        builder = builder.add_component(MetricStrip::new(seo_strip).compact());
    }

    if !seo.meta_tags.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Field" } else { "Feld" }).with_width("24%"),
            TableColumn::new(if is_en(i18n) { "Value" } else { "Wert" }).with_width("76%"),
        ])
        .with_title(if is_en(i18n) {
            "Meta tags"
        } else {
            "Meta-Tags"
        });
        for (k, v) in &seo.meta_tags {
            table = table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder.add_component(table);
    }

    if !seo.meta_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Field" } else { "Feld" }),
            TableColumn::new(i18n.t("label-severity")),
            TableColumn::new(if is_en(i18n) {
                "Description"
            } else {
                "Beschreibung"
            }),
        ])
        .with_title(if is_en(i18n) {
            "Meta tag issues"
        } else {
            "Meta-Tag Probleme"
        });
        for (field, sev, msg) in &seo.meta_issues {
            table = table.add_row(vec![field.as_str(), sev.label(), msg.as_str()]);
        }
        builder = builder.add_component(table);
    }

    if !seo.heading_summary.is_empty() || !seo.social_summary.is_empty() {
        let mut kv = KeyValueList::new();
        if !seo.heading_summary.is_empty() {
            kv = kv.add(
                if is_en(i18n) {
                    "Headings"
                } else {
                    "Überschriften"
                },
                &seo.heading_summary,
            );
        }
        if !seo.social_summary.is_empty() {
            kv = kv.add(
                if is_en(i18n) {
                    "Social tags"
                } else {
                    "Social Tags"
                },
                &seo.social_summary,
            );
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
            let more_note = if is_en(i18n) {
                format!(
                    "{} additional signals in the detailed appendix.",
                    seo.technical_summary.len() - 5
                )
            } else {
                format!(
                    "{} weitere Signale im detaillierten Anhang.",
                    seo.technical_summary.len() - 5
                )
            };
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
        let title = if is_en(i18n) {
            "Image Efficiency"
        } else {
            "Bild-Effizienz"
        };
        let mut kv = KeyValueList::new().with_title(title);
        kv = kv.add(
            if is_en(i18n) {
                "Total images"
            } else {
                "Bilder gesamt"
            },
            ie.total_images.to_string(),
        );
        kv = kv.add(
            if is_en(i18n) {
                "Modern formats"
            } else {
                "Moderne Formate"
            },
            format!("{:.1}% (WebP/AVIF/SVG)", ie.modern_format_pct),
        );
        if ie.legacy_count > 0 {
            kv = kv.add(
                if is_en(i18n) {
                    "Legacy formats"
                } else {
                    "Legacy-Formate"
                },
                format!("{} (JPG/PNG/GIF)", ie.legacy_count),
            );
        }
        builder = builder.add_component(kv);

        if !ie.oversized.is_empty() {
            let title_tbl = if is_en(i18n) {
                "Oversized images (top 5)"
            } else {
                "Übergroße Bilder (Top 5)"
            };
            let col_src = if is_en(i18n) { "Source" } else { "Quelle" };
            let col_nat = if is_en(i18n) { "Natural" } else { "Nativ" };
            let col_dis = if is_en(i18n) {
                "Displayed"
            } else {
                "Angezeigt"
            };
            let mut table = AuditTable::new(vec![
                TableColumn::new(col_src).with_width("56%"),
                TableColumn::new(col_nat).with_width("22%"),
                TableColumn::new(col_dis).with_width("22%"),
            ])
            .with_title(title_tbl);
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

    let summary = if is_en(i18n) {
        format!(
            "{} signals checked — {} OK, {} warnings, {} failures.",
            serp.signals.len(),
            serp.pass_count,
            serp.warning_count,
            serp.fail_count,
        )
    } else {
        format!(
            "{} Signale geprüft — {} OK, {} Warnungen, {} Fehler.",
            serp.signals.len(),
            serp.pass_count,
            serp.warning_count,
            serp.fail_count,
        )
    };
    let serp_readiness_title = i18n.t("seo-serp-readiness");
    builder = if serp.fail_count > 0 {
        builder.add_component(Callout::warning(&summary).with_title(&serp_readiness_title))
    } else if serp.warning_count > 0 {
        builder.add_component(Callout::info(&summary).with_title(&serp_readiness_title))
    } else {
        builder.add_component(Callout::success(&summary).with_title(&serp_readiness_title))
    };

    if !serp.signals.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Category" } else { "Kategorie" }).with_width("22%"),
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
        let text = if is_en(i18n) {
            format!(
                "Rich result types possible: {}",
                serp.rich_result_types.join(", ")
            )
        } else {
            format!(
                "Rich-Result-Typen möglich: {}",
                serp.rich_result_types.join(", ")
            )
        };
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
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Issue" } else { "Problem" }).with_width("55%"),
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
        let validator_title = if is_en(i18n) {
            "W3C HTML validator"
        } else {
            "W3C HTML Validator"
        };
        let callout = match status.as_str() {
            "Ausgeführt" => Callout::info(detail).with_title(validator_title),
            "Fehlgeschlagen" => Callout::warning(detail).with_title(validator_title),
            _ => Callout::info(detail).with_title(validator_title),
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
            .with_title(if is_en(i18n) {
                "www consolidation"
            } else {
                "www-Konsolidierung"
            }),
        );
    }

    // HTML validation table
    if !ph.html_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Check" } else { "Prüfung" }).with_width("40%"),
            TableColumn::new(if is_en(i18n) { "Count" } else { "Anzahl" }).with_width("15%"),
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
        return builder.add_component(if is_en(i18n) {
            Callout::warning(format!("robots.txt could not be loaded: {err}"))
                .with_title("No access")
        } else {
            Callout::warning(format!("robots.txt konnte nicht geladen werden: {err}"))
                .with_title("Kein Zugriff")
        });
    }

    // Summary callout — only warn for genuinely problematic configurations
    if robots.has_wildcard_disallow_all {
        builder = builder.add_component(if is_en(i18n) {
            Callout::warning(
                "All crawlers fully blocked (User-agent: * / Disallow: /). \
                 On staging domains this is correct — on production it would \
                 prevent search engines from crawling the site entirely.",
            )
            .with_title("All crawlers blocked")
        } else {
            Callout::warning(
                "Alle Crawler vollständig gesperrt (User-agent: * / Disallow: /). \
                 Auf Staging-Domains ist das korrekt — auf der Produktiv-Domain würde dies \
                 das vollständige Crawling durch Suchmaschinen verhindern.",
            )
            .with_title("Alle Crawler gesperrt")
        });
    } else if robots.blocks_ai_citation {
        builder = builder.add_component(if is_en(i18n) {
            Callout::info(
                "AI search bots (e.g. PerplexityBot, Amazonbot) are blocked. \
                 This is a deliberate choice — content will not appear in \
                 AI-generated answers. Blocking AI training bots (GPTBot etc.) \
                 is common practice and not a problem.",
            )
            .with_title("Limited AI visibility")
        } else {
            Callout::info(
                "KI-Suchbots (z. B. PerplexityBot, Amazonbot) sind blockiert. \
                 Das ist eine bewusste Entscheidung — Inhalte erscheinen nicht in \
                 KI-generierten Antworten. Das Sperren von KI-Trainingsbots (GPTBot etc.) \
                 ist dagegen übliche Praxis und kein Problem.",
            )
            .with_title("Eingeschränkte KI-Sichtbarkeit")
        });
    } else if !robots.blocked_ai_bots.is_empty() {
        // Training bots blocked, citation bots allowed — this is the citationFriendly default
        builder = builder.add_component(if is_en(i18n) {
            Callout::info(format!(
                "Policy: {} — AI training bots ({}) are blocked, \
                 AI search bots have access. This matches the recommended default configuration.",
                robots.inferred_policy,
                robots.blocked_ai_bots.join(", ")
            ))
            .with_title("AI training blocked (default)")
        } else {
            Callout::info(format!(
                "Policy: {} — KI-Trainingsbots ({}) sind gesperrt, \
                 KI-Suchbots haben Zugang. Das entspricht der empfohlenen Standardkonfiguration.",
                robots.inferred_policy,
                robots.blocked_ai_bots.join(", ")
            ))
            .with_title("KI-Training blockiert (Standard)")
        });
    }

    // Bot overview table
    if !robots.bot_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("User-agent").with_width("28%"),
            TableColumn::new(if is_en(i18n) { "Category" } else { "Kategorie" }).with_width("26%"),
            TableColumn::new(if is_en(i18n) { "Allowed" } else { "Erlaubt" }).with_width("13%"),
            TableColumn::new(if is_en(i18n) { "Blocked" } else { "Gesperrt" }).with_width("13%"),
            TableColumn::new("Status").with_width("20%"),
        ])
        .with_title(if is_en(i18n) {
            "Crawler rules"
        } else {
            "Crawler-Regeln"
        });

        for (ua, class, allows, disallows, fully_blocked) in &robots.bot_rows {
            let status = if *fully_blocked {
                if is_en(i18n) {
                    "Fully blocked"
                } else {
                    "Vollständig gesperrt"
                }
            } else if *disallows > 0 {
                if is_en(i18n) {
                    "Partially blocked"
                } else {
                    "Teilweise gesperrt"
                }
            } else if is_en(i18n) {
                "Allowed"
            } else {
                "Erlaubt"
            };
            table = table.add_row(vec![
                ua.clone(),
                class.clone(),
                allows.to_string(),
                disallows.to_string(),
                status.to_string(),
            ]);
        }

        builder = builder.add_component(table);
    }

    // Sitemaps
    if !robots.sitemaps.is_empty() {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "Sitemap entries"
        } else {
            "Sitemap-Einträge"
        });
        for (i, sitemap) in robots.sitemaps.iter().enumerate() {
            kv = kv.add(format!("Sitemap {}", i + 1), sitemap);
        }
        builder = builder.add_component(kv);
    }

    // Crawl delays
    if !robots.crawl_delays.is_empty() {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "Crawl-delay values"
        } else {
            "Crawl-Delay-Werte"
        });
        for (ua, delay) in &robots.crawl_delays {
            kv = kv.add(
                ua,
                if is_en(i18n) {
                    format!("{delay} seconds")
                } else {
                    format!("{delay} Sekunden")
                },
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
        TableColumn::new(if is_en(i18n) { "Aspect" } else { "Aspekt" }).with_width("24%"),
        TableColumn::new(if is_en(i18n) { "Value" } else { "Wert" }).with_width("76%"),
    ])
    .with_title(if is_en(i18n) {
        "Content profile"
    } else {
        "Inhaltsprofil"
    });
    for (key, value) in &profile.identity_facts {
        identity_table = identity_table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut page_profile_table = AuditTable::new(vec![
        TableColumn::new(if is_en(i18n) { "Aspect" } else { "Aspekt" }).with_width("24%"),
        TableColumn::new(if is_en(i18n) { "Value" } else { "Wert" }).with_width("76%"),
    ])
    .with_title(if is_en(i18n) {
        "Page profile"
    } else {
        "Seitenprofil"
    });
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
            if is_en(i18n) {
                "Content depth"
            } else {
                "Content-Tiefe"
            },
            profile.content_depth_score,
            score_quality_label(profile.content_depth_score),
            score_quality_color(profile.content_depth_score),
        ),
        (
            if is_en(i18n) {
                "Structural quality"
            } else {
                "Strukturqualität"
            },
            profile.structural_richness_score,
            score_quality_label(profile.structural_richness_score),
            score_quality_color(profile.structural_richness_score),
        ),
        (
            if is_en(i18n) {
                "Media balance"
            } else {
                "Medienbalance"
            },
            profile.media_text_balance_score,
            score_quality_label(profile.media_text_balance_score),
            score_quality_color(profile.media_text_balance_score),
        ),
        (
            "Intent-Fit",
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
    let mut identity = KeyValueList::new().with_title(if is_en(i18n) {
        "Website identity"
    } else {
        "Website-Identität"
    });
    identity = identity.add("Website", &profile.site_name);
    identity = identity.add(
        if is_en(i18n) {
            "Content type"
        } else {
            "Inhaltstyp"
        },
        &profile.content_type,
    );
    identity = identity.add(
        if is_en(i18n) { "Language" } else { "Sprache" },
        &profile.language,
    );
    if !profile.category_hints.is_empty() {
        identity = identity.add(
            if is_en(i18n) {
                "Schema types"
            } else {
                "Schema-Typen"
            },
            profile.category_hints.join(", "),
        );
    }
    builder = builder.add_component(identity);

    // Schema Inventory
    if !profile.schema_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) {
                "Schema type"
            } else {
                "Schema-Typ"
            }),
            TableColumn::new(if is_en(i18n) {
                "Completeness"
            } else {
                "Vollständigkeit"
            }),
            TableColumn::new("Details"),
        ])
        .with_title(if is_en(i18n) {
            format!("Structured data ({} schemas)", profile.schema_count)
        } else {
            format!("Strukturierte Daten ({} Schemas)", profile.schema_count)
        });
        for (typ, completeness, details) in &profile.schema_rows {
            table = table.add_row(vec![typ.as_str(), completeness.as_str(), details.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Strength Overview
    if !profile.signal_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Category" } else { "Kategorie" }),
            TableColumn::new(if is_en(i18n) { "Rating" } else { "Bewertung" }),
            TableColumn::new(if is_en(i18n) {
                "Classification"
            } else {
                "Einstufung"
            }),
        ])
        .with_title(if is_en(i18n) {
            format!(
                "SEO signal strength (overall: {}%)",
                profile.signal_overall_pct
            )
        } else {
            format!("SEO-Signalstärke (Gesamt: {}%)", profile.signal_overall_pct)
        });
        for (cat, score, rating) in &profile.signal_rows {
            table = table.add_row(vec![cat.as_str(), score.as_str(), rating.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Details per category
    for (cat_name, checks) in &profile.signal_details {
        let mut detail_table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Check" } else { "Prüfung" }),
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
    let mut maturity = KeyValueList::new().with_title(if is_en(i18n) {
        "SEO maturity"
    } else {
        "SEO-Reifegrad"
    });
    maturity = maturity.add("Level", &profile.maturity_level);
    maturity = maturity.add(
        if is_en(i18n) { "Rating" } else { "Bewertung" },
        &profile.maturity_description,
    );
    maturity = maturity.add(
        if is_en(i18n) {
            "Techniques"
        } else {
            "Techniken"
        },
        if is_en(i18n) {
            format!(
                "{} of {} detected",
                profile.maturity_techniques_used, profile.maturity_techniques_total
            )
        } else {
            format!(
                "{} von {} erkannt",
                profile.maturity_techniques_used, profile.maturity_techniques_total
            )
        },
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
                    if is_en(i18n) {
                        "Yes"
                    } else {
                        "Ja"
                    }
                } else if is_en(i18n) {
                    "Unclear"
                } else {
                    "Unklar"
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
            TableColumn::new(if is_en(i18n) { "Value" } else { "Wert" }),
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
        let title = if is_en(i18n) {
            "Protection Infrastructure"
        } else {
            "Schutzinfrastruktur"
        };
        let mut kv = KeyValueList::new().with_title(title);
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
            Callout::info(&ux.interpretation).with_title(if is_en(i18n) {
                "UX — Overview"
            } else {
                "UX — Überblick"
            }),
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
        let more_note = if is_en(i18n) {
            format!(
                "{} additional issues in the detailed appendix.",
                ux.issues.len() - 3
            )
        } else {
            format!(
                "{} weitere Befunde im detaillierten Anhang.",
                ux.issues.len() - 3
            )
        };
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
            Callout::info(&journey.interpretation).with_title(if is_en(i18n) {
                "Journey — Overview"
            } else {
                "Journey — Überblick"
            }),
        )
        .add_component(
            ScoreCard::new(i18n.t("journey-score-card"), journey.score)
                .with_description(i18n.t("label-heuristic-indicator"))
                .with_thresholds(80, 50),
        );

    // Page intent
    let mut kv = KeyValueList::new().with_title(i18n.t("journey-page-type-dimensions"));
    kv = kv.add(
        if is_en(i18n) {
            "Detected page type"
        } else {
            "Erkannter Seitentyp"
        },
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
        let more_note = if is_en(i18n) {
            format!(
                "{} additional issues in the detailed appendix.",
                journey.friction_points.len() - 3
            )
        } else {
            format!(
                "{} weitere Befunde im detaillierten Anhang.",
                journey.friction_points.len() - 3
            )
        };
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
        if is_en(i18n) {
            "Supported"
        } else {
            "Unterstützt"
        }
    } else if is_en(i18n) {
        "Not supported"
    } else {
        "Nicht unterstützt"
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new(i18n.t("section-dark-mode")).with_level(2))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Dark mode score"
                } else {
                    "Dark Mode Score"
                },
                dm.score,
            )
            .with_thresholds(80, 50),
        );

    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new("Status", support_label)
                .with_status(if dm.supported { "good" } else { "warn" })
                .with_accent(if dm.supported { "#0f766e" } else { "#d97706" }),
            MetricStripItem::new(
                if is_en(i18n) { "Methods" } else { "Methoden" },
                dm.detection_methods.len().to_string(),
            )
            .with_accent("#2563eb"),
            MetricStripItem::new(
                if is_en(i18n) {
                    "CSS variables"
                } else {
                    "CSS Variablen"
                },
                dm.css_custom_properties.to_string(),
            )
            .with_accent("#7c3aed"),
        ])
        .compact(),
    );

    let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
        "Dark mode overview"
    } else {
        "Dark Mode Übersicht"
    });
    kv = kv.add(
        if is_en(i18n) {
            "Support"
        } else {
            "Unterstützung"
        },
        support_label,
    );
    if !dm.detection_methods.is_empty() {
        kv = kv.add(
            if is_en(i18n) {
                "Implementation methods"
            } else {
                "Implementierungsmethoden"
            },
            dm.detection_methods.join(", "),
        );
    }
    kv = kv.add(
        "color-scheme CSS",
        if dm.color_scheme_css {
            if is_en(i18n) {
                "Yes"
            } else {
                "Ja"
            }
        } else if is_en(i18n) {
            "No"
        } else {
            "Nein"
        },
    );
    if let Some(ref meta) = dm.meta_color_scheme {
        kv = kv.add("<meta name=\"color-scheme\">", meta.as_str());
    }
    if dm.css_custom_properties > 0 {
        kv = kv.add(
            if is_en(i18n) {
                "CSS custom properties (colors)"
            } else {
                "CSS Custom Properties (Farben)"
            },
            dm.css_custom_properties.to_string(),
        );
    }
    if dm.supported {
        kv = kv.add(
            if is_en(i18n) {
                "Contrast violations in dark mode"
            } else {
                "Kontrast-Violations im Dark Mode"
            },
            dm.dark_contrast_violations.to_string(),
        );
        if dm.dark_only_violations > 0 {
            kv = kv.add(
                if is_en(i18n) {
                    "Dark-mode-only issues"
                } else {
                    "Nur-Dark-Mode-Probleme"
                },
                if is_en(i18n) {
                    format!("{} (not in light mode)", dm.dark_only_violations)
                } else {
                    format!("{} (nicht im Light Mode)", dm.dark_only_violations)
                },
            );
        }
        if dm.light_only_violations > 0 {
            kv = kv.add(
                if is_en(i18n) {
                    "Resolved in dark mode"
                } else {
                    "Im Dark Mode behoben"
                },
                if is_en(i18n) {
                    format!(
                        "{} light-mode issues disappear in dark mode",
                        dm.light_only_violations
                    )
                } else {
                    format!(
                        "{} Light-Mode-Probleme verschwinden im Dark Mode",
                        dm.light_only_violations
                    )
                },
            );
        }
    }
    builder = builder.add_component(kv);

    if !dm.issues.is_empty() {
        for (severity, description) in &dm.issues {
            builder = builder.add_component(match severity.as_str() {
                "high" => Callout::warning(description).with_title(if is_en(i18n) {
                    "Dark mode issue"
                } else {
                    "Dark Mode Problem"
                }),
                _ => Callout::info(description).with_title(if is_en(i18n) {
                    "Dark mode note"
                } else {
                    "Dark Mode Hinweis"
                }),
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
        .add_component(
            Section::new(if is_en(i18n) {
                "Source quality"
            } else {
                "Quellenqualität"
            })
            .with_level(2),
        )
        .add_component(Callout::info(&sq.disclaimer).with_title(if is_en(i18n) {
            "Source Quality — Overview"
        } else {
            "Quellqualität — Überblick"
        }))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Source quality (indicator)"
                } else {
                    "Quellenqualität (Indikator)"
                },
                sq.score,
            )
            .with_description(if is_en(i18n) {
                format!(
                    "Grade: {} — {} · Heuristic estimate, not a measured value",
                    sq.grade,
                    score_quality_label(sq.score)
                )
            } else {
                format!(
                    "Grade: {} — {} · Heuristische Schätzung, kein Messwert",
                    sq.grade,
                    score_quality_label(sq.score)
                )
            })
            .with_thresholds(70, 50),
        );

    if sq.score >= 80 {
        return builder.add_component(Callout::success(if is_en(i18n) {
            "All source quality signals look good. No critical issues detected."
        } else {
            "Alle Quellenqualitäts-Signale sind in Ordnung. Keine kritischen Probleme erkannt."
        }));
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
        .add_component(
            Section::new(if is_en(i18n) {
                "Tech Stack"
            } else {
                "Tech-Stack"
            })
            .with_level(2),
        )
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Stack security score"
                } else {
                    "Stack-Sicherheitsscore"
                },
                ts.score,
            )
            .with_description(format!("Grade: {}", ts.grade))
            .with_thresholds(80, 50),
        );

    if !ts.detected.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) {
                "Technology"
            } else {
                "Technologie"
            })
            .with_width("28%"),
            TableColumn::new(if is_en(i18n) { "Category" } else { "Kategorie" }).with_width("22%"),
            TableColumn::new("Version").with_width("18%"),
            TableColumn::new(if is_en(i18n) {
                "Confidence"
            } else {
                "Konfidenz"
            })
            .with_width("14%"),
            TableColumn::new("Detail").with_width("18%"),
        ])
        .with_title(if is_en(i18n) {
            "Detected technologies"
        } else {
            "Erkannte Technologien"
        });

        for tech in &ts.detected {
            let confidence = match tech.confidence {
                Confidence::High => {
                    if is_en(i18n) {
                        "High"
                    } else {
                        "Hoch"
                    }
                }
                Confidence::Medium => {
                    if is_en(i18n) {
                        "Medium"
                    } else {
                        "Mittel"
                    }
                }
                Confidence::Low => {
                    if is_en(i18n) {
                        "Low"
                    } else {
                        "Niedrig"
                    }
                }
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
            TableColumn::new(if is_en(i18n) { "Finding" } else { "Befund" }).with_width("35%"),
            TableColumn::new(if is_en(i18n) {
                "Severity"
            } else {
                "Schweregrad"
            })
            .with_width("15%"),
            TableColumn::new("Detail").with_width("50%"),
        ])
        .with_title(if is_en(i18n) {
            "Stack security findings"
        } else {
            "Stack-Sicherheitsbefunde"
        });

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
    let indicator_note_ai = if is_en(i18n) {
        "Experimental heuristics — not a standardised metric. Indicators are heuristic estimates based on measurable signals, not direct ranking signals. Use them as pointers to optimisation potential, not as evidence of LLM behaviour."
    } else {
        "Experimentelle Heuristiken — keine standardisierte Metrik. Die Indikatoren sind heuristische Schätzwerte auf Basis messbarer Signale, kein direktes Ranking-Signal. Sie zeigen Optimierungspotenzial, beweisen aber kein konkretes LLM-Verhalten."
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(
            Section::new(if is_en(i18n) {
                "AI Visibility (indicator)"
            } else {
                "AI-Sichtbarkeit (Indikator)"
            })
            .with_level(2),
        )
        .add_component(Callout::info(indicator_note_ai).with_title(if is_en(i18n) {
            "Indicator-based metrics"
        } else {
            "Indikator-basierte Metriken"
        }))
        .add_component(Callout::info(&av.disclaimer).with_title(if is_en(i18n) {
            "AI Visibility — Overview"
        } else {
            "KI-Sichtbarkeit — Überblick"
        }))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "AI visibility (indicator)"
                } else {
                    "AI-Sichtbarkeit (Indikator)"
                },
                av.score,
            )
            .with_description(if is_en(i18n) {
                format!(
                    "Grade: {} — {} · Heuristic estimate, not a measured value",
                    av.grade,
                    score_quality_label(av.score)
                )
            } else {
                format!(
                    "Grade: {} — {} · Heuristische Schätzung, kein Messwert",
                    av.grade,
                    score_quality_label(av.score)
                )
            })
            .with_thresholds(70, 50),
        );

    if av.score >= 80 {
        return builder.add_component(Callout::success(if is_en(i18n) {
            "All AI visibility signals look good. No optimization required."
        } else {
            "Alle KI-Sichtbarkeits-Signale sind in Ordnung. Kein Optimierungsbedarf."
        }));
    }

    // Render each dimension
    for (dim, title) in [
        (
            &av.readability.dimension,
            if is_en(i18n) {
                "AI readability"
            } else {
                "KI-Lesbarkeit"
            },
        ),
        (
            &av.citation.dimension,
            if is_en(i18n) {
                "Citability"
            } else {
                "Zitierbarkeit"
            },
        ),
        (
            &av.chunks.dimension,
            if is_en(i18n) {
                "AI readability"
            } else {
                "Technische KI-Lesbarkeit"
            },
        ),
        (
            &av.knowledge_graph.dimension,
            if is_en(i18n) {
                "Structured data"
            } else {
                "Strukturierte Daten"
            },
        ),
        (
            &av.policy.dimension,
            if is_en(i18n) {
                "AI policy"
            } else {
                "AI-Policy"
            },
        ),
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
                let more_note = if is_en(i18n) {
                    format!(
                        "{} additional signals in the detailed appendix.",
                        dim.signals.len() - 5
                    )
                } else {
                    format!(
                        "{} weitere Signale im detaillierten Anhang.",
                        dim.signals.len() - 5
                    )
                };
                builder = builder.add_component(Callout::info(&more_note));
            }
        }
    }

    // Chunk sections summary
    if !av.chunks.sections.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-content-sections")).with_level(3));

        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Section" } else { "Abschnitt" }).with_width("70%"),
            TableColumn::new("Level").with_width("15%"),
            TableColumn::new(if is_en(i18n) { "Words" } else { "Wörter" }).with_width("15%"),
        ])
        .with_title(if is_en(i18n) {
            "Sections"
        } else {
            "Abschnitte"
        });

        for section in &av.chunks.sections {
            table = table.add_row(vec![
                &section.heading,
                &format!("H{}", section.level),
                &section.word_count.to_string(),
            ]);
        }
        builder = builder.add_component(table);
        builder = builder.add_component(Callout::info(&av.chunks.recommendation).with_title(
            if is_en(i18n) {
                "Recommendation"
            } else {
                "Empfehlung"
            },
        ));
    }

    // Knowledge graph entities
    if !av.knowledge_graph.entities.is_empty() {
        builder =
            builder.add_component(Section::new(i18n.t("section-detected-entities")).with_level(3));

        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Entity" } else { "Entität" }),
            TableColumn::new(if is_en(i18n) { "Type" } else { "Typ" }),
            TableColumn::new(if is_en(i18n) { "Source" } else { "Quelle" }),
        ])
        .with_title(if is_en(i18n) {
            "Entities"
        } else {
            "Entitäten"
        });

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
            TableColumn::new(if is_en(i18n) { "Subject" } else { "Subjekt" }),
            TableColumn::new(if is_en(i18n) { "Relation" } else { "Beziehung" }),
            TableColumn::new(if is_en(i18n) { "Object" } else { "Objekt" }),
        ])
        .with_title(if is_en(i18n) {
            "Relationships"
        } else {
            "Beziehungen"
        });

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
        builder = builder.add_component(if is_en(i18n) {
            Callout::warning(
                "All crawlers blocked (Disallow: *) — AI search bots have no access either.",
            )
            .with_title("No AI access")
        } else {
            Callout::warning(
                "Alle Crawler gesperrt (Disallow: *) — auch KI-Suchbots haben keinen Zugang.",
            )
            .with_title("Kein KI-Zugang")
        });
    } else if av.policy.blocks_ai_citation {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "AI visibility limited"
        } else {
            "KI-Sichtbarkeit eingeschränkt"
        });
        kv = kv.add("Policy", &av.policy.inferred_policy);
        kv = kv.add(
            "Status",
            if is_en(i18n) {
                "AI search bots blocked — content will not appear in AI-generated answers"
            } else {
                "KI-Suchbots blockiert — Inhalte erscheinen nicht in KI-generierten Antworten"
            },
        );
        builder = builder.add_component(kv);
    } else if av.policy.blocks_ai_training {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "AI policy"
        } else {
            "KI-Policy"
        });
        kv = kv.add("Policy", &av.policy.inferred_policy);
        kv = kv.add(
            "Status",
            if is_en(i18n) {
                "AI training bots blocked, AI search bots have access — recommended default"
            } else {
                "KI-Trainingsbots blockiert, KI-Suchbots haben Zugang — empfohlener Standard"
            },
        );
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
        .add_component(
            Section::new(if is_en(i18n) {
                "Content Visibility & Trust (indicator)"
            } else {
                "Content Visibility & Trust (Indikator)"
            })
            .with_level(2),
        )
        .add_component(Callout::info(if is_en(i18n) {
            "Aggregated indicator from SEO, Source Quality, and AI Visibility signals. \
             Covers organic search indexability, E-E-A-T authority markers, local business \
             presence, content depth, and topical authority heuristics. \
             Heuristic estimate — not a directly measured value."
        } else {
            "Aggregierter Indikator aus SEO-, Quellenqualitäts- und KI-Sichtbarkeitssignalen. \
             Umfasst organische Indexierbarkeit, E-E-A-T-Autoritätssignale, lokale Geschäftspräsenz, \
             Inhaltstiefe und topische Relevanz-Heuristiken. \
             Heuristischer Schätzwert — kein direkt gemessener Wert."
        }).with_title(
            if is_en(i18n) { "Content Visibility — Overview" } else { "Content-Sichtbarkeit — Überblick" }
        ))
        .add_component(TextBlock::new(&if is_en(i18n) {
            format!(
                "{} signals analyzed, {} with optimization potential.",
                cv.signal_count, cv.problem_count
            )
        } else {
            format!(
                "{} Signale analysiert, {} Hinweise auf Optimierungsbedarf.",
                cv.signal_count, cv.problem_count
            )
        }));

    let areas: &[(&str, &str, &[crate::assessment::ContentSignal])] = &[
        (
            "Organische Sichtbarkeit",
            "Organic Visibility",
            &cv.organic_visibility,
        ),
        (
            "Local Business & Vertrauensdaten",
            "Local Business & Trust Data",
            &cv.local_business,
        ),
        ("E-E-A-T-Indizien", "E-E-A-T Indicators", &cv.eeat),
        (
            "Inhaltstiefe & Lokalisierung",
            "Content Depth & Localization",
            &cv.content_depth,
        ),
        (
            "Topical Authority (Heuristik)",
            "Topical Authority (heuristic)",
            &cv.topical_authority,
        ),
    ];

    let mut not_testable_rows: Vec<ChecklistRow> = Vec::new();

    for (de_name, en_name, signals) in areas {
        let area_name = if is_en(i18n) { en_name } else { de_name };
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

        builder = builder.add_component(Section::new(*area_name).with_level(3));

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
        let title = if is_en(i18n) {
            "Manual review required"
        } else {
            "Manuell prüfen"
        };
        builder = builder
            .add_component(Section::new(title).with_level(3))
            .add_component(ChecklistPanel::new(not_testable_rows).with_title(title));
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
            ScoreCard::new(
                if is_en(i18n) {
                    "Best Practices score"
                } else {
                    "Best Practices Score"
                },
                bp.score,
            )
            .with_thresholds(80, 50),
        );

    if bp.score >= 90
        && bp.console_errors.error_count == 0
        && !bp.vulnerable_libraries.has_vulnerabilities
    {
        return builder.add_component(TextBlock::new(if is_en(i18n) {
            "No console errors and no vulnerable libraries detected."
        } else {
            "Keine Konsolfehler und keine anfälligen Bibliotheken erkannt."
        }));
    }

    // Console errors
    if bp.console_errors.error_count > 0 {
        let title = if is_en(i18n) {
            "Console Errors"
        } else {
            "Konsolenfehler"
        };
        let mut table = AuditTable::new(vec![
            TableColumn::new("Level").with_width("15%"),
            TableColumn::new(if is_en(i18n) { "Message" } else { "Meldung" }).with_width("85%"),
        ])
        .with_title(title);
        for error in &bp.console_errors.errors {
            table = table.add_row(vec![error.level.clone(), error.message.clone()]);
        }
        builder = builder.add_component(table);
    }

    // Vulnerable libraries
    if bp.vulnerable_libraries.has_vulnerabilities {
        let title = if is_en(i18n) {
            "Vulnerable Libraries"
        } else {
            "Anfällige Bibliotheken"
        };
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Library" } else { "Bibliothek" }).with_width("20%"),
            TableColumn::new("Version").with_width("15%"),
            TableColumn::new(if is_en(i18n) { "Severity" } else { "Schwere" }).with_width("15%"),
            TableColumn::new(if is_en(i18n) { "Issue" } else { "Problem" }).with_width("35%"),
            TableColumn::new(if is_en(i18n) { "Fix" } else { "Lösung" }).with_width("15%"),
        ])
        .with_title(title);
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
        builder = builder.add_component(TextBlock::new(if is_en(i18n) {
            "Detected libraries appear to be up to date — no known vulnerabilities found."
        } else {
            "Erkannte Bibliotheken scheinen aktuell zu sein — keine bekannten Schwachstellen."
        }));
    }

    builder
}
