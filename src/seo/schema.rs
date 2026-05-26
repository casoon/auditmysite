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
    /// Validation issues: missing required properties per schema block
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_issues: Vec<SchemaIssue>,
}

/// A required-property validation issue for a JSON-LD block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaIssue {
    /// The @type of the affected schema block
    pub schema_type: String,
    /// Machine-readable issue key
    pub issue_type: String,
    /// Human-readable description
    pub message: String,
}

/// JSON-LD schema data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLdSchema {
    /// Schema @type
    pub schema_type: String,
    /// All schema @type values found on the root object
    #[serde(default)]
    pub schema_types: Vec<String>,
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

impl std::str::FromStr for SchemaType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
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
        })
    }
}

impl SchemaType {
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

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let mut json_ld = Vec::new();
    let mut types = Vec::new();
    let mut rich_snippets_potential = Vec::new();

    // Parse JSON-LD schemas
    if let Some(schemas) = parsed["jsonLd"].as_array() {
        for schema in schemas {
            let is_valid = schema.get("error").is_none();

            // Extract @type (can be string or array)
            let schema_types = extract_types(schema);

            for type_str in &schema_types {
                let schema_type: SchemaType = type_str
                    .parse()
                    .unwrap_or(SchemaType::Other(type_str.to_string()));

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
                schema_types,
                content: schema.clone(),
                is_valid,
            });

            // Expand @graph items into separate entries so property validation
            // can check each typed item individually
            if is_valid {
                if let Some(graph) = schema["@graph"].as_array() {
                    for graph_item in graph {
                        let item_types: Vec<String> = if let Some(s) = graph_item["@type"].as_str()
                        {
                            vec![s.to_string()]
                        } else if let Some(arr) = graph_item["@type"].as_array() {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        } else {
                            vec![]
                        };
                        if item_types.is_empty() {
                            continue;
                        }
                        json_ld.push(JsonLdSchema {
                            schema_type: item_types.first().cloned().unwrap_or_default(),
                            schema_types: item_types,
                            content: graph_item.clone(),
                            is_valid: true,
                        });
                    }
                }
            }
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

    let schema_issues = json_ld
        .iter()
        .flat_map(|s| validate_schema_properties(s))
        .collect();

    Ok(StructuredData {
        json_ld,
        types,
        has_structured_data,
        rich_snippets_potential,
        schema_issues,
    })
}

fn validate_schema_properties(schema: &JsonLdSchema) -> Vec<SchemaIssue> {
    if !schema.is_valid {
        return vec![];
    }

    // Required properties per schema type (Google rich-result spec)
    let required: &[(&str, &[&str])] = &[
        ("Article", &["headline", "image", "author", "datePublished"]),
        (
            "BlogPosting",
            &["headline", "image", "author", "datePublished"],
        ),
        (
            "NewsArticle",
            &["headline", "image", "author", "datePublished"],
        ),
        ("Product", &["name", "image"]),
        ("Event", &["name", "startDate"]),
        (
            "VideoObject",
            &["name", "description", "thumbnailUrl", "uploadDate"],
        ),
        ("Recipe", &["name", "image"]),
        ("HowTo", &["name"]),
        ("Review", &["author"]),
    ];

    let types_to_check: Vec<String> = if schema.schema_types.is_empty() {
        schema
            .schema_type
            .split('/')
            .last()
            .map(|s| vec![s.to_string()])
            .unwrap_or_default()
    } else {
        schema
            .schema_types
            .iter()
            .map(|t| t.split('/').last().unwrap_or(t).to_string())
            .collect()
    };

    let mut issues = Vec::new();
    for (type_name, props) in required {
        if types_to_check.iter().any(|t| t == type_name) {
            for prop in *props {
                if schema.content[prop].is_null() {
                    issues.push(SchemaIssue {
                        schema_type: type_name.to_string(),
                        issue_type: format!("schema_missing_{}", prop),
                        message: format!(
                            "{}: Pflichtfeld \"{}\" fehlt im JSON-LD",
                            type_name, prop
                        ),
                    });
                }
            }
        }
    }
    issues
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
        assert_eq!(
            "Article".parse::<SchemaType>().unwrap(),
            SchemaType::Article
        );
        assert_eq!(
            "Product".parse::<SchemaType>().unwrap(),
            SchemaType::Product
        );
        assert!(matches!(
            "CustomType".parse::<SchemaType>().unwrap(),
            SchemaType::Other(_)
        ));
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
