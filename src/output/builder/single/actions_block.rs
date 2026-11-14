use crate::audit::prioritization::Effort;
use crate::i18n::I18n;
use crate::output::report_model::{
    ActionItem, ActionPlan, ActionsBlock, PhasePreview, Priority, RoadmapColumnData,
    RoadmapItemData, TaskSummary,
};

use super::super::actions::derive_user_effect_from_action;

/// plan/6-remediation-leverage-metric.md: a plain-language "is this fix
/// worth doing first" verdict combining reach (`occurrence_count`), cost
/// (`effort`), and risk (`priority`) — fixed rule logic, no new score.
/// `occurrence_count >= 10` mirrors the existing "looks like a shared
/// component" threshold (`FindingGroup::is_component_issue`,
/// `finding_group_from_normalized`) so this tier and that language stay
/// consistent about what counts as a big-reach fix.
fn remediation_leverage(
    occurrence_count: usize,
    effort: Effort,
    priority: Priority,
    i18n: &I18n,
) -> String {
    let tier = if occurrence_count >= 10 {
        2
    } else if occurrence_count >= 5
        || (matches!(priority, Priority::Critical | Priority::High) && effort == Effort::Quick)
    {
        1
    } else {
        0
    };
    i18n.t(match tier {
        2 => "pdf-findings-code-very-high",
        1 => "severity-high",
        _ => "severity-low",
    })
}

