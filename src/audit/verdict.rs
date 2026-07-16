use serde::{Deserialize, Serialize};

use crate::audit::normalized::NormalizedReport;
use crate::audit::report::BatchSummary;
use crate::cli::config::VerdictConfig;

/// Three-level CI verdict derived from audit results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
}

impl Verdict {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Pass => 0,
            Self::Warn => 1,
            Self::Fail => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }
}

/// Verdict plus the reasons that drove it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictResult {
    pub verdict: Verdict,
    pub reasons: Vec<String>,
}

/// Compute verdict for a single-page audit.
pub fn compute_verdict(normalized: &NormalizedReport, config: &VerdictConfig) -> VerdictResult {
    let fail_on_legal = config.fail_on_legal_flags.unwrap_or(true);
    let fail_on_blocking = config.fail_on_blocking_issues.unwrap_or(true);
    let warn_below = config.warn_below_score.unwrap_or(70);

    let mut fail_reasons = Vec::new();

    if fail_on_legal && normalized.risk.legal_flags > 0 {
        fail_reasons.push(format!("legal_flags: {}", normalized.risk.legal_flags));
    }
    if fail_on_blocking && normalized.risk.blocking_issues > 0 {
        fail_reasons.push(format!(
            "blocking_issues: {}",
            normalized.risk.blocking_issues
        ));
    }
    if let Some(threshold) = config.fail_below_score {
        if threshold > 0 && normalized.score < threshold {
            fail_reasons.push(format!(
                "score {} < fail_below_score {}",
                normalized.score, threshold
            ));
        }
    }

    if !fail_reasons.is_empty() {
        return VerdictResult {
            verdict: Verdict::Fail,
            reasons: fail_reasons,
        };
    }

    let mut warn_reasons = Vec::new();
    if normalized.execution.quality.qualified_results {
        warn_reasons.push(format!(
            "audit_quality: {:?}",
            normalized.execution.quality.status
        ));
    }
    if normalized.score < warn_below {
        warn_reasons.push(format!(
            "score {} < warn_below_score {}",
            normalized.score, warn_below
        ));
    } else if normalized.severity_counts.total > 0 {
        let n = normalized.severity_counts.total;
        warn_reasons.push(format!(
            "{} {} present",
            n,
            if n == 1 { "finding" } else { "findings" }
        ));
    }

    if !warn_reasons.is_empty() {
        VerdictResult {
            verdict: Verdict::Warn,
            reasons: warn_reasons,
        }
    } else {
        VerdictResult {
            verdict: Verdict::Pass,
            reasons: vec![],
        }
    }
}

/// Compute verdict for a batch audit from its summary.
pub fn compute_batch_verdict(summary: &BatchSummary, config: &VerdictConfig) -> VerdictResult {
    let fail_on_legal = config.fail_on_legal_flags.unwrap_or(true);
    let fail_on_blocking = config.fail_on_blocking_issues.unwrap_or(true);
    let warn_below = config.warn_below_score.unwrap_or(70);
    let average_score = summary.average_score.round() as u32;

    let mut fail_reasons = Vec::new();

    if fail_on_legal && summary.legal_flags > 0 {
        fail_reasons.push(format!(
            "legal_flags: {} (across all pages)",
            summary.legal_flags
        ));
    }
    if fail_on_blocking && summary.blocking_issues > 0 {
        fail_reasons.push(format!(
            "blocking_issues: {} (across all pages)",
            summary.blocking_issues
        ));
    }
    if let Some(threshold) = config.fail_below_score {
        if threshold > 0 && average_score < threshold {
            fail_reasons.push(format!(
                "average score {} < fail_below_score {}",
                average_score, threshold
            ));
        }
    }

    if !fail_reasons.is_empty() {
        return VerdictResult {
            verdict: Verdict::Fail,
            reasons: fail_reasons,
        };
    }

    let mut warn_reasons = Vec::new();
    if summary.audit_quality.qualified_results {
        warn_reasons.push(format!("audit_quality: {:?}", summary.audit_quality.status));
    }
    if average_score < warn_below {
        warn_reasons.push(format!(
            "average score {} < warn_below_score {}",
            average_score, warn_below
        ));
    } else if summary.failed > 0 {
        warn_reasons.push(format!(
            "{} {} failed quality threshold",
            summary.failed,
            if summary.failed == 1 { "URL" } else { "URLs" }
        ));
    } else if summary.total_violations > 0 {
        warn_reasons.push(format!(
            "{} {} found",
            summary.total_violations,
            if summary.total_violations == 1 {
                "violation"
            } else {
                "violations"
            }
        ));
    }

    if !warn_reasons.is_empty() {
        VerdictResult {
            verdict: Verdict::Warn,
            reasons: warn_reasons,
        }
    } else {
        VerdictResult {
            verdict: Verdict::Pass,
            reasons: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_batch_cannot_pass_unqualified() {
        let summary = BatchSummary {
            total_urls: 1,
            passed: 1,
            failed: 0,
            average_score: 100.0,
            total_violations: 0,
            top_recurring_rules: Vec::new(),
            violated_rule_count: 0,
            legal_flags: 0,
            blocking_issues: 0,
            risk: crate::audit::normalized::RiskLevel::Low,
            verdict_key: String::new(),
            template_clusters: Vec::new(),
            audit_quality: crate::audit::AuditQuality {
                status: crate::audit::AuditQualityStatus::Partial,
                qualified_results: true,
                failed_rule_checks: 0,
                partial_or_failed_modules: 1,
                reasons: vec!["module_runs_incomplete".to_string()],
            },
        };

        let result = compute_batch_verdict(&summary, &VerdictConfig::default());

        assert_eq!(result.verdict, Verdict::Warn);
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.starts_with("audit_quality:")));
    }
}
