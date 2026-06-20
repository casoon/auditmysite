//! Commerce / shop audit — schema-driven commercial-completeness signals.
//!
//! Derive-only analysis (no CDP): reads the JSON-LD already collected by the SEO
//! module (`discoverability.seo.structured_data`) and reports whether a product
//! page exposes the commercial information shoppers (and search/AI) expect —
//! price, availability/delivery, shipping, returns and reviews — as
//! machine-readable structured data.
//!
//! Self-gating: produces `Some(..)` only on pages that carry `Product`/`Offer`
//! schema; otherwise `None` (skipped on landing/editorial pages). Absence of a
//! signal means "not exposed as structured data", which is itself an SEO/AI
//! visibility gap — it does NOT assert the shop lacks the information on-page.
//!
//! Localization (#406): the stored struct is canonical English; findings carry a
//! `kind` enum and `commerce_finding_text(kind, en)` is the single text source.

pub mod module;

pub use module::CommerceModule;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::seo::StructuredData;
use crate::taxonomy::Severity;

/// Schema-derived commercial completeness for a product page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommerceAnalysis {
    /// Price + currency + validity from `Offer`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PriceInfo>,
    /// `Offer.availability` (schema.org URL or token, e.g. `InStock`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<String>,
    /// Human delivery time from `shippingDetails.deliveryTime`, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_time: Option<String>,
    /// Whether `OfferShippingDetails` is present (and free-shipping if derivable).
    pub shipping: ShippingInfo,
    /// Whether a `MerchantReturnPolicy` is present (and its window, if given).
    pub returns: ReturnsInfo,
    /// `AggregateRating` rating value + review count, if present.
    pub reviews: ReviewInfo,
    /// Findings for missing/incomplete commercial structured data.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<CommerceFinding>,
    /// Share of expected commercial signals exposed (0–100).
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceInfo {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// `priceValidUntil` raw value (ISO date), if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ShippingInfo {
    /// `OfferShippingDetails` present anywhere on the offer/product.
    pub has_shipping_details: bool,
    /// A shipping rate value was found (`shippingRate.value`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_rate: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReturnsInfo {
    /// `MerchantReturnPolicy` / `hasMerchantReturnPolicy` present.
    pub has_return_policy: bool,
    /// `merchantReturnDays` value, if given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_days: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReviewInfo {
    /// `aggregateRating.ratingValue`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating_value: Option<String>,
    /// `aggregateRating.reviewCount` / `ratingCount`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_count: Option<String>,
}

/// Canonical finding kinds. Text is derived via [`commerce_finding_text`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommerceFindingKind {
    MissingPrice,
    MissingAvailability,
    MissingShippingDetails,
    MissingReturnPolicy,
    MissingReviews,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommerceFinding {
    pub kind: CommerceFindingKind,
    pub severity: Severity,
    /// Canonical English message (#406); PDF re-derives via the run locale.
    pub message: String,
}

const EXPECTED_SIGNALS: u32 = 5;

/// Analyze commercial completeness from already-collected structured data.
/// Returns `None` when the page carries no `Product`/`Offer` schema.
pub fn analyze_commerce(structured_data: &StructuredData) -> Option<CommerceAnalysis> {
    let product = structured_data
        .json_ld
        .iter()
        .find(|s| schema_is(&s.content, "Product") || schema_is(&s.content, "Offer"))?;
    let content = &product.content;

    // The Offer may be the root (schema_type Offer) or nested under `offers`
    // (single object or array). Shipping/returns can live on offer or product.
    let offer = first_offer(content);
    let offer_or_product = offer.unwrap_or(content);

    let price = extract_price(offer_or_product, content);
    let availability = str_field(offer_or_product, "availability").map(normalize_schema_token);

    let shipping = extract_shipping(offer_or_product, content);
    let delivery_time = extract_delivery_time(offer_or_product, content);
    let returns = extract_returns(offer_or_product, content);
    let reviews = extract_reviews(content);

    let mut findings = Vec::new();
    let mut present = 0u32;

    if price.is_some() {
        present += 1;
    } else {
        findings.push(finding(CommerceFindingKind::MissingPrice, Severity::High));
    }
    if availability.is_some() {
        present += 1;
    } else {
        findings.push(finding(
            CommerceFindingKind::MissingAvailability,
            Severity::Medium,
        ));
    }
    if shipping.has_shipping_details {
        present += 1;
    } else {
        findings.push(finding(
            CommerceFindingKind::MissingShippingDetails,
            Severity::Medium,
        ));
    }
    if returns.has_return_policy {
        present += 1;
    } else {
        findings.push(finding(
            CommerceFindingKind::MissingReturnPolicy,
            Severity::Medium,
        ));
    }
    if reviews.rating_value.is_some() {
        present += 1;
    } else {
        findings.push(finding(CommerceFindingKind::MissingReviews, Severity::Low));
    }

    let score = (present * 100) / EXPECTED_SIGNALS;

    Some(CommerceAnalysis {
        price,
        availability,
        delivery_time,
        shipping,
        returns,
        reviews,
        findings,
        score,
    })
}

