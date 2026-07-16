//! Template-level root-cause clustering for batch audits.
//!
//! When N audited pages share one broken component (header/nav/footer/shared
//! template), the same WCAG finding fires on every page with a structurally
//! identical occurrence. This module verifies that claim from evidence already
//! present in each page's occurrences (selector + raw HTML snippet) instead of
//! inferring it from a bare occurrence count, so a batch report can say "one
//! fix resolves N pages" only when that is actually demonstrated.
//!
//! Explicitly NOT caught (deliberate scope boundary, not a bug):
//! - CSS-in-JS / hashed class names that differ per page or build
//!   (`css-x8f2ka`) — normalization does not touch alphanumeric hashes, so
//!   these fall back to no cluster.
//! - The same template markup wrapped in page-specific classes that produce
//!   genuinely different selectors.
//! - Template occurrences beyond the per-finding sample cap on noisy pages —
//!   coverage can be undercounted; the 60% threshold is deliberately below
//!   "appears on every page" to absorb this.
//! - Page-level rules without a selector are grouped through a dedicated
//!   `document` key. They remain `likely` unless matching snippets provide
//!   stronger evidence.
//! - SEO-category findings (consistent with `top_recurring_rules`, which is
//!   WCAG-only).

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::audit::normalized::NormalizedReport;
use crate::audit::occurrence_analysis::normalize_selector_cluster;
use crate::taxonomy::Severity;

/// Absolute floor: a cluster needs at least this many distinct pages,
/// regardless of coverage percentage. Keeps small batches (2 pages sharing a
/// finding) from ever being reported as a "template" cause.
const MIN_DISTINCT_PAGES: usize = 3;

/// A cluster must cover at least this share of audited pages to be surfaced.
/// Deliberately below "appears on every page" to absorb undercounting from
/// the per-finding occurrence sample cap (see module docs).
const MIN_COVERAGE_PCT: f64 = 60.0;

/// A WCAG finding confirmed (or strongly suspected) to originate from one
/// shared template/component instance across multiple audited pages.
///
/// Canonical English data (#406) — this is structured evidence, not report
/// prose; the PDF presentation layer derives localized wording from
/// `confidence` at render time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateCluster {
    pub rule_id: String,
    pub title: String,
    pub wcag_criterion: String,
    pub severity: Severity,
    /// Shortest representative raw selector seen for this cluster.
    pub selector: String,
    /// `"confirmed"` when the raw HTML snippet shape matches on every member
    /// page, `"likely"` when only the normalized selector matches (occurrence
    /// could in principle be N distinct copies rather than one shared
    /// component).
    pub confidence: String,
    /// Number of distinct pages that contributed a matching occurrence.
    pub affected_pages: usize,
    /// `affected_pages / total audited pages`, as a rounded percentage.
    pub page_coverage_pct: u32,
    /// Up to 5 sample URLs demonstrating the cluster.
    pub sample_urls: Vec<String>,
    /// One representative raw HTML snippet, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_snippet: Option<String>,
    /// Coarse landmark context inferred from the selector when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Detect template-level root-cause clusters across a batch of normalized
