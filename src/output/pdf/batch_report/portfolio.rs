use crate::output::localized::is_english;
use renderreport::components::advanced::{PageBreak, SectionHeaderSplit, SoftPageBreak};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

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

    builder = builder.add_component(SoftPageBreak::needs(super::causes::SECTION_OPENER_ROOM));
    builder =
        builder.add_component(Section::new(i18n.t("batch-module-portfolio-title")).with_level(2));

    // Average, then the spread over the pages that measured the module —
    // no "clean pages" from score ≥ 90, which says nothing about findings.
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
            localized_module_name(name, i18n),
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

pub(super) fn localized_module_name(name: &str, i18n: &I18n) -> String {
    i18n.t(match name {
        "Accessibility" => "module-accessibility",
        "Best Practices" => "module-best-practices",
        "Mobile" => "module-mobile",
        "Performance" => "module-performance",
        "Security" => "module-security",
        "HTML Conformance" => "batch-report-module-html-conformance",
        "SEO" => "module-seo",
        "UX" => "module-ux",
        "Journey" => "module-journey",
        "AI Visibility" => "module-ai-visibility",
        "Source Quality" => "module-source-quality",
        "Tech Stack" => "module-tech-stack",
        "Privacy" => "module-privacy",
        "Commerce" => "module-commerce",
        _ => return name.to_string(),
    })
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

/// At most this many pages in the per-page type and top-issue table.
const MAX_FOCUS_PAGES: usize = 10;

/// The pages of the per-page type and top-issue table: every page while they
/// fit, otherwise the lowest accessibility scores. It used to take the first
/// ten in sitemap order and call them "problematic" (#1039).
fn focus_pages(details: &[CompactUrlSummary]) -> Vec<&CompactUrlSummary> {
    let mut pages: Vec<&CompactUrlSummary> = details.iter().collect();
    if pages.len() > MAX_FOCUS_PAGES {
        pages.sort_by(|a, b| a.score.total_cmp(&b.score).then_with(|| a.url.cmp(&b.url)));
        pages.truncate(MAX_FOCUS_PAGES);
    }
    pages
}

/// The title of that table: it selects "problematic" pages only when it
/// leaves pages out; listing all of them, it is a per-page overview.
pub(super) fn focus_table_title_key(pages: usize) -> &'static str {
    if pages > MAX_FOCUS_PAGES {
        "batch-table-focus-pages"
    } else {
        "batch-table-pages-profile"
    }
}

/// How many of the lowest and the highest ranked pages the technical part
/// shows; the full list is in the appendix of report level technical.
const RANKING_EDGE: usize = 5;

/// More pages than both edges show: the technical part prints the compact
/// table.
pub(super) fn ranking_is_compact(pres: &BatchPresentation) -> bool {
    pres.url_ranking.len() > 2 * RANKING_EDGE
}

/// The short column head of a module in the per-page table.
fn module_column_label(module: &str, i18n: &I18n) -> String {
    match module {
        "Accessibility" => i18n.t("batch-ranking-col-accessibility"),
        "SEO" => i18n.t("batch-ranking-col-seo"),
        "Performance" => i18n.t("batch-ranking-col-performance"),
        "Security" => i18n.t("batch-ranking-col-security"),
        "HTML Conformance" => i18n.t("batch-ranking-col-html-conformance"),
        other => localized_module_name(other, i18n),
    }
}

/// One table with every module's score per page, the lowest accessibility
/// first (the order is the rank): the page without its domain, the
/// site-quality score, one column per module that ran, and the critical and
/// high findings (grouped per page).
fn module_scores_table(pres: &BatchPresentation, rows: &[usize], i18n: &I18n) -> AuditTable {
    let modules: Vec<&str> = pres
        .portfolio_summary
        .module_averages
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let numeric = modules.len() + 2;
    let width = format!("{}%", 76 / numeric.max(1));
    let mut columns = vec![
        TableColumn::new(i18n.t("batch-ranking-col-page")).with_width("24%"),
        TableColumn::new(i18n.t("batch-ranking-col-site-quality")).with_width(&width),
    ];
    for module in &modules {
        columns.push(TableColumn::new(module_column_label(module, i18n)).with_width(&width));
    }
    columns.push(TableColumn::new(i18n.t("batch-ranking-col-critical-high")).with_width(&width));
    let mut table = AuditTable::new(columns);
    for &i in rows {
        let page = &pres.url_ranking[i];
        let detail = pres.url_details.iter().find(|d| d.url == page.url);
        let mut row = vec![
            super::compare::url_path(&page.url),
            page.site_quality_score.to_string(),
        ];
        for module in &modules {
            row.push(
                detail
                    .and_then(|d| d.module_scores.iter().find(|(name, _)| name == module))
                    .map_or_else(|| "–".to_string(), |(_, score)| score.to_string()),
            );
        }
        row.push(page.critical_violations.to_string());
        table = table.add_row(row);
    }
    table
}

