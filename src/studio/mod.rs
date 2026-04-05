//! Studio Contract — shared types between auditmysite and auditmysite_studio
//!
//! These types define the exact data contract for the GUI application.
//! Studio imports them directly — any field change here causes a compile
//! error in Studio, not a silent runtime failure.
//!
//! # Usage from Studio
//!
//! ```ignore
//! use auditmysite::studio::{StudioAuditResponse, StudioHistoryEntry};
//!
//! let response = StudioAuditResponse::from_normalized(&normalized, &report);
//! let history_entry = StudioHistoryEntry::from_response(&response);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::normalized::{NormalizedReport, RiskLevel};
use crate::audit::AuditReport;

// ─── Audit Response (full result sent to GUI after audit) ───────────

/// Complete audit result for the Studio GUI.
///
/// Contains everything the dashboard needs: scores, risk, module breakdown,
/// finding previews, and the full JSON report for the detail tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioAuditResponse {
    // ── Identity ────────────────────────────────────────────────────
    pub url: String,
    pub timestamp: DateTime<Utc>,

    // ── Scores ──────────────────────────────────────────────────────
    /// WCAG accessibility score (0–100)
    pub accessibility_score: u32,
    /// Weighted overall score across all active modules (0–100)
    pub overall_score: u32,
    /// Grade (A–F)
    pub grade: String,
    /// Certificate level (PLATINUM / GOLD / SILVER / BRONZE / FAILED)
    pub certificate: String,

    // ── Risk (independent from score) ───────────────────────────────
    pub risk_level: String,
    pub risk_summary: String,
    pub legal_flags: usize,
    pub blocking_issues: usize,

    // ── Severity counts ─────────────────────────────────────────────
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub total_issues: usize,

    // ── Module scores ───────────────────────────────────────────────
    pub module_scores: Vec<StudioModuleScore>,

    // ── Finding previews (compact, for list display) ────────────────
    pub findings: Vec<StudioFindingPreview>,

    // ── Metadata ────────────────────────────────────────────────────
    pub nodes_analyzed: usize,
    pub execution_time_ms: u64,

    // ── Full JSON report (for detail tab / export) ──────────────────
    pub json_report: String,
}

/// Per-module score entry for the dashboard gauge row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioModuleScore {
    pub name: String,
    pub score: u32,
    pub grade: String,
    pub weight_pct: u32,
}

/// Compact finding for the findings list — no full descriptions or code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioFindingPreview {
    pub rule_id: String,
    pub title: String,
    pub severity: String,
    pub occurrences: usize,
    pub user_impact: String,
    pub wcag_criterion: String,
}

impl StudioAuditResponse {
    /// Build a Studio response from the normalized report + raw report.
    ///
    /// The `json_report` must be pre-rendered — this function does not
    /// call the JSON formatter itself to avoid circular dependencies.
    pub fn from_normalized(
        normalized: &NormalizedReport,
        _report: &AuditReport,
        json_report: String,
    ) -> Self {
        let module_scores: Vec<StudioModuleScore> = normalized
            .module_scores
            .iter()
            .map(|m| StudioModuleScore {
                name: m.name.clone(),
                score: m.score,
                grade: m.grade.clone(),
                weight_pct: m.weight_pct,
            })
            .collect();

        let findings: Vec<StudioFindingPreview> = normalized
            .findings
            .iter()
            .map(|f| StudioFindingPreview {
                rule_id: f.rule_id.clone(),
                title: f.title.clone(),
                severity: format!("{:?}", f.severity).to_lowercase(),
                occurrences: f.occurrence_count,
                user_impact: f.user_impact.clone(),
                wcag_criterion: f.wcag_criterion.clone(),
            })
            .collect();

        Self {
            url: normalized.url.clone(),
            timestamp: normalized.timestamp,
            accessibility_score: normalized.score,
            overall_score: normalized.overall_score,
            grade: normalized.grade.clone(),
            certificate: normalized.certificate.clone(),
            risk_level: risk_level_string(normalized.risk.level),
            risk_summary: normalized.risk.summary.clone(),
            legal_flags: normalized.risk.legal_flags,
            blocking_issues: normalized.risk.blocking_issues,
            critical_issues: normalized.severity_counts.critical,
            high_issues: normalized.severity_counts.high,
            medium_issues: normalized.severity_counts.medium,
            low_issues: normalized.severity_counts.low,
            total_issues: normalized.severity_counts.total,
            module_scores,
            findings,
            nodes_analyzed: normalized.nodes_analyzed,
            execution_time_ms: normalized.duration_ms,
            json_report,
        }
    }
}

