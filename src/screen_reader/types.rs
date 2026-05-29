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
    pub deadline: Option<String>,
    pub affected_node_ids: Vec<String>,
}
