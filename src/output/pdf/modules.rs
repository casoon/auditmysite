//! Module overview components for PDF reports.

use renderreport::components::advanced::{Grid, List, SectionHeaderSplit};
use renderreport::components::{AuditTable, MetricCard, ScoreCard, TableColumn};
use renderreport::prelude::*;

use super::findings::first_sentence;
use crate::output::report_model::*;

pub(super) fn build_summary_overview(summary: &SummaryBlock) -> Grid {
    let score_card = serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new("Accessibility-Score", summary.score)
            .with_description(format!("Grade {} — {}", summary.grade, summary.maturity_label))
            .with_thresholds(70, 50)
            .with_height("100%")
            .inverted()
            .to_data()
    });
    let mut grid = Grid::new(2)
        .with_item_min_height("132pt")
        .add_item(score_card);

    for metric in summary.metrics.iter().take(3) {
        let mut card = MetricCard::new(&metric.title, &metric.value).with_height("100%");
        if let Some(ref color) = metric.accent_color {
            card = card.with_accent_color(color);
        }
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": card.to_data()
        }));
    }

    grid
}

pub(super) enum WasJetztTunContent {
    Table(AuditTable),
    Empty(Callout),
}

/// Build the improvement table (max 5 actions)
pub(super) fn build_was_jetzt_tun_table(vm: &ReportViewModel) -> WasJetztTunContent {
    let all_items: Vec<&RoadmapItemData> = vm
        .actions
        .roadmap_columns
        .iter()
        .flat_map(|col| col.items.iter())
        .collect();

    let selected: Vec<&RoadmapItemData> = all_items.into_iter().take(5).collect();

    if selected.is_empty() {
        return WasJetztTunContent::Empty(
            Callout::success(
                "Keine offenen Verbesserungsfelder identifiziert — Qualität sichern und regelmäßige Audits einplanen.",
            )
            .with_title("Aktuell keine offenen Verbesserungsfelder"),
        );
    }

    let table_title = match selected.len() {
        1 => "Verbesserungsfelder (1 Punkt)".to_string(),
        n => format!("Verbesserungsfelder ({n} Punkte)"),
    };

    let mut table = AuditTable::new(vec![
        TableColumn::new("Maßnahme"),
        TableColumn::new("Nutzer-Effekt"),
        TableColumn::new("Risiko"),
        TableColumn::new("Rolle / Aufwand"),
    ])
    .with_title(&table_title);

    for item in selected {
        table = table.add_row(vec![
            item.action.clone(),
            item.user_effect.clone(),
            item.risk_effect.clone(),
            format!("{} / {}", item.role, item.effort),
        ]);
    }

    WasJetztTunContent::Table(table)
}

/// Closing section: additional verification notes.
pub(super) fn render_next_steps_single(
    mut builder: renderreport::engine::ReportBuilder,
    vm: &ReportViewModel,
) -> renderreport::engine::ReportBuilder {
    builder = builder.add_component(
        SectionHeaderSplit::new(
            &vm.executive.next_steps_title,
            &vm.executive.next_steps_intro,
        )
        .with_level(2),
    );

    let mut steps: Vec<String> = Vec::new();

    for col in &vm.actions.roadmap_columns {
        for item in &col.items {
            if steps.len() >= 3 {
                break;
            }
            steps.push(item.action.clone());
        }
    }

    // Fallback from findings
    if steps.is_empty() {
        for group in vm.findings.top_findings.iter().take(3) {
            let rec = first_sentence(&group.recommendation);
            if !rec.is_empty() {
                steps.push(rec.to_string());
            }
        }
    }

    if !steps.is_empty() {
        let mut list = List::new();
        for action in &steps {
            list = list.add_item(action);
        }
        builder = builder.add_component(list);
    }

    builder = builder.add_component(
        Callout::info(&vm.executive.next_steps_callout_body)
            .with_title(&vm.executive.next_steps_callout_title),
    );

    builder
}
