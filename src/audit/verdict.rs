//! The decision contract (#781): one policy, one place.
//!
//! Every PASS/WARN/FAIL a user sees — terminal, JSON, summary feed, SARIF,
//! Studio, PDF, the batch `passed` count and the process exit code — is
//! `NormalizedReport.decision` (single page) or `BatchSummary.decision`
//! (batch). Both are computed here, once, during normalization, from the
//! [`DecisionProfile`] stored with the raw report. Outputs read the stored
//! result; none of them recomputes it. `validate` rejects a stored decision
//! that its own profile does not reproduce.
//!
//! Score basis: thresholds compare the **accessibility score**
//! (`NormalizedReport.score`), the subject of the report. Grade and
//! certificate are bands of the same score (certificate additionally capped
//! by the risk veto). `overall_score` is the module-weighted technical value;
//! it carries no grade and never decides the verdict.

use serde::{Deserialize, Serialize};

use crate::audit::normalized::NormalizedReport;
use crate::audit::report::BatchSummary;

/// Version of the decision contract. Bumped whenever a rule below changes
/// what a profile decides.
pub const DECISION_CONTRACT: &str = "auditmysite.decision/1";

/// The thresholds a verdict is computed with. Stored with every report, so a
/// stored decision stays reproducible from its declared inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionProfile {
    /// `default` when every value is the contract default, else `custom`.
    pub name: String,
    pub fail_on_legal_flags: bool,
    pub fail_on_blocking_issues: bool,
    pub fail_on_technical_blockers: bool,
    /// Fail below this accessibility score; `None` disables the check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_below_score: Option<u32>,
    /// Warn below this accessibility score.
    pub warn_below_score: u32,
}

impl Default for DecisionProfile {
    fn default() -> Self {
        Self::resolve(None, None, None, None, None)
    }
}

impl DecisionProfile {
    /// Resolve configured values against the contract defaults.
    pub fn resolve(
        fail_on_legal_flags: Option<bool>,
        fail_on_blocking_issues: Option<bool>,
        fail_on_technical_blockers: Option<bool>,
        fail_below_score: Option<u32>,
        warn_below_score: Option<u32>,
    ) -> Self {
        let mut profile = Self {
            name: String::new(),
            fail_on_legal_flags: fail_on_legal_flags.unwrap_or(true),
            fail_on_blocking_issues: fail_on_blocking_issues.unwrap_or(true),
            fail_on_technical_blockers: fail_on_technical_blockers.unwrap_or(false),
            fail_below_score: fail_below_score.filter(|threshold| *threshold > 0),
            warn_below_score: warn_below_score.unwrap_or(70),
        };
        let default = Self {
            name: String::new(),
            fail_on_legal_flags: true,
            fail_on_blocking_issues: true,
            fail_on_technical_blockers: false,
            fail_below_score: None,
            warn_below_score: 70,
        };
        profile.name = if profile == default {
            "default"
        } else {
            "custom"
        }
        .to_string();
        profile
    }
}

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

/// Verdict plus the reasons that drove it, the contract version and the
/// profile it was computed with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerdictResult {
    pub contract: String,
    pub profile: DecisionProfile,
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

impl VerdictResult {
    /// Placeholder while the inputs of a decision are assembled; replaced
    /// before the containing struct leaves its constructor. Fails, so a
    /// forgotten replacement can never read as a pass.
    pub(crate) fn undecided() -> Self {
        Self {
            contract: DECISION_CONTRACT.to_string(),
            profile: DecisionProfile::default(),
            verdict: Verdict::Fail,
            reasons: vec!["undecided".to_string()],
            reason_kinds: Vec::new(),
        }
    }
}

/// Why a verdict is not `Pass`, with the numbers that decided it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictReasonKind {
    LegalFlags(usize),
    /// `keyboard` is the subset not operable by keyboard (2.1.1); the rest
    /// lack an accessible name or role and stay operable.
    BlockingIssues {
        total: usize,
        keyboard: usize,
    },
    ScoreBelowFailThreshold {
        score: u32,
        threshold: u32,
    },
    ScoreBelowWarnThreshold {
        score: u32,
        threshold: u32,
    },
    AuditQualityNotComplete,
    FindingsPresent(usize),
    /// Findings with a gate effect, e.g. noindex on a page meant to be
    /// indexed (#780).
    TechnicalBlockers(usize),
}

