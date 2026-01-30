//! WCAG Types - Common types for WCAG rule checking
//!
//! Defines violations, severities, WCAG levels, and rule metadata.

use serde::{Deserialize, Serialize};

use crate::cli::WcagLevel;

/// A WCAG violation found during audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// The WCAG rule that was violated (e.g., "1.1.1")
    pub rule: String,
    /// Human-readable rule name
    pub rule_name: String,
    /// WCAG conformance level (A, AA, AAA)
    pub level: WcagLevel,
    /// Severity of the violation
    pub severity: Severity,
    /// Description of the violation
    pub message: String,
    /// The AXTree node ID that has the violation
    pub node_id: String,
    /// The element's role
    pub role: Option<String>,
    /// The element's accessible name (if any)
    pub name: Option<String>,
    /// HTML selector or description for locating the element
    pub selector: Option<String>,
    /// Suggested fix for the violation
    pub fix_suggestion: Option<String>,
    /// Link to WCAG documentation
    pub help_url: Option<String>,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        rule: impl Into<String>,
        rule_name: impl Into<String>,
        level: WcagLevel,
        severity: Severity,
        message: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Self {
        Self {
            rule: rule.into(),
            rule_name: rule_name.into(),
            level,
            severity,
            message: message.into(),
            node_id: node_id.into(),
            role: None,
            name: None,
            selector: None,
            fix_suggestion: None,
            help_url: None,
        }
    }

    /// Add element role
    pub fn with_role(mut self, role: Option<String>) -> Self {
        self.role = role;
        self
    }

    /// Add element name
    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    /// Add selector
    pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
        self.selector = Some(selector.into());
        self
    }

    /// Add fix suggestion
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix_suggestion = Some(fix.into());
        self
    }

    /// Add help URL
    pub fn with_help_url(mut self, url: impl Into<String>) -> Self {
        self.help_url = Some(url.into());
        self
    }
}

/// Severity levels for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical - Blocks users completely
    Critical,
    /// Serious - Major barrier for users
    Serious,
    /// Moderate - Degraded experience
    Moderate,
    /// Minor - Small inconvenience
    Minor,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::Serious => write!(f, "serious"),
            Severity::Moderate => write!(f, "moderate"),
            Severity::Minor => write!(f, "minor"),
        }
    }
}

/// Metadata for a WCAG rule
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// WCAG success criterion (e.g., "1.1.1")
    pub id: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// WCAG conformance level
    pub level: WcagLevel,
    /// Default severity for violations
    pub severity: Severity,
    /// Brief description
    pub description: &'static str,
    /// URL to WCAG documentation
    pub help_url: &'static str,
}

/// Result of running all WCAG checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WcagResults {
    /// All violations found
    pub violations: Vec<Violation>,
    /// Number of rules that passed
    pub passes: usize,
    /// Number of elements that couldn't be checked
    pub incomplete: usize,
    /// Total nodes checked
    pub nodes_checked: usize,
}

impl WcagResults {
    /// Create new empty results
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            passes: 0,
            incomplete: 0,
            nodes_checked: 0,
        }
    }

    /// Add a violation
    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    /// Count violations by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.violations.iter().filter(|v| v.severity == severity).count()
    }

    /// Count violations by level
    pub fn count_by_level(&self, level: WcagLevel) -> usize {
        self.violations.iter().filter(|v| v.level == level).count()
    }

    /// Get all critical violations
    pub fn critical_violations(&self) -> Vec<&Violation> {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .collect()
    }

    /// Calculate an accessibility score (0-100)
    pub fn calculate_score(&self) -> u8 {
        if self.nodes_checked == 0 {
            return 100;
        }

        // Weight violations by severity
        let weighted_violations: f64 = self
            .violations
            .iter()
            .map(|v| match v.severity {
                Severity::Critical => 10.0,
                Severity::Serious => 5.0,
                Severity::Moderate => 2.0,
                Severity::Minor => 1.0,
            })
            .sum();

        // Calculate score (max penalty capped at 100)
        let penalty = (weighted_violations / self.nodes_checked as f64) * 100.0;
        let score = (100.0 - penalty).max(0.0);

        score as u8
    }

    /// Check if the page passes at a given WCAG level
    pub fn passes_level(&self, level: WcagLevel) -> bool {
        !self.violations.iter().any(|v| {
            match level {
                WcagLevel::A => v.level == WcagLevel::A,
                WcagLevel::AA => v.level == WcagLevel::A || v.level == WcagLevel::AA,
                WcagLevel::AAA => true, // AAA includes all levels
            }
        })
    }

    /// Merge results from another check
    pub fn merge(&mut self, other: WcagResults) {
        self.violations.extend(other.violations);
        self.passes += other.passes;
        self.incomplete += other.incomplete;
        self.nodes_checked += other.nodes_checked;
    }
}

impl Default for WcagResults {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_builder() {
        let violation = Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::Serious,
            "Image missing alt text",
            "node-123",
        )
        .with_role(Some("image".to_string()))
        .with_fix("Add alt attribute to the image");

        assert_eq!(violation.rule, "1.1.1");
        assert_eq!(violation.role, Some("image".to_string()));
        assert!(violation.fix_suggestion.is_some());
    }

    #[test]
    fn test_wcag_results_score() {
        let mut results = WcagResults::new();
        results.nodes_checked = 100;

        // No violations = 100 score
        assert_eq!(results.calculate_score(), 100);

        // Add some violations
        results.add_violation(Violation::new(
            "1.1.1",
            "Test",
            WcagLevel::A,
            Severity::Serious, // 5 points
            "Test",
            "1",
        ));

        // 5 / 100 * 100 = 5 penalty, 95 score
        assert_eq!(results.calculate_score(), 95);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical < Severity::Serious);
        assert!(Severity::Serious < Severity::Moderate);
        assert!(Severity::Moderate < Severity::Minor);
    }
}
