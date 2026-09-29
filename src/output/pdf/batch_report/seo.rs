use crate::output::localized::is_english;
use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, SectionHeaderSplit,
};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

pub(super) fn render_batch_seo_section(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-seo-potential-title"),
            i18n.t("batch-seo-potential-intro"),
        )
        .with_level(1),
    );

    // Weakest pages
    builder = render_seo_weakest_pages(builder, pres, i18n);

    // Distribution insights
    builder = render_seo_distribution_insights(builder, pres, i18n);

    // Near-duplicates
    builder = render_seo_near_duplicates(builder, pres, i18n);

    // Cross-page duplicate content (identical title / meta description / H1)
    builder = render_seo_duplicate_content(builder, pres, i18n);

    // Cross-page missing-tag prevalence (meta description / canonical / og:image / og:title)
    builder = render_seo_missing_tags(builder, pres, i18n);

    // Canonical conflicts (noindex / og:url mismatch)
    builder = render_seo_canonical_issues(builder, pres, i18n);

    // Non-reciprocal hreflang relationships
    builder = render_seo_hreflang_issues(builder, pres, i18n);

    // Sitemap HTTP/indexability and orphan checks
    builder = render_seo_sitemap_http_issues(builder, pres, i18n);

    builder = render_seo_sitemap_orphans(builder, pres, i18n);

    // Sitemap entries blocked by robots.txt (#549)
    builder = render_seo_robots_conflicts(builder, pres, i18n);

    // Crawl depth: BFS click-distance from a heuristic start page through
    // this batch's already-extracted internal link graph (#548). Purely
    // informational — no scoring path reads this.
    builder = render_seo_crawl_depth(builder, pres, i18n);

    // Page type distribution
    builder = render_seo_page_type_distribution(builder, pres, i18n);

    // Schema distribution
    builder = render_seo_schema_distribution(builder, pres, i18n);

    // Strongest pages
    builder = render_seo_strongest_pages(builder, pres, i18n);

    builder
}

fn render_seo_weakest_pages(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.weakest_content_pages.is_empty() {
        let issues_title = i18n.t("batch-seo-issues-title");
        let mut issues_kv = KeyValueList::new().with_title(issues_title);
        for (url, page_type, score) in &pres.portfolio_summary.weakest_content_pages {
            let relevance =
                super::super::business_relevance(Some(page_type.as_str()), url, i18n.locale());
            let high_marker = if is_english(i18n) { "high" } else { "hoch" };
            let impact = if relevance == high_marker {
                i18n.t("batch-seo-impact-ranking-loss")
            } else if *score < 30 {
                i18n.t("batch-seo-impact-weak-visibility")
            } else {
                i18n.t("batch-seo-impact-opt-potential")
            };
            let value = i18n.t_args(
                "batch-seo-recommendation-words",
                &[("page_type", page_type.clone()), ("impact", impact)],
            );
            let key_str = i18n.t_args(
                "batch-seo-profile-label",
                &[
                    ("url", truncate_url(url, 35)),
                    ("score", format!("{score}")),
                ],
            );
            issues_kv = issues_kv.add(key_str, value);
        }
        builder = builder.add_component(issues_kv);
    }

    builder
}

fn render_seo_distribution_insights(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.distribution_insights.is_empty() {
        let action_label = i18n.t("batch-seo-action-needed");
        let panel_title = i18n.t("batch-seo-patterns-impact-title");
        let rows: Vec<ChecklistRow> = pres
            .portfolio_summary
            .distribution_insights
            .iter()
            .map(|insight| {
                let impact = if insight.contains("Thin") || insight.contains("dünn") {
                    i18n.t_args("batch-seo-impact-thin", &[("insight", insight.clone())])
                } else if insight.contains("Duplikat") || insight.contains("duplicate") {
                    i18n.t_args(
                        "batch-seo-impact-duplicate",
                        &[("insight", insight.clone())],
                    )
                } else {
                    insight.clone()
                };
                ChecklistRow::new(action_label.clone(), &impact).with_status("warn")
            })
            .collect();
        builder = builder.add_component(ChecklistPanel::new(rows).with_title(panel_title));
    }

    builder
}

