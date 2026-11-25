//! Central evaluation and interpretation layer for audit results.
//!
//! Produces a structured [`AuditSummary`] from a [`NormalizedReport`],
//! moving evaluation logic out of the renderer into a dedicated analysis step.
//! This allows reports to react to *what* is wrong, not just *how many* issues
//! there are — and ensures sites with very different problem loads read differently.

use std::collections::HashMap;

use crate::audit::normalized::{NormalizedFinding, NormalizedReport};
use crate::i18n::{count_text, text, text_args};
use crate::taxonomy::Severity;
use fluent_bundle::FluentValue;

// ── Site-state ────────────────────────────────────────────────────────────────

/// Four-level assessment of the overall site health.
///
/// The score part follows the score bands (90/75/60/40, single source
/// `ScoreBand::from_score`), so the label and the cover sentence never name
/// contradicting tiers (#899): Excellent = Polished, Good/NeedsImprovement =
/// NeedsWork, Weak = Weak, Critical = Critical. Open urgent findings can only
/// lower the state, never raise it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SiteState {
    /// score band 90-100 and no urgent (critical/high) issues
    Polished,
    /// score band 60-89, or a better score with urgent issues open
    NeedsWork,
    /// score band 40-59, or several urgent issues including a critical one
    Weak,
    /// score below 40 or ≥ 3 critical issues — acute action required
    Critical,
}

impl SiteState {
    /// The state the score alone allows, banded like the score bands.
    pub fn from_score(score: u32) -> Self {
        use crate::audit::interpretation::ScoreBand;
        match ScoreBand::from_score(score as f32) {
            ScoreBand::Excellent => Self::Polished,
            ScoreBand::Good | ScoreBand::NeedsImprovement => Self::NeedsWork,
            ScoreBand::Weak => Self::Weak,
            ScoreBand::Critical => Self::Critical,
        }
    }