fn finding(kind: CommerceFindingKind, severity: Severity) -> CommerceFinding {
    CommerceFinding {
        kind,
        severity,
        message: commerce_finding_text(kind, true),
    }
}

/// Single source of truth for finding wording (#406). Analysis bakes English
/// (`en = true`); the PDF presentation calls it with the run locale.
pub fn commerce_finding_text(kind: CommerceFindingKind, en: bool) -> String {
    use CommerceFindingKind::*;
    match (kind, en) {
        (MissingPrice, true) => {
            "No price exposed in the product's structured data (Offer.price).".into()
        }
        (MissingPrice, false) => {
            "Kein Preis in den strukturierten Produktdaten ausgewiesen (Offer.price).".into()
        }
        (MissingAvailability, true) => {
            "No availability exposed in the structured data (Offer.availability).".into()
        }
        (MissingAvailability, false) => {
            "Keine Verfügbarkeit in den strukturierten Daten ausgewiesen (Offer.availability).".into()
        }
        (MissingShippingDetails, true) => {
            "No shipping details in the structured data (OfferShippingDetails) — shipping cost and delivery time are not machine-readable.".into()
        }
        (MissingShippingDetails, false) => {
            "Keine Versandangaben in den strukturierten Daten (OfferShippingDetails) — Versandkosten und Lieferzeit sind nicht maschinenlesbar.".into()
        }
        (MissingReturnPolicy, true) => {
            "No return policy in the structured data (MerchantReturnPolicy).".into()
        }
        (MissingReturnPolicy, false) => {
            "Keine Rückgaberichtlinie in den strukturierten Daten (MerchantReturnPolicy).".into()
        }
        (MissingReviews, true) => {
            "No aggregate rating in the structured data (aggregateRating) — review stars cannot appear in search results.".into()
        }
        (MissingReviews, false) => {
            "Keine Sammelbewertung in den strukturierten Daten (aggregateRating) — Bewertungssterne können nicht in Suchergebnissen erscheinen.".into()
        }
    }
}

// ─── JSON-LD navigation helpers ────────────────────────────────────────────

/// True if the value's `@type` equals or contains `wanted` (handles string and
/// array `@type`).
fn schema_is(v: &Value, wanted: &str) -> bool {
    match v.get("@type") {
        Some(Value::String(s)) => s == wanted,
        Some(Value::Array(arr)) => arr.iter().any(|t| t.as_str() == Some(wanted)),
        _ => false,
    }
}

/// The first `Offer` object: the `offers` field (object or array element), else
/// the root itself if it is an Offer.
fn first_offer(content: &Value) -> Option<&Value> {
    match content.get("offers") {
        Some(Value::Object(_)) => content.get("offers"),
        Some(Value::Array(arr)) => arr.first(),
        _ => {
            if schema_is(content, "Offer") {
                Some(content)
            } else {
                None
            }
        }
    }
}

fn extract_price(offer: &Value, product: &Value) -> Option<PriceInfo> {
    let value = value_to_string(offer.get("price")).or_else(|| {
        // Some shops use priceSpecification.price.
        offer
            .get("priceSpecification")
            .and_then(|ps| value_to_string(ps.get("price")))
    })?;
    let currency = str_field(offer, "priceCurrency")
        .or_else(|| str_field(product, "priceCurrency"))
        .or_else(|| {
            offer
                .get("priceSpecification")
                .and_then(|ps| str_field(ps, "priceCurrency"))
        });
    let valid_until = str_field(offer, "priceValidUntil");
    Some(PriceInfo {
        value,
        currency,
        valid_until,
    })
}

fn extract_shipping(offer: &Value, product: &Value) -> ShippingInfo {
    let details = offer
        .get("shippingDetails")
        .or_else(|| product.get("shippingDetails"));
    match details {
        Some(d) => ShippingInfo {
            has_shipping_details: true,
            shipping_rate: d
                .get("shippingRate")
                .and_then(|r| value_to_string(r.get("value"))),
        },
        None => ShippingInfo::default(),
    }
}

fn extract_delivery_time(offer: &Value, product: &Value) -> Option<String> {
    let details = offer
        .get("shippingDetails")
        .or_else(|| product.get("shippingDetails"))?;
    let dt = details.get("deliveryTime")?;
    // deliveryTime is usually a ShippingDeliveryTime object; surface a compact
    // human hint if a transit-time bound is given, else just note its presence.
    let transit = dt
        .get("transitTime")
        .or_else(|| dt.get("handlingTime"))
        .and_then(|t| t.get("maxValue").or_else(|| t.get("minValue")))
        .and_then(value_to_string_owned);
    Some(transit.unwrap_or_else(|| "specified".to_string()))
}

