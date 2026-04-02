//! Baseline / Waiver model for CI diff tracking
//!
//! Provides types and logic for storing a baseline snapshot of violations
//! and comparing future audit runs against it to identify regressions.

use serde::{Deserialize, Serialize};

use crate::wcag::types::Violation;

/// A waived/accepted violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaivedViolation {
    /// axe-core style rule ID
    pub rule_id: String,
    /// WCAG rule number (e.g., "1.1.1")
    pub rule: String,
    /// AXTree node ID (if known)
    pub node_id: Option<String>,
    /// Human-readable reason for the waiver
    pub reason: Option<String>,
    /// ISO date string when the waiver was created
    pub waived_at: String,
}

/// A stored baseline snapshot for a single URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// The URL this baseline was captured from
    pub url: String,
    /// ISO date string when the baseline was created
    pub created_at: String,
    /// The violations captured at baseline time
    pub violations: Vec<BaselineViolation>,
}

/// A single violation entry stored in the baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineViolation {
    /// WCAG rule number (e.g., "1.1.1")
    pub rule: String,
    /// axe-core style rule ID (if available)
    pub rule_id: Option<String>,
    /// Severity as string
    pub severity: String,
    /// Human-readable violation message
    pub message: String,
    /// AXTree node ID
    pub node_id: String,
}

/// Result of comparing current violations against a baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineDiff {
    /// Violations present now but not in the baseline (regressions)
    pub new_violations: Vec<Violation>,
    /// Baseline entries no longer present (fixes)
    pub resolved_violations: Vec<BaselineViolation>,
    /// Number of violations that appear in both current and baseline
    pub unchanged_count: usize,
}

impl Baseline {
    /// Create a baseline from current violations
    pub fn from_violations(url: &str, violations: &[Violation]) -> Self {
        let created_at = chrono_or_fallback();
        Self {
            url: url.to_string(),
            created_at,
            violations: violations
                .iter()
                .map(BaselineViolation::from_violation)
                .collect(),
        }
    }

    /// Compare current violations against this baseline.
    ///
    /// Matching is done by `rule` + `node_id`. A violation is "unchanged" if the
    /// same rule+node_id combo exists in the baseline.
    pub fn diff(&self, current: &[Violation]) -> BaselineDiff {
        let mut new_violations = Vec::new();
        let mut resolved_violations = Vec::new();
        let mut unchanged_count = 0usize;

        // For each current violation, check if it exists in baseline
        for violation in current {
            let in_baseline = self.violations.iter().any(|bv| {
                bv.rule == violation.rule
                    && (bv.node_id == violation.node_id
                        || bv.node_id.is_empty() && violation.node_id.is_empty())
            });
            if in_baseline {
                unchanged_count += 1;
            } else {
                new_violations.push(violation.clone());
            }
        }

        // For each baseline violation, check if it still exists in current
        for bv in &self.violations {
            let still_present = current.iter().any(|v| {
                v.rule == bv.rule
                    && (v.node_id == bv.node_id || v.node_id.is_empty() && bv.node_id.is_empty())
            });
            if !still_present {
                resolved_violations.push(bv.clone());
            }
        }

        BaselineDiff {
            new_violations,
            resolved_violations,
            unchanged_count,
        }
    }

    /// Load from a JSON file. Returns `None` if the file doesn't exist or can't be parsed.
    pub fn load(path: &std::path::Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to a JSON file.
    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl BaselineViolation {
    fn from_violation(v: &Violation) -> Self {
        Self {
            rule: v.rule.clone(),
            rule_id: v.rule_id.clone(),
            severity: format!("{:?}", v.severity),
            message: v.message.clone(),
            node_id: v.node_id.clone(),
        }
    }
}

/// Returns the current date as an ISO 8601 string.
/// Falls back to a static string if system time is unavailable.
fn chrono_or_fallback() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => {
            // Format as basic ISO date from unix timestamp (good enough for baseline keys)
            let secs = d.as_secs();
            let days = secs / 86400;
            let years_since_epoch = days / 365; // approximate
            let year = 1970 + years_since_epoch;
            let day_of_year = days % 365;
            let month = (day_of_year / 30) + 1;
            let day = (day_of_year % 30) + 1;
            format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
        }
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::WcagLevel;
    use crate::taxonomy::Severity;

    fn make_violation(rule: &str, node_id: &str) -> Violation {
        Violation::new(
            rule,
            "Test Rule",
            WcagLevel::A,
            Severity::Medium,
            "Test violation",
            node_id,
        )
    }

    #[test]
    fn test_baseline_from_violations() {
        let violations = vec![
            make_violation("1.1.1", "node-1"),
            make_violation("4.1.2", "node-2"),
        ];
        let baseline = Baseline::from_violations("https://example.com", &violations);
        assert_eq!(baseline.url, "https://example.com");
        assert_eq!(baseline.violations.len(), 2);
    }

    #[test]
    fn test_diff_new_violations() {
        let baseline_violations = vec![make_violation("1.1.1", "node-1")];
        let baseline = Baseline::from_violations("https://example.com", &baseline_violations);

        let current = vec![
            make_violation("1.1.1", "node-1"), // unchanged
            make_violation("4.1.2", "node-2"), // new
        ];
        let diff = baseline.diff(&current);
        assert_eq!(diff.unchanged_count, 1);
        assert_eq!(diff.new_violations.len(), 1);
        assert_eq!(diff.new_violations[0].rule, "4.1.2");
        assert!(diff.resolved_violations.is_empty());
    }

    #[test]
    fn test_diff_resolved_violations() {
        let baseline_violations = vec![
            make_violation("1.1.1", "node-1"),
            make_violation("4.1.2", "node-2"),
        ];
        let baseline = Baseline::from_violations("https://example.com", &baseline_violations);

        let current = vec![make_violation("1.1.1", "node-1")]; // node-2 resolved
        let diff = baseline.diff(&current);
        assert_eq!(diff.unchanged_count, 1);
        assert!(diff.new_violations.is_empty());
        assert_eq!(diff.resolved_violations.len(), 1);
        assert_eq!(diff.resolved_violations[0].rule, "4.1.2");
    }

    #[test]
    fn test_diff_all_unchanged() {
        let violations = vec![make_violation("1.1.1", "node-1")];
        let baseline = Baseline::from_violations("https://example.com", &violations);
        let diff = baseline.diff(&violations);
        assert_eq!(diff.unchanged_count, 1);
        assert!(diff.new_violations.is_empty());
        assert!(diff.resolved_violations.is_empty());
    }

    #[test]
    fn test_baseline_save_load() {
        let violations = vec![make_violation("1.1.1", "node-1")];
        let baseline = Baseline::from_violations("https://example.com", &violations);

        let path = std::env::temp_dir().join("auditmysite_test_baseline.json");
        baseline.save(&path).expect("save should succeed");
        let loaded = Baseline::load(&path).expect("load should succeed");
        assert_eq!(loaded.url, baseline.url);
        assert_eq!(loaded.violations.len(), 1);
        let _ = std::fs::remove_file(&path);
    }
}
