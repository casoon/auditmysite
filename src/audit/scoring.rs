use crate::wcag::types::{Severity, Violation};

/// Calculates accessibility scores and grades based on WCAG violations
pub struct AccessibilityScorer;

impl AccessibilityScorer {
    /// Calculate accessibility score (0-100) based on violations
    ///
    /// Scoring algorithm:
    /// - Start at 100 points
    /// - Deduct 2.5 points per error
    /// - Deduct 1.0 point per warning
    /// - Additional specific penalties for critical issues
    pub fn calculate_score(violations: &[Violation]) -> f32 {
        let errors = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Critical | Severity::Serious))
            .count();
        let warnings = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Moderate))
            .count();

        let mut score = 100.0;

        // Base deductions
        score -= errors as f32 * 2.5;
        score -= warnings as f32 * 1.0;

        // Specific penalties for critical WCAG violations
        if Self::has_rule_violation(violations, "1.1.1") {
            score -= 3.0; // Images without alt text
        }
        if Self::has_rule_violation(violations, "4.1.2") {
            score -= 5.0; // Buttons/forms without labels
        }
        if Self::has_rule_violation(violations, "2.4.6") {
            score -= 20.0; // No headings (critical for navigation)
        }
        if Self::has_rule_violation(violations, "1.4.3") {
            score -= 5.0; // Contrast failures
        }
        if Self::has_rule_violation(violations, "3.1.1") {
            score -= 10.0; // Missing language attribute
        }

        score.max(0.0).min(100.0)
    }

    /// Calculate letter grade (A-F) based on score
    pub fn calculate_grade(score: f32) -> &'static str {
        match score as u32 {
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
    /// - NEEDS_IMPROVEMENT: <65% (significant issues)
    pub fn calculate_certificate(score: f32) -> &'static str {
        match score as u32 {
            95..=100 => "PLATINUM",
            85..=94 => "GOLD",
            75..=84 => "SILVER",
            65..=74 => "BRONZE",
            _ => "NEEDS_IMPROVEMENT",
        }
    }

    /// Check if violations contain a specific WCAG rule
    fn has_rule_violation(violations: &[Violation], rule_code: &str) -> bool {
        violations.iter().any(|v| v.rule == rule_code)
    }

    /// Calculate detailed statistics for a set of violations
    pub fn calculate_statistics(violations: &[Violation]) -> ViolationStatistics {
        let total = violations.len();
        let errors = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Critical | Severity::Serious))
            .count();
        let warnings = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Moderate))
            .count();
        let notices = violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Minor))
            .count();

        // Count violations by WCAG principle
        let perceivable = Self::count_by_principle(violations, "1.");
        let operable = Self::count_by_principle(violations, "2.");
        let understandable = Self::count_by_principle(violations, "3.");
        let robust = Self::count_by_principle(violations, "4.");

        ViolationStatistics {
            total,
            errors,
            warnings,
            notices,
            by_principle: PrincipleBreakdown {
                perceivable,
                operable,
                understandable,
                robust,
            },
        }
    }

    /// Count violations that belong to a specific WCAG principle
    fn count_by_principle(violations: &[Violation], prefix: &str) -> usize {
        violations
            .iter()
            .filter(|v| v.rule.starts_with(prefix))
            .count()
    }
}

/// Detailed statistics about violations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViolationStatistics {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub notices: usize,
    pub by_principle: PrincipleBreakdown,
}

/// Breakdown of violations by WCAG principle
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrincipleBreakdown {
    pub perceivable: usize,    // WCAG 1.x
    pub operable: usize,       // WCAG 2.x
    pub understandable: usize, // WCAG 3.x
    pub robust: usize,         // WCAG 4.x
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
                Severity::Serious,
                "Image missing alt",
                "img1",
            ),
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::Serious,
                "Image missing alt",
                "img2",
            ),
        ];

        // Base: 100
        // 2 errors × 2.5 = -5
        // 1.1.1 penalty = -3
        // Expected: 92
        let score = AccessibilityScorer::calculate_score(&violations);
        assert_eq!(score, 92.0);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "A");
    }

    #[test]
    fn test_score_with_warnings() {
        use crate::cli::WcagLevel;

        let violations = vec![Violation::new(
            "2.4.4",
            "Link Purpose",
            WcagLevel::A,
            Severity::Moderate,
            "Link text not descriptive",
            "link1",
        )];

        // Base: 100
        // 1 warning × 1.0 = -1
        // No special penalties for 2.4.4
        // Expected: 99
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
            Severity::Serious,
            "No headings found",
            "document",
        )];

        // Base: 100
        // 1 error × 2.5 = -2.5
        // 2.4.6 penalty = -20
        // Expected: 77.5
        let score = AccessibilityScorer::calculate_score(&violations);
        assert_eq!(score, 77.5);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "C");
        assert_eq!(AccessibilityScorer::calculate_certificate(score), "SILVER");
    }

    #[test]
    fn test_score_floor_at_zero() {
        use crate::cli::WcagLevel;

        // Create many violations to test floor
        let violations: Vec<Violation> = (0..100)
            .map(|i| {
                Violation::new(
                    "1.1.1",
                    "Non-text Content",
                    WcagLevel::A,
                    Severity::Critical,
                    "Error",
                    &format!("node{}", i),
                )
            })
            .collect();

        let score = AccessibilityScorer::calculate_score(&violations);
        assert_eq!(score, 0.0);
        assert_eq!(AccessibilityScorer::calculate_grade(score), "F");
        assert_eq!(
            AccessibilityScorer::calculate_certificate(score),
            "NEEDS_IMPROVEMENT"
        );
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
                Severity::Moderate,
                "Warning",
                "2",
            ),
            Violation::new(
                "3.1.1",
                "Language of Page",
                WcagLevel::A,
                Severity::Minor,
                "Notice",
                "3",
            ),
            Violation::new(
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::Serious,
                "Error",
                "4",
            ),
        ];

        let stats = AccessibilityScorer::calculate_statistics(&violations);
        assert_eq!(stats.total, 4);
        assert_eq!(stats.errors, 2);
        assert_eq!(stats.warnings, 1);
        assert_eq!(stats.notices, 1);
        assert_eq!(stats.by_principle.perceivable, 1); // 1.1.1
        assert_eq!(stats.by_principle.operable, 1); // 2.4.6
        assert_eq!(stats.by_principle.understandable, 1); // 3.1.1
        assert_eq!(stats.by_principle.robust, 1); // 4.1.2
    }
}
