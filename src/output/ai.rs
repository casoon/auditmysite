//! AI/LLM-Optimised JSON Output Formatter
//!
//! Produces a compact, task-oriented JSON document from an AuditReport.
//! Each WCAG violation becomes a discrete "task" with a stable ID, sorted
//! by impact severity so the most critical work appears first.

use serde::Serialize;

use crate::audit::AuditReport;

/// Impact sort order — lower number = higher priority.
fn impact_order(impact: &str) -> u8 {
    match impact {
        "critical" => 0,
        "serious" => 1,
        "moderate" => 2,
        "minor" => 3,
        _ => 4,
    }
}

/// A single actionable task derived from a WCAG violation.
#[derive(Debug, Serialize)]
pub struct AiTask {
    pub task_id: String,
    pub rule_id: String,
    pub impact: String,
    pub wcag: String,
    pub tags: Vec<String>,
    pub title: String,
    pub issue: String,
    pub fix: String,
    pub selector: String,
    pub node_id: String,
    pub help_url: String,
}

/// Violation summary counts broken down by impact level.
#[derive(Debug, Serialize)]
pub struct AiSummary {
    pub total_violations: usize,
    pub critical: usize,
    pub serious: usize,
    pub moderate: usize,
    pub minor: usize,
}

/// Report metadata block.
#[derive(Debug, Serialize)]
pub struct AiMetadata {
    pub tool: String,
    pub version: String,
    pub wcag_level: String,
}

/// Top-level AI/LLM-optimised report document.
#[derive(Debug, Serialize)]
pub struct AiReport {
    pub url: String,
    pub audit_date: String,
    pub overall_score: u32,
    pub summary: AiSummary,
    pub tasks: Vec<AiTask>,
    pub passing_checks: usize,
    pub metadata: AiMetadata,
}

/// Build an [`AiReport`] from a raw [`AuditReport`] and serialise it as
/// pretty-printed JSON.
pub fn format_ai_json(report: &AuditReport) -> String {
    let violations = &report.wcag_results.violations;

    // Severity counts
    let critical = violations
        .iter()
        .filter(|v| v.impact.as_deref().unwrap_or(v.impact_str()) == "critical")
        .count();
    let serious = violations
        .iter()
        .filter(|v| v.impact.as_deref().unwrap_or(v.impact_str()) == "serious")
        .count();
    let moderate = violations
        .iter()
        .filter(|v| v.impact.as_deref().unwrap_or(v.impact_str()) == "moderate")
        .count();
    let minor = violations
        .iter()
        .filter(|v| v.impact.as_deref().unwrap_or(v.impact_str()) == "minor")
        .count();

    // Build tasks, then sort by impact severity
    let mut tasks: Vec<AiTask> = violations
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let rule_id = v.rule_id.clone().unwrap_or_else(|| v.rule.clone());
            let impact = v
                .impact
                .clone()
                .unwrap_or_else(|| v.impact_str().to_string());
            AiTask {
                task_id: format!("{}-{:03}", rule_id, idx + 1),
                rule_id,
                impact,
                wcag: v.rule.clone(),
                tags: v.tags.clone(),
                title: v.rule_name.clone(),
                issue: v.message.clone(),
                fix: v.fix_suggestion.clone().unwrap_or_default(),
                selector: v.selector.clone().unwrap_or_default(),
                node_id: v.node_id.clone(),
                help_url: v.help_url.clone().unwrap_or_default(),
            }
        })
        .collect();

    tasks.sort_by_key(|t| impact_order(&t.impact));

    let ai_report = AiReport {
        url: report.url.clone(),
        audit_date: report.timestamp.format("%Y-%m-%d").to_string(),
        overall_score: report.score.round() as u32,
        summary: AiSummary {
            total_violations: violations.len(),
            critical,
            serious,
            moderate,
            minor,
        },
        tasks,
        passing_checks: report.wcag_results.passes,
        metadata: AiMetadata {
            tool: "auditmysite".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            wcag_level: report.wcag_level.to_string(),
        },
    };

    serde_json::to_string_pretty(&ai_report)
        .unwrap_or_else(|e| format!("{{\"error\": \"AI JSON serialization failed: {}\"}}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditReport;
    use crate::cli::WcagLevel;
    use crate::taxonomy::Severity;
    use crate::wcag::{Violation, WcagResults};

    fn make_report_with_violations(violations: Vec<Violation>) -> AuditReport {
        let mut results = WcagResults::new();
        results.passes = 42;
        for v in violations {
            results.add_violation(v);
        }
        AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            500,
        )
    }

    #[test]
    fn test_format_ai_json_empty_violations() {
        let report = make_report_with_violations(vec![]);
        let output = format_ai_json(&report);

        assert!(output.contains("\"url\": \"https://example.com\""));
        assert!(output.contains("\"total_violations\": 0"));
        assert!(output.contains("\"tasks\": []"));
        assert!(output.contains("\"passing_checks\": 42"));
        assert!(output.contains("\"tool\": \"auditmysite\""));
        assert!(output.contains("\"wcag_level\": \"AA\""));
    }

    #[test]
    fn test_format_ai_json_tasks_sorted_by_impact() {
        let v_minor = Violation::new(
            "2.4.6",
            "Headings and Labels",
            WcagLevel::AA,
            Severity::Low,
            "Heading is not descriptive",
            "node-1",
        );
        let v_critical = Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::Critical,
            "Image missing alt text",
            "node-2",
        );
        let v_serious = Violation::new(
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            Severity::High,
            "Form control missing label",
            "node-3",
        );

        let report = make_report_with_violations(vec![v_minor, v_critical, v_serious]);
        let output = format_ai_json(&report);

        // Critical task must appear before serious, which must appear before minor
        let pos_critical = output.find("\"critical\"").unwrap();
        let pos_serious = output.find("\"serious\"").unwrap();
        let pos_minor = output.find("\"minor\"").unwrap_or(usize::MAX);

        // In the tasks array, sorted order should be: critical < serious < minor
        // (the summary block also contains these keys, so find first occurrence in tasks)
        let tasks_start = output.find("\"tasks\"").unwrap();
        let task_impacts: Vec<usize> = ["critical", "serious", "minor"]
            .iter()
            .filter_map(|label| output[tasks_start..].find(label).map(|p| p + tasks_start))
            .collect();

        assert!(
            task_impacts.windows(2).all(|w| w[0] < w[1]),
            "tasks must be sorted: critical before serious before minor"
        );

        // summary counts
        assert!(output.contains("\"total_violations\": 3"));
        let _ = (pos_critical, pos_serious, pos_minor); // suppress unused warnings
    }

    #[test]
    fn test_format_ai_json_task_fields() {
        let v = Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::Critical,
            "Image missing alt text",
            "node-99",
        )
        .with_selector("img.hero")
        .with_fix("Add alt attribute")
        .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html")
        .with_rule_id("image-alt")
        .with_tags(vec!["wcag2a".to_string(), "wcag111".to_string()]);

        let report = make_report_with_violations(vec![v]);
        let output = format_ai_json(&report);

        assert!(output.contains("\"task_id\": \"image-alt-001\""));
        assert!(output.contains("\"rule_id\": \"image-alt\""));
        assert!(output.contains("\"wcag\": \"1.1.1\""));
        assert!(output.contains("\"selector\": \"img.hero\""));
        assert!(output.contains("\"node_id\": \"node-99\""));
        assert!(output.contains("\"fix\": \"Add alt attribute\""));
        assert!(
            output.contains("https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html")
        );
        assert!(output.contains("\"wcag2a\""));
    }
}
