//! Module detail renderers (performance, SEO, security, mobile, dark mode, AI visibility).

use renderreport::components::advanced::{KeyValueList, List, MetricStrip, MetricStripItem};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, Finding, ScoreCard, SummaryBox, TableColumn};
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

    builder = builder.add_component(
        Section::new(if is_en(i18n) {
            "Performance budget violations"
        } else {
            "Performance-Budget-Verletzungen"
        })
        .with_level(2),
    );

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
        builder.add_component(Callout::warning(&summary_text).with_title(if is_en(i18n) {
            "Budget exceeded"
        } else {
            "Budget überschritten"
        }))
    } else {
        builder.add_component(Callout::info(&summary_text).with_title(if is_en(i18n) {
            "Budget notes"
        } else {
            "Budget-Hinweise"
        }))
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new(if is_en(i18n) { "Metric" } else { "Metrik" }),
        TableColumn::new("Budget"),
        TableColumn::new(if is_en(i18n) { "Actual" } else { "Ist-Wert" }),
        TableColumn::new(if is_en(i18n) {
            "Exceeded by"
        } else {
            "Überschreitung"
        }),
        TableColumn::new(if is_en(i18n) {
            "Severity"
        } else {
            "Schweregrad"
        }),
    ])
    .with_title(if is_en(i18n) {
        "Budget details"
    } else {
        "Budget-Details"
    });

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
    builder = builder
        .add_component(Section::new("Performance").with_level(2))
        .add_component(TextBlock::new(&perf.interpretation))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Performance score"
                } else {
                    "Performance Score"
                },
                perf.score,
            )
            .with_description(format!("Grade: {}", perf.grade))
            .with_thresholds(75, 50),
        );

    // ── User-perceived Performance (Core Web Vitals) ─────────────────
    if !perf.vitals.is_empty() {
        builder = builder.add_component(
            Section::new(if is_en(i18n) {
                "User Experience"
            } else {
                "Nutzererlebnis"
            })
            .with_level(3),
        );

        let strip = perf
            .vitals
            .iter()
            .take(4)
            .map(|(name, value, rating)| {
                MetricStripItem::new(name.replace(" (", "\n("), value)
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

    // ── Technical Complexity ─────────────────────────────────────────
    if !perf.additional_metrics.is_empty() || perf.has_render_blocking {
        builder = builder.add_component(
            Section::new(if is_en(i18n) {
                "Technical Complexity"
            } else {
                "Technische Komplexität"
            })
            .with_level(3),
        );

        if !perf.additional_metrics.is_empty() {
            let mut metrics = SummaryBox::new(if is_en(i18n) {
                "Technical indicators"
            } else {
                "Technische Indikatoren"
            });
            for (k, v) in &perf.additional_metrics {
                metrics = metrics.add_item(k, v);
            }
            builder = builder.add_component(metrics);
        }

        if perf.has_render_blocking {
            if !perf.render_blocking_metrics.is_empty() {
                let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
                    "Render-blocking analysis"
                } else {
                    "Render-Blocking Analyse"
                });
                for (k, v) in &perf.render_blocking_metrics {
                    kv = kv.add(k, v);
                }
                builder = builder.add_component(kv);
            }

            if !perf.render_blocking_suggestions.is_empty() {
                let mut suggestions = List::new().with_title(if is_en(i18n) {
                    "Recommendations"
                } else {
                    "Empfehlungen"
                });
                for s in &perf.render_blocking_suggestions {
                    suggestions = suggestions.add_item(s);
                }
                builder = builder.add_component(suggestions);
            }
        }
    }

    // ── Improvement suggestions (across both layers) ─────────────────
    if !perf.recommendations.is_empty() {
        let mut rec_list = List::new().with_title(if is_en(i18n) {
            "Improvement suggestions"
        } else {
            "Verbesserungsvorschläge"
        });
        for recommendation in &perf.recommendations {
            rec_list = rec_list.add_item(recommendation);
        }
        builder = builder.add_component(rec_list);
    }

    builder
}

