//! SEO heading structure analysis
//!
//! Collects the H1-H6 headings for the SEO view. The structural checks
//! (missing/multiple H1, skipped level, empty heading) are the shared
//! `headings/*` rules from a11y-rules, read from the accessibility results
//! instead of evaluated a second time (#724); long headings stay an SEO rule.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};
use crate::taxonomy::Severity;
use crate::util::truncate_url;

/// Heading structure analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingStructure {
    /// Number of H1 elements
    pub h1_count: usize,
    /// H1 text content (first one if multiple)
    pub h1_text: Option<String>,
    /// All headings in order
    pub headings: Vec<HeadingInfo>,
    /// Heading issues found
    pub issues: Vec<HeadingIssue>,
    /// Total heading count
    pub total_count: usize,
}

/// Information about a single heading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingInfo {
    /// Heading level (1-6)
    pub level: u8,
    /// Heading text content
    pub text: String,
    /// Character count
    pub length: usize,
    /// Whether heading text ends with a question mark
    #[serde(default)]
    pub is_question: bool,
    /// Whether heading sits inside an FAQ context (e.g. details/summary or class/id containing faq)
    #[serde(default)]
    pub in_faq_context: bool,
    /// Word count of the visible text between this heading and the next heading
    /// (or end of the content area if it's the last one)
    #[serde(default)]
    pub word_count_after: u32,
}

/// Heading-related SEO issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingIssue {
    /// Issue type
    pub issue_type: String,
    /// Issue description
    pub message: String,
    /// Severity: "error", "warning"
    pub severity: Severity,
}

