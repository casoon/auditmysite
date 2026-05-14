//! WCAG Types - Common types for WCAG rule checking
//!
//! Defines violations, severities, WCAG levels, and rule metadata.

use serde::{Deserialize, Serialize};

use crate::cli::WcagLevel;

// Re-export Severity from taxonomy module (single source of truth)
pub use crate::taxonomy::Severity;

/// Confidence level of a WCAG finding.
///
/// Distinguishes between definitive violations detected by the accessibility
/// tree and heuristic suspicions that require human verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// Automated check detected a concrete problem.
    #[default]
    Violation,
    /// Heuristic suspicion — automated tool cannot definitively confirm without
    /// behavioral testing (e.g. interactive-role without `focusable` attribute).
    Warning,
    /// Good accessibility pattern actively detected (skip link, main landmark,
    /// semantic structure). Surfaced for transparency, not as a problem.
    Positive,
    /// WCAG criterion exists but cannot be evaluated without human interaction
    /// (e.g. cognitive load, timed content, screen-reader behavior).
    NotTestable,
}

/// Machine-readable provenance for a single WCAG finding (issue #52).
///
/// `source` uses the same vocabulary as `assessment::EvidenceSource` but as a
/// plain string to avoid a circular crate dependency. Values: `"ax_tree"`,
/// `"dom_attribute"`, `"meta"`, `"css_property"`, `"http_header"`, `"computed"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationEvidence {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl ViolationEvidence {
    pub fn ax_tree(value: impl Into<String>) -> Self {
        Self {
            source: "ax_tree".to_string(),
            field: None,
            value: Some(value.into()),
        }
    }

    pub fn dom_attribute(field: impl Into<String>, value: Option<String>) -> Self {
        Self {
            source: "dom_attribute".to_string(),
            field: Some(field.into()),
            value,
        }
    }
}

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
    /// Stable axe-core-compatible rule ID (e.g., "landmark-one-main", "aria-required-attr")
    #[serde(default)]
    pub rule_id: Option<String>,
    /// Rule tags (e.g., ["wcag2a", "wcag412", "best-practice"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// Impact level in axe-core terms: "critical", "serious", "moderate", "minor"
    #[serde(default)]
    pub impact: Option<String>,
    /// Raw outer HTML of the problematic element (fetched from live DOM during enrichment)
    #[serde(default)]
    pub html_snippet: Option<String>,
    /// Concrete code fix — the corrected HTML showing how the element should look
    #[serde(default)]
    pub suggested_code: Option<String>,
    /// Confidence level: whether this is a definitive violation, a heuristic
    /// warning, a positive signal, or an untestable criterion (issue #36).
    #[serde(default)]
    pub kind: FindingKind,
    /// Machine-readable provenance for this finding (issue #52).
    /// Populated incrementally — not all violations carry evidence yet.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<ViolationEvidence>,
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
        let sev = severity;
        let impact = Some(Self::severity_to_impact(sev).to_string());
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
            rule_id: None,
            tags: Vec::new(),
            impact,
            html_snippet: None,
            suggested_code: None,
            kind: FindingKind::Violation,
            evidence: Vec::new(),
        }
    }

    /// Convert severity to axe-core impact string
    fn severity_to_impact(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "critical",
            Severity::High => "serious",
            Severity::Medium => "moderate",
            Severity::Low => "minor",
        }
    }

    /// Derive impact string from the violation's current severity
    pub fn impact_str(&self) -> &str {
        Self::severity_to_impact(self.severity)
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

    /// Set the stable axe-core-compatible rule ID
    pub fn with_rule_id(mut self, id: impl Into<String>) -> Self {
        self.rule_id = Some(id.into());
        self
    }

    /// Set rule tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set raw HTML snippet of the affected element
    pub fn with_html_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.html_snippet = Some(snippet.into());
        self
    }

    /// Set the concrete code fix
    pub fn with_suggested_code(mut self, code: impl Into<String>) -> Self {
        self.suggested_code = Some(code.into());
        self
    }

    /// Override the default finding kind (defaults to `Violation`).
    pub fn with_kind(mut self, kind: FindingKind) -> Self {
        self.kind = kind;
        self
    }

    /// Convenience: mark as a heuristic warning rather than a confirmed violation.
    pub fn with_evidence_item(mut self, ev: ViolationEvidence) -> Self {
        self.evidence.push(ev);
        self
    }

    pub fn as_warning(self) -> Self {
        self.with_kind(FindingKind::Warning)
    }

    /// Convenience: mark as a positive signal (good pattern detected).
    pub fn as_positive(self) -> Self {
        self.with_kind(FindingKind::Positive)
    }
}

