use crate::output::localized::is_english;
use renderreport::components::advanced::{KeyValueList, List, SectionHeaderSplit};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::short_url;

use super::super::findings::first_sentence;
use super::super::helpers::{
    effort_label_i18n, effort_short_label_i18n, priority_label_i18n, role_label_i18n,
};
use super::management::render_batch_decision_actions;

/// Concrete quick actions for batch report
pub(super) fn build_batch_quick_actions(pres: &BatchPresentation, _i18n: &I18n) -> Vec<String> {
    let mut actions: Vec<String> = Vec::new();

    for item in &pres.action_plan.quick_wins {
        if actions.len() >= 3 {
            break;
        }
        let action_lower = item.action.to_lowercase();
        let scope = if action_lower.contains("alle") || action_lower.contains("global") {
            " (global)"
        } else {
            ""
        };
        actions.push(format!("{}{}", item.full_text(), scope));
    }

    // Fallback from top issues
    if actions.is_empty() {
        for group in pres.top_issues.iter().take(3) {
            let rec = first_sentence(&group.recommendation);
            if !rec.is_empty() {
                actions.push(rec.to_string());
            }
        }
    }

    actions
}

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

/// Action plan grouped by effort; the scope is the measured reach of the
/// findings behind each action, not a guess from its wording (#1039).
pub(super) fn render_batch_action_plan_enhanced(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
    total_urls: usize,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let render_section = |mut b: renderreport::engine::ReportBuilder,
                          title: String,
                          items: &[crate::output::report_model::ActionItem],
                          i18n: &I18n|
     -> renderreport::engine::ReportBuilder {
        if items.is_empty() {
            return b;
        }
        b = b.add_component(Section::new(title).with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("column-action")),
            TableColumn::new(i18n.t("column-timeframe")),
            TableColumn::new(i18n.t("batch-meta-scope")),
            TableColumn::new(i18n.t("batch-col-role")),
            TableColumn::new(i18n.t("column-priority")),
        ]);
        for item in items {
            table = table.add_row(vec![
                item.full_text(),
                effort_label_i18n(item.effort, i18n),
                batch_reach_label(item.affected_urls.len(), total_urls, i18n),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        b.add_component(table)
    };

    builder = render_section(
        builder,
        i18n.t("section-quick-wins"),
        &plan.quick_wins,
        i18n,
    );
    builder = render_section(
        builder,
        i18n.t("section-medium-actions"),
        &plan.medium_term,
        i18n,
    );
    builder = render_section(
        builder,
        i18n.t("section-structural-actions"),
        &plan.structural,
        i18n,
    );
    builder
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

pub(super) fn render_batch_top_issues(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let top_intro = i18n.t("batch-top-issues-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-section-most-frequent"), top_intro).with_level(1),
    );

    // Verified template causes — evidence-backed root-cause clusters, ahead
    // of the plain frequency table so the "one fix, N pages" claims are read
    // before the raw occurrence counts.
    // Template causes are the confirmed and likely root causes; an open one
    // makes no claim about a shared component.
    let template_causes: Vec<&TemplateClusterView> = pres
        .template_clusters
        .iter()
        .filter(|c| c.confidence != "open")
        .collect();
    if !template_causes.is_empty() {
        builder = builder.add_component(TextBlock::new(i18n.t("batch-template-section-intro")));
        // A plain bullet list, not a KeyValueList: `cluster.selector` is an
        // arbitrary-length CSS path with no natural wrap points short
        // enough for a KeyValueList's `auto`-sized key column, which could
        // consume nearly the full row width and squeeze `headline` into a
        // one-word-per-line sliver spanning dozens of page-breaking rows
        // (see #518). `headline` already embeds the selector via
        // `{ $selector }` interpolation, so nothing is lost.
        //
        // One entry per place: two rules at the same selector (the skip link
        // on inros-lackner: landmark and positive tabindex) are one place
        // with two findings, listed underneath — not two causes (plan 85).
        let mut template_list = List::new().with_title(i18n.t("batch-template-section-title"));
        let mut seen: Vec<&str> = Vec::new();
        for cluster in &template_causes {
            if seen.contains(&cluster.selector.as_str()) {
                continue;
            }
            seen.push(cluster.selector.as_str());
            let children = template_causes
                .iter()
                .filter(|c| c.selector == cluster.selector)
                .map(|c| renderreport::components::advanced::ListItem {
                    content: pres
                        .top_issues
                        .iter()
                        .find(|g| g.finding.rule_id == c.rule_id)
                        .map(|g| g.title.clone())
                        .unwrap_or_else(|| c.rule_id.clone()),
                    icon: None,
                    children: Vec::new(),
                })
                .collect();
            template_list
                .items
                .push(renderreport::components::advanced::ListItem {
                    content: cluster.headline.clone(),
                    icon: None,
                    children,
                });
        }
        builder = builder.add_component(template_list);
    }

    // Frequency table
    if !pres.issue_frequency.is_empty() {
        let affected_col = i18n.t("batch-col-affected-urls");
        let mut freq_table = AuditTable::new(vec![
            TableColumn::new(i18n.t("batch-col-problem")),
            TableColumn::new("WCAG"),
            TableColumn::new(i18n.t("batch-col-occurrences")),
            TableColumn::new(affected_col),
            TableColumn::new(i18n.t("batch-col-priority")),
        ])
        .with_title(i18n.t("batch-section-most-frequent-violations"));

        for issue in &pres.issue_frequency {
            freq_table = freq_table.add_row(vec![
                issue.problem.clone(),
                issue.wcag.clone(),
                issue.occurrences.to_string(),
                issue.affected_urls.to_string(),
                super::super::helpers::priority_label_i18n(issue.priority, i18n),
            ]);
        }
        builder = builder.add_component(freq_table);
    }

    builder = render_batch_decision_actions(builder, pres, i18n);

    // Unified problem blocks
    let effort_word = i18n.t("batch-meta-effort");
    let scope_word = i18n.t("batch-meta-scope");
    let impact_user_label = i18n.t("batch-meta-impact-user");
    let impact_business_label = i18n.t("batch-meta-impact-business");
    let fix_label = i18n.t("batch-meta-fix");
    let meta_label = i18n.t("batch-meta-classification");

    for group in pres.top_issues.iter().take(5) {
        // Three tiers: a finding on 9 of 20 pages is neither global nor
        // "isolated content" (plan 99).
        let urls = group.affected_urls.len();
        let scope = batch_reach_label(urls, pres.portfolio_summary.total_urls, i18n);
        let affected_urls_word = i18n.t_args("batch-meta-affected-urls", &[("count", urls as i64)]);
        let effort_label = effort_short_label_i18n(group.effort, i18n);
        let meta_line = format!(
            "{} · {} {} · {}: {} · {}: {}",
            i18n.count("batch-meta-occurrences", group.finding.occurrence_count),
            urls,
            affected_urls_word,
            effort_word,
            effort_label,
            scope_word,
            scope
        );

        let (problem, user_impact) = card_problem_and_impact(group);
        let mut kv = KeyValueList::new().with_title(&group.title);
        if let Some(problem) = problem {
            kv = kv.add(i18n.t("findings-card-key-problem"), problem);
        }
        if let Some(user_impact) = user_impact {
            kv = kv.add(&impact_user_label, user_impact);
        }
        kv = kv.add(&impact_business_label, &group.business_impact);
        if !group.typical_cause.is_empty() {
            kv = kv.add(i18n.t("findings-card-key-cause"), &group.typical_cause);
        }
        if !group.recommendation.is_empty() {
            kv = kv.add(&fix_label, &group.recommendation);
        }
        kv = kv.add(&meta_label, meta_line);
        builder = builder.add_component(kv);
    }

    builder
}

/// The problem and user-impact lines of a cross-page card. An HTML
/// conformance finding's text is one occurrence's checker message, with its
/// line:column — page detail, not a statement about the rule across pages.
/// A rule without its own user-impact text repeats the problem there. Both
/// are left out rather than printed as page detail or twice (#1039).
pub(super) fn card_problem_and_impact(group: &FindingGroup) -> (Option<&str>, Option<&str>) {
    if group.finding.rule_id.starts_with("html_conform.") {
        return (None, None);
    }
    let problem = Some(group.customer_description.as_str()).filter(|t| !t.trim().is_empty());
    let user_impact = Some(group.user_impact.as_str())
        .filter(|t| !t.trim().is_empty() && Some(t.trim()) != problem.map(str::trim));
    (problem, user_impact)
}

pub(super) fn render_batch_action_plan_section(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let action_intro = i18n.t("batch-action-plan-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-action-plan-title"), action_intro).with_level(1),
    );
    builder = render_batch_action_plan_enhanced(
        builder,
        &pres.action_plan,
        pres.portfolio_summary.total_urls,
        i18n,
    );

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

        // The same files across pages (plan 93): one fix reaches every page
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
