//! Meta tags extraction and validation
//!
//! Extracts and validates title, description, keywords, and other meta tags.

use chromiumoxide::Page;
use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};
use tracing::info;
use web_checks::meta::LengthClass;

use crate::error::Result;
use crate::taxonomy::Severity;

/// Extracted meta tags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaTags {
    /// Page title
    pub title: Option<String>,
    /// Meta description
    pub description: Option<String>,
    /// Meta keywords
    pub keywords: Option<String>,
    /// Robots meta tag
    pub robots: Option<String>,
    /// Author meta tag
    pub author: Option<String>,
    /// Viewport meta tag
    pub viewport: Option<String>,
    /// Charset
    pub charset: Option<String>,
    /// Canonical URL
    pub canonical: Option<String>,
    /// Language (from html lang attribute)
    pub lang: Option<String>,
}

/// Meta tag validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaValidation {
    /// Field name
    pub field: String,
    /// Issue description
    pub message: String,
    /// Severity
    pub severity: Severity,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// What kind of problem this is; the PDF re-derives `message` from it in
    /// the run language (#898). In-memory only.
    #[serde(skip)]
    pub kind: Option<MetaIssueKind>,
    /// Raw values of the finding (length, offending value). In-memory only.
    #[serde(skip)]
    pub values: MetaIssueValues,
}

/// Canonical kind of a meta-tag finding (#898).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaIssueKind {
    MissingTitle,
    TitleTooShort,
    TitleTooLong,
    MissingDescription,
    DescriptionTooShort,
    DescriptionTooLong,
    MissingViewport,
    MissingLang,
    MissingCanonical,
    InvalidTwitterCard,
    RelativeOgImage,
}

/// Raw values of a meta-tag finding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetaIssueValues {
    /// Length of the title or description in characters.
    pub length: Option<usize>,
    /// The offending attribute value (twitter:card type, og:image URL).
    pub value: Option<String>,
    /// Allowed values, comma separated (twitter:card).
    pub allowed: Option<String>,
}

impl MetaValidation {
    /// A finding of `kind` with its message in canonical English.
    pub(crate) fn new(
        field: &str,
        kind: MetaIssueKind,
        values: MetaIssueValues,
        severity: Severity,
        suggestion: &str,
    ) -> Self {
        Self {
            field: field.to_string(),
            message: meta_issue_text(kind, &values, true),
            severity,
            suggestion: Some(suggestion.to_string()),
            kind: Some(kind),
            values,
        }
    }

    /// The message in the requested language; a finding without a kind (read
    /// back from JSON) keeps its stored canonical English.
    pub fn localized_message(&self, en: bool) -> String {
        match self.kind {
            Some(kind) => meta_issue_text(kind, &self.values, en),
            None => self.message.clone(),
        }
    }
}

/// The only source of meta-tag finding wording (#406, #898). The analysis calls
/// it with `en = true` for the stored message, the PDF with the run language.
/// The wording is the `seo-meta-*` message of `locales/{en,de}/seo-issues.ftl`.
pub fn meta_issue_text(kind: MetaIssueKind, values: &MetaIssueValues, en: bool) -> String {
    use MetaIssueKind::*;
    let key = match kind {
        MissingTitle => "seo-meta-missing-title",
        TitleTooShort => "seo-meta-title-too-short",
        TitleTooLong => "seo-meta-title-too-long",
        MissingDescription => "seo-meta-missing-description",
        DescriptionTooShort => "seo-meta-description-too-short",
        DescriptionTooLong => "seo-meta-description-too-long",
        MissingViewport => "seo-meta-missing-viewport",
        MissingLang => "seo-meta-missing-lang",
        MissingCanonical => "seo-meta-missing-canonical",
        InvalidTwitterCard => "seo-meta-invalid-twitter-card",
        RelativeOgImage => "seo-meta-relative-og-image",
    };
    let args = [
        (
            "length",
            FluentValue::from(values.length.unwrap_or(0) as i64),
        ),
        (
            "value",
            FluentValue::from(values.value.clone().unwrap_or_default()),
        ),
        (
            "allowed",
            FluentValue::from(values.allowed.clone().unwrap_or_default()),
        ),
    ];
    crate::i18n::text_args(key, &args, en)
}

impl MetaTags {
    /// Validate meta tags and return issues
    pub fn validate(&self) -> Vec<MetaValidation> {
        use MetaIssueKind::*;
        let mut issues = Vec::new();
        let none = MetaIssueValues::default;
        let sized = |len: usize| MetaIssueValues {
            length: Some(len),
            ..Default::default()
        };

        // Title validation
        match &self.title {
            None => issues.push(MetaValidation::new(
                "title",
                MissingTitle,
                none(),
                Severity::High,
                "Add a <title> tag to the page",
            )),
            Some(title) => {
                let len = web_checks::meta::length(title);
                match web_checks::meta::TITLE.classify(title) {
                    LengthClass::TooShort => issues.push(MetaValidation::new(
                        "title",
                        TitleTooShort,
                        sized(len),
                        Severity::Medium,
                        "Expand title to 30-60 characters",
                    )),
                    LengthClass::TooLong => issues.push(MetaValidation::new(
                        "title",
                        TitleTooLong,
                        sized(len),
                        Severity::Medium,
                        "Shorten title to under 60 characters",
                    )),
                    _ => {}
                }
            }
        }

        // Description validation
        match &self.description {
            None => issues.push(MetaValidation::new(
                "description",
                MissingDescription,
                none(),
                Severity::High,
                "Add a meta description tag",
            )),
            Some(desc) => {
                let len = web_checks::meta::length(desc);
                match web_checks::meta::DESCRIPTION.classify(desc) {
                    LengthClass::TooShort => issues.push(MetaValidation::new(
                        "description",
                        DescriptionTooShort,
                        sized(len),
                        Severity::Medium,
                        "Expand description to 120-160 characters",
                    )),
                    LengthClass::TooLong => issues.push(MetaValidation::new(
                        "description",
                        DescriptionTooLong,
                        sized(len),
                        Severity::Medium,
                        "Shorten description to under 160 characters",
                    )),
                    _ => {}
                }
            }
        }

        // Viewport validation
        if self.viewport.is_none() {
            issues.push(MetaValidation::new(
                "viewport",
                MissingViewport,
                none(),
                Severity::High,
                "Add <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
            ));
        }

        // Language validation
        if self.lang.is_none() {
            issues.push(MetaValidation::new(
                "lang",
                MissingLang,
                none(),
                Severity::Medium,
                "Add lang attribute: <html lang=\"en\">",
            ));
        }

        // Canonical validation
        if self.canonical.is_none() {
            issues.push(MetaValidation::new(
                "canonical",
                MissingCanonical,
                none(),
                Severity::Low,
                "Add <link rel=\"canonical\" href=\"...\">",
            ));
        }

        issues
    }

