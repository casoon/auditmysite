//! Strict comparison of one corpus case against what a detector reported
//! (#783).
//!
//! Every corpus harness (WCAG, security, schema rules, vulnerable libraries)
//! turns its detector's output into an [`Observation`] and hands it to
//! [`diff_case`]. Keeping the comparison in one browser-free function has two
//! purposes: all harnesses apply the same rules, and every failure class can
//! be provoked directly in `tests/corpus_diff_contract.rs` against a
//! manipulated observation — no Chrome needed to prove the gate is not green
//! by accident.
//!
//! What counts as a mismatch:
//! - an expected finding that is missing, or found with the wrong count;
//! - a finding that no expectation of the case accounts for (*unexpected*):
//!   a case declares everything the detector may report on it;
//! - a `needs_review` expectation met by a confirmed violation
//!   (Review→Fail escalation);
//! - `pass` / `not_applicable` without proof that the rule ran;
//! - an execution record that contradicts the findings (a clean run next to
//!   a violation, or a counted run with no violation behind it);
//! - a confirmed violation that lost its location evidence.

use crate::common::detection_corpus::{Expectation, ExpectedCase, Verdict};
use std::collections::BTreeSet;

/// How certain a reported finding is. Mirrors the corpus verdicts:
/// `Fail` answers `violation`, `Review` answers `needs_review`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingClass {
    /// Confirmed violation.
    Fail,
    /// Heuristic warning or "a human must look" (`Review` / `Untested`).
    Review,
}

#[derive(Debug, Clone)]
pub struct ObservedFinding {
    pub rule_id: String,
    pub selector: Option<String>,
    pub class: FindingClass,
    /// Whether the finding still points at something concrete (a selector,
    /// a node, an HTML excerpt). Page-level rules may legitimately lack it.
    pub located: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    /// The rule ran to completion and counted this many findings.
    Ran { findings: usize },
    /// The rule did not run; the reason is the domain's own wording.
    NotRun(String),
}

#[derive(Debug, Clone)]
pub struct ObservedRun {
    pub rule_id: String,
    pub state: RunState,
}

#[derive(Debug, Clone, Default)]
pub struct Observation {
    pub findings: Vec<ObservedFinding>,
    /// Execution records. A domain without per-rule records proves execution
    /// some other way and synthesizes a record per rule it actually ran.
    pub runs: Vec<ObservedRun>,
}

