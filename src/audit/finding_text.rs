//! Run-language text of non-WCAG findings (#898, #406).
//!
//! A finding stores its message in canonical English (the JSON never depends on
//! the run language). The PDF must show the message in the run language, but a
//! plain string cannot be translated back. So every occurrence also carries a
//! [`MessageText`]: the module's own pure `*_text(kind, values, en)` function
//! with the kind and the raw values bound in. The analysis calls it with
//! `en = true` for the stored message, the PDF builder with the run language
//! ([`localize_domain_texts`]). The text function stays the only source of
//! wording; nothing here writes a sentence.
//!
//! The binding is memory-only: it is never serialized, so a report read back
//! from JSON keeps its canonical English text.

use std::fmt;
use std::sync::Arc;

use crate::audit::normalized::NormalizedFinding;

/// A message as a function of the language: `true` = canonical English.
#[derive(Clone)]
pub struct MessageText(Arc<dyn Fn(bool) -> String + Send + Sync>);

impl MessageText {
    /// Bind a module's text function with its kind and raw values.
    pub fn new(text: impl Fn(bool) -> String + Send + Sync + 'static) -> Self {
        Self(Arc::new(text))
    }

    /// The message in the requested language.
    pub fn text(&self, en: bool) -> String {
        (self.0)(en)
    }
}

impl fmt::Debug for MessageText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MessageText(..)")
    }
}