    /// Check if essential meta tags are present
    pub fn has_essentials(&self) -> bool {
        self.title.is_some() && self.description.is_some() && self.viewport.is_some()
    }
}

/// Meta tag script; the SEO score reads it.
pub(crate) static META: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "seo.meta",
        module: "seo",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: true,
    };

/// Extract meta tags from a page
pub async fn extract_meta_tags(
    page: &Page,
    log: &crate::audit::collection::CollectionLog,
) -> Result<MetaTags> {
    info!("Extracting meta tags...");

    let js_code = include_str!("js/meta_tags.js");

    let meta: MetaTags = log
        .record(
            &META,
            crate::audit::collection::evaluate_js(page, js_code).await,
        )
        .into_result(&META)?;

    info!(
        "Meta tags: title={}, description={}, viewport={}",
        meta.title.is_some(),
        meta.description.is_some(),
        meta.viewport.is_some()
    );

    Ok(meta)
}

/// Counter-cases of [`META`] (#777).
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_contract::<MetaTags>(serde_json::json!({}));
    // A failed collection stops the module instead of yielding a default.
    assert!(crate::audit::collection::Collected::<MetaTags>::Failed("x")
        .into_result(&META)
        .is_err());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_validation_missing_title() {
        let meta = MetaTags::default();
        let issues = meta.validate();

        assert!(issues
            .iter()
            .any(|i| i.field == "title" && i.severity == Severity::High));
    }

    #[test]
    fn test_meta_validation_short_title() {
        let meta = MetaTags {
            title: Some("Short".to_string()),
            ..Default::default()
        };
        let issues = meta.validate();

        assert!(issues
            .iter()
            .any(|i| i.field == "title" && i.severity == Severity::Medium));
    }

    #[test]
    fn test_meta_validation_good_title() {
        let meta = MetaTags {
            title: Some("This is a good page title with proper length".to_string()),
            description: Some("This is a well-crafted meta description that provides a clear summary of the page content for search engines.".to_string()),
            viewport: Some("width=device-width, initial-scale=1".to_string()),
            lang: Some("en".to_string()),
            canonical: Some("https://example.com/page".to_string()),
            ..Default::default()
        };
        let issues = meta.validate();

        // Should have no errors (High or Critical)
        assert!(!issues
            .iter()
            .any(|i| matches!(i.severity, Severity::High | Severity::Critical)));
    }

    fn all_kinds() -> Vec<MetaIssueKind> {
        use MetaIssueKind::*;
        vec![
            MissingTitle,
            TitleTooShort,
            TitleTooLong,
            MissingDescription,
            DescriptionTooShort,
            DescriptionTooLong,
            MissingViewport,
            MissingLang,
            MissingCanonical,
            InvalidTwitterCard,
            RelativeOgImage,
        ]
    }

    /// #406 guard: English wording has no German characters, German wording
    /// none of the English phrases of the same messages (#898).
    #[test]
    fn meta_texts_are_clean_in_both_languages() {
        let values = MetaIssueValues {
            length: Some(176),
            value: Some("summary_big".into()),
            allowed: Some("summary, summary_large_image".into()),
        };
        for kind in all_kinds() {
            let en = meta_issue_text(kind, &values, true);
            let de = meta_issue_text(kind, &values, false);
            assert!(!en.chars().any(|c| "äöüÄÖÜß".contains(c)), "{en}");
            for phrase in [
                "is too",
                "Missing",
                "chars",
                "recommended",
                "is not an",
                "Invalid",
            ] {
                assert!(!de.contains(phrase), "{phrase:?} in {de}");
            }
            assert_ne!(en, de);
        }
    }

    #[test]
    fn meta_message_is_the_english_rendering_of_kind_and_values() {
        let meta = MetaTags {
            description: Some("x".repeat(176)),
            ..Default::default()
        };
        let issue = meta
            .validate()
            .into_iter()
            .find(|i| i.field == "description")
            .unwrap();
        assert_eq!(
            issue.message,
            "Description is too long (176 chars, recommended: 120-160)"
        );
        assert_eq!(
            issue.localized_message(false),
            "Beschreibung ist zu lang (176 Zeichen, Empfehlung: 120-160)"
        );
    }

    #[test]
    fn test_has_essentials() {
        let meta = MetaTags {
            title: Some("Title".to_string()),
            description: Some("Description".to_string()),
            viewport: Some("width=device-width".to_string()),
            ..Default::default()
        };

        assert!(meta.has_essentials());
    }
}
