use serde::{Deserialize, Serialize};

use super::navigator::NavigationViews;

/// A node in the order a screen reader would encounter it in the AXTree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingItem {
    pub seq: usize,
    pub role: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value: Option<String>,
    pub states: Vec<String>,
    pub tab_stop: bool,
    pub depth: usize,
    pub node_id: String,
}

/// Diagnostic entry for ignored AXNodes skipped by the standard reading order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IgnoredReadingNode {
    pub node_id: String,
    pub role: Option<String>,
    pub name: Option<String>,
    pub depth: usize,
    pub reasons: Vec<String>,
}

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SrAuditSummary {
    pub total_announced_nodes: usize,
    pub tab_stops: usize,
    pub bfsg_violations: usize,
    pub name_quality_score: u32,
    pub landmark_quality_score: u32,
    pub heading_quality_score: u32,
    /// Whether the audited tree looks complete or is likely a consent-blocked
    /// page (very few nodes and no structural landmark). On a consent wall the
    /// quality scores and BFSG verdict reflect an audit limitation, not a real
    /// accessibility failure (#483).
    pub audit_quality: SrAuditQuality,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BfsgCompliance {
    pub verdict: BfsgVerdict,
    pub violations: Vec<BfsgViolation>,
    pub passed_criteria: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BfsgVerdict {
    NotEvaluated,
    Compliant,
    NonCompliant,
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
                audit_quality: SrAuditQuality::Ok,
            },
            issues: vec![issue("high"), issue("High"), issue("medium"), issue("low")],
            bfsg_compliance: BfsgCompliance {
                verdict: BfsgVerdict::NotEvaluated,
                violations: vec![],
                passed_criteria: vec![],
            },
        };
        assert_eq!(summary.count_severity("high"), 2);
        assert_eq!(summary.count_severity("medium"), 1);
        assert_eq!(summary.count_severity("low"), 1);
        assert_eq!(summary.count_severity("critical"), 0);
    }
}
