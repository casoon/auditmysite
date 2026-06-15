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
    /// Whether the missing property is required or recommended for rich-result quality
    pub severity: SchemaIssueSeverity,
    /// Machine-readable issue key
    pub issue_type: String,
    /// Human-readable description
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaIssueSeverity {
    Required,
    Recommended,
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
    pub fn as_str(&self) -> &str {
        match self {
            Self::Organization => "Organization",
            Self::LocalBusiness => "LocalBusiness",
            Self::Person => "Person",
            Self::Article => "Article",
            Self::BlogPosting => "BlogPosting",
            Self::NewsArticle => "NewsArticle",
            Self::Product => "Product",
            Self::Offer => "Offer",
            Self::Event => "Event",
            Self::Recipe => "Recipe",
            Self::VideoObject => "VideoObject",
            Self::WebPage => "WebPage",
            Self::WebSite => "WebSite",
            Self::BreadcrumbList => "BreadcrumbList",
            Self::FAQPage => "FAQPage",
            Self::HowTo => "HowTo",
            Self::Review => "Review",
            Self::AggregateRating => "AggregateRating",
            Self::Other(s) => s,
        }
    }

    pub fn is_organization_like(&self) -> bool {
        matches!(self, Self::Organization | Self::LocalBusiness)
            || matches!(
                self.as_str(),
                "Airline"
                    | "Consortium"
                    | "Corporation"
                    | "EducationalOrganization"
                    | "FundingScheme"
                    | "GovernmentOrganization"
                    | "LibrarySystem"
                    | "LocalBusiness"
                    | "MedicalOrganization"
                    | "NGO"
                    | "NewsMediaOrganization"
                    | "OnlineBusiness"
                    | "PerformingGroup"
                    | "Project"
                    | "ResearchOrganization"
                    | "SearchRescueOrganization"
                    | "SportsOrganization"
                    | "WorkersUnion"
                    | "LegalService"
                    | "ProfessionalService"
            )
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
        .flat_map(validate_schema_properties)
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
        ("Product", &["name", "image", "offers"]),
        ("FAQPage", &["mainEntity"]),
        ("BreadcrumbList", &["itemListElement"]),
        ("LocalBusiness", &["name", "address"]),
        ("Organization", &["name", "url"]),
        ("Event", &["name", "startDate", "location"]),
        (
            "VideoObject",
            &["name", "description", "thumbnailUrl", "uploadDate"],
        ),
        (
            "Recipe",
            &["name", "image", "recipeIngredient", "recipeInstructions"],
        ),
        ("HowTo", &["name"]),
        ("Review", &["author"]),
    ];
    let recommended: &[(&str, &[&str])] = &[
        (
            "Article",
            &[
                "dateModified",
                "publisher",
                "description",
                "mainEntityOfPage",
            ],
        ),
        (
            "BlogPosting",
            &[
                "dateModified",
                "publisher",
                "description",
                "mainEntityOfPage",
            ],
        ),
        (
            "NewsArticle",
            &[
                "dateModified",
                "publisher",
                "description",
                "mainEntityOfPage",
            ],
        ),
        (
            "Product",
            &["description", "sku", "brand", "aggregateRating", "review"],
        ),
        ("FAQPage", &[]),
        ("BreadcrumbList", &[]),
        (
            "LocalBusiness",
            &["telephone", "url", "openingHours", "geo", "aggregateRating"],
        ),
        ("Organization", &["logo", "sameAs", "contactPoint"]),
        (
            "Event",
            &[
                "endDate",
                "image",
                "description",
                "offers",
                "performer",
                "eventStatus",
                "eventAttendanceMode",
            ],
        ),
        (
            "Recipe",
            &[
                "author",
                "datePublished",
                "prepTime",
                "cookTime",
                "totalTime",
                "nutrition",
                "aggregateRating",
                "video",
            ],
        ),
    ];

    let types_to_check: Vec<String> = if schema.schema_types.is_empty() {
        schema
            .schema_type
            .split('/')
            .next_back()
            .map(|s| vec![s.to_string()])
            .unwrap_or_default()
    } else {
        schema
            .schema_types
            .iter()
            .map(|t| t.split('/').next_back().unwrap_or(t).to_string())
            .collect()
    };

    let mut issues = Vec::new();
    for type_name in types_to_check {
        if let Some((_, props)) = required.iter().find(|(name, _)| *name == type_name) {
            for prop in *props {
                if !has_schema_property(&schema.content, prop) {
                    issues.push(SchemaIssue {
                        schema_type: type_name.to_string(),
                        severity: SchemaIssueSeverity::Required,
                        issue_type: format!("schema_missing_{}", prop),
                        message: format!(
                            "{}: Pflichtfeld \"{}\" fehlt im JSON-LD",
                            type_name, prop
                        ),
                    });
                }
            }
        }
        if let Some((_, props)) = recommended.iter().find(|(name, _)| *name == type_name) {
            for prop in *props {
                if !has_schema_property(&schema.content, prop) {
                    issues.push(SchemaIssue {
                        schema_type: type_name.to_string(),
                        severity: SchemaIssueSeverity::Recommended,
                        issue_type: format!("schema_recommended_missing_{}", prop),
                        message: format!(
                            "{}: Empfohlenes Feld \"{}\" fehlt im JSON-LD",
                            type_name, prop
                        ),
                    });
                }
            }
        }
    }
    issues
}