// Severity enum is now defined in crate::taxonomy::severity
// and re-exported above. Old variants mapping:
// Minor → Low, Moderate → Medium, Serious → High, Critical → Critical

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
    /// Stable axe-core-compatible ID
    pub axe_id: &'static str,
    /// Tags for this rule
    pub tags: &'static [&'static str],
}

/// Result of running all WCAG checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WcagResults {
    /// Confirmed violations (kind = Violation).
    pub violations: Vec<Violation>,
    /// Heuristic suspicions that need human confirmation (kind = Warning).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Violation>,
    /// Good accessibility patterns actively detected (kind = Positive).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub positives: Vec<Violation>,
    /// Criteria that cannot be evaluated automatically — require manual testing (kind = NotTestable).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_testables: Vec<Violation>,
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
            warnings: Vec::new(),
            positives: Vec::new(),
            not_testables: Vec::new(),
            passes: 0,
            incomplete: 0,
            nodes_checked: 0,
        }
    }

    /// Add a finding. Routes to the appropriate list based on `finding.kind`.
    pub fn add_violation(&mut self, finding: Violation) {
        match finding.kind {
            FindingKind::Warning => self.warnings.push(finding),
            FindingKind::Positive => self.positives.push(finding),
            FindingKind::NotTestable => self.not_testables.push(finding),
            FindingKind::Violation => self.violations.push(finding),
        }
    }

    /// Route each finding by its kind (use instead of `.violations.extend()` for mixed vecs).
    pub fn extend_findings(&mut self, findings: Vec<Violation>) {
        for f in findings {
            self.add_violation(f);
        }
    }

    /// Add a heuristic warning directly (without `as_warning()` call on the finding).
    pub fn add_warning(&mut self, mut finding: Violation) {
        finding.kind = FindingKind::Warning;
        self.warnings.push(finding);
    }

    /// Add a positive signal directly.
    pub fn add_positive(&mut self, mut finding: Violation) {
        finding.kind = FindingKind::Positive;
        self.positives.push(finding);
    }

    /// Add a not-testable note directly.
    pub fn add_not_testable(&mut self, mut finding: Violation) {
        finding.kind = FindingKind::NotTestable;
        self.not_testables.push(finding);
    }

    /// Count violations by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .count()
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
        self.warnings.extend(other.warnings);
        self.positives.extend(other.positives);
        self.not_testables.extend(other.not_testables);
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
            Severity::High,
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
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn violation_evidence_ax_tree_constructor() {
        let ev = ViolationEvidence::ax_tree("img#logo");
        assert_eq!(ev.source, "ax_tree");
        assert_eq!(ev.value.as_deref(), Some("img#logo"));
        assert!(ev.field.is_none());
    }

    #[test]
    fn violation_evidence_dom_attribute_constructor() {
        let ev = ViolationEvidence::dom_attribute("alt", Some("".to_string()));
        assert_eq!(ev.source, "dom_attribute");
        assert_eq!(ev.field.as_deref(), Some("alt"));
        assert_eq!(ev.value.as_deref(), Some(""));
    }

    #[test]
    fn violation_evidence_dom_attribute_no_value() {
        let ev = ViolationEvidence::dom_attribute("role", None);
        assert_eq!(ev.source, "dom_attribute");
        assert!(ev.value.is_none());
    }

    #[test]
    fn violation_with_evidence_item_accumulates() {
        let v = Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            Severity::High,
            "msg",
            "n1",
        )
        .with_evidence_item(ViolationEvidence::ax_tree("img.hero"))
        .with_evidence_item(ViolationEvidence::dom_attribute("alt", None));
        assert_eq!(v.evidence.len(), 2);
        assert_eq!(v.evidence[0].source, "ax_tree");
        assert_eq!(v.evidence[1].source, "dom_attribute");
    }

    #[test]
    fn violation_new_starts_with_empty_evidence() {
        let v = Violation::new(
            "2.1.1",
            "Keyboard",
            WcagLevel::A,
            Severity::High,
            "msg",
            "n1",
        );
        assert!(v.evidence.is_empty());
    }
}