fn extract_returns(offer: &Value, product: &Value) -> ReturnsInfo {
    let policy = offer
        .get("hasMerchantReturnPolicy")
        .or_else(|| product.get("hasMerchantReturnPolicy"))
        .or_else(|| {
            // Some shops emit a standalone MerchantReturnPolicy node.
            if schema_is(product, "MerchantReturnPolicy") {
                Some(product)
            } else {
                None
            }
        });
    match policy {
        Some(p) => ReturnsInfo {
            has_return_policy: true,
            return_days: value_to_string(p.get("merchantReturnDays")),
        },
        None => ReturnsInfo::default(),
    }
}

fn extract_reviews(product: &Value) -> ReviewInfo {
    let agg = product.get("aggregateRating");
    match agg {
        Some(a) => ReviewInfo {
            rating_value: value_to_string(a.get("ratingValue")),
            review_count: value_to_string(a.get("reviewCount"))
                .or_else(|| value_to_string(a.get("ratingCount"))),
        },
        None => ReviewInfo::default(),
    }
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
}

/// Stringify a scalar JSON value (string or number); `None` for other shapes.
fn value_to_string(v: Option<&Value>) -> Option<String> {
    v.and_then(value_to_string_owned)
}

fn value_to_string_owned(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.trim().is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// `https://schema.org/InStock` → `InStock`; leaves bare tokens untouched.
fn normalize_schema_token(s: String) -> String {
    s.rsplit('/').next().unwrap_or(&s).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seo::schema::JsonLdSchema;

    fn structured(content: Value) -> StructuredData {
        StructuredData {
            json_ld: vec![JsonLdSchema {
                schema_type: content
                    .get("@type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
                schema_types: vec![],
                content,
                is_valid: true,
            }],
            types: vec![],
            has_structured_data: true,
            rich_snippets_potential: vec![],
            schema_issues: vec![],
        }
    }

    #[test]
    fn non_product_page_returns_none() {
        let sd = structured(serde_json::json!({"@type": "Article", "headline": "x"}));
        assert!(analyze_commerce(&sd).is_none());
    }

    #[test]
    fn full_product_scores_100_no_findings() {
        let sd = structured(serde_json::json!({
            "@type": "Product",
            "name": "Widget",
            "aggregateRating": {"ratingValue": "4.6", "reviewCount": "120"},
            "offers": {
                "@type": "Offer",
                "price": "19.99",
                "priceCurrency": "EUR",
                "availability": "https://schema.org/InStock",
                "shippingDetails": {"@type": "OfferShippingDetails", "shippingRate": {"value": "3.90"}},
                "hasMerchantReturnPolicy": {"@type": "MerchantReturnPolicy", "merchantReturnDays": "30"}
            }
        }));
        let c = analyze_commerce(&sd).expect("product");
        assert_eq!(c.score, 100);
        assert!(c.findings.is_empty());
        assert_eq!(c.price.unwrap().value, "19.99");
        assert_eq!(c.availability.as_deref(), Some("InStock"));
        assert!(c.shipping.has_shipping_details);
        assert_eq!(c.shipping.shipping_rate.as_deref(), Some("3.90"));
        assert!(c.returns.has_return_policy);
        assert_eq!(c.returns.return_days.as_deref(), Some("30"));
        assert_eq!(c.reviews.rating_value.as_deref(), Some("4.6"));
    }

    #[test]
    fn bare_product_flags_all_gaps() {
        let sd = structured(serde_json::json!({"@type": "Product", "name": "Bare"}));
        let c = analyze_commerce(&sd).expect("product");
        assert_eq!(c.score, 0);
        assert_eq!(c.findings.len(), 5);
    }

    #[test]
    fn offers_as_array_is_handled() {
        let sd = structured(serde_json::json!({
            "@type": "Product",
            "offers": [{"@type": "Offer", "price": "5", "priceCurrency": "EUR"}]
        }));
        let c = analyze_commerce(&sd).expect("product");
        assert_eq!(c.price.unwrap().value, "5");
    }

    #[test]
    fn english_finding_text_has_no_german_umlauts() {
        for kind in [
            CommerceFindingKind::MissingPrice,
            CommerceFindingKind::MissingAvailability,
            CommerceFindingKind::MissingShippingDetails,
            CommerceFindingKind::MissingReturnPolicy,
            CommerceFindingKind::MissingReviews,
        ] {
            let t = commerce_finding_text(kind, true);
            assert!(
                !t.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "EN text has umlaut: {t}"
            );
        }
    }
}
