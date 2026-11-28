//! SEO heading structure analysis
//!
//! Collects the H1-H6 headings for the SEO view. The structural checks
//! (missing/multiple H1, skipped level, empty heading) are the shared
//! `headings/*` rules from a11y-rules, read from the accessibility results
//! instead of evaluated a second time (#724); long headings stay an SEO rule.

use chromiumoxide::Page;
use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::Result;
use crate::taxonomy::Severity;
use crate::util::truncate_url;

/// Heading structure analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingStructure {
    /// Number of H1 elements of the page's content: an H1 inside a dialog or
    /// consent layer is not counted here (#1132), see `dialog_h1_count`.
    pub h1_count: usize,
    /// H1 text content (first one if multiple), outside dialogs
    pub h1_text: Option<String>,
    /// H1 elements inside a dialog or consent layer. They do not describe the
    /// page topic, so they are neither `h1_count` nor `h1_text`.
    #[serde(default)]
    pub dialog_h1_count: usize,
    /// All headings in order
    pub headings: Vec<HeadingInfo>,
    /// Heading issues found
    pub issues: Vec<HeadingIssue>,
    /// Total heading count
    pub total_count: usize,
}

/// Information about a single heading
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeadingInfo {
    /// Heading level (1-6)
    pub level: u8,
    /// Visible DOM text (layout-aware)
    pub text: String,
    /// Accessible name (`aria-label`/`aria-labelledby` or non-`aria-hidden` DOM text) (#1272)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessible_name: Option<String>,
    /// Raw extracted text as a crawler without ARIA reads it (#1272)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_text: Option<String>,
    /// Heading text exists only in `aria-label`/`aria-labelledby` while visible DOM children are `aria-hidden="true"` (#1272)
    #[serde(default)]
    pub has_aria_label_only: bool,
    /// Number of leaf inline elements containing exactly one character (per-letter animation spans) (#1272)
    #[serde(default)]
    pub single_char_span_count: u32,
    /// Share of non-whitespace characters wrapped in single-character elements (0.0..=1.0) (#1272)
    #[serde(default)]
    pub single_char_span_ratio: f32,
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
    /// Not exposed to users or assistive technology (`hidden`, `inert`,
    /// `aria-hidden`, or not rendered). Still counted for the SEO rules, but
    /// no reader meets it: it never decides which heading comes first (#952-1).
    #[serde(default)]
    pub hidden: bool,
    /// Inside a dialog or a consent layer (`<dialog>`, `role="dialog"`,
    /// `aria-modal`, a known consent manager): part of that layer, not of the
    /// page's content (#1132). It never counts as the page's H1 and never
    /// decides which heading comes first.
    #[serde(default)]
    pub in_dialog: bool,
}

impl HeadingStructure {
    /// Level of the first heading of the page's content that a reader
    /// encounters: the first one that is exposed and not part of a dialog.
    /// One definition for the SEO heading signals, the structure signal and
    /// the screen-reader heading audit (which reads the accessibility tree
    /// and never sees hidden headings).
    pub fn first_exposed_level(&self) -> Option<u8> {
        self.headings
            .iter()
            .find(|h| !h.hidden && !h.in_dialog)
            .map(|h| h.level)
    }
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
    /// Raw values of an own `long_heading` issue, to re-derive `message` in the
    /// run language (#898). In-memory only; shared-rule issues carry none.
    #[serde(skip)]
    pub long_heading: Option<LongHeading>,
}

/// Raw values of a `long_heading` issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongHeading {
    pub level: u8,
    pub length: usize,
    /// First characters of the heading text (page content, not translated).
    pub excerpt: String,
}

impl HeadingIssue {
    /// The message in the requested language; issues from the shared rules or
    /// read back from JSON keep their stored text.
    pub fn localized_message(&self, en: bool) -> String {
        match &self.long_heading {
            Some(values) => long_heading_text(values, en),
            None => self.message.clone(),
        }
    }
}

/// The only source of the own heading-issue wording (#406, #898).
pub fn long_heading_text(values: &LongHeading, en: bool) -> String {
    crate::i18n::text_args(
        "audit-interpretation-long-heading",
        &[
            ("level", FluentValue::from(values.level.to_string())),
            ("length", FluentValue::from(values.length.to_string())),
            ("excerpt", FluentValue::from(values.excerpt.clone())),
        ],
        en,
    )
}

