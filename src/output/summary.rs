use crate::audit::normalized::NormalizedReport;
use crate::wcag::Severity;
use serde::Serialize;

const TOP_FINDINGS_LIMIT: usize = 10;
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compact export for ranking dashboards. Field names match the `lastAudit`
/// schema consumed by rankinglab so the consumer can write this directly.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryReport {
    pub url: String,
    pub date: String,
    pub score: u32,
    pub grade: String,
    pub label: String,
    pub medal_color: String,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub version: String,
    /// Accessibility-only score (equals `score` when no --full modules active).
    pub accessibility_score: u32,
    /// Number of distinct rule categories with violations.
    pub rule_types: usize,
    pub top_findings: Vec<TopFinding>,
}

#[derive(Debug, Serialize)]
pub struct TopFinding {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub count: usize,
}

fn label_and_medal(score: u32) -> (&'static str, &'static str) {
    let label = crate::registry::MEDAL.label(score as f32, false);
    let medal_color = match label {
        "GOLD" => "gold",
        "SILVER" => "silver",
        "BRONZE" => "bronze",
        _ => "failed",
    };
    (label, medal_color)
}

fn severity_rank(s: &Severity) -> u8 {
    match s {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
    }
}

pub fn format_summary(normalized: &NormalizedReport) -> anyhow::Result<String> {
    let score = normalized.overall_score;
    let (label, medal_color) = label_and_medal(score);

    let mut findings = normalized.findings.clone();
    findings.sort_by(|a, b| {
        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            .then(b.occurrence_count.cmp(&a.occurrence_count))
    });

    let top_findings = findings
        .iter()
        .take(TOP_FINDINGS_LIMIT)
        .map(|f| TopFinding {
            id: f.rule_id.clone(),
            severity: f.severity.to_string().to_lowercase(),
            title: f.title.clone(),
            count: f.occurrence_count,
        })
        .collect();

    let report = SummaryReport {
        url: normalized.url.clone(),
        date: normalized.timestamp.format("%Y-%m-%d").to_string(),
        score,
        grade: normalized.grade.clone(),
        label: label.to_string(),
        medal_color: medal_color.to_string(),
        total_issues: normalized.severity_counts.total,
        critical_issues: normalized.severity_counts.critical,
        high_issues: normalized.severity_counts.high,
        version: format!("auditmysite v{VERSION}"),
        accessibility_score: normalized.score,
        rule_types: normalized.findings.len(),
        top_findings,
    };

    serde_json::to_string_pretty(&report).map_err(|e| anyhow::anyhow!(e))
}
