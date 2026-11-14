use renderreport::components::advanced::{
    ChecklistPanel, ChecklistRow, KeyValueList, PageBreak, SectionHeaderSplit,
};
use renderreport::components::text::{Label, TextBlock};
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::audit::BatchReport;
use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

use super::super::design::tokens;
use super::super::helpers::severity_label_i18n;

pub(super) fn render_batch_crawl_links(
    mut builder: renderreport::engine::ReportBuilder,
    crawl_links: &crate::output::report_model::CrawlLinkSummary,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let target_col = i18n.t("batch-crawl-col-target");
    let type_col = i18n.t("batch-crawl-col-type");
    let direct_label = i18n.t("batch-crawl-label-direct");
    let hops_label = i18n.t("batch-crawl-label-hops");
    let internal_intro = i18n.t_args(
        "batch-crawl-internal-intro",
        &[
            ("seed", crawl_links.seed_url.clone()),
            ("checked", crawl_links.checked_internal_links.to_string()),
            (
                "broken",
                crawl_links.broken_internal_links.len().to_string(),
            ),
        ],
    );
    builder = builder
        .add_component(Section::new(i18n.t("batch-section-broken-links-internal")).with_level(1))
        .add_component(TextBlock::new(internal_intro));

    if crawl_links.broken_internal_links.is_empty() {
        builder = builder.add_component(Callout::info(i18n.t("batch-crawl-no-broken-internal")));
    } else {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-source")),
            TableColumn::new(target_col.clone()),
            TableColumn::new(i18n.t("batch-col-status-code")),
            TableColumn::new(type_col.clone()),
        ])
        .with_title(i18n.t("batch-table-broken-internal"));

        for row in &crawl_links.broken_internal_links {
            let typ_label = if row.redirect_hops > 0 {
                format!("→{} {}", row.redirect_hops, hops_label)
            } else {
                direct_label.to_string()
            };
            table = table.add_row(vec![
                short_url(&row.source_url, 30),
                short_url(&row.target_url, 38),
                row.status.clone(),
                typ_label,
            ]);
        }

        builder = builder.add_component(table);
    }

    // External broken links
    if !crawl_links.broken_external_links.is_empty() {
        let ext_intro = i18n.t_args(
            "batch-crawl-external-intro",
            &[
                ("checked", crawl_links.checked_external_links.to_string()),
                (
                    "broken",
                    crawl_links.broken_external_links.len().to_string(),
                ),
            ],
        );
        builder = builder
            .add_component(
                Section::new(i18n.t("batch-section-broken-links-external")).with_level(2),
            )
            .add_component(TextBlock::new(ext_intro));

        let mut ext_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-source")),
            TableColumn::new(target_col.clone()),
            TableColumn::new(i18n.t("batch-col-status-code")),
            TableColumn::new(type_col.clone()),
        ])
        .with_title(i18n.t("batch-table-broken-external"));

        for row in &crawl_links.broken_external_links {
            let typ_label = if row.redirect_hops > 0 {
                format!("→{} {}", row.redirect_hops, hops_label)
            } else {
                direct_label.to_string()
            };
            ext_table = ext_table.add_row(vec![
                short_url(&row.source_url, 30),
                short_url(&row.target_url, 38),
                row.status.clone(),
                typ_label,
            ]);
        }

        builder = builder.add_component(ext_table);
    } else if crawl_links.checked_external_links > 0 {
        let ext_clean_msg = i18n.t_args(
            "batch-crawl-external-clean",
            &[("checked", crawl_links.checked_external_links.to_string())],
        );
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-external-links")).with_level(2))
            .add_component(Callout::info(ext_clean_msg));
    }

    // Redirect chains
    if !crawl_links.redirect_chains.is_empty() {
        let chain_intro = i18n.t_args(
            "batch-crawl-redirect-chains-intro",
            &[("count", crawl_links.redirect_chains.len().to_string())],
        );
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-redirect-chains")).with_level(2))
            .add_component(TextBlock::new(chain_intro));

        let mut chain_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-source")),
            TableColumn::new(i18n.t("batch-col-target")),
            TableColumn::new("Hops"),
            TableColumn::new(i18n.t("batch-col-final-url")),
        ])
        .with_title(i18n.t("batch-redirect-chains-title"));

        for chain in &crawl_links.redirect_chains {
            chain_table = chain_table.add_row(vec![
                short_url(&chain.source_url, 28),
                short_url(&chain.target_url, 28),
                chain.hops.to_string(),
                short_url(&chain.final_url, 32),
            ]);
        }

        builder = builder.add_component(chain_table);
    }

    builder
}

