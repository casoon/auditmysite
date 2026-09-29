use crate::output::localized::is_english;
use renderreport::components::advanced::{Grid, KeyValueList, SectionHeaderSplit};
use renderreport::components::charts::{Gauge, GaugeThreshold};
use renderreport::components::{AuditTable, BenchmarkRow, BenchmarkTable, TableColumn};
use renderreport::prelude::*;

use crate::cli::ReportLevel;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::super::design::tokens;
use super::appendix::render_batch_crawl_links;

pub(super) fn render_batch_module_portfolio(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if pres.portfolio_summary.module_averages.is_empty() {
        return builder;
    }

    let en = is_english(i18n);
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-module-portfolio-title"),
            i18n.t("batch-module-portfolio-intro"),
        )
        .with_level(1),
    );

    let mut grid = Grid::new(4).with_item_min_height("118pt");
    for (name, score) in &pres.portfolio_summary.module_averages {
        let mut gauge = Gauge::new(localized_module_name(name, en), *score as f64);
        gauge.thresholds = vec![
            GaugeThreshold {
                value: 0.0,
                color: tokens::DANGER.to_string(),
            },
            GaugeThreshold {
                value: 50.0,
                color: tokens::WARN_DEEP.to_string(),
            },
            GaugeThreshold {
                value: 90.0,
                color: tokens::SUCCESS.to_string(),
            },
        ];
        grid = grid.add_item(serde_json::json!({
            "type": "gauge",
            "data": gauge.to_data()
        }));
    }
    builder = builder.add_component(grid);

    let total = pres.portfolio_summary.total_urls;
    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("batch-col-module")).with_width("20%"),
        TableColumn::new(i18n.t("batch-col-score")).with_width("14%"),
        TableColumn::new(i18n.t("batch-col-clean-pages")).with_width("18%"),
        TableColumn::new(i18n.t("batch-col-effect")).with_width("48%"),
    ])
    .with_title(i18n.t("batch-module-portfolio-table-title"));

    for (name, score) in &pres.portfolio_summary.module_averages {
        let clean_pages = clean_pages_for_module(pres, name);
        table = table.add_row(vec![
            localized_module_name(name, en).to_string(),
            format!("{score}/100"),
            i18n.t_args(
                "batch-module-clean-count",
                &[
                    ("clean", clean_pages.to_string()),
                    ("total", total.to_string()),
                ],
            ),
            module_effect_sentence(name, *score, clean_pages, total, en),
        ]);
    }
    builder = builder.add_component(table);

    let all_clean: Vec<String> = pres
        .portfolio_summary
        .module_averages
        .iter()
        .filter(|(name, _)| clean_pages_for_module(pres, name) == total && total > 0)
        .map(|(name, _)| localized_module_name(name, en).to_string())
        .collect();
    if !all_clean.is_empty() {
        builder = builder.add_component(
            Callout::success(
                i18n.t_args(
                    "batch-module-clean-confirmation",
                    &[
                        ("modules", all_clean.join(", ")),
                        ("total", total.to_string()),
                    ],
                )
                .as_str(),
            )
            .with_title(i18n.t("batch-module-clean-title")),
        );
    }

    builder
}

pub(super) fn clean_pages_for_module(pres: &BatchPresentation, module_name: &str) -> usize {
    pres.url_details
        .iter()
        .filter(|detail| {
            detail
                .module_scores
                .iter()
                .find(|(name, _)| name == module_name)
                .is_some_and(|(_, score)| *score >= 90)
        })
        .count()
}

pub(super) fn localized_module_name(name: &str, en: bool) -> &str {
    match (name, en) {
        ("Accessibility", false) => "Barrierefreiheit",
        ("Best Practices", false) => "Best Practices",
        ("Dark Mode", false) => "Dark Mode",
        ("Mobile", false) => "Mobile",
        ("Performance", false) => "Performance",
        ("Security", false) => "Security",
        ("HTML Conformance", false) => "HTML-Konformität",
        ("SEO", false) => "SEO",
        ("UX", false) => "UX",
        ("Journey", false) => "Journey",
        ("AI Visibility", false) => "KI-Sichtbarkeit",
        ("Content Visibility", false) => "Content Visibility",
        ("Source Quality", false) => "Source Quality",
        ("Tech Stack", false) => "Tech-Stack",
        _ => name,
    }
}

