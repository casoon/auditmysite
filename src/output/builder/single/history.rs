use crate::output::report_model::{HistoryTrendBlock, ReportHistoryPreview};

use super::super::helpers::build_trend_label;

pub(super) fn build_history_trend_block(
    locale: &str,
    preview: &ReportHistoryPreview,
) -> HistoryTrendBlock {
    let trend_label = build_trend_label(
        locale,
        preview.delta_accessibility,
        preview.delta_total_issues,
    );
    let en = locale == "en";

    let trend_interpretation = if en {
        match trend_label.as_str() {
            "Significantly improved" => format!(
                "Accessibility has improved significantly versus the run on {} (+{} points, {} fewer issues).",
                preview.previous_date,
                preview.delta_accessibility,
                -preview.delta_total_issues
            ),
            "Improved" => format!(
                "Accessibility has improved versus the run on {}.",
                preview.previous_date
            ),
            "Stable" => format!(
                "Accessibility is unchanged compared with the run on {}.",
                preview.previous_date
            ),
            "Significantly regressed" => format!(
                "Accessibility has regressed significantly versus the run on {} ({} points, +{} issues). Action needed.",
                preview.previous_date,
                preview.delta_accessibility,
                preview.delta_total_issues
            ),
            _ => format!(
                "Accessibility has slightly regressed versus the run on {}.",
                preview.previous_date
            ),
        }
    } else {
        match trend_label.as_str() {
            "Deutlich verbessert" => format!(
                "Die Barrierefreiheit hat sich gegenüber dem letzten Lauf vom {} deutlich verbessert (+{} Punkte, {} Issues weniger).",
                preview.previous_date,
                preview.delta_accessibility,
                -preview.delta_total_issues
            ),
            "Verbessert" => format!(
                "Die Barrierefreiheit hat sich gegenüber dem letzten Lauf vom {} verbessert.",
                preview.previous_date
            ),
            "Stabil" => format!(
                "Die Barrierefreiheit ist gegenüber dem letzten Lauf vom {} unverändert stabil.",
                preview.previous_date
            ),
            "Deutlich verschlechtert" => format!(
                "Die Barrierefreiheit ist gegenüber dem letzten Lauf vom {} deutlich zurückgegangen ({} Punkte, +{} Issues). Handlungsbedarf.",
                preview.previous_date,
                preview.delta_accessibility,
                preview.delta_total_issues
            ),
            _ => format!(
                "Die Barrierefreiheit ist gegenüber dem letzten Lauf vom {} leicht zurückgegangen.",
                preview.previous_date
            ),
        }
    };

    let summary = if en {
        format!(
            "{} The history covers {} usable snapshots.",
            trend_interpretation, preview.timeline_entries
        )
    } else {
        format!(
            "{} Die Historie umfasst {} verwertbare Snapshots.",
            trend_interpretation, preview.timeline_entries
        )
    };

    let (acc_delta, total_delta, issue_delta, crit_delta, prev_acc, prev_total) = if en {
        (
            "Accessibility delta",
            "Overall delta",
            "Issue delta",
            "Critical+High delta",
            "Previous accessibility",
            "Previous overall",
        )
    } else {
        (
            "Accessibility-Delta",
            "Gesamt-Delta",
            "Issue-Delta",
            "Kritisch+Hoch-Delta",
            "Vorher Accessibility",
            "Vorher Gesamt",
        )
    };

    HistoryTrendBlock {
        previous_date: preview.previous_date.clone(),
        timeline_entries: preview.timeline_entries,
        trend_label,
        summary,
        metrics: vec![
            (
                acc_delta.to_string(),
                format!("{:+}", preview.delta_accessibility),
            ),
            (
                total_delta.to_string(),
                format!("{:+}", preview.delta_overall),
            ),
            (
                issue_delta.to_string(),
                format!("{:+}", preview.delta_total_issues),
            ),
            (
                crit_delta.to_string(),
                format!("{:+}", preview.delta_critical_issues),
            ),
            (
                prev_acc.to_string(),
                preview.previous_accessibility_score.to_string(),
            ),
            (
                prev_total.to_string(),
                preview.previous_overall_score.to_string(),
            ),
        ],
        timeline_rows: preview
            .recent_entries
            .iter()
            .map(|entry| {
                (
                    entry.0.clone(),
                    entry.1.to_string(),
                    entry.2.to_string(),
                    entry.3.clone(),
                    entry.4.to_string(),
                )
            })
            .collect(),
        new_findings: preview.new_findings.clone(),
        resolved_findings: preview.resolved_findings.clone(),
    }
}
