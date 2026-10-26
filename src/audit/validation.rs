//! Format-independent contracts for data handed to report consumers.

use std::collections::HashSet;
use std::fmt;

use serde::Serialize;

use super::{AccessibilityScorer, AuditQualityStatus, ExecutionStatus};

pub(crate) struct CaptureGaps {
    pub stability_fallbacks: usize,
    pub uninspected_frames: usize,
    pub exhausted_stability_budgets: usize,
}

/// Intentional exclusions and invisible frames are outside the measured
/// scope. A visible frame that could not be inspected is a coverage gap.
pub(crate) fn capture_gaps(execution: &super::AuditExecution) -> CaptureGaps {
    use super::frames::FrameSkipReason;
    use crate::interaction::stability::StabilityStatus;
    CaptureGaps {
        stability_fallbacks: execution
            .navigation
            .stability
            .iter()
            .filter(|s| s.status == StabilityStatus::Fallback)
            .count(),
        exhausted_stability_budgets: execution
            .navigation
            .stability
            .iter()
            .filter(|s| s.status == StabilityStatus::BudgetExhausted)
            .count(),
        uninspected_frames: execution
            .frames
            .iter()
            .flat_map(|frames| &frames.skipped)
            .filter(|frame| {
                !matches!(
                    frame.reason,
                    Some(FrameSkipReason::Hidden | FrameSkipReason::Excluded)
                )
            })
            .count(),
    }
}
use super::{AuditContext, AuditReport, BatchReport, NormalizedReport};