pub(super) fn module_effect_sentence(
    module_name: &str,
    score: u32,
    clean_pages: usize,
    total: usize,
    en: bool,
) -> String {
    if clean_pages == total && total > 0 {
        return if en {
            "Checked across the audited URL set with no relevant anomalies in the green band."
                .to_string()
        } else {
            "Im geprüften URL-Set ohne relevante Auffälligkeiten im grünen Bereich.".to_string()
        };
    }

    let band = match score {
        90..=100 => 0,
        75..=89 => 1,
        60..=74 => 2,
        _ => 3,
    };

    match (module_name, band, en) {
        ("Accessibility", 0, true) => {
            "Accessibility is broadly stable; remaining findings are concentrated on individual pages."
        }
        ("Accessibility", 0, false) => {
            "Barrierefreiheit ist breit stabil; verbleibende Befunde konzentrieren sich auf einzelne Seiten."
        }
        ("Accessibility", _, true) => {
            "Accessibility barriers affect repeated user paths and should be prioritized by reach."
        }
        ("Accessibility", _, false) => {
            "Barrieren betreffen wiederkehrende Nutzerpfade und sollten nach Reichweite priorisiert werden."
        }
        ("Performance", 0, true) => {
            "Loading behavior is reliable across most audited pages."
        }
        ("Performance", 0, false) => {
            "Das Ladeverhalten ist über die meisten geprüften Seiten zuverlässig."
        }
        ("Performance", _, true) => {
            "Performance variance can slow important page groups and weaken completion rates."
        }
        ("Performance", _, false) => {
            "Performance-Streuung kann wichtige Seitengruppen verlangsamen und Abschlüsse erschweren."
        }
        ("SEO", 0, true) => {
            "Search visibility basics are consistent across the audited URL set."
        }
        ("SEO", 0, false) => {
            "Grundlagen für Sichtbarkeit in Suchmaschinen sind im geprüften URL-Set konsistent."
        }
        ("SEO", _, true) => {
            "SEO inconsistencies can dilute discoverability across page templates."
        }
        ("SEO", _, false) => {
            "SEO-Inkonsistenzen können die Auffindbarkeit über Seitentemplates hinweg schwächen."
        }
        ("Security", 0, true) => {
            "Security signals are clean in the automated checks across the audited pages."
        }
        ("Security", 0, false) => {
            "Security-Signale sind in den automatisierten Checks über die geprüften Seiten sauber."
        }
        ("Security", _, true) => {
            "Security findings should be resolved centrally because they often affect all templates."
        }
        ("Security", _, false) => {
            "Security-Befunde sollten zentral gelöst werden, da sie häufig alle Templates betreffen."
        }
        ("Mobile", 0, true) => "Mobile usage is stable across the audited pages.",
        ("Mobile", 0, false) => {
            "Die Nutzung auf Mobilgeräten ist über die geprüften Seiten stabil."
        }
        ("Mobile", _, true) => {
            "Mobile issues can reduce usability on smaller screens across page groups."
        }
        ("Mobile", _, false) => {
            "Mobile Probleme können die Nutzung auf kleineren Displays über Seitengruppen erschweren."
        }
        ("HTML Conformance", 0, true) => {
            "HTML markup is spec-conformant across the audited pages, which keeps browsers and assistive technology parsing it consistently."
        }
        ("HTML Conformance", 0, false) => {
            "Das HTML-Markup ist über die geprüften Seiten spezifikationskonform, wodurch Browser und assistive Technologien es konsistent interpretieren."
        }
        ("HTML Conformance", _, true) => {
            "HTML conformance issues often come from a shared template and can affect parsing and accessibility tooling across many pages at once."
        }
        ("HTML Conformance", _, false) => {
            "HTML-Konformitätsprobleme stammen häufig aus einem gemeinsamen Template und können die Interpretation durch Browser und assistive Technologien seitenübergreifend beeinträchtigen."
        }
        (_, 0, true) => "The module is stable across most audited pages.",
        (_, 0, false) => "Das Modul ist über die meisten geprüften Seiten stabil.",
        (_, 1, true) => "The module has a usable foundation with targeted cleanup potential.",
        (_, 1, false) => "Das Modul hat eine tragfähige Basis mit gezieltem Bereinigungsbedarf.",
        (_, 2, true) => "The module shows recurring weaknesses across the audited URL set.",
        (_, 2, false) => "Das Modul zeigt wiederkehrende Schwächen im geprüften URL-Set.",
        (_, _, true) => "The module needs structural attention across multiple pages.",
        (_, _, false) => "Das Modul benötigt strukturelle Aufmerksamkeit über mehrere Seiten.",
    }
    .to_string()
}

