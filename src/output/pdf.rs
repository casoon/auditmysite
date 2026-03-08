//! PDF Report Generator using renderreport/Typst
//!
//! Generates professional, customer-facing PDF reports for WCAG accessibility audits.
//! Uses the presentation model from report_builder for structured, grouped output.

use renderreport::components::advanced::{KeyValueList, List, PageBreak, TableOfContents};
use renderreport::components::charts::{Chart, Gauge};
use renderreport::components::text::{Label, Text};
use renderreport::prelude::*;
use renderreport::theme::{Theme, TokenValue};
use renderreport::Engine;

// New composite components (re-exported via prelude from components::standard)
use renderreport::components::{
    ActionRoadmap, DashboardModule, HeroMetric, HeroSummary, ModuleDashboard, RoadmapColumn,
    RoadmapItem,
};

use crate::audit::{AuditReport, BatchReport};
use crate::cli::ReportLevel;
use crate::output::report_builder::{build_batch_presentation, build_single_presentation};
use crate::output::report_model::*;
use crate::util::truncate_url;

/// Create engine with proper font configuration for German text
fn create_engine() -> anyhow::Result<Engine> {
    let mut engine = Engine::new()?;

    // Override default theme to use system fonts that support German characters
    let mut theme = Theme::default_theme();
    theme.tokens.set("font.body", TokenValue::Font("Helvetica Neue".into()));
    theme.tokens.set("font.heading", TokenValue::Font("Helvetica Neue".into()));
    theme.tokens.set("font.mono", TokenValue::Font("Menlo".into()));
    engine.set_default_theme(theme);

    Ok(engine)
}

/// Map our severity to renderreport severity
fn map_severity(severity: &crate::wcag::Severity) -> Severity {
    match severity {
        crate::wcag::Severity::Critical => Severity::Critical,
        crate::wcag::Severity::Serious => Severity::High,
        crate::wcag::Severity::Moderate => Severity::Medium,
        crate::wcag::Severity::Minor => Severity::Low,
    }
}

// ─── Single Report ──────────────────────────────────────────────────────────

/// Generate PDF report for a single audit
fn validate_presentation(pres: &ReportPresentation) {
    // Score consistency check
    let card_score = pres.score_breakdown.accessibility.score;
    let verdict_score = pres.brief_verdict.score.round() as u32;
    if card_score != verdict_score {
        tracing::warn!(
            "Score mismatch: score_breakdown={}, verdict={}",
            card_score,
            verdict_score
        );
    }

    // Severity validity check
    let valid = ["Critical", "Serious", "Moderate", "Minor"];
    for v in &pres.appendix.violations {
        if !valid.contains(&v.severity.as_str()) {
            tracing::warn!("Invalid severity '{}' for rule {}", v.severity, v.rule);
        }
    }
}

