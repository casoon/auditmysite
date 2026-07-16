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
    /// Detected schema types associated with rich-result features.
    /// Kept under its legacy field name for JSON compatibility.
    pub rich_snippets_potential: Vec<String>,
    /// Validation issues: missing required properties per schema block
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_issues: Vec<SchemaIssue>,
    /// Feature-specific required/recommended property assessment per node.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rule_assessments: Vec<crate::seo::schema_rules::SchemaRuleAssessment>,
    /// Fit between visible page intent and the detected primary schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit_assessment: Option<crate::seo::schema_fit::SchemaFitAssessment>,
    /// Conservative comparisons between visible content and JSON-LD values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_parity: Vec<crate::seo::schema_parity::ContentParityAssessment>,
    /// Collection-only input for page-type derivation; full visible values are
    /// intentionally not duplicated in the public JSON report.
    #[serde(skip)]
    pub(crate) visible_facts: crate::seo::schema_parity::VisibleSchemaFacts,
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
    /// Canonical-English description for language-independent JSON output.
    /// PDF output re-derives localized text via [`schema_issue_text`].
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaIssueSeverity {
    Required,
    Recommended,
}

/// Localize a schema issue from its stable machine-readable key.
pub fn schema_issue_text(issue: &SchemaIssue, en: bool) -> String {
    let schema_type = if issue.schema_type.is_empty() {
        "JSON-LD"
    } else {
        issue.schema_type.as_str()
    };

    match issue.issue_type.as_str() {
        "jsonld_parse_error" => {
            if en {
                "The JSON-LD block contains invalid JSON and cannot be evaluated.".to_string()
            } else {
                "Der JSON-LD-Block enthält ungültiges JSON und kann nicht ausgewertet werden."
                    .to_string()
            }
        }
        "jsonld_invalid_root" => {
            if en {
                "The JSON-LD root must be an object or an array of objects.".to_string()
            } else {
                "Die JSON-LD-Wurzel muss ein Objekt oder eine Liste von Objekten sein.".to_string()
            }
        }
        "jsonld_missing_context" => {
            if en {
                "No schema.org @context was found; schema terms cannot be interpreted reliably."
                    .to_string()
            } else {
                "Kein schema.org-@context gefunden; die Schema-Begriffe sind nicht zuverlässig interpretierbar."
                    .to_string()
            }
        }
        "jsonld_missing_type" => {
            if en {
                "A JSON-LD node has no @type and cannot be assigned to a schema type.".to_string()
            } else {
                "Ein JSON-LD-Knoten hat keinen @type und kann keinem Schema-Typ zugeordnet werden."
                    .to_string()
            }
        }
        "jsonld_graph_not_array" => {
            if en {
                "The @graph value must be an array of JSON-LD nodes.".to_string()
            } else {
                "Der Wert von @graph muss eine Liste von JSON-LD-Knoten sein.".to_string()
            }
        }
        "jsonld_empty_document" => {
            if en {
                "The JSON-LD block contains no nodes.".to_string()
            } else {
                "Der JSON-LD-Block enthält keine Knoten.".to_string()
            }
        }
        issue_type => {
            if let Some(prop) = issue_type.strip_prefix("schema_recommended_missing_") {
                if en {
                    format!("{schema_type}: recommended property \"{prop}\" is missing")
                } else {
                    format!("{schema_type}: Empfohlenes Feld \"{prop}\" fehlt")
                }
            } else if let Some(prop) = issue_type.strip_prefix("schema_missing_") {
                if en {
                    format!("{schema_type}: required property \"{prop}\" is missing")
                } else {
                    format!("{schema_type}: Pflichtfeld \"{prop}\" fehlt")
                }
            } else {
                issue.message.clone()
            }
        }
    }
}