impl VerdictReasonKind {
    /// Reader-facing phrasing. The single source for this text.
    pub fn text(self, en: bool) -> String {
        match self {
            VerdictReasonKind::LegalFlags(n) => match (en, n) {
                (true, 1) => "1 finding with possible legal relevance (BFSG/EAA)".to_string(),
                (true, n) => format!("{n} findings with possible legal relevance (BFSG/EAA)"),
                (false, 1) => "1 Befund mit möglicher Rechtsrelevanz (BFSG/EAA)".to_string(),
                (false, n) => format!("{n} Befunde mit möglicher Rechtsrelevanz (BFSG/EAA)"),
            },
            VerdictReasonKind::BlockingIssues { total, keyboard } => {
                super::management_risk::blocking_controls_text(total, keyboard, en)
            }
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
            VerdictReasonKind::TechnicalBlockers(n) => match (en, n) {
                (true, 1) => {
                    "1 technical blocker (page meant to be indexed carries noindex)".to_string()
                }
                (true, n) => format!("{n} technical blockers"),
                (false, 1) => {
                    "1 technischer Blocker (Seite soll indexiert werden, trägt aber noindex)"
                        .to_string()
                }
                (false, n) => format!("{n} technische Blocker"),
            },
            VerdictReasonKind::FindingsPresent(n) => match (en, n) {
                (true, 1) => "1 open finding".to_string(),
                (true, n) => format!("{n} open findings"),
                (false, 1) => "1 offener Befund".to_string(),
                (false, n) => format!("{n} offene Befunde"),
            },
        }
    }
}

/// Number of findings that act as a gate (technical blockers).
pub fn technical_blockers(normalized: &NormalizedReport) -> usize {
    normalized
        .findings
        .iter()
        .filter(|f| {
            f.contract.scoring.effect == crate::audit::finding_contract::ScoringEffect::Gate
        })
        .count()
}

/// Decide a single page under `profile`. The only implementation of the
/// single-page policy; normalization stores its result in
/// `NormalizedReport.decision`.
pub fn compute_verdict(normalized: &NormalizedReport, profile: &DecisionProfile) -> VerdictResult {
    let mut fail = Reasons::default();
    let risk = &normalized.risk;
    if profile.fail_on_legal_flags && risk.legal_flags > 0 {
        fail.push(
            format!("legal_flags: {}", risk.legal_flags),
            VerdictReasonKind::LegalFlags(risk.legal_flags),
        );
    }
    if profile.fail_on_blocking_issues && risk.blocking_issues > 0 {
        fail.push(
            format!("blocking_issues: {}", risk.blocking_issues),
            VerdictReasonKind::BlockingIssues {
                total: risk.blocking_issues,
                keyboard: risk.keyboard_blocking_issues,
            },
        );
    }
    let blockers = technical_blockers(normalized);
    if profile.fail_on_technical_blockers && blockers > 0 {
        fail.push(
            format!("technical_blockers: {blockers}"),
            VerdictReasonKind::TechnicalBlockers(blockers),
        );
    }
    if let Some(threshold) = profile.fail_below_score {
        if normalized.score < threshold {
            fail.push(
                format!(
                    "score {} < fail_below_score {}",
                    normalized.score, threshold
                ),
                VerdictReasonKind::ScoreBelowFailThreshold {
                    score: normalized.score,
                    threshold,
                },
            );
        }
    }
    if !fail.is_empty() {
        return fail.into_result(Verdict::Fail, profile);
    }

    let mut warn = Reasons::default();
    if normalized.execution.quality.qualified_results {
        warn.push(
            format!("audit_quality: {:?}", normalized.execution.quality.status),
            VerdictReasonKind::AuditQualityNotComplete,
        );
    }
    if blockers > 0 {
        warn.push(
            format!("technical_blockers: {blockers}"),
            VerdictReasonKind::TechnicalBlockers(blockers),
        );
    }
    if normalized.score < profile.warn_below_score {
        warn.push(
            format!(
                "score {} < warn_below_score {}",
                normalized.score, profile.warn_below_score
            ),
            VerdictReasonKind::ScoreBelowWarnThreshold {
                score: normalized.score,
                threshold: profile.warn_below_score,
            },
        );
    } else if normalized.severity_counts.total > 0 {
        let n = normalized.severity_counts.total;
        warn.push(
            format!(
                "{} {} present",
                n,
                if n == 1 { "finding" } else { "findings" }
            ),
            VerdictReasonKind::FindingsPresent(n),
        );
    }
    if warn.is_empty() {
        Reasons::default().into_result(Verdict::Pass, profile)
    } else {
        warn.into_result(Verdict::Warn, profile)
    }
}

