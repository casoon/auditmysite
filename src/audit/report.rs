//! Audit Report data structure
//!
//! Contains the complete results of an accessibility audit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::scoring::{AccessibilityScorer, ViolationStatistics};
use crate::mobile::MobileFriendliness;
use crate::performance::{PerformanceScore, WebVitals};
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::wcag::WcagResults;

/// Complete audit report for a single URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// The URL that was audited
    pub url: String,
    /// Timestamp when the audit was performed
    pub timestamp: DateTime<Utc>,
    /// WCAG check results
    pub wcag_results: WcagResults,
    /// Overall accessibility score (0-100)
    pub score: f32,
    /// Letter grade (A-F)
    pub grade: String,
    /// Certificate level (PLATINUM, GOLD, SILVER, BRONZE, NEEDS_IMPROVEMENT)
    pub certificate: String,
    /// Detailed violation statistics
    pub statistics: ViolationStatistics,
    /// Number of AXTree nodes analyzed
    pub nodes_analyzed: usize,
    /// Time taken to complete the audit (milliseconds)
    pub duration_ms: u64,
    /// Performance analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceResults>,
    /// SEO analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo: Option<SeoAnalysis>,
    /// Security analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityAnalysis>,
    /// Mobile friendliness analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<MobileFriendliness>,
}

/// Performance analysis results wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResults {
    /// Core Web Vitals
    pub vitals: WebVitals,
    /// Performance score
    pub score: PerformanceScore,
}

impl AuditReport {
    /// Create a new audit report
    pub fn new(url: String, wcag_results: WcagResults, duration_ms: u64) -> Self {
        let score = AccessibilityScorer::calculate_score(&wcag_results.violations);
        let grade = AccessibilityScorer::calculate_grade(score).to_string();
        let certificate = AccessibilityScorer::calculate_certificate(score).to_string();
        let statistics = AccessibilityScorer::calculate_statistics(&wcag_results.violations);
        let nodes_analyzed = wcag_results.nodes_checked;

        Self {
            url,
            timestamp: Utc::now(),
            wcag_results,
            score,
            grade,
            certificate,
            statistics,
            nodes_analyzed,
            duration_ms,
            performance: None,
            seo: None,
            security: None,
            mobile: None,
        }
    }

    /// Set performance results
    pub fn with_performance(mut self, performance: PerformanceResults) -> Self {
        self.performance = Some(performance);
        self
    }

    /// Set SEO results
    pub fn with_seo(mut self, seo: SeoAnalysis) -> Self {
        self.seo = Some(seo);
        self
    }

    /// Set security results
    pub fn with_security(mut self, security: SecurityAnalysis) -> Self {
        self.security = Some(security);
        self
    }

    /// Set mobile friendliness results
    pub fn with_mobile(mut self, mobile: MobileFriendliness) -> Self {
        self.mobile = Some(mobile);
        self
    }

    /// Get the total number of violations
    pub fn violation_count(&self) -> usize {
        self.wcag_results.violations.len()
    }

    /// Check if the audit passed (no critical violations, score >= 70)
    pub fn passed(&self) -> bool {
        self.score >= 70.0
            && !self
                .wcag_results
                .violations
                .iter()
                .any(|v| v.severity == crate::wcag::Severity::Critical)
    }

    /// Calculate overall score across all modules
    pub fn overall_score(&self) -> u32 {
        let mut total = self.score as u32;
        let mut count = 1u32;

        if let Some(ref perf) = self.performance {
            total += perf.score.overall;
            count += 1;
        }
        if let Some(ref seo) = self.seo {
            total += seo.score;
            count += 1;
        }
        if let Some(ref security) = self.security {
            total += security.score;
            count += 1;
        }
        if let Some(ref mobile) = self.mobile {
            total += mobile.score;
            count += 1;
        }

        total / count
    }
}

/// Batch audit report for multiple URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReport {
    /// Individual reports for each URL
    pub reports: Vec<AuditReport>,
    /// Summary statistics
    pub summary: BatchSummary,
    /// Total execution time
    pub total_duration_ms: u64,
}

/// Summary statistics for a batch audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Total URLs audited
    pub total_urls: usize,
    /// URLs that passed
    pub passed: usize,
    /// URLs that failed
    pub failed: usize,
    /// Average score across all URLs
    pub average_score: f64,
    /// Total violations found
    pub total_violations: usize,
}

impl BatchReport {
    /// Create a batch report from individual reports
    pub fn from_reports(reports: Vec<AuditReport>, total_duration_ms: u64) -> Self {
        let total_urls = reports.len();
        let passed = reports.iter().filter(|r| r.passed()).count();
        let failed = total_urls - passed;

        let average_score = if total_urls > 0 {
            reports.iter().map(|r| r.score as f64).sum::<f64>() / total_urls as f64
        } else {
            0.0
        };

        let total_violations = reports.iter().map(|r| r.violation_count()).sum();

        Self {
            reports,
            summary: BatchSummary {
                total_urls,
                passed,
                failed,
                average_score,
                total_violations,
            },
            total_duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::WcagResults;

    #[test]
    fn test_audit_report_new() {
        let results = WcagResults::new();
        let report = AuditReport::new("https://example.com".to_string(), results, 500);

        assert_eq!(report.url, "https://example.com");
        assert_eq!(report.score, 100.0); // No violations = perfect score
        assert_eq!(report.duration_ms, 500);
        assert!(report.performance.is_none());
        assert!(report.seo.is_none());
        assert!(report.security.is_none());
        assert!(report.mobile.is_none());
    }

    #[test]
    fn test_batch_report() {
        let reports = vec![
            AuditReport::new("https://a.com".to_string(), WcagResults::new(), 100),
            AuditReport::new("https://b.com".to_string(), WcagResults::new(), 200),
        ];

        let batch = BatchReport::from_reports(reports, 300);

        assert_eq!(batch.summary.total_urls, 2);
        assert_eq!(batch.summary.passed, 2);
        assert_eq!(batch.summary.average_score, 100.0);
    }
}
