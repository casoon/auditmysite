//! Root causes for batch audits: finding patterns that recur across pages.
//!
//! When N audited pages share one broken component (header/nav/footer/shared
//! template), the same rule fires on every page at the same element pattern.
//! [`detect_root_causes`] groups those patterns by their id
//! (`a11y_report::RootCauseId`, the `findings[].patterns[].id`) and grades the
//! evidence from what each page's occurrences show (selector + raw HTML
//! snippet), so a batch report says "one fix resolves N pages" only as far as
//! the evidence goes.
//!
//! Explicitly NOT caught (deliberate scope boundary, not a bug):
//! - CSS-in-JS / hashed class names that differ per page or build
//!   (`css-x8f2ka`) — normalization does not touch alphanumeric hashes, so
//!   these stay separate patterns.
//! - The same template markup wrapped in page-specific classes that produce
//!   genuinely different selectors.
//! - Markup evidence beyond the per-finding occurrence sample: a page whose
//!   sample holds no snippet for the pattern keeps the cause at `likely`.
//! - Page-level rules without a selector form a `document` pattern. They stay
//!   `likely` unless matching snippets provide stronger evidence.
//! - SEO-category findings.

use std::collections::{HashMap, HashSet};

use crate::audit::normalized::NormalizedReport;
use crate::taxonomy::Severity;

/// Absolute floor for `likely`/`confirmed`: at least this many distinct pages,
/// regardless of coverage. A pattern on fewer pages stays `open` — two pages
/// sharing a finding are no template claim.
const MIN_DISTINCT_PAGES: usize = 3;

/// Coverage floor for `likely`/`confirmed`, as a share of audited pages;
/// below it the pattern stays `open`. Deliberately below "appears on every
/// page" to absorb undercounting from the per-finding occurrence sample cap.
const MIN_COVERAGE_PCT: f64 = 60.0;

/// A rule at one element pattern that recurs on several audited pages —
/// the batch view of a finding pattern (2.0 finding model, plan 100).
///
/// `id` is the pattern id of the findings (`a11y_report::RootCauseId`), so a
/// root cause links straight to `findings[].patterns[]` on every page.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootCause {
    pub id: String,
    pub rule_id: String,
    pub title: String,
    pub wcag_criterion: String,
    pub severity: Severity,
    /// Shortest raw selector seen for this pattern.
    pub selector: String,
    /// `confirmed`: same markup shape on every page; `likely`: same pattern,
    /// coverage thresholds met; `open`: recurs, but below the thresholds or
    /// too generic to say more. Never raised beyond what the evidence shows.
    pub confidence: a11y_report::CauseConfidence,
    pub url_count: usize,
    /// `url_count / audited pages`, rounded.
    pub page_coverage_pct: u32,
    pub occurrence_count: usize,
    /// Every page with this pattern.
    pub urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_snippet: Option<String>,
}