/// Rule-specific knowledge the comparison needs, taken from the rule
/// contracts (`tests/fixtures/rule_contracts/`).
#[derive(Debug, Clone, Default)]
pub struct DiffRules {
    /// Rules whose findings concern the whole page and therefore carry no
    /// element location. Every other confirmed violation must be located.
    pub page_level: BTreeSet<String>,
    /// Domains whose run records count findings per rule. Off where a run
    /// record is synthesized by the harness.
    pub run_counts_findings: bool,
    /// Page-wide review notes that appear on every page (contracts with
    /// `ambient = true`, all manual checks). Such a note is accounted for
    /// without a declaration, but only as a review: escalated to a
    /// violation it is unexpected, and a case that declares the rule itself
    /// is held to its own declaration.
    pub ambient_review: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MismatchKind {
    MissingExpected,
    WrongOccurrences,
    UnexpectedFinding,
    ReviewEscalatedToFail,
    PassWithoutRun,
    NotApplicableWithFinding,
    NotApplicableWithoutRun,
    ErrorNotReported,
    FindingWithoutRun,
    RunContradictsFindings,
    LostEvidence,
}

impl MismatchKind {
    /// Missing findings are the dangerous direction (a real barrier goes
    /// unreported); everything else is reported as a false positive or a
    /// broken record. Kept apart in the message, as before.
    pub fn is_false_negative(self) -> bool {
        matches!(
            self,
            MismatchKind::MissingExpected
                | MismatchKind::PassWithoutRun
                | MismatchKind::NotApplicableWithoutRun
                | MismatchKind::ErrorNotReported
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    pub kind: MismatchKind,
    pub detail: String,
}

fn expectation_matches(exp: &Expectation, finding: &ObservedFinding) -> bool {
    finding.rule_id == exp.rule_id
        && exp
            .selector
            .as_deref()
            .is_none_or(|s| finding.selector.as_deref() == Some(s))
}

fn label(exp: &Expectation, detail: &str) -> String {
    match &exp.selector {
        Some(selector) => format!("{} @ {selector}: {detail}", exp.rule_id),
        None => format!("{}: {detail}", exp.rule_id),
    }
}

fn finding_label(f: &ObservedFinding) -> String {
    let class = match f.class {
        FindingClass::Fail => "violation",
        FindingClass::Review => "needs_review",
    };
    match &f.selector {
        Some(selector) => format!("{} @ {selector} ({class})", f.rule_id),
        None => format!("{} ({class})", f.rule_id),
    }
}

fn ran(obs: &Observation, rule_id: &str) -> bool {
    obs.runs
        .iter()
        .any(|r| r.rule_id == rule_id && matches!(r.state, RunState::Ran { .. }))
}

/// Compare one case against one observation. An empty result means the case
/// holds exactly.
pub fn diff_case(case: &ExpectedCase, obs: &Observation, rules: &DiffRules) -> Vec<Mismatch> {
    let mut out = Vec::new();
    let mut push = |kind, detail: String| out.push(Mismatch { kind, detail });

    for exp in &case.expectations {
        let fails = obs
            .findings
            .iter()
            .filter(|f| f.class == FindingClass::Fail && expectation_matches(exp, f))
            .count();
        let reviews = obs
            .findings
            .iter()
            .filter(|f| f.class == FindingClass::Review && expectation_matches(exp, f))
            .count();

        match exp.verdict {
            Verdict::Violation => {
                if fails == 0 {
                    push(
                        MismatchKind::MissingExpected,
                        label(exp, "expected violation, found none"),
                    );
                } else if let Some(expected) = exp.occurrences {
                    if fails != expected {
                        push(
                            MismatchKind::WrongOccurrences,
                            label(
                                exp,
                                &format!("expected {expected} occurrences, found {fails}"),
                            ),
                        );
                    }
                }
            }
            Verdict::NeedsReview => {
                if fails > 0 {
                    push(
                        MismatchKind::ReviewEscalatedToFail,
                        label(exp, "review was escalated to a confirmed violation"),
                    );
                }
                if reviews == 0 {
                    push(
                        MismatchKind::MissingExpected,
                        label(exp, "expected needs_review, found none"),
                    );
                } else if let Some(expected) = exp.occurrences {
                    if reviews != expected {
                        push(
                            MismatchKind::WrongOccurrences,
                            label(
                                exp,
                                &format!("expected {expected} review findings, found {reviews}"),
                            ),
                        );
                    }
                }
            }
            Verdict::Pass | Verdict::Absent => {
                if !ran(obs, &exp.rule_id) {
                    push(
                        MismatchKind::PassWithoutRun,
                        label(exp, "expected a completed run, but the rule did not run"),
                    );
                }
                // Both are reported as unexpected findings below, which is
                // where a pass or absent expectation never accounts for one.
            }
            Verdict::Errored => {
                let not_run = obs
                    .runs
                    .iter()
                    .any(|r| r.rule_id == exp.rule_id && matches!(r.state, RunState::NotRun(_)));
                if !not_run || ran(obs, &exp.rule_id) {
                    push(
                        MismatchKind::ErrorNotReported,
                        label(exp, "technical failure must be recorded as not run, without a completed run"),
                    );
                }
            }
            Verdict::NotApplicable => {
                if !ran(obs, &exp.rule_id) {
                    push(
                        MismatchKind::NotApplicableWithoutRun,
                        label(
                            exp,
                            "expected a completed run without findings, but the rule did not run",
                        ),
                    );
                }
                if fails + reviews > 0 {
                    push(
                        MismatchKind::NotApplicableWithFinding,
                        label(exp, "rule does not apply here, but reported a finding"),
                    );
                }
            }
        }
    }

    // Every reported finding must be accounted for by a positive expectation
    // of its own class. A pass/absent/not_applicable expectation never
    // accounts for one, so a finding there surfaces here as well.
    for f in &obs.findings {
        let wanted = match f.class {
            FindingClass::Fail => Verdict::Violation,
            FindingClass::Review => Verdict::NeedsReview,
        };
        let declared = case.expectations.iter().any(|exp| exp.rule_id == f.rule_id);
        let accounted = case
            .expectations
            .iter()
            .any(|exp| exp.verdict == wanted && expectation_matches(exp, f))
            || (f.class == FindingClass::Review
                && !declared
                && rules.ambient_review.contains(&f.rule_id));
        // A confirmed violation where review was expected is already
        // reported as an escalation; don't count it twice.
        let escalated = f.class == FindingClass::Fail
            && case
                .expectations
                .iter()
                .any(|exp| exp.verdict == Verdict::NeedsReview && expectation_matches(exp, f));
        if !accounted && !escalated {
            push(
                MismatchKind::UnexpectedFinding,
                format!("{}: not declared by the case", finding_label(f)),
            );
        }
        if f.class == FindingClass::Fail && !f.located && !rules.page_level.contains(&f.rule_id) {
            push(
                MismatchKind::LostEvidence,
                format!(
                    "{}: confirmed violation without element location",
                    finding_label(f)
                ),
            );
        }
    }

    // Execution records against findings.
    let finding_ids: BTreeSet<&str> = obs.findings.iter().map(|f| f.rule_id.as_str()).collect();
    for id in &finding_ids {
        if !ran(obs, id) {
            push(
                MismatchKind::FindingWithoutRun,
                format!("{id}: reported findings, but has no completed run record"),
            );
        }
    }
    if rules.run_counts_findings {
        let run_ids: BTreeSet<&str> = obs.runs.iter().map(|r| r.rule_id.as_str()).collect();
        for id in run_ids {
            let reported = obs.findings.iter().filter(|f| f.rule_id == id).count();
            let fails = obs
                .findings
                .iter()
                .filter(|f| f.rule_id == id && f.class == FindingClass::Fail)
                .count();
            let counted: Vec<usize> = obs
                .runs
                .iter()
                .filter(|r| r.rule_id == id)
                .filter_map(|r| match r.state {
                    RunState::Ran { findings } => Some(findings),
                    RunState::NotRun(_) => None,
                })
                .collect();
            if counted.is_empty() {
                continue;
            }
            // A record counts the findings the rule contributed, of any
            // outcome (`a11y_report::RuleRun::findings`); hosts that count
            // confirmed violations only stay below that. One record per
            // viewport, findings deduplicated across viewports — so only the
            // extremes bind: no record may claim more than the report holds,
            // and with violations reported one record must have counted.
            let max = counted.iter().copied().max().unwrap_or(0);
            if max > reported {
                push(
                    MismatchKind::RunContradictsFindings,
                    format!("{id}: run record counts {max} finding(s), report has {reported}"),
                );
            }
            if fails > 0 && max == 0 {
                push(
                    MismatchKind::RunContradictsFindings,
                    format!("{id}: {fails} violation(s) reported, but every run record says clean"),
                );
            }
        }
    }

    out.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.detail.cmp(&b.detail)));
    out.dedup();
    out
}

/// Render the mismatches of several cases as one panic message.
pub fn render_report(title: &str, diffs: &[(String, Vec<Mismatch>)]) -> String {
    let mut msg = format!("{title}:\n");
    for (case, mismatches) in diffs {
        msg.push_str(&format!("  case '{case}':\n"));
        for m in mismatches {
            let side = if m.kind.is_false_negative() {
                "FALSE NEGATIVE"
            } else {
                "FALSE POSITIVE"
            };
            msg.push_str(&format!("    {side} [{:?}]: {}\n", m.kind, m.detail));
        }
    }
    msg
}

/// The WCAG engine's results as an [`Observation`]: violations are `Fail`,
/// warnings and not-testable notes are `Review` (both mean "no automated
/// verdict, a human must look"), positive signals are not findings.
pub fn observe_wcag(results: &auditmysite::wcag::WcagResults) -> Observation {
    let finding = |v: &auditmysite::Violation, class| ObservedFinding {
        rule_id: v.rule_id.clone().unwrap_or_else(|| "<no rule_id>".into()),
        selector: v.selector.clone(),
        class,
        located: v.selector.as_deref().is_some_and(|s| !s.trim().is_empty())
            || v.html_snippet
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty()),
    };
    let mut findings: Vec<ObservedFinding> = results
        .violations
        .iter()
        .map(|v| finding(v, FindingClass::Fail))
        .collect();
    findings.extend(
        results
            .warnings
            .iter()
            .chain(&results.not_testables)
            .map(|v| finding(v, FindingClass::Review)),
    );
    let runs = results
        .rule_outcomes
        .iter()
        .map(|run| ObservedRun {
            rule_id: run.rule_id.clone(),
            state: match run.not_run {
                None => RunState::Ran {
                    findings: run.findings,
                },
                Some(reason) => RunState::NotRun(format!("{reason:?}")),
            },
        })
        .collect();
    Observation { findings, runs }
}