// ─── History Entry (persisted per audit, used in sidebar) ───────────

/// History entry for the sidebar list.
///
/// Lightweight subset of StudioAuditResponse — only what's needed
/// to render the history list and compare audits at a glance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioHistoryEntry {
    pub id: String,
    pub url: String,
    pub timestamp: DateTime<Utc>,
    /// WCAG accessibility score
    pub accessibility_score: u32,
    /// Weighted overall score
    pub overall_score: u32,
    pub grade: String,
    pub certificate: String,
    pub risk_level: String,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub execution_time_ms: u64,
    /// Per-module scores (compact)
    pub module_scores: Vec<StudioModuleScore>,
}

impl StudioHistoryEntry {
    /// Create a history entry from a Studio audit response.
    pub fn from_response(response: &StudioAuditResponse) -> Self {
        Self {
            id: format!("{}", response.timestamp.timestamp_millis()),
            url: response.url.clone(),
            timestamp: response.timestamp,
            accessibility_score: response.accessibility_score,
            overall_score: response.overall_score,
            grade: response.grade.clone(),
            certificate: response.certificate.clone(),
            risk_level: response.risk_level.clone(),
            total_issues: response.total_issues,
            critical_issues: response.critical_issues,
            high_issues: response.high_issues,
            execution_time_ms: response.execution_time_ms,
            module_scores: response.module_scores.clone(),
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────

fn risk_level_string(level: RiskLevel) -> String {
    match level {
        RiskLevel::Low => "low".to_string(),
        RiskLevel::Medium => "medium".to_string(),
        RiskLevel::High => "high".to_string(),
        RiskLevel::Critical => "critical".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalize;
    use crate::cli::WcagLevel;
    use crate::wcag::WcagResults;

    #[test]
    fn test_studio_response_from_empty_report() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let normalized = normalize(&report);
        let response =
            StudioAuditResponse::from_normalized(&normalized, &report, "{}".to_string());

        assert_eq!(response.url, "https://example.com");
        assert_eq!(response.accessibility_score, 100);
        assert_eq!(response.risk_level, "low");
        assert_eq!(response.total_issues, 0);
        assert!(!response.grade.is_empty());
    }

    #[test]
    fn test_history_entry_from_response() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let normalized = normalize(&report);
        let response =
            StudioAuditResponse::from_normalized(&normalized, &report, "{}".to_string());
        let entry = StudioHistoryEntry::from_response(&response);

        assert_eq!(entry.url, response.url);
        assert_eq!(entry.accessibility_score, response.accessibility_score);
        assert_eq!(entry.overall_score, response.overall_score);
        assert_eq!(entry.risk_level, response.risk_level);
    }

    #[test]
    fn test_module_scores_propagated() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let normalized = normalize(&report);
        let response =
            StudioAuditResponse::from_normalized(&normalized, &report, "{}".to_string());

        // At minimum, Accessibility module should be present
        assert!(
            response
                .module_scores
                .iter()
                .any(|m| m.name == "Accessibility"),
            "Accessibility module must be in studio response"
        );
    }

    #[test]
    fn test_studio_response_is_serializable() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            500,
        );
        let normalized = normalize(&report);
        let response =
            StudioAuditResponse::from_normalized(&normalized, &report, "{}".to_string());

        // Must serialize without error
        let json = serde_json::to_string(&response).expect("serialization must work");
        assert!(json.contains("example.com"));

        // Must deserialize back
        let parsed: StudioAuditResponse =
            serde_json::from_str(&json).expect("deserialization must work");
        assert_eq!(parsed.url, response.url);
        assert_eq!(parsed.accessibility_score, response.accessibility_score);
    }
}