fn has_schema_property(content: &serde_json::Value, prop: &str) -> bool {
    match content.get(prop) {
        Some(serde_json::Value::Null) | None => false,
        Some(serde_json::Value::String(s)) => !s.trim().is_empty(),
        Some(serde_json::Value::Array(items)) => !items.is_empty(),
        Some(serde_json::Value::Object(map)) => !map.is_empty(),
        Some(_) => true,
    }
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

    #[test]
    fn organization_like_accepts_schema_subtypes() {
        assert!(SchemaType::Organization.is_organization_like());
        assert!(SchemaType::LocalBusiness.is_organization_like());
        assert!(SchemaType::Other("LegalService".into()).is_organization_like());
        assert!(SchemaType::Other("MedicalOrganization".into()).is_organization_like());
        assert!(!SchemaType::Person.is_organization_like());
        assert!(!SchemaType::WebSite.is_organization_like());
    }

    #[test]
    fn product_schema_requires_offers_for_rich_result_eligibility() {
        let schema = JsonLdSchema {
            schema_type: "Product".to_string(),
            schema_types: vec!["Product".to_string()],
            content: serde_json::json!({
                "@type": "Product",
                "name": "Audit",
                "image": "https://example.com/audit.png"
            }),
            is_valid: true,
        };

        let issues = validate_schema_properties(&schema);

        assert!(issues.iter().any(|issue| {
            issue.schema_type == "Product"
                && issue.severity == SchemaIssueSeverity::Required
                && issue.issue_type == "schema_missing_offers"
        }));
    }

    #[test]
    fn article_schema_reports_recommended_properties_separately() {
        let schema = JsonLdSchema {
            schema_type: "Article".to_string(),
            schema_types: vec!["Article".to_string()],
            content: serde_json::json!({
                "@type": "Article",
                "headline": "Accessibility audit",
                "image": "https://example.com/article.png",
                "author": { "@type": "Person", "name": "Ada" },
                "datePublished": "2026-01-01"
            }),
            is_valid: true,
        };

        let issues = validate_schema_properties(&schema);

        assert!(!issues
            .iter()
            .any(|issue| issue.severity == SchemaIssueSeverity::Required));
        assert!(issues.iter().any(|issue| {
            issue.severity == SchemaIssueSeverity::Recommended
                && issue.issue_type == "schema_recommended_missing_dateModified"
        }));
    }

    #[test]
    fn empty_schema_properties_count_as_missing() {
        let schema = JsonLdSchema {
            schema_type: "BreadcrumbList".to_string(),
            schema_types: vec!["BreadcrumbList".to_string()],
            content: serde_json::json!({
                "@type": "BreadcrumbList",
                "itemListElement": []
            }),
            is_valid: true,
        };

        let issues = validate_schema_properties(&schema);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "schema_missing_itemListElement");
        assert_eq!(issues[0].severity, SchemaIssueSeverity::Required);
    }
}
