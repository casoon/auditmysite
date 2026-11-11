//! Conservative parity checks between visible page facts and JSON-LD values.
//!
//! Only semantically anchored, unambiguous visible values are compared. An
//! absent or ambiguous visible fact is reported as not evaluated, never as a
//! mismatch.

use chromiumoxide::Page;
use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::seo::schema::JsonLdSchema;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisibleSchemaFacts {
    pub document_title: Option<String>,
    pub h1: Option<String>,
    #[serde(default)]
    pub prices: Vec<String>,
    #[serde(default)]
    pub currencies: Vec<String>,
    #[serde(default)]
    pub availability: Vec<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub dates: Vec<String>,
    #[serde(default)]
    pub faq_questions: Vec<String>,
    #[serde(default)]
    pub faq_answers: Vec<String>,
    #[serde(default)]
    pub breadcrumbs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentParityStatus {
    Match,
    Mismatch,
    NotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentParityAssessment {
    pub node_index: usize,
    pub schema_type: String,
    pub property: String,
    pub status: ContentParityStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_value: Option<String>,
    /// Short canonical-English evidence suitable for JSON and report tables.
    pub evidence: String,
}

impl ContentParityAssessment {
    /// The `seo-schema-parity-status-*` message of `locales/{en,de}/seo-issues.ftl`.
    pub fn status_text(&self, en: bool) -> String {
        let key = match self.status {
            ContentParityStatus::Match => "seo-schema-parity-status-match",
            ContentParityStatus::Mismatch => "seo-schema-parity-status-mismatch",
            ContentParityStatus::NotEvaluated => "seo-schema-parity-status-not-evaluated",
        };
        crate::i18n::text(key, en)
    }

    /// The `seo-schema-parity-evidence-*` message of
    /// `locales/{en,de}/seo-issues.ftl`; a missing value is shown as `-`.
    pub fn evidence_text(&self, en: bool) -> String {
        let value = |v: &Option<String>| FluentValue::from(v.as_deref().unwrap_or("-").to_string());
        match self.status {
            ContentParityStatus::Match => crate::i18n::text_args(
                "seo-schema-parity-evidence-match",
                &[("value", value(&self.schema_value))],
                en,
            ),
            ContentParityStatus::Mismatch => crate::i18n::text_args(
                "seo-schema-parity-evidence-mismatch",
                &[
                    ("schema", value(&self.schema_value)),
                    ("visible", value(&self.visible_value)),
                ],
                en,
            ),
            ContentParityStatus::NotEvaluated => {
                crate::i18n::text("seo-schema-parity-evidence-not-evaluated", en)
            }
        }
    }
}

/// Visible text that structured data claims (names, prices, dates, …).
pub(crate) static VISIBLE_FACTS: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "seo.schema_visible_facts",
        module: "seo",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: false,
    };

/// The visible facts, or the reason they could not be read. Without them
/// content parity is not assessed at all (#777).
pub async fn extract_visible_schema_facts(
    page: &Page,
) -> crate::audit::collection::Collected<VisibleSchemaFacts> {
    crate::audit::collection::evaluate_js(page, include_str!("js/visible_schema_facts.js")).await
}

/// Counter-cases of [`VISIBLE_FACTS`] (#777).
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_contract::<VisibleSchemaFacts>(serde_json::json!({}));
}

