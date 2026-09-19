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
    /// Canonical machine-readable reasons — the JSON contract and what the
    /// CLI prints. Never render these to a reader: they are tokens
    /// (`legal_flags: 5`), and in a German report they are also the wrong
    /// language (#406).
    pub reasons: Vec<String>,
    /// The same reasons as data, so the PDF can phrase them in the run
    /// language (plan 34). Not serialized: the JSON contract is `reasons`,
    /// and duplicating it would give consumers two sources for one fact.
    #[serde(skip)]
    pub reason_kinds: Vec<VerdictReasonKind>,
}

/// Why a verdict is not `Pass`, with the numbers that decided it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictReasonKind {
    LegalFlags(usize),
    BlockingIssues(usize),
    ScoreBelowFailThreshold { score: u32, threshold: u32 },
    ScoreBelowWarnThreshold { score: u32, threshold: u32 },
    AuditQualityNotComplete,
    FindingsPresent(usize),
}

impl VerdictReasonKind {
    /// Reader-facing phrasing. The single source for this text.
    pub fn text(self, en: bool) -> String {
        match self {
            VerdictReasonKind::LegalFlags(n) => match (en, n) {
                (true, 1) => "1 legally relevant finding".to_string(),
                (true, n) => format!("{n} legally relevant findings"),
                (false, 1) => "1 rechtlich relevanter Befund".to_string(),
                (false, n) => format!("{n} rechtlich relevante Befunde"),
            },
            VerdictReasonKind::BlockingIssues(n) => match (en, n) {
                (true, 1) => "1 control not operable with assistive technology".to_string(),
                (true, n) => format!("{n} controls not operable with assistive technology"),
                (false, 1) => "1 Bedienelement nicht mit Hilfsmitteln bedienbar".to_string(),
                (false, n) => format!("{n} Bedienelemente nicht mit Hilfsmitteln bedienbar"),
            },
            VerdictReasonKind::ScoreBelowFailThreshold { score, threshold } => {
                if en {
                    format!("score {score} below the fail threshold of {threshold}")
                } else {
                    format!("Score {score} unter der Fail-Schwelle von {threshold}")
                }
            }
            VerdictReasonKind::ScoreBelowWarnThreshold { score, threshold } => {
                if en {
                    format!("score {score} below the warning threshold of {threshold}")
                } else {
                    format!("Score {score} unter der Warnschwelle von {threshold}")
                }
            }
            VerdictReasonKind::AuditQualityNotComplete => {
                if en {
                    "the run did not complete in full".to_string()
                } else {
                    "der Lauf ist nicht vollständig durchgelaufen".to_string()
                }
            }
            VerdictReasonKind::FindingsPresent(n) => match (en, n) {
                (true, 1) => "1 open finding".to_string(),
                (true, n) => format!("{n} open findings"),
                (false, 1) => "1 offener Befund".to_string(),
                (false, n) => format!("{n} offene Befunde"),
            },
        }
    }
}

