use crate::output::localized::is_english;
use renderreport::components::advanced::{KeyValueList, PageBreak, SectionHeaderSplit};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

use super::super::helpers::{effort_short_label_i18n, priority_label_i18n, role_label_i18n};
use super::management::{batch_cause_text, batch_score_effect};

/// Decision level of the batch report: the most important recurring problems
/// as one line each (finding id, title, reach, priority). The cards of the
/// recurring-problems section tell each of them once in full.
pub(super) fn build_batch_decision_lines(pres: &BatchPresentation, i18n: &I18n) -> Vec<String> {
    pres.top_issues
        .iter()
        .take(BATCH_DECISION_LINES)
        .map(|group| {
            let total_urls = pres.portfolio_summary.total_urls;
            let urls = group.affected_urls.len();
            let reach_in_title = crate::audit::finding_titles::display_title_names_reach(
                &group.finding.rule_id,
                crate::audit::finding_titles::FindingReach {
                    pages_affected: urls.max(1),
                    pages_audited: total_urls,
                    occurrences: group.finding.occurrence_count,
                },
            );
            i18n.t_args(
                if reach_in_title {
                    "batch-decision-line-reach-in-title"
                } else {
                    "batch-decision-line"
                },
                &[
                    ("id", group.finding.id.clone()),
                    ("title", group.title.clone()),
                    ("reach", batch_reach_label(urls, total_urls, i18n)),
                    ("priority", priority_label_i18n(group.priority, i18n)),
                ],
            )
        })
        .collect()
}

/// How many problems the decision level names.
const BATCH_DECISION_LINES: usize = 5;

/// How many recurring problems get a card; the rest is in the JSON report.
const BATCH_PROBLEM_CARDS: usize = crate::audit::normalized::BATCH_TOP_ACTION_COUNT;

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

/// Closing section for batch report. The action plan above already lists
/// every action with its scope; this section only adds what the automated
/// audit leaves open (a repeat of the first three actions with a bare rank
/// was dropped, #1039).
pub(super) fn render_next_steps_batch(
    mut builder: renderreport::engine::ReportBuilder,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            i18n.t("section-next-steps-recommended"),
            i18n.t("batch-next-steps-intro"),
        )
        .with_level(1),
    );

    // The share comes from the criteria list the JSON reports as
    // `summary.wcag_coverage`, not from a fixed "30–40%" (#1039).
    let (automated, total) = crate::wcag::coverage::coverage_stats();
    let callout_body = i18n.t_args(
        "batch-next-steps-manual",
        &[("automated", automated as i64), ("total", total as i64)],
    );
    builder = builder
        .add_component(Callout::info(&callout_body).with_title(i18n.t("section-next-steps-block")));

    builder
}

/// The one cross-page section for recurring problems and actions: one card
/// per problem with its reach, the cause and how sure the report is about it,
/// and the measure, with the finding id below. It replaces the frequency
/// table, the template list, the top-actions table, the problem blocks and
/// the action plan, which told the same problems four times.
pub(super) fn render_batch_recurring_problems(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // The cards are the centre of the batch report: they start a page, so
    // the header never stands alone at the foot of the previous one.
    builder = builder.add_component(PageBreak::new()).add_component(
        SectionHeaderSplit::new(
            i18n.t("batch-section-recurring"),
            i18n.t("batch-recurring-intro"),
        )
        .with_level(1),
    );

    let total_urls = pres.portfolio_summary.total_urls;
    for group in pres.top_issues.iter().take(BATCH_PROBLEM_CARDS) {
        let urls = group.affected_urls.len();
        let reach = format!(
            "{} · {}",
            i18n.count("batch-meta-occurrences", group.finding.occurrence_count),
            batch_reach_label(urls, total_urls, i18n)
        );
        let mut kv = KeyValueList::new()
            .with_title(&group.title)
            .add(i18n.t("batch-recurring-key-reach"), reach)
            .add(
                i18n.t("batch-recurring-key-cause"),
                batch_cause_text(group, pres, i18n),
            );
        // Contrast measured under opacity: name the cause, not only the
        // ratio (#1112).
        if let Some(opacity) = super::super::findings::opacity_pattern_text(
            &group.finding.occurrences,
            is_english(i18n),
        ) {
            kv = kv.add(i18n.t("pdf-findings-opacity"), opacity);
        }
        // The provider is named once on the cross-page card, not once per URL (#1109).
        if let Some(origin) = super::super::findings::origin_text(&group.finding, i18n) {
            kv = kv.add(i18n.t("pdf-findings-origin"), origin);
        }
        if !group.recommendation.is_empty() {
            kv = kv.add(i18n.t("batch-meta-fix"), &group.recommendation);
        }
        kv = kv.add(
            i18n.t("batch-recurring-key-priority"),
            i18n.t_args(
                "batch-recurring-priority-effort",
                &[
                    ("priority", priority_label_i18n(group.priority, i18n)),
                    ("effort", effort_short_label_i18n(group.effort, i18n)),
                    ("role", role_label_i18n(group.responsible_role, i18n)),
                ],
            ),
        );
        if let Some(effect) = batch_score_effect(group, i18n) {
            kv = kv.add(i18n.t("batch-management-col-score-gain"), effect);
        }
        kv = kv.add(i18n.t("batch-recurring-key-finding"), &group.finding.id);
        builder = builder.add_component(kv);
    }
    if pres.top_issues.len() > BATCH_PROBLEM_CARDS {
        builder = builder.add_component(TextBlock::new(i18n.t_args(
            "batch-recurring-more",
            &[(
                "count",
                (pres.top_issues.len() - BATCH_PROBLEM_CARDS) as i64,
            )],
        )));
    }
    if pres
        .top_issues
        .iter()
        .take(BATCH_PROBLEM_CARDS)
        .any(|g| batch_score_effect(g, i18n).is_some())
    {
        builder = builder.add_component(TextBlock::new(i18n.t("batch-management-score-gain-note")));
    }

    builder
}

/// Cross-page signals of the technical modules that no finding card carries:
/// shared render-blocking files, minification across pages, redirect chains
/// and performance budgets.
pub(super) fn render_batch_technical_signals(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    // Render blocking
    if !pres.portfolio_summary.render_blocking_summary.is_empty() {
        let mut kv = KeyValueList::new().with_title(i18n.t("batch-render-blocking-kv-title"));
        for (label, value) in &pres.portfolio_summary.render_blocking_summary {
            kv = kv.add(label, value);
        }
        builder = builder
            .add_component(Section::new(i18n.t("batch-render-blocking-section")).with_level(1))
            .add_component(TextBlock::new(i18n.t("batch-render-blocking-intro")))
            .add_component(kv);

        // The same files across pages: one fix reaches every page
        // listed, which the summed count above hides.
        let shared = &pres.portfolio_summary.shared_blocking_resources;
        if !shared.is_empty() {
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
                Section::new(i18n.t("batch-minification-inconsistency-section")).with_level(1),
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
                Section::new(i18n.t("batch-redirect-chain-issues-section")).with_level(1),
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
            .add_component(Section::new(i18n.t("batch-section-performance-budgets")).with_level(1))
            .add_component(TextBlock::new(budgets_intro))
            .add_component(table);
    }

    builder
}