#[derive(Debug, Clone, Serialize)]
pub struct ContractViolation {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportContractError {
    pub violations: Vec<ContractViolation>,
}

impl fmt::Display for ReportContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, violation) in self.violations.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(
                f,
                "{} at {}: {}",
                violation.code, violation.path, violation.message
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for ReportContractError {}

impl From<ReportContractError> for crate::error::AuditError {
    fn from(error: ReportContractError) -> Self {
        Self::ReportGenerationFailed {
            reason: format!("Audit contract violation: {error}"),
        }
    }
}

impl NormalizedReport {
    /// Check domain facts before choosing an output format. A qualified,
    /// partial audit is valid data; contradictory claims are not.
    pub fn validate(&self) -> Result<(), ReportContractError> {
        let mut violations = Vec::new();
        let mut check = |valid: bool, code, path: String, message: String| {
            if !valid {
                violations.push(ContractViolation {
                    code,
                    path,
                    message,
                });
            }
        };
        check(
            self.score <= 100 && self.overall_score <= 100,
            "score_range",
            "scores".into(),
            "scores must be between 0 and 100".into(),
        );
        check(
            self.grade == AccessibilityScorer::calculate_grade(self.score as f32),
            "grade",
            "grade".into(),
            "grade must describe the accessibility score".into(),
        );
        let certificate = super::normalized::gate_certificate_by_risk(
            AccessibilityScorer::calculate_certificate(self.score as f32).to_string(),
            &self.risk.level,
            self.risk.legal_flags,
            self.risk.blocking_issues,
        );
        check(
            self.certificate == certificate,
            "certificate",
            "certificate".into(),
            "certificate must apply the canonical score and risk policy".into(),
        );

        let mut finding_counts = [0usize; 5];
        let mut occurrence_counts = [0usize; 5];
        for finding in self.findings.iter().filter(|f| f.category == "wcag") {
            let index = match finding.severity {
                crate::taxonomy::Severity::Critical => 0,
                crate::taxonomy::Severity::High => 1,
                crate::taxonomy::Severity::Medium => 2,
                crate::taxonomy::Severity::Low => 3,
            };
            finding_counts[index] += 1;
            finding_counts[4] += 1;
            occurrence_counts[index] =
                occurrence_counts[index].saturating_add(finding.occurrence_count);
            occurrence_counts[4] = occurrence_counts[4].saturating_add(finding.occurrence_count);
            check(
                finding.occurrence_count > 0,
                "finding_occurrences",
                format!("findings[{}]", finding.rule_id),
                "a finding must retain at least one occurrence".into(),
            );
        }
        let c = &self.severity_counts;
        check(
            [c.critical, c.high, c.medium, c.low, c.total] == finding_counts,
            "severity_counts",
            "severity_counts".into(),
            "counts must match WCAG finding groups".into(),
        );
        let c = &self.occurrence_counts;
        check(
            [c.critical, c.high, c.medium, c.low, c.total] == occurrence_counts,
            "occurrence_counts",
            "occurrence_counts".into(),
            "counts must match WCAG occurrences".into(),
        );
        let level_a_urgent = self
            .findings
            .iter()
            .filter(|f| {
                f.category == "wcag"
                    && f.wcag_level == "A"
                    && matches!(
                        f.severity,
                        crate::taxonomy::Severity::Critical | crate::taxonomy::Severity::High
                    )
            })
            .map(|f| f.occurrence_count)
            .fold(0usize, usize::saturating_add);
        let limit = super::scoring::semantic_cap(
            occurrence_counts[0],
            occurrence_counts[0].saturating_add(occurrence_counts[1]),
            level_a_urgent,
        );
        check(
            self.score as f32 <= limit,
            "semantic_score_cap",
            "score".into(),
            format!("retained findings bound accessibility at {limit}"),
        );

        let mut names = HashSet::new();
        let mut weighted = 0u64;
        let mut weight = 0u64;
        for module in &self.module_scores {
            let path = format!("module_scores[{}]", module.name);
            check(
                names.insert(module.name.as_str()),
                "module_identity",
                path.clone(),
                "module scores must be unique".into(),
            );
            check(
                module.score <= 100,
                "score_range",
                path.clone(),
                "module score must be between 0 and 100".into(),
            );
            let expected = if module.contributes_to_overall {
                crate::taxonomy::module_weight(&module.name)
            } else {
                0
            };
            check(
                module.weight_pct == expected && (!module.contributes_to_overall || expected > 0),
                "module_weight",
                path.clone(),
                "weight must match the canonical contributing-module policy".into(),
            );
            check(
                !module.contributes_to_overall || module.measurement_type == "measured",
                "weighted_measurement",
                path.clone(),
                "only measured modules may contribute to the overall score".into(),
            );
            check(
                module.grade.as_deref()
                    == module
                        .contributes_to_overall
                        .then(|| AccessibilityScorer::calculate_grade(module.score as f32)),
                "module_grade",
                path,
                "an unweighted indicator must not carry a letter grade".into(),
            );
            if module.name == "Accessibility" {
                check(
                    module.score == self.score && module.contributes_to_overall,
                    "accessibility_module",
                    "module_scores[Accessibility]".into(),
                    "accessibility module must match the primary score".into(),
                );
            }
            if module.contributes_to_overall {
                weighted =
                    weighted.saturating_add(u64::from(module.score) * u64::from(module.weight_pct));
                weight = weight.saturating_add(u64::from(module.weight_pct));
            }
        }
        check(
            names.contains("Accessibility"),
            "accessibility_module",
            "module_scores".into(),
            "primary accessibility module is required".into(),
        );
        check(
            self.score_calculation_method == "module_weighted",
            "score_method",
            "score_calculation_method".into(),
            "legacy score methods require explicit migration before presentation".into(),
        );
        if weight > 0 {
            let expected = (weighted as f64 / weight as f64).round() as u32;
            check(
                self.overall_score == expected,
                "overall_score",
                "overall_score".into(),
                format!(
                    "published module weights reconstruct {expected}, not {}",
                    self.overall_score
                ),
            );
        }
        for requested in &self.execution.scope.requested_modules {
            let matching: Vec<_> = self
                .execution
                .module_runs
                .iter()
                .filter(|run| &run.module == requested)
                .collect();
            check(
                matching.len() == 1 && matching[0].status != ExecutionStatus::NotRequested,
                "requested_execution",
                format!("execution.module_runs[{requested}]"),
                "each requested module needs exactly one explicit execution outcome".into(),
            );
        }
        // Counts are lower bounds per viewport, not a sum across reloads.
        // Reviews and explicit exclusions are accounted outcomes too.
        for run in &self.rule_outcomes {
            if run.not_run.is_some() || run.findings == 0 {
                continue;
            }
            let canonical =
                crate::taxonomy::RuleLookup::by_legacy_wcag_id(&run.rule_id).map(|rule| rule.id);
            let retained: usize = self
                .findings
                .iter()
                .filter(|finding| {
                    finding.axe_id.as_deref() == Some(run.rule_id.as_str())
                        || canonical == Some(finding.rule_id.as_str())
                })
                .map(|finding| finding.occurrence_count)
                .fold(0usize, usize::saturating_add);
            let assessments = self
                .accessibility_assessments
                .iter()
                .filter(|finding| finding.rule_id == run.rule_id)
                .count();
            let excluded: usize = self
                .execution
                .exclusions
                .iter()
                .flat_map(|exclusions| &exclusions.rules)
                .filter(|rule| rule.rule_id == run.rule_id)
                .map(|rule| rule.occurrences)
                .fold(0usize, usize::saturating_add);
            let accounted = retained
                .saturating_add(assessments)
                .saturating_add(excluded);
            check(
                accounted >= run.findings,
                "rule_findings_preserved",
                format!("rule_outcomes[{}]", run.rule_id),
                format!(
                    "{} reported occurrences, but only {accounted} retained/reviewed/excluded",
                    run.findings
                ),
            );
        }
        let q = &self.execution.quality;
        let gaps = capture_gaps(&self.execution);
        check(
            q.qualified_results == (q.status != AuditQualityStatus::Complete),
            "quality_qualification",
            "execution.quality".into(),
            "qualification must match completeness".into(),
        );
        let incomplete = self.execution.module_runs.iter().any(|run| {
            matches!(
                run.status,
                ExecutionStatus::Partial | ExecutionStatus::Failed
            ) || run.subchecks.iter().any(|subcheck| {
                matches!(
                    subcheck.status,
                    ExecutionStatus::Partial | ExecutionStatus::Failed
                )
            })
        }) || self.rule_outcomes.iter().any(|run| {
            matches!(
                run.not_run,
                Some(crate::wcag::NotRun::Errored | crate::wcag::NotRun::CapabilityMissing)
            )
        });
        check(
            !(incomplete
                || gaps.stability_fallbacks > 0
                || gaps.exhausted_stability_budgets > 0
                || gaps.uninspected_frames > 0)
                || q.status != AuditQualityStatus::Complete,
            "execution_completeness",
            "execution.quality".into(),
            "incomplete checks cannot produce a complete audit".into(),
        );
        finish(violations)
    }
}

pub fn normalize_checked(report: &AuditReport) -> Result<AuditContext<'_>, ReportContractError> {
    let context = super::normalize(report);
    context.normalized.validate()?;
    Ok(context)
}

