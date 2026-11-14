//! Text export of `actions_block.rs`, slice 18 of #936: every text the block
//! builds (systemic and local column labels and descriptions, block title and
//! intro, risk effect, leverage, and the localized enum labels it passes on),
//! across an enumerated input space in both languages, as records in
//! `reports/actions-block-texts.json` (gitignored; the path can be set with
//! `ACTIONS_BLOCK_TEXT_EXPORT`). Run it before and after a change of how the
//! texts are stored and compare the two files byte for byte.

use super::*;
use crate::audit::summary::SiteState;
use crate::output::report_model::{ExecutionPriority, Role, RoleAssignment};
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

const PRIORITIES: [Priority; 4] = [
    Priority::Critical,
    Priority::High,
    Priority::Medium,
    Priority::Low,
];

const EFFORTS: [Effort; 3] = [Effort::Quick, Effort::Medium, Effort::Structural];

const EXECUTION: [ExecutionPriority; 3] = [
    ExecutionPriority::Immediate,
    ExecutionPriority::Important,
    ExecutionPriority::Optional,
];

const ROLES: [Role; 4] = [
    Role::Development,
    Role::Editorial,
    Role::DesignUx,
    Role::ProjectManagement,
];

/// Action sentences that reach several branches of the user effect, in both
/// languages, plus one that reaches none.
const ACTIONS: [&str; 6] = [
    "Label interactive elements",
    "Alternativtext für Bilder ergänzen",
    "Increase contrast of text",
    "Überschriften korrekt strukturieren",
    "Set the page title",
    "Reduce layout shift",
];

fn item(
    priority: Priority,
    effort: Effort,
    systemic: bool,
    occurrences: usize,
    seq: usize,
) -> ActionItem {
    ActionItem {
        action: format!("{} #{seq}", ACTIONS[seq % ACTIONS.len()]),
        hint: if seq.is_multiple_of(2) {
            String::new()
        } else {
            "Hint text".to_string()
        },
        benefit: "Benefit text".to_string(),
        role: ROLES[seq % ROLES.len()],
        priority,
        execution_priority: EXECUTION[seq % EXECUTION.len()],
        effort,
        is_systemic: systemic,
        occurrence_count: occurrences,
        rule_id: format!("rule/{seq}"),
        affected_urls: Vec::new(),
    }
}

fn plan(
    quick_wins: Vec<ActionItem>,
    medium_term: Vec<ActionItem>,
    structural: Vec<ActionItem>,
) -> ActionPlan {
    ActionPlan {
        quick_wins,
        medium_term,
        structural,
        role_assignments: vec![],
    }
}

/// Every combination of priority, effort, systemic flag and occurrence tier.
fn full_plan(systemic_filter: Option<bool>) -> ActionPlan {
    let mut p = plan(vec![], vec![], vec![]);
    let mut seq = 0usize;
    for priority in PRIORITIES {
        for effort in EFFORTS {
            for systemic in [true, false] {
                for occurrences in [1usize, 5, 10] {
                    let it = item(priority, effort, systemic, occurrences, seq);
                    seq += 1;
                    if systemic_filter.is_some_and(|f| f != systemic) {
                        continue;
                    }
                    match effort {
                        Effort::Quick => p.quick_wins.push(it),
                        Effort::Medium => p.medium_term.push(it),
                        Effort::Structural => p.structural.push(it),
                    }
                }
            }
        }
    }
    p
}

fn assignments(counts: &[(Role, usize)]) -> Vec<RoleAssignment> {
    counts
        .iter()
        .map(|(role, n)| RoleAssignment {
            role: *role,
            responsibilities: (0..*n).map(|i| format!("task {i}")).collect(),
        })
        .collect()
}

