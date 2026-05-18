use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::taxonomy::{
    criterion_for_rule, principle_for_criterion, RuleLookup, Scaling, ScoreImpact, WcagPrinciple,
};
use crate::wcag::coverage::AUTOMATED_CRITERIA;
use crate::wcag::types::{Severity, Violation};

/// Number of automatically-checked WCAG criteria belonging to a principle.
fn checked_criteria_count(principle: WcagPrinciple) -> usize {
    AUTOMATED_CRITERIA
        .iter()
        .filter(|(c, _)| principle_for_criterion(c) == Some(principle))
        .count()
}

/// Calculates accessibility scores and grades based on WCAG violations
pub struct AccessibilityScorer;

/// Below this raw score the hard `clamp(0)` is replaced by an asymptotic
/// curve, so that semantically different catastrophic states (score 0 vs 5)
/// remain distinguishable instead of collapsing onto a single value.
const SOFT_FLOOR_START: f32 = 15.0;

/// Diversity factor for `unique_criteria` violated WCAG success criteria.
///
/// A focused failure (≤3 criteria) gets no surcharge; broad, systemic
/// failure is penalised, with logarithmic — bounded — growth.
fn diversity_factor(unique_criteria: usize) -> f32 {
    if unique_criteria <= 3 {
        1.0
    } else {
        (1.0 + 0.15 * (1.0 + (unique_criteria as f32 / 5.0).ln())).min(1.5)
    }
}

/// Soft floor: above `SOFT_FLOOR_START` the raw score passes through
/// unchanged; below it, an exponential curve approaches zero asymptotically.
fn apply_soft_floor(raw_score: f32) -> f32 {
    if raw_score >= SOFT_FLOOR_START {
        raw_score
    } else {
        SOFT_FLOOR_START * ((raw_score - SOFT_FLOOR_START) / 8.0).exp()
    }
}