pub fn validate_batch(batch: &BatchReport) -> Result<(), ReportContractError> {
    let mut violations = Vec::new();
    let mut total = 0u64;
    let mut passed = 0usize;
    for (index, report) in batch.reports.iter().enumerate() {
        let context = super::normalize(report);
        if let Err(error) = context.normalized.validate() {
            violations.extend(error.violations.into_iter().map(|mut v| {
                v.path = format!("reports[{index}].{}", v.path);
                v
            }));
        }
        total += u64::from(context.normalized.score);
        passed += usize::from(
            super::compute_verdict(&context.normalized, &Default::default()).verdict
                == super::Verdict::Pass,
        );
    }
    let count = batch.reports.len();
    let expected_average = if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    };
    let summary = &batch.summary;
    for (valid, path, message) in [
        (
            summary.total_urls == count,
            "total_urls",
            "successful URL count must match measured pages",
        ),
        (
            summary.passed == passed && summary.failed == count - passed,
            "passed",
            "pass/fail counts must follow the canonical verdict",
        ),
        (
            summary.average_score.is_finite()
                && (summary.average_score - expected_average).abs() < 1e-6,
            "average_score",
            "average must use the measured accessibility scores and their actual denominator",
        ),
        (
            batch.errors.is_empty() || summary.audit_quality.status != AuditQualityStatus::Complete,
            "audit_quality",
            "failed requested URLs must qualify the batch",
        ),
        (
            count > 0 || summary.audit_quality.status == AuditQualityStatus::Insufficient,
            "audit_quality",
            "a batch without measured pages is insufficient",
        ),
    ] {
        if !valid {
            violations.push(ContractViolation {
                code: "batch_aggregation",
                path: format!("summary.{path}"),
                message: message.into(),
            });
        }
    }
    finish(violations)
}

fn finish(violations: Vec<ContractViolation>) -> Result<(), ReportContractError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ReportContractError { violations })
    }
}
