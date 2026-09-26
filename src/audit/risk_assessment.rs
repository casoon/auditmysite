//! Risikobewertung des normalisierten Reports.
//!
//! Leitet aus den Befunden, ihren Vorkommen, den interaktiven Befunden und
//! der Screenreader-Zusammenfassung die `RiskAssessment` ab: Stufe, Score,
//! auslösende Schwelle, rechtlich relevante und blockierende Befunde.
//! Aufgerufen nur von `normalize`.

use crate::audit::normalized::{
    InteractiveFinding, NormalizedFinding, RiskAssessment, RiskLevel, SeverityCounts,
};
use crate::taxonomy::Severity;

pub(super) fn compute_risk_assessment(
    findings: &[NormalizedFinding],
    occurrence_counts: &SeverityCounts,
    interactive_findings: &[InteractiveFinding],
    screen_reader: Option<&crate::screen_reader::ScreenReaderSummary>,
    score: u32,
    overall_score: u32,
) -> RiskAssessment {
    // Risk thresholds reflect total affected elements (occurrence_counts),
    // not the number of distinct rules (severity_counts).
    let critical_issues = occurrence_counts.critical;
    let high_issues = occurrence_counts.high;

    // Legal flags: count distinct WCAG Level A rules with High/Critical severity.
    // Per-occurrence counting would inflate the number (e.g. 1000 images without
    // alt text is one rule violation, not 1000 legal flags).
    let mut legal_flags = findings
        .iter()
        .filter(|f| {
            f.wcag_level == "A" && matches!(f.severity, Severity::Critical | Severity::High)
        })
        .count();

    // The screen-reader audit can detect journey-level BFSG barriers the static
    // WCAG engine misses, so the main report and the screen-reader sidecar should
    // not disagree on legal status (#484). Only *confirmed* Level-A blockers raise
    // the legal flag — the SR "high" severity findings (empty interactive elements
    // 4.1.2, unlabeled form fields 3.3.2). Medium/low heuristics (generic link
    // text, tab-stop count, heading order) are deliberately excluded so soft
    // signals do not inflate legal risk on otherwise well-maintained sites. The
    // consent-wall guard (#483) keeps incomplete audits from contributing.
    if let Some(sr) = screen_reader {
        let consent_wall = matches!(
            sr.summary.audit_quality,
            crate::screen_reader::SrAuditQuality::ConsentWallSuspected
        );
        if sr.bfsg_compliance.verdict == crate::screen_reader::BfsgVerdict::NonCompliant
            && sr.count_severity("high") > 0
            && !consent_wall
        {
            legal_flags = legal_flags.max(1);
        }
    }

    // Blocking issues: interactive elements without accessible names (4.1.2/2.1.1).
    // Only Medium+ severity — Low findings (e.g. accordion advisory) are not blockers.
    let blocking_issues = findings
        .iter()
        .filter(|f| {
            (f.wcag_criterion == "4.1.2" || f.wcag_criterion == "2.1.1")
                && matches!(
                    f.severity,
                    Severity::Medium | Severity::High | Severity::Critical
                )
        })
        .map(|f| f.occurrence_count)
        .sum::<usize>();
    let interactive_critical_issues = interactive_findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .count();
    let interactive_high_issues = interactive_findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();

    // Screen-reader audit issues are heuristic quality signals (reading order,
    // landmark/heading quality). They contribute to the risk score but never to
    // the legally-relevant severity_counts/legal_flags (#411). SR emits no
    // Critical severity — only low/medium/high strings.
    let (sr_high_issues, sr_medium_issues) = screen_reader
        .map(|sr| (sr.count_severity("high"), sr.count_severity("medium")))
        .unwrap_or((0, 0));

    let risk_score = (legal_flags as u32 * 20
        + critical_issues as u32 * 10
        + high_issues as u32 * 3
        + blocking_issues as u32 * 2
        + interactive_critical_issues as u32 * 10
        + interactive_high_issues as u32 * 5
        + sr_high_issues as u32 * 3
        + sr_medium_issues as u32)
        .min(100);

    // Risk level — explicit precedence; legal_flags and blocking_issues both
    // raise the floor even when critical_issues is zero (see issue #250).
    //
    // "Critical" is reserved for *systemic* legal exposure: breadth (≥3 distinct
    // WCAG Level A rules with High/Critical severity) OR volume (≥5 critical
    // occurrences). The breadth path must NOT also require a critical occurrence
    // — legal_flags already counts High-severity Level A rules, so gating it on
    // `critical_issues > 0` made a site with 4 High-severity legal barriers rank
    // *below* one with a single flag but 5+ critical occurrences (#457). A single
    // isolated legal flag with a few critical occurrences stays High (#250).
    let level = if legal_flags >= 3 || critical_issues >= 5 {
        RiskLevel::Critical
    } else if (legal_flags > 0 && critical_issues > 0)
        || critical_issues >= 3
        || blocking_issues >= 5
        || risk_score >= 80
    {
        RiskLevel::High
    } else if (high_issues >= 3 && score < 80)
        || critical_issues >= 1
        || legal_flags > 0
        || blocking_issues >= 1
        || interactive_critical_issues > 0
        || interactive_high_issues > 0
        || score <= 20
    {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };

    let plural = |n: usize, singular: &str, plural: &str| -> String {
        if n == 1 {
            format!("{} {}", n, singular)
        } else {
            format!("{} {}", n, plural)
        }
    };
    let summary = match level {
        RiskLevel::Critical => {
            // If the overall score is high (≥ 80, i.e. Grade A or B), the
            // Critical risk level alongside a strong grade looks
            // contradictory. Surface the contrast explicitly so the report
            // explains why both signals can hold at once. See issue #237.
            let prefix = if overall_score >= 80 {
                "Critical risk despite high overall score"
            } else {
                "Critical risk"
            };
            if legal_flags >= 3 {
                // Breadth-driven: multiple distinct legally-relevant barriers.
                format!(
                    "{}: {} with legal relevance (BFSG). {}.",
                    prefix,
                    plural(
                        legal_flags,
                        "WCAG Level A violation",
                        "WCAG Level A violations"
                    ),
                    plural(
                        blocking_issues,
                        "blocker on interactive elements",
                        "blockers on interactive elements"
                    )
                )
            } else {
                // Volume-driven: a high number of critical occurrences.
                format!(
                    "{}: {} on interactive elements and content.",
                    prefix,
                    plural(critical_issues, "critical violation", "critical violations")
                )
            }
        }
        RiskLevel::High => format!(
            "High risk: {} and {}. Users are actively excluded.",
            plural(critical_issues, "critical issue", "critical issues"),
            plural(high_issues, "serious issue", "serious issues")
        ),
        RiskLevel::Medium => {
            if legal_flags > 0 {
                format!(
                    "Medium risk: {} with legal relevance (BFSG){}.",
                    plural(
                        legal_flags,
                        "WCAG Level A violation",
                        "WCAG Level A violations"
                    ),
                    if blocking_issues > 0 {
                        format!(
                            ", {} on interactive elements",
                            plural(blocking_issues, "blocker", "blockers")
                        )
                    } else {
                        String::new()
                    }
                )
            } else if blocking_issues > 0 {
                format!(
                    "Medium risk: {} detected on interactive elements. \
                     Restrictions for certain user groups.",
                    plural(blocking_issues, "blocker", "blockers")
                )
            } else if interactive_critical_issues > 0 {
                format!(
                    "Medium risk: {} from interactive keyboard and state-transition tests.",
                    plural(
                        interactive_critical_issues,
                        "critical finding",
                        "critical findings"
                    )
                )
            } else if interactive_high_issues > 0 {
                format!(
                    "Medium risk: {} from interactive keyboard and state-transition tests.",
                    plural(
                        interactive_high_issues,
                        "serious finding",
                        "serious findings"
                    )
                )
            } else {
                format!(
                    "Medium risk: {} detected. Restrictions for certain user groups.",
                    plural(
                        high_issues + critical_issues,
                        "serious issue",
                        "serious issues"
                    )
                )
            }
        }
        RiskLevel::Low => {
            let notable = interactive_high_issues + interactive_critical_issues;
            if notable > 0 {
                format!(
                    "Low risk: No critical violations — keyboard journey contains {}, manual review recommended.",
                    plural(notable, "notable finding", "notable findings")
                )
            } else {
                "Low risk: No critical violations — improvement potential exists.".to_string()
            }
        }
    };
    let (threshold, driven_by) = match level {
        RiskLevel::Critical => (
            60u32,
            // Attribute to the condition that actually triggered Critical:
            // breadth of legal exposure (≥3 distinct Level A rules) vs. a high
            // volume of critical occurrences. A single flag that rode in on
            // volume must not be labelled "Legal Compliance" (#457).
            if legal_flags >= 3 {
                "Legal Compliance"
            } else {
                "Accessibility"
            }
            .to_string(),
        ),
        RiskLevel::High => (30u32, "Accessibility".to_string()),
        RiskLevel::Medium => (
            10u32,
            if interactive_critical_issues > 0 {
                "Accessibility Journey"
            } else if score <= 20 {
                "Score"
            } else {
                "Accessibility"
            }
            .to_string(),
        ),
        RiskLevel::Low => (0u32, String::new()),
    };

    RiskAssessment {
        level,
        score: risk_score,
        threshold,
        driven_by,
        critical_issues,
        high_issues,
        legal_flags,
        blocking_issues,
        interactive_critical_issues,
        interactive_high_issues,
        summary,
    }
}
