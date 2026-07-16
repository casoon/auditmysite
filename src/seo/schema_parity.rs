//! Conservative parity checks between visible page facts and JSON-LD values.
//!
//! Only semantically anchored, unambiguous visible values are compared. An
//! absent or ambiguous visible fact is reported as not evaluated, never as a
//! mismatch.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AuditError, Result};
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
    pub fn status_text(&self, en: bool) -> &'static str {
        match (self.status, en) {
            (ContentParityStatus::Match, true) => "Matches visible content",
            (ContentParityStatus::Match, false) => "Stimmt mit sichtbarem Inhalt überein",
            (ContentParityStatus::Mismatch, true) => "Mismatch",
            (ContentParityStatus::Mismatch, false) => "Abweichung",
            (ContentParityStatus::NotEvaluated, true) => "Not evaluated",
            (ContentParityStatus::NotEvaluated, false) => "Nicht geprüft",
        }
    }

    pub fn evidence_text(&self, en: bool) -> String {
        match self.status {
            ContentParityStatus::Match => {
                let value = self.schema_value.as_deref().unwrap_or("-");
                if en {
                    format!("Schema and visible content agree: {value}")
                } else {
                    format!("Schema und sichtbarer Inhalt stimmen überein: {value}")
                }
            }
            ContentParityStatus::Mismatch => {
                let schema = self.schema_value.as_deref().unwrap_or("-");
                let visible = self.visible_value.as_deref().unwrap_or("-");
                if en {
                    format!("Schema: {schema}; visible: {visible}")
                } else {
                    format!("Schema: {schema}; sichtbar: {visible}")
                }
            }
            ContentParityStatus::NotEvaluated => {
                if en {
                    "No single unambiguous visible value was found".to_string()
                } else {
                    "Kein einzelner eindeutiger sichtbarer Wert erkannt".to_string()
                }
            }
        }
    }
}

pub async fn extract_visible_schema_facts(page: &Page) -> Result<VisibleSchemaFacts> {
    let result = page
        .evaluate(
            r#"
            (() => {
                const clean = value => (value || '').replace(/\s+/g, ' ').trim().slice(0, 240);
                const unique = values => [...new Set(values.map(clean).filter(Boolean))].slice(0, 20);
                const values = selectors => unique(selectors.flatMap(selector =>
                    [...document.querySelectorAll(selector)].map(element =>
                        element.getAttribute('content') ||
                        element.getAttribute('datetime') ||
                        element.getAttribute('data-price') ||
                        element.textContent
                    )
                ));
                const priceValues = values([
                    '[itemprop="price"]', '[data-price]', '.price', '.product-price',
                    '[class*="price__"]', '[class*="product-price"]'
                ]);
                const currencyValues = values([
                    '[itemprop="priceCurrency"]', '[data-currency]', '.price-currency'
                ]);
                const availabilityValues = values([
                    '[itemprop="availability"]', '[data-availability]', '.availability',
                    '[class*="stock-status"]'
                ]);
                const authors = values([
                    '[rel="author"]', '[itemprop="author"]', '.author', '.byline',
                    '[class*="author-name"]'
                ]);
                const dates = values([
                    'time[datetime]', '[itemprop="datePublished"]', '[itemprop="datePosted"]',
                    '[itemprop="startDate"]'
                ]);
                const faqQuestions = unique([
                    ...document.querySelectorAll(
                        '[itemtype*="Question"] [itemprop="name"], details > summary, .faq h2, .faq h3, [class*="faq"] h2, [class*="faq"] h3'
                    )
                ].map(element => element.textContent));
                const faqAnswers = unique([
                    ...document.querySelectorAll(
                        '[itemtype*="Answer"] [itemprop="text"], details > :not(summary), .faq-answer, [class*="faq-answer"]'
                    )
                ].map(element => element.textContent));
                const breadcrumbRoot = document.querySelector(
                    'nav[aria-label*="breadcrumb" i], [itemtype*="BreadcrumbList"], .breadcrumb, [class*="breadcrumb"]'
                );
                const breadcrumbs = breadcrumbRoot ? unique(
                    [...breadcrumbRoot.querySelectorAll('a, [aria-current="page"], [itemprop="name"]')]
                        .map(element => element.textContent)
                ) : [];
                return JSON.stringify({
                    document_title: clean(document.title) || null,
                    h1: clean(document.querySelector('h1')?.textContent) || null,
                    prices: priceValues,
                    currencies: currencyValues,
                    availability: availabilityValues,
                    authors,
                    dates,
                    faq_questions: faqQuestions,
                    faq_answers: faqAnswers,
                    breadcrumbs
                });
            })()
            "#,
        )
        .await
        .map_err(|error| {
            AuditError::CdpError(format!("Visible structured-data facts failed: {error}"))
        })?;
    let json = result.value().and_then(Value::as_str).unwrap_or("{}");
    Ok(serde_json::from_str(json).unwrap_or_default())
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