fn render_seo_near_duplicates(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.near_duplicates.is_empty() {
        let near_dup_title = i18n.t("batch-seo-near-dup-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-page-b")),
            TableColumn::new(i18n.t("batch-col-similarity")),
            TableColumn::new(i18n.t("batch-col-risk")),
        ])
        .with_title(near_dup_title);

        for (url_a, url_b, sim) in &pres.portfolio_summary.near_duplicates {
            let risk = if *sim >= 95 {
                i18n.t("batch-seo-risk-high")
            } else {
                i18n.t("batch-seo-risk-medium")
            };
            table = table.add_row(vec![
                truncate_url(url_a, 35),
                truncate_url(url_b, 35),
                format!("{sim}%"),
                risk.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_duplicate_content(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.duplicate_content.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-dup-value")),
            TableColumn::new(i18n.t("batch-col-dup-count")),
            TableColumn::new(i18n.t("batch-col-pages-list")),
        ])
        .with_title(i18n.t("batch-seo-duplicate-title"));

        for group in &pres.portfolio_summary.duplicate_content {
            let kind_label = match group.kind.as_str() {
                "title" => i18n.t("batch-dup-kind-title"),
                "meta_description" => i18n.t("batch-dup-kind-description"),
                "h1" => i18n.t("batch-dup-kind-h1"),
                "og_image" => i18n.t("batch-dup-kind-og-image"),
                other => other.to_string(),
            };
            let examples = group
                .urls
                .iter()
                .take(3)
                .map(|u| truncate_url(u, 30))
                .collect::<Vec<_>>()
                .join(", ");
            table = table.add_row(vec![
                kind_label,
                group.value.clone(),
                group.urls.len().to_string(),
                examples,
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_missing_tags(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.missing_tag_prevalence.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-missing-count")),
            TableColumn::new(i18n.t("batch-col-missing-total")),
        ])
        .with_title(i18n.t("batch-seo-missing-tags-title"));

        for entry in &pres.portfolio_summary.missing_tag_prevalence {
            let kind_label = match entry.kind.as_str() {
                "meta_description" => i18n.t("batch-dup-kind-description"),
                "canonical" => i18n.t("batch-missing-kind-canonical"),
                "og_image" => i18n.t("batch-missing-kind-og-image"),
                "og_title" => i18n.t("batch-missing-kind-og-title"),
                other => other.to_string(),
            };
            table = table.add_row(vec![
                kind_label,
                entry.missing_count.to_string(),
                entry.total_count.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_canonical_issues(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.canonical_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-dup-value")),
        ])
        .with_title(i18n.t("batch-seo-canonical-title"));

        for issue in &pres.portfolio_summary.canonical_issues {
            let kind_label = match issue.kind.as_str() {
                "noindex_conflict" => i18n.t("batch-canonical-noindex"),
                "og_url_mismatch" => i18n.t("batch-canonical-ogurl"),
                other => other.to_string(),
            };
            let detail = if issue.detail.chars().count() > 50 {
                format!("{}…", issue.detail.chars().take(50).collect::<String>())
            } else {
                issue.detail.clone()
            };
            table = table.add_row(vec![kind_label, truncate_url(&issue.url, 35), detail]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_hreflang_issues(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.hreflang_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-hreflang-source")),
            TableColumn::new(i18n.t("batch-col-hreflang-target")),
            TableColumn::new(i18n.t("batch-col-hreflang-lang")),
        ])
        .with_title(i18n.t("batch-seo-hreflang-title"));

        for issue in &pres.portfolio_summary.hreflang_issues {
            table = table.add_row(vec![
                truncate_url(&issue.source_url, 32),
                truncate_url(&issue.target_url, 32),
                issue.lang.clone(),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_sitemap_http_issues(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.sitemap_http_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-status-code")),
            TableColumn::new(i18n.t("batch-col-dup-value")),
        ])
        .with_title(i18n.t("batch-seo-sitemap-http-title"));

        for issue in &pres.portfolio_summary.sitemap_http_issues {
            let kind_label = match issue.kind.as_str() {
                "status" => i18n.t("batch-sitemap-kind-status"),
                "redirect" => i18n.t("batch-sitemap-kind-redirect"),
                "noindex" => i18n.t("batch-sitemap-kind-noindex"),
                "fetch_error" => i18n.t("batch-sitemap-kind-fetch-error"),
                other => other.to_string(),
            };
            let detail = issue
                .final_url
                .as_ref()
                .map(|url| truncate_url(url, 35))
                .unwrap_or_else(|| truncate_url(&issue.detail, 45));
            table = table.add_row(vec![
                kind_label,
                truncate_url(&issue.url, 35),
                issue
                    .status_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "n/a".to_string()),
                detail,
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_sitemap_orphans(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.orphan_sitemap_urls.is_empty()
        || !pres.portfolio_summary.linked_not_in_sitemap.is_empty()
    {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-dup-type")),
            TableColumn::new(i18n.t("batch-col-page-a")),
        ])
        .with_title(i18n.t("batch-seo-sitemap-orphan-title"));

        for url in pres.portfolio_summary.orphan_sitemap_urls.iter().take(20) {
            table = table.add_row(vec![
                i18n.t("batch-sitemap-kind-orphan"),
                truncate_url(url, 50),
            ]);
        }
        for url in pres.portfolio_summary.linked_not_in_sitemap.iter().take(20) {
            table = table.add_row(vec![
                i18n.t("batch-sitemap-kind-linked-missing"),
                truncate_url(url, 50),
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_robots_conflicts(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.robots_conflicts.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-page-a")),
            TableColumn::new(i18n.t("batch-col-robots-rule")),
        ])
        .with_title(i18n.t("batch-seo-robots-conflicts-title"));

        for conflict in &pres.portfolio_summary.robots_conflicts {
            table = table.add_row(vec![truncate_url(&conflict.url, 45), conflict.rule.clone()]);
        }
        builder = builder.add_component(table);
    }

    builder
}

fn render_seo_crawl_depth(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if let Some(diag) = &pres.portfolio_summary.crawl_depth_diagnostics {
        let histogram_summary = diag
            .depth_histogram
            .iter()
            .map(|(depth, count)| {
                i18n.t_args(
                    "batch-crawl-depth-histogram-entry",
                    &[("depth", depth.to_string()), ("count", count.to_string())],
                )
            })
            .collect::<Vec<_>>()
            .join(" · ");
        let intro = i18n.t_args(
            "batch-crawl-depth-intro",
            &[
                ("start_url", truncate_url(&diag.start_url, 50)),
                ("histogram", histogram_summary),
            ],
        );

        builder = builder
            .add_component(Section::new(i18n.t("batch-crawl-depth-section")).with_level(1))
            .add_component(TextBlock::new(intro));

        if diag.partial_batch {
            builder = builder.add_component(Callout::info(i18n.t("batch-crawl-depth-caveat")));
        }

        if !diag.deepest_pages.is_empty() || !diag.unreachable_pages.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new(i18n.t("batch-col-dup-type")),
                TableColumn::new(i18n.t("batch-col-page-a")),
                TableColumn::new(i18n.t("batch-col-crawl-depth")),
            ])
            .with_title(i18n.t("batch-crawl-depth-title"));

            for entry in &diag.deepest_pages {
                table = table.add_row(vec![
                    i18n.t("batch-crawl-depth-kind-deepest"),
                    truncate_url(&entry.url, 50),
                    entry.depth.to_string(),
                ]);
            }
            for url in &diag.unreachable_pages {
                table = table.add_row(vec![
                    i18n.t("batch-crawl-depth-kind-unreachable"),
                    truncate_url(url, 50),
                    "—".to_string(),
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    builder
}

fn render_seo_page_type_distribution(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.page_type_distribution.is_empty() {
        let high_label = i18n.t("batch-relevance-high");
        let medium_label = i18n.t("batch-relevance-medium");
        let low_label = i18n.t("batch-relevance-low");
        let mut type_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-pages-list")),
            TableColumn::new(i18n.t("batch-col-share")),
            TableColumn::new(i18n.t("batch-col-relevance")),
        ])
        .with_title(i18n.t("batch-table-page-type-distribution"));

        for (label, count, pct) in &pres.portfolio_summary.page_type_distribution {
            let relevance = match label.as_str() {
                "Marketing / Landing Page"
                | "Transaktional / Utility"
                | "Transactional / Utility" => &high_label,
                "Editorial / Artikel"
                | "Editorial / Article"
                | "Strukturierter Wissensinhalt"
                | "Structured knowledge content" => &medium_label,
                "Thin / Minimal Content" => &low_label,
                _ => &medium_label,
            };
            type_table = type_table.add_row(vec![
                label.clone(),
                count.to_string(),
                format!("{pct}%"),
                relevance.to_string(),
            ]);
        }
        builder = builder.add_component(type_table);
    }

    builder
}

fn render_seo_schema_distribution(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.schema_distribution.is_empty() {
        let total = pres.portfolio_summary.total_urls;
        let without = pres.portfolio_summary.pages_without_schema;
        let summary = if without == 0 {
            i18n.t_args("batch-schema-summary-all", &[("total", total.to_string())])
        } else {
            i18n.t_args(
                "batch-schema-summary-some",
                &[
                    ("without", without.to_string()),
                    ("total", total.to_string()),
                ],
            )
        };
        let schema_callout_title = i18n.t("batch-schema-callout-title");
        builder = builder.add_component(Callout::info(&summary).with_title(schema_callout_title));
        let mut schema_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-schema-type")).with_width("55%"),
            TableColumn::new(i18n.t("batch-col-pages-list")).with_width("20%"),
            TableColumn::new(i18n.t("batch-col-share")).with_width("25%"),
        ])
        .with_title(i18n.t("batch-table-schema-distribution"));
        for (schema_type, count) in &pres.portfolio_summary.schema_distribution {
            let pct = (*count * 100).checked_div(total).unwrap_or(0);
            schema_table = schema_table.add_row(vec![
                schema_type.clone(),
                count.to_string(),
                format!("{pct}%"),
            ]);
        }
        builder = builder.add_component(schema_table);
    }

    builder
}

fn render_seo_strongest_pages(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !pres.portfolio_summary.strongest_content_pages.is_empty() {
        let mut strengths = AuditTable::new(vec![
            TableColumn::new("URL"),
            TableColumn::new(i18n.t("batch-col-page-type")),
            TableColumn::new(i18n.t("batch-col-profile")),
        ])
        .with_title(i18n.t("batch-table-top-pages"));

        for (url, page_type, score) in &pres.portfolio_summary.strongest_content_pages {
            strengths = strengths.add_row(vec![
                truncate_url(url, 42),
                page_type.clone(),
                format!("{score}/100"),
            ]);
        }
        builder = builder.add_component(strengths);
    }

    builder
}