pub(super) fn render_seo(
    mut builder: renderreport::engine::ReportBuilder,
    seo: &SeoPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(
            Section::new(if is_en(i18n) {
                "SEO analysis"
            } else {
                "SEO-Analyse"
            })
            .with_level(2),
        )
        .add_component(TextBlock::new(&seo.interpretation))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Technical SEO"
                } else {
                    "Technisches SEO"
                },
                seo.score,
            )
            .with_description(if is_en(i18n) {
                "Measures technical signals (meta, structure, schema, hreflang). Content depth is evaluated separately."
            } else {
                "Misst technische Signale (Meta, Struktur, Schema, hreflang). Inhaltliche Tiefe wird separat bewertet."
            })
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
            TableColumn::new(if is_en(i18n) {
                "Severity"
            } else {
                "Schweregrad"
            }),
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

    builder = builder
        .add_component(TextBlock::new(&seo.heading_summary))
        .add_component(TextBlock::new(&seo.social_summary));

    if !seo.tracking_summary.is_empty() {
        let mut tracking_table = AuditTable::new(vec![
            TableColumn::new("Signal").with_width("32%"),
            TableColumn::new("Status").with_width("68%"),
        ])
        .with_title(if is_en(i18n) {
            "Tracking and external services"
        } else {
            "Tracking und externe Dienste"
        });
        for (k, v) in &seo.tracking_summary {
            tracking_table = tracking_table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder
            .add_component(
                Callout::info(&seo.tracking_summary_text).with_title(if is_en(i18n) {
                    "Classification"
                } else {
                    "Einordnung"
                }),
            )
            .add_component(tracking_table);
    }

    if !seo.technical_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "Technical SEO"
        } else {
            "Technisches SEO"
        });
        for (k, v) in &seo.technical_summary {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
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

    builder
}

