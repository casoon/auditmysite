//! Cross-layer conservation laws. These inspect the exported facts rather
//! than trusting that producer and consumer used the same implementation.

use serde_json::Value;

use super::LintFinding;
use crate::taxonomy::{RuleLookup, Severity};

fn failure(
    findings: &mut Vec<LintFinding>,
    id: &'static str,
    path: String,
    expected: String,
    actual: String,
) {
    findings.push(LintFinding {
        check_id: id,
        evidence_path: path,
        expected,
        actual,
        severity: Severity::High,
    });
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

pub(super) fn check(report: &Value, findings: &mut Vec<LintFinding>) {
    if !array(report, "errors").is_empty() && report["audit_quality"]["status"] == "complete" {
        failure(
            findings,
            "audit_completeness_matches_execution",
            "audit_quality".into(),
            "qualified audit when attempted URLs failed".into(),
            "complete with errors".into(),
        );
    }
    for (i, page) in array(report, "pages").iter().enumerate() {
        let prefix = format!("pages[{i}]");
        if page["score_calculation_method"] == "module_weighted" {
            let mut weighted = 0u64;
            let mut total = 0u64;
            for module in array(page, "module_scores") {
                if module["contributes_to_overall"] == true {
                    if let (Some(score), Some(weight)) =
                        (module["score"].as_u64(), module["weight_pct"].as_u64())
                    {
                        weighted += score * weight;
                        total += weight;
                    }
                }
            }
            if total > 0 {
                let expected = (weighted as f64 / total as f64).round() as u64;
                if page["overall_score"].as_u64() != Some(expected) {
                    failure(
                        findings,
                        "overall_score_matches_module_weights",
                        format!("{prefix}.overall_score"),
                        expected.to_string(),
                        page["overall_score"].to_string(),
                    );
                }
            }
        }

        let runs = array(page, "module_runs");
        for requested in array(&report["audit_scope"], "requested_modules")
            .iter()
            .filter_map(Value::as_str)
        {
            if !runs.iter().any(|run| run["module"] == requested) {
                failure(
                    findings,
                    "requested_module_has_outcome",
                    format!("{prefix}.module_runs"),
                    format!("execution outcome for {requested}"),
                    "missing".into(),
                );
            }
        }
        let rule_runs = array(page, "rule_outcomes");
        let has_failure = runs
            .iter()
            .any(|run| matches!(run["status"].as_str(), Some("partial" | "failed")))
            || rule_runs.iter().any(|run| {
                matches!(
                    run["not_run"].as_str(),
                    Some("errored" | "capability_missing")
                )
            });
        if has_failure && page["audit_quality"]["status"] == "complete" {
            failure(
                findings,
                "audit_completeness_matches_execution",
                format!("{prefix}.audit_quality"),
                "qualified audit after an incomplete check".into(),
                "complete".into(),
            );
        }

        // Per-rule maxima across viewports are a lower bound on surviving
        // occurrences. Exclusions and review/positive outcomes also account
        // for results, so neither is misdiagnosed as a lost violation.
        for run in rule_runs {
            let Some(id) = run["rule_id"].as_str() else {
                continue;
            };
            let count = run["findings"].as_u64().unwrap_or(0);
            if count == 0 || !run["not_run"].is_null() {
                continue;
            }
            let canonical = RuleLookup::by_legacy_wcag_id(id).map(|rule| rule.id);
            let retained: u64 = array(page, "findings")
                .iter()
                .filter(|f| {
                    f["axe_id"] == id
                        || canonical.is_some_and(|canonical| f["rule_id"] == canonical)
                })
                .filter_map(|f| f["occurrence_count"].as_u64())
                .sum();
            let assessments = array(page, "accessibility_assessments")
                .iter()
                .filter(|f| f["rule_id"] == id)
                .count() as u64;
            let excluded: u64 = array(&page["exclusions"], "rules")
                .iter()
                .filter(|f| f["rule_id"] == id)
                .filter_map(|f| f["occurrences"].as_u64())
                .sum();
            if count > retained + assessments + excluded {
                failure(
                    findings,
                    "rule_findings_preserved",
                    format!("{prefix}.rule_outcomes[{id}]"),
                    format!("at least {count} accounted occurrences"),
                    format!(
                        "{} retained/reviewed/excluded",
                        retained + assessments + excluded
                    ),
                );
            }
        }
    }
}