pub(super) fn build_actions_block(
    i18n: &I18n,
    plan: &ActionPlan,
    score: f32,
    _site_state: &crate::audit::summary::SiteState,
    listed_rule_ids: &[String],
) -> ActionsBlock {
    let locale = i18n.locale();
    let is_good_site = score >= 85.0
        || (plan.quick_wins.is_empty() && plan.medium_term.len() + plan.structural.len() <= 3);
    let item_cap: usize = if is_good_site { 2 } else { usize::MAX };

    let en = locale == "en";

    // Collect all items from all effort buckets, then re-bucket by semantic priority
    let all_items: Vec<ActionItem> = plan
        .quick_wins
        .iter()
        .chain(plan.medium_term.iter())
        .chain(plan.structural.iter())
        .cloned()
        .collect();

    let mut blockers: Vec<ActionItem> = Vec::new();
    let mut high_prio: Vec<ActionItem> = Vec::new();
    let mut medium_prio: Vec<ActionItem> = Vec::new();
    let mut low_prio: Vec<ActionItem> = Vec::new();

    for item in all_items {
        match item.priority {
            Priority::Critical => blockers.push(item),
            Priority::High => high_prio.push(item),
            Priority::Medium => medium_prio.push(item),
            Priority::Low => low_prio.push(item),
        }
    }

    // Cross-bucket dedup: derive_action_plan deduplicates within effort buckets,
    // but two findings with slightly different raw recommendations can produce the
    // same humanized action text and land in different priority buckets here.
    {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        blockers.retain(|i| seen.insert(i.action.clone()));
        high_prio.retain(|i| seen.insert(i.action.clone()));
        medium_prio.retain(|i| seen.insert(i.action.clone()));
        low_prio.retain(|i| seen.insert(i.action.clone()));
    }

    // Within each bucket sort by execution_priority descending
    let sort_bucket = |mut v: Vec<ActionItem>| -> Vec<ActionItem> {
        v.sort_by_key(|i| std::cmp::Reverse(i.execution_priority));
        v
    };
    let blockers = sort_bucket(blockers);
    let high_prio = sort_bucket(high_prio);
    let medium_prio = sort_bucket(medium_prio);
    let low_prio = sort_bucket(low_prio);

    let map_items = |items: &[ActionItem]| -> Vec<RoadmapItemData> {
        // A good site shows only a short list, but never drops an action the
        // key-measures list names: that list and this plan must agree
        // (#902-8).
        items
            .iter()
            .enumerate()
            .filter(|(idx, i)| *idx < item_cap || listed_rule_ids.contains(&i.rule_id))
            .map(|(_, i)| {
                let user_effect = derive_user_effect_from_action(i18n, &i.action, i.effort);
                let risk_effect = i18n.t(match i.priority {
                    Priority::Critical => "actions-block-risk-critical",
                    Priority::High => "actions-block-risk-high",
                    Priority::Medium => "actions-block-risk-medium",
                    Priority::Low => "actions-block-risk-low",
                });
                RoadmapItemData {
                    action: i.action.clone(),
                    hint: i.hint.clone(),
                    role: i.role.label(en).to_string(),
                    priority: i.priority.label(en).to_string(),
                    execution_priority: i.execution_priority.label(en).to_string(),
                    effort: i.effort.short_label(en),
                    benefit: i.benefit.clone(),
                    user_effect,
                    risk_effect,
                    occurrence_count: i.occurrence_count,
                    rule_id: i.rule_id.clone(),
                    leverage: remediation_leverage(i.occurrence_count, i.effort, i.priority, i18n),
                }
            })
            .collect()
    };

    // #8: group the roadmap by WHERE the problem lives, not by abstract
    // priority. Pull from the already-deduped, priority-sorted buckets so the
    // most urgent action stays first within each level.
    let mut systemic: Vec<ActionItem> = blockers
        .iter()
        .chain(&high_prio)
        .chain(&medium_prio)
        .chain(&low_prio)
        .filter(|i| i.is_systemic)
        .cloned()
        .collect();
    // Re-sort the systemic column by root-cause occurrence count so the
    // biggest root cause leads the list — a stable sort keeps the existing
    // execution_priority ordering as the tie-break for equal counts (#3 fix).
    systemic.sort_by_key(|i| std::cmp::Reverse(i.occurrence_count));
    let local: Vec<ActionItem> = blockers
        .iter()
        .chain(&high_prio)
        .chain(&medium_prio)
        .chain(&low_prio)
        .filter(|i| !i.is_systemic)
        .cloned()
        .collect();

    // #575: this report audits exactly one page, so "affects all pages" is
    // never something the tool actually observed — reworded from an
    // assertion into a hypothesis, with an explicit verification task
    // attached (only these systemic/component-issue actions carry this
    // ambiguity; local actions below don't).
    let systemic_label = i18n.t("actions-block-systemic-label");
    let systemic_desc = i18n.t("actions-block-systemic-desc");
    let local_label = i18n.t("actions-block-local-label");
    let local_desc = i18n.t("actions-block-local-desc");

    let mut phase_preview = Vec::new();
    let mut columns = Vec::new();

    let push_group = |items: &Vec<ActionItem>,
                      label: &str,
                      desc: &str,
                      color: &str,
                      preview: &mut Vec<PhasePreview>,
                      cols: &mut Vec<RoadmapColumnData>| {
        if !items.is_empty() {
            preview.push(PhasePreview {
                phase_label: label.into(),
                accent_color: color.into(),
                description: desc.into(),
                item_count: items.len(),
                top_items: items.iter().map(|i| i.action.clone()).collect(),
            });
            cols.push(RoadmapColumnData {
                title: label.into(),
                description: desc.into(),
                accent_color: color.into(),
                items: map_items(items),
            });
        }
    };

    push_group(
        &systemic,
        &systemic_label,
        &systemic_desc,
        // 4-color palette INFO blue; literal here because the PDF design module
        // is `#[cfg(feature = "pdf")]` and the builder also runs without it.
        "#2563eb",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &local,
        &local_label,
        &local_desc,
        // 4-color palette NEUTRAL slate (see note above re: pdf feature gate).
        "#475569",
        &mut phase_preview,
        &mut columns,
    );

    // Determine primary responsible role from the largest group
    let primary_role = {
        let mut role_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for r in &plan.role_assignments {
            *role_counts.entry(r.role.label(en)).or_default() += r.responsibilities.len();
        }
        role_counts
            .into_iter()
            .max_by(|(a_role, a_count), (b_role, b_count)| {
                a_count.cmp(b_count).then_with(|| b_role.cmp(a_role))
            })
            .map(|(r, _)| r)
            .unwrap_or_default()
    };

    let task_summary = TaskSummary {
        blocker_count: blockers.len(),
        high_count: high_prio.len(),
        medium_count: medium_prio.len(),
        low_count: low_prio.len(),
        total_count: blockers.len() + high_prio.len() + medium_prio.len() + low_prio.len(),
        primary_role,
    };

    let (block_title, intro_text) = if is_good_site {
        (
            i18n.t("actions-block-title-good"),
            i18n.t("actions-block-intro-good"),
        )
    } else {
        (
            i18n.t("actions-block-title-default"),
            i18n.t("actions-block-intro-default"),
        )
    };

    ActionsBlock {
        roadmap_columns: columns,
        role_assignments: plan.role_assignments.clone(),
        intro_text,
        phase_preview,
        block_title,
        task_summary,
    }
}

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::summary::SiteState;
    use crate::output::report_model::{ExecutionPriority, Role};

    fn item(rule_id: &str, occurrences: usize) -> ActionItem {
        ActionItem {
            action: format!("fix {rule_id}"),
            hint: String::new(),
            benefit: String::new(),
            role: Role::Development,
            priority: Priority::Low,
            execution_priority: ExecutionPriority::Important,
            effort: Effort::Medium,
            is_systemic: false,
            occurrence_count: occurrences,
            rule_id: rule_id.to_string(),
            affected_urls: Vec::new(),
        }
    }

    fn planned_rules(block: &ActionsBlock) -> Vec<String> {
        block
            .roadmap_columns
            .iter()
            .flat_map(|c| c.items.iter().map(|i| i.rule_id.clone()))
            .collect()
    }

    /// #902-8: a good site shows a short plan, but every action the
    /// key-measures list names is in it.
    #[test]
    fn plan_carries_every_action_the_key_measures_name() {
        let i18n = I18n::new("de").expect("locale loads");
        let plan = ActionPlan {
            quick_wins: vec![],
            medium_term: vec![item("a", 5), item("b", 4), item("c", 3), item("d", 2)],
            structural: vec![],
            role_assignments: vec![],
        };
        let listed = vec!["c".to_string(), "d".to_string()];
        let block = build_actions_block(&i18n, &plan, 93.0, &SiteState::Polished, &listed);
        let rules = planned_rules(&block);
        for rule in &listed {
            assert!(rules.contains(rule), "{rule} missing from {rules:?}");
        }
    }
}