pub(super) fn render_serp(
    mut builder: renderreport::engine::ReportBuilder,
    serp: &crate::output::report_model::SerpPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        Section::new(if is_en(i18n) {
            "SERP analysis"
        } else {
            "SERP-Analyse"
        })
        .with_level(3),
    );

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
    let serp_readiness_title = if is_en(i18n) {
        "SERP readiness"
    } else {
        "SERP-Bereitschaft"
    };
    builder = if serp.fail_count > 0 {
        builder.add_component(Callout::warning(&summary).with_title(serp_readiness_title))
    } else if serp.warning_count > 0 {
        builder.add_component(Callout::info(&summary).with_title(serp_readiness_title))
    } else {
        builder.add_component(Callout::success(&summary).with_title(serp_readiness_title))
    };

    if !serp.signals.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Category" } else { "Kategorie" }).with_width("22%"),
            TableColumn::new("Signal").with_width("28%"),
            TableColumn::new("Status").with_width("14%"),
            TableColumn::new("Detail").with_width("36%"),
        ])
        .with_title(if is_en(i18n) {
            "SERP signals"
        } else {
            "SERP-Signale"
        });
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
    builder = builder.add_component(
        Section::new(if is_en(i18n) {
            "Page health"
        } else {
            "Seitengesundheit"
        })
        .with_level(3),
    );

    // Issues table (if any)
    if !ph.issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Issue" } else { "Problem" }).with_width("55%"),
            TableColumn::new(if is_en(i18n) {
                "Severity"
            } else {
                "Schweregrad"
            })
            .with_width("25%"),
        ])
        .with_title(if is_en(i18n) {
            "Detected issues"
        } else {
            "Gefundene Probleme"
        });
        for (_, msg, sev) in &ph.issues {
            table = table.add_row(vec![msg.as_str(), sev.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // URL info KV
    if !ph.url_info.is_empty() {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "URL analysis"
        } else {
            "URL-Analyse"
        });
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
            TableColumn::new(if is_en(i18n) {
                "Severity"
            } else {
                "Schweregrad"
            })
            .with_width("20%"),
            TableColumn::new("Detail").with_width("25%"),
        ])
        .with_title(if is_en(i18n) {
            "HTML validation"
        } else {
            "HTML-Validierung"
        });
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
    builder = builder.add_component(
        Section::new(if is_en(i18n) {
            "robots.txt audit"
        } else {
            "robots.txt Audit"
        })
        .with_level(3),
    );

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
    builder = builder.add_component(
        Section::new(if is_en(i18n) {
            "SEO content profile"
        } else {
            "SEO-Inhaltsprofil"
        })
        .with_level(3),
    );

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
            Callout::info(&profile.identity_summary).with_title(if is_en(i18n) {
                "Classification"
            } else {
                "Einordnung"
            }),
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
    let mut maturity = SummaryBox::new(if is_en(i18n) {
        "SEO maturity"
    } else {
        "SEO-Reifegrad"
    });
    maturity = maturity.add_item("Level", &profile.maturity_level);
    maturity = maturity.add_item(
        if is_en(i18n) { "Rating" } else { "Bewertung" },
        &profile.maturity_description,
    );
    maturity = maturity.add_item(
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
        .add_component(
            Section::new(if is_en(i18n) {
                "Security"
            } else {
                "Sicherheit"
            })
            .with_level(2),
        )
        .add_component(TextBlock::new(&sec.interpretation))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Security score"
                } else {
                    "Security Score"
                },
                sec.score,
            )
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

    for (title, sev, msg) in &sec.issues {
        builder = builder.add_component(Finding::new(title, map_severity(sev), msg));
    }

    if !sec.recommendations.is_empty() {
        let mut rec_list = List::new().with_title(if is_en(i18n) {
            "Improvement suggestions"
        } else {
            "Verbesserungsvorschläge"
        });
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
        .add_component(
            Section::new(if is_en(i18n) {
                "Mobile usability"
            } else {
                "Mobile Nutzbarkeit"
            })
            .with_level(2),
        )
        .add_component(TextBlock::new(&mobile.interpretation))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Mobile score"
                } else {
                    "Mobile Score"
                },
                mobile.score,
            )
            .with_thresholds(80, 50),
        );

    let viewport_status = mobile
        .viewport
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("viewport"))
        .map(|(_, v)| v.as_str())
        .unwrap_or(if is_en(i18n) {
            "Configured"
        } else {
            "Konfiguriert"
        });
    let touch_targets = mobile
        .touch_targets
        .iter()
        .find(|(k, _)| k.to_lowercase().contains("zu klein") || k.to_lowercase().contains("small"))
        .map(|(_, v)| v.as_str())
        .unwrap_or("n/a");
    builder = builder.add_component(
        MetricStrip::new(vec![
            MetricStripItem::new("Viewport", viewport_status).with_accent("#0f766e"),
            MetricStripItem::new(
                if is_en(i18n) {
                    "Touch targets"
                } else {
                    "Touch Targets"
                },
                touch_targets,
            )
            .with_accent("#d97706"),
            MetricStripItem::new("Issues", mobile.issues.len().to_string()).with_accent("#dc2626"),
        ])
        .compact(),
    );

    if !mobile.viewport.is_empty() {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "Viewport configuration"
        } else {
            "Viewport-Konfiguration"
        });
        for (k, v) in &mobile.viewport {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.touch_targets.is_empty() {
        let mut box_ = SummaryBox::new(if is_en(i18n) {
            "Touch targets"
        } else {
            "Touch Targets"
        });
        for (k, v) in &mobile.touch_targets {
            box_ = box_.add_item(k, v);
        }
        builder = builder.add_component(box_);
    }
    if !mobile.font_analysis.is_empty() {
        let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
            "Font analysis"
        } else {
            "Schriftanalyse"
        });
        for (k, v) in &mobile.font_analysis {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }
    if !mobile.content_sizing.is_empty() {
        let mut box_ = SummaryBox::new(if is_en(i18n) {
            "Content sizing"
        } else {
            "Content Sizing"
        });
        for (k, v) in &mobile.content_sizing {
            box_ = box_.add_item(k, v);
        }
        builder = builder.add_component(box_);
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
        .add_component(Section::new("User Experience").with_level(2))
        .add_component(TextBlock::new(&ux.interpretation))
        .add_component(
            ScoreCard::new(if is_en(i18n) { "UX score" } else { "UX Score" }, ux.score)
                .with_thresholds(80, 50),
        );

    // Dimension scores as KeyValueList
    let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
        "UX dimensions"
    } else {
        "UX-Dimensionen"
    });
    for dim in &ux.dimensions {
        kv = kv.add(&dim.name, format!("{}/100 — {}", dim.score, dim.summary));
    }
    builder = builder.add_component(kv);

    // Issues as findings
    for issue in &ux.issues {
        let sev = map_severity(&match issue.severity.as_str() {
            "high" => crate::taxonomy::Severity::High,
            "medium" => crate::taxonomy::Severity::Medium,
            "low" => crate::taxonomy::Severity::Low,
            _ => crate::taxonomy::Severity::Medium,
        });
        let desc = format!("{} — {}", issue.impact, issue.recommendation);
        builder = builder.add_component(Finding::new(&issue.dimension, sev, &desc));
    }
    builder
}

