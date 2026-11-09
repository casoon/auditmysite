//! AI/LLM-Optimised JSON Output Formatter
//!
//! Produces a compact, task-oriented JSON document from an AuditReport.
//! Each WCAG violation becomes a discrete "task" with a stable ID, sorted
//! by impact severity so the most critical work appears first.

use serde::Serialize;

use crate::audit::{normalize_checked, AuditReport, ReportContractError};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_snippet: Option<String>,
    /// `findings[].id` of the JSON report; set on annotated tasks only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finding_id: Option<String>,
    /// External assessments of this task's finding (`--annotations`, #961),
    /// labelled by author kind. They are not part of `issue` and change no
    /// impact.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<AiAnnotation>,
}

/// An assessment as a remediation pass reads it.
#[derive(Debug, Serialize)]
pub struct AiAnnotation {
    /// `Assessment (agent)` or `Assessment (reviewer)`.
    pub label: &'static str,
    #[serde(flatten)]
    pub annotation: crate::annotations::Annotation,
}

impl AiAnnotation {
    fn new(annotation: &crate::annotations::Annotation) -> Self {
        Self {
            label: crate::annotations::author_label_en(annotation.author.kind),
            annotation: annotation.clone(),
        }
    }
}

/// Assessments that are not on a task, so they are not lost in this format.
#[derive(Debug, Serialize)]
pub struct AiAnnotations {
    /// Scope `reading_flow`: belongs to the page's reading flow.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reading_flow: Vec<AiAnnotation>,
    /// Matched a finding that has no task here (only WCAG violations are tasks).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub matched_without_task: Vec<AiAnnotation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unmatched: Vec<crate::annotations::Unmatched>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
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
    pub score: u32,
    pub overall_score: u32,
    pub summary: AiSummary,
    pub tasks: Vec<AiTask>,
    pub passing_checks: usize,
    pub metadata: AiMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<AiAnnotations>,
}

/// Build an [`AiReport`] from a raw [`AuditReport`] and serialise it as
/// pretty-printed JSON. A report whose data contradicts itself is refused
/// (#790) with the machine-readable codes and data paths of the contract.
pub fn format_ai_json(report: &AuditReport) -> Result<String, ReportContractError> {
    format_ai_json_with_annotations(report, None)
}

/// [`format_ai_json`] with the assessments of `--annotations` (#961): annotated
/// tasks carry them next to the task. `None` is exactly `format_ai_json`.
pub fn format_ai_json_with_annotations(
    report: &AuditReport,
    annotations: Option<&crate::annotations::Resolved>,
) -> Result<String, ReportContractError> {
    let normalized = normalize_checked(report)?;
    let violations = &report.accessibility.wcag_results.violations;

    // Build tasks, then sort by impact severity.
    // task_id is numbered per rule_id (a stable, rule-local sequence) rather
    // than by position in the unsorted violation list.
    let mut per_rule_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut tasks: Vec<AiTask> = violations
        .iter()
        .map(|v| {
            let rule_id = v.rule_id.clone().unwrap_or_else(|| v.rule.clone());
            let impact = v
                .impact
                .clone()
                .unwrap_or_else(|| v.impact_str().to_string());
            let seq = {
                let c = per_rule_counts.entry(rule_id.clone()).or_insert(0);
                *c += 1;
                *c
            };
            let (finding_id, task_annotations) = match annotations {
                Some(resolved) => {
                    let taxonomy_id = crate::audit::normalized::violation_rule_id(v);
                    let finding_id =
                        a11y_report::FindingId::new(&taxonomy_id, std::iter::empty()).to_string();
                    let pattern_id = a11y_report::RootCauseId::new(
                        &taxonomy_id,
                        v.selector.as_deref().unwrap_or_default(),
                    )
                    .to_string();
                    let attached: Vec<AiAnnotation> = resolved
                        .for_finding(&finding_id)
                        .filter(|m| m.pattern_id.as_deref().is_none_or(|p| p == pattern_id))
                        .map(|m| AiAnnotation::new(&m.annotation))
                        .collect();
                    ((!attached.is_empty()).then_some(finding_id), attached)
                }
                None => (None, Vec::new()),
            };
            AiTask {
                task_id: format!("{}-{:03}", rule_id, seq),
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
                html_snippet: v.html_snippet.clone(),
                finding_id,
                annotations: task_annotations,
            }
        })
        .collect();

    let annotations = annotations.map(|resolved| AiAnnotations {
        reading_flow: resolved
            .reading_flow()
            .map(|m| AiAnnotation::new(&m.annotation))
            .collect(),
        matched_without_task: resolved
            .matched
            .iter()
            .filter(|m| m.scope == crate::annotations::Scope::Finding)
            .filter(|m| {
                !tasks.iter().any(|t| {
                    t.finding_id == m.finding_id
                        && t.annotations.iter().any(|a| a.annotation == m.annotation)
                })
            })
            .map(|m| AiAnnotation::new(&m.annotation))
            .collect(),
        unmatched: resolved.unmatched.clone(),
        warnings: resolved.warnings.clone(),
    });

    tasks.sort_by_key(|t| impact_order(&t.impact));

    let ai_report = AiReport {
        url: report.url.clone(),
        audit_date: report.timestamp.format("%Y-%m-%d").to_string(),
        score: normalized.normalized.score,
        overall_score: normalized.normalized.overall_score,
        summary: AiSummary {
            total_violations: normalized.normalized.severity_counts.total,
            critical: normalized.normalized.severity_counts.critical,
            serious: normalized.normalized.severity_counts.high,
            moderate: normalized.normalized.severity_counts.medium,
            minor: normalized.normalized.severity_counts.low,
        },
        tasks,
        passing_checks: report.accessibility.wcag_results.passes,
        metadata: AiMetadata {
            tool: "auditmysite".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            wcag_level: report.wcag_level.to_string(),
        },
        annotations,
    };

    Ok(serde_json::to_string_pretty(&ai_report)
        .unwrap_or_else(|e| format!("{{\"error\": \"AI JSON serialization failed: {}\"}}", e)))
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
        let output = format_ai_json(&report).unwrap();

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
        let output = format_ai_json(&report).unwrap();

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
        .with_help_url("https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html")
        .with_rule_id("image-alt")
        .with_tags(vec!["wcag2a".to_string(), "wcag111".to_string()]);

        let report = make_report_with_violations(vec![v]);
        let output = format_ai_json(&report).unwrap();

        assert!(output.contains("\"task_id\": \"image-alt-001\""));
        assert!(output.contains("\"rule_id\": \"image-alt\""));
        assert!(output.contains("\"wcag\": \"1.1.1\""));
        assert!(output.contains("\"selector\": \"img.hero\""));
        assert!(output.contains("\"node_id\": \"node-99\""));
        assert!(output.contains("\"fix\": \"Add alt attribute\""));
        assert!(
            output.contains("https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html")
        );
        assert!(output.contains("\"wcag2a\""));
    }
}
