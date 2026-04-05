//! Module detail renderers (performance, SEO, security, mobile, dark mode).

use renderreport::components::advanced::{KeyValueList, List, PageBreak};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, Finding, ScoreCard, SummaryBox, TableColumn};
use renderreport::prelude::*;

use crate::output::report_model::*;

use super::helpers::{map_severity, score_quality_color, score_quality_label};

pub(super) fn render_budget_violations(
    mut builder: renderreport::engine::ReportBuilder,
    violations: &[crate::audit::BudgetViolation],
) -> renderreport::engine::ReportBuilder {
    use crate::audit::BudgetSeverity;

    builder = builder.add_component(Section::new("Performance-Budget-Verletzungen").with_level(2));

    let error_count = violations
        .iter()
        .filter(|v| v.severity == BudgetSeverity::Error)
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| v.severity == BudgetSeverity::Warning)
        .count();

    let summary_text = format!(
        "{} Budget-Verletzung{} erkannt: {} kritisch (>50% überschritten), {} Warnung{}.",
        violations.len(),
        if violations.len() == 1 { "" } else { "en" },
        error_count,
        warning_count,
        if warning_count == 1 { "" } else { "en" },
    );

    builder = if error_count > 0 {
        builder.add_component(Callout::warning(&summary_text).with_title("Budget überschritten"))
    } else {
        builder.add_component(Callout::info(&summary_text).with_title("Budget-Hinweise"))
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new("Metrik"),
        TableColumn::new("Budget"),
        TableColumn::new("Ist-Wert"),
        TableColumn::new("Überschreitung"),
        TableColumn::new("Schweregrad"),
    ])
    .with_title("Budget-Details");

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
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(Section::new("Performance").with_level(2))
        .add_component(TextBlock::new(&perf.interpretation))
        .add_component(
            ScoreCard::new("Performance Score", perf.score)
                .with_description(format!("Grade: {}", perf.grade))
                .with_thresholds(75, 50),
        );

    if !perf.vitals.is_empty() {
        let mut kv = KeyValueList::new().with_title("Core Web Vitals");
        for (name, value, rating) in &perf.vitals {
            kv = kv.add(name, format!("{} — {}", value, rating));
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

    if !perf.recommendations.is_empty() {
        let mut rec_list = List::new().with_title("Verbesserungsvorschläge");
        for recommendation in &perf.recommendations {
            rec_list = rec_list.add_item(recommendation);
        }
        builder = builder.add_component(rec_list);
    }

    if perf.has_render_blocking {
        builder =
            builder.add_component(Section::new("Render Blocking & Asset-Größen").with_level(3));

        if !perf.render_blocking_metrics.is_empty() {
            let mut kv = KeyValueList::new().with_title("Render-Blocking Analyse");
            for (k, v) in &perf.render_blocking_metrics {
                kv = kv.add(k, v);
            }
            builder = builder.add_component(kv);
        }

        if !perf.render_blocking_suggestions.is_empty() {
            let mut suggestions = List::new().with_title("Empfehlungen");
            for s in &perf.render_blocking_suggestions {
                suggestions = suggestions.add_item(s);
            }
            builder = builder.add_component(suggestions);
        }
    }

    builder
}

pub(super) fn render_seo(
    mut builder: renderreport::engine::ReportBuilder,
    seo: &SeoPresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("SEO-Analyse").with_level(2))
        .add_component(TextBlock::new(&seo.interpretation))
        .add_component(ScoreCard::new("SEO Score", seo.score).with_thresholds(80, 50));

    if !seo.meta_tags.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Feld").with_width("24%"),
            TableColumn::new("Wert").with_width("76%"),
        ])
        .with_title("Meta-Tags");
        for (k, v) in &seo.meta_tags {
            table = table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder.add_component(table);
    }

    if !seo.meta_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Feld"),
            TableColumn::new("Schweregrad"),
            TableColumn::new("Beschreibung"),
        ])
        .with_title("Meta-Tag Probleme");
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
        .with_title("Tracking und externe Dienste");
        for (k, v) in &seo.tracking_summary {
            tracking_table = tracking_table.add_row(vec![k.clone(), v.clone()]);
        }
        builder = builder
            .add_component(Callout::info(&seo.tracking_summary_text).with_title("Einordnung"))
            .add_component(tracking_table);
    }

    if !seo.technical_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title("Technisches SEO");
        for (k, v) in &seo.technical_summary {
            kv = kv.add(k, v);
        }
        builder = builder.add_component(kv);
    }

    // SEO Content Profile
    if let Some(profile) = &seo.profile {
        builder = render_seo_profile(builder, profile);
    }

    builder
}