pub(super) fn render_journey(
    mut builder: renderreport::engine::ReportBuilder,
    journey: &JourneyPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(Section::new("User Journey").with_level(2))
        .add_component(TextBlock::new(&journey.interpretation))
        .add_component(
            ScoreCard::new(
                if is_en(i18n) {
                    "Journey score (indicator)"
                } else {
                    "Journey Score (Indikator)"
                },
                journey.score,
            )
            .with_description(if is_en(i18n) {
                "Heuristic estimate based on structural signals"
            } else {
                "Heuristische Schätzung auf Basis struktureller Signale"
            })
            .with_thresholds(80, 50),
        );

    // Page intent
    let mut kv = KeyValueList::new().with_title(if is_en(i18n) {
        "Page type & dimensions"
    } else {
        "Seitentyp & Dimensionen"
    });
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

    // Friction points as findings
    for fp in &journey.friction_points {
        let sev = map_severity(&match fp.severity.as_str() {
            "high" => crate::taxonomy::Severity::High,
            "medium" => crate::taxonomy::Severity::Medium,
            "low" => crate::taxonomy::Severity::Low,
            _ => crate::taxonomy::Severity::Medium,
        });
        let desc = format!("[{}] {} — {}", fp.step, fp.impact, fp.recommendation);
        builder = builder.add_component(Finding::new(&fp.problem, sev, &desc));
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
        .add_component(
            Section::new(if is_en(i18n) {
                "Dark mode"
            } else {
                "Dark Mode"
            })
            .with_level(2),
        )
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
        .add_component(
            Section::new(if is_en(i18n) {
                "Source quality"
            } else {
                "Quellenqualität"
            })
            .with_level(2),
        )
        .add_component(Callout::info(&sq.disclaimer).with_title(if is_en(i18n) {
            "Note"
        } else {
            "Hinweis"
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

    for dim in [&sq.substance, &sq.consistency, &sq.authority] {
        builder = builder.add_component(Section::new(&dim.name).with_level(3));

        builder = builder.add_component(
            ScoreCard::new(&dim.name, dim.score)
                .with_description(&dim.label)
                .with_thresholds(70, 50),
        );

        if !dim.signals.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Signal"),
                TableColumn::new("Status"),
                TableColumn::new("Detail"),
            ])
            .with_title(&dim.name);

            for signal in &dim.signals {
                let status = if signal.present { "✓" } else { "✗" };
                table = table.add_row(vec![&signal.name, status, &signal.detail]);
            }
            builder = builder.add_component(table);
        }
    }

    builder
}

// ─── AI Visibility ──────────────────────────────────────────────────────────

pub(super) fn render_ai_visibility(
    mut builder: renderreport::engine::ReportBuilder,
    av: &crate::ai_visibility::AiVisibilityAnalysis,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(
            Section::new(if is_en(i18n) {
                "AI visibility"
            } else {
                "AI-Sichtbarkeit"
            })
            .with_level(2),
        )
        .add_component(Callout::info(&av.disclaimer).with_title(if is_en(i18n) {
            "Note"
        } else {
            "Hinweis"
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

    // Render each dimension
    for (dim, title) in [
        (
            &av.readability.dimension,
            if is_en(i18n) {
                "LLM readability"
            } else {
                "LLM-Lesbarkeit"
            },
        ),
        (
            &av.citation.dimension,
            if is_en(i18n) {
                "Citability"
            } else {
                "Zitatfähigkeit"
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
        builder = builder.add_component(
            ScoreCard::new(&dim.name, dim.score)
                .with_description(&dim.label)
                .with_thresholds(70, 50),
        );

        if !dim.signals.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Signal"),
                TableColumn::new("Status"),
                TableColumn::new("Detail"),
            ])
            .with_title(&dim.name);

            for signal in &dim.signals {
                let status = if signal.present { "✓" } else { "✗" };
                table = table.add_row(vec![&signal.name, status, &signal.detail]);
            }
            builder = builder.add_component(table);
        }
    }

    // Chunk sections summary
    if !av.chunks.sections.is_empty() {
        builder = builder.add_component(
            Section::new(if is_en(i18n) {
                "Content sections"
            } else {
                "Content-Abschnitte"
            })
            .with_level(3),
        );

        let mut table = AuditTable::new(vec![
            TableColumn::new(if is_en(i18n) { "Section" } else { "Abschnitt" }),
            TableColumn::new("Level"),
            TableColumn::new(if is_en(i18n) { "Words" } else { "Wörter" }),
            TableColumn::new(if is_en(i18n) { "Quality" } else { "Qualität" }),
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
                &section.quality.to_string(),
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
        builder = builder.add_component(
            Section::new(if is_en(i18n) {
                "Detected entities"
            } else {
                "Erkannte Entitäten"
            })
            .with_level(3),
        );

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
        builder = builder.add_component(
            Section::new(if is_en(i18n) {
                "Link suggestions"
            } else {
                "Verlinkungsvorschläge"
            })
            .with_level(3),
        );

        let mut list = List::new().with_title(if is_en(i18n) {
            "Link suggestions"
        } else {
            "Verlinkungsvorschläge"
        });
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
        builder = builder.add_component(if is_en(i18n) {
            Callout::info(format!(
                "Policy: {} — AI search bots are blocked. \
                 Content will not appear in AI-generated answers.",
                av.policy.inferred_policy
            ))
            .with_title("AI visibility limited")
        } else {
            Callout::info(format!(
                "Policy: {} — KI-Suchbots sind blockiert. \
                 Inhalte erscheinen nicht in KI-generierten Antworten.",
                av.policy.inferred_policy
            ))
            .with_title("KI-Sichtbarkeit eingeschränkt")
        });
    } else if av.policy.blocks_ai_training {
        builder = builder.add_component(if is_en(i18n) {
            Callout::info(format!(
                "Policy: {} — AI training bots blocked, AI search bots have access. \
                 This is the recommended default configuration.",
                av.policy.inferred_policy
            ))
            .with_title("AI policy: default")
        } else {
            Callout::info(format!(
                "Policy: {} — KI-Trainingsbots blockiert, KI-Suchbots haben Zugang. \
                 Das ist die empfohlene Standardkonfiguration.",
                av.policy.inferred_policy
            ))
            .with_title("KI-Policy: Standard")
        });
    }

    builder
}
