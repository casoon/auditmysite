use std::collections::HashMap;

use crate::taxonomy::{RuleLookup, Scaling, ScoreImpact};
use crate::wcag::types::{Severity, Violation};

/// Calculates accessibility scores and grades based on WCAG violations
pub struct AccessibilityScorer;

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

        (100.0 - total_penalty).clamp(0.0, 100.0)
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
    /// - PLATINUM: ≥95% (exemplary accessibility)
    /// - GOLD: ≥85% (excellent accessibility)
    /// - SILVER: ≥75% (good accessibility)
    /// - BRONZE: ≥65% (acceptable accessibility)
    /// - FAILED: <65% (significant issues)
    pub fn calculate_certificate(score: f32) -> &'static str {
        match score.round() as u32 {
            95..=100 => "PLATINUM",
            85..=94 => "GOLD",
            75..=84 => "SILVER",
            65..=74 => "BRONZE",
            _ => "FAILED",
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
            "PLATINUM"
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
        assert_eq!(AccessibilityScorer::calculate_certificate(score), "SILVER");
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
        assert_eq!(score, 0.0);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "F");
        assert_eq!(AccessibilityScorer::calculate_certificate(score), "FAILED");
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