/// reports. Pure function — no I/O, no locale dependence (all fields are
/// canonical English / structured data).
pub fn detect_template_clusters(reports: &[NormalizedReport]) -> Vec<TemplateCluster> {
    let total_pages = reports.len();
    if total_pages < MIN_DISTINCT_PAGES {
        return Vec::new();
    }

    struct Acc {
        title: String,
        wcag_criterion: String,
        severity: Severity,
        selector: String,
        pages: BTreeSet<String>,
        sample_urls: Vec<String>,
        snippet_shapes: HashSet<String>,
        has_missing_snippet: bool,
        sample_html_snippet: Option<String>,
        region: Option<String>,
    }

    let mut clusters: HashMap<(String, String), Acc> = HashMap::new();

    for report in reports {
        for finding in report.findings.iter().filter(|f| f.category == "wcag") {
            for occ in &finding.occurrences {
                let selector = occ
                    .selector
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("document");

                let normalized_selector = if selector == "document" {
                    "__pagewide__".to_string()
                } else {
                    normalize_selector_cluster(selector)
                };
                let key = (finding.rule_id.clone(), normalized_selector);
                let entry = clusters.entry(key).or_insert_with(|| Acc {
                    title: finding.title.clone(),
                    wcag_criterion: finding.wcag_criterion.clone(),
                    severity: finding.severity,
                    selector: selector.to_string(),
                    pages: BTreeSet::new(),
                    sample_urls: Vec::new(),
                    snippet_shapes: HashSet::new(),
                    has_missing_snippet: false,
                    sample_html_snippet: None,
                    region: selector_region(selector),
                });

                if selector.len() < entry.selector.len() {
                    entry.selector = selector.to_string();
                }
                if entry.pages.insert(report.url.clone()) && entry.sample_urls.len() < 5 {
                    entry.sample_urls.push(report.url.clone());
                }

                match occ.html_snippet.as_deref() {
                    Some(snippet) if !snippet.trim().is_empty() => {
                        entry.snippet_shapes.insert(snippet_shape_hash(snippet));
                        if entry.sample_html_snippet.is_none() {
                            entry.sample_html_snippet = Some(snippet.to_string());
                        }
                    }
                    _ => entry.has_missing_snippet = true,
                }
            }
        }
    }

    let mut result: Vec<TemplateCluster> = clusters
        .into_iter()
        .filter_map(|((rule_id, _normalized_selector), acc)| {
            let affected_pages = acc.pages.len();
            if affected_pages < MIN_DISTINCT_PAGES {
                return None;
            }
            let page_coverage = (affected_pages as f64 / total_pages as f64) * 100.0;
            if page_coverage < MIN_COVERAGE_PCT {
                return None;
            }

            let snippet_confirmed = !acc.has_missing_snippet && acc.snippet_shapes.len() == 1;
            // Bare-tag selectors (`img`, `a`, `button`, …) are too generic to
            // trust on their own — the same rule can coincidentally fire on
            // unrelated elements across pages. Only cluster them when the raw
            // HTML snippet shape also agrees across every member page.
            if acc.selector != "document"
                && !is_specific_selector(&acc.selector)
                && !snippet_confirmed
            {
                return None;
            }

            Some(TemplateCluster {
                rule_id,
                title: acc.title,
                wcag_criterion: acc.wcag_criterion,
                severity: acc.severity,
                selector: acc.selector,
                confidence: if snippet_confirmed {
                    "confirmed".to_string()
                } else {
                    "likely".to_string()
                },
                affected_pages,
                page_coverage_pct: page_coverage.round() as u32,
                sample_urls: acc.sample_urls,
                html_snippet: acc.sample_html_snippet,
                region: acc.region,
            })
        })
        .collect();

    result.sort_by(|a, b| {
        b.affected_pages
            .cmp(&a.affected_pages)
            .then_with(|| a.rule_id.cmp(&b.rule_id))
    });
    result
}

fn selector_region(selector: &str) -> Option<String> {
    let selector = selector.to_ascii_lowercase();
    ["header", "nav", "main", "footer"]
        .into_iter()
        .find(|region| selector.contains(region))
        .map(str::to_string)
}

/// A selector is "specific" enough to trust on its own (without a snippet
/// match) when it names an id, a class, or a combinator/descendant
/// relationship — i.e. it is more than a bare tag name.
fn is_specific_selector(selector: &str) -> bool {
    selector.contains('#')
        || selector.contains('.')
        || selector.contains('>')
        || selector.contains('+')
        || selector.contains('~')
        || selector.trim().contains(' ')
}