/// Analyze heading structure of a page
pub async fn analyze_heading_structure(page: &Page) -> Result<HeadingStructure> {
    info!("Analyzing heading structure...");

    let js_code = r#"
    (() => {
        // Headings in open shadow roots count too, in the composed tree's
        // order (a shadow root's content right after its host): the shared
        // `headings/*` rules read that tree, and search engines index
        // rendered shadow DOM content (#724). Closed roots stay out of reach.
        const headingEls = [];
        const collect = (root) => {
            for (const el of root.querySelectorAll('*')) {
                if (/^H[1-6]$/.test(el.tagName)) headingEls.push(el);
                if (el.shadowRoot) collect(el.shadowRoot);
            }
        };
        collect(document);
        const countWords = (s) => {
            const t = s.trim();
            return t.length ? t.split(/\s+/).length : 0;
        };
        const headings = headingEls.map((h, i) => {
            const level = parseInt(h.tagName.charAt(1));
            const text = h.textContent.trim();
            const is_question = text.endsWith('?');
            const in_faq_context = !!(
                h.closest('[itemtype*="Question"]') ||
                h.closest('[itemtype*="FAQPage"]') ||
                h.closest('details') ||
                h.closest('.faq') ||
                h.closest('[class*="faq"]') ||
                h.closest('[id*="faq"]')
            );

            // Word count of the visible text between this heading and the next
            // heading (or end of the content area for the last heading).
            let word_count_after = 0;
            try {
                // A range cannot cross a shadow boundary: it ends at the next
                // heading of the same tree, else at the end of that tree.
                const range = document.createRange();
                range.setStartAfter(h);
                const root = h.getRootNode();
                const next = headingEls.slice(i + 1).find(n => n.getRootNode() === root);
                const container = root === document ? document.body : root;
                if (next) {
                    range.setEndBefore(next);
                } else if (container.lastChild) {
                    range.setEndAfter(container.lastChild);
                } else {
                    range.setEnd(container, 0);
                }
                const frag = range.cloneContents();
                const holder = document.createElement('div');
                holder.appendChild(frag);
                holder.querySelectorAll('script, style, noscript').forEach(e => e.remove());
                word_count_after = countWords(holder.textContent || '');
            } catch (e) {
                word_count_after = 0;
            }

            return { level, text, length: text.length, is_question, in_faq_context, word_count_after };
        });
        return JSON.stringify(headings);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Heading analysis failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("[]");

    let headings: Vec<HeadingInfo> = serde_json::from_str(json_str).unwrap_or_default();

    Ok(check_headings_structure(headings))
}

fn check_headings_structure(headings: Vec<HeadingInfo>) -> HeadingStructure {
    // Analyze structure
    let h1_headings: Vec<_> = headings.iter().filter(|h| h.level == 1).collect();
    let h1_count = h1_headings.len();
    let h1_text = h1_headings.first().map(|h| h.text.clone());

    // Missing and multiple H1, skipped levels and empty headings come from
    // the shared `headings/*` rules once the accessibility results exist
    // (`apply_shared_findings`, #724). Long headings are an SEO rule of
    // their own.
    let mut issues = Vec::new();

    // Check for very long headings
    for heading in &headings {
        let max_len = if heading.is_question || heading.in_faq_context {
            100
        } else if heading.level <= 2 {
            70
        } else {
            90
        };

        if heading.length > max_len {
            issues.push(HeadingIssue {
                issue_type: "long_heading".to_string(),
                message: format!(
                    "H{} is too long ({} chars): \"{}...\"",
                    heading.level,
                    heading.length,
                    truncate_url(&heading.text, 30)
                ),
                severity: Severity::Medium,
            });
        }
    }

    info!(
        "Heading structure: {} total, {} H1s, {} issues",
        headings.len(),
        h1_count,
        issues.len()
    );

    HeadingStructure {
        h1_count,
        h1_text,
        total_count: headings.len(),
        headings,
        issues,
    }
}

/// Shared rule id → SEO heading issue type and the SEO weight of it.
const SHARED_HEADING_ISSUES: [(&str, &str, Severity); 4] = [
    ("headings/h1-missing", "missing_h1", Severity::High),
    ("headings/h1-multiple", "multiple_h1", Severity::Medium),
    ("headings/skip-level", "skipped_level", Severity::Medium),
    ("headings/empty", "empty_heading", Severity::High),
];

/// Puts the shared `headings/*` findings (violations and review notes, as
/// the report shows them after exclusions) in front of the SEO-only issues.
/// A finding reported in both viewport passes counts once.
pub(crate) fn apply_shared_findings(
    structure: &mut HeadingStructure,
    results: &crate::wcag::WcagResults,
) {
    let mut seen = std::collections::HashSet::new();
    let mut shared = Vec::new();
    for (rule_id, issue_type, severity) in SHARED_HEADING_ISSUES {
        for finding in results.violations.iter().chain(&results.warnings) {
            if finding.rule_id.as_deref() != Some(rule_id) {
                continue;
            }
            let location = finding
                .selector
                .clone()
                .unwrap_or_else(|| finding.node_id.clone());
            if !seen.insert((rule_id, location, finding.message.clone())) {
                continue;
            }
            shared.push(HeadingIssue {
                issue_type: issue_type.to_string(),
                message: finding.message.clone(),
                severity,
            });
        }
    }
    let own = std::mem::take(&mut structure.issues)
        .into_iter()
        .filter(|i| {
            !SHARED_HEADING_ISSUES
                .iter()
                .any(|(_, t, _)| *t == i.issue_type)
        });
    structure.issues = shared.into_iter().chain(own).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_heading_findings_replace_the_structural_seo_checks() {
        // #724: missing H1 and a skipped level come from `headings/*`, each
        // once even when both viewport passes reported them; the long
        // heading stays an SEO issue; other rules are ignored.
        let finding = |rule: &str, selector: &str, message: &str| {
            crate::wcag::Violation::new(
                "1.3.1",
                "Info and Relationships",
                crate::cli::WcagLevel::A,
                crate::wcag::types::Severity::Medium,
                message,
                selector,
            )
            .with_selector(selector)
            .with_rule_id(rule)
        };
        let mut results = crate::wcag::WcagResults::new();
        results.violations = vec![
            finding(
                "headings/skip-level",
                "h4#a",
                "Heading level skips from h2 to h4",
            ),
            finding(
                "headings/skip-level",
                "h4#a",
                "Heading level skips from h2 to h4",
            ),
            finding("headings/h1-missing", "html", "The page has no h1"),
            finding("images/alt-missing", "img", "Image without alt"),
        ];
        let long = "x".repeat(120);
        let mut structure = check_headings_structure(vec![
            HeadingInfo {
                level: 2,
                text: long.clone(),
                length: long.len(),
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
            },
            HeadingInfo {
                level: 4,
                text: "Deep".into(),
                length: 4,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
            },
        ]);
        // Before the shared findings arrive only the SEO rule has fired.
        let types = |s: &HeadingStructure| -> Vec<String> {
            s.issues.iter().map(|i| i.issue_type.clone()).collect()
        };
        assert_eq!(types(&structure), ["long_heading"]);

        apply_shared_findings(&mut structure, &results);
        assert_eq!(
            types(&structure),
            ["missing_h1", "skipped_level", "long_heading"]
        );
        assert_eq!(structure.issues[0].severity, Severity::High);
        assert_eq!(
            structure.issues[1].message,
            "Heading level skips from h2 to h4"
        );
    }

    #[test]
    fn test_heading_info() {
        let heading = HeadingInfo {
            level: 1,
            text: "Test Heading".to_string(),
            length: 12,
            is_question: false,
            in_faq_context: false,
            word_count_after: 0,
        };

        assert_eq!(heading.level, 1);
        assert_eq!(heading.length, 12);
    }

    #[test]
    fn test_faq_question_heading_length() {
        let headings = vec![
            HeadingInfo {
                level: 2,
                text: "Wie läuft eine hyperbare Sauerstofftherapie in unserer Praxis in München ab?".to_string(),
                length: 76,
                is_question: true,
                in_faq_context: false,
                word_count_after: 0,
            },
            HeadingInfo {
                level: 2,
                text: "Normaler Heading mit mehr als 70 Zeichen der eigentlich einen Fehler werfen sollte".to_string(),
                length: 82,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
            }
        ];
        let res = check_headings_structure(headings);
        let long_heading_issues: Vec<_> = res
            .issues
            .iter()
            .filter(|i| i.issue_type == "long_heading")
            .collect();
        assert_eq!(long_heading_issues.len(), 1);
        assert!(long_heading_issues[0].message.contains("H2 is too long"));
    }
}
