use crate::output::localized::is_english;
use renderreport::components::advanced::KeyValueList;
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

/// How far a cross-page item reaches, in the words of the batch cards: all
/// audited pages, some of them, or one page. The one wording for the cards
/// and the action plan, so the two cannot name the scope differently (#1039).
pub(super) fn batch_reach_label(urls: usize, total_urls: usize, i18n: &I18n) -> String {
    if urls >= total_urls {
        i18n.t("batch-meta-global")
    } else if urls <= 1 {
        i18n.t("batch-meta-single-page")
    } else {
        i18n.t_args(
            "batch-meta-some-pages",
            &[("count", urls as i64), ("total", total_urls as i64)],
        )
    }
}

/// Cross-page signals of the technical modules that no finding card carries:
/// shared render-blocking files, minification across pages, redirect chains
/// and performance budgets.
///
/// The overview of render-blocking resources is part of the compact technical
/// part; the file tables, the minification, redirect chain and budget tables
/// are per-asset detail and follow with `detail` (report level technical).
pub(super) fn render_batch_technical_signals(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    detail: bool,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // Render blocking
    if !pres.portfolio_summary.render_blocking_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("batch-render-blocking-kv-title"));
        for (label, value) in &pres.portfolio_summary.render_blocking_summary {
            kv = kv.add(label, value);
        }
        builder = builder
            .add_component(Section::new(i18n.t("batch-render-blocking-section")).with_level(2))
            .add_component(TextBlock::new(i18n.t("batch-render-blocking-intro")))
            .add_component(kv);

        // The same files across pages: one fix reaches every page
        // listed, which the summed count above hides.
        let shared = &pres.portfolio_summary.shared_blocking_resources;
        if detail && !shared.is_empty() {
            let mut table = AuditTable::new(vec![
                TableColumn::new(i18n.t("batch-col-minify-asset")).with_width("58%"),
                TableColumn::new(i18n.t("batch-col-minify-kind")).with_width("12%"),
                TableColumn::new(i18n.t("batch-col-pages-list")).with_width("15%"),
                TableColumn::new(i18n.t("batch-col-size")).with_width("15%"),
            ])
            .with_title(i18n.t("batch-shared-blocking-title"));
            for res in shared {
                table = table.add_row(vec![
                    crate::util::short_url(&res.url, 60),
                    res.kind.clone(),
                    format!("{} / {}", res.pages, pres.portfolio_summary.total_urls),
                    if res.transfer_bytes > 0 {
                        format!(
                            "{} KB",
                            crate::i18n::number::decimal(
                                res.transfer_bytes as f64 / 1024.0,
                                1,
                                is_english(i18n)
                            )
                        )
                    } else {
                        "–".to_string()
                    },
                ]);
            }
            builder = builder.add_component(table);
        }
    }

    if !detail {
        return builder;
    }

    // Cross-page minification consistency (#537)
    if !pres
        .portfolio_summary
        .minification_inconsistencies
        .is_empty()
    {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-minify-asset")),
            TableColumn::new(i18n.t("batch-col-minify-kind")),
            TableColumn::new(i18n.t("batch-col-minify-minified-on")),
            TableColumn::new(i18n.t("batch-col-minify-unminified-on")),
        ])
        .with_title(i18n.t("batch-minification-inconsistency-title"));

        for entry in &pres.portfolio_summary.minification_inconsistencies {
            let kind_label = match entry.kind.as_str() {
                "script" => i18n.t("batch-minify-kind-script"),
                "css" => i18n.t("batch-minify-kind-css"),
                other => other.to_string(),
            };
            table = table.add_row(vec![
                short_url(&entry.url, 45),
                kind_label,
                entry.minified_on_count.to_string(),
                entry.unminified_on_count.to_string(),
            ]);
        }
        let minify_intro = i18n.t("batch-minification-inconsistency-intro");
        builder = builder
            .add_component(
                Section::new(i18n.t("batch-minification-inconsistency-section")).with_level(2),
            )
            .add_component(TextBlock::new(minify_intro))
            .add_component(table);
    }

    // Redirect chains and loops across audited URLs (#546)
    if !pres.portfolio_summary.redirect_chain_issues.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-redirect-url")),
            TableColumn::new(i18n.t("batch-col-redirect-hops")),
            TableColumn::new(i18n.t("batch-col-redirect-type")),
            TableColumn::new(i18n.t("batch-col-redirect-chain")),
        ])
        .with_title(i18n.t("batch-redirect-chain-issues-title"));

        for entry in &pres.portfolio_summary.redirect_chain_issues {
            let type_label = if entry.is_loop {
                i18n.t("batch-redirect-chain-type-loop")
            } else {
                i18n.t("batch-redirect-chain-type-long")
            };
            let chain_str = entry
                .chain
                .iter()
                .map(|u| short_url(u, 20))
                .collect::<Vec<_>>()
                .join(" → ");
            table = table.add_row(vec![
                short_url(&entry.url, 35),
                entry.hop_count.to_string(),
                type_label,
                chain_str,
            ]);
        }
        let redirect_intro = i18n.t("batch-redirect-chain-issues-intro");
        builder = builder
            .add_component(
                Section::new(i18n.t("batch-redirect-chain-issues-section")).with_level(2),
            )
            .add_component(TextBlock::new(redirect_intro))
            .add_component(table);
    }

    // Performance budgets
    if !pres.portfolio_summary.budget_summary.is_empty() {
        let pages_col = i18n.t("batch-budget-pages-col");
        let budget_table_title = i18n.t("batch-budget-table-title");
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-metric")),
            TableColumn::new(i18n.t("batch-col-budget")),
            TableColumn::new(pages_col),
            TableColumn::new(i18n.t("batch-col-severity")),
        ])
        .with_title(budget_table_title);
        for (metric, budget, count, sev) in &pres.portfolio_summary.budget_summary {
            table = table.add_row(vec![
                metric.clone(),
                budget.clone(),
                count.to_string(),
                sev.clone(),
            ]);
        }
        let budgets_intro = i18n.t("batch-budget-intro");
        builder = builder
            .add_component(Section::new(i18n.t("batch-section-performance-budgets")).with_level(2))
            .add_component(TextBlock::new(budgets_intro))
            .add_component(table);
    }

    builder
}
