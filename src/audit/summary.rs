//! Central evaluation and interpretation layer for audit results.
//!
//! Produces a structured [`AuditSummary`] from a [`NormalizedReport`],
//! moving evaluation logic out of the renderer into a dedicated analysis step.
//! This allows reports to react to *what* is wrong, not just *how many* issues
//! there are — and ensures sites with very different problem loads read differently.

use std::collections::HashMap;

use crate::audit::normalized::{NormalizedFinding, NormalizedReport};
use crate::taxonomy::Severity;

// ── Site-state ────────────────────────────────────────────────────────────────

/// Four-level assessment of the overall site health.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteState {
    /// score ≥ 88, no urgent (critical/high) issues — near-perfect
    Polished,
    /// score 70–87 or a few urgent issues — solid but improvable
    NeedsWork,
    /// score 50–69 or multiple critical issues — significant barriers present
    Weak,
    /// score < 50 or ≥ 3 critical issues — acute action required
    Critical,
}

impl SiteState {
    pub fn from_normalized(normalized: &NormalizedReport) -> Self {
        let score = normalized.score;
        let critical = normalized.severity_counts.critical;
        let high = normalized.severity_counts.high;
        let urgent = critical + high;

        if score < 50 || critical >= 3 {
            Self::Critical
        } else if score < 70 || (critical >= 1 && urgent >= 3) {
            Self::Weak
        } else if score < 88 || urgent > 0 {
            Self::NeedsWork
        } else {
            Self::Polished
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Polished => "Stark",
            Self::NeedsWork => "Solide Basis",
            Self::Weak => "Instabil",
            Self::Critical => "Kritisch",
        }
    }
}

// ── Dominant issue ────────────────────────────────────────────────────────────

/// Describes the dominant problem when a single rule accounts for the
/// majority of critical/high findings (threshold: ≥ 45 %).
#[derive(Debug, Clone)]
pub struct DominantIssue {
    pub rule_id: String,
    pub title: String,
    pub severity: Severity,
    /// Number of finding groups (distinct NormalizedFinding entries) for this rule.
    pub count: usize,
    /// Total occurrences across all finding groups (sum of occurrence_count).
    pub occurrence_total: usize,
    /// Share of urgent (critical+high) findings from this rule, 0.0–100.0.
    pub share_pct: f32,
}

// ── Issue pattern ─────────────────────────────────────────────────────────────

/// How problems are distributed across findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssuePattern {
    /// No findings at all.
    Minimal,
    /// One rule accounts for ≥ 45 % of urgent findings.
    SingleDominant,
    /// Multiple distinct clusters of related problems.
    Clustered,
    /// Many small, unrelated issues spread across rules.
    Scattered,
}

// ── Cross-impact ──────────────────────────────────────────────────────────────

/// A finding that spans more than one audit dimension.
#[derive(Debug, Clone)]
pub struct CrossImpact {
    /// E.g. "Accessibility + SEO"
    pub dimensions: String,
    pub description: String,
}

// ── AuditSummary ──────────────────────────────────────────────────────────────

