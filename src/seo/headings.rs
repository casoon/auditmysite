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
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
                hidden: false,
                in_dialog: false,
            },
            HeadingInfo {
                level: 4,
                text: "Deep".into(),
                length: 4,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
                hidden: false,
                in_dialog: false,
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
            hidden: false,
            in_dialog: false,
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
                hidden: false,
                in_dialog: false,
            },
            HeadingInfo {
                level: 2,
                text: "Normaler Heading mit mehr als 70 Zeichen der eigentlich einen Fehler werfen sollte".to_string(),
                length: 82,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
                hidden: false,
                in_dialog: false,
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
            is_question: false,
            in_faq_context: false,
            word_count_after: 0,
            hidden: false,
            in_dialog,
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
}