/// Default penalty for rules not found in the taxonomy registry
fn default_impact(severity: Severity) -> ScoreImpact {
    match severity {
        Severity::Critical => ScoreImpact {
            base_penalty: 5.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        Severity::High => ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        Severity::Medium => ScoreImpact {
            base_penalty: 1.5,
            max_penalty: 5.0,
            occurrence_scaling: Scaling::Logarithmic,
        },
        Severity::Low => ScoreImpact {
            base_penalty: 0.5,
            max_penalty: 2.0,
            occurrence_scaling: Scaling::Fixed,
        },
    }
}

impl AccessibilityScorer {
    /// Calculate accessibility score (0-100) based on violations
    ///
    /// Uses the taxonomy rule registry for per-rule score impacts.
    /// Groups violations by rule, looks up ScoreImpact, and applies
    /// occurrence-based penalty scaling.
    pub fn calculate_score(violations: &[Violation]) -> f32 {
        if violations.is_empty() {
            return 100.0;
        }

        // Group violations by rule ID and track severity
        let mut rule_counts: HashMap<&str, (usize, Severity)> = HashMap::new();
        for v in violations {
            let entry = rule_counts.entry(&v.rule).or_insert((0, v.severity));
            entry.0 += 1;
            // Keep highest severity if same rule has mixed severities
            if v.severity > entry.1 {
                entry.1 = v.severity;
            }
        }

        let mut total_penalty = 0.0f32;

        for (rule_id, (count, severity)) in &rule_counts {
            // Look up rule in taxonomy (by legacy WCAG ID)
            let impact = RuleLookup::by_legacy_wcag_id(rule_id)
                .map(|r| r.score_impact)
                .unwrap_or_else(|| default_impact(*severity));

            total_penalty += impact.calculate_penalty(*count);
        }

        // Diversity factor: failing many distinct WCAG criteria is a broader,
        // more systemic problem than failing one criterion repeatedly.
        let unique_criteria: HashSet<String> = rule_counts
            .keys()
            .map(|rule_id| criterion_for_rule(rule_id).unwrap_or_else(|| (*rule_id).to_string()))
            .collect();
        total_penalty *= diversity_factor(unique_criteria.len());

        // Soft floor instead of a hard clamp(0): keeps the 1–15 band usable
        // for distinguishing degrees of catastrophic failure.
        let raw_score = apply_soft_floor(100.0 - total_penalty).min(100.0);

        // Semantic score cap: a site with open critical/high issues cannot be
        // reported as near-perfect even if penalties are individually small.
        let critical_count = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Critical))
            .count();
        let urgent_count = critical_count
            + violations
                .iter()
                .filter(|v| matches!(v.severity, Severity::High))
                .count();

        let score_cap: f32 = if critical_count >= 3 {
            94.0
        } else if critical_count >= 1 {
            96.0
        } else if urgent_count >= 5 {
            92.0
        } else {
            100.0
        };

        raw_score.min(score_cap)
    }

    /// Calculate letter grade (A-F) based on score
    pub fn calculate_grade(score: f32) -> &'static str {
        match score.round() as u32 {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        }
    }

    /// Calculate certificate level based on score
    ///
    /// Certificate levels:
    /// - SEHR GUT: ≥95% (exemplary accessibility)
    /// - GUT: ≥85% (high accessibility)
    /// - SOLIDE: ≥75% (good accessibility)
    /// - AUSBAUFÄHIG: ≥65% (acceptable accessibility)
    /// - UNGENÜGEND: <65% (significant issues)
    pub fn calculate_certificate(score: f32) -> &'static str {
        match score.round() as u32 {
            95..=100 => "SEHR GUT",
            85..=94 => "GUT",
            75..=84 => "SOLIDE",
            65..=74 => "AUSBAUFÄHIG",
            _ => "UNGENÜGEND",
        }
    }

    /// Calculate WCAG principle coverage — a purely informative secondary
    /// indicator that does *not* feed into the numeric score.
    ///
    /// For each of the four WCAG principles it reports how many of the
    /// criteria the tool checks passed (had no violation).
    pub fn calculate_coverage(violations: &[Violation]) -> PrincipleCoverage {
        let failed: HashSet<String> = violations
            .iter()
            .filter_map(|v| criterion_for_rule(&v.rule))
            .collect();

        let ratio_for = |principle: WcagPrinciple| -> CoverageRatio {
            let total = checked_criteria_count(principle) as u32;
            let failed_in_principle = failed
                .iter()
                .filter(|c| principle_for_criterion(c) == Some(principle))
                .count() as u32;
            let passed = total.saturating_sub(failed_in_principle);
            CoverageRatio {
                passed,
                total,
                ratio: if total == 0 {
                    1.0
                } else {
                    passed as f32 / total as f32
                },
            }
        };

        PrincipleCoverage {
            perceivable: ratio_for(WcagPrinciple::Perceivable),
            operable: ratio_for(WcagPrinciple::Operable),
            understandable: ratio_for(WcagPrinciple::Understandable),
            robust: ratio_for(WcagPrinciple::Robust),
        }
    }

    /// Calculate detailed statistics for a set of violations
    pub fn calculate_statistics(violations: &[Violation]) -> ViolationStatistics {
        let total = violations.len();
        let critical = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Critical))
            .count();
        let high = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::High))
            .count();
        let medium = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Medium))
            .count();
        let low = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Low))
            .count();

        ViolationStatistics {
            total,
            critical,
            high,
            medium,
            low,
        }
    }
}

/// Pass/total ratio of checked WCAG criteria for one principle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageRatio {
    pub passed: u32,
    pub total: u32,
    /// 0.0 – 1.0
    pub ratio: f32,
}

/// WCAG principle coverage — informative secondary indicator (#99).
///
/// Shows *where* a site fails: a deep failure in one area versus a broad
/// failure across all four principles. Does not affect the numeric score.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrincipleCoverage {
    pub perceivable: CoverageRatio,
    pub operable: CoverageRatio,
    pub understandable: CoverageRatio,
    pub robust: CoverageRatio,
}

