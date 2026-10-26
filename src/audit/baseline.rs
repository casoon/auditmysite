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
    /// Stable DOM location, including frame context when present. AX node IDs
    /// are only a compatibility fallback for old baselines without selectors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
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
        let created_at = chrono::Utc::now().format("%Y-%m-%d").to_string();
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
    /// Match concrete rule, severity and stable location one-to-one. A changed
    /// rule or severity is a change, even when its WCAG criterion is unchanged.
    pub fn diff(&self, current: &[Violation]) -> BaselineDiff {
        let mut new_violations = Vec::new();
        let mut resolved_violations = Vec::new();
        let mut unchanged_count = 0usize;
        let mut matched = vec![false; self.violations.len()];

        // For each current violation, check if it exists in baseline
        for violation in current {
            let match_index = self
                .violations
                .iter()
                .enumerate()
                .position(|(i, bv)| !matched[i] && bv.matches(violation));
            if let Some(i) = match_index {
                matched[i] = true;
                unchanged_count += 1;
            } else {
                new_violations.push(violation.clone());
            }
        }

        // For each baseline violation, check if it still exists in current
        for (i, bv) in self.violations.iter().enumerate() {
            if !matched[i] {
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
    fn matches(&self, v: &Violation) -> bool {
        self.rule == v.rule
            && self.rule_id == v.rule_id
            && self.severity == format!("{:?}", v.severity)
            && match (&self.selector, &v.selector) {
                (Some(a), Some(b)) => a == b,
                (None, None) => self.node_id == v.node_id,
                _ => false,
            }
    }

    fn from_violation(v: &Violation) -> Self {
        Self {
            rule: v.rule.clone(),
            rule_id: v.rule_id.clone(),
            severity: format!("{:?}", v.severity),
            message: v.message.clone(),
            node_id: v.node_id.clone(),
            selector: v.selector.clone(),
        }
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

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("baseline.json");
        baseline.save(&path).expect("save should succeed");
        let loaded = Baseline::load(&path).expect("load should succeed");
        assert_eq!(loaded.url, baseline.url);
        assert_eq!(loaded.violations.len(), 1);
    }
}