pub(super) fn render_batch_impact_summary(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let dist = &pres.portfolio_summary.severity_distribution;
    let a11y_avg = pres.portfolio_summary.average_score.round() as u32;

    let user_impact = match (a11y_avg, en) {
        (s, true) if s < 50 => {
            "Critical — core functions are unreachable for users with disabilities"
        }
        (s, false) if s < 50 => {
            "Kritisch — zentrale Funktionen sind für Nutzer mit Einschränkungen nicht erreichbar"
        }
        (s, true) if s < 70 => {
            "Limited — structural issues impede users with assistive technologies"
        }
        (s, false) if s < 70 => {
            "Eingeschränkt — strukturelle Probleme behindern Nutzer mit Hilfstechnologien"
        }
        (s, true) if s < 85 => {
            "Good — individual barriers for assistive technologies on several pages"
        }
        (s, false) if s < 85 => {
            "Gut — einzelne Barrieren für Hilfstechnologien auf mehreren Seiten"
        }
        (_, true) => "Very good — assistive technologies are largely supported",
        (_, false) => "Sehr gut — Hilfstechnologien werden weitgehend unterstützt",
    };
    let business_impact = if dist.critical > 0 {
        if en {
            "Large parts of the website are unusable or barely usable for certain user groups."
        } else {
            "Weite Teile der Website sind für bestimmte Nutzergruppen nicht oder kaum nutzbar."
        }
    } else if dist.high > 0 {
        if en {
            "Individual functional areas are problematic for users with disabilities."
        } else {
            "Einzelne Funktionsbereiche sind für Nutzer mit Einschränkungen problematisch."
        }
    } else if en {
        "Low impact — users can fundamentally use the website."
    } else {
        "Geringe Auswirkung — Nutzer können die Website grundsätzlich verwenden."
    };
    let legal_impact = if dist.critical > 0 {
        if en {
            "WCAG Level A violations detected automatically — manual review required for a defensible BFSG classification."
        } else {
            "WCAG-Level-A-Verstöße automatisiert erkannt — für belastbare BFSG-Einordnung ist manuelle Prüfung nötig."
        }
    } else if en {
        "No critical violations detected automatically — manual review recommended for full classification."
    } else {
        "Automatisiert keine kritischen Verstöße erkannt — manuelle Prüfung für vollständige Einordnung empfohlen."
    };

    let (user_label, business_label, risk_label) = if en {
        ("User", "Business", "Risk")
    } else {
        ("Nutzer", "Business", "Risiko")
    };
    let mut impact_kv = KeyValueList::new().with_title(i18n.t("narrative-impact-title"));
    impact_kv = impact_kv
        .add(user_label, user_impact)
        .add(business_label, business_impact)
        .add(risk_label, legal_impact);
    builder = builder.add_component(impact_kv);

    builder
}

