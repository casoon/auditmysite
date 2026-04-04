//! Module overview components for PDF reports.

use renderreport::components::advanced::Grid;
use renderreport::components::charts::{Chart, ChartType};
use renderreport::components::{AuditTable, MetricCard, ScoreCard, TableColumn};
use renderreport::prelude::*;

use crate::output::report_model::*;

use super::helpers::score_quality_color;

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

pub(super) fn build_overall_score_card(score: u32) -> Grid {
    let (grade, description) = match score {
        90..=100 => ("A", "Sehr gut — Maßstab für die Branche"),
        80..=89 => ("B", "Gut — kleinere Schwächen, klarer Kurs"),
        70..=79 => ("C", "Solide — spürbarer Nachbesserungsbedarf"),
        60..=69 => ("D", "Ausbaufähig — strukturelle Defizite"),
        _ => ("F", "Kritisch — dringender Handlungsbedarf"),
    };
    let description = format!("Note {grade}  ·  {description}");
    Grid::new(1).add_item(serde_json::json!({
        "type": "score-card",
        "data": ScoreCard::new("Gesamtscore", score)
            .with_description(description)
            .with_thresholds(70, 50)
            .to_data()
    }))
}

pub(super) fn build_module_radar_chart(modules: &ModulesBlock) -> Chart {
    let data: Vec<(String, f64)> = modules
        .dashboard
        .iter()
        .map(|m| (m.name.clone(), m.score as f64))
        .collect();
    Chart::new("Modulscores im Überblick", ChartType::Radar).add_series("Score", data)
}

/// Build a card-per-module grid: score + status + key lever. No tables.
pub(super) fn build_module_cards_grid(modules: &ModulesBlock) -> Grid {
    let cols: usize = if modules.dashboard.len() <= 3 {
        modules.dashboard.len()
    } else {
        3
    };
    let mut grid = Grid::new(cols).with_item_min_height("140pt");
    for module in &modules.dashboard {
        let status = if module.score >= module.good_threshold {
            "Stark"
        } else if module.score >= module.warn_threshold {
            "Solide"
        } else {
            "Handlungsbedarf"
        };
        let accent = score_quality_color(module.score);
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": MetricCard::new(&module.name, format!("{}/100", module.score))
                .with_subtitle(status.to_string())
                .with_accent_color(accent)
                .with_height("100%")
                .to_data()
        }));
    }
    grid
}

pub(super) enum WasJetztTunContent {
    Table(AuditTable),
    Empty(Callout),
}

/// Build the Top-Hebel table: top findings sorted by occurrence share, max 5 rows.
pub(super) fn build_top_hebel_table(
    findings: &FindingsBlock,
    total_critical_high: usize,
) -> Option<AuditTable> {
    let mut groups: Vec<&FindingGroup> = findings.top_findings.iter().collect();
    if groups.is_empty() {
        return None;
    }
    // Sort by occurrence_count descending
    groups.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));

    let mut table = AuditTable::new(vec![
        TableColumn::new("Problem").with_width("42%"),
        TableColumn::new("Anteil").with_width("13%"),
        TableColumn::new("Wirkung").with_width("45%"),
    ]);

    for group in groups.iter().take(5) {
        let share = if total_critical_high > 0 {
            let pct = group.occurrence_count * 100 / total_critical_high;
            format!("{}%", pct.min(99))
        } else {
            "—".to_string()
        };
        let impact = &group.user_impact;
        table = table.add_row(vec![
            group.title.clone(),
            share,
            if impact.is_empty() {
                group.recommendation.clone()
            } else {
                impact.clone()
            },
        ]);
    }

    Some(table)
}

/// Build the "Was jetzt tun?" task table (max 5 actions)
pub(super) fn build_was_jetzt_tun_table(vm: &ReportViewModel) -> WasJetztTunContent {
    // Collect top items from action roadmap, prioritize by execution priority
    let all_items: Vec<&RoadmapItemData> = vm
        .actions
        .roadmap_columns
        .iter()
        .flat_map(|col| col.items.iter())
        .collect();

    // Sort: Sofort beheben / Direkt angehen first
    let mut sorted: Vec<&RoadmapItemData> = all_items;
    sorted.sort_by_key(|i| {
        let ep = i.execution_priority.as_str();
        if ep.contains("Direkt") || ep.contains("Sofort") {
            0u8
        } else if ep.contains("Nächstes") || ep.contains("Wichtig") {
            1
        } else {
            2
        }
    });

    let selected: Vec<&RoadmapItemData> = sorted.into_iter().take(5).collect();

    if selected.is_empty() {
        return WasJetztTunContent::Empty(
            Callout::success(
                "Keine priorisierten Maßnahmen identifiziert — Qualität sichern und regelmäßige Audits einplanen.",
            )
            .with_title("Aktuell keine offenen Maßnahmen"),
        );
    }

    let table_title = match selected.len() {
        1 => "Maßnahmenplan (1 Maßnahme) — Executive View".to_string(),
        n => format!("Maßnahmenplan ({n} Maßnahmen) — Executive View"),
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