pub fn generate_pdf(report: &AuditReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let pres = build_single_presentation(report);
    validate_presentation(&pres);

    let author = config
        .company_name
        .as_deref()
        .unwrap_or("AuditMySit");

    let mut builder = engine
        .report("wcag-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", author)
        .metadata("score", &format!("{:.0}/100", pres.brief_verdict.score));

    // Logo on cover page
    if let Some(ref logo_path) = config.logo_path {
        if logo_path.exists() {
            builder = builder.add_component(
                Image::new(logo_path.to_string_lossy().to_string())
                    .with_width("30%"),
            );
        }
    }

    // Cover page break + Table of Contents
    builder = builder.add_component(PageBreak::new());
    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    // ── 1. Kurzfazit (Hero Summary) ──────────────────────────────────────
    // Compute KPI metrics
    let best_module = find_best_module(&pres.score_breakdown);
    let quick_win_count = pres.action_plan.quick_wins.len();

    let hero = HeroSummary::new(
        pres.brief_verdict.score.round() as u32,
        &pres.brief_verdict.grade,
        &pres.cover.url,
    )
    .with_date(&pres.cover.date)
    .with_verdict(&pres.brief_verdict.verdict_text)
    .add_metric(HeroMetric {
        title: "Verstöße gesamt".into(),
        value: pres.brief_verdict.total_violations.to_string(),
        accent_color: None,
    })
    .add_metric(HeroMetric {
        title: "Kritisch".into(),
        value: pres.brief_verdict.critical_count.to_string(),
        accent_color: Some("#ef4444".into()),
    })
    .add_metric(HeroMetric {
        title: "Stärkstes Modul".into(),
        value: best_module,
        accent_color: Some("#22c55e".into()),
    })
    .add_metric(HeroMetric {
        title: "Quick Wins".into(),
        value: quick_win_count.to_string(),
        accent_color: Some("#2563eb".into()),
    })
    .with_top_actions(pres.brief_verdict.top_actions.clone())
    .with_positive_aspects(
        pres.positive_aspects
            .iter()
            .map(|a| format!("{}: {}", a.area, a.description))
            .collect(),
    )
    .with_thresholds(70, 50);

    builder = builder
        .add_component(Section::new("Kurzfazit").with_level(1))
        .add_component(hero);

    builder = builder.add_component(PageBreak::new());

    // ── 2. Methodik ─────────────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Prüfumfang und Methodik").with_level(1))
        .add_component(Text::new(&pres.methodology.scope))
        .add_component(Text::new(&pres.methodology.method))
        .add_component(
            Callout::info(&pres.methodology.limitations)
                .with_title("Grenzen automatisierter Tests"),
        )
        .add_component(
            Callout::warning(&pres.methodology.disclaimer)
                .with_title("Hinweis"),
        );

    // Executive level: skip to score breakdown + action plan
    if config.level == ReportLevel::Executive {
        // Compact score overview
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Bewertung nach Modulen").with_level(1));

        {
            let mut scores: Vec<(String, f64)> = vec![
                ("Accessibility".to_string(), pres.score_breakdown.accessibility.score as f64),
            ];
            if let Some(ref sd) = pres.score_breakdown.performance {
                scores.push(("Performance".to_string(), sd.score as f64));
            }
            if let Some(ref sd) = pres.score_breakdown.seo {
                scores.push(("SEO".to_string(), sd.score as f64));
            }
            if let Some(ref sd) = pres.score_breakdown.security {
                scores.push(("Sicherheit".to_string(), sd.score as f64));
            }
            if let Some(ref sd) = pres.score_breakdown.mobile {
                scores.push(("Mobile".to_string(), sd.score as f64));
            }
            if scores.len() > 1 {
                builder = builder.add_component(
                    Chart::bar("Modul-Scores im Vergleich")
                        .add_series("Score", scores)
                        .with_labels("Modul", "Punkte"),
                );
            }
        }

        // Action plan
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Maßnahmenplan").with_level(1))
            .add_component(Text::new(
                "Auf Basis der identifizierten Probleme empfehlen wir die folgenden Maßnahmen.",
            ));
        builder = builder.add_component(build_action_roadmap(&pres.action_plan));

        let built_report = builder.build();
        let pdf_bytes = engine.render_pdf(&built_report)?;
        return Ok(pdf_bytes);
    }

    builder = builder.add_component(PageBreak::new());

    // ── 3. Score Breakdown (Module Dashboard) ──────────────────────────
    {
        let mut modules = vec![DashboardModule {
            name: "Barrierefreiheit".into(),
            score: pres.score_breakdown.accessibility.score,
            interpretation: pres.score_breakdown.accessibility.interpretation.clone(),
            good_threshold: 75,
            warn_threshold: 50,
        }];
        if let Some(ref sd) = pres.score_breakdown.performance {
            modules.push(DashboardModule {
                name: "Performance".into(),
                score: sd.score,
                interpretation: sd.interpretation.clone(),
                good_threshold: 75,
                warn_threshold: 50,
            });
        }
        if let Some(ref sd) = pres.score_breakdown.seo {
            modules.push(DashboardModule {
                name: "SEO".into(),
                score: sd.score,
                interpretation: sd.interpretation.clone(),
                good_threshold: 75,
                warn_threshold: 50,
            });
        }
        if let Some(ref sd) = pres.score_breakdown.security {
            modules.push(DashboardModule {
                name: "Sicherheit".into(),
                score: sd.score,
                interpretation: sd.interpretation.clone(),
                good_threshold: 75,
                warn_threshold: 50,
            });
        }
        if let Some(ref sd) = pres.score_breakdown.mobile {
            modules.push(DashboardModule {
                name: "Mobile".into(),
                score: sd.score,
                interpretation: sd.interpretation.clone(),
                good_threshold: 75,
                warn_threshold: 50,
            });
        }

        builder = builder
            .add_component(Section::new("Bewertung nach Modulen").with_level(1))
            .add_component(ModuleDashboard::new(modules));

        // Keep bar chart if multiple modules for visual comparison
        let mut scores: Vec<(String, f64)> = vec![(
            "Accessibility".to_string(),
            pres.score_breakdown.accessibility.score as f64,
        )];
        if let Some(ref sd) = pres.score_breakdown.performance {
            scores.push(("Performance".to_string(), sd.score as f64));
        }
        if let Some(ref sd) = pres.score_breakdown.seo {
            scores.push(("SEO".to_string(), sd.score as f64));
        }
        if let Some(ref sd) = pres.score_breakdown.security {
            scores.push(("Sicherheit".to_string(), sd.score as f64));
        }
        if let Some(ref sd) = pres.score_breakdown.mobile {
            scores.push(("Mobile".to_string(), sd.score as f64));
        }
        if scores.len() > 1 {
            builder = builder.add_component(
                Chart::bar("Modul-Scores im Vergleich")
                    .add_series("Score", scores)
                    .with_labels("Modul", "Punkte"),
            );
        }
    }

    // ── 4. Findings (differentiated by report level) ───────────────────
    // Severity summary
    let severity_summary = build_severity_summary(&pres.accessibility_details);

    match config.level {
        ReportLevel::Technical => {
            // Brief overview
            builder = builder
                .add_component(PageBreak::new())
                .add_component(Section::new("Übersicht der Probleme").with_level(1));

            if pres.accessibility_details.is_empty() {
                builder = builder.add_component(
                    Callout::success(
                        "Keine automatisch erkennbaren Barrierefreiheitsprobleme gefunden.",
                    )
                    .with_title("Ausgezeichnete Barrierefreiheit"),
                );
            } else {
                builder = builder.add_component(severity_summary.clone());
                for group in pres.top_findings.iter().take(5) {
                    builder = render_finding_compact(builder, group);
                }
            }

            // Full technical detail
            if !pres.accessibility_details.is_empty() {
                builder = builder
                    .add_component(PageBreak::new())
                    .add_component(Section::new("Technische Detailanalyse").with_level(1));
                for group in &pres.accessibility_details {
                    builder = render_finding_technical(builder, group);
                }
            }
        }
        _ => {
            // Standard: all findings in compact form
            builder = builder
                .add_component(PageBreak::new())
                .add_component(Section::new("Erkannte Probleme").with_level(1));

            if pres.accessibility_details.is_empty() {
                builder = builder.add_component(
                    Callout::success(
                        "Keine automatisch erkennbaren Barrierefreiheitsprobleme gefunden.",
                    )
                    .with_title("Ausgezeichnete Barrierefreiheit"),
                );
            } else {
                builder = builder.add_component(severity_summary);
                for group in &pres.accessibility_details {
                    builder = render_finding_compact(builder, group);
                }
            }
        }
    }

    // ── 5. Module Details ───────────────────────────────────────────────
    if pres.module_details.performance.is_some()
        || pres.module_details.seo.is_some()
        || pres.module_details.security.is_some()
        || pres.module_details.mobile.is_some()
    {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Weitere Analysen").with_level(1))
            .add_component(Text::new(
                "Neben der Barrierefreiheit wurden weitere Aspekte der Website analysiert. \
                 Die folgenden Abschnitte zeigen die Ergebnisse der Performance-, SEO-, \
                 Sicherheits- und Mobile-Analyse im Detail.",
            ));
    }

    if let Some(ref perf) = pres.module_details.performance {
        builder = builder
            .add_component(Section::new("Performance").with_level(2))
            .add_component(Text::new(&perf.interpretation))
            .add_component(
                ScoreCard::new("Performance Score", perf.score)
                    .with_description(&format!("Grade: {}", perf.grade))
                    .with_thresholds(75, 50),
            );

        if !perf.vitals.is_empty() {
            let mut kv = KeyValueList::new().with_title("Core Web Vitals");
            for (name, value, rating) in &perf.vitals {
                kv = kv.add(name, &format!("{} — {}", value, rating));
            }
            builder = builder.add_component(kv);
        }

        if !perf.additional_metrics.is_empty() {
            let mut metrics = SummaryBox::new("Weitere Metriken");
            for (k, v) in &perf.additional_metrics {
                metrics = metrics.add_item(k, v);
            }
            builder = builder.add_component(metrics);
        }
    }

    if let Some(ref seo) = pres.module_details.seo {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("SEO-Analyse").with_level(2))
            .add_component(Text::new(&seo.interpretation))
            .add_component(
                ScoreCard::new("SEO Score", seo.score)
                    .with_thresholds(80, 50),
            );

        if !seo.meta_tags.is_empty() {
            let mut kv = KeyValueList::new().with_title("Meta-Tags");
            for (k, v) in &seo.meta_tags {
                kv = kv.add(k, v);
            }
            builder = builder.add_component(kv);
        }

        if !seo.meta_issues.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Feld"),
                TableColumn::new("Schweregrad"),
                TableColumn::new("Beschreibung"),
            ])
            .with_title("Meta-Tag Probleme");
            for (field, sev, msg) in &seo.meta_issues {
                table = table.add_row(vec![field.as_str(), sev.as_str(), msg.as_str()]);
            }
            builder = builder.add_component(table);
        }

        builder = builder.add_component(Text::new(&seo.heading_summary));
        builder = builder.add_component(Text::new(&seo.social_summary));

        if !seo.technical_summary.is_empty() {
            let mut kv = KeyValueList::new().with_title("Technisches SEO");
            for (k, v) in &seo.technical_summary {
                kv = kv.add(k, v);
            }
            builder = builder.add_component(kv);
        }
    }

    if let Some(ref sec) = pres.module_details.security {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Sicherheit").with_level(2))
            .add_component(Text::new(&sec.interpretation))
            .add_component(
                ScoreCard::new("Security Score", sec.score)
                    .with_description(&format!("Grade: {}", sec.grade))
                    .with_thresholds(70, 50),
            );

        if !sec.headers.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new("Header"),
                TableColumn::new("Status"),
                TableColumn::new("Wert"),
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
            let severity = match sev.as_str() {
                "critical" => Severity::Critical,
                "high" => Severity::High,
                "medium" => Severity::Medium,
                _ => Severity::Low,
            };
            builder = builder.add_component(Finding::new(title, severity, msg));
        }

        if !sec.recommendations.is_empty() {
            let mut rec_list = List::new().with_title("Empfehlungen");
            for rec in &sec.recommendations {
                rec_list = rec_list.add_item(rec);
            }
            builder = builder.add_component(rec_list);
        }
    }

    if let Some(ref mobile) = pres.module_details.mobile {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Mobile Nutzbarkeit").with_level(2))
            .add_component(Text::new(&mobile.interpretation))
            .add_component(
                ScoreCard::new("Mobile Score", mobile.score)
                    .with_thresholds(80, 50),
            );

        if !mobile.viewport.is_empty() {
            let mut kv = KeyValueList::new().with_title("Viewport-Konfiguration");
            for (k, v) in &mobile.viewport {
                kv = kv.add(k, v);
            }
            builder = builder.add_component(kv);
        }

        if !mobile.touch_targets.is_empty() {
            let mut box_ = SummaryBox::new("Touch Targets");
            for (k, v) in &mobile.touch_targets {
                box_ = box_.add_item(k, v);
            }
            builder = builder.add_component(box_);
        }

        if !mobile.font_analysis.is_empty() {
            let mut kv = KeyValueList::new().with_title("Schriftanalyse");
            for (k, v) in &mobile.font_analysis {
                kv = kv.add(k, v);
            }
            builder = builder.add_component(kv);
        }

        if !mobile.content_sizing.is_empty() {
            let mut box_ = SummaryBox::new("Content Sizing");
            for (k, v) in &mobile.content_sizing {
                box_ = box_.add_item(k, v);
            }
            builder = builder.add_component(box_);
        }

        for (cat, sev, msg) in &mobile.issues {
            let severity = match sev.as_str() {
                "critical" => Severity::Critical,
                "high" => Severity::High,
                "medium" => Severity::Medium,
                _ => Severity::Low,
            };
            builder = builder.add_component(Finding::new(cat, severity, msg));
        }
    }

    // ── 8. Maßnahmenplan (Roadmap) ────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Maßnahmenplan").with_level(1))
        .add_component(Text::new(
            "Auf Basis der identifizierten Probleme empfehlen wir die folgenden Maßnahmen, \
             gegliedert nach Aufwand und Wirkung.",
        ));

    builder = builder.add_component(build_action_roadmap(&pres.action_plan));

    // ── 9. Anhang ───────────────────────────────────────────────────────
    if !pres.appendix.violations.is_empty() {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Anhang: Technische Details").with_level(1))
            .add_component(Text::new(
                "Die folgende Tabelle enthält alle erkannten Verstöße mit \
                 technischen Details für die Umsetzung.",
            ))
            .add_component(Text::new(&pres.appendix.score_methodology));

        if config.level == ReportLevel::Technical {
            // Extended appendix: one Finding per violation with full details
            for v in &pres.appendix.violations {
                let mut desc = v.message.clone();
                if let Some(ref fix) = v.fix_suggestion {
                    desc.push_str(&format!("\n\nFix: {}", fix));
                }
                builder = builder.add_component(
                    Finding::new(
                        &format!("{} — {}", v.rule, v.rule_name),
                        match v.severity.to_lowercase().as_str() {
                            "critical" => Severity::Critical,
                            "serious" => Severity::High,
                            "moderate" => Severity::Medium,
                            _ => Severity::Low,
                        },
                        &desc,
                    )
                    .with_affected(&v.selector.clone().unwrap_or_else(|| v.node_id.clone())),
                );
            }
        } else {
            // Standard appendix: compact table
            let mut table = AuditTable::new(vec![
                TableColumn::new("Regel"),
                TableColumn::new("Schweregrad"),
                TableColumn::new("Beschreibung"),
                TableColumn::new("Element"),
            ])
            .with_title("Alle Verstöße");

            for v in &pres.appendix.violations {
                table = table.add_row(vec![
                    format!("{} — {}", v.rule, v.rule_name),
                    v.severity.clone(),
                    v.message.clone(),
                    v.selector.clone().unwrap_or_else(|| v.node_id.clone()),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    let built_report = builder.build();
    let pdf_bytes = engine.render_pdf(&built_report)?;
    Ok(pdf_bytes)
}

// ─── Batch Report ───────────────────────────────────────────────────────────

/// Generate PDF report for batch audits
fn validate_batch_presentation(pres: &BatchPresentation) {
    let valid = ["Critical", "Serious", "Moderate", "Minor"];
    for url_appendix in &pres.appendix.per_url {
        for v in &url_appendix.violations {
            if !valid.contains(&v.severity.as_str()) {
                tracing::warn!(
                    "Invalid severity '{}' for rule {} in {}",
                    v.severity,
                    v.rule,
                    url_appendix.url
                );
            }
        }
    }
}

pub fn generate_batch_pdf(batch: &BatchReport, config: &ReportConfig) -> anyhow::Result<Vec<u8>> {
    let engine = create_engine()?;
    let pres = build_batch_presentation(batch);
    validate_batch_presentation(&pres);

    let success_rate = if pres.portfolio_summary.total_urls > 0 {
        (pres.portfolio_summary.passed as f64 / pres.portfolio_summary.total_urls as f64) * 100.0
    } else {
        0.0
    };

    let author = config
        .company_name
        .as_deref()
        .unwrap_or("AuditMySit");

    let mut builder = engine
        .report("wcag-batch-audit")
        .title(&pres.cover.title)
        .subtitle(&pres.cover.url)
        .metadata("date", &pres.cover.date)
        .metadata("version", &pres.cover.version)
        .metadata("author", author);

    // Logo on cover page
    if let Some(ref logo_path) = config.logo_path {
        if logo_path.exists() {
            builder = builder.add_component(
                Image::new(logo_path.to_string_lossy().to_string())
                    .with_width("30%"),
            );
        }
    }

    // Cover page break + Table of Contents
    builder = builder.add_component(PageBreak::new());
    if config.level != ReportLevel::Executive {
        builder = builder.add_component(TableOfContents::new().with_depth(1));
    }

    // ── 1. Batch-Fazit ──────────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("Gesamtübersicht").with_level(1))
        .add_component(
            ScoreCard::new(
                "Durchschnittlicher Accessibility Score",
                pres.portfolio_summary.average_score.round() as u32,
            )
            .with_description(&format!(
                "{} URLs geprüft | {} bestanden | {} nicht bestanden | Erfolgsrate: {:.0}%",
                pres.portfolio_summary.total_urls,
                pres.portfolio_summary.passed,
                pres.portfolio_summary.failed,
                success_rate
            ))
            .with_thresholds(70, 50),
        )
        .add_component(Gauge::new("Durchschnitt", pres.portfolio_summary.average_score))
        .add_component(Text::new(&pres.portfolio_summary.verdict_text))
        .add_component(
            SummaryBox::new("Portfolio-Statistiken")
                .add_item("Geprüfte URLs", &pres.portfolio_summary.total_urls.to_string())
                .add_item("Bestanden", &pres.portfolio_summary.passed.to_string())
                .add_item("Nicht bestanden", &pres.portfolio_summary.failed.to_string())
                .add_item("Verstöße gesamt", &pres.portfolio_summary.total_violations.to_string())
                .add_item("Prüfdauer", &format!("{}ms", pres.portfolio_summary.duration_ms)),
        );

    // Severity distribution chart
    {
        let dist = &pres.portfolio_summary.severity_distribution;
        let total = dist.critical + dist.serious + dist.moderate + dist.minor;
        if total > 0 {
            builder = builder.add_component(
                Chart::pie("Verstöße nach Schweregrad")
                    .add_series(
                        "Schweregrad",
                        vec![
                            ("Kritisch".to_string(), dist.critical as f64),
                            ("Schwerwiegend".to_string(), dist.serious as f64),
                            ("Mäßig".to_string(), dist.moderate as f64),
                            ("Geringfügig".to_string(), dist.minor as f64),
                        ],
                    ),
            );
        }
    }

    // URL score comparison chart
    if pres.url_ranking.len() > 1 {
        let url_scores: Vec<(String, f64)> = pres
            .url_ranking
            .iter()
            .map(|u| (truncate_url(&u.url, 30), u.score as f64))
            .collect();
        builder = builder.add_component(
            Chart::bar("Scores im Vergleich")
                .add_series("Score", url_scores)
                .with_labels("URL", "Punkte"),
        );
    }

    // Worst / best URLs
    if !pres.portfolio_summary.worst_urls.is_empty() {
        let mut worst_list = List::new().with_title("Kritischste URLs");
        for (url, score) in &pres.portfolio_summary.worst_urls {
            worst_list = worst_list.add_item(&format!("{} — {:.0}/100", url, score));
        }
        builder = builder.add_component(worst_list);
    }

    builder = builder.add_component(PageBreak::new());

    // ── 2. URL-Ranking ──────────────────────────────────────────────────
    builder = builder
        .add_component(Section::new("URL-Ranking").with_level(1))
        .add_component(Text::new(
            "Übersicht aller geprüften URLs, sortiert nach Score (aufsteigend). \
             URLs mit niedrigerem Score haben höheren Handlungsbedarf.",
        ));

    let mut ranking_table = AuditTable::new(vec![
        TableColumn::new("URL"),
        TableColumn::new("Score"),
        TableColumn::new("Note"),
        TableColumn::new("Krit. Verstöße"),
        TableColumn::new("Gesamt"),
        TableColumn::new("Priorität"),
    ])
    .with_title("URL-Übersicht")
    ;

    for url in &pres.url_ranking {
        ranking_table = ranking_table.add_row(vec![
            truncate_url(&url.url, 45),
            format!("{:.0}", url.score),
            url.grade.clone(),
            url.critical_violations.to_string(),
            url.total_violations.to_string(),
            url.priority.label().to_string(),
        ]);
    }
    builder = builder.add_component(ranking_table);

    builder = builder.add_component(PageBreak::new());

    // ── 3. Top-Probleme über alle URLs ──────────────────────────────────
    builder = builder
        .add_component(Section::new("Häufigste Probleme").with_level(1))
        .add_component(Text::new(
            "Die folgenden Problemgruppen treten über mehrere URLs hinweg auf. \
             Durch Behebung dieser Probleme wird die größte Verbesserung erzielt, \
             da sie viele Seiten gleichzeitig betreffen.",
        ));

    // Frequency table
    if !pres.issue_frequency.is_empty() {
        let mut freq_table = AuditTable::new(vec![
            TableColumn::new("Problem"),
            TableColumn::new("WCAG"),
            TableColumn::new("Vorkommen"),
            TableColumn::new("Betr. URLs"),
            TableColumn::new("Priorität"),
        ])
        .with_title("Häufigste Verstöße")
        ;

        for issue in &pres.issue_frequency {
            freq_table = freq_table.add_row(vec![
                issue.problem.clone(),
                issue.wcag.clone(),
                issue.occurrences.to_string(),
                issue.affected_urls.to_string(),
                issue.priority.label().to_string(),
            ]);
        }
        builder = builder.add_component(freq_table);
    }

    // Detailed finding groups for top issues
    for group in &pres.top_issues {
        builder = render_finding_group(builder, group);
    }

    // ── 4. Maßnahmenplan ────────────────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Maßnahmenplan").with_level(1))
        .add_component(Text::new(
            "Die folgenden Maßnahmen sind nach Aufwand und Wirkung priorisiert. \
             Maßnahmen, die viele Seiten gleichzeitig verbessern, haben Vorrang.",
        ));

    builder = render_action_plan(builder, &pres.action_plan);

    // ── 5. Kompakte URL-Summaries ───────────────────────────────────────
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Einzelergebnisse je URL").with_level(1))
        .add_component(Text::new(
            "Kompakte Zusammenfassung der Ergebnisse pro geprüfter URL. \
             Detaillierte technische Daten finden sich im Anhang.",
        ));

    for detail in &pres.url_details {
        builder = builder
            .add_component(Section::new(&truncate_url(&detail.url, 70)).with_level(2))
            .add_component(
                ScoreCard::new("Score", detail.score.round() as u32)
                    .with_description(&format!("Note: {}", detail.grade))
                    .with_thresholds(70, 50),
            );

        if !detail.module_scores.is_empty() {
            let mut scores = SummaryBox::new("Modul-Scores");
            for (module, score) in &detail.module_scores {
                scores = scores.add_item(module, &format!("{}/100", score));
            }
            builder = builder.add_component(scores);
        }

        if !detail.top_issues.is_empty() {
            let mut issues = List::new().with_title("Wichtigste Probleme");
            for issue in &detail.top_issues {
                issues = issues.add_item(issue);
            }
            builder = builder.add_component(issues);
        }
    }

    // ── 6. Anhang ───────────────────────────────────────────────────────
    if !pres.appendix.per_url.is_empty() {
        builder = builder
            .add_component(PageBreak::new())
            .add_component(Section::new("Anhang: Technische Details").with_level(1))
            .add_component(Text::new(
                "Vollständige Auflistung aller erkannten Verstöße pro URL \
                 mit technischen Details für die Umsetzung.",
            ));

        for url_appendix in &pres.appendix.per_url {
            if url_appendix.violations.is_empty() {
                continue;
            }

            builder = builder.add_component(
                Section::new(&truncate_url(&url_appendix.url, 70)).with_level(2),
            );

            let mut table = AuditTable::new(vec![
                TableColumn::new("Regel"),
                TableColumn::new("Schweregrad"),
                TableColumn::new("Beschreibung"),
                TableColumn::new("Element"),
            ])
            ;

            for v in &url_appendix.violations {
                table = table.add_row(vec![
                    format!("{} — {}", v.rule, v.rule_name),
                    v.severity.clone(),
                    v.message.clone(),
                    v.selector.clone().unwrap_or_else(|| v.node_id.clone()),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    let built_report = builder.build();
    let pdf_bytes = engine.render_pdf(&built_report)?;
    Ok(pdf_bytes)
}

// ─── Shared rendering helpers ───────────────────────────────────────────────

/// Render a FindingGroup as a rich, multi-part section
fn render_finding_group(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
) -> renderreport::engine::ReportBuilder {
    // Section header with WCAG criterion
    builder = builder.add_component(
        Section::new(&format!("{} (WCAG {})", group.title, group.wcag_criterion)).with_level(2),
    );

    // Critical/serious findings get a prominent Callout, others use Finding component
    let is_critical = matches!(
        group.severity,
        crate::wcag::Severity::Critical | crate::wcag::Severity::Serious
    );

    if is_critical {
        builder = builder.add_component(
            Callout::error(&group.customer_description)
                .with_title(&format!(
                    "{} — Priorität: {}",
                    group.title,
                    group.priority.label()
                )),
        );
    }

    let mut finding = Finding::new(
        &group.title,
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(&format!(
        "Priorität: {} | Zuständig: {} | Aufwand: {}",
        group.priority.label(),
        group.responsible_role.label(),
        group.effort.label()
    ));

    if group.occurrence_count > 0 {
        finding = finding.with_affected(&format!(
            "{} Vorkommen, {} Elemente betroffen{}",
            group.occurrence_count,
            group.affected_elements,
            if group.affected_urls.is_empty() {
                String::new()
            } else {
                format!(", {} URLs", group.affected_urls.len())
            }
        ));
    }

    builder = builder.add_component(finding);

    // User impact
    if !group.user_impact.is_empty() {
        builder = builder.add_component(
            Callout::info(&group.user_impact).with_title("Auswirkung auf Nutzer"),
        );
    }

    // Typical cause
    if !group.typical_cause.is_empty() {
        builder = builder.add_component(Text::new(&format!(
            "Typische Ursache: {}",
            group.typical_cause
        )));
    }

    // Technical note
    if !group.technical_note.is_empty() {
        builder = builder.add_component(Text::new(&format!(
            "Technischer Hinweis: {}",
            group.technical_note
        )));
    }

    // Code examples with visual distinction
    if !group.examples.is_empty() {
        builder = builder.add_component(Section::new("Codebeispiel").with_level(3));
        for example in &group.examples {
            builder = builder.add_component(
                Callout::error(&example.bad).with_title("Falsch"),
            );
            builder = builder.add_component(
                Callout::success(&example.good).with_title("Richtig"),
            );
            if let Some(ref dec) = example.decorative {
                builder = builder.add_component(
                    Callout::info(dec).with_title("Dekorativ"),
                );
            }
        }
    }

    // Affected URLs (batch context)
    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title("Betroffene URLs");
        for url in &group.affected_urls {
            url_list = url_list.add_item(&truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    } else if group.affected_urls.len() > 10 {
        builder = builder.add_component(Text::new(&format!(
            "Betrifft {} URLs (zu viele für Einzelauflistung — siehe Anhang).",
            group.affected_urls.len()
        )));
    }

    builder
}

/// Compact finding renderer for Standard reports — one Finding component per group
fn render_finding_compact(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
) -> renderreport::engine::ReportBuilder {
    let finding = Finding::new(
        &format!("{} (WCAG {})", group.title, group.wcag_criterion),
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(&format!(
        "Priorität: {} | {} Vorkommen",
        group.priority.label(),
        group.occurrence_count
    ))
    .with_affected(&format!("{} Elemente betroffen", group.affected_elements));

    builder = builder.add_component(finding);
    builder
}

/// Technical finding renderer — full detail with code examples, no TOC pollution
fn render_finding_technical(
    mut builder: renderreport::engine::ReportBuilder,
    group: &FindingGroup,
) -> renderreport::engine::ReportBuilder {
    // Label instead of Section → no TOC entry
    builder = builder.add_component(
        Label::new(&format!(
            "{} — WCAG {} ({})",
            group.title, group.wcag_criterion, group.wcag_level
        ))
        .bold()
        .with_size("14pt"),
    );

    let finding = Finding::new(
        &group.title,
        map_severity(&group.severity),
        &group.customer_description,
    )
    .with_recommendation(&group.recommendation)
    .with_category(&format!(
        "Priorität: {} | Zuständig: {} | Aufwand: {}",
        group.priority.label(),
        group.responsible_role.label(),
        group.effort.label()
    ))
    .with_affected(&format!(
        "{} Vorkommen, {} Elemente",
        group.occurrence_count, group.affected_elements
    ));

    builder = builder.add_component(finding);

    // Technical note
    if !group.technical_note.is_empty() {
        builder = builder.add_component(
            Callout::info(&group.technical_note).with_title("Technischer Hinweis"),
        );
    }

    // Code examples
    for example in &group.examples {
        builder = builder
            .add_component(Callout::error(&example.bad).with_title("Falsch"))
            .add_component(Callout::success(&example.good).with_title("Richtig"));
        if let Some(ref dec) = example.decorative {
            builder = builder.add_component(Callout::info(dec).with_title("Dekorativ"));
        }
    }

    // Affected URLs (batch context)
    if !group.affected_urls.is_empty() && group.affected_urls.len() <= 10 {
        let mut url_list = List::new().with_title("Betroffene URLs");
        for url in &group.affected_urls {
            url_list = url_list.add_item(&truncate_url(url, 70));
        }
        builder = builder.add_component(url_list);
    }

    builder
}

/// Render the action plan section (used by batch reports)
fn render_action_plan(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
) -> renderreport::engine::ReportBuilder {
    // Quick Wins
    if !plan.quick_wins.is_empty() {
        builder = builder.add_component(Section::new("Quick Wins").with_level(2));

        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ])
        ;

        for item in &plan.quick_wins {
            table = table.add_row(vec![
                item.action.clone(),
                truncate_url(&item.benefit, 60),
                item.role.label().to_string(),
                item.priority.label().to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Medium-term
    if !plan.medium_term.is_empty() {
        builder = builder.add_component(Section::new("Mittelfristige Maßnahmen").with_level(2));

        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ])
        ;

        for item in &plan.medium_term {
            table = table.add_row(vec![
                item.action.clone(),
                truncate_url(&item.benefit, 60),
                item.role.label().to_string(),
                item.priority.label().to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    // Structural
    if !plan.structural.is_empty() {
        builder = builder.add_component(Section::new("Strukturelle Maßnahmen").with_level(2));

        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ])
        ;

        for item in &plan.structural {
            table = table.add_row(vec![
                item.action.clone(),
                truncate_url(&item.benefit, 60),
                item.role.label().to_string(),
                item.priority.label().to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

/// Find the best-scoring module name
fn find_best_module(breakdown: &ScoreBreakdown) -> String {
    let mut best = ("Barrierefreiheit", breakdown.accessibility.score);
    if let Some(ref sd) = breakdown.performance {
        if sd.score > best.1 {
            best = ("Performance", sd.score);
        }
    }
    if let Some(ref sd) = breakdown.seo {
        if sd.score > best.1 {
            best = ("SEO", sd.score);
        }
    }
    if let Some(ref sd) = breakdown.security {
        if sd.score > best.1 {
            best = ("Sicherheit", sd.score);
        }
    }
    if let Some(ref sd) = breakdown.mobile {
        if sd.score > best.1 {
            best = ("Mobile", sd.score);
        }
    }
    best.0.to_string()
}

/// Build severity summary box from finding groups
fn build_severity_summary(groups: &[FindingGroup]) -> SummaryBox {
    use crate::wcag::types::Severity as WcagSeverity;

    let critical = groups
        .iter()
        .filter(|g| matches!(g.severity, WcagSeverity::Critical))
        .count();
    let serious = groups
        .iter()
        .filter(|g| matches!(g.severity, WcagSeverity::Serious))
        .count();
    let moderate = groups
        .iter()
        .filter(|g| matches!(g.severity, WcagSeverity::Moderate))
        .count();
    let minor = groups
        .iter()
        .filter(|g| matches!(g.severity, WcagSeverity::Minor))
        .count();

    SummaryBox::new("Problemübersicht")
        .add_item_with_status(
            "Kritisch / Schwerwiegend",
            &format!("{}", critical + serious),
            if critical + serious > 0 {
                ScoreStatus::Bad
            } else {
                ScoreStatus::Good
            },
        )
        .add_item_with_status(
            "Moderat",
            &moderate.to_string(),
            if moderate > 0 {
                ScoreStatus::Warning
            } else {
                ScoreStatus::Good
            },
        )
        .add_item_with_status("Gering", &minor.to_string(), ScoreStatus::Good)
}

/// Build ActionRoadmap from action plan
fn build_action_roadmap(plan: &ActionPlan) -> ActionRoadmap {
    let map_items = |items: &[ActionItem]| -> Vec<RoadmapItem> {
        items
            .iter()
            .map(|i| RoadmapItem {
                action: i.action.clone(),
                role: i.role.label().to_string(),
                priority: i.priority.label().to_string(),
                benefit: i.benefit.clone(),
            })
            .collect()
    };

    let mut columns = Vec::new();

    if !plan.quick_wins.is_empty() {
        columns.push(RoadmapColumn {
            title: "Quick Wins".into(),
            accent_color: Some("#22c55e".into()),
            items: map_items(&plan.quick_wins),
        });
    }
    if !plan.medium_term.is_empty() {
        columns.push(RoadmapColumn {
            title: "Mittelfristig".into(),
            accent_color: Some("#f59e0b".into()),
            items: map_items(&plan.medium_term),
        });
    }
    if !plan.structural.is_empty() {
        columns.push(RoadmapColumn {
            title: "Strukturell".into(),
            accent_color: Some("#2563eb".into()),
            items: map_items(&plan.structural),
        });
    }

    ActionRoadmap::new(columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_url() {
        assert_eq!(
            truncate_url("https://example.com/very/long/path/that/exceeds/limit", 30),
            "https://example.com/very/lo..."
        );

        assert_eq!(
            truncate_url("https://example.com", 30),
            "https://example.com"
        );
    }
}