/// Re-derive the descriptions of non-WCAG findings in the run language.
///
/// Replaces `description`, `user_impact`, `technical_impact` and every
/// occurrence message that has a bound [`MessageText`]; the canonical English
/// of the JSON is not touched because the PDF builders work on a copy.
pub fn localize_domain_texts(findings: &mut [NormalizedFinding], en: bool) {
    for finding in findings {
        let mut first: Option<String> = None;
        for occurrence in &mut finding.occurrences {
            if let Some(text) = &occurrence.text {
                let localized = text.text(en);
                first.get_or_insert_with(|| localized.clone());
                occurrence.message = localized;
            }
        }
        if let Some(message) = first {
            finding.description = message.clone();
            finding.user_impact = message.clone();
            finding.technical_impact = message;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::finding_contract::build_domain_findings;
    use crate::audit::AuditReport;
    use crate::seo::{MetaTags, PageHealthAnalysis, SeoAnalysis};
    use crate::wcag::WcagResults;

    /// English wording that must never reach a German report.
    const ENGLISH_PHRASES: &[&str] = &[
        "is too",
        "Missing ",
        "chars",
        "recommended",
        "elements",
        "DOM size",
        "Smallest font",
        "touch targets",
        "not served",
        "without",
        "images",
        // html-conform (#902)
        "unexpected",
        "Unexpected",
        "xhtml",
        "is unnecessary",
        "lacks heading",
        "invalid value",
        "must have",
    ];

    fn report() -> AuditReport {
        let mut report = AuditReport::new(
            "https://example.com".into(),
            crate::cli::WcagLevel::AA,
            WcagResults::new(),
            100,
        );
        let mut seo = SeoAnalysis::default();
        seo.meta = MetaTags {
            title: Some("Kurz".into()),
            description: Some("x".repeat(176)),
            ..Default::default()
        };
        seo.meta_issues = seo.meta.validate();
        let mut health = PageHealthAnalysis {
            has_doctype: true,
            dom_node_count: 1714,
            dom_max_depth: 17,
            images_without_srcset: 12,
            ..Default::default()
        };
        health.issues = crate::seo::collect_issues(&health, true);
        seo.page_health = Some(health);
        seo.technical.issues =
            crate::seo::technical::collect_technical_issues(&seo.technical, true);
        report.discoverability.seo = Some(seo);
        report.html_conform = Some(crate::html_conform::HtmlConformAnalysis {
            score: 80,
            checked: true,
            error_count: 3,
            warning_count: 1,
            info_count: 0,
            distinct_defect_count: 4,
            findings: [
                (
                    "schema.html5",
                    "error",
                    "unexpected element `{http://www.w3.org/1999/xhtml}span` at 190:12271",
                ),
                (
                    "schema.html5",
                    "error",
                    "invalid value `search` for attribute `type` at 281:508",
                ),
                (
                    "assertion.roles.unnecessary-list",
                    "warning",
                    "The \"list\" role is unnecessary for this element.",
                ),
                (
                    "assertion.elements.img-missing-alt",
                    "error",
                    "An img element must have an alt attribute, except under certain conditions.",
                ),
            ]
            .into_iter()
            .map(
                |(rule_id, severity, message)| crate::html_conform::HtmlConformFinding {
                    rule_id: rule_id.to_string(),
                    severity: severity.to_string(),
                    message: message.to_string(),
                    location: Some("190:12271".to_string()),
                    byte_offset: None,
                },
            )
            .collect(),
            raw_html: None,
        });
        report
    }

    fn domain_findings() -> Vec<NormalizedFinding> {
        build_domain_findings(&report(), &[], 0, None, 5).0
    }

    /// The stored message is the `en = true` rendering of the bound text, so
    /// the JSON does not change (#898).
    #[test]
    fn bound_text_in_english_equals_the_stored_message() {
        let findings = domain_findings();
        assert!(findings.len() >= 5, "fixture yields too few findings");
        for finding in &findings {
            for occurrence in &finding.occurrences {
                let text = occurrence.text.as_ref().unwrap_or_else(|| {
                    panic!("{}: no text bound", finding.rule_id);
                });
                assert_eq!(text.text(true), occurrence.message, "{}", finding.rule_id);
            }
        }
    }

    #[test]
    fn english_text_has_no_german_characters() {
        for finding in &domain_findings() {
            for occurrence in &finding.occurrences {
                let en = occurrence.text.as_ref().unwrap().text(true);
                assert!(
                    !en.chars().any(|c| "äöüÄÖÜß".contains(c)),
                    "{}: {en}",
                    finding.rule_id
                );
            }
        }
    }

    #[test]
    fn german_text_and_title_contain_no_english_phrases() {
        let mut findings = domain_findings();
        localize_domain_texts(&mut findings, false);
        for finding in &findings {
            let title = crate::audit::finding_titles::german_finding_title(&finding.rule_id)
                .unwrap_or_else(|| panic!("{}: no German title", finding.rule_id));
            for text in std::iter::once(title.to_string())
                .chain(std::iter::once(finding.description.clone()))
                .chain(finding.occurrences.iter().map(|o| o.message.clone()))
            {
                for phrase in ENGLISH_PHRASES {
                    assert!(
                        !text.contains(phrase),
                        "{}: English phrase {phrase:?} in {text:?}",
                        finding.rule_id
                    );
                }
            }
        }
    }

    /// The validator's parser noise stays in the JSON text and is gone from
    /// the German one (#902).
    #[test]
    fn html_conform_messages_are_readable_german_without_parser_noise() {
        let mut findings = domain_findings();
        let stored: Vec<String> = findings
            .iter()
            .filter(|f| f.rule_id.starts_with("html_conform."))
            .flat_map(|f| f.occurrences.iter().map(|o| o.message.clone()))
            .collect();
        assert!(stored
            .iter()
            .any(|m| m.contains("{http://www.w3.org/1999/xhtml}")));
        localize_domain_texts(&mut findings, false);
        for finding in findings
            .iter()
            .filter(|f| f.rule_id.starts_with("html_conform."))
        {
            for o in &finding.occurrences {
                assert!(!o.message.contains("xhtml"), "{}", o.message);
                assert!(!o.message.contains("12271"), "{}", o.message);
            }
        }
        let schema = findings
            .iter()
            .find(|f| f.rule_id == "html_conform.schema.html5")
            .unwrap();
        assert!(schema
            .occurrences
            .iter()
            .any(|o| o.message == "Unerwartetes Element <span> an dieser Stelle."));
    }

    #[test]
    fn localizing_to_english_changes_nothing() {
        let mut findings = domain_findings();
        let before: Vec<String> = findings.iter().map(|f| f.description.clone()).collect();
        localize_domain_texts(&mut findings, true);
        let after: Vec<String> = findings.iter().map(|f| f.description.clone()).collect();
        assert_eq!(before, after);
    }

    /// The reported leak (#898): meta length and DOM size.
    #[test]
    fn the_reported_messages_are_german_in_a_german_run() {
        let mut findings = domain_findings();
        localize_domain_texts(&mut findings, false);
        let text = |rule: &str| {
            findings
                .iter()
                .find(|f| f.rule_id == rule)
                .unwrap_or_else(|| panic!("{rule} missing"))
                .description
                .clone()
        };
        assert!(text("seo.meta.description").starts_with("Beschreibung ist zu lang (176 Zeichen"));
        assert!(text("seo.page_health.excessive_dom")
            .starts_with("DOM-Größe kritisch: 1714 HTML-Elemente"));
    }
}
