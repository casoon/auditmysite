use crate::output::localized::is_english;
use renderreport::components::advanced::{Grid, KeyValueList, SectionHeaderSplit};
use renderreport::components::charts::{Gauge, GaugeThreshold};
use renderreport::components::{AuditTable, BenchmarkRow, BenchmarkTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::model::ReportLevel;
use crate::output::report_model::*;
use crate::util::short_url;

use super::super::design::tokens;
use super::appendix::render_batch_crawl_links;

pub(super) fn render_batch_module_portfolio(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    normalized_reports: &[crate::audit::normalized::NormalizedReport],
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

    // Average, then the spread over the pages that measured the module —
    // no "clean pages" from score ≥ 90, which says nothing about findings
    // (plan 86).
    let mut table = AuditTable::new(vec![
        TableColumn::new(i18n.t("batch-col-module")).with_width("20%"),
        TableColumn::new(i18n.t("batch-col-score")).with_width("14%"),
        TableColumn::new(i18n.t("batch-col-spread")).with_width("22%"),
        TableColumn::new(i18n.t("batch-col-effect")).with_width("44%"),
    ])
    .with_title(i18n.t("batch-module-portfolio-table-title"));

    for (name, score) in &pres.portfolio_summary.module_averages {
        let measured = module_score_spread(pres, name);
        let spread = match measured {
            Some((min, max, pages)) if min == max => i18n.t_args(
                "batch-module-spread-constant",
                &[("value", i64::from(min)), ("pages", pages as i64)],
            ),
            Some((min, max, pages)) => i18n.t_args(
                "batch-module-spread-range",
                &[
                    ("min", i64::from(min)),
                    ("max", i64::from(max)),
                    ("pages", pages as i64),
                ],
            ),
            None => "—".to_string(),
        };
        table = table.add_row(vec![
            localized_module_name(name, en).to_string(),
            format!("{score}/100"),
            spread,
            module_effect_sentence(
                name,
                *score,
                measured.map(|(min, _, _)| min),
                module_high_findings(normalized_reports, name),
                i18n,
            ),
        ]);
    }
    builder = builder.add_component(table);

    builder
}

/// (min, max, pages) of a module's score over the URLs that measured it.
pub(super) fn module_score_spread(
    pres: &BatchPresentation,
    module_name: &str,
) -> Option<(u32, u32, usize)> {
    let scores: Vec<u32> = pres
        .url_details
        .iter()
        .filter_map(|detail| {
            detail
                .module_scores
                .iter()
                .find(|(name, _)| name == module_name)
                .map(|(_, score)| *score)
        })
        .collect();
    Some((*scores.iter().min()?, *scores.iter().max()?, scores.len()))
}

pub(super) fn localized_module_name(name: &str, en: bool) -> &str {
    match (name, en) {
        ("Accessibility", false) => "Barrierefreiheit",
        ("Best Practices", false) => "Best Practices",
        ("Mobile", false) => "Mobile",
        ("Performance", false) => "Performance",
        ("Security", false) => "Security",
        ("HTML Conformance", false) => "HTML-Konformität",
        ("SEO", false) => "SEO",
        ("UX", false) => "UX",
        ("Journey", false) => "Journey",
        ("AI Visibility", false) => "KI-Sichtbarkeit",
        ("Content Visibility", false) => "Content Visibility",
        ("Source Quality", false) => "Technische Aufbereitung",
        ("Source Quality", true) => "Technical preparation",
        ("Tech Stack", false) => "Tech-Stack",
        _ => name,
    }
}

/// Open findings of high or critical severity that belong to a module,
/// summed over the pages.
pub(super) fn module_high_findings(
    normalized_reports: &[crate::audit::normalized::NormalizedReport],
    module_name: &str,
) -> usize {
    use crate::audit::finding_contract::FindingDomain as D;
    let domains: &[D] = match module_name {
        "Accessibility" => &[D::Accessibility, D::AccessibilityJourney, D::ScreenReader],
        "Performance" => &[D::Performance],
        "SEO" => &[D::Seo],
        "Security" => &[D::Security],
        "Mobile" => &[D::Mobile],
        "HTML Conformance" => &[D::HtmlConformance],
        "Best Practices" => &[D::BestPractices],
        _ => &[],
    };
    normalized_reports
        .iter()
        .flat_map(|report| report.findings.iter())
        .filter(|f| {
            domains.contains(&f.contract.domain) && f.severity >= crate::taxonomy::Severity::High
        })
        .count()
}

/// The effect sentence of a module row. It claims a clean or stable state
/// only when the data shows one — every measured page at 90 or above and no
/// open high-severity finding of the module; the band of the average alone
/// put "signals are clean" next to a high finding and "variance" next to the
/// same value on every page (#1039).
pub(super) fn module_effect_sentence(
    module_name: &str,
    score: u32,
    weakest_page: Option<u32>,
    high_findings: usize,
    i18n: &I18n,
) -> String {
    let slug = match module_name {
        "Accessibility" => "accessibility",
        "Performance" => "performance",
        "SEO" => "seo",
        "Security" => "security",
        "Mobile" => "mobile",
        "HTML Conformance" => "html-conformance",
        _ => "other",
    };
    if score >= 90 {
        if high_findings > 0 {
            return i18n.count("batch-effect-open-high", high_findings);
        }
        if let Some(min) = weakest_page.filter(|min| *min < 90) {
            return i18n.t_args("batch-effect-uneven", &[("min", i64::from(min))]);
        }
        return i18n.t(&format!("batch-effect-{slug}-good"));
    }
    if slug != "other" {
        return i18n.t(&format!("batch-effect-{slug}-weak"));
    }
    i18n.t(match score {
        75..=89 => "batch-effect-other-usable",
        60..=74 => "batch-effect-other-recurring",
        _ => "batch-effect-other-structural",
    })
}

pub(super) fn render_batch_impact_summary(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    legal_flags: usize,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let dist = &pres.portfolio_summary.severity_distribution;
    let a11y_avg = pres
        .portfolio_summary
        .average_score
        .expect(super::UNMEASURED_REFUSED)
        .round() as u32;

    let user_impact = match (a11y_avg, en) {
        (s, true) if s < 50 => {
            "Critical — on the audited pages, barriers severely hamper use with assistive technologies"
        }
        (s, false) if s < 50 => {
            "Kritisch — auf den geprüften Seiten erschweren Barrieren die Nutzung mit Hilfstechnologien erheblich"
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
        // About the audited pages: how far beyond the sample the barriers
        // reach is not measured (plan 80).
        if en {
            "The audited pages contain critical barriers for certain user groups; how far they reach beyond these pages is not measured."
        } else {
            "Die geprüften Seiten enthalten kritische Barrieren für bestimmte Nutzergruppen; wie weit sie über diese Seiten hinausreichen, ist nicht gemessen."
        }
    } else if dist.high > 0 {
        if en {
            "Individual functional areas are problematic for users with disabilities."
        } else {
            "Einzelne Funktionsbereiche sind für Nutzer mit Einschränkungen problematisch."
        }
    } else if en {
        "Low impact on the audited pages — no critical or high barriers detected automatically."
    } else {
        "Geringe Auswirkung auf den geprüften Seiten — automatisiert keine kritischen oder hohen Barrieren erkannt."
    };
    // The legal flags the batch decision fails on, not critical findings
    // alone (#1039).
    let legal_impact = if legal_flags > 0 {
        if en {
            "WCAG Level A/AA violations detected automatically — manual review required for a defensible BFSG classification."
        } else {
            "WCAG-Level-A/AA-Verstöße automatisiert erkannt — für belastbare BFSG-Einordnung ist manuelle Prüfung nötig."
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
/// `(module_scores name, Fluent key of the short column header)`.
/// `BenchmarkRow::with_column` (renderreport >= 0.5.0) is fully generic —
/// adding a dimension here is the only change needed; no renderreport-side
/// release is required.
const RANKING_EXTRA_COLUMNS: &[(&str, &str)] = &[
    ("SEO", "batch-ranking-col-seo"),
    ("Performance", "batch-ranking-col-performance"),
    ("Security", "batch-ranking-col-security"),
    ("HTML Conformance", "batch-ranking-col-html-conformance"),
];

pub(super) fn render_batch_url_ranking(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let extra_headers: Vec<(&str, String)> = RANKING_EXTRA_COLUMNS
        .iter()
        .map(|(module_name, key)| (*module_name, i18n.t(key)))
        .collect();
    let rows: Vec<BenchmarkRow> = pres
        .url_ranking
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let mut row = BenchmarkRow::new(
                (i + 1) as u32,
                &short_url(&u.url, 35),
                u.overall_score,
                u.score as u32,
                u.critical_violations as u32,
            );
            if let Some(detail) = pres.url_details.iter().find(|detail| detail.url == u.url) {
                for (module_name, header) in &extra_headers {
                    if let Some((_, score)) = detail
                        .module_scores
                        .iter()
                        .find(|(module, _)| module == module_name)
                    {
                        row = row.with_column(header.as_str(), *score);
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
        .add_component(ranking_table(rows));

    builder
}

/// The ranking table without renderreport's built-in German title
/// ("Website-Ranking"): the section header above already names it in the run
/// language (#1039).
fn ranking_table(rows: Vec<BenchmarkRow>) -> BenchmarkTable {
    let mut table = BenchmarkTable::new(rows);
    table.title = None;
    table
}

pub(super) fn render_batch_tech_url_matrix(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    config: &ReportConfig,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
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
        // The accessibility score; "Score" in the ranking above is the
        // combined technical value (#1039).
        TableColumn::new(i18n.t("batch-matrix-col-accessibility")).with_width("12%"),
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
            short_url(&row.url, 34),
            row.title
                .as_deref()
                .map(|t| short_url(t, 36))
                .unwrap_or_else(|| "—".to_string()),
            row.inbound_links.to_string(),
            row.outbound_links.to_string(),
            crate::i18n::number::integer(i64::from(row.word_count), en),
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
                short_url(&detail.url, 38),
                detail.page_type.clone().unwrap_or_else(|| "—".to_string()),
                // A short word list: shortening it cut words in the middle
                // ("knowledge-ori…uctured", #1039).
                if detail.page_attributes.is_empty() {
                    "—".to_string()
                } else {
                    detail.page_attributes.join(", ")
                },
                if detail.top_issues.is_empty() {
                    "—".to_string()
                } else {
                    short_url(&detail.top_issues.join(", "), 52)
                },
            ]);
        }
        builder = builder.add_component(focus_table);
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditReport, BatchReport};
    use crate::model::WcagLevel;
    use crate::wcag::WcagResults;

    /// Plan 86: identical scores read as one value on all measured pages,
    /// and a module no page measured has no spread at all.
    #[test]
    fn module_spread_reports_min_max_over_measured_pages() {
        let reports = ["https://x.test/a", "https://x.test/b"]
            .iter()
            .map(|url| AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100))
            .collect();
        let pres = crate::output::builder::build_batch_presentation(&BatchReport::from_reports(
            reports,
            vec![],
            100,
        ));
        assert_eq!(
            module_score_spread(&pres, "Accessibility"),
            Some((100, 100, 2))
        );
        assert_eq!(module_score_spread(&pres, "Performance"), None);
    }
}