/// Compute verdict for a single-page audit.
pub fn compute_verdict(normalized: &NormalizedReport, config: &VerdictConfig) -> VerdictResult {
    let fail_on_legal = config.fail_on_legal_flags.unwrap_or(true);
    let fail_on_blocking = config.fail_on_blocking_issues.unwrap_or(true);
    let warn_below = config.warn_below_score.unwrap_or(70);

    let mut fail_reasons = Vec::new();
    let mut fail_kinds = Vec::new();

    if fail_on_legal && normalized.risk.legal_flags > 0 {
        fail_reasons.push(format!("legal_flags: {}", normalized.risk.legal_flags));
        fail_kinds.push(VerdictReasonKind::LegalFlags(normalized.risk.legal_flags));
    }
    if fail_on_blocking && normalized.risk.blocking_issues > 0 {
        fail_reasons.push(format!(
            "blocking_issues: {}",
            normalized.risk.blocking_issues
        ));
        fail_kinds.push(VerdictReasonKind::BlockingIssues(
            normalized.risk.blocking_issues,
        ));
    }
    if let Some(threshold) = config.fail_below_score {
        if threshold > 0 && normalized.score < threshold {
            fail_reasons.push(format!(
                "score {} < fail_below_score {}",
                normalized.score, threshold
            ));
            fail_kinds.push(VerdictReasonKind::ScoreBelowFailThreshold {
                score: normalized.score,
                threshold,
            });
        }
    }

    if !fail_reasons.is_empty() {
        return VerdictResult {
            verdict: Verdict::Fail,
            reasons: fail_reasons,
            reason_kinds: fail_kinds,
        };
    }

    let mut warn_reasons = Vec::new();
    let mut warn_kinds = Vec::new();
    if normalized.execution.quality.qualified_results {
        warn_reasons.push(format!(
            "audit_quality: {:?}",
            normalized.execution.quality.status
        ));
        warn_kinds.push(VerdictReasonKind::AuditQualityNotComplete);
    }
    if normalized.score < warn_below {
        warn_reasons.push(format!(
            "score {} < warn_below_score {}",
            normalized.score, warn_below
        ));
        warn_kinds.push(VerdictReasonKind::ScoreBelowWarnThreshold {
            score: normalized.score,
            threshold: warn_below,
        });
    } else if normalized.severity_counts.total > 0 {
        let n = normalized.severity_counts.total;
        warn_reasons.push(format!(
            "{} {} present",
            n,
            if n == 1 { "finding" } else { "findings" }
        ));
        warn_kinds.push(VerdictReasonKind::FindingsPresent(n));
    }

    if !warn_reasons.is_empty() {
        VerdictResult {
            verdict: Verdict::Warn,
            reasons: warn_reasons,
            reason_kinds: warn_kinds,
        }
    } else {
        VerdictResult {
            verdict: Verdict::Pass,
            reasons: vec![],
            reason_kinds: Vec::new(),
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
            // Not populated on the batch path: the batch PDF does not render
            // the reasons yet, and the batch-specific ones (failed URLs,
            // average score) need their own variants. `reasons` stays the
            // source there.
            reason_kinds: Vec::new(),
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
            reason_kinds: Vec::new(),
        }
    } else {
        VerdictResult {
            verdict: Verdict::Pass,
            reasons: vec![],
            reason_kinds: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 34/44: the reasons the PDF prints must be readable text in the
    /// run language, never the canonical machine tokens the JSON carries.
    #[test]
    fn verdict_reason_text_is_localized_and_inflected() {
        use super::VerdictReasonKind::*;
        for (kind, de, en) in [
            (
                LegalFlags(1),
                "1 rechtlich relevanter Befund",
                "1 legally relevant finding",
            ),
            (
                LegalFlags(4),
                "4 rechtlich relevante Befunde",
                "4 legally relevant findings",
            ),
            (
                BlockingIssues(1),
                "1 Bedienelement nicht mit Hilfsmitteln bedienbar",
                "1 control not operable with assistive technology",
            ),
            (FindingsPresent(1), "1 offener Befund", "1 open finding"),
            (FindingsPresent(3), "3 offene Befunde", "3 open findings"),
        ] {
            assert_eq!(kind.text(false), de, "{kind:?}");
            assert_eq!(kind.text(true), en, "{kind:?}");
        }
    }

    /// The English texts must not leak German characters (#406).
    #[test]
    fn verdict_reason_english_has_no_german_characters() {
        use super::VerdictReasonKind::*;
        for kind in [
            LegalFlags(2),
            BlockingIssues(2),
            ScoreBelowFailThreshold {
                score: 10,
                threshold: 40,
            },
            ScoreBelowWarnThreshold {
                score: 50,
                threshold: 70,
            },
            AuditQualityNotComplete,
            FindingsPresent(2),
        ] {
            let en = kind.text(true);
            assert!(!en.chars().any(|c| "äöüßÄÖÜ".contains(c)), "{kind:?}: {en}",);
        }
    }

    /// `reason_kinds` must stay in step with the canonical `reasons` the JSON
    /// publishes — two sources for one fact only help if they agree on how
    /// many facts there are.
    #[test]
    fn reason_kinds_match_the_canonical_reasons() {
        let mut normalized = crate::audit::normalized::normalize(&crate::audit::AuditReport::new(
            "https://example.com".to_string(),
            crate::WcagLevel::AA,
            crate::wcag::WcagResults::new(),
            100,
        ))
        .normalized;
        normalized.risk.legal_flags = 3;
        normalized.risk.blocking_issues = 7;

        let result = compute_verdict(&normalized, &VerdictConfig::default());
        assert_eq!(result.verdict, Verdict::Fail);
        assert_eq!(result.reasons.len(), result.reason_kinds.len());
        assert!(result
            .reason_kinds
            .contains(&super::VerdictReasonKind::LegalFlags(3)));
        assert!(result
            .reason_kinds
            .contains(&super::VerdictReasonKind::BlockingIssues(7)));
    }

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
