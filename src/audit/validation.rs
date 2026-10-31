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
        // The measurement profile and the requested interaction depth tell
        // one story: a read-only profile lists no state-changing journey, and
        // a run that lists one says so (#787).
        let profile = &self.execution.scope.measurement_profile;
        if profile.version > 0 {
            check(
                profile.read_only == profile.state_changing_journeys.is_empty(),
                "measurement_profile_read_only",
                "execution.scope.measurement_profile".into(),
                "read_only must match the listed state-changing journeys".into(),
            );
            let expected = crate::cli::InteractiveMode::from_scope_name(
                &self.execution.scope.interactive_mode,
            )
            .map(|mode| {
                crate::a11y_journey::state_changing_journeys(
                    mode,
                    !profile.state_changing_journeys.is_empty(),
                )
            });
            if let Some(expected) = expected {
                check(
                    profile.state_changing_journeys == expected,
                    "measurement_profile_journeys",
                    "execution.scope.measurement_profile".into(),
                    format!(
                        "interactive mode `{}` permits {expected:?}",
                        self.execution.scope.interactive_mode
                    ),
                );
            }
        }
        // Every requested rule has exactly one outcome per viewport it was
        // requested in; a missing one is a gap, not a pass (#778). An outcome
        // without a viewport (reports from before the field) counts for all.
        for rule in &self.execution.scope.requested_rules {
            for viewport in &rule.viewports {
                let outcomes = self
                    .rule_outcomes
                    .iter()
                    .filter(|run| {
                        run.rule_id == rule.rule_id
                            && run.viewport.as_deref().is_none_or(|v| v == viewport)
                    })
                    .count();
                check(
                    outcomes >= 1,
                    "requested_rule_outcome",
                    format!("rule_outcomes[{}@{viewport}]", rule.rule_id),
                    "each requested rule needs an explicit execution outcome".into(),
                );
            }
        }
        // ... and not more than one: a doubled outcome counts one execution
        // twice.
        let mut seen: HashSet<(&str, Option<&str>)> = HashSet::new();
        for run in &self.rule_outcomes {
            check(
                seen.insert((run.rule_id.as_str(), run.viewport.as_deref())),
                "rule_outcome_duplicate",
                format!("rule_outcomes[{}]", run.rule_id),
                "a rule has one outcome per viewport".into(),
            );
        }
        // A rule above the configured level did not run.
        let level = match self.execution.scope.wcag_level.as_str() {
            "A" => Some(crate::cli::WcagLevel::A),
            "AA" => Some(crate::cli::WcagLevel::AA),
            "AAA" => Some(crate::cli::WcagLevel::AAA),
            _ => None,
        };
        if let Some(level) = level {
            for run in self.rule_outcomes.iter().filter(|r| r.not_run.is_none()) {
                check(
                    crate::wcag::expectation::registered_min_level(&run.rule_id)
                        .is_none_or(|min| min <= level),
                    "rule_outcome_above_level",
                    format!("rule_outcomes[{}]", run.rule_id),
                    format!("rule ran above the configured level {level}"),
                );
            }
        }
        // A module that lost a scoring collection was not fully measured: its
        // run is not `Completed` and its score neither claims a measurement
        // nor feeds the overall score (#777).
        for run in &self.execution.module_runs {
            // Every collection a run reports is one the inventory knows, and
            // a state without data explains itself in machine-readable form.
            for sub in run.subchecks.iter().filter(|s| s.state.is_some()) {
                let path = format!(
                    "execution.module_runs[{}].subchecks[{}]",
                    run.module, sub.subcheck
                );
                check(
                    super::collection::collector(&sub.subcheck).is_some(),
                    "collector_unknown",
                    path.clone(),
                    "the collection is not part of the collector inventory".into(),
                );
                let explained = match sub.state {
                    Some(super::collection::CollectionState::Partial) => !sub.gaps.is_empty(),
                    Some(
                        super::collection::CollectionState::Failed
                        | super::collection::CollectionState::Unsupported
                        | super::collection::CollectionState::NotApplicable,
                    ) => sub.reason_code.is_some(),
                    _ => true,
                };
                check(
                    explained,
                    "collection_state_unexplained",
                    path,
                    "a partial, failed, unsupported or not applicable collection needs gaps or a reason code".into(),
                );
            }
            let Some(gap) = run.subchecks.iter().find(|s| s.is_scoring_gap()) else {
                continue;
            };
            let path = format!("execution.module_runs[{}]", run.module);
            check(
                run.status != ExecutionStatus::Completed,
                "collection_gap_completed",
                path.clone(),
                format!("collector {} lost its measurement", gap.subcheck),
            );
            // The module score the run feeds, by its name in `module_scores`.
            // Modules without an entry (design quality, tech stack, …) show
            // no score a lost collection could inflate.
            let score_name = match run.module.as_str() {
                "seo" => "SEO",
                "security" => "Security",
                "mobile" => "Mobile",
                "performance" => "Performance",
                "best_practices" => "Best Practices",
                "dark_mode" => "Dark Mode",
                "html_conform" => "HTML Conformance",
                _ => continue,
            };
            if let Some(module) = self.module_scores.iter().find(|m| m.name == score_name) {
                check(
                    !module.contributes_to_overall && module.measurement_type != "measured",
                    "collection_gap_scored",
                    path,
                    format!(
                        "collector {} lost its measurement, so {score_name} is no measured score",
                        gap.subcheck
                    ),
                );
            }
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
        // The decision is the contract's result under its own stored
        // profile (#781); another value means a projection changed it.
        check(
            self.decision.contract == super::DECISION_CONTRACT,
            "decision_contract",
            "decision.contract".into(),
            format!(
                "decision contract {} is not {}",
                self.decision.contract,
                super::DECISION_CONTRACT
            ),
        );
        let expected = super::compute_verdict(self, &self.decision.profile);
        check(
            expected.verdict == self.decision.verdict && expected.reasons == self.decision.reasons,
            "decision",
            "decision".into(),
            format!(
                "profile {} decides {:?} ({}), stored {:?} ({})",
                self.decision.profile.name,
                expected.verdict,
                expected.reasons.join(", "),
                self.decision.verdict,
                self.decision.reasons.join(", ")
            ),
        );

        // Every collected problem has one representation (#780).
        let mut finding_ids = HashSet::new();
        for finding in &self.findings {
            check(
                finding_ids.insert(finding.id.as_str()),
                "finding_identity",
                format!("findings[{}]", finding.rule_id),
                "finding ids must be unique on a page".into(),
            );
            check(
                finding.category == finding.contract.domain.category(),
                "finding_domain",
                format!("findings[{}].contract.domain", finding.rule_id),
                "category must name the contract domain".into(),
            );
            check(
                finding.occurrence_count > 0,
                "finding_occurrences",
                format!("findings[{}]", finding.rule_id),
                "a finding must retain at least one occurrence".into(),
            );
        }
        for entry in &self.finding_inventory {
            let path = format!("finding_inventory[{}.{}]", entry.source, entry.check);
            match &entry.disposition {
                super::finding_contract::Disposition::Finding { finding_id } => {
                    let finding = self.findings.iter().find(|f| &f.id == finding_id);
                    check(
                        finding.is_some_and(|f| {
                            f.occurrence_count == entry.collected
                                && f.contract.source == entry.source
                                && f.contract.check == entry.check
                        }),
                        "finding_inventory",
                        path,
                        format!(
                            "{} collected problems are not represented by finding {finding_id}",
                            entry.collected
                        ),
                    );
                }
                super::finding_contract::Disposition::Excluded { .. } => check(
                    entry.collected > 0,
                    "finding_inventory",
                    path,
                    "an exclusion must account for collected problems".into(),
                ),
            }
        }
        for finding in self.findings.iter().filter(|f| f.category != "wcag") {
            check(
                self.finding_inventory.iter().any(|entry| {
                    matches!(&entry.disposition,
                        super::finding_contract::Disposition::Finding { finding_id }
                            if finding_id == &finding.id)
                }),
                "finding_inventory",
                format!("findings[{}]", finding.rule_id),
                "a finding without inventory entry has no traceable source".into(),
            );
        }
        // One score charges a cause once: grouping by cause is separate from
        // scoring, and a second charging finding must be a reference.
        let mut charged: HashSet<(&str, &str)> = HashSet::new();
        for finding in &self.findings {
            let scoring = &finding.contract.scoring;
            if !scoring.charges() {
                continue;
            }
            let owner = scoring.owner.as_deref().unwrap_or("");
            check(
                charged.insert((owner, finding.contract.cause.as_str())),
                "cause_charged_twice",
                format!("findings[{}].contract.scoring", finding.rule_id),
                format!(
                    "cause {} is already charged in {owner}",
                    finding.contract.cause
                ),
            );
        }
        // A gate caps its owner's score; a published score above the cap
        // contradicts the finding.
        if self.findings.iter().any(|f| {
            f.contract.scoring.effect == super::finding_contract::ScoringEffect::Gate
                && f.contract.scoring.owner.as_deref() == Some("SEO")
        }) {
            if let Some(seo) = self.module_scores.iter().find(|m| m.name == "SEO") {
                check(
                    seo.score <= crate::seo::INDEXABILITY_BLOCKER_CAP,
                    "gate_score_cap",
                    "module_scores[SEO]".into(),
                    format!(
                        "a technical blocker caps SEO at {}",
                        crate::seo::INDEXABILITY_BLOCKER_CAP
                    ),
                );
            }
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
        let errored = crate::wcag::expectation::count_checks(
            &self.rule_outcomes,
            crate::wcag::rule_run_errored,
        );
        check(
            q.failed_rule_checks == errored,
            "failed_rule_checks",
            "execution.quality.failed_rule_checks".into(),
            format!(
                "quality counts {} failed rule checks, outcomes show {errored}",
                q.failed_rule_checks
            ),
        );
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
    let context = super::normalized::normalize(report);
    context.normalized.validate()?;
    Ok(context)
}

pub fn validate_batch(batch: &BatchReport) -> Result<(), ReportContractError> {
    let mut violations = Vec::new();
    let mut total = 0u64;
    let mut passed = 0usize;
    for (index, report) in batch.reports.iter().enumerate() {
        let context = super::normalized::normalize(report);
        if let Err(error) = context.normalized.validate() {
            violations.extend(error.violations.into_iter().map(|mut v| {
                v.path = format!("reports[{index}].{}", v.path);
                v
            }));
        }
        total += u64::from(context.normalized.score);
        passed += usize::from(context.normalized.decision.verdict == super::Verdict::Pass);
        if &context.normalized.decision.profile != summary_profile(batch) {
            violations.push(ContractViolation {
                code: "decision_profile",
                path: format!("reports[{index}].run_context.decision_profile"),
                message: "every page of a batch is decided under the batch's profile".into(),
            });
        }
    }
    let count = batch.reports.len();
    let expected_average = (count > 0).then(|| total as f64 / count as f64);
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
            summary.decision.contract == super::DECISION_CONTRACT
                && summary.decision
                    == super::compute_batch_verdict(summary, &summary.decision.profile),
            "decision",
            "the stored batch decision must follow from its own profile",
        ),
        (
            match (summary.average_score, expected_average) {
                (None, None) => true,
                (Some(a), Some(e)) => a.is_finite() && (a - e).abs() < 1e-6,
                _ => false,
            } && summary.average_over == count,
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
    if let Some(sample) = batch.sample.as_ref().filter(|s| !s.strata.is_empty()) {
        let selected: usize = sample.strata.iter().map(|stratum| stratum.selected).sum();
        if selected != sample.audited {
            violations.push(ContractViolation {
                code: "sample_strata",
                path: "sample.strata".into(),
                message: format!(
                    "strata select {selected} URLs but the sample audited {}",
                    sample.audited
                ),
            });
        }
    }
    finish(violations)
}

fn summary_profile(batch: &BatchReport) -> &super::DecisionProfile {
    &batch.summary.decision.profile
}

fn finish(violations: Vec<ContractViolation>) -> Result<(), ReportContractError> {
    if violations.is_empty() {
        Ok(())
    } else {
        Err(ReportContractError { violations })
    }
}