pub(super) fn render_seo_profile(
    mut builder: renderreport::engine::ReportBuilder,
    profile: &SeoProfilePresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(Section::new("SEO-Inhaltsprofil").with_level(3));

    let mut identity_table = AuditTable::new(vec![
        TableColumn::new("Aspekt").with_width("24%"),
        TableColumn::new("Wert").with_width("76%"),
    ])
    .with_title("Inhaltsprofil");
    for (key, value) in &profile.identity_facts {
        identity_table = identity_table.add_row(vec![key.clone(), value.clone()]);
    }

    let mut page_profile_table = AuditTable::new(vec![
        TableColumn::new("Aspekt").with_width("24%"),
        TableColumn::new("Wert").with_width("76%"),
    ])
    .with_title("Seitenprofil");
    for (key, value) in &profile.page_profile_facts {
        page_profile_table = page_profile_table.add_row(vec![key.clone(), value.clone()]);
    }

    builder = builder
        .add_component(Callout::info(&profile.identity_summary).with_title("Einordnung"))
        .add_component(identity_table)
        .add_component(page_profile_table);

    let mut score_grid = renderreport::components::advanced::Grid::new(2);
    for (title, score, subtitle, accent) in [
        (
            "Content-Tiefe",
            profile.content_depth_score,
            score_quality_label(profile.content_depth_score),
            score_quality_color(profile.content_depth_score),
        ),
        (
            "Strukturqualität",
            profile.structural_richness_score,
            score_quality_label(profile.structural_richness_score),
            score_quality_color(profile.structural_richness_score),
        ),
        (
            "Medienbalance",
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
    let mut identity = KeyValueList::new().with_title("Website-Identität");
    identity = identity.add("Website", &profile.site_name);
    identity = identity.add("Inhaltstyp", &profile.content_type);
    identity = identity.add("Sprache", &profile.language);
    if !profile.category_hints.is_empty() {
        identity = identity.add("Schema-Typen", profile.category_hints.join(", "));
    }
    builder = builder.add_component(identity);

    // Schema Inventory
    if !profile.schema_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Schema-Typ"),
            TableColumn::new("Vollständigkeit"),
            TableColumn::new("Details"),
        ])
        .with_title(format!(
            "Strukturierte Daten ({} Schemas)",
            profile.schema_count
        ));
        for (typ, completeness, details) in &profile.schema_rows {
            table = table.add_row(vec![typ.as_str(), completeness.as_str(), details.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Strength Overview
    if !profile.signal_rows.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new("Kategorie"),
            TableColumn::new("Bewertung"),
            TableColumn::new("Einstufung"),
        ])
        .with_title(format!(
            "SEO-Signalstärke (Gesamt: {}%)",
            profile.signal_overall_pct
        ));
        for (cat, score, rating) in &profile.signal_rows {
            table = table.add_row(vec![cat.as_str(), score.as_str(), rating.as_str()]);
        }
        builder = builder.add_component(table);
    }

    // Signal Details per category
    for (cat_name, checks) in &profile.signal_details {
        let mut detail_table = AuditTable::new(vec![
            TableColumn::new("Prüfung"),
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
    let mut maturity = SummaryBox::new("SEO-Reifegrad");
    maturity = maturity.add_item("Level", &profile.maturity_level);
    maturity = maturity.add_item("Bewertung", &profile.maturity_description);
    maturity = maturity.add_item(
        "Techniken",
        format!(
            "{} von {} erkannt",
            profile.maturity_techniques_used, profile.maturity_techniques_total
        ),
    );
    builder = builder.add_component(maturity);

    builder
}

pub(super) fn render_security(
    mut builder: renderreport::engine::ReportBuilder,
    sec: &SecurityPresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Sicherheit").with_level(2))
        .add_component(TextBlock::new(&sec.interpretation))
        .add_component(
            ScoreCard::new("Security Score", sec.score)
                .with_description(format!("Grade: {}", sec.grade))
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
        builder = builder.add_component(Finding::new(title, map_severity(sev), msg));
    }

    if !sec.recommendations.is_empty() {
        let mut rec_list = List::new().with_title("Verbesserungsvorschläge");
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
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Mobile Nutzbarkeit").with_level(2))
        .add_component(TextBlock::new(&mobile.interpretation))
        .add_component(ScoreCard::new("Mobile Score", mobile.score).with_thresholds(80, 50));

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
        builder = builder.add_component(Finding::new(cat, map_severity(sev), msg));
    }
    builder
}

pub(super) fn render_ux(
    mut builder: renderreport::engine::ReportBuilder,
    ux: &UxPresentation,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("User Experience").with_level(2))
        .add_component(TextBlock::new(&ux.interpretation))
        .add_component(ScoreCard::new("UX Score", ux.score).with_thresholds(80, 50));

    // Dimension scores as KeyValueList
    let mut kv = KeyValueList::new().with_title("UX-Dimensionen");
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
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("User Journey").with_level(2))
        .add_component(TextBlock::new(&journey.interpretation))
        .add_component(ScoreCard::new("Journey Score", journey.score).with_thresholds(80, 50));

    // Page intent
    let mut kv = KeyValueList::new().with_title("Seitentyp & Dimensionen");
    kv = kv.add("Erkannter Seitentyp", &journey.page_intent);
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
) -> renderreport::engine::ReportBuilder {
    let support_label = if dm.supported {
        "Unterstützt"
    } else {
        "Nicht unterstützt"
    };
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Dark Mode").with_level(2))
        .add_component(ScoreCard::new("Dark Mode Score", dm.score).with_thresholds(80, 50));

    let mut kv = KeyValueList::new().with_title("Dark Mode Übersicht");
    kv = kv.add("Unterstützung", support_label);
    if !dm.detection_methods.is_empty() {
        kv = kv.add("Implementierungsmethoden", dm.detection_methods.join(", "));
    }
    kv = kv.add(
        "color-scheme CSS",
        if dm.color_scheme_css { "Ja" } else { "Nein" },
    );
    if let Some(ref meta) = dm.meta_color_scheme {
        kv = kv.add("<meta name=\"color-scheme\">", meta.as_str());
    }
    if dm.css_custom_properties > 0 {
        kv = kv.add(
            "CSS Custom Properties (Farben)",
            dm.css_custom_properties.to_string(),
        );
    }
    if dm.supported {
        kv = kv.add(
            "Kontrast-Violations im Dark Mode",
            dm.dark_contrast_violations.to_string(),
        );
        if dm.dark_only_violations > 0 {
            kv = kv.add(
                "Nur-Dark-Mode-Probleme",
                format!("{} (nicht im Light Mode)", dm.dark_only_violations),
            );
        }
        if dm.light_only_violations > 0 {
            kv = kv.add(
                "Im Dark Mode behoben",
                format!(
                    "{} Light-Mode-Probleme verschwinden im Dark Mode",
                    dm.light_only_violations
                ),
            );
        }
    }
    builder = builder.add_component(kv);

    if !dm.issues.is_empty() {
        for (severity, description) in &dm.issues {
            builder = builder.add_component(match severity.as_str() {
                "high" => Callout::warning(description).with_title("Dark Mode Problem"),
                _ => Callout::info(description).with_title("Dark Mode Hinweis"),
            });
        }
    }

    builder
}

pub(super) fn render_source_quality(
    mut builder: renderreport::engine::ReportBuilder,
    sq: &crate::source_quality::SourceQualityAnalysis,
) -> renderreport::engine::ReportBuilder {
    builder = builder
        .add_component(PageBreak::new())
        .add_component(Section::new("Quellenqualität").with_level(2))
        .add_component(Callout::info(&sq.disclaimer).with_title("Hinweis"))
        .add_component(
            ScoreCard::new("Quellenqualität", sq.score)
                .with_description(format!(
                    "Grade: {} — {}",
                    sq.grade,
                    score_quality_label(sq.score)
                ))
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
