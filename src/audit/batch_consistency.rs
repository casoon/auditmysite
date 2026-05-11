//! Batch consistency analysis (issues #44, #45, #46).
//!
//! Aggregates per-page signals across a `BatchReport` and reports whether
//! shared structural elements are consistent: navigation landmarks, heading
//! hierarchy starts, canonical domain variant.
//!
//! These checks complement WCAG 3.2.3 (Consistent Navigation) and 3.2.4
//! (Consistent Identification) without requiring runtime interaction.

use serde::{Deserialize, Serialize};

use crate::audit::report::{AuditReport, BatchReport};

/// Aggregated consistency analysis across all pages in a batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchConsistencyAnalysis {
    pub navigation: NavigationConsistency,
    pub headings: HeadingConsistency,
    pub canonical: CanonicalConsistency,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavigationConsistency {
    /// Pages where the MainNavigation pattern was recognized.
    pub pages_with_main_nav: usize,
    /// Pages where the SkipLink pattern was recognized.
    pub pages_with_skip_link: usize,
    pub total_pages: usize,
    /// Human-readable notes about inconsistencies.
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingConsistency {
    /// Pages with exactly one H1.
    pub pages_with_single_h1: usize,
    /// Pages with zero H1.
    pub pages_with_no_h1: usize,
    /// Pages with multiple H1s.
    pub pages_with_multiple_h1: usize,
    pub total_pages: usize,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanonicalConsistency {
    /// Number of pages canonicalizing to a www.* host.
    pub www_count: usize,
    /// Number of pages canonicalizing to a non-www host.
    pub non_www_count: usize,
    /// Number of pages with no canonical URL set.
    pub missing_count: usize,
    pub total_pages: usize,
    pub findings: Vec<String>,
}

/// Run all consistency analyses against a `BatchReport`. Returns
/// `None` when the batch has fewer than 2 pages (consistency is not
/// meaningful for a single page).
pub fn analyze(batch: &BatchReport) -> Option<BatchConsistencyAnalysis> {
    if batch.reports.len() < 2 {
        return None;
    }
    Some(BatchConsistencyAnalysis {
        navigation: analyze_navigation(&batch.reports),
        headings: analyze_headings(&batch.reports),
        canonical: analyze_canonical(&batch.reports),
    })
}

fn analyze_navigation(reports: &[AuditReport]) -> NavigationConsistency {
    let total_pages = reports.len();
    let mut pages_with_main_nav = 0;
    let mut pages_with_skip_link = 0;
    let mut missing_nav = Vec::new();
    let mut missing_skip = Vec::new();

    for r in reports {
        let recognized = r
            .patterns
            .as_ref()
            .map(|p| p.recognized.as_slice())
            .unwrap_or(&[]);
        let has_nav = recognized.iter().any(|p| p.pattern == "MainNavigation");
        let has_skip = recognized.iter().any(|p| p.pattern == "SkipLink");
        if has_nav {
            pages_with_main_nav += 1;
        } else {
            missing_nav.push(r.url.clone());
        }
        if has_skip {
            pages_with_skip_link += 1;
        } else {
            missing_skip.push(r.url.clone());
        }
    }

    let mut findings = Vec::new();
    if pages_with_main_nav > 0 && pages_with_main_nav < total_pages {
        let missing = total_pages - pages_with_main_nav;
        findings.push(format!(
            "{missing} of {total_pages} page(s) have no recognized main navigation landmark — inconsistent navigation structure across the site."
        ));
    }
    if pages_with_skip_link > 0 && pages_with_skip_link < total_pages {
        let missing = total_pages - pages_with_skip_link;
        findings.push(format!(
            "Skip link present on {pages_with_skip_link} of {total_pages} page(s); missing on {missing}. Skip links should appear on every page."
        ));
    }

    NavigationConsistency {
        pages_with_main_nav,
        pages_with_skip_link,
        total_pages,
        findings,
    }
}

fn analyze_headings(reports: &[AuditReport]) -> HeadingConsistency {
    let total_pages = reports.len();
    let mut single = 0;
    let mut none = 0;
    let mut multi = 0;
    let mut findings = Vec::new();

    for r in reports {
        let h1_count = r.seo.as_ref().map(|s| s.headings.h1_count).unwrap_or(0);
        match h1_count {
            0 => none += 1,
            1 => single += 1,
            _ => multi += 1,
        }
    }

    if none > 0 {
        findings.push(format!(
            "{none} of {total_pages} page(s) have no H1 heading. Every page should start with a single H1."
        ));
    }
    if multi > 0 {
        findings.push(format!(
            "{multi} of {total_pages} page(s) have multiple H1 headings. Use exactly one H1 per page."
        ));
    }

    HeadingConsistency {
        pages_with_single_h1: single,
        pages_with_no_h1: none,
        pages_with_multiple_h1: multi,
        total_pages,
        findings,
    }
}

fn analyze_canonical(reports: &[AuditReport]) -> CanonicalConsistency {
    let total_pages = reports.len();
    let mut www = 0;
    let mut non_www = 0;
    let mut missing = 0;
    let mut findings = Vec::new();

    for r in reports {
        let canonical = r
            .seo
            .as_ref()
            .and_then(|s| s.technical.canonical_url.as_deref());
        match canonical {
            None => missing += 1,
            Some(url) => match canonical_host(url) {
                Some(host) if host.starts_with("www.") => www += 1,
                Some(_) => non_www += 1,
                None => missing += 1,
            },
        }
    }

    if www > 0 && non_www > 0 {
        findings.push(format!(
            "Mixed canonical strategy: {www} page(s) canonicalize to www, {non_www} to non-www. Pick one variant and use it everywhere."
        ));
    }
    if missing > 0 {
        findings.push(format!(
            "{missing} of {total_pages} page(s) have no canonical URL. Set <link rel=\"canonical\"> on every page."
        ));
    }

    CanonicalConsistency {
        www_count: www,
        non_www_count: non_www,
        missing_count: missing,
        total_pages,
        findings,
    }
}

fn canonical_host(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::report::AuditReport;
    use crate::cli::WcagLevel;
    use crate::patterns::{PatternAnalysis, PatternConfidence, RecognizedPattern};
    use crate::seo::technical::TechnicalSeo;
    use crate::seo::{HeadingStructure, SeoAnalysis};
    use crate::wcag::WcagResults;

    fn make_report(
        url: &str,
        h1_count: usize,
        canonical: Option<&str>,
        recognized: Vec<&str>,
    ) -> AuditReport {
        let mut report = AuditReport::new(url.into(), WcagLevel::AA, WcagResults::new(), 100);
        let mut seo = SeoAnalysis::default();
        seo.headings = HeadingStructure {
            h1_count,
            ..Default::default()
        };
        seo.technical = TechnicalSeo {
            canonical_url: canonical.map(String::from),
            ..Default::default()
        };
        report.seo = Some(seo);
        report.patterns = Some(PatternAnalysis {
            recognized: recognized
                .into_iter()
                .map(|p| RecognizedPattern {
                    pattern: p.to_string(),
                    message: "test".to_string(),
                    confidence: PatternConfidence::Strong,
                })
                .collect(),
            violations: vec![],
        });
        report
    }

    #[test]
    fn test_consistent_pages_no_findings() {
        let reports = vec![
            make_report(
                "https://a.com/",
                1,
                Some("https://a.com/"),
                vec!["MainNavigation", "SkipLink"],
            ),
            make_report(
                "https://a.com/x",
                1,
                Some("https://a.com/x"),
                vec!["MainNavigation", "SkipLink"],
            ),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a.navigation.findings.is_empty());
        assert!(a.headings.findings.is_empty());
        assert!(a.canonical.findings.is_empty());
    }

    #[test]
    fn test_mixed_canonical_strategy_flagged() {
        let reports = vec![
            make_report("https://a.com/", 1, Some("https://www.a.com/"), vec![]),
            make_report("https://a.com/x", 1, Some("https://a.com/x"), vec![]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a
            .canonical
            .findings
            .iter()
            .any(|f| f.contains("Mixed canonical")));
    }

    #[test]
    fn test_inconsistent_navigation_flagged() {
        let reports = vec![
            make_report(
                "https://a.com/",
                1,
                None,
                vec!["MainNavigation", "SkipLink"],
            ),
            make_report("https://a.com/x", 1, None, vec!["MainNavigation"]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a
            .navigation
            .findings
            .iter()
            .any(|f| f.contains("Skip link")));
    }

    #[test]
    fn test_missing_h1_flagged() {
        let reports = vec![
            make_report("https://a.com/", 0, None, vec![]),
            make_report("https://a.com/x", 1, None, vec![]),
        ];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        let a = analyze(&batch).expect("batch ≥ 2");
        assert!(a.headings.findings.iter().any(|f| f.contains("no H1")));
    }

    #[test]
    fn test_single_page_returns_none() {
        let reports = vec![make_report("https://a.com/", 1, None, vec![])];
        let batch = BatchReport::from_reports(reports, vec![], 100);
        assert!(analyze(&batch).is_none());
    }
}
