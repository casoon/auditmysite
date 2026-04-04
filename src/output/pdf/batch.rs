//! Batch report components for PDF output.

use renderreport::components::advanced::Grid;
use renderreport::components::{AuditTable, BenchmarkSummary, MetricCard, TableColumn};
use renderreport::prelude::*;

use crate::i18n::I18n;
use crate::output::report_model::*;
use crate::util::truncate_url;

use super::helpers::{priority_label_i18n, role_label_i18n, score_quality_color};

pub(super) fn build_batch_overview_grid(
    total_urls: u32,
    average_score: u32,
    total_violations: u32,
    critical_and_high: u32,
    broken_internal_links: Option<u32>,
) -> Grid {
    let mut metrics = vec![
        (
            "Durchschnitt",
            format!("{average_score} / 100"),
            Some(score_quality_color(average_score)),
        ),
        ("Geprüfte Websites", total_urls.to_string(), Some("#0f766e")),
        (
            "Verstöße gesamt",
            total_violations.to_string(),
            Some("#b45309"),
        ),
        (
            "Kritisch + Hoch",
            critical_and_high.to_string(),
            Some("#dc2626"),
        ),
    ];

    if let Some(count) = broken_internal_links {
        metrics.push((
            "Broken Links",
            count.to_string(),
            Some(if count > 0 { "#dc2626" } else { "#0f766e" }),
        ));
    }

    let mut grid = Grid::new(2);
    for (title, value, accent) in metrics {
        let mut card = MetricCard::new(title, value);
        if let Some(color) = accent {
            card = card.with_accent_color(color);
        }
        grid = grid.add_item(serde_json::json!({
            "type": "metric-card",
            "data": card.to_data()
        }));
    }

    grid
}

pub(super) fn build_batch_benchmark_summary(pres: &BatchPresentation) -> BenchmarkSummary {
    let dist = &pres.portfolio_summary.severity_distribution;
    let mut summary = BenchmarkSummary::new(
        pres.portfolio_summary.total_urls as u32,
        pres.portfolio_summary.average_score.round() as u32,
    )
    .with_issues(
        pres.portfolio_summary.total_violations as u32,
        dist.critical as u32,
    );
    if let Some(b) = pres.url_ranking.last() {
        summary = summary.with_best(&truncate_url(&b.url, 40), b.score.round() as u32);
    }
    if let Some(w) = pres.url_ranking.first() {
        summary = summary.with_worst(&truncate_url(&w.url, 40), w.score.round() as u32);
    }
    summary
}

/// Render action plan for batch reports (using AuditTable)
pub(super) fn render_action_plan(
    mut builder: renderreport::engine::ReportBuilder,
    plan: &ActionPlan,
    i18n: &I18n,
) -> renderreport::engine::ReportBuilder {
    if !plan.quick_wins.is_empty() {
        builder = builder.add_component(Section::new("Quick Wins").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ]);
        for item in &plan.quick_wins {
            table = table.add_row(vec![
                item.action.clone(),
                item.benefit.clone(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        builder = builder.add_component(table);
    }
    if !plan.medium_term.is_empty() {
        builder = builder.add_component(Section::new("Mittelfristige Maßnahmen").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ]);
        for item in &plan.medium_term {
            table = table.add_row(vec![
                item.action.clone(),
                item.benefit.clone(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        builder = builder.add_component(table);
    }
    if !plan.structural.is_empty() {
        builder = builder.add_component(Section::new("Strukturelle Maßnahmen").with_level(2));
        let mut table = AuditTable::new(vec![
            TableColumn::new("Maßnahme"),
            TableColumn::new("Nutzen"),
            TableColumn::new("Rolle"),
            TableColumn::new("Priorität"),
        ]);
        for item in &plan.structural {
            table = table.add_row(vec![
                item.action.clone(),
                item.benefit.clone(),
                role_label_i18n(item.role, i18n),
                priority_label_i18n(item.priority, i18n),
            ]);
        }
        builder = builder.add_component(table);
    }
    builder
}