/// The module scores per page of the technical part: every page, or the five
/// lowest and the five highest ranked ones, the full list in the appendix.
pub(super) fn render_batch_url_ranking(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    has_appendix: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let compact = ranking_is_compact(pres);
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-url-ranking-title"),
            ranking_intro(compact, has_appendix, i18n),
        )
        .with_level(2),
    );
    let (shown, _) = super::compare::compared_pages(pres);
    builder.add_component(module_scores_table(pres, &shown, i18n))
}

/// The full module scores per page, one table, for the technical appendix.
pub(super) fn render_batch_full_ranking(
    builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let all: Vec<usize> = (0..pres.url_ranking.len()).collect();
    builder
        .add_component(Section::new(i18n.t("batch-url-ranking-full")).with_level(2))
        .add_component(module_scores_table(pres, &all, i18n))
}

/// The intro of the module scores per page; a compact table says where the
/// full list is.
pub(super) fn ranking_intro(compact: bool, has_appendix: bool, i18n: &I18n) -> String {
    let base = i18n.t("batch-url-ranking-intro");
    if !compact {
        return base;
    }
    let rest = i18n.t(if has_appendix {
        "batch-url-ranking-rest-appendix"
    } else {
        "batch-url-ranking-rest-json"
    });
    format!("{base} {rest}")
}

/// Broken links and redirect chains of a crawl run: part of the technical
/// part at every report level.
pub(super) fn render_batch_crawl_diagnostics(
    builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    match pres.portfolio_summary.crawl_links {
        Some(ref crawl_links) => render_batch_crawl_links(builder, crawl_links, i18n),
        None => builder,
    }
}

pub(super) fn render_batch_tech_url_matrix(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    config: &ReportConfig,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    // A new page: the detail chapters start here, and the header never stands
    // alone at the foot of the page before.
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-section-tech-url-matrix"),
            i18n.t("batch-section-tech-url-matrix-intro"),
        )
        .with_level(1),
    );

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
        // site-quality score (#1039).
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
                .map(|t| crate::util::truncate_words(t, 36))
                .unwrap_or_else(|| "—".to_string()),
            row.inbound_links.to_string(),
            row.outbound_links.to_string(),
            crate::i18n::number::integer(i64::from(row.word_count), en),
            score_str,
        ]);
    }
    builder = builder.add_component(matrix);

    if !config.level.is_summary() {
        let mut focus_table = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-attributes")),
            TableColumn::new(i18n.t("batch-col-top-issues")),
        ])
        .with_title(i18n.t(focus_table_title_key(pres.url_details.len())));

        for detail in focus_pages(&pres.url_details) {
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
                    leading_items(&detail.top_issues, 2, i18n)
                },
            ]);
        }
        builder = builder.add_component(focus_table);
    }

    builder
}

/// The first `max_items` items of a list, each whole, and how many more there
/// are. The number shown is a count, not a character budget: titles differ in
/// length between the languages, so a budget showed two titles in English and
/// three in German for the same page (#1039). One cut through the joined list
/// printed "Insufficient colo… frame-ancestors directive missing".
pub(super) fn leading_items(items: &[String], max_items: usize, i18n: &I18n) -> String {
    let shown = &items[..items.len().min(max_items)];
    let list = shown.join(", ");
    let rest = items.len() - shown.len();
    if rest == 0 {
        return list;
    }
    i18n.t_args(
        "batch-dup-pages-more",
        &[
            ("list", fluent_bundle::FluentValue::from(list)),
            ("count", fluent_bundle::FluentValue::from(rest as i64)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditReport, BatchReport};
    use crate::model::WcagLevel;
    use crate::wcag::WcagResults;

    /// Identical scores read as one value on all measured pages,
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
