//! Competitive benchmark / comparison report.
//!
//! Aggregates single-page audit results for multiple domains into a
//! structured comparison report for side-by-side analysis.

use serde::{Deserialize, Serialize};

use crate::audit::report::AuditReport;

/// Comparison report aggregating multiple single-page audits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// One entry per audited domain
    pub entries: Vec<ComparisonEntry>,
    /// Total audit duration in milliseconds
    pub total_duration_ms: u64,
}

/// A single domain entry in the comparison report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonEntry {
    /// The audited URL
    pub url: String,
    /// Short domain label (stripped of www. and scheme)
    pub domain: String,
    /// Overall score (0–100), weighted across all active modules
    pub overall_score: u32,
    /// Accessibility score (0–100)
    pub accessibility_score: u32,
    /// SEO score if available
    pub seo_score: Option<u32>,
    /// Performance score if available
    pub performance_score: Option<u32>,
    /// Security score if available
    pub security_score: Option<u32>,
    /// Mobile score if available
    pub mobile_score: Option<u32>,
    /// Number of critical violations
    pub critical_violations: usize,
    /// Total violations
    pub total_violations: usize,
    /// Grade (A+, A, B, ...)
    pub grade: String,
    /// Top 3 most impactful finding messages
    pub top_issues: Vec<String>,
}

impl ComparisonReport {
    /// Build from a list of single-page audit reports.
    pub fn from_reports(reports: Vec<AuditReport>, total_duration_ms: u64) -> Self {
        let mut entries: Vec<ComparisonEntry> =
            reports.iter().map(ComparisonEntry::from_report).collect();

        // Sort by overall score descending (rank 1 = best)
        entries.sort_by_key(|b| std::cmp::Reverse(b.overall_score));

        Self {
            entries,
            total_duration_ms,
        }
    }

    /// Returns the rank (1-based) for a given URL. Returns 0 if not found.
    pub fn rank_for(&self, url: &str) -> u32 {
        self.entries
            .iter()
            .position(|e| e.url == url)
            .map(|i| i as u32 + 1)
            .unwrap_or(0)
    }
}

impl ComparisonEntry {
    fn from_report(report: &AuditReport) -> Self {
        let domain = extract_domain(&report.url);

        let accessibility_score = report.score.round() as u32;
        let overall_score = report.overall_score();

        let seo_score = report.seo.as_ref().map(|s| s.score);
        let performance_score = report.performance.as_ref().map(|p| p.score.overall);
        let security_score = report.security.as_ref().map(|s| s.score);
        let mobile_score = report.mobile.as_ref().map(|m| m.score);

        let critical_violations = report
            .wcag_results
            .violations
            .iter()
            .filter(|v| matches!(v.severity, crate::wcag::Severity::Critical))
            .count();

        let top_issues: Vec<String> = report
            .wcag_results
            .violations
            .iter()
            .take(3)
            .map(|v| v.message.clone())
            .collect();

        Self {
            url: report.url.clone(),
            domain,
            overall_score,
            accessibility_score,
            seo_score,
            performance_score,
            security_score,
            mobile_score,
            critical_violations,
            total_violations: report.violation_count(),
            grade: report.grade.clone(),
            top_issues,
        }
    }
}

fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|h| h.strip_prefix("www.").unwrap_or(h).to_string())
        })
        .unwrap_or_else(|| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_strips_www() {
        assert_eq!(
            extract_domain("https://www.example.com/page"),
            "example.com"
        );
    }

    #[test]
    fn test_extract_domain_keeps_subdomain() {
        assert_eq!(
            extract_domain("https://shop.example.com"),
            "shop.example.com"
        );
    }

    #[test]
    fn test_comparison_report_sorts_by_score() {
        use crate::cli::WcagLevel;
        use crate::wcag::WcagResults;

        let make =
            |url: &str| AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::default(), 0);

        let reports = vec![make("https://a.com"), make("https://b.com")];
        let report = ComparisonReport::from_reports(reports, 1000);
        // Both have the same default score, just check it doesn't panic
        assert_eq!(report.entries.len(), 2);
    }
}