    pub fn from_normalized(normalized: &NormalizedReport) -> Self {
        // Use occurrence-level counts so thresholds reflect the total mass of
        // affected elements, not just the number of distinct rules fired.
        let critical = normalized.occurrence_counts.critical;
        let high = normalized.occurrence_counts.high;
        let urgent = critical + high;

        let floor = if critical >= 3 {
            Self::Critical
        } else if critical >= 1 && urgent >= 3 {
            Self::Weak
        } else if urgent > 0 {
            Self::NeedsWork
        } else {
            Self::Polished
        };
        Self::from_score(normalized.score).max(floor)
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Polished => "Strong",
            Self::NeedsWork => "Stable foundation",
            Self::Weak => "Unstable",
            Self::Critical => "Critical",
        }
    }

    pub fn label_localized(&self, i18n: &crate::i18n::I18n) -> String {
        let key = match self {
            Self::Polished => "site-state-polished",
            Self::NeedsWork => "site-state-needs-work",
            Self::Weak => "site-state-weak",
            Self::Critical => "site-state-critical",
        };
        i18n.t(key)
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

/// Produce a full [`AuditSummary`] from a normalized report (German narrative).
pub fn analyze(normalized: &NormalizedReport) -> AuditSummary {
    analyze_with_locale(normalized, "de")
}

/// Locale-aware variant of [`analyze`]. Currently supports `"de"` (default)
/// and `"en"`; unknown locales fall back to the German narrative.
pub fn analyze_with_locale(normalized: &NormalizedReport, locale: &str) -> AuditSummary {
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
    let cross_impacts = detect_cross_impacts(normalized, locale);

    let verdict_intro = build_verdict_intro(
        locale,
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

    let problem_type_label = build_problem_type_label(
        locale,
        &site_state,
        &issue_pattern,
        &dominant_issue,
        normalized,
    );

    let en = locale == "en";
    let dominant_issue_note = dominant_issue.as_ref().map(|d| {
        let groups = count_text("audit-interpretation-summary-groups", d.count, en);
        let occurrences = if d.occurrence_total > d.count {
            count_text(
                "audit-interpretation-summary-occurrences-note",
                d.occurrence_total,
                en,
            )
        } else {
            String::new()
        };
        text_args(
            "audit-interpretation-summary-dominant-note",
            &[
                ("title", FluentValue::from(d.title.clone())),
                ("share", FluentValue::from(format!("{:.0}", d.share_pct))),
                ("groups", FluentValue::from(groups)),
                ("occurrences", FluentValue::from(occurrences)),
            ],
            en,
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

/// A count as a Fluent number, so a plural selector can use it.
fn number(value: usize) -> FluentValue<'static> {
    FluentValue::from(value as i64)
}

/// `urgent_occurrences` is the total occurrence count of all critical/high findings.
/// Dominance is determined by occurrence share, not group count.
fn detect_dominant_issue(
    findings: &[NormalizedFinding],
    urgent_occurrences: usize,
) -> Option<DominantIssue> {
    // Phase 1: dominance among critical/high findings (threshold ≥ 45 %).
    if urgent_occurrences > 0 {
        let mut rule_counts: HashMap<&str, (&NormalizedFinding, usize, usize)> = HashMap::new();
        for f in findings {
            if matches!(f.severity, Severity::Critical | Severity::High) {
                let entry = rule_counts.entry(&f.rule_id).or_insert((f, 0, 0));
                entry.1 += 1; // group count
                entry.2 += f.occurrence_count; // occurrence total
            }
        }
        let result = rule_counts
            .into_values()
            .filter(|(_, _, occ_total)| (*occ_total as f32 / urgent_occurrences as f32) >= 0.45)
            .max_by(|a, b| a.2.cmp(&b.2).then_with(|| b.0.rule_id.cmp(&a.0.rule_id)))
            .map(|(f, count, occurrence_total)| DominantIssue {
                rule_id: f.rule_id.clone(),
                title: f.title.clone(),
                severity: f.severity,
                count,
                occurrence_total,
                share_pct: occurrence_total as f32 / urgent_occurrences as f32 * 100.0,
            });
        if result.is_some() {
            return result;
        }
    }

    // Phase 2: when no critical/high dominance is found, check all findings.
    // Higher threshold (≥ 60 %) because medium/low findings have less individual weight.
    let all_occurrences: usize = findings.iter().map(|f| f.occurrence_count).sum();
    if all_occurrences == 0 {
        return None;
    }
    let mut rule_counts: HashMap<&str, (&NormalizedFinding, usize, usize)> = HashMap::new();
    for f in findings {
        let entry = rule_counts.entry(&f.rule_id).or_insert((f, 0, 0));
        entry.1 += 1;
        entry.2 += f.occurrence_count;
    }
    rule_counts
        .into_values()
        .filter(|(_, _, occ_total)| (*occ_total as f32 / all_occurrences as f32) >= 0.60)
        .max_by(|a, b| a.2.cmp(&b.2).then_with(|| b.0.rule_id.cmp(&a.0.rule_id)))
        .map(|(f, count, occurrence_total)| DominantIssue {
            rule_id: f.rule_id.clone(),
            title: f.title.clone(),
            severity: f.severity,
            count,
            occurrence_total,
            share_pct: occurrence_total as f32 / all_occurrences as f32 * 100.0,
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

fn detect_cross_impacts(normalized: &NormalizedReport, locale: &str) -> Vec<CrossImpact> {
    let mut impacts = Vec::new();
    let en = locale == "en";

    let has_weak_seo = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "SEO")
        .map(|m| m.score < 65)
        .unwrap_or(false);
    let has_heading_issues = normalized.findings.iter().any(|f| {
        f.rule_id.to_lowercase().contains("heading")
            || f.title.to_lowercase().contains("überschrift")
            || f.title.to_lowercase().contains("h1")
    });
    if has_weak_seo && has_heading_issues {
        impacts.push(CrossImpact {
            dimensions: "Accessibility + SEO".into(),
            description: text("audit-interpretation-summary-cross-headings", en),
        });
    }

    let has_security_issues = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Security")
        .map(|m| m.score < 60)
        .unwrap_or(false);
    let has_mobile_issues = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Mobile")
        .map(|m| m.score < 60)
        .unwrap_or(false);
    if has_security_issues && has_mobile_issues {
        impacts.push(CrossImpact {
            dimensions: "Security + Mobile".into(),
            description: text("audit-interpretation-summary-cross-security-mobile", en),
        });
    }

    let has_perf_issues = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Performance")
        .map(|m| m.score < 60)
        .unwrap_or(false);
    if has_perf_issues && has_mobile_issues {
        impacts.push(CrossImpact {
            dimensions: "Performance + Mobile".into(),
            description: text("audit-interpretation-summary-cross-performance-mobile", en),
        });
    }

    impacts
}

#[allow(clippy::too_many_arguments)]
fn build_verdict_intro(
    locale: &str,
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
    let en = locale == "en";
    let text_with = |key: &str, args: &[(&str, FluentValue<'_>)]| {
        let mut all = vec![("cross", FluentValue::from(cross_note.clone()))];
        all.extend(args.iter().cloned());
        text_args(key, &all, en)
    };

    // Systematic problems override pattern-specific wording
    if is_systematic {
        return text_with(
            "audit-interpretation-summary-intro-systematic",
            &[
                ("total", number(total)),
                ("critical", number(critical)),
                ("high", number(high)),
            ],
        );
    }

    // Single dominant issue: language shifts to focus on the one root cause
    if let Some(d) = dominant_issue {
        let detail = if d.occurrence_total > d.count {
            count_text(
                "audit-interpretation-summary-detail-occurrences",
                d.occurrence_total,
                en,
            )
        } else {
            count_text("audit-interpretation-summary-detail-hits", d.count, en)
        };
        return text_with(
            "audit-interpretation-summary-intro-dominant",
            &[
                ("title", FluentValue::from(d.title.clone())),
                ("share", FluentValue::from(format!("{:.0}", d.share_pct))),
                ("detail", FluentValue::from(detail)),
            ],
        );
    }

    match (site_state, issue_pattern) {
        (SiteState::Polished, _) => {
            if urgent == 0 {
                text_with("audit-interpretation-summary-intro-polished-clear", &[])
            } else {
                text_with(
                    "audit-interpretation-summary-intro-polished-topics",
                    &[("urgent", number(urgent))],
                )
            }
        }
        (SiteState::NeedsWork, IssuePattern::Clustered) => text_with(
            "audit-interpretation-summary-intro-clustered",
            &[("urgent", number(urgent))],
        ),
        (SiteState::NeedsWork, _) => {
            if urgent == 0 {
                text_with(
                    "audit-interpretation-summary-intro-needs-work-clear",
                    &[("total", number(total))],
                )
            } else {
                text_with(
                    "audit-interpretation-summary-intro-needs-work-urgent",
                    &[("urgent", number(urgent))],
                )
            }
        }
        (SiteState::Weak, _) => text_with(
            "audit-interpretation-summary-intro-weak",
            &[("urgent", number(urgent))],
        ),
        (SiteState::Critical, _) => text_with(
            "audit-interpretation-summary-intro-critical",
            &[("critical", number(critical)), ("high", number(high))],
        ),
    }
}

fn build_problem_type_label(
    locale: &str,
    site_state: &SiteState,
    issue_pattern: &IssuePattern,
    dominant_issue: &Option<DominantIssue>,
    normalized: &NormalizedReport,
) -> String {
    // Structural thresholds reflect element-level mass, not the number of
    // distinct rules — keep using occurrence_counts here.
    let total = normalized.occurrence_counts.total;
    let critical = normalized.occurrence_counts.critical;
    let high = normalized.occurrence_counts.high;
    let rule_count = normalized
        .findings
        .iter()
        .map(|f| f.wcag_criterion.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    let is_structural = total > 30
        || (critical >= 5 && total > 20)
        || (rule_count >= 7 && total > 25 && (critical + high) >= 15);

    let en = locale == "en";

    match issue_pattern {
        IssuePattern::Minimal => text("audit-interpretation-summary-problem-minimal", en),
        IssuePattern::SingleDominant => {
            if let Some(d) = dominant_issue {
                text_args(
                    "audit-interpretation-summary-problem-dominant",
                    &[("title", FluentValue::from(d.title.clone()))],
                    en,
                )
            } else {
                text("audit-interpretation-summary-problem-single", en)
            }
        }
        IssuePattern::Clustered if is_structural => {
            text("audit-interpretation-summary-problem-structural", en)
        }
        IssuePattern::Clustered => text("audit-interpretation-summary-problem-clusters", en),
        IssuePattern::Scattered => {
            if matches!(site_state, SiteState::Polished | SiteState::NeedsWork) {
                text("audit-interpretation-summary-problem-polish", en)
            } else {
                text("audit-interpretation-summary-problem-isolated", en)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::{
        ComplexityKind, ExpectedImpactKind, NormalizedFinding, NormalizedReport,
        ReportVisibilityData, ScoreEffect, ScoreImpactData, SeverityCounts,
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
            known: false,
            contract: Default::default(),
            origin: Default::default(),
            providers: Default::default(),
            observed_roles: Default::default(),
            id: String::new(),
            patterns: Vec::new(),
            pattern_count: 0,
            rule_id: rule_id.into(),
            wcag_criterion: "1.1.1".into(),
            axe_id: None,
            wcag_level: "A".into(),
            dimension: "Accessibility".into(),
            subcategory: "Images".into(),
            issue_class: "Missing".into(),
            dimension_kind: crate::taxonomy::Dimension::Accessibility,
            subcategory_kind: crate::taxonomy::Subcategory::ContentAlternatives,
            issue_class_kind: crate::taxonomy::IssueClass::Missing,
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
            help_url: None,
            understanding_url: None,
            occurrence_count: count,
            priority_score: 1.0,
            confidence: "very_high".into(),
            false_positive_risk: "very_low".into(),
            verification: "automatically_confirmed".into(),
            assurance: Default::default(),
            complexity: "low".into(),
            complexity_reason: "Test fixture".into(),
            complexity_kind: ComplexityKind::LowScope,
            expected_impact: "Test fixture".into(),
            expected_impact_kind: ExpectedImpactKind::Wcag {
                occurrence_count: count,
                score_effect: ScoreEffect::Low,
                wcag_level: "A".into(),
                score_gain: None,
            },
            bfsg_relevance: "medium".into(),
            remediation_priority: "normal".into(),
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
        let crit_occ: usize = findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .map(|f| f.occurrence_count)
            .sum();
        let high_occ: usize = findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .map(|f| f.occurrence_count)
            .sum();
        let medium_occ: usize = findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .map(|f| f.occurrence_count)
            .sum();
        let low_occ: usize = findings
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .map(|f| f.occurrence_count)
            .sum();
        let total_occ: usize = findings.iter().map(|f| f.occurrence_count).sum();

        NormalizedReport {
            decision: crate::audit::verdict::VerdictResult::undecided(),
            baseline: None,
            finding_inventory: Vec::new(),
            url: "https://example.com".into(),
            wcag_level: WcagLevel::AA,
            timestamp: chrono::Utc::now(),
            duration_ms: 0,
            nodes_analyzed: 100,
            score,
            grade: "B".into(),
            site_quality_score: score,
            findings,
            severity_counts: SeverityCounts {
                critical,
                high,
                medium,
                low,
                total,
            },
            occurrence_counts: SeverityCounts {
                critical: crit_occ,
                high: high_occ,
                medium: medium_occ,
                low: low_occ,
                total: total_occ,
            },
            accessibility_assessments: vec![],
            rule_outcomes: vec![],
            execution: Default::default(),
            module_scores: vec![],
            security_category_scores: vec![],
            audit_flags: vec![],
            consent_privacy: None,
            has_screenshots: false,
            viewport_scores: None,
            score_calculation_method: "module_weighted".to_string(),
            score_breakdown: None,
            interactive_findings: Vec::new(),
            accessibility_journey: None,
            screen_reader: None,
            interpretation: None,
            risk: crate::audit::normalized::RiskAssessment {
                level: crate::audit::normalized::RiskLevel::Low,
                score: 0,
                threshold: 0,
                driven_by: String::new(),
                critical_issues: 0,
                high_issues: 0,
                legal_flags: 0,
                severe_legal_flags: 0,
                blocking_issues: 0,
                keyboard_blocking_issues: 0,
                interactive_critical_issues: 0,
                interactive_high_issues: 0,
                summary: String::new(),
            },
            principle_coverage: Default::default(),
        }
    }

    /// #899: sweeping the score, the state label never contradicts the
    /// score band.
    #[test]
    fn site_state_never_contradicts_score_band() {
        use crate::audit::interpretation::ScoreBand;
        for score in 0..=100u32 {
            let state = SiteState::from_normalized(&make_report(score, vec![]));
            let expected = match ScoreBand::from_score(score as f32) {
                ScoreBand::Excellent => SiteState::Polished,
                ScoreBand::Good | ScoreBand::NeedsImprovement => SiteState::NeedsWork,
                ScoreBand::Weak => SiteState::Weak,
                ScoreBand::Critical => SiteState::Critical,
            };
            assert_eq!(state, expected, "score {score}");
        }
        // Band edges 90/75/60/40.
        assert_eq!(SiteState::from_score(90), SiteState::Polished);
        assert_eq!(SiteState::from_score(89), SiteState::NeedsWork);
        assert_eq!(SiteState::from_score(60), SiteState::NeedsWork);
        assert_eq!(SiteState::from_score(59), SiteState::Weak);
        assert_eq!(SiteState::from_score(40), SiteState::Weak);
        assert_eq!(SiteState::from_score(39), SiteState::Critical);
    }

    #[test]
    fn site_state_urgent_findings_only_lower_the_state() {
        let one_high = vec![make_finding("r1", "T1", Severity::High, 1)];
        assert_eq!(
            SiteState::from_normalized(&make_report(95, one_high)),
            SiteState::NeedsWork
        );
        let weak_score = vec![make_finding("r1", "T1", Severity::High, 1)];
        assert_eq!(
            SiteState::from_normalized(&make_report(45, weak_score)),
            SiteState::Weak
        );
    }

    #[test]
    fn test_site_state_polished() {
        let report = make_report(92, vec![]);
        assert_eq!(SiteState::from_normalized(&report), SiteState::Polished);
    }

    #[test]
    fn test_site_state_critical_by_score() {
        let report = make_report(30, vec![]);
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