/// Reduce a raw HTML snippet to a structural "shape": lowercase, digits
/// stripped, whitespace collapsed, and long attribute values masked (kills
/// per-page URLs/hashes in `href`/`srcset`/`class` so the same component
/// rendered with different link targets or CMS ids still hashes identically).
fn snippet_shape_hash(html: &str) -> String {
    // Pass 1: mask attribute values longer than ~24 chars.
    let mut masked = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' || ch == '\'' {
            let quote = ch;
            let mut value = String::new();
            let mut closed = false;
            for inner in chars.by_ref() {
                if inner == quote {
                    closed = true;
                    break;
                }
                value.push(inner);
            }
            masked.push(quote);
            if value.chars().count() > 24 {
                masked.push('…');
            } else {
                masked.push_str(&value);
            }
            if closed {
                masked.push(quote);
            }
            continue;
        }
        masked.push(ch);
    }

    // Pass 2: lowercase, strip digits, collapse whitespace.
    let mut shape = String::with_capacity(masked.len());
    let mut last_was_space = false;
    for ch in masked.chars() {
        if ch.is_ascii_digit() {
            continue;
        }
        if ch.is_whitespace() {
            if !last_was_space {
                shape.push(' ');
            }
            last_was_space = true;
        } else {
            shape.push(ch.to_ascii_lowercase());
            last_was_space = false;
        }
    }
    shape.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::normalized::{
        ComplexityKind, ExpectedImpactKind, NormalizedFinding, OccurrenceDetail,
        ReportVisibilityData, ScoreEffect, ScoreImpactData, SeverityCounts,
    };
    use crate::WcagLevel;

    fn make_occurrence(
        node_id: &str,
        selector: &str,
        html_snippet: Option<&str>,
    ) -> OccurrenceDetail {
        OccurrenceDetail {
            node_id: node_id.to_string(),
            message: "test occurrence".to_string(),
            selector: Some(selector.to_string()),
            fix_suggestion: None,
            html_snippet: html_snippet.map(str::to_string),
            suggested_code: None,
            tags: Vec::new(),
            ..Default::default()
        }
    }

    fn make_finding(rule_id: &str, occurrences: Vec<OccurrenceDetail>) -> NormalizedFinding {
        let count = occurrences.len();
        NormalizedFinding {
            category: "wcag".into(),
            rule_id: rule_id.into(),
            wcag_criterion: "2.5.3".into(),
            axe_id: None,
            wcag_level: "A".into(),
            dimension: "Accessibility".into(),
            subcategory: "Forms Interaction".into(),
            issue_class: "Invalid".into(),
            dimension_kind: crate::taxonomy::Dimension::Accessibility,
            subcategory_kind: crate::taxonomy::Subcategory::FormsInteraction,
            issue_class_kind: crate::taxonomy::IssueClass::Invalid,
            severity: Severity::High,
            user_impact: String::new(),
            technical_impact: String::new(),
            score_impact: ScoreImpactData {
                base_penalty: 5.0,
                max_penalty: 20.0,
                scaling: "Logarithmic".into(),
            },
            report_visibility: ReportVisibilityData::default(),
            aggregation_key: rule_id.into(),
            title: "Label in Name mismatch".into(),
            description: String::new(),
            help_url: None,
            occurrence_count: count,
            priority_score: 1.0,
            confidence: "very_high".into(),
            false_positive_risk: "very_low".into(),
            verification: "automatically_confirmed".into(),
            complexity: "low".into(),
            complexity_reason: "Test fixture".into(),
            complexity_kind: ComplexityKind::LowScope,
            expected_impact: "Test fixture".into(),
            expected_impact_kind: ExpectedImpactKind::Wcag {
                occurrence_count: count,
                score_effect: ScoreEffect::Low,
                wcag_level: "A".into(),
            },
            bfsg_relevance: "medium".into(),
            remediation_priority: "normal".into(),
            occurrences,
        }
    }

    fn make_report(url: &str, findings: Vec<NormalizedFinding>) -> NormalizedReport {
        let total = findings.len();
        NormalizedReport {
            url: url.to_string(),
            wcag_level: WcagLevel::AA,
            timestamp: chrono::Utc::now(),
            duration_ms: 0,
            nodes_analyzed: 100,
            score: 80,
            grade: "B".into(),
            certificate: "None".into(),
            overall_score: 80,
            findings,
            severity_counts: SeverityCounts {
                critical: 0,
                high: total,
                medium: 0,
                low: 0,
                total,
            },
            occurrence_counts: SeverityCounts {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                total: 0,
            },
            accessibility_assessments: vec![],
            rule_outcomes: vec![],
            execution: Default::default(),
            module_scores: vec![],
            audit_flags: vec![],
            consent_privacy: None,
            has_screenshots: false,
            viewport_scores: None,
            score_calculation_method: "module_weighted".to_string(),
            score_breakdown: None,
            interactive_findings: Vec::new(),
            accessibility_journey: None,
            screen_reader: None,
            interpretation: None,
            risk: crate::audit::normalized::RiskAssessment {
                level: crate::audit::normalized::RiskLevel::Low,
                score: 0,
                threshold: 0,
                driven_by: String::new(),
                critical_issues: 0,
                high_issues: 0,
                legal_flags: 0,
                blocking_issues: 0,
                interactive_critical_issues: 0,
                interactive_high_issues: 0,
                summary: String::new(),
            },
            principle_coverage: crate::audit::scoring::AccessibilityScorer::calculate_coverage(&[]),
        }
    }

    #[test]
    fn identical_nav_selector_and_snippet_on_all_pages_is_a_confirmed_cluster() {
        let snippet = r#"<button aria-label="Menu">Open menu</button>"#;
        let reports: Vec<NormalizedReport> = (0..5)
            .map(|i| {
                let url = format!("https://example.com/page-{i}");
                let occ = make_occurrence("node-1", "nav > button.menu-toggle", Some(snippet));
                make_report(&url, vec![make_finding("a11y.label_in_name", vec![occ])])
            })
            .collect();

        let clusters = detect_template_clusters(&reports);
        assert_eq!(
            clusters.len(),
            1,
            "expected exactly one cluster: {clusters:?}"
        );
        let cluster = &clusters[0];
        assert_eq!(cluster.affected_pages, 5);
        assert_eq!(cluster.confidence, "confirmed");
        assert_eq!(cluster.page_coverage_pct, 100);
    }

    #[test]
    fn bare_tag_selector_with_different_snippets_does_not_cluster() {
        // Distinct per-page team photos: different tag content (not merely
        // digits, which the shape hash intentionally neutralizes), so the
        // snippet shape genuinely diverges and the bare `img` selector must
        // not be trusted on its own.
        let names = ["alice", "bob", "carla", "deepak", "elin"];
        let reports: Vec<NormalizedReport> = names
            .iter()
            .map(|name| {
                let url = format!("https://example.com/team/{name}");
                let snippet = format!(r#"<img src="/team/{name}.jpg" alt="Photo of {name}">"#);
                let occ = make_occurrence("node-1", "img", Some(&snippet));
                make_report(&url, vec![make_finding("a11y.alt_text", vec![occ])])
            })
            .collect();

        let clusters = detect_template_clusters(&reports);
        assert!(
            clusters.is_empty(),
            "bare tag selector with divergent snippet shapes must not cluster: {clusters:?}"
        );
    }

    #[test]
    fn numeric_cms_ids_cluster_via_digit_normalization() {
        let reports: Vec<NormalizedReport> = ["#post-123", "#post-456", "#post-789"]
            .iter()
            .enumerate()
            .map(|(i, selector)| {
                let url = format!("https://example.com/post-{i}");
                let occ = make_occurrence("node-1", selector, None);
                make_report(&url, vec![make_finding("a11y.heading_order", vec![occ])])
            })
            .collect();

        let clusters = detect_template_clusters(&reports);
        assert_eq!(
            clusters.len(),
            1,
            "numeric CMS ids should cluster via digit normalization: {clusters:?}"
        );
        assert_eq!(clusters[0].affected_pages, 3);
    }

    #[test]
    fn two_page_batch_never_clusters() {
        let snippet = r#"<button aria-label="Menu">Open menu</button>"#;
        let reports: Vec<NormalizedReport> = (0..2)
            .map(|i| {
                let url = format!("https://example.com/page-{i}");
                let occ = make_occurrence("node-1", "nav > button.menu-toggle", Some(snippet));
                make_report(&url, vec![make_finding("a11y.label_in_name", vec![occ])])
            })
            .collect();

        let clusters = detect_template_clusters(&reports);
        assert!(
            clusters.is_empty(),
            "a 2-page batch must never produce a template cluster: {clusters:?}"
        );
    }

    #[test]
    fn exactly_sixty_percent_coverage_clusters() {
        let reports: Vec<NormalizedReport> = (0..5)
            .map(|i| {
                let findings = if i < 3 {
                    vec![make_finding(
                        "a11y.skip_link",
                        vec![make_occurrence("node-1", "header .skip-link", None)],
                    )]
                } else {
                    vec![]
                };
                make_report(&format!("https://example.com/page-{i}"), findings)
            })
            .collect();

        let clusters = detect_template_clusters(&reports);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].page_coverage_pct, 60);
        assert_eq!(clusters[0].region.as_deref(), Some("header"));
        assert_eq!(clusters[0].confidence, "likely");
    }

    #[test]
    fn heterogeneous_site_below_sixty_percent_does_not_cluster() {
        let reports: Vec<NormalizedReport> = (0..10)
            .map(|i| {
                let findings = if i < 5 {
                    vec![make_finding(
                        "a11y.navigation_name",
                        vec![make_occurrence("node-1", "nav.primary", None)],
                    )]
                } else {
                    vec![]
                };
                make_report(&format!("https://example.com/page-{i}"), findings)
            })
            .collect();

        assert!(detect_template_clusters(&reports).is_empty());
    }

    #[test]
    fn selectorless_pagewide_findings_use_document_cluster() {
        let reports: Vec<NormalizedReport> = (0..3)
            .map(|i| {
                let occurrence = OccurrenceDetail {
                    selector: None,
                    ..make_occurrence("document", "", None)
                };
                make_report(
                    &format!("https://example.com/page-{i}"),
                    vec![make_finding("a11y.document_language", vec![occurrence])],
                )
            })
            .collect();

        let clusters = detect_template_clusters(&reports);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].selector, "document");
        assert_eq!(clusters[0].confidence, "likely");
    }
}
