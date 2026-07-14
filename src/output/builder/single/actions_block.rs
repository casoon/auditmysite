use crate::i18n::I18n;
use crate::output::report_model::{
    ActionItem, ActionPlan, ActionsBlock, PhasePreview, Priority, RoadmapColumnData,
    RoadmapItemData, TaskSummary,
};

use super::super::actions::{derive_conversion_effect_from_action, derive_user_effect_from_action};

pub(super) fn build_actions_block(
    i18n: &I18n,
    plan: &ActionPlan,
    score: f32,
    _site_state: &crate::audit::summary::SiteState,
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
        items
            .iter()
            .take(item_cap)
            .map(|i| {
                let user_effect = derive_user_effect_from_action(i18n, &i.action, i.effort);
                let risk_effect = match (i.priority, en) {
                    (Priority::Critical, true) => {
                        "Directly reduces critical WCAG violation risk".to_string()
                    }
                    (Priority::Critical, false) => {
                        "Reduziert kritisches WCAG-Verstoßrisiko direkt".to_string()
                    }
                    (Priority::High, true) => "Reduces high accessibility risk".to_string(),
                    (Priority::High, false) => {
                        "Reduziert hohes Barrierefreiheitsrisiko".to_string()
                    }
                    (Priority::Medium, true) => "Lowers medium accessibility risk".to_string(),
                    (Priority::Medium, false) => {
                        "Verringert mittleres Barrierefreiheitsrisiko".to_string()
                    }
                    (Priority::Low, true) => "Improves WCAG conformance in detail".to_string(),
                    (Priority::Low, false) => "Verbessert WCAG-Konformität im Detail".to_string(),
                };
                let conversion_effect =
                    derive_conversion_effect_from_action(i18n, &i.action, i.effort);
                RoadmapItemData {
                    action: i.action.clone(),
                    role: i.role.label(en).to_string(),
                    priority: i.priority.label(en).to_string(),
                    execution_priority: i.execution_priority.label(en).to_string(),
                    effort: i.effort.label(en).to_string(),
                    benefit: i.benefit.clone(),
                    user_effect,
                    risk_effect,
                    conversion_effect,
                    occurrence_count: i.occurrence_count,
                    rule_id: i.rule_id.clone(),
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

    let (systemic_label, systemic_desc) = if en {
        (
            "Systemic actions",
            "Fix once in the template or component — this resolves the issue across all affected pages.",
        )
    } else {
        (
            "Systemische Maßnahmen",
            "Einmal in Vorlage oder Komponente beheben — wirkt auf allen betroffenen Seiten zugleich.",
        )
    };
    let (local_label, local_desc) = if en {
        (
            "Local actions & individual cases",
            "Targeted corrections on individual pages, images or editorial content.",
        )
    } else {
        (
            "Lokale Maßnahmen & Einzelfälle",
            "Punktuelle Korrekturen an einzelnen Seiten, Bildern oder redaktionellen Inhalten.",
        )
    };

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
        systemic_label,
        systemic_desc,
        // 4-color palette INFO blue; literal here because the PDF design module
        // is `#[cfg(feature = "pdf")]` and the builder also runs without it.
        "#2563eb",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &local,
        local_label,
        local_desc,
        // 4-color palette NEUTRAL slate (see note above re: pdf feature gate).
        "#475569",
        &mut phase_preview,
        &mut columns,
    );

    // Determine primary responsible role from the largest group
    let primary_role = {
        let mut role_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for r in &plan.role_assignments {
            *role_counts.entry(r.role.label(en)).or_default() += r.responsibilities.len();
        }
        role_counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(r, _)| r.to_string())
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

    let block_title = if is_good_site {
        if en {
            "Last optimization steps".to_string()
        } else {
            "Letzte Optimierungsschritte".to_string()
        }
    } else if en {
        "Action plan by priority".to_string()
    } else {
        "Maßnahmenplan nach Priorität".to_string()
    };

    let intro_text = if is_good_site {
        if en {
            "The site is technically well positioned. The following are final optimization levers without structural pressure.".to_string()
        } else {
            "Die Seite ist technisch stark aufgestellt. Die folgenden Punkte sind letzte Optimierungshebel ohne strukturellen Druck.".to_string()
        }
    } else if en {
        "Priority 1 issues carry the highest risk and should be addressed first. Priority 2 and 3 items follow. Priority 4 items are recommended improvements.".to_string()
    } else {
        "Befunde der Priorität 1 tragen das höchste Risiko und sollten bevorzugt behandelt werden. Danach folgen Priorität 2 und 3. Priorität 4 sind ergänzende Empfehlungen.".to_string()
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
