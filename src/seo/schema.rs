//! Structured data (Schema.org) detection
//!
//! Detects JSON-LD, Microdata, and RDFa structured data.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// Structured data analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredData {
    /// JSON-LD scripts found
    pub json_ld: Vec<JsonLdSchema>,
    /// Schema types detected
    pub types: Vec<SchemaType>,
    /// Has any structured data
    pub has_structured_data: bool,
    /// Rich snippets potential
    pub rich_snippets_potential: Vec<String>,
}

/// JSON-LD schema data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLdSchema {
    /// Schema @type
    pub schema_type: String,
    /// Raw JSON content
    pub content: serde_json::Value,
    /// Is valid JSON-LD
    pub is_valid: bool,
}

/// Known schema types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaType {
    Organization,
    LocalBusiness,
    Person,
    Article,
    BlogPosting,
    NewsArticle,
    Product,
    Offer,
    Event,
    Recipe,
    VideoObject,
    WebPage,
    WebSite,
    BreadcrumbList,
    FAQPage,
    HowTo,
    Review,
    AggregateRating,
    Other(String),
}

impl SchemaType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Organization" => SchemaType::Organization,
            "LocalBusiness" => SchemaType::LocalBusiness,
            "Person" => SchemaType::Person,
            "Article" => SchemaType::Article,
            "BlogPosting" => SchemaType::BlogPosting,
            "NewsArticle" => SchemaType::NewsArticle,
            "Product" => SchemaType::Product,
            "Offer" => SchemaType::Offer,
            "Event" => SchemaType::Event,
            "Recipe" => SchemaType::Recipe,
            "VideoObject" => SchemaType::VideoObject,
            "WebPage" => SchemaType::WebPage,
            "WebSite" => SchemaType::WebSite,
            "BreadcrumbList" => SchemaType::BreadcrumbList,
            "FAQPage" => SchemaType::FAQPage,
            "HowTo" => SchemaType::HowTo,
            "Review" => SchemaType::Review,
            "AggregateRating" => SchemaType::AggregateRating,
            other => SchemaType::Other(other.to_string()),
        }
    }

    pub fn rich_snippet_type(&self) -> Option<&'static str> {
        match self {
            SchemaType::Article | SchemaType::BlogPosting | SchemaType::NewsArticle => {
                Some("Article Rich Snippet")
            }
            SchemaType::Product => Some("Product Rich Snippet"),
            SchemaType::Recipe => Some("Recipe Rich Snippet"),
            SchemaType::Event => Some("Event Rich Snippet"),
            SchemaType::FAQPage => Some("FAQ Rich Snippet"),
            SchemaType::HowTo => Some("How-To Rich Snippet"),
            SchemaType::Review | SchemaType::AggregateRating => Some("Review Rich Snippet"),
            SchemaType::BreadcrumbList => Some("Breadcrumb Rich Snippet"),
            SchemaType::VideoObject => Some("Video Rich Snippet"),
            SchemaType::LocalBusiness => Some("Local Business Rich Snippet"),
            _ => None,
        }
    }
}

/// Detect structured data on a page
pub async fn detect_structured_data(page: &Page) -> Result<StructuredData> {
    info!("Detecting structured data...");

    let js_code = r#"
    (() => {
        const result = { jsonLd: [], microdata: false, rdfa: false };

        // Find JSON-LD scripts
        document.querySelectorAll('script[type="application/ld+json"]').forEach(script => {
            try {
                const content = JSON.parse(script.textContent);
                result.jsonLd.push(content);
            } catch (e) {
                result.jsonLd.push({ error: 'Invalid JSON', raw: script.textContent.substring(0, 200) });
            }
        });

        // Check for microdata
        result.microdata = document.querySelectorAll('[itemscope]').length > 0;

        // Check for RDFa
        result.rdfa = document.querySelectorAll('[typeof], [property]').length > 0;

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Structured data detection failed: {}", e)))?;

    let json_str = js_result
        .value()
        .and_then(|v| v.as_str())
        .unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let mut json_ld = Vec::new();
    let mut types = Vec::new();
    let mut rich_snippets_potential = Vec::new();

    // Parse JSON-LD schemas
    if let Some(schemas) = parsed["jsonLd"].as_array() {
        for schema in schemas {
            let is_valid = !schema.get("error").is_some();

            // Extract @type (can be string or array)
            let schema_types = extract_types(schema);

            for type_str in &schema_types {
                let schema_type = SchemaType::from_str(type_str);

                if let Some(rich_snippet) = schema_type.rich_snippet_type() {
                    if !rich_snippets_potential.contains(&rich_snippet.to_string()) {
                        rich_snippets_potential.push(rich_snippet.to_string());
                    }
                }

                if !types.contains(&schema_type) {
                    types.push(schema_type);
                }
            }

            json_ld.push(JsonLdSchema {
                schema_type: schema_types.first().cloned().unwrap_or_default(),
                content: schema.clone(),
                is_valid,
            });
        }
    }

    let has_structured_data = !json_ld.is_empty()
        || parsed["microdata"].as_bool().unwrap_or(false)
        || parsed["rdfa"].as_bool().unwrap_or(false);

    info!(
        "Structured data: {} JSON-LD schemas, {} types, {} rich snippet opportunities",
        json_ld.len(),
        types.len(),
        rich_snippets_potential.len()
    );

    Ok(StructuredData {
        json_ld,
        types,
        has_structured_data,
        rich_snippets_potential,
    })
}

fn extract_types(schema: &serde_json::Value) -> Vec<String> {
    let mut types = Vec::new();

    if let Some(type_str) = schema["@type"].as_str() {
        types.push(type_str.to_string());
    } else if let Some(type_arr) = schema["@type"].as_array() {
        for t in type_arr {
            if let Some(s) = t.as_str() {
                types.push(s.to_string());
            }
        }
    }

    // Also check @graph
    if let Some(graph) = schema["@graph"].as_array() {
        for item in graph {
            types.extend(extract_types(item));
        }
    }

    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_type_from_str() {
        assert_eq!(SchemaType::from_str("Article"), SchemaType::Article);
        assert_eq!(SchemaType::from_str("Product"), SchemaType::Product);
        assert!(matches!(SchemaType::from_str("CustomType"), SchemaType::Other(_)));
    }

    #[test]
    fn test_rich_snippet_type() {
        assert_eq!(
            SchemaType::Article.rich_snippet_type(),
            Some("Article Rich Snippet")
        );
        assert_eq!(
            SchemaType::Product.rich_snippet_type(),
            Some("Product Rich Snippet")
        );
        assert_eq!(SchemaType::Organization.rich_snippet_type(), None);
    }
}