pub fn assess_content_parity(
    schemas: &[JsonLdSchema],
    facts: &VisibleSchemaFacts,
) -> Vec<ContentParityAssessment> {
    let mut results = Vec::new();
    for (node_index, schema) in schemas
        .iter()
        .enumerate()
        .filter(|(_, schema)| schema.is_valid)
    {
        for schema_type in &schema.schema_types {
            match schema_type.as_str() {
                "Product" | "SoftwareApplication" | "WebApplication" | "MobileApplication" => {
                    compare_title(
                        node_index,
                        schema_type,
                        &schema.content,
                        facts,
                        &mut results,
                    );
                    compare_single_visible(
                        node_index,
                        schema_type,
                        "offers.price",
                        schema_values(&schema.content, "offers.price"),
                        &facts.prices,
                        &mut results,
                    );
                    compare_single_visible(
                        node_index,
                        schema_type,
                        "offers.priceCurrency",
                        schema_values(&schema.content, "offers.priceCurrency"),
                        &facts.currencies,
                        &mut results,
                    );
                    compare_single_visible(
                        node_index,
                        schema_type,
                        "offers.availability",
                        schema_values(&schema.content, "offers.availability"),
                        &facts.availability,
                        &mut results,
                    );
                }
                "Article" | "BlogPosting" | "NewsArticle" | "Recipe" => {
                    compare_title(
                        node_index,
                        schema_type,
                        &schema.content,
                        facts,
                        &mut results,
                    );
                    compare_single_visible(
                        node_index,
                        schema_type,
                        "author.name",
                        schema_values(&schema.content, "author.name"),
                        &facts.authors,
                        &mut results,
                    );
                    compare_single_visible(
                        node_index,
                        schema_type,
                        "datePublished",
                        schema_values(&schema.content, "datePublished"),
                        &facts.dates,
                        &mut results,
                    );
                }
                "Event" => {
                    compare_title(
                        node_index,
                        schema_type,
                        &schema.content,
                        facts,
                        &mut results,
                    );
                    compare_single_visible(
                        node_index,
                        schema_type,
                        "startDate",
                        schema_values(&schema.content, "startDate"),
                        &facts.dates,
                        &mut results,
                    );
                }
                "JobPosting" => {
                    compare_property_to_title(
                        node_index,
                        schema_type,
                        "title",
                        &schema.content,
                        facts,
                        &mut results,
                    );
                    compare_single_visible(
                        node_index,
                        schema_type,
                        "datePosted",
                        schema_values(&schema.content, "datePosted"),
                        &facts.dates,
                        &mut results,
                    );
                }
                "FAQPage" => {
                    compare_lists(
                        node_index,
                        schema_type,
                        "mainEntity.name",
                        schema_values(&schema.content, "mainEntity.name"),
                        &facts.faq_questions,
                        &mut results,
                    );
                    compare_lists(
                        node_index,
                        schema_type,
                        "mainEntity.acceptedAnswer.text",
                        schema_values(&schema.content, "mainEntity.acceptedAnswer.text"),
                        &facts.faq_answers,
                        &mut results,
                    );
                }
                "BreadcrumbList" => compare_lists(
                    node_index,
                    schema_type,
                    "itemListElement.name",
                    schema_values(&schema.content, "itemListElement.name"),
                    &facts.breadcrumbs,
                    &mut results,
                ),
                "WebPage" | "ProfilePage" | "Person" => compare_title(
                    node_index,
                    schema_type,
                    &schema.content,
                    facts,
                    &mut results,
                ),
                _ => {}
            }
        }
    }
    results
}

fn compare_title(
    node_index: usize,
    schema_type: &str,
    content: &Value,
    facts: &VisibleSchemaFacts,
    output: &mut Vec<ContentParityAssessment>,
) {
    let property = if schema_values(content, "headline").is_empty() {
        "name"
    } else {
        "headline"
    };
    compare_property_to_title(node_index, schema_type, property, content, facts, output);
}

fn compare_property_to_title(
    node_index: usize,
    schema_type: &str,
    property: &str,
    content: &Value,
    facts: &VisibleSchemaFacts,
    output: &mut Vec<ContentParityAssessment>,
) {
    let visible = facts
        .h1
        .as_ref()
        .or(facts.document_title.as_ref())
        .cloned()
        .into_iter()
        .collect::<Vec<_>>();
    compare_single_visible(
        node_index,
        schema_type,
        property,
        schema_values(content, property),
        &visible,
        output,
    );
}

fn compare_single_visible(
    node_index: usize,
    schema_type: &str,
    property: &str,
    schema: Vec<String>,
    visible: &[String],
    output: &mut Vec<ContentParityAssessment>,
) {
    let status = if schema.is_empty() || visible.len() != 1 {
        ContentParityStatus::NotEvaluated
    } else if schema
        .iter()
        .any(|schema_value| values_match(schema_value, &visible[0]))
    {
        ContentParityStatus::Match
    } else {
        ContentParityStatus::Mismatch
    };
    output.push(ContentParityAssessment {
        node_index,
        schema_type: schema_type.to_string(),
        property: property.to_string(),
        status,
        schema_value: schema.first().cloned(),
        visible_value: visible.first().cloned(),
        evidence: evidence(status, &schema, visible),
    });
}