/// Root causes across a batch: every finding pattern that occurs on at least
/// two pages. Built from `findings[].patterns[]` (all occurrences), and the
/// markup evidence from the stored occurrence sample (`MIN_DISTINCT_PAGES`,
/// `MIN_COVERAGE_PCT`, snippet shape). A pattern on one page is no root
/// cause claim; it stays in its finding.
pub fn detect_root_causes(reports: &[NormalizedReport]) -> Vec<RootCause> {
    use a11y_report::{normalize_selector, CauseConfidence};

    struct Acc {
        rule_id: String,
        title: String,
        wcag_criterion: String,
        severity: Severity,
        selector: String,
        urls: Vec<String>,
        occurrence_count: usize,
        shapes: HashSet<String>,
        pages_without_snippet: usize,
        html_snippet: Option<String>,
    }

    let total_pages = reports.len();
    let mut by_id: HashMap<String, Acc> = HashMap::new();
    for report in reports {
        for finding in report.findings.iter().filter(|f| f.category == "wcag") {
            for pattern in &finding.patterns {
                let normalized = normalize_selector(&pattern.selector);
                let snippets: Vec<&str> = finding
                    .occurrences
                    .iter()
                    .filter(|o| {
                        normalize_selector(o.selector.as_deref().unwrap_or("document"))
                            == normalized
                    })
                    .filter_map(|o| o.html_snippet.as_deref())
                    .filter(|h| !h.trim().is_empty())
                    .collect();
                let acc = by_id.entry(pattern.id.clone()).or_insert_with(|| Acc {
                    rule_id: finding.rule_id.clone(),
                    title: finding.title.clone(),
                    wcag_criterion: finding.wcag_criterion.clone(),
                    severity: finding.severity,
                    selector: pattern.selector.clone(),
                    urls: Vec::new(),
                    occurrence_count: 0,
                    shapes: HashSet::new(),
                    pages_without_snippet: 0,
                    html_snippet: None,
                });
                if pattern.selector.len() < acc.selector.len() {
                    acc.selector = pattern.selector.clone();
                }
                if finding.severity > acc.severity {
                    acc.severity = finding.severity;
                }
                if !acc.urls.contains(&report.url) {
                    acc.urls.push(report.url.clone());
                    if snippets.is_empty() {
                        acc.pages_without_snippet += 1;
                    }
                }
                acc.occurrence_count += pattern.occurrence_count;
                for snippet in snippets {
                    acc.shapes.insert(snippet_shape_hash(snippet));
                    if acc.html_snippet.is_none() {
                        acc.html_snippet = Some(snippet.to_string());
                    }
                }
            }
        }
    }

    let mut causes: Vec<RootCause> = by_id
        .into_iter()
        .filter(|(_, acc)| acc.urls.len() >= 2)
        .map(|(id, acc)| {
            let url_count = acc.urls.len();
            let coverage = url_count as f64 / total_pages.max(1) as f64 * 100.0;
            let thresholds_met = url_count >= MIN_DISTINCT_PAGES && coverage >= MIN_COVERAGE_PCT;
            let same_markup = acc.pages_without_snippet == 0 && acc.shapes.len() == 1;
            let specific = acc.selector == "document" || is_specific_selector(&acc.selector);
            let confidence = match (thresholds_met, same_markup, specific) {
                (true, true, _) => CauseConfidence::Confirmed,
                (true, false, true) => CauseConfidence::Likely,
                _ => CauseConfidence::Open,
            };
            RootCause {
                id,
                rule_id: acc.rule_id,
                title: acc.title,
                wcag_criterion: acc.wcag_criterion,
                severity: acc.severity,
                selector: acc.selector,
                confidence,
                url_count,
                page_coverage_pct: coverage.round() as u32,
                occurrence_count: acc.occurrence_count,
                urls: acc.urls,
                html_snippet: acc.html_snippet,
            }
        })
        .collect();
    causes.sort_by(|a, b| {
        b.url_count
            .cmp(&a.url_count)
            .then_with(|| b.occurrence_count.cmp(&a.occurrence_count))
            .then_with(|| a.id.cmp(&b.id))
    });
    causes
}

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
    use a11y_report::CauseConfidence;

    fn page_with(url: &str, selector: Option<&str>, snippet: Option<&str>) -> NormalizedReport {
        use crate::wcag::{Violation, WcagResults};
        let mut violation = Violation::new(
            "4.1.2",
            "x",
            crate::cli::WcagLevel::A,
            Severity::High,
            "The link has no accessible name.",
            "n1",
        )
        .with_rule_id("buttons/name-missing");
        if let Some(selector) = selector {
            violation = violation.with_selector(selector.to_string());
        }
        if let Some(snippet) = snippet {
            violation = violation.with_html_snippet(snippet.to_string());
        }
        let mut results = WcagResults::new();
        results.add_violation(violation);
        let report =
            crate::audit::AuditReport::new(url.to_string(), crate::cli::WcagLevel::AA, results, 1);
        crate::audit::normalize(&report).normalized
    }

    fn pages(n: usize, selector: Option<&str>, snippet: Option<&str>) -> Vec<NormalizedReport> {
        (0..n)
            .map(|i| page_with(&format!("https://e.x/{i}"), selector, snippet))
            .collect()
    }

    const NAV: &str = "<a class=\"mainnav-toggle\" href=\"#\"></a>";

    #[test]
    fn same_markup_on_every_page_is_confirmed() {
        let batch = pages(3, Some("a.mainnav-toggle"), Some(NAV));
        let causes = detect_root_causes(&batch);
        assert_eq!(causes.len(), 1);
        assert_eq!(causes[0].confidence, CauseConfidence::Confirmed);
        assert_eq!(causes[0].url_count, 3);
        assert_eq!(
            causes[0].id, batch[0].findings[0].patterns[0].id,
            "root cause id is the pattern id of the findings"
        );
    }

    #[test]
    fn same_pattern_with_differing_markup_is_likely() {
        let batch: Vec<_> = (0..3)
            .map(|i| {
                let html = format!("<a class=\"t\"><span>{}</span><b/></a>", "x".repeat(i + 1));
                page_with(
                    &format!("https://e.x/{i}"),
                    Some("a.mainnav-toggle"),
                    Some(&html),
                )
            })
            .collect();
        assert_eq!(
            detect_root_causes(&batch)[0].confidence,
            CauseConfidence::Likely
        );
    }

    #[test]
    fn bare_tag_with_differing_markup_stays_open() {
        let batch: Vec<_> = (0..3)
            .map(|i| {
                let html = format!("<img src=\"{}.png\"><i/>{}", i, "y".repeat(i + 1));
                page_with(&format!("https://e.x/{i}"), Some("img"), Some(&html))
            })
            .collect();
        assert_eq!(
            detect_root_causes(&batch)[0].confidence,
            CauseConfidence::Open
        );
    }

    #[test]
    fn numeric_cms_ids_form_one_pattern() {
        let batch = vec![
            page_with("https://e.x/a", Some("div#block-123 > a"), Some(NAV)),
            page_with("https://e.x/b", Some("div#block-456 > a"), Some(NAV)),
            page_with("https://e.x/c", Some("div#block-7 > a"), Some(NAV)),
        ];
        let causes = detect_root_causes(&batch);
        assert_eq!(causes.len(), 1);
        assert_eq!(causes[0].url_count, 3);
    }

    #[test]
    fn two_pages_are_no_template_claim() {
        let causes = detect_root_causes(&pages(2, Some("a.mainnav-toggle"), Some(NAV)));
        assert_eq!(causes[0].confidence, CauseConfidence::Open);
    }

    #[test]
    fn coverage_threshold_is_sixty_percent() {
        let mixed = |hits: usize, total: usize| {
            let mut batch = pages(hits, Some("a.mainnav-toggle"), Some(NAV));
            batch.extend(
                (hits..total)
                    .map(|i| page_with(&format!("https://e.x/{i}"), Some("footer a.x"), None)),
            );
            detect_root_causes(&batch)
                .into_iter()
                .find(|c| c.selector == "a.mainnav-toggle")
                .expect("pattern")
                .confidence
        };
        assert_eq!(mixed(3, 5), CauseConfidence::Confirmed, "exactly 60 %");
        assert_eq!(mixed(3, 6), CauseConfidence::Open, "50 %");
    }

    #[test]
    fn selectorless_findings_form_a_document_pattern() {
        let causes = detect_root_causes(&pages(3, None, None));
        assert_eq!(causes.len(), 1);
        assert_eq!(causes[0].selector, "document");
        assert_eq!(causes[0].confidence, CauseConfidence::Likely);
    }

    #[test]
    fn a_pattern_on_one_page_is_no_root_cause() {
        let batch = vec![
            page_with("https://e.x/a", Some("a.only-here"), None),
            page_with("https://e.x/b", Some("a.elsewhere"), None),
        ];
        assert!(detect_root_causes(&batch).is_empty());
    }
}
