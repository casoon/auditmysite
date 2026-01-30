//! Meta tags extraction and validation
//!
//! Extracts and validates title, description, keywords, and other meta tags.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{AuditError, Result};

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
    /// Severity: "error", "warning", "info"
    pub severity: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl MetaTags {
    /// Validate meta tags and return issues
    pub fn validate(&self) -> Vec<MetaValidation> {
        let mut issues = Vec::new();

        // Title validation
        match &self.title {
            None => {
                issues.push(MetaValidation {
                    field: "title".to_string(),
                    message: "Missing page title".to_string(),
                    severity: "error".to_string(),
                    suggestion: Some("Add a <title> tag to the page".to_string()),
                });
            }
            Some(title) => {
                let len = title.len();
                if len < 30 {
                    issues.push(MetaValidation {
                        field: "title".to_string(),
                        message: format!("Title is too short ({} chars, recommended: 30-60)", len),
                        severity: "warning".to_string(),
                        suggestion: Some("Expand title to 30-60 characters".to_string()),
                    });
                } else if len > 60 {
                    issues.push(MetaValidation {
                        field: "title".to_string(),
                        message: format!("Title is too long ({} chars, recommended: 30-60)", len),
                        severity: "warning".to_string(),
                        suggestion: Some("Shorten title to under 60 characters".to_string()),
                    });
                }
            }
        }

        // Description validation
        match &self.description {
            None => {
                issues.push(MetaValidation {
                    field: "description".to_string(),
                    message: "Missing meta description".to_string(),
                    severity: "error".to_string(),
                    suggestion: Some("Add a meta description tag".to_string()),
                });
            }
            Some(desc) => {
                let len = desc.len();
                if len < 120 {
                    issues.push(MetaValidation {
                        field: "description".to_string(),
                        message: format!(
                            "Description is too short ({} chars, recommended: 120-160)",
                            len
                        ),
                        severity: "warning".to_string(),
                        suggestion: Some("Expand description to 120-160 characters".to_string()),
                    });
                } else if len > 160 {
                    issues.push(MetaValidation {
                        field: "description".to_string(),
                        message: format!(
                            "Description is too long ({} chars, recommended: 120-160)",
                            len
                        ),
                        severity: "warning".to_string(),
                        suggestion: Some("Shorten description to under 160 characters".to_string()),
                    });
                }
            }
        }

        // Viewport validation
        if self.viewport.is_none() {
            issues.push(MetaValidation {
                field: "viewport".to_string(),
                message: "Missing viewport meta tag".to_string(),
                severity: "error".to_string(),
                suggestion: Some(
                    "Add <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
                        .to_string(),
                ),
            });
        }

        // Language validation
        if self.lang.is_none() {
            issues.push(MetaValidation {
                field: "lang".to_string(),
                message: "Missing lang attribute on <html> element".to_string(),
                severity: "warning".to_string(),
                suggestion: Some("Add lang attribute: <html lang=\"en\">".to_string()),
            });
        }

        // Canonical validation
        if self.canonical.is_none() {
            issues.push(MetaValidation {
                field: "canonical".to_string(),
                message: "Missing canonical URL".to_string(),
                severity: "info".to_string(),
                suggestion: Some("Add <link rel=\"canonical\" href=\"...\">".to_string()),
            });
        }

        issues
    }

    /// Check if essential meta tags are present
    pub fn has_essentials(&self) -> bool {
        self.title.is_some() && self.description.is_some() && self.viewport.is_some()
    }
}

/// Extract meta tags from a page
pub async fn extract_meta_tags(page: &Page) -> Result<MetaTags> {
    info!("Extracting meta tags...");

    let js_code = r#"
    (() => {
        const result = {};

        // Title
        result.title = document.title || null;

        // Meta tags
        const getMeta = (name) => {
            const el = document.querySelector(`meta[name="${name}"], meta[property="${name}"]`);
            return el ? el.getAttribute('content') : null;
        };

        result.description = getMeta('description');
        result.keywords = getMeta('keywords');
        result.robots = getMeta('robots');
        result.author = getMeta('author');
        result.viewport = getMeta('viewport');

        // Charset
        const charset = document.querySelector('meta[charset]');
        result.charset = charset ? charset.getAttribute('charset') : null;

        // Canonical
        const canonical = document.querySelector('link[rel="canonical"]');
        result.canonical = canonical ? canonical.getAttribute('href') : null;

        // Language
        result.lang = document.documentElement.getAttribute('lang');

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Meta extraction failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let meta: MetaTags = serde_json::from_str(json_str).unwrap_or_else(|e| {
        warn!("Failed to parse meta tags JSON: {}", e);
        MetaTags::default()
    });

    info!(
        "Meta tags: title={}, description={}, viewport={}",
        meta.title.is_some(),
        meta.description.is_some(),
        meta.viewport.is_some()
    );

    Ok(meta)
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
            .any(|i| i.field == "title" && i.severity == "error"));
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
            .any(|i| i.field == "title" && i.severity == "warning"));
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

        // Should have no errors
        assert!(!issues.iter().any(|i| i.severity == "error"));
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