/// Detailed statistics about violations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationStatistics {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_score() {
        let violations = vec![];
        let score = AccessibilityScorer::calculate_score(&violations);
        assert_eq!(score, 100.0);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "A");
        assert_eq!(
            AccessibilityScorer::calculate_certificate(score),
            "SEHR GUT"
        );
    }

    #[test]
    fn test_score_with_errors() {
        use crate::cli::WcagLevel;

        let violations = vec![
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "Image missing alt",
                "img1",
            ),
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "Image missing alt",
                "img2",
            ),
        ];

        // Rule 1.1.1: base=3.0, max=10.0, Logarithmic
        // 2 occurrences: 3.0 * (1 + ln(2)) ≈ 5.08
        let score = AccessibilityScorer::calculate_score(&violations);
        assert!(score > 94.0 && score < 96.0, "Score was {}", score);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "A");
    }

    #[test]
    fn test_score_with_warnings() {
        use crate::cli::WcagLevel;

        let violations = vec![Violation::new(
            "2.4.4",
            "Link Purpose",
            WcagLevel::A,
            Severity::Medium,
            "Link text not descriptive",
            "link1",
        )];

        // Rule 2.4.4: base=1.0, max=5.0, Logarithmic
        // 1 occurrence: 1.0 * (1 + ln(1)) = 1.0
        let score = AccessibilityScorer::calculate_score(&violations);
        assert_eq!(score, 99.0);
    }

    #[test]
    fn test_critical_penalty_no_headings() {
        use crate::cli::WcagLevel;

        let violations = vec![Violation::new(
            "2.4.6",
            "Headings and Labels",
            WcagLevel::AA,
            Severity::High,
            "No headings found",
            "document",
        )];

        // Rule 2.4.6: base=20.0, max=20.0, Fixed
        // 1 occurrence: 20.0
        let score = AccessibilityScorer::calculate_score(&violations);
        assert_eq!(score, 80.0);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "B");
        assert_eq!(AccessibilityScorer::calculate_certificate(score), "SOLIDE");
    }

    #[test]
    fn test_score_floor_at_zero() {
        use crate::cli::WcagLevel;

        // Create many violations across ALL mapped rules to exceed 100 penalty
        let rules = [
            ("1.1.1", "Non-text Content"),
            ("1.3.1", "Info and Relationships"),
            ("1.4.3", "Contrast"),
            ("2.1.1", "Keyboard"),
            ("2.4.1", "Bypass Blocks"),
            ("2.4.2", "Page Titled"),
            ("2.4.6", "Headings"),
            ("2.4.7", "Focus Visible"),
            ("3.1.1", "Language"),
            ("3.3.2", "Labels"),
            ("4.1.2", "Name Role Value"),
            ("2.4.4", "Link Purpose"),
            ("2.4.3", "Focus Order"),
            ("1.4.4", "Resize Text"),
        ];
        let mut violations = Vec::new();
        for (rule, name) in &rules {
            for i in 0..20 {
                violations.push(Violation::new(
                    *rule,
                    *name,
                    WcagLevel::A,
                    Severity::Critical,
                    "Error",
                    format!("n{}", i),
                ));
            }
        }

        let score = AccessibilityScorer::calculate_score(&violations);
        // Soft floor: catastrophic failure approaches 0 asymptotically rather
        // than hard-clamping, but stays well below 2.0.
        assert!(score < 2.0, "score was {}", score);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "F");
        assert_eq!(
            AccessibilityScorer::calculate_certificate(score),
            "UNGENÜGEND"
        );
    }

    #[test]
    fn test_diversity_penalises_breadth() {
        use crate::cli::WcagLevel;

        // 5× the same High violation — a focused failure.
        let same: Vec<Violation> = (0..5)
            .map(|i| {
                Violation::new(
                    "1.1.1",
                    "Non-text Content",
                    WcagLevel::A,
                    Severity::High,
                    "Image missing alt",
                    format!("n{}", i),
                )
            })
            .collect();

        // 5 distinct High violations — a broader failure.
        let diverse_rules = [
            ("1.1.1", "Non-text Content"),
            ("1.3.1", "Info and Relationships"),
            ("2.4.3", "Focus Order"),
            ("3.3.2", "Labels"),
            ("1.4.4", "Resize Text"),
        ];
        let diverse: Vec<Violation> = diverse_rules
            .iter()
            .map(|(rule, name)| {
                Violation::new(*rule, *name, WcagLevel::A, Severity::High, "Error", "node")
            })
            .collect();

        let score_same = AccessibilityScorer::calculate_score(&same);
        let score_diverse = AccessibilityScorer::calculate_score(&diverse);
        assert!(
            score_diverse < score_same,
            "diverse {} should score lower than same {}",
            score_diverse,
            score_same
        );
    }

    #[test]
    fn test_diversity_factor_growth() {
        // ≤3 criteria: no surcharge.
        assert_eq!(diversity_factor(1), 1.0);
        assert_eq!(diversity_factor(3), 1.0);
        // Logarithmic, bounded growth.
        assert!(diversity_factor(5) > 1.0);
        assert!(diversity_factor(20) > diversity_factor(5));
        assert!(diversity_factor(40) <= 1.5);
    }

    #[test]
    fn test_soft_floor_curve() {
        // Above the floor start: unchanged.
        assert_eq!(apply_soft_floor(20.0), 20.0);
        assert_eq!(apply_soft_floor(50.0), 50.0);
        // Below: approaches 0 but never hard-clamps to it.
        assert!(apply_soft_floor(0.0) > 0.0);
        assert!(apply_soft_floor(-30.0) > 0.0);
        assert!(apply_soft_floor(-30.0) < apply_soft_floor(0.0));
        // Catastrophic penalties drop below 1.0.
        assert!(apply_soft_floor(-60.0) < 1.0);
    }

    #[test]
    fn test_coverage_all_pass_when_no_violations() {
        let coverage = AccessibilityScorer::calculate_coverage(&[]);
        for ratio in [
            &coverage.perceivable,
            &coverage.operable,
            &coverage.understandable,
            &coverage.robust,
        ] {
            assert_eq!(ratio.passed, ratio.total);
            assert!(ratio.total > 0);
            assert_eq!(ratio.ratio, 1.0);
        }
    }

    #[test]
    fn test_coverage_reflects_violations() {
        use crate::cli::WcagLevel;

        let violations = vec![Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "Missing alt",
            "n1",
        )];
        let coverage = AccessibilityScorer::calculate_coverage(&violations);
        // One Perceivable criterion failed.
        assert_eq!(coverage.perceivable.passed, coverage.perceivable.total - 1);
        // Other principles untouched.
        assert_eq!(coverage.operable.passed, coverage.operable.total);
        assert_eq!(coverage.robust.passed, coverage.robust.total);
    }

    #[test]
    fn test_statistics_calculation() {
        use crate::cli::WcagLevel;

        let violations = vec![
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::Critical,
                "Error",
                "1",
            ),
            Violation::new(
                "2.4.6",
                "Headings and Labels",
                WcagLevel::AA,
                Severity::Medium,
                "Warning",
                "2",
            ),
            Violation::new(
                "3.1.1",
                "Language of Page",
                WcagLevel::A,
                Severity::Low,
                "Notice",
                "3",
            ),
            Violation::new(
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::High,
                "Error",
                "4",
            ),
        ];

        let stats = AccessibilityScorer::calculate_statistics(&violations);
        assert_eq!(stats.total, 4);
        assert_eq!(stats.critical, 1);
        assert_eq!(stats.high, 1);
        assert_eq!(stats.medium, 1);
        assert_eq!(stats.low, 1);
    }
}
