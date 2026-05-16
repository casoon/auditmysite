use crate::output::report_model::{
    ActionItem, ActionPlan, ActionsBlock, PhasePreview, Priority, RoadmapColumnData,
    RoadmapItemData, TaskSummary,
};

use super::super::actions::{derive_conversion_effect_from_action, derive_user_effect_from_action};

pub(super) fn build_actions_block(
    locale: &str,
    plan: &ActionPlan,
    score: f32,
    _site_state: &crate::audit::summary::SiteState,
) -> ActionsBlock {
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
                let user_effect = derive_user_effect_from_action(locale, &i.action, i.effort);
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
                    derive_conversion_effect_from_action(locale, &i.action, i.effort);
                RoadmapItemData {
                    action: i.action.clone(),
                    role: i.role.label().to_string(),
                    priority: i.priority.label().to_string(),
                    execution_priority: i.execution_priority.label().to_string(),
                    effort: i.effort.label().to_string(),
                    benefit: i.benefit.clone(),
                    user_effect,
                    risk_effect,
                    conversion_effect,
                }
            })
            .collect()
    };

    // Bucket labels and colors
    let (blocker_label, blocker_desc) = if en {
        (
            "Blocker — fix immediately",
            "Acute barriers — highest risk, must be resolved before anything else",
        )
    } else {
        (
            "Blocker — Sofort beheben",
            "Akute Barrieren — höchstes Risiko, vor allen anderen Punkten beheben",
        )
    };
    let (high_label, high_desc) = if en {
        (
            "High priority",
            "Significant barriers with direct usability impact",
        )
    } else {
        (
            "Hohe Priorität",
            "Relevante Barrieren mit direktem Impact auf Nutzbarkeit",
        )
    };
    let (medium_label, medium_desc) = if en {
        (
            "Medium priority",
            "Quality improvements with moderate accessibility benefit",
        )
    } else {
        (
            "Mittlere Priorität",
            "Qualitätsverbesserungen mit moderatem Barrierefreiheits-Nutzen",
        )
    };
    let (low_label, low_desc) = if en {
        ("Low priority", "Fine-tuning and optional improvements")
    } else {
        (
            "Niedrige Priorität",
            "Feinschliff und optionale Verbesserungen",
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
                accent_color: color.into(),
                items: map_items(items),
            });
        }
    };

    push_group(
        &blockers,
        blocker_label,
        blocker_desc,
        "#dc2626",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &high_prio,
        high_label,
        high_desc,
        "#f59e0b",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &medium_prio,
        medium_label,
        medium_desc,
        "#2563eb",
        &mut phase_preview,
        &mut columns,
    );
    push_group(
        &low_prio,
        low_label,
        low_desc,
        "#6b7280",
        &mut phase_preview,
        &mut columns,
    );

    // Determine primary responsible role from the largest group
    let primary_role = {
        let mut role_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for r in &plan.role_assignments {
            *role_counts.entry(r.role.label()).or_default() += r.responsibilities.len();
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
        "Blockers must be resolved first — they carry the highest risk. High and medium priority items follow. Low priority items are optional improvements.".to_string()
    } else {
        "Blocker zuerst beheben — sie tragen das höchste Risiko. Danach folgen hohe und mittlere Priorität. Niedrige Priorität sind optionale Verbesserungen.".to_string()
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
