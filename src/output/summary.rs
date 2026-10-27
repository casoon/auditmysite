use crate::audit::normalized::NormalizedReport;
use crate::wcag::Severity;
use serde::Serialize;

const TOP_FINDINGS_LIMIT: usize = 10;
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compact export for ranking dashboards (`lastAudit` schema of rankinglab).
///
/// One score semantic (#781): `grade`, `label` and `medalColor` are bands of
/// `accessibilityScore`, the score the report is about and the verdict
/// thresholds compare. `overallScore` is the module-weighted technical value
/// and carries no band. There is no unqualified `score`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryReport {
    pub url: String,
    pub date: String,
    pub accessibility_score: u32,
    pub overall_score: u32,
    pub grade: String,
    pub label: String,
    pub medal_color: String,
    /// The stored decision (`pass` / `warn` / `fail`) and its reasons.
    pub verdict: crate::audit::Verdict,
    pub verdict_reasons: Vec<String>,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub version: String,
    /// Number of distinct WCAG rules with violations.
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
    normalized.validate()?;
    let (label, medal_color) = label_and_medal(normalized.score);

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
            severity: f.severity.as_str().to_string(),
            title: f.title.clone(),
            count: f.occurrence_count,
        })
        .collect();

    let report = SummaryReport {
        url: normalized.url.clone(),
        date: normalized.timestamp.format("%Y-%m-%d").to_string(),
        accessibility_score: normalized.score,
        overall_score: normalized.overall_score,
        grade: normalized.grade.clone(),
        label: label.to_string(),
        medal_color: medal_color.to_string(),
        verdict: normalized.decision.verdict,
        verdict_reasons: normalized.decision.reasons.clone(),
        total_issues: normalized.severity_counts.total,
        critical_issues: normalized.severity_counts.critical,
        high_issues: normalized.severity_counts.high,
        version: format!("auditmysite v{VERSION}"),
        rule_types: normalized
            .findings
            .iter()
            .filter(|f| f.category == "wcag")
            .count(),
        top_findings,
    };

    serde_json::to_string_pretty(&report).map_err(|e| anyhow::anyhow!(e))
}