/// Extra scored dimensions shown in the URL ranking table, as
/// `(module_scores name, short column header)`. `BenchmarkRow::with_column`
/// (renderreport >= 0.5.0) is fully generic — adding a dimension here is the
/// only change needed; no renderreport-side release is required.
const RANKING_EXTRA_COLUMNS: &[(&str, &str)] = &[
    ("SEO", "SEO"),
    ("Performance", "Perf"),
    ("Security", "Sec"),
    ("HTML Conformance", "Konf."),
];

pub(super) fn render_batch_url_ranking(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let rows: Vec<BenchmarkRow> = pres
        .url_ranking
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let mut row = BenchmarkRow::new(
                (i + 1) as u32,
                &truncate_url(&u.url, 35),
                u.overall_score,
                u.score as u32,
                u.critical_violations as u32,
            );
            if let Some(detail) = pres.url_details.iter().find(|detail| detail.url == u.url) {
                for (module_name, header) in RANKING_EXTRA_COLUMNS {
                    if let Some((_, score)) = detail
                        .module_scores
                        .iter()
                        .find(|(module, _)| module == module_name)
                    {
                        row = row.with_column(*header, *score);
                    }
                }
            }
            row
        })
        .collect();

    builder = builder
        .add_component(
            SectionHeaderSplit::new(
                i18n.t("batch-url-ranking-title"),
                i18n.t("batch-url-ranking-intro"),
            )
            .with_level(1),
        )
        .add_component(BenchmarkTable::new(rows));

    builder
}

pub(super) fn render_batch_tech_url_matrix(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    config: &ReportConfig,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-section-tech-url-matrix"),
            i18n.t("batch-section-tech-url-matrix-intro"),
        )
        .with_level(1),
    );

    if let Some(ref crawl_links) = pres.portfolio_summary.crawl_links {
        builder = render_batch_crawl_links(builder, crawl_links, i18n);
    }

    // URL matrix table
    let page_col = i18n.t("batch-matrix-col-page");
    let title_col = i18n.t("batch-matrix-col-title");
    let mut matrix = AuditTable::new(vec![
        TableColumn::new("#").with_width("4%"),
        TableColumn::new(page_col).with_width("26%"),
        TableColumn::new(title_col).with_width("28%"),
        TableColumn::new(i18n.t("batch-col-links-to")).with_width("10%"),
        TableColumn::new(i18n.t("batch-col-links-from")).with_width("10%"),
        TableColumn::new(i18n.t("batch-col-words")).with_width("10%"),
        TableColumn::new("Score").with_width("12%"),
    ])
    .with_title(i18n.t("batch-table-pages-overview"));

    for row in &pres.url_matrix {
        let score_str = pres
            .url_details
            .iter()
            .find(|d| d.url == row.url)
            .map(|d| format!("{}/100", d.score.round() as u32))
            .unwrap_or_else(|| "—".to_string());
        matrix = matrix.add_row(vec![
            row.rank.to_string(),
            truncate_url(&row.url, 34),
            row.title
                .as_deref()
                .map(|t| truncate_url(t, 36))
                .unwrap_or_else(|| "—".to_string()),
            row.inbound_links.to_string(),
            row.outbound_links.to_string(),
            super::super::format_word_count(row.word_count),
            score_str,
        ]);
    }
    builder = builder.add_component(matrix);

    if config.level != ReportLevel::Executive {
        let mut focus_table = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-attributes")),
            TableColumn::new(i18n.t("batch-col-top-issues")),
        ])
        .with_title(i18n.t("batch-table-focus-pages"));

        for detail in pres.url_details.iter().take(10) {
            focus_table = focus_table.add_row(vec![
                truncate_url(&detail.url, 38),
                detail.page_type.clone().unwrap_or_else(|| "—".to_string()),
                if detail.page_attributes.is_empty() {
                    "—".to_string()
                } else {
                    truncate_url(&detail.page_attributes.join(", "), 40)
                },
                if detail.top_issues.is_empty() {
                    "—".to_string()
                } else {
                    truncate_url(&detail.top_issues.join(", "), 52)
                },
            ]);
        }
        builder = builder.add_component(focus_table);
    }

    builder
}