/// Short localized label for report tables.
pub fn schema_issue_label(issue: &SchemaIssue, en: bool) -> &'static str {
    match issue.issue_type.as_str() {
        "jsonld_parse_error" => {
            if en {
                "JSON syntax"
            } else {
                "JSON-Syntax"
            }
        }
        "jsonld_invalid_root" => {
            if en {
                "Root structure"
            } else {
                "Wurzelstruktur"
            }
        }
        "jsonld_missing_context" => {
            if en {
                "Schema context"
            } else {
                "Schema-Kontext"
            }
        }
        "jsonld_missing_type" => {
            if en {
                "Schema type"
            } else {
                "Schema-Typ"
            }
        }
        "jsonld_graph_not_array" => "@graph",
        "jsonld_empty_document" => {
            if en {
                "Empty block"
            } else {
                "Leerer Block"
            }
        }
        _ => "Schema.org",
    }
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
    CollectionPage,
    ItemList,
    ProfilePage,
    JobPosting,
    SoftwareApplication,
    WebApplication,
    MobileApplication,
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
            "CollectionPage" => SchemaType::CollectionPage,
            "ItemList" => SchemaType::ItemList,
            "ProfilePage" => SchemaType::ProfilePage,
            "JobPosting" => SchemaType::JobPosting,
            "SoftwareApplication" => SchemaType::SoftwareApplication,
            "WebApplication" => SchemaType::WebApplication,
            "MobileApplication" => SchemaType::MobileApplication,
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
            Self::CollectionPage => "CollectionPage",
            Self::ItemList => "ItemList",
            Self::ProfilePage => "ProfilePage",
            Self::JobPosting => "JobPosting",
            Self::SoftwareApplication => "SoftwareApplication",
            Self::WebApplication => "WebApplication",
            Self::MobileApplication => "MobileApplication",
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
            // Google removed HowTo rich results; keep the schema in the
            // inventory without presenting it as a current search feature.
            SchemaType::HowTo => None,
            SchemaType::Review | SchemaType::AggregateRating => Some("Review Rich Snippet"),
            SchemaType::BreadcrumbList => Some("Breadcrumb Rich Snippet"),
            SchemaType::VideoObject => Some("Video Rich Snippet"),
            SchemaType::ProfilePage => Some("Profile Page Rich Snippet"),
            SchemaType::JobPosting => Some("Job Posting Rich Snippet"),
            SchemaType::SoftwareApplication
            | SchemaType::WebApplication
            | SchemaType::MobileApplication => Some("Software App Rich Snippet"),
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

        // Return raw script contents. Rust owns parsing and normalization so
        // top-level arrays, @graph and parse failures share one testable path.
        document.querySelectorAll('script[type="application/ld+json"]').forEach(script => {
            result.jsonLd.push(script.textContent || '');
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
    let scripts: Vec<String> = parsed["jsonLd"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let mut structured_data = analyze_structured_data_payloads(
        &scripts,
        parsed["microdata"].as_bool().unwrap_or(false),
        parsed["rdfa"].as_bool().unwrap_or(false),
    );
    structured_data.visible_facts =
        crate::seo::schema_parity::extract_visible_schema_facts(page).await?;
    structured_data.content_parity = crate::seo::schema_parity::assess_content_parity(
        &structured_data.json_ld,
        &structured_data.visible_facts,
    );

    info!(
        "Structured data: {} JSON-LD schemas, {} types, {} rich snippet opportunities",
        structured_data
            .json_ld
            .iter()
            .filter(|schema| schema.is_valid && !schema.schema_types.is_empty())
            .count(),
        structured_data.types.len(),
        structured_data.rich_snippets_potential.len()
    );

    Ok(structured_data)
}

pub(crate) fn analyze_structured_data_payloads(
    scripts: &[String],
    has_microdata: bool,
    has_rdfa: bool,
) -> StructuredData {
    let mut json_ld = Vec::new();
    let mut schema_issues = Vec::new();

    for (block_index, raw) in scripts.iter().enumerate() {
        match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(value) => {
                normalize_json_ld_document(&value, block_index, &mut json_ld, &mut schema_issues)
            }
            Err(error) => {
                json_ld.push(JsonLdSchema {
                    schema_type: String::new(),
                    schema_types: Vec::new(),
                    content: serde_json::json!({
                        "raw": raw.chars().take(200).collect::<String>()
                    }),
                    is_valid: false,
                });
                schema_issues.push(SchemaIssue {
                    schema_type: "JSON-LD".to_string(),
                    severity: SchemaIssueSeverity::Required,
                    issue_type: "jsonld_parse_error".to_string(),
                    message: format!(
                        "JSON-LD block {} contains invalid JSON: {}",
                        block_index + 1,
                        error
                    ),
                });
            }
        }
    }

    let mut types = Vec::new();
    let mut rich_snippets_potential = Vec::new();
    for schema in json_ld.iter().filter(|schema| schema.is_valid) {
        for type_str in &schema.schema_types {
            let schema_type: SchemaType = type_str
                .parse()
                .unwrap_or(SchemaType::Other(type_str.to_string()));
            if let Some(rich_snippet) = schema_type.rich_snippet_type() {
                if !rich_snippets_potential
                    .iter()
                    .any(|item| item == rich_snippet)
                {
                    rich_snippets_potential.push(rich_snippet.to_string());
                }
            }
            if !types.contains(&schema_type) {
                types.push(schema_type);
            }
        }
    }

    let rule_assessments = build_rule_assessments(
        &json_ld,
        crate::seo::schema_rules::ProductRuleContext::Indeterminate,
    );
    schema_issues.extend(rule_assessments.iter().flat_map(rule_assessment_issues));

    StructuredData {
        json_ld,
        types,
        has_structured_data: !scripts.is_empty() || has_microdata || has_rdfa,
        rich_snippets_potential,
        schema_issues,
        rule_assessments,
        fit_assessment: None,
        content_parity: Vec::new(),
        visible_facts: crate::seo::schema_parity::VisibleSchemaFacts::default(),
    }
}

pub(crate) fn refresh_rule_assessments(
    structured_data: &mut StructuredData,
    product_context: crate::seo::schema_rules::ProductRuleContext,
) {
    structured_data
        .schema_issues
        .retain(|issue| !issue.issue_type.starts_with("schema_rule_"));
    structured_data.rule_assessments =
        build_rule_assessments(&structured_data.json_ld, product_context);
    structured_data.schema_issues.extend(
        structured_data
            .rule_assessments
            .iter()
            .flat_map(rule_assessment_issues),
    );
}

fn build_rule_assessments(
    schemas: &[JsonLdSchema],
    product_context: crate::seo::schema_rules::ProductRuleContext,
) -> Vec<crate::seo::schema_rules::SchemaRuleAssessment> {
    schemas
        .iter()
        .enumerate()
        .filter(|(_, schema)| schema.is_valid)
        .flat_map(|(node_index, schema)| {
            schema.schema_types.iter().flat_map(move |schema_type| {
                crate::seo::schema_rules::assess_node(
                    node_index,
                    schema_type,
                    &schema.content,
                    product_context,
                )
            })
        })
        .collect()
}

fn rule_assessment_issues(
    assessment: &crate::seo::schema_rules::SchemaRuleAssessment,
) -> Vec<SchemaIssue> {
    if assessment.requirement_status
        != crate::seo::schema_rules::SchemaRequirementStatus::MissingRequiredProperties
    {
        return Vec::new();
    }

    assessment
        .missing_required
        .iter()
        .map(|property| SchemaIssue {
            schema_type: assessment.schema_type.clone(),
            severity: SchemaIssueSeverity::Required,
            issue_type: format!(
                "schema_rule_missing_{}_{}",
                assessment.feature.key(),
                property
                    .chars()
                    .map(|character| {
                        if character.is_ascii_alphanumeric() {
                            character
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>()
                    .trim_matches('_')
            ),
            message: format!(
                "{}: required property condition \"{}\" is not met for {}",
                assessment.schema_type,
                property,
                assessment.feature.label(true)
            ),
        })
        .collect()
}

fn normalize_json_ld_document(
    value: &serde_json::Value,
    block_index: usize,
    schemas: &mut Vec<JsonLdSchema>,
    issues: &mut Vec<SchemaIssue>,
) {
    match value {
        serde_json::Value::Array(items) => {
            if items.is_empty() {
                issues.push(structural_issue(
                    "jsonld_empty_document",
                    format!("JSON-LD block {} contains no nodes", block_index + 1),
                ));
                return;
            }
            for item in items {
                normalize_json_ld_root(item, false, schemas, issues);
            }
        }
        serde_json::Value::Object(_) => {
            normalize_json_ld_root(value, false, schemas, issues);
        }
        _ => issues.push(structural_issue(
            "jsonld_invalid_root",
            format!(
                "JSON-LD block {} has a non-object root value",
                block_index + 1
            ),
        )),
    }
}

fn normalize_json_ld_root(
    value: &serde_json::Value,
    context_inherited: bool,
    schemas: &mut Vec<JsonLdSchema>,
    issues: &mut Vec<SchemaIssue>,
) {
    let Some(object) = value.as_object() else {
        issues.push(structural_issue(
            "jsonld_invalid_root",
            "JSON-LD array entries must be objects".to_string(),
        ));
        return;
    };

    let has_context = context_inherited || has_schema_org_context(object.get("@context"));
    if !has_context {
        issues.push(structural_issue(
            "jsonld_missing_context",
            "No schema.org @context found for JSON-LD node".to_string(),
        ));
    }

    let root_types = extract_types(value);
    let graph = object.get("@graph");

    if !root_types.is_empty() {
        schemas.push(JsonLdSchema {
            schema_type: root_types.first().cloned().unwrap_or_default(),
            schema_types: root_types,
            content: value.clone(),
            is_valid: true,
        });
    } else if graph.is_none() {
        schemas.push(JsonLdSchema {
            schema_type: String::new(),
            schema_types: Vec::new(),
            content: value.clone(),
            is_valid: true,
        });
        issues.push(structural_issue(
            "jsonld_missing_type",
            "JSON-LD node has no @type".to_string(),
        ));
    }

    if let Some(graph) = graph {
        let Some(items) = graph.as_array() else {
            issues.push(structural_issue(
                "jsonld_graph_not_array",
                "JSON-LD @graph value is not an array".to_string(),
            ));
            return;
        };
        if items.is_empty() {
            issues.push(structural_issue(
                "jsonld_empty_document",
                "JSON-LD @graph contains no nodes".to_string(),
            ));
        }
        for item in items {
            normalize_json_ld_root(item, has_context, schemas, issues);
        }
    }
}

fn structural_issue(issue_type: &str, message: String) -> SchemaIssue {
    SchemaIssue {
        schema_type: "JSON-LD".to_string(),
        severity: SchemaIssueSeverity::Required,
        issue_type: issue_type.to_string(),
        message,
    }
}

fn has_schema_org_context(context: Option<&serde_json::Value>) -> bool {
    match context {
        Some(serde_json::Value::String(value)) => {
            matches!(
                value.trim_end_matches('/'),
                "https://schema.org" | "http://schema.org"
            )
        }
        Some(serde_json::Value::Array(values)) => values
            .iter()
            .any(|value| has_schema_org_context(Some(value))),
        Some(serde_json::Value::Object(values)) => values
            .get("@vocab")
            .is_some_and(|value| has_schema_org_context(Some(value))),
        _ => false,
    }
}

fn extract_types(schema: &serde_json::Value) -> Vec<String> {
    let mut types = Vec::new();

    if let Some(type_str) = schema["@type"].as_str() {
        if let Some(normalized) = normalize_schema_type(type_str) {
            types.push(normalized);
        }
    } else if let Some(type_arr) = schema["@type"].as_array() {
        for t in type_arr {
            if let Some(s) = t.as_str() {
                if let Some(normalized) = normalize_schema_type(s) {
                    types.push(normalized);
                }
            }
        }
    }

    types
}

fn normalize_schema_type(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    Some(
        trimmed
            .rsplit(['/', '#'])
            .next()
            .unwrap_or(trimmed)
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn astro_structured_data_exports_work_inline_and_as_graph_without_runtime_dependency() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/astro_structured_data_components.json"
        ))
        .unwrap();
        let components = fixture["components"].as_array().unwrap();
        let inline = fixture["inline"].as_array().unwrap();
        assert_eq!(components.len(), 18);
        assert_eq!(inline.len(), 17); // SchemaGraph is the graph container.

        for item in inline {
            let payload = item["payload"].to_string();
            let data = analyze_structured_data_payloads(&[payload], false, false);
            assert!(
                !data.json_ld.is_empty(),
                "{} inline fixture",
                item["component"]
            );
            assert!(!data
                .schema_issues
                .iter()
                .any(|issue| issue.issue_type.starts_with("jsonld_")));
        }

        let graph = fixture["useGraph"].to_string();
        let data = analyze_structured_data_payloads(&[graph], false, false);
        assert_eq!(data.json_ld.len(), inline.len());
        assert!(!data
            .schema_issues
            .iter()
            .any(|issue| issue.issue_type.starts_with("jsonld_")));
    }

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
        assert_eq!(SchemaType::HowTo.rich_snippet_type(), None);
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
    fn product_schema_accepts_review_instead_of_offer() {
        let data = analyze_structured_data_payloads(
            &[serde_json::json!({
                "@context": "https://schema.org",
                "@type": "Product",
                "name": "Audit",
                "review": {"@type": "Review", "author": {"@type": "Person", "name": "Ada"}}
            })
            .to_string()],
            false,
            false,
        );

        assert!(!data.schema_issues.iter().any(|issue| issue
            .issue_type
            .starts_with("schema_rule_missing_product_snippet")));
    }

    #[test]
    fn article_schema_reports_recommended_properties_separately() {
        let data = analyze_structured_data_payloads(
            &[serde_json::json!({
                "@context": "https://schema.org",
                "@type": "Article",
                "headline": "Accessibility audit",
                "image": "https://example.com/article.png",
                "author": { "@type": "Person", "name": "Ada" },
                "datePublished": "2026-01-01"
            })
            .to_string()],
            false,
            false,
        );

        assert!(!data
            .schema_issues
            .iter()
            .any(|issue| issue.severity == SchemaIssueSeverity::Required));
        assert!(data.rule_assessments[0]
            .missing_recommended
            .contains(&"dateModified".to_string()));
    }

    #[test]
    fn empty_schema_properties_count_as_missing() {
        let data = analyze_structured_data_payloads(
            &[serde_json::json!({
                "@context": "https://schema.org",
                "@type": "BreadcrumbList",
                "itemListElement": []
            })
            .to_string()],
            false,
            false,
        );

        assert!(data.schema_issues.iter().any(|issue| {
            issue.severity == SchemaIssueSeverity::Required
                && issue
                    .issue_type
                    .starts_with("schema_rule_missing_breadcrumb_itemListElement")
        }));
    }

    #[test]
    fn graph_nodes_are_normalized_once_without_container_false_positives() {
        let data = analyze_structured_data_payloads(
            &[serde_json::json!({
                "@context": "https://schema.org",
                "@graph": [
                    {
                        "@type": "Product",
                        "name": "Gadget",
                        "image": "https://example.com/gadget.jpg",
                        "offers": {"@type": "Offer", "price": "29.99", "priceCurrency": "EUR"}
                    },
                    {
                        "@type": "BreadcrumbList",
                        "itemListElement": [
                            {"@type": "ListItem", "position": 1, "name": "Home", "item": "https://example.com"},
                            {"@type": "ListItem", "position": 2, "name": "Products"}
                        ]
                    }
                ]
            })
            .to_string()],
            false,
            false,
        );

        assert_eq!(data.json_ld.len(), 2);
        assert_eq!(data.json_ld[0].schema_type, "Product");
        assert_eq!(data.json_ld[1].schema_type, "BreadcrumbList");
        assert!(!data
            .schema_issues
            .iter()
            .any(|issue| issue.severity == SchemaIssueSeverity::Required));
    }

    #[test]
    fn top_level_array_is_expanded_into_individual_nodes() {
        let data = analyze_structured_data_payloads(
            &[serde_json::json!([
                {
                    "@context": "https://schema.org",
                    "@type": "WebSite",
                    "name": "Example",
                    "url": "https://example.com"
                },
                {
                    "@context": "https://schema.org",
                    "@type": "WebPage",
                    "name": "About",
                    "url": "https://example.com/about"
                }
            ])
            .to_string()],
            false,
            false,
        );

        assert_eq!(data.json_ld.len(), 2);
        assert_eq!(data.types, vec![SchemaType::WebSite, SchemaType::WebPage]);
    }

    #[test]
    fn invalid_json_is_preserved_as_a_visible_structural_issue() {
        let data = analyze_structured_data_payloads(
            &[r#"{"@context":"https://schema.org","@type":"Product""#.to_string()],
            false,
            false,
        );

        assert!(data.has_structured_data);
        assert_eq!(data.json_ld.len(), 1);
        assert!(!data.json_ld[0].is_valid);
        assert!(data
            .schema_issues
            .iter()
            .any(|issue| issue.issue_type == "jsonld_parse_error"));
        assert!(data.schema_issues[0].message.starts_with("JSON-LD block 1"));
    }

    #[test]
    fn stored_schema_issue_messages_remain_canonical_english() {
        let data = analyze_structured_data_payloads(
            &[
                serde_json::json!({
                    "@context": "https://schema.org",
                    "@type": "Product",
                    "name": "Incomplete product"
                })
                .to_string(),
                r#"{"@type":"Product""#.to_string(),
            ],
            false,
            false,
        );

        assert!(data
            .schema_issues
            .iter()
            .all(|issue| !issue.message.contains("Pflichtfeld")
                && !issue.message.contains("Empfohlenes Feld")
                && !issue.message.contains("ungültig")));
    }

    #[test]
    fn full_schema_type_iri_is_normalized_to_known_type() {
        let data = analyze_structured_data_payloads(
            &[serde_json::json!({
                "@context": "https://schema.org",
                "@type": "https://schema.org/Product",
                "name": "Gadget",
                "image": "https://example.com/gadget.jpg",
                "offers": {"@type": "Offer", "price": "29.99", "priceCurrency": "EUR"}
            })
            .to_string()],
            false,
            false,
        );

        assert_eq!(data.json_ld[0].schema_type, "Product");
        assert_eq!(data.types, vec![SchemaType::Product]);
    }

    #[test]
    fn missing_context_and_type_are_reported_separately() {
        let data = analyze_structured_data_payloads(
            &[serde_json::json!({"name": "Untyped node"}).to_string()],
            false,
            false,
        );

        let issue_types: Vec<_> = data
            .schema_issues
            .iter()
            .map(|issue| issue.issue_type.as_str())
            .collect();
        assert!(issue_types.contains(&"jsonld_missing_context"));
        assert!(issue_types.contains(&"jsonld_missing_type"));
    }
}