fn block_json(block: &ActionsBlock) -> Value {
    json!({
        "intro_text": block.intro_text,
        "block_title": block.block_title,
        "task_summary": [
            block.task_summary.blocker_count,
            block.task_summary.high_count,
            block.task_summary.medium_count,
            block.task_summary.low_count,
            block.task_summary.total_count,
            block.task_summary.primary_role,
        ],
        "role_assignments": block
            .role_assignments
            .iter()
            .map(|r| json!([format!("{:?}", r.role), r.responsibilities]))
            .collect::<Vec<_>>(),
        "phase_preview": block
            .phase_preview
            .iter()
            .map(|p| json!([
                p.phase_label,
                p.accent_color,
                p.description,
                p.item_count,
                p.top_items
            ]))
            .collect::<Vec<_>>(),
        "columns": block
            .roadmap_columns
            .iter()
            .map(|c| json!({
                "title": c.title,
                "description": c.description,
                "accent_color": c.accent_color,
                "items": c
                    .items
                    .iter()
                    .map(|i| json!([
                        i.action,
                        i.hint,
                        i.role,
                        i.priority,
                        i.execution_priority,
                        i.effort,
                        i.benefit,
                        i.user_effect,
                        i.risk_effect,
                        i.occurrence_count,
                        i.rule_id,
                        i.leverage
                    ]))
                    .collect::<Vec<_>>()
            }))
            .collect::<Vec<_>>(),
    })
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let mut records: Vec<Value> = Vec::new();
    let listed: Vec<String> = (0..12).map(|i| format!("rule/{}", i * 7)).collect();
    let none: Vec<String> = vec![];
    // (name, plan, score, rule ids the key-measures list names)
    let scenarios: Vec<(&str, ActionPlan, f32, &Vec<String>)> = vec![
        // Not a good site: the whole plan, all four priorities, both
        // columns, every leverage tier.
        ("full plan, score 50", full_plan(None), 50.0, &none),
        // Good by score: capped at two per priority, except listed rules.
        (
            "full plan, score 93, none listed",
            full_plan(None),
            93.0,
            &none,
        ),
        (
            "full plan, score 93, some listed",
            full_plan(None),
            93.0,
            &listed,
        ),
        (
            "systemic only, score 50",
            full_plan(Some(true)),
            50.0,
            &none,
        ),
        ("local only, score 50", full_plan(Some(false)), 50.0, &none),
        (
            "systemic only, score 93",
            full_plan(Some(true)),
            93.0,
            &none,
        ),
        ("local only, score 93", full_plan(Some(false)), 93.0, &none),
        // Good by plan shape: no quick wins, at most three others.
        (
            "empty plan, score 40",
            plan(vec![], vec![], vec![]),
            40.0,
            &none,
        ),
        (
            "three medium actions, score 40",
            plan(
                vec![],
                vec![
                    item(Priority::Medium, Effort::Medium, true, 3, 1),
                    item(Priority::High, Effort::Medium, false, 3, 2),
                    item(Priority::Low, Effort::Medium, false, 3, 3),
                ],
                vec![],
            ),
            40.0,
            &none,
        ),
        (
            "one quick win, score 40",
            plan(
                vec![item(Priority::Critical, Effort::Quick, true, 1, 4)],
                vec![],
                vec![],
            ),
            40.0,
            &none,
        ),
    ];
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for (name, base, score, listed) in &scenarios {
            for roles in [
                vec![],
                vec![(Role::Development, 2)],
                vec![(Role::Development, 2), (Role::Editorial, 3)],
                vec![(Role::DesignUx, 3), (Role::ProjectManagement, 3)],
                vec![
                    (Role::Editorial, 1),
                    (Role::DesignUx, 2),
                    (Role::Development, 2),
                ],
            ] {
                let plan = ActionPlan {
                    quick_wins: base.quick_wins.clone(),
                    medium_term: base.medium_term.clone(),
                    structural: base.structural.clone(),
                    role_assignments: assignments(&roles),
                };
                let block = build_actions_block(&i18n, &plan, *score, &SiteState::Polished, listed);
                records.push(json!([
                    "build_actions_block",
                    format!("{name} roles={roles:?}"),
                    lang,
                    block_json(&block)
                ]));
            }
        }
    }
    records
}

#[test]
#[ignore = "writes reports/actions-block-texts.json; run before and after a change of the text storage"]
fn export_actions_block_texts() {
    let records = records();
    let path = std::env::var("ACTIONS_BLOCK_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/actions-block-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} actions block text records written to {path}",
        records.len()
    );
}
