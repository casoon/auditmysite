use crate::output::localized::is_english;
use renderreport::components::advanced::{KeyValueList, List, SectionHeaderSplit};
use renderreport::components::text::TextBlock;
use renderreport::components::{AuditTable, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::super::findings::first_sentence;
use super::super::helpers::{effort_label_i18n, priority_label_i18n, role_label_i18n};
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
        actions.push(format!("{}{}", item.action, scope));
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

/// Enhanced action plan with effort + scope columns
pub(super) fn render_batch_action_plan_enhanced(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let effort_col = if en { "Effort" } else { "Aufwand" };
    let role_col = if en { "Role" } else { "Rolle" };
    let scope_global = "global";
    let scope_content = "Content";
    let scope_component = if en { "Component" } else { "Komponente" };

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
            TableColumn::new(effort_col),
            TableColumn::new(if en { "Scope" } else { "Reichweite" }),
            TableColumn::new(role_col),
            TableColumn::new(i18n.t("column-priority")),
        ]);
        for item in items {
            let action_lower = item.action.to_lowercase();
            let scope = if action_lower.contains("alle")
                || action_lower.contains("global")
                || action_lower.contains("designsystem")
                || action_lower.contains("design system")
                || action_lower.contains("seitenübergreifend")
                || action_lower.contains("site-wide")
            {
                scope_global
            } else if action_lower.contains("content")
                || action_lower.contains("text")
                || action_lower.contains("bild")
                || action_lower.contains("image")
            {
                scope_content
            } else {
                scope_component
            };
            table = table.add_row(vec![
                item.action.clone(),
                effort_label_i18n(item.effort, i18n),
                scope.to_string(),
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

/// Closing section for batch report
pub(super) fn render_next_steps_batch(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let en = is_english(i18n);
    let intro = if en {
        "Concrete recommendation for implementation."
    } else {
        "Konkrete Handlungsempfehlung für die Umsetzung."
    };
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("section-next-steps-recommended"), intro).with_level(1),
    );

    let mut steps: Vec<(String, &str)> = Vec::new();

    let scope_global = "global";
    let scope_component = if en {
        "component-based"
    } else {
        "komponentenbasiert"
    };

    // From quick wins
    for item in &pres.action_plan.quick_wins {
        if steps.len() >= 3 {
            break;
        }
        let action_lower = item.action.to_lowercase();
        let scope = if action_lower.contains("alle")
            || action_lower.contains("designsystem")
            || action_lower.contains("design system")
            || action_lower.contains("global")
        {
            scope_global
        } else {
            scope_component
        };
        steps.push((item.action.clone(), scope));
    }

    // Fallback from medium_term
    if steps.len() < 3 {
        for item in &pres.action_plan.medium_term {
            if steps.len() >= 3 {
                break;
            }
            steps.push((item.action.clone(), scope_component));
        }
    }

    if !steps.is_empty() {
        let mut table = AuditTable::new(vec![
            TableColumn::new(i18n.t("column-priority")),
            TableColumn::new(i18n.t("column-action")),
            TableColumn::new(if en { "Scope" } else { "Reichweite" }),
        ]);
        for (i, (action, scope)) in steps.iter().enumerate() {
            table = table.add_row(vec![
                format!("{}", i + 1),
                action.clone(),
                scope.to_string(),
            ]);
        }
        builder = builder.add_component(table);
    }

    let callout_body = if en {
        "For a complete WCAG conformance check we additionally recommend a manual audit with assistive technologies (screen reader, keyboard navigation). This automated audit covers about 30–40% of WCAG criteria."
    } else {
        "Für eine vollständige WCAG-Konformitätsprüfung empfehlen wir ergänzend einen manuellen Audit mit assistiven Technologien (Screenreader, Tastaturnavigation). Dieser automatisierte Audit deckt ca. 30–40% der WCAG-Kriterien ab."
    };
    builder = builder
        .add_component(Callout::info(callout_body).with_title(i18n.t("section-next-steps-block")));

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
    if !pres.template_clusters.is_empty() {
        builder = builder.add_component(TextBlock::new(i18n.t("batch-template-section-intro")));
        // A plain bullet list, not a KeyValueList: `cluster.selector` is an
        // arbitrary-length CSS path with no natural wrap points short
        // enough for a KeyValueList's `auto`-sized key column, which could
        // consume nearly the full row width and squeeze `headline` into a
        // one-word-per-line sliver spanning dozens of page-breaking rows
        // (see #518). `headline` already embeds the selector via
        // `{ $selector }` interpolation, so nothing is lost.
        let mut template_list = List::new().with_title(i18n.t("batch-template-section-title"));
        for cluster in &pres.template_clusters {
            template_list = template_list.add_item(&cluster.headline);
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
    let scope_global_word = i18n.t("batch-meta-global");
    let scope_individual = i18n.t("batch-meta-individual");
    let occurrences_word_top = i18n.t("batch-meta-occurrences");
    let affected_urls_word = i18n.t("batch-meta-affected-urls");
    let effort_word = i18n.t("batch-meta-effort");
    let scope_word = i18n.t("batch-meta-scope");
    let impact_user_label = i18n.t("batch-meta-impact-user");
    let impact_business_label = i18n.t("batch-meta-impact-business");
    let fix_label = i18n.t("batch-meta-fix");
    let meta_label = i18n.t("batch-meta-classification");

    for group in pres.top_issues.iter().take(5) {
        let scope = if group.affected_urls.len() >= pres.portfolio_summary.total_urls {
            &scope_global_word
        } else {
            &scope_individual
        };
        let effort_label = effort_label_i18n(group.effort, i18n);
        let meta_line = format!(
            "{} {} · {} {} · {}: {} · {}: {}",
            group.occurrence_count,
            occurrences_word_top,
            group.affected_urls.len(),
            affected_urls_word,
            effort_word,
            effort_label,
            scope_word,
            scope
        );

        let mut kv = KeyValueList::new()
            .with_title(&group.title)
            .add(
                i18n.t("findings-card-key-problem"),
                &group.customer_description,
            )
            .add(&impact_user_label, &group.user_impact)
            .add(&impact_business_label, &group.business_impact);
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

pub(super) fn render_batch_action_plan_section(
    mut builder: renderreport::engine::ReportBuilder,
    pres: &BatchPresentation,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    let action_intro = i18n.t("batch-action-plan-intro");
    builder = builder.add_component(
        SectionHeaderSplit::new(i18n.t("batch-action-plan-title"), action_intro).with_level(1),
    );
    builder = render_batch_action_plan_enhanced(builder, &pres.action_plan, i18n);

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
                truncate_url(&entry.url, 45),
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
                .map(|u| truncate_url(u, 20))
                .collect::<Vec<_>>()
                .join(" → ");
            table = table.add_row(vec![
                truncate_url(&entry.url, 35),
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
