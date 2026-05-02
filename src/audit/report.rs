//! Audit Report data structure
//!
//! Contains the complete results of an accessibility audit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::scoring::{AccessibilityScorer, ViolationStatistics};
use crate::cli::WcagLevel;
use crate::dark_mode::DarkModeAnalysis;
use crate::mobile::MobileFriendliness;
use crate::performance::{ContentWeight, PerformanceScore, RenderBlockingAnalysis, WebVitals};
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::ux::UxAnalysis;
use crate::wcag::WcagResults;

/// Screenshot bytes captured during the audit (desktop + mobile viewports).
/// Not serialized — only used for PDF output.
#[derive(Debug, Clone)]
pub struct PageScreenshots {
    pub desktop: Vec<u8>,
    pub mobile: Vec<u8>,
}

/// Complete audit report for a single URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// The URL that was audited
    pub url: String,
    /// WCAG conformance level used for the audit
    pub wcag_level: WcagLevel,
    /// Timestamp when the audit was performed
    pub timestamp: DateTime<Utc>,
    /// WCAG check results
    pub wcag_results: WcagResults,
    /// Overall accessibility score (0-100)
    pub score: f32,
    /// Letter grade (A-F)
    pub grade: String,
    /// Certificate level (SEHR GUT, GUT, SOLIDE, AUSBAUFÄHIG, UNGENÜGEND)
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
    /// Budget violations detected for this page
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub budget_violations: Vec<crate::audit::budget::BudgetViolation>,
    /// UX analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux: Option<UxAnalysis>,
    /// Journey analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journey: Option<crate::journey::JourneyAnalysis>,
    /// Dark mode support and quality analysis (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_mode: Option<DarkModeAnalysis>,
    /// Source quality analysis (Substanz / Konsistenz / Autorität)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_quality: Option<crate::source_quality::SourceQualityAnalysis>,
    /// AI visibility analysis (LLM-Readability, Citation, Chunks, Knowledge Graph, Policy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_visibility: Option<crate::ai_visibility::AiVisibilityAnalysis>,
    /// Screenshots for PDF cover page (captured during audit, not serialized).
    #[serde(skip)]
    pub page_screenshots: Option<PageScreenshots>,
}

/// Performance analysis results wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResults {
    /// Core Web Vitals
    pub vitals: WebVitals,
    /// Performance score
    pub score: PerformanceScore,
    /// Render-blocking resource analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_blocking: Option<RenderBlockingAnalysis>,
    /// Page content weight / resource breakdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_weight: Option<ContentWeight>,
}

impl AuditReport {
    /// Create a new audit report
    pub fn new(
        url: String,
        wcag_level: WcagLevel,
        wcag_results: WcagResults,
        duration_ms: u64,
    ) -> Self {
        let score = AccessibilityScorer::calculate_score(&wcag_results.violations);
        let grade = AccessibilityScorer::calculate_grade(score).to_string();
        let certificate = AccessibilityScorer::calculate_certificate(score).to_string();
        let statistics = AccessibilityScorer::calculate_statistics(&wcag_results.violations);
        let nodes_analyzed = wcag_results.nodes_checked;

        Self {
            url,
            wcag_level,
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
            ux: None,
            journey: None,
            budget_violations: Vec::new(),
            dark_mode: None,
            source_quality: None,
            ai_visibility: None,
            page_screenshots: None,
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

    /// Set UX analysis results
    pub fn with_ux(mut self, ux: UxAnalysis) -> Self {
        self.ux = Some(ux);
        self
    }

    /// Set journey analysis results
    pub fn with_journey(mut self, journey: crate::journey::JourneyAnalysis) -> Self {
        self.journey = Some(journey);
        self
    }

    /// Set dark mode analysis results
    pub fn with_dark_mode(mut self, dark_mode: DarkModeAnalysis) -> Self {
        self.dark_mode = Some(dark_mode);
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

    /// Calculate weighted overall score across all active modules
    ///
    /// Weights (normalized to active modules):
    /// - WCAG Accessibility: 40%
    /// - Performance: 20%
    /// - SEO: 20%
    /// - Security: 10%
    /// - Mobile: 10%
    pub fn overall_score(&self) -> u32 {
        let mut weighted_sum = self.score as f64 * 40.0;
        let mut total_weight = 40.0;

        if let Some(ref perf) = self.performance {
            weighted_sum += perf.score.overall as f64 * 20.0;
            total_weight += 20.0;
        }
        if let Some(ref seo) = self.seo {
            weighted_sum += seo.score as f64 * 20.0;
            total_weight += 20.0;
        }
        if let Some(ref security) = self.security {
            weighted_sum += security.score as f64 * 10.0;
            total_weight += 10.0;
        }
        if let Some(ref mobile) = self.mobile {
            weighted_sum += mobile.score as f64 * 10.0;
            total_weight += 10.0;
        }

        (weighted_sum / total_weight).round() as u32
    }
}

/// Batch audit report for multiple URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReport {
    /// Individual reports for each URL
    pub reports: Vec<AuditReport>,
    /// URLs that failed to audit (with error messages)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BatchError>,
    /// Summary statistics
    pub summary: BatchSummary,
    /// Optional crawl/link diagnostics if the batch originated from crawler discovery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crawl_diagnostics: Option<CrawlDiagnostics>,
    /// Total execution time
    pub total_duration_ms: u64,
}

/// Severity of a broken or problematic link finding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrokenLinkSeverity {
    High,
    Medium,
    Low,
}

/// A redirect chain with more than 1 hop detected during link checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectChain {
    /// Page that contains the link
    pub source_url: String,
    /// Original link target
    pub target_url: String,
    /// Final resolved URL after all redirects
    pub final_url: String,
    /// Number of redirect hops
    pub hops: u8,
    /// Whether the link points to an external domain
    pub is_external: bool,
}

