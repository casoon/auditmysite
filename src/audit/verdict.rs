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
