pub use a11y_perception::{IgnoredReadingNode, ReadingItem};
use serde::{Deserialize, Serialize};

use crate::wcag::coverage::ClaimLimits;

use super::navigator::NavigationViews;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrAuditReport {
    pub schema_version: &'static str,
    pub report_type: &'static str,
    pub url: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tool_version: &'static str,
    pub summary: SrAuditSummary,
    pub reading_sequence: Vec<AnnouncedReadingItem>,
    pub navigation_views: NavigationViews,
    pub issues: Vec<SrAuditIssue>,
    pub bfsg_compliance: BfsgCompliance,
    /// AX node ids dropped from `issues` as audit exclusions (#645, #703).
    /// Kept so the PDF, which re-derives the issues in the run language,
    /// drops the same nodes.
    #[serde(skip)]
    pub excluded_node_ids: std::collections::BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrAuditSummary {
    pub total_announced_nodes: usize,
    pub tab_stops: usize,
    pub bfsg_violations: usize,
    pub name_quality_score: u32,
    pub landmark_quality_score: u32,
    pub heading_quality_score: u32,
    /// Shared meaning for the three `*_quality_score` fields.
    #[serde(default)]
    pub quality_score_context: QualityScoreContext,
    /// Whether the audited tree looks complete or is likely a consent-blocked
    /// page (very few nodes and no structural landmark). On a consent wall the
    /// quality scores and BFSG verdict reflect an audit limitation, not a real
    /// accessibility failure (#483).
    pub audit_quality: SrAuditQuality,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityScoreContext {
    pub minimum: u32,
    pub maximum: u32,
    pub higher_is_better: bool,
    pub meaning: String,
}

impl Default for QualityScoreContext {
    fn default() -> Self {
        Self {
            minimum: 0,
            maximum: 100,
            higher_is_better: true,
            meaning: "Structural quality of detected accessible names, landmarks, or headings; announced nodes and tab stops are counts, not quality scores.".to_string(),
        }
    }
}

/// Confidence qualifier for a screen-reader audit (#483).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SrAuditQuality {
    /// The audited tree looks complete.
    Ok,
    /// Too few nodes and no structural landmark — likely a consent wall blocked
    /// the audit, so findings may be incomplete.
    ConsentWallSuspected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnouncedReadingItem {
    #[serde(flatten)]
    pub item: ReadingItem,
    pub announcement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrAuditIssue {
    pub wcag_criterion: Option<String>,
    pub severity: String,
    pub affected_node_ids: Vec<String>,
    pub message: String,
    /// A heuristic that needs a reader's judgement (generic or repeated link
    /// text, heading order, landmark structure, icon-font characters) rather
    /// than a confirmed failure. Review items never count as BFSG violations
    /// and never as passed (plan 74).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub review: bool,
}

/// Compact screen-reader audit view for the unified report envelope and PDF.
///
/// Carries the actionable parts — quality scores, issues and the BFSG verdict —
/// while the full `reading_sequence` / `navigation_views` trace stays in the
/// separate sidecar JSON to keep the envelope lean (#411).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenReaderSummary {
    pub summary: SrAuditSummary,
    pub issues: Vec<SrAuditIssue>,
    pub bfsg_compliance: BfsgCompliance,
}

impl ScreenReaderSummary {
    pub fn from_report(report: &SrAuditReport) -> Self {
        Self {
            summary: report.summary.clone(),
            issues: report.issues.clone(),
            bfsg_compliance: report.bfsg_compliance.clone(),
        }
    }

    /// Number of issues at a given severity string (`"low"`/`"medium"`/`"high"`).
    pub fn count_severity(&self, severity: &str) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity.eq_ignore_ascii_case(severity))
            .count()
    }
}

/// What the screen-reader module found on the BFSG-relevant criteria it can
/// check. It reads structure and names in Chrome's accessibility tree — no
/// real screen reader, no legal assessment — so it never states conformance
/// or BFSG applicability, and says so in `limits` (#782).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BfsgCompliance {
    pub verdict: BfsgVerdict,
    pub violations: Vec<BfsgViolation>,
    /// The WCAG A/AA criteria this module checks at all: the scope every
    /// verdict is bounded by.
    pub evaluated_criteria: Vec<String>,
    /// Evaluated criteria without a violation or open heuristic. Not
    /// "passed": the module checks only some aspects of each criterion.
    pub criteria_without_findings: Vec<String>,
    /// Heuristic findings on BFSG-relevant criteria (generic link text,
    /// heading order, …): to be judged by a reader, neither violations nor
    /// passed (plan 74).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs_review: Vec<BfsgViolation>,
    #[serde(flatten)]
    pub limits: ClaimLimits,
}

/// Outcome on the evaluated criteria — never a conformance or legal verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BfsgVerdict {
    /// Nothing was audited (empty accessibility tree, or no audit ran).
    NotEvaluated,
    /// Confirmed violations on the evaluated criteria.
    ViolationsFound,
    /// No confirmed violation, but heuristic findings that need a reader.
    NeedsReview,
    /// No finding, but the tree looks incomplete (consent wall suspected,
    /// #483), so the absence of findings says nothing.
    Incomplete,
    /// No finding on the evaluated criteria. Its own state: not conformance.
    NoViolationsFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BfsgViolation {
    pub wcag_criterion: String,
    pub en_301_549_clause: Option<String>,
    pub bfsg_reference: Option<String>,
    pub fix_required: bool,
    pub affected_node_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(severity: &str) -> SrAuditIssue {
        SrAuditIssue {
            review: false,
            wcag_criterion: None,
            severity: severity.to_string(),
            affected_node_ids: vec![],
            message: "x".to_string(),
        }
    }

    #[test]
    fn count_severity_is_case_insensitive_and_per_level() {
        let summary = ScreenReaderSummary {
            summary: SrAuditSummary {
                total_announced_nodes: 0,
                tab_stops: 0,
                bfsg_violations: 0,
                name_quality_score: 100,
                landmark_quality_score: 100,
                heading_quality_score: 100,
                quality_score_context: QualityScoreContext::default(),
                audit_quality: SrAuditQuality::Ok,
            },
            issues: vec![issue("high"), issue("High"), issue("medium"), issue("low")],
            bfsg_compliance: BfsgCompliance {
                verdict: BfsgVerdict::NotEvaluated,
                violations: vec![],
                evaluated_criteria: vec![],
                criteria_without_findings: vec![],
                needs_review: Vec::new(),
                limits: ClaimLimits::with_manual_review(vec![]),
            },
        };
        assert_eq!(summary.count_severity("high"), 2);
        assert_eq!(summary.count_severity("medium"), 1);
        assert_eq!(summary.count_severity("low"), 1);
        assert_eq!(summary.count_severity("critical"), 0);
    }
}
