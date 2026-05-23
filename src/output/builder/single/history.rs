use crate::output::report_model::{HistoryTrendBlock, ReportHistoryPreview};

pub(super) fn build_history_trend_block(
    i18n: &crate::i18n::I18n,
    preview: &ReportHistoryPreview,
) -> HistoryTrendBlock {
    let (trend_key, trend_label_key) = if preview.delta_accessibility >= 10
        || (preview.delta_accessibility >= 5 && preview.delta_total_issues <= -5)
    {
        (
            "history-trend-significantly-improved",
            "trend-significantly-improved",
        )
    } else if preview.delta_accessibility > 0 || preview.delta_total_issues < 0 {
        ("history-trend-improved", "trend-improved")
    } else if preview.delta_accessibility == 0 && preview.delta_total_issues == 0 {
        ("history-trend-stable", "trend-stable")
    } else if preview.delta_accessibility >= -5 && preview.delta_total_issues <= 5 {
        (
            "history-trend-slightly-regressed",
            "trend-slightly-regressed",
        )
    } else {
        (
            "history-trend-significantly-regressed",
            "trend-significantly-regressed",
        )
    };

    let trend_label = i18n.t(trend_label_key);

    let trend_interpretation = i18n.t_args(
        trend_key,
        &[
            ("previous_date", preview.previous_date.as_str()),
            (
                "delta_accessibility",
                &preview.delta_accessibility.to_string(),
            ),
            (
                "delta_total_issues",
                &preview.delta_total_issues.to_string(),
            ),
            (
                "delta_issues_abs",
                &(-preview.delta_total_issues).to_string(),
            ),
        ],
    );

    let summary = i18n.t_args(
        "history-summary",
        &[
            ("trend_interpretation", trend_interpretation.as_str()),
            ("timeline_entries", &preview.timeline_entries.to_string()),
        ],
    );

    let acc_delta = i18n.t("history-metric-acc-delta");
    let total_delta = i18n.t("history-metric-total-delta");
    let issue_delta = i18n.t("history-metric-issue-delta");
    let crit_delta = i18n.t("history-metric-crit-delta");
    let prev_acc = i18n.t("history-metric-prev-acc");
    let prev_total = i18n.t("history-metric-prev-total");

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