#[derive(Default)]
struct Reasons {
    tokens: Vec<String>,
    kinds: Vec<VerdictReasonKind>,
}

impl Reasons {
    fn push(&mut self, token: String, kind: VerdictReasonKind) {
        self.tokens.push(token);
        self.kinds.push(kind);
    }

    fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    fn into_result(self, verdict: Verdict, profile: &DecisionProfile) -> VerdictResult {
        VerdictResult {
            contract: DECISION_CONTRACT.to_string(),
            profile: profile.clone(),
            verdict,
            reasons: self.tokens,
            reason_kinds: self.kinds,
        }
    }
}

/// Decide a batch under `profile` from its summary. The only implementation
/// of the batch policy; `BatchReport::from_reports` stores its result in
/// `BatchSummary.decision`.
pub fn compute_batch_verdict(summary: &BatchSummary, profile: &DecisionProfile) -> VerdictResult {
    let average_score = summary.average_score.map(|a| a.round() as u32);
    let mut fail = Vec::new();
    // Nothing was measured: no score, no findings, so no PASS/WARN either (#768).
    if summary.coverage.attempted > 0 && summary.coverage.succeeded == 0 {
        fail.push(format!(
            "no_url_audited: 0 of {} requested URLs produced a report ({} failed, {} blocked)",
            summary.coverage.attempted, summary.coverage.failed, summary.coverage.blocked
        ));
    }
    if profile.fail_on_legal_flags && summary.legal_flags > 0 {
        fail.push(format!(
            "legal_flags: {} (across all pages)",
            summary.legal_flags
        ));
    }
    if profile.fail_on_blocking_issues && summary.blocking_issues > 0 {
        fail.push(format!(
            "blocking_issues: {} (across all pages)",
            summary.blocking_issues
        ));
    }
    if profile.fail_on_technical_blockers && summary.technical_blockers > 0 {
        fail.push(format!(
            "technical_blockers: {} (across all pages)",
            summary.technical_blockers
        ));
    }
    if let Some(threshold) = profile.fail_below_score {
        if average_score.is_some_and(|a| a < threshold) {
            fail.push(format!(
                "average score {} < fail_below_score {}",
                average_score.unwrap_or(0),
                threshold
            ));
        }
    }
    // `reason_kinds` stay empty on the batch path: the batch PDF does not
    // render the reasons, and the batch-specific ones need their own
    // variants. `reasons` is the source there.
    let result = |verdict, reasons| VerdictResult {
        contract: DECISION_CONTRACT.to_string(),
        profile: profile.clone(),
        verdict,
        reasons,
        reason_kinds: Vec::new(),
    };
    if !fail.is_empty() {
        return result(Verdict::Fail, fail);
    }

    let mut warn = Vec::new();
    if !summary.coverage.is_complete() {
        warn.push(format!(
            "incomplete_scope: {} of {} requested URLs audited ({} failed, {} blocked, {} not attempted)",
            summary.coverage.succeeded,
            summary.coverage.selected.max(summary.coverage.attempted),
            summary.coverage.failed,
            summary.coverage.blocked,
            summary.coverage.not_attempted()
        ));
    }
    if summary.audit_quality.qualified_results {
        warn.push(format!("audit_quality: {:?}", summary.audit_quality.status));
    }
    if summary.technical_blockers > 0 {
        warn.push(format!(
            "technical_blockers: {} (across all pages)",
            summary.technical_blockers
        ));
    }
    if average_score.is_some_and(|a| a < profile.warn_below_score) {
        warn.push(format!(
            "average score {} < warn_below_score {}",
            average_score.unwrap_or(0),
            profile.warn_below_score
        ));
    } else if summary.failed > 0 {
        warn.push(format!(
            "{} {} failed quality threshold",
            summary.failed,
            if summary.failed == 1 { "URL" } else { "URLs" }
        ));
    } else if summary.total_violations > 0 {
        warn.push(format!(
            "{} {} found",
            summary.total_violations,
            if summary.total_violations == 1 {
                "violation"
            } else {
                "violations"
            }
        ));
    }
    if warn.is_empty() {
        result(Verdict::Pass, Vec::new())
    } else {
        result(Verdict::Warn, warn)
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
                "1 Befund mit möglicher Rechtsrelevanz (BFSG/EAA)",
                "1 finding with possible legal relevance (BFSG/EAA)",
            ),
            (
                LegalFlags(4),
                "4 Befunde mit möglicher Rechtsrelevanz (BFSG/EAA)",
                "4 findings with possible legal relevance (BFSG/EAA)",
            ),
            (
                BlockingIssues {
                    total: 1,
                    keyboard: 0,
                },
                "1 Bedienelement ohne zugänglichen Namen oder Rolle",
                "1 control without an accessible name or role",
            ),
            (
                BlockingIssues {
                    total: 3,
                    keyboard: 1,
                },
                "2 Bedienelemente ohne zugänglichen Namen oder Rolle und 1 Bedienelement nicht per Tastatur bedienbar",
                "2 controls without an accessible name or role and 1 control not operable by keyboard",
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
            BlockingIssues {
                total: 2,
                keyboard: 1,
            },
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

        let result = compute_verdict(&normalized, &DecisionProfile::default());
        assert_eq!(result.verdict, Verdict::Fail);
        assert_eq!(result.reasons.len(), result.reason_kinds.len());
        assert!(result
            .reason_kinds
            .contains(&super::VerdictReasonKind::LegalFlags(3)));
        assert!(result
            .reason_kinds
            .contains(&super::VerdictReasonKind::BlockingIssues {
                total: 7,
                keyboard: 0
            }));
    }

    #[test]
    fn incomplete_batch_cannot_pass_unqualified() {
        let summary = BatchSummary {
            total_urls: 1,
            coverage: Default::default(),
            passed: 1,
            failed: 0,
            average_score: Some(100.0),
            average_over: 1,
            total_violations: 0,
            violated_rule_count: 0,
            legal_flags: 0,
            blocking_issues: 0,
            risk: crate::audit::normalized::RiskLevel::Low,
            score_band_key: String::new(),
            technical_blockers: 0,
            decision: VerdictResult::undecided(),
            root_causes: Vec::new(),
            audit_quality: crate::audit::AuditQuality {
                status: crate::audit::AuditQualityStatus::Partial,
                qualified_results: true,
                failed_rule_checks: 0,
                partial_or_failed_modules: 1,
                reasons: vec!["module_runs_incomplete".to_string()],
            },
            score_spread: None,
            selection_profile: Default::default(),
        };

        let result = compute_batch_verdict(&summary, &DecisionProfile::default());

        assert_eq!(result.verdict, Verdict::Warn);
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.starts_with("audit_quality:")));
    }

    // ── One decision through every projection (#781) ────────────────────

    use crate::audit::normalized::normalize;
    use crate::audit::{AuditReport, BatchReport};
    use crate::wcag::{Violation, WcagResults};

    /// One High failure of a Level-A criterion — the U16 reproduction: it
    /// read PASS in the terminal and FAIL in CI.
    fn high_level_a_report() -> AuditReport {
        let mut results = WcagResults::new();
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                crate::cli::WcagLevel::A,
                crate::taxonomy::Severity::High,
                "Image has no text alternative",
                "n1",
            )
            .with_rule_id("image-alt")
            .with_selector("img.hero"),
        );
        AuditReport::new(
            "https://example.com".to_string(),
            crate::cli::WcagLevel::AA,
            results,
            100,
        )
    }

    /// The verdict marker the terminal's first line starts with (ASCII
    /// theme without colour, Unicode otherwise).
    fn terminal_says(rendered: &str, verdict: Verdict) -> bool {
        let markers: &[&str] = match verdict {
            Verdict::Pass => &["[OK]", "\u{2714}"],
            Verdict::Warn => &["[WARN]", "\u{26a0}"],
            Verdict::Fail => &["[FAIL]", "\u{2716}"],
        };
        let first = rendered.lines().next().unwrap_or_default();
        markers.iter().any(|m| first.starts_with(m))
    }

    /// Every output of `report` publishes `expected`; returns nothing, panics
    /// with the output that disagrees.
    fn assert_every_output_decides(report: &AuditReport, expected: Verdict) {
        use clap::Parser;
        let ctx = normalize(report);
        ctx.normalized.validate().unwrap();
        let decision = &ctx.normalized.decision;
        assert_eq!(
            decision.verdict, expected,
            "normalized: {:?}",
            decision.reasons
        );
        assert_eq!(decision.profile, report.run_context.decision_profile);
        assert_eq!(
            report.passed(),
            expected == Verdict::Pass,
            "AuditReport::passed"
        );
        assert_eq!(
            decision.verdict.exit_code(),
            expected.exit_code(),
            "exit code (main maps the stored decision)"
        );

        let mut args = crate::cli::Args::parse_from(["auditmysite", "https://example.com"]);
        args.color = crate::cli::ColorPolicy::Never;
        let terminal = crate::output::terminal::render_single_report(report, &args);
        assert!(
            terminal_says(&terminal, expected),
            "terminal does not say {expected:?}:\n{terminal}"
        );

        let json: serde_json::Value = serde_json::from_str(
            &crate::output::UnifiedReport::single(&ctx, report)
                .to_json(true)
                .unwrap(),
        )
        .unwrap();
        let label = serde_json::to_value(expected).unwrap();
        assert_eq!(json["decision"]["verdict"], label, "JSON decision");
        assert_eq!(json["pages"][0]["decision"]["verdict"], label, "JSON page");
        assert_eq!(
            json["summary"]["passed_url_count"],
            serde_json::json!(usize::from(expected == Verdict::Pass)),
            "JSON passed count"
        );
        assert!(json["summary"].get("score").is_none(), "no score alias");

        let summary: serde_json::Value =
            serde_json::from_str(&crate::output::summary::format_summary(&ctx.normalized).unwrap())
                .unwrap();
        assert_eq!(summary["verdict"], label, "summary feed");

        let studio =
            crate::studio::StudioAuditResponse::from_normalized(&ctx, report, String::new());
        assert_eq!(studio.decision.verdict, expected, "studio");

        let vm = crate::output::builder::build_view_model(&ctx, &Default::default());
        assert_eq!(vm.summary.ci_verdict, expected, "PDF input model");

        let batch = BatchReport::from_reports(vec![report.clone()], Vec::new(), 1);
        crate::audit::validation::validate_batch(&batch).unwrap();
        assert_eq!(batch.summary.passed, usize::from(expected == Verdict::Pass));
        let batch_terminal = crate::output::terminal::render_batch_report(&batch, &args);
        assert!(
            terminal_says(&batch_terminal, batch.summary.decision.verdict),
            "batch terminal:\n{batch_terminal}"
        );
        let batch_json: serde_json::Value = serde_json::from_str(
            &crate::output::UnifiedReport::batch(&batch)
                .to_json(true)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            batch_json["decision"],
            serde_json::to_value(&batch.summary.decision).unwrap()
        );
    }

    #[test]
    fn high_level_a_defect_fails_in_every_output() {
        let report = high_level_a_report();
        assert_every_output_decides(&report, Verdict::Fail);
        let n = normalize(&report).normalized;
        assert!(n
            .decision
            .reason_kinds
            .iter()
            .any(|k| matches!(k, VerdictReasonKind::LegalFlags(_))));
    }

    /// A custom profile is stored with the report; every output follows it.
    /// Before, the exit code used the profile and everything else the
    /// default — FAIL in the terminal, WARN in CI.
    #[test]
    fn custom_profile_reaches_every_output() {
        let mut report = high_level_a_report();
        report.run_context.decision_profile =
            DecisionProfile::resolve(Some(false), None, None, None, None);
        assert_eq!(report.run_context.decision_profile.name, "custom");
        assert_every_output_decides(&report, Verdict::Warn);

        let clean = AuditReport::new(
            "https://example.com".to_string(),
            crate::cli::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        assert_every_output_decides(&clean, Verdict::Pass);
    }

    #[test]
    fn profile_resolution_matches_the_contract_defaults() {
        let default = DecisionProfile::default();
        assert_eq!(default.name, "default");
        assert!(default.fail_on_legal_flags && default.fail_on_blocking_issues);
        assert!(!default.fail_on_technical_blockers);
        assert_eq!(
            (default.fail_below_score, default.warn_below_score),
            (None, 70)
        );
        // 0 disables the fail threshold, as documented for `[verdict]`.
        assert_eq!(
            DecisionProfile::resolve(None, None, None, Some(0), None),
            default
        );
        assert_eq!(
            crate::cli::config::VerdictConfig {
                fail_below_score: Some(60),
                ..Default::default()
            }
            .profile()
            .fail_below_score,
            Some(60)
        );
    }

    /// Thresholds compare the accessibility score at the documented
    /// boundaries: below warns/fails, equal does not.
    #[test]
    fn score_boundaries_follow_the_profile() {
        let clean = AuditReport::new(
            "https://example.com".to_string(),
            crate::cli::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let mut n = normalize(&clean).normalized;
        n.overall_score = 100;
        let profile = DecisionProfile::resolve(None, None, None, Some(50), Some(70));
        for (score, expected) in [
            (70, Verdict::Pass),
            (69, Verdict::Warn),
            (50, Verdict::Warn),
            (49, Verdict::Fail),
        ] {
            n.score = score;
            assert_eq!(
                compute_verdict(&n, &profile).verdict,
                expected,
                "score {score}"
            );
        }
        // The overall score never decides.
        n.score = 100;
        n.overall_score = 10;
        assert_eq!(compute_verdict(&n, &profile).verdict, Verdict::Pass);
    }

    /// A stored decision its own profile does not reproduce is rejected, with
    /// every other contradiction collected alongside — no output can publish
    /// it by picking another format.
    #[test]
    fn contradicting_decision_is_rejected_with_all_violations() {
        let mut n = normalize(&high_level_a_report()).normalized;
        n.decision.verdict = Verdict::Pass;
        n.decision.reasons.clear();
        n.severity_counts.total += 1;
        let error = n.validate().unwrap_err();
        let codes: Vec<_> = error.violations.iter().map(|v| v.code).collect();
        assert!(codes.contains(&"decision"), "{codes:?}");
        assert!(codes.contains(&"severity_counts"), "{codes:?}");
        assert!(error
            .violations
            .iter()
            .any(|v| v.code == "decision" && v.path == "decision"));
        assert!(crate::output::summary::format_summary(&n).is_err());
        assert!(crate::output::sarif::format_sarif(&[&n]).is_err());

        let mut n = normalize(&high_level_a_report()).normalized;
        n.decision.contract = "auditmysite.decision/0".into();
        let codes: Vec<_> = n
            .validate()
            .unwrap_err()
            .violations
            .iter()
            .map(|v| v.code)
            .collect();
        assert!(codes.contains(&"decision_contract"), "{codes:?}");
    }

    /// A batch mixing profiles, or a summary whose decision does not follow
    /// from its profile, is rejected.
    #[test]
    fn batch_decision_follows_its_profile() {
        let mut strict = high_level_a_report();
        strict.run_context.decision_profile =
            DecisionProfile::resolve(None, None, None, Some(95), None);
        let batch = BatchReport::from_reports(vec![high_level_a_report(), strict], Vec::new(), 1);
        let codes: Vec<_> = crate::audit::validation::validate_batch(&batch)
            .unwrap_err()
            .violations
            .iter()
            .map(|v| v.code)
            .collect();
        assert!(codes.contains(&"decision_profile"), "{codes:?}");

        let mut batch = BatchReport::from_reports(vec![high_level_a_report()], Vec::new(), 1);
        assert_eq!(batch.summary.decision.verdict, Verdict::Fail);
        batch.summary.decision.verdict = Verdict::Pass;
        let codes: Vec<_> = crate::audit::validation::validate_batch(&batch)
            .unwrap_err()
            .violations
            .iter()
            .map(|v| v.path.clone())
            .collect();
        assert!(codes.contains(&"summary.decision".to_string()), "{codes:?}");
    }

    /// Compact feed with diverging scores: one grade basis (accessibility),
    /// both scores named, no unqualified `score`. Before, `score` was the
    /// overall value and `grade` was banded from it while the report's grade
    /// described accessibility.
    #[test]
    fn compact_summary_names_both_scores_and_one_grade_basis() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            crate::cli::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        report.discoverability.seo = Some(crate::seo::SeoAnalysis {
            score: 10,
            ..Default::default()
        });
        let n = normalize(&report).normalized;
        assert!(
            n.overall_score + 10 < n.score,
            "{} vs {}",
            n.overall_score,
            n.score
        );
        let summary: serde_json::Value =
            serde_json::from_str(&crate::output::summary::format_summary(&n).unwrap()).unwrap();
        assert!(summary.get("score").is_none());
        assert_eq!(summary["accessibilityScore"], n.score);
        assert_eq!(summary["overallScore"], n.overall_score);
        assert_eq!(summary["grade"], n.grade.as_str());
        assert_eq!(
            summary["label"],
            crate::registry::MEDAL.label(n.score as f32, false)
        );
    }
}