/// Heading outline script; the SEO score reads it.
pub(crate) static HEADINGS: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "seo.headings",
        module: "seo",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 3,
        scoring: true,
    };

/// Analyze heading structure of a page
pub async fn analyze_heading_structure(
    page: &Page,
    log: &crate::audit::collection::CollectionLog,
) -> Result<HeadingStructure> {
    info!("Analyzing heading structure...");

    let js_code = include_str!("js/headings.js");

    let headings: Vec<HeadingInfo> = log
        .record(
            &HEADINGS,
            crate::audit::collection::evaluate_js(page, js_code).await,
        )
        .empty_if(Vec::is_empty)
        .into_result(&HEADINGS)?;

    Ok(check_headings_structure(headings))
}

fn check_headings_structure(headings: Vec<HeadingInfo>) -> HeadingStructure {
    // Analyze structure
    let h1_headings: Vec<_> = headings
        .iter()
        .filter(|h| h.level == 1 && !h.in_dialog)
        .collect();
    let h1_count = h1_headings.len();
    let h1_text = h1_headings.first().map(|h| h.text.clone());
    let dialog_h1_count = headings
        .iter()
        .filter(|h| h.level == 1 && h.in_dialog)
        .count();

    // Missing and multiple H1, skipped levels and empty headings come from
    // the shared `headings/*` rules once the accessibility results exist
    // (`apply_shared_findings`, #724). Long headings are an SEO rule of
    // their own.
    let mut issues = Vec::new();

    // Check for very long headings
    for heading in headings.iter().filter(|h| !h.in_dialog) {
        let max_len = if heading.is_question || heading.in_faq_context {
            100
        } else if heading.level <= 2 {
            70
        } else {
            90
        };

        if heading.length > max_len {
            let values = LongHeading {
                level: heading.level,
                length: heading.length,
                excerpt: truncate_url(&heading.text, 30),
            };
            issues.push(HeadingIssue {
                issue_type: "long_heading".to_string(),
                message: long_heading_text(&values, true),
                severity: Severity::Medium,
                long_heading: Some(values),
            });
        }

        // #1272: Heading extractability checks (H1/H2)
        if heading.level <= 2 {
            if heading.has_aria_label_only {
                let severity = if heading.level == 1 {
                    Severity::High
                } else {
                    Severity::Medium
                };
                issues.push(HeadingIssue {
                    issue_type: "heading_aria_label_only".to_string(),
                    message: format!(
                        "H{} text exists only in aria-label/aria-labelledby while visible content is aria-hidden — accessible to screen readers, but search engines and AI extractors may see an empty or different heading",
                        heading.level
                    ),
                    severity,
                    long_heading: None,
                });
            } else if let Some(ref acc) = heading.accessible_name {
                let norm_vis = normalize_heading_compare(&heading.text);
                let norm_acc = normalize_heading_compare(acc);
                if !norm_vis.is_empty() && !norm_acc.is_empty() && norm_vis != norm_acc {
                    issues.push(HeadingIssue {
                        issue_type: "heading_accessible_name_mismatch".to_string(),
                        message: format!(
                            "H{} visible text ('{}') differs from its accessible name ('{}') — crawlers and assistive tools read different headings",
                            heading.level,
                            truncate_url(&heading.text, 40),
                            truncate_url(acc, 40),
                        ),
                        severity: Severity::Medium,
                        long_heading: None,
                    });
                }
            }

            if heading.single_char_span_count >= 5 && heading.single_char_span_ratio >= 0.5 {
                issues.push(HeadingIssue {
                    issue_type: "heading_single_char_spans".to_string(),
                    message: format!(
                        "H{} text is split across {} single-character elements ({:.0}% of characters) — raw text extraction without layout can produce fragmented letter sequences",
                        heading.level,
                        heading.single_char_span_count,
                        heading.single_char_span_ratio * 100.0,
                    ),
                    severity: Severity::Low,
                    long_heading: None,
                });
            }
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
        dialog_h1_count,
        total_count: headings.len(),
        headings,
        issues,
    }
}

fn normalize_heading_compare(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

const STOPWORDS: &[&str] = &[
    // German
    "aber", "alle", "als", "also", "auch", "auf", "aus", "bei", "bis", "das", "dass", "dem", "den",
    "der", "des", "die", "dies", "diese", "dieser", "ein", "eine", "einem", "einen", "einer",
    "eines", "für", "fuer", "hat", "hier", "ihr", "ihre", "ihrem", "ihren", "ihrer", "ist", "kann",
    "kein", "keine", "mit", "nach", "nicht", "noch", "nur", "oder", "sehr", "sein", "seine",
    "sich", "sie", "sind", "über", "ueber", "und", "uns", "unser", "unsere", "vom", "von", "vor",
    "was", "wie", "wir", "wird", "zum", "zur", // English
    "about", "all", "and", "are", "but", "can", "for", "from", "has", "have", "here", "how",
    "into", "its", "more", "not", "our", "out", "over", "that", "the", "their", "them", "they",
    "this", "was", "were", "what", "when", "where", "which", "who", "will", "with", "you", "your",
];

fn content_tokens(text: &str) -> std::collections::HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(|w| w.to_lowercase())
        .filter(|w| w.chars().count() >= 3 && !STOPWORDS.contains(&w.as_str()))
        .collect()
}

/// Checks whether the page's H1 shares at least one meaningful (non-stopword)
/// term with `<title>` or `<meta name="description">` (#1272).
pub(crate) fn check_h1_title_alignment(
    structure: &mut HeadingStructure,
    title: Option<&str>,
    description: Option<&str>,
) {
    let Some(h1) = structure
        .headings
        .iter()
        .find(|h| h.level == 1 && !h.in_dialog)
    else {
        return;
    };
    let h1_basis = h1
        .accessible_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(&h1.text);
    let h1_tokens = content_tokens(h1_basis);
    if h1_tokens.is_empty() {
        return;
    }

    let mut meta_tokens = std::collections::HashSet::new();
    if let Some(t) = title {
        meta_tokens.extend(content_tokens(t));
    }
    if let Some(d) = description {
        meta_tokens.extend(content_tokens(d));
    }
    if meta_tokens.is_empty() {
        return;
    }

    if h1_tokens.is_disjoint(&meta_tokens) {
        structure.issues.push(HeadingIssue {
            issue_type: "h1_no_title_overlap".to_string(),
            message: format!(
                "H1 ('{}') shares no content terms with <title> or meta description — search engines and AI systems may struggle to align the main heading with the page topic",
                truncate_url(h1_basis, 40),
            ),
            severity: Severity::Low,
            long_heading: None,
        });
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
                long_heading: None,
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

/// Counter-cases of [`HEADINGS`] (#777).
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_contract::<Vec<HeadingInfo>>(serde_json::json!([]));
    // A failed collection stops the module instead of yielding a default.
    assert!(
        crate::audit::collection::Collected::<Vec<HeadingInfo>>::Failed("x")
            .into_result(&HEADINGS)
            .is_err()
    );
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
                crate::wcag::WcagLevel::A,
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
                ..Default::default()
            },
            HeadingInfo {
                level: 4,
                text: "Deep".into(),
                length: 4,
                ..Default::default()
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
            ..Default::default()
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
                ..Default::default()
            },
            HeadingInfo {
                level: 2,
                text: "Normaler Heading mit mehr als 70 Zeichen der eigentlich einen Fehler werfen sollte".to_string(),
                length: 82,
                ..Default::default()
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

    fn heading(level: u8, text: &str, in_dialog: bool) -> HeadingInfo {
        HeadingInfo {
            level,
            text: text.to_string(),
            length: text.chars().count(),
            in_dialog,
            ..Default::default()
        }
    }

    /// #1132: the H1 of a consent dialog is not the page's H1. SEO listed it
    /// as "H1 present" while the keyboard journey, reading the page content,
    /// reported none.
    #[test]
    fn a_dialog_h1_is_not_the_pages_h1() {
        let structure = check_headings_structure(vec![
            heading(1, "Privacy settings", true),
            heading(2, "Services", false),
        ]);
        assert_eq!(structure.h1_count, 0);
        assert_eq!(structure.h1_text, None);
        assert_eq!(structure.dialog_h1_count, 1);
        assert_eq!(structure.total_count, 2);
    }

    #[test]
    fn the_page_h1_is_found_next_to_a_dialog_h1() {
        let structure = check_headings_structure(vec![
            heading(1, "Privacy settings", true),
            heading(1, "Our services", false),
        ]);
        assert_eq!(structure.h1_count, 1);
        assert_eq!(structure.h1_text.as_deref(), Some("Our services"));
        assert_eq!(structure.dialog_h1_count, 1);
    }

    #[test]
    fn a_dialog_heading_never_decides_which_heading_comes_first() {
        let structure = check_headings_structure(vec![
            heading(1, "Privacy settings", true),
            heading(2, "Services", false),
        ]);
        assert_eq!(structure.first_exposed_level(), Some(2));
    }

    #[test]
    fn a_long_dialog_heading_is_no_page_issue() {
        let long = "x".repeat(120);
        let structure = check_headings_structure(vec![heading(2, &long, true)]);
        assert!(structure.issues.is_empty());
    }

    /// Reports written before `in_dialog` existed still read back.
    #[test]
    fn old_json_without_the_dialog_fields_reads_back() {
        let info: HeadingInfo =
            serde_json::from_str(r#"{"level":1,"text":"T","length":1}"#).unwrap();
        assert!(!info.in_dialog);
        let structure: HeadingStructure = serde_json::from_str(
            r#"{"h1_count":1,"h1_text":"T","headings":[],"issues":[],"total_count":0}"#,
        )
        .unwrap();
        assert_eq!(structure.dialog_h1_count, 0);
    }

    /// #1272: H1 with text only in aria-label, visible content aria-hidden and
    /// split into single-char spans, with no overlap with `<title>` triggers
    /// the error and both hints; JSON exposes all three readings.
    #[test]
    fn h1_aria_label_only_single_char_spans_and_title_mismatch() {
        let h1 = HeadingInfo {
            level: 1,
            text: "Digitales Handwerk".into(),
            accessible_name: Some("Digitales Handwerk".into()),
            extracted_text: Some("D i g i t a l e s H a n d w e r k".into()),
            has_aria_label_only: true,
            single_char_span_count: 17,
            single_char_span_ratio: 1.0,
            length: 18,
            ..Default::default()
        };
        let serialized = serde_json::to_value(&h1).unwrap();
        assert_eq!(serialized["text"], "Digitales Handwerk");
        assert_eq!(serialized["accessible_name"], "Digitales Handwerk");
        assert_eq!(
            serialized["extracted_text"],
            "D i g i t a l e s H a n d w e r k"
        );

        let mut structure = check_headings_structure(vec![h1]);
        check_h1_title_alignment(
            &mut structure,
            Some("Webdesign Agentur Rostock | Casoon"),
            Some("Barrierefreie Webseiten und Audits in Mecklenburg-Vorpommern"),
        );

        let by_type = |t: &str| structure.issues.iter().find(|i| i.issue_type == t);
        assert_eq!(
            by_type("heading_aria_label_only").unwrap().severity,
            Severity::High
        );
        assert_eq!(
            by_type("heading_single_char_spans").unwrap().severity,
            Severity::Low
        );
        assert_eq!(
            by_type("h1_no_title_overlap").unwrap().severity,
            Severity::Low
        );
    }

    /// #1272: Normal H1 with span line breaks and overlapping `<title>` produces
    /// no extractability findings.
    #[test]
    fn normal_h1_with_line_break_spans_has_no_extractability_issues() {
        let h1 = HeadingInfo {
            level: 1,
            text: "Webdesign & Entwicklung in Rostock".into(),
            accessible_name: Some("Webdesign & Entwicklung in Rostock".into()),
            extracted_text: Some("Webdesign & Entwicklung in Rostock".into()),
            has_aria_label_only: false,
            single_char_span_count: 0,
            single_char_span_ratio: 0.0,
            length: 34,
            ..Default::default()
        };
        let mut structure = check_headings_structure(vec![h1]);
        check_h1_title_alignment(
            &mut structure,
            Some("Webdesign Agentur Rostock | Casoon"),
            None,
        );
        assert!(structure.issues.is_empty(), "{:?}", structure.issues);
    }

    /// #1272: Visible text differing from accessible name is flagged as Medium.
    #[test]
    fn h1_accessible_name_mismatch_is_flagged() {
        let h1 = HeadingInfo {
            level: 1,
            text: "Willkommen".into(),
            accessible_name: Some("Casoon Webdesign Rostock".into()),
            extracted_text: Some("Willkommen".into()),
            has_aria_label_only: false,
            length: 10,
            ..Default::default()
        };
        let structure = check_headings_structure(vec![h1]);
        let issue = structure
            .issues
            .iter()
            .find(|i| i.issue_type == "heading_accessible_name_mismatch")
            .expect("mismatch issue");
        assert_eq!(issue.severity, Severity::Medium);
    }
}