pub(super) fn render_batch_appendix(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let appendix_intro = i18n.t("batch-appendix-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("section-appendix"), appendix_intro).with_level(1),
    );

    let rule_col = i18n.t("batch-appendix-col-rule");
    let elements_col = i18n.t("batch-appendix-col-elements");
    for url_appendix in &pres.appendix.per_url {
        if url_appendix.violations.is_empty() {
            continue;
        }

        builder =
            builder.add_component(Section::new(short_url(&url_appendix.url, 70)).with_level(2));

        let mut table = AuditTable::new(vec![
            TableColumn::new(rule_col.clone()),
            TableColumn::new(i18n.t("batch-col-severity")),
            TableColumn::new(i18n.t("batch-col-description")),
            TableColumn::new(elements_col.clone()),
        ]);

        for v in &url_appendix.violations {
            let elements = v
                .affected_elements
                .iter()
                .map(|e| e.selector.clone())
                .collect::<Vec<_>>()
                .join("; ");
            table = table.add_row(vec![
                format!(
                    "{} — {} ({}×)",
                    v.rule,
                    v.rule_name,
                    v.affected_elements.len()
                ),
                severity_label_i18n(v.severity, i18n),
                v.message.clone(),
                elements,
            ]);
        }
        builder = builder.add_component(table);
    }

    builder
}

// ─── Commerce site-wide roll-up ─────────────────────────────────────────────

fn batch_trust_page_label(key: &str, i18n: &I18n) -> String {
    i18n.t(match key {
        "impressum" => "batch-report-commerce-trust-impressum",
        "agb" => "batch-report-commerce-trust-agb",
        "widerruf" => "batch-report-commerce-trust-widerruf",
        "versand" => "batch-report-commerce-trust-versand",
        "zahlungsarten" => "batch-report-commerce-trust-zahlungsarten",
        _ => "batch-report-commerce-trust-kontakt",
    })
}

/// Site-wide commerce roll-up: which mandatory/trust pages are linked
/// *anywhere* in the audited set, and how many product pages were audited.
/// Only rendered when at least one audited page produced commerce data
/// (i.e. the site was gated as a shop at all) — `aggregate_site_commerce`
/// returns `None` otherwise. Deliberately a single-page-scoped signal, not a
/// checkout/payment/cart audit (this tool has no cross-page session state);
/// the scope note states this explicitly (product decision, 2026-09-01).
pub(super) fn render_batch_commerce(
    mut builder: renderreport::engine::ReportBuilder,
    batch: &BatchReport,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let Some(summary) = crate::commerce::aggregate_site_commerce(
        batch.reports.iter().filter_map(|r| r.commerce.as_ref()),
    ) else {
        return builder;
    };

    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new("Commerce", i18n.t("batch-report-commerce-intro"))
            .with_eyebrow("COMMERCE")
            .with_level(2),
    );
    builder = builder.add_component(
        Label::new(i18n.t("batch-report-commerce-scope-note"))
            .with_size("10.5pt")
            .with_color(tokens::NEUTRAL),
    );

    let product_pages_label = i18n.count(
        "batch-report-commerce-product-pages-audited",
        summary.product_pages,
    );
    builder = builder.add_component(
        KeyValueList::new()
            .with_title(i18n.t("pdf-commerce-overview-title"))
            .add(
                i18n.t("batch-report-commerce-product-pages"),
                &product_pages_label,
            ),
    );

    let tp = &summary.trust_pages_linked;
    let rows =
        [
            ("impressum", tp.impressum),
            ("agb", tp.agb),
            ("widerruf", tp.widerruf),
            ("versand", tp.versand),
            ("zahlungsarten", tp.zahlungsarten),
            ("kontakt", tp.kontakt),
        ]
        .into_iter()
        .map(|(key, linked)| {
            let status_text = i18n.t(if linked {
                "pdf-commerce-linked"
            } else {
                "batch-report-commerce-not-linked"
            });
            ChecklistRow::new(batch_trust_page_label(key, i18n), status_text)
                .with_status(if linked { "good" } else { "warn" })
        })
        .collect::<Vec<_>>();
    builder = builder.add_component(
        ChecklistPanel::new(rows).with_title(i18n.t("batch-report-commerce-trust-title")),
    );

    builder
}
