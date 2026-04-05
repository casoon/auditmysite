//! Batch report components for PDF output.

use renderreport::components::advanced::Grid;
use renderreport::components::MetricCard;
use renderreport::Component;

use super::helpers::score_quality_color;

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