/// Complete interpretation of a [`NormalizedReport`].
/// Created once by [`analyze`], then consumed by the view-model builder.
#[derive(Debug, Clone)]
pub struct AuditSummary {
    pub site_state: SiteState,
    pub dominant_issue: Option<DominantIssue>,
    pub issue_pattern: IssuePattern,
    pub cross_impacts: Vec<CrossImpact>,
    /// Severity- and pattern-aware intro text (replaces generic score-only verdict).
    pub verdict_intro: String,
    /// Pattern-aware problem classification label (replaces build_problem_type).
    pub problem_type_label: String,
    /// One-line note about the dominant issue for highlighted display (if any).
    pub dominant_issue_note: Option<String>,
    pub is_systematic: bool,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Produce a full [`AuditSummary`] from a normalized report.
pub fn analyze(normalized: &NormalizedReport) -> AuditSummary {
    let site_state = SiteState::from_normalized(normalized);
    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let total = normalized.severity_counts.total;
    let urgent = critical + high;

    let is_systematic = total > 40 || (critical >= 5 && total > 25);

    // For dominance detection, compare occurrence counts (not finding groups)
    // so that a rule with many instances is correctly identified as dominant.
    let urgent_occurrences: usize = normalized
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
        .map(|f| f.occurrence_count)
        .sum();

    let dominant_issue = detect_dominant_issue(&normalized.findings, urgent_occurrences);
    let issue_pattern = classify_issue_pattern(total, urgent, &dominant_issue);
    let cross_impacts = detect_cross_impacts(normalized);

    let verdict_intro = build_verdict_intro(
        &site_state,
        &dominant_issue,
        &issue_pattern,
        is_systematic,
        urgent,
        critical,
        high,
        total,
        &cross_impacts,
    );

    let problem_type_label =
        build_problem_type_label(&site_state, &issue_pattern, &dominant_issue, normalized);

    let dominant_issue_note = dominant_issue.as_ref().map(|d| {
        let group_label = if d.count == 1 {
            "1 Problemgruppe".to_string()
        } else {
            format!("{} Problemgruppen", d.count)
        };
        let occurrence_note = if d.occurrence_total > d.count {
            format!(", {} Vorkommen", d.occurrence_total)
        } else {
            String::new()
        };
        format!(
            "\"{}\" macht {:.0}\u{202f}% der kritischen/hohen Findings aus ({group_label}{occurrence_note}).",
            d.title, d.share_pct
        )
    });

    AuditSummary {
        site_state,
        dominant_issue,
        issue_pattern,
        cross_impacts,
        verdict_intro,
        problem_type_label,
        dominant_issue_note,
        is_systematic,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// `urgent_occurrences` is the total occurrence count of all critical/high findings.
/// Dominance is determined by occurrence share, not group count.
fn detect_dominant_issue(
    findings: &[NormalizedFinding],
    urgent_occurrences: usize,
) -> Option<DominantIssue> {
    if urgent_occurrences == 0 {
        return None;
    }

    // Accumulate group count and occurrence total per rule_id.
    let mut rule_counts: HashMap<&str, (&NormalizedFinding, usize, usize)> = HashMap::new();
    for f in findings {
        if matches!(f.severity, Severity::Critical | Severity::High) {
            let entry = rule_counts.entry(&f.rule_id).or_insert((f, 0, 0));
            entry.1 += 1; // group count
            entry.2 += f.occurrence_count; // occurrence total
        }
    }

    rule_counts
        .into_values()
        .filter(|(_, _, occ_total)| (*occ_total as f32 / urgent_occurrences as f32) >= 0.45)
        .max_by_key(|(_, _, occ_total)| *occ_total)
        .map(|(f, count, occurrence_total)| DominantIssue {
            rule_id: f.rule_id.clone(),
            title: f.title.clone(),
            severity: f.severity,
            count,
            occurrence_total,
            share_pct: occurrence_total as f32 / urgent_occurrences as f32 * 100.0,
        })
}

fn classify_issue_pattern(
    total: usize,
    urgent: usize,
    dominant_issue: &Option<DominantIssue>,
) -> IssuePattern {
    if total == 0 {
        IssuePattern::Minimal
    } else if dominant_issue.is_some() {
        IssuePattern::SingleDominant
    } else if urgent > 3 {
        IssuePattern::Clustered
    } else {
        IssuePattern::Scattered
    }
}

fn detect_cross_impacts(normalized: &NormalizedReport) -> Vec<CrossImpact> {
    let mut impacts = Vec::new();

    let has_weak_seo = normalized
        .raw_seo
        .as_ref()
        .map(|s| s.score < 65)
        .unwrap_or(false);
    let has_heading_issues = normalized.findings.iter().any(|f| {
        f.rule_id.to_lowercase().contains("heading")
            || f.title.to_lowercase().contains("überschrift")
            || f.title.to_lowercase().contains("h1")
    });
    if has_weak_seo && has_heading_issues {
        impacts.push(CrossImpact {
            dimensions: "Accessibility + SEO".into(),
            description: "Fehlende Überschriftenstruktur wirkt gleichzeitig als SEO-Schwäche und Accessibility-Barriere.".into(),
        });
    }

    let has_security_issues = normalized
        .raw_security
        .as_ref()
        .map(|s| s.score < 60)
        .unwrap_or(false);
    let has_mobile_issues = normalized
        .raw_mobile
        .as_ref()
        .map(|m| m.score < 60)
        .unwrap_or(false);
    if has_security_issues && has_mobile_issues {
        impacts.push(CrossImpact {
            dimensions: "Security + Mobile".into(),
            description: "Security-Schwächen und Mobile-Probleme treten gemeinsam auf — Optimierungsbedarf zieht sich durch mehrere Bereiche.".into(),
        });
    }

    let has_perf_issues = normalized
        .raw_performance
        .as_ref()
        .map(|p| p.score.overall < 60)
        .unwrap_or(false);
    if has_perf_issues && has_mobile_issues {
        impacts.push(CrossImpact {
            dimensions: "Performance + Mobile".into(),
            description: "Schlechte Ladezeiten verstärken Mobile-Probleme — Mobile-Nutzbarkeit ist doppelt eingeschränkt.".into(),
        });
    }

    impacts
}

#[allow(clippy::too_many_arguments)]
fn build_verdict_intro(
    site_state: &SiteState,
    dominant_issue: &Option<DominantIssue>,
    issue_pattern: &IssuePattern,
    is_systematic: bool,
    urgent: usize,
    critical: usize,
    high: usize,
    total: usize,
    cross_impacts: &[CrossImpact],
) -> String {
    let cross_note = cross_impacts
        .first()
        .map(|c| format!(" {}", c.description))
        .unwrap_or_default();

    // Systematic problems override pattern-specific wording
    if is_systematic {
        return format!(
            "Kein Einzelproblem — {total} Verstöße über {critical} kritische und {high} hohe Themen sind ein \
             systematisches Muster. Betrifft große Teile der Seite, nicht einzelne Stellen.{cross_note}"
        );
    }

    // Single dominant issue: language shifts to focus on the one root cause
    if let Some(d) = dominant_issue {
        let detail = if d.occurrence_total > d.count {
            format!("{} Vorkommen", d.occurrence_total)
        } else {
            format!("{} Treffer", d.count)
        };
        return format!(
            "Ein Problem dominiert: \"{}\" verursacht {:.0} % der kritischen Findings ({detail}). \
             Hier konzentrieren -- eine Ursache, hoher Impact.{cross_note}",
            d.title, d.share_pct
        );
    }

    match (site_state, issue_pattern) {
        (SiteState::Polished, _) => {
            if urgent == 0 {
                format!("Sehr gutes Ergebnis — keine dringenden Barrieren.{cross_note} Niveau halten und regelmäßig nachprüfen.")
            } else {
                format!(
                    "Technisch stark aufgestellt. {} priorisierte{} Thema{} -- gezielt beheben, bevor sie sich häufen.{cross_note}",
                    urgent,
                    if urgent == 1 { "s" } else { "" },
                    if urgent == 1 { "" } else { "n" }
                )
            }
        }
        (SiteState::NeedsWork, IssuePattern::Clustered) => format!(
            "Solide Basis, aber {urgent} priorisierte Themen verteilen sich auf mehrere unabhängige Bereiche. \
             Strukturiert priorisieren — nicht alles auf einmal.{cross_note}"
        ),
        (SiteState::NeedsWork, _) => {
            if urgent == 0 {
                format!("Solide Basis ohne akute Risiken. {total} Verbesserungen möglich — gut priorisierbar.{cross_note}")
            } else {
                format!(
                    "Gute Basis, aber {} priorisierte{} Thema{} braucht{} sofortige Aufmerksamkeit.{cross_note}",
                    urgent,
                    if urgent == 1 { "s" } else { "" },
                    if urgent == 1 { "" } else { "n" },
                    if urgent == 1 { "" } else { "n" }
                )
            }
        }
        (SiteState::Weak, _) => format!(
            "Relevante Barrieren vorhanden -- {urgent} davon kritisch oder hoch. \
             Strukturiert priorisieren und Phase 1 starten.{cross_note}"
        ),
        (SiteState::Critical, _) => format!(
            "Akuter Handlungsbedarf: {critical} kritische, {high} hohe Issues. \
             Die Seite ist für einen Teil der Nutzer schwer nutzbar -- sofort Phase 1 starten.{cross_note}"
        ),
    }
}

fn build_problem_type_label(
    site_state: &SiteState,
    issue_pattern: &IssuePattern,
    dominant_issue: &Option<DominantIssue>,
    normalized: &NormalizedReport,
) -> String {
    let total = normalized.severity_counts.total;
    let critical = normalized.severity_counts.critical;
    let high = normalized.severity_counts.high;
    let rule_count = normalized
        .findings
        .iter()
        .map(|f| f.wcag_criterion.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    let is_structural = total > 30
        || (critical >= 5 && total > 20)
        || (rule_count >= 7 && total > 25 && (critical + high) >= 15);

    match issue_pattern {
        IssuePattern::Minimal => {
            "Keine Verstöße gefunden — volle Konformität im geprüften Umfang".into()
        }
        IssuePattern::SingleDominant => {
            if let Some(d) = dominant_issue {
                format!(
                    "Dominierendes Einzelproblem: \"{}\" — konzentriert und gezielt behebbar",
                    d.title
                )
            } else {
                "Einzelproblem — konzentriert und gezielt behebbar".into()
            }
        }
        IssuePattern::Clustered if is_structural => {
            "Strukturelle Defizite — flächendeckende Barrieren in mehreren Bereichen".into()
        }
        IssuePattern::Clustered => {
            "Mehrere Problemcluster — über verschiedene Bereiche verteilt, gezielt behebbar".into()
        }
        IssuePattern::Scattered => {
            if matches!(site_state, SiteState::Polished | SiteState::NeedsWork) {
                "Feinschliff — keine strukturellen Defizite, letzte Optimierungshebel".into()
            } else {
                "Mehrere kritische Einzelprobleme — konzentriert und gezielt behebbar".into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::{
        NormalizedFinding, NormalizedReport, ReportVisibilityData, ScoreImpactData, SeverityCounts,
    };
    use crate::taxonomy::Severity;
    use crate::WcagLevel;

    fn make_finding(
        rule_id: &str,
        title: &str,
        severity: Severity,
        count: usize,
    ) -> NormalizedFinding {
        NormalizedFinding {
            rule_id: rule_id.into(),
            wcag_criterion: "1.1.1".into(),
            wcag_level: "A".into(),
            dimension: "Accessibility".into(),
            subcategory: "Images".into(),
            issue_class: "Missing".into(),
            severity,
            user_impact: String::new(),
            technical_impact: String::new(),
            score_impact: ScoreImpactData {
                base_penalty: 5.0,
                max_penalty: 20.0,
                scaling: "Logarithmic".into(),
            },
            report_visibility: ReportVisibilityData::default(),
            aggregation_key: rule_id.into(),
            title: title.into(),
            description: String::new(),
            occurrence_count: count,
            priority_score: 1.0,
            occurrences: vec![],
        }
    }

    fn make_report(score: u32, findings: Vec<NormalizedFinding>) -> NormalizedReport {
        let critical = findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count();
        let high = findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .count();
        let medium = findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .count();
        let low = findings
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .count();
        let total = findings.len();

        NormalizedReport {
            url: "https://example.com".into(),
            wcag_level: WcagLevel::AA,
            timestamp: chrono::Utc::now(),
            duration_ms: 0,
            nodes_analyzed: 100,
            score,
            grade: "B".into(),
            certificate: "None".into(),
            overall_score: score,
            findings,
            severity_counts: SeverityCounts {
                critical,
                high,
                medium,
                low,
                total,
            },
            module_scores: vec![],
            raw_performance: None,
            raw_seo: None,
            raw_security: None,
            raw_mobile: None,
            risk: crate::audit::normalized::RiskAssessment {
                level: crate::audit::normalized::RiskLevel::Low,
                critical_issues: 0,
                high_issues: 0,
                legal_flags: 0,
                blocking_issues: 0,
                summary: String::new(),
            },
            raw_ux: None,
            raw_journey: None,
            raw_dark_mode: None,
            raw_source_quality: None,
            raw_wcag: crate::wcag::WcagResults::new(),
        }
    }

    #[test]
    fn test_site_state_polished() {
        let report = make_report(92, vec![]);
        assert_eq!(SiteState::from_normalized(&report), SiteState::Polished);
    }

    #[test]
    fn test_site_state_critical_by_score() {
        let report = make_report(40, vec![]);
        assert_eq!(SiteState::from_normalized(&report), SiteState::Critical);
    }

    #[test]
    fn test_site_state_critical_by_critical_count() {
        let findings = vec![
            make_finding("r1", "T1", Severity::Critical, 1),
            make_finding("r2", "T2", Severity::Critical, 1),
            make_finding("r3", "T3", Severity::Critical, 1),
        ];
        let report = make_report(75, findings);
        assert_eq!(SiteState::from_normalized(&report), SiteState::Critical);
    }

    #[test]
    fn test_dominant_issue_detected() {
        // 5 occurrences from one rule out of 6 urgent = 83%
        let findings = vec![
            make_finding("r.dominant", "Alt-Text fehlt", Severity::Critical, 5),
            make_finding("r.other", "Kontrast", Severity::High, 1),
        ];
        let report = make_report(60, findings);
        let summary = analyze(&report);
        assert!(summary.dominant_issue.is_some());
        let d = summary.dominant_issue.unwrap();
        assert_eq!(d.rule_id, "r.dominant");
        assert!(d.share_pct >= 50.0);
        assert_eq!(summary.issue_pattern, IssuePattern::SingleDominant);
    }

    #[test]
    fn test_no_dominant_issue_when_evenly_spread() {
        let findings = vec![
            make_finding("r1", "A", Severity::Critical, 2),
            make_finding("r2", "B", Severity::High, 2),
            make_finding("r3", "C", Severity::High, 2),
        ];
        let report = make_report(55, findings);
        let summary = analyze(&report);
        assert!(summary.dominant_issue.is_none());
    }

    #[test]
    fn test_verdict_intro_differs_by_state() {
        let polished = analyze(&make_report(95, vec![]));
        let critical_findings = vec![
            make_finding("r1", "T", Severity::Critical, 1),
            make_finding("r2", "T", Severity::Critical, 1),
            make_finding("r3", "T", Severity::Critical, 1),
        ];
        let critical = analyze(&make_report(40, critical_findings));
        assert_ne!(polished.verdict_intro, critical.verdict_intro);
    }

    #[test]
    fn test_issue_pattern_minimal_when_no_findings() {
        let report = make_report(100, vec![]);
        let summary = analyze(&report);
        assert_eq!(summary.issue_pattern, IssuePattern::Minimal);
    }
}