fn compare_lists(
    node_index: usize,
    schema_type: &str,
    property: &str,
    schema: Vec<String>,
    visible: &[String],
    output: &mut Vec<ContentParityAssessment>,
) {
    let status = if schema.is_empty() || visible.is_empty() {
        ContentParityStatus::NotEvaluated
    } else if schema.len() == visible.len()
        && schema
            .iter()
            .zip(visible)
            .all(|(schema, visible)| values_match(schema, visible))
    {
        ContentParityStatus::Match
    } else {
        ContentParityStatus::Mismatch
    };
    output.push(ContentParityAssessment {
        node_index,
        schema_type: schema_type.to_string(),
        property: property.to_string(),
        status,
        schema_value: schema.first().cloned(),
        visible_value: visible.first().cloned(),
        evidence: evidence(status, &schema, visible),
    });
}

fn evidence(status: ContentParityStatus, schema: &[String], visible: &[String]) -> String {
    let text = match status {
        ContentParityStatus::Match => format!(
            "Schema and visible content agree: {}",
            schema.first().cloned().unwrap_or_default()
        ),
        ContentParityStatus::Mismatch => format!(
            "Schema: {}; visible: {}",
            schema.first().cloned().unwrap_or_default(),
            visible.first().cloned().unwrap_or_default()
        ),
        ContentParityStatus::NotEvaluated => {
            "Not evaluated because no single unambiguous visible value was found".to_string()
        }
    };
    text.chars().take(180).collect()
}

fn values_match(left: &str, right: &str) -> bool {
    let left = normalize(left);
    let right = normalize(right);
    left == right
        || (left.len() >= 8 && right.contains(&left))
        || (right.len() >= 8 && left.contains(&right))
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(value)
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn schema_values(value: &Value, path: &str) -> Vec<String> {
    fn collect(value: &Value, parts: &[&str], output: &mut Vec<String>) {
        if parts.is_empty() {
            match value {
                Value::Array(values) => {
                    for value in values {
                        collect(value, parts, output);
                    }
                }
                Value::String(value) => output.push(value.clone()),
                Value::Number(value) => output.push(value.to_string()),
                _ => {}
            }
            return;
        }
        match value {
            Value::Array(values) => {
                for value in values {
                    collect(value, parts, output);
                }
            }
            Value::Object(values) => {
                if let Some(value) = values.get(parts[0]) {
                    collect(value, &parts[1..], output);
                }
            }
            _ => {}
        }
    }

    let mut output = Vec::new();
    collect(value, &path.split('.').collect::<Vec<_>>(), &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema(schema_type: &str, content: Value) -> JsonLdSchema {
        JsonLdSchema {
            schema_type: schema_type.to_string(),
            schema_types: vec![schema_type.to_string()],
            content,
            is_valid: true,
        }
    }

    #[test]
    fn title_parity_detects_match_and_mismatch() {
        let facts = VisibleSchemaFacts {
            h1: Some("Visible product".to_string()),
            ..Default::default()
        };
        let matching = assess_content_parity(
            &[schema(
                "Product",
                serde_json::json!({"name":"Visible product"}),
            )],
            &facts,
        );
        assert_eq!(matching[0].status, ContentParityStatus::Match);

        let mismatch = assess_content_parity(
            &[schema(
                "Product",
                serde_json::json!({"name":"Other product"}),
            )],
            &facts,
        );
        assert_eq!(mismatch[0].status, ContentParityStatus::Mismatch);
    }

    #[test]
    fn ambiguous_visible_prices_are_not_called_a_mismatch() {
        let facts = VisibleSchemaFacts {
            prices: vec!["10".to_string(), "20".to_string()],
            ..Default::default()
        };
        let results = assess_content_parity(
            &[schema(
                "Product",
                serde_json::json!({"name":"Product","offers":{"price":"10"}}),
            )],
            &facts,
        );
        let price = results
            .iter()
            .find(|result| result.property == "offers.price")
            .unwrap();
        assert_eq!(price.status, ContentParityStatus::NotEvaluated);
    }
}