/// Optional crawl/link diagnostics attached to crawler-driven batch reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlDiagnostics {
    /// Seed URL that started discovery
    pub seed_url: String,
    /// Number of discovered pages in the crawl set
    pub discovered_urls: usize,
    /// Number of unique internal links that were status-checked
    pub checked_internal_links: usize,
    /// Broken internal links (4xx/5xx or fetch failure)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub broken_internal_links: Vec<BrokenLink>,
    /// Number of unique external links that were status-checked
    pub checked_external_links: usize,
    /// Broken external links (4xx/5xx or fetch failure)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub broken_external_links: Vec<BrokenLink>,
    /// Links with more than 1 redirect hop
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub redirect_chains: Vec<RedirectChain>,
}

/// A broken internal link found during crawl-based link checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenLink {
    /// Page that contains the broken link
    pub source_url: String,
    /// Link target that failed
    pub target_url: String,
    /// HTTP status code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    /// Error reason for network/content failures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether the link points to an external domain
    pub is_external: bool,
    /// Number of redirect hops before reaching the final status (0 = direct)
    #[serde(default)]
    pub redirect_hops: u8,
    /// Severity derived from link type and status
    pub severity: BrokenLinkSeverity,
}

/// A failed URL audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// The URL that failed
    pub url: String,
    /// Error message
    pub error: String,
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
    /// Create a batch report from individual reports and errors
    pub fn from_reports(
        reports: Vec<AuditReport>,
        errors: Vec<BatchError>,
        total_duration_ms: u64,
    ) -> Self {
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
            errors,
            summary: BatchSummary {
                total_urls,
                passed,
                failed,
                average_score,
                total_violations,
            },
            crawl_diagnostics: None,
            total_duration_ms,
        }
    }

    pub fn with_crawl_diagnostics(mut self, diagnostics: CrawlDiagnostics) -> Self {
        self.crawl_diagnostics = Some(diagnostics);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::WcagResults;

    #[test]
    fn test_audit_report_new() {
        let results = WcagResults::new();
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            500,
        );

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
            AuditReport::new(
                "https://a.com".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ),
            AuditReport::new(
                "https://b.com".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                200,
            ),
        ];

        let batch = BatchReport::from_reports(reports, vec![], 300);

        assert_eq!(batch.summary.total_urls, 2);
        assert_eq!(batch.summary.passed, 2);
        assert_eq!(batch.summary.average_score, 100.0);
    }

    #[test]
    fn test_passed_with_perfect_score() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        assert!(report.passed());
    }

    #[test]
    fn test_passed_with_critical_violation() {
        let mut results = WcagResults::new();
        results.add_violation(crate::wcag::Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::wcag::Severity::Critical,
            "Missing alt",
            "node-1",
        ));
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        assert!(!report.passed());
    }

    #[test]
    fn test_overall_score_wcag_only() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        // WCAG only: overall = WCAG score
        assert_eq!(report.overall_score(), 100);
    }

    #[test]
    fn test_overall_score_weighted() {
        let mut report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        // WCAG = 100 (weight 40), Security = 50 (weight 10)
        report.security = Some(crate::security::SecurityAnalysis {
            score: 50,
            grade: "D".to_string(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
        });
        // Weighted: (100*40 + 50*10) / (40+10) = 4500/50 = 90
        assert_eq!(report.overall_score(), 90);
    }

    #[test]
    fn test_with_builder_methods() {
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        );

        let report = report.with_security(crate::security::SecurityAnalysis {
            score: 80,
            grade: "A".to_string(),
            headers: Default::default(),
            ssl: Default::default(),
            issues: vec![],
            recommendations: vec![],
        });

        assert!(report.security.is_some());
        assert_eq!(report.security.as_ref().unwrap().score, 80);
    }

    #[test]
    fn test_violation_count() {
        let mut results = WcagResults::new();
        results.add_violation(crate::wcag::Violation::new(
            "1.1.1",
            "Non-text Content",
            WcagLevel::A,
            crate::wcag::Severity::High,
            "Missing alt",
            "node-1",
        ));
        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        assert_eq!(report.violation_count(), 1);
    }
}
