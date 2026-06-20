//! Commerce / shop audit.
//!
//! Derive-only analysis (no CDP). Two signal groups, both gated to shop context
//! (Product/Offer schema present, or an e-commerce tech stack detected) so it
//! never fires on landing/editorial pages:
//!
//! - **Product completeness** (slice 1): from the JSON-LD the SEO module already
//!   collected — price, availability/delivery, shipping, returns, reviews.
//! - **Mandatory pages** (slice 2): whether the page links to the legally/UX
//!   expected pages (Impressum, AGB, Widerruf/Retoure, Versand, Zahlungsarten,
//!   Kontakt), detected from the screen-reader link inventory's anchor texts.
//!   The batch layer aggregates these per-page booleans into a site-wide view.
//!
//! Localization (#406): stored struct is canonical English; findings carry a
//! `kind` enum and `commerce_finding_text(kind, en)` is the single text source.
//! Honest wording: absence means "not exposed as structured data" / "not linked
//! from this page", never a claim of legal (non-)compliance.

pub mod module;

pub use module::CommerceModule;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::seo::StructuredData;
use crate::taxonomy::Severity;

/// Commerce signals for one shop page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommerceAnalysis {
    /// Coarse commerce page type (foundation for page-type-specific heuristics).
    pub page_kind: CommercePageKind,
    /// Product structured-data completeness; `None` when the page carries no
    /// `Product`/`Offer` schema (e.g. a category or info page of a shop).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<ProductCommerce>,
    /// Which mandatory/trust pages this page links to.
    pub trust_pages: TrustPages,
    /// Page-type-gated conversion signals (payment methods, free-shipping
    /// threshold, guest checkout).
    pub conversion: ConversionSignals,
    /// Findings across all groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<CommerceFinding>,
}

/// Presence-based conversion signals detected from the page's visible text.
/// Presence only — never a judgment of effectiveness or "honesty".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConversionSignals {
    /// Payment methods named on the page (canonical labels, deduplicated).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub payment_methods: Vec<String>,
    /// A free-shipping threshold ("versandkostenfrei ab …") is mentioned.
    pub free_shipping_threshold: bool,
    /// Guest checkout: `Some(true/false)` only on cart/checkout pages where it is
    /// meaningful; `None` elsewhere (not applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guest_checkout: Option<bool>,
}

/// Coarse classification of a shop page, from URL path + product schema.
/// Foundation for page-type-specific commerce heuristics (cart/checkout/category).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercePageKind {
    ProductDetail,
    Category,
    Cart,
    Checkout,
    #[default]
    Other,
}

/// Classify a shop page from its URL path and whether it carries Product schema.
/// Checkout/cart are matched before product/category so a `/checkout` URL that
/// still embeds product data is classified by its funnel stage.
pub fn detect_page_kind(url: &str, has_product_schema: bool) -> CommercePageKind {
    // Path including the leading slash (so "/kategorie/" needles match), host stripped.
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let path = match after_scheme.find('/') {
        Some(i) => after_scheme[i..].to_lowercase(),
        None => String::new(),
    };
    let has = |needles: &[&str]| needles.iter().any(|n| path.contains(n));

    if has(&["checkout", "kasse", "bestellung", "zur-kasse"]) {
        CommercePageKind::Checkout
    } else if has(&["/cart", "warenkorb", "/basket", "/cart/"]) || path.ends_with("cart") {
        CommercePageKind::Cart
    } else if has_product_schema || has(&["/product/", "/produkt/", "/p/", "/dp/", "/artikel/"]) {
        CommercePageKind::ProductDetail
    } else if has(&[
        "/category/",
        "/kategorie/",
        "/collection",
        "/collections/",
        "/c/",
        "/shop/",
        "/kategorien/",
    ]) {
        CommercePageKind::Category
    } else {
        CommercePageKind::Other
    }
}

/// Product structured-data completeness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductCommerce {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PriceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_time: Option<String>,
    pub shipping: ShippingInfo,
    pub returns: ReturnsInfo,
    pub reviews: ReviewInfo,
    /// Share of the 5 expected product signals exposed (0–100).
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceInfo {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ShippingInfo {
    pub has_shipping_details: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_rate: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReturnsInfo {
    pub has_return_policy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_days: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReviewInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_count: Option<String>,
}

/// Which mandatory/trust pages the page links to (by anchor text).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrustPages {
    pub impressum: bool,
    pub agb: bool,
    pub widerruf: bool,
    pub versand: bool,
    pub zahlungsarten: bool,
    pub kontakt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommerceFindingKind {
    // Product structured data.
    MissingPrice,
    MissingAvailability,
    MissingShippingDetails,
    MissingReturnPolicy,
    MissingReviews,
    // Mandatory / trust pages.
    MissingImpressumLink,
    MissingAgbLink,
    MissingWiderrufLink,
    MissingShippingPageLink,
    MissingPaymentLink,
    MissingContactLink,
    // Conversion (page-type-gated).
    NoPaymentMethodsVisible,
    NoGuestCheckout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommerceFinding {
    pub kind: CommerceFindingKind,
    pub severity: Severity,
    pub message: String,
}

/// Site-wide commerce roll-up across an audited batch (slice 2b).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommerceSiteSummary {
    /// Trust pages linked on at least one audited page (union).
    pub trust_pages_linked: TrustPages,
    /// Mandatory/trust categories not linked anywhere in the audited set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_site_wide: Vec<CommerceFindingKind>,
    /// Number of audited pages carrying Product schema.
    pub product_pages: usize,
}

/// Aggregate per-page commerce results into a site-wide roll-up. Returns `None`
/// when no audited page produced commerce data (i.e. not a shop).
pub fn aggregate_site_commerce<'a>(
    commerces: impl Iterator<Item = &'a CommerceAnalysis>,
) -> Option<CommerceSiteSummary> {
    use CommerceFindingKind::*;
    let mut u = TrustPages::default();
    let mut product_pages = 0usize;
    let mut any = false;
    for c in commerces {
        any = true;
        if c.product.is_some() {
            product_pages += 1;
        }
        u.impressum |= c.trust_pages.impressum;
        u.agb |= c.trust_pages.agb;
        u.widerruf |= c.trust_pages.widerruf;
        u.versand |= c.trust_pages.versand;
        u.zahlungsarten |= c.trust_pages.zahlungsarten;
        u.kontakt |= c.trust_pages.kontakt;
    }
    if !any {
        return None;
    }
    let mut missing = Vec::new();
    if !u.impressum {
        missing.push(MissingImpressumLink);
    }
    if !u.widerruf {
        missing.push(MissingWiderrufLink);
    }
    if !u.agb {
        missing.push(MissingAgbLink);
    }
    if !u.versand {
        missing.push(MissingShippingPageLink);
    }
    if !u.zahlungsarten {
        missing.push(MissingPaymentLink);
    }
    if !u.kontakt {
        missing.push(MissingContactLink);
    }
    Some(CommerceSiteSummary {
        trust_pages_linked: u,
        missing_site_wide: missing,
        product_pages,
    })
}

const EXPECTED_PRODUCT_SIGNALS: u32 = 5;

/// Analyze commerce signals. `anchor_texts` are the page's link anchor texts
/// (from the screen-reader link inventory); `is_ecommerce_stack` is true when a
/// shop system (Shopify/WooCommerce/…) was detected. Returns `None` for pages
/// with neither product schema nor a shop stack (non-shop pages).
pub fn analyze_commerce(
    url: &str,
    structured_data: &StructuredData,
    anchor_texts: &[String],
    page_texts: &[String],
    is_ecommerce_stack: bool,
) -> Option<CommerceAnalysis> {
    let product = analyze_product(structured_data);
    if product.is_none() && !is_ecommerce_stack {
        return None;
    }

    let page_kind = detect_page_kind(url, product.is_some());
    let trust_pages = detect_trust_pages(anchor_texts);
    let conversion = detect_conversion(page_texts, page_kind);

    let mut findings = Vec::new();
    if let Some(p) = &product {
        push_product_findings(p, &mut findings);
    }
    push_trust_findings(&trust_pages, &mut findings);
    push_conversion_findings(&conversion, page_kind, &mut findings);

    Some(CommerceAnalysis {
        page_kind,
        product,
        trust_pages,
        conversion,
        findings,
    })
}

/// Canonical payment-method labels mapped from their detection keywords.
const PAYMENT_METHODS: &[(&str, &[&str])] = &[
    ("PayPal", &["paypal"]),
    ("Klarna", &["klarna"]),
    ("Invoice", &["rechnung", "invoice"]),
    ("Direct debit", &["lastschrift", "sepa", "direct debit"]),
    (
        "Credit card",
        &["kreditkarte", "credit card", "visa", "mastercard", "amex"],
    ),
    ("Apple Pay", &["apple pay"]),
    ("Google Pay", &["google pay"]),
    ("Amazon Pay", &["amazon pay"]),
    ("Sofort", &["sofort", "sofortüberweisung"]),
    ("giropay", &["giropay"]),
];

const FREE_SHIPPING_MARKERS: &[&str] = &[
    "versandkostenfrei ab",
    "gratisversand ab",
    "kostenloser versand ab",
    "kostenlose lieferung ab",
    "free shipping over",
    "free shipping on orders over",
    "free delivery over",
];

const GUEST_CHECKOUT_MARKERS: &[&str] = &[
    "als gast",
    "gast-bestellung",
    "gastbestellung",
    "ohne registrierung",
    "ohne anmeldung",
    "ohne kundenkonto",
    "ohne konto",
    "guest checkout",
    "checkout as guest",
    "continue as guest",
];

fn detect_conversion(page_texts: &[String], page_kind: CommercePageKind) -> ConversionSignals {
    let lower: Vec<String> = page_texts.iter().map(|t| t.to_lowercase()).collect();
    let corpus_has = |needle: &str| lower.iter().any(|t| t.contains(needle));

    let mut payment_methods: Vec<String> = PAYMENT_METHODS
        .iter()
        .filter(|(_, kws)| kws.iter().any(|kw| corpus_has(kw)))
        .map(|(label, _)| label.to_string())
        .collect();
    payment_methods.dedup();

    let free_shipping_threshold = FREE_SHIPPING_MARKERS.iter().any(|m| corpus_has(m));

    let guest_checkout = match page_kind {
        CommercePageKind::Cart | CommercePageKind::Checkout => {
            Some(GUEST_CHECKOUT_MARKERS.iter().any(|m| corpus_has(m)))
        }
        _ => None,
    };

    ConversionSignals {
        payment_methods,
        free_shipping_threshold,
        guest_checkout,
    }
}

fn push_conversion_findings(
    c: &ConversionSignals,
    page_kind: CommercePageKind,
    out: &mut Vec<CommerceFinding>,
) {
    use CommerceFindingKind::*;
    // Only flag where it's a confident expectation: payment methods on the
    // cart/checkout funnel, guest checkout on the checkout page.
    if c.payment_methods.is_empty()
        && matches!(
            page_kind,
            CommercePageKind::Cart | CommercePageKind::Checkout
        )
    {
        out.push(finding(NoPaymentMethodsVisible, Severity::Medium));
    }
    if c.guest_checkout == Some(false) && page_kind == CommercePageKind::Checkout {
        out.push(finding(NoGuestCheckout, Severity::Medium));
    }
}

fn analyze_product(structured_data: &StructuredData) -> Option<ProductCommerce> {
    let product = structured_data
        .json_ld
        .iter()
        .find(|s| schema_is(&s.content, "Product") || schema_is(&s.content, "Offer"))?;
    let content = &product.content;
    let offer = first_offer(content);
    let offer_or_product = offer.unwrap_or(content);

    let price = extract_price(offer_or_product, content);
    let availability = str_field(offer_or_product, "availability").map(normalize_schema_token);
    let shipping = extract_shipping(offer_or_product, content);
    let delivery_time = extract_delivery_time(offer_or_product, content);
    let returns = extract_returns(offer_or_product, content);
    let reviews = extract_reviews(content);

    let present = [
        price.is_some(),
        availability.is_some(),
        shipping.has_shipping_details,
        returns.has_return_policy,
        reviews.rating_value.is_some(),
    ]
    .iter()
    .filter(|p| **p)
    .count() as u32;
    let score = (present * 100) / EXPECTED_PRODUCT_SIGNALS;

    Some(ProductCommerce {
        price,
        availability,
        delivery_time,
        shipping,
        returns,
        reviews,
        score,
    })
}

fn push_product_findings(p: &ProductCommerce, out: &mut Vec<CommerceFinding>) {
    use CommerceFindingKind::*;
    if p.price.is_none() {
        out.push(finding(MissingPrice, Severity::High));
    }
    if p.availability.is_none() {
        out.push(finding(MissingAvailability, Severity::Medium));
    }
    if !p.shipping.has_shipping_details {
        out.push(finding(MissingShippingDetails, Severity::Medium));
    }
    if !p.returns.has_return_policy {
        out.push(finding(MissingReturnPolicy, Severity::Medium));
    }
    if p.reviews.rating_value.is_none() {
        out.push(finding(MissingReviews, Severity::Low));
    }
}

fn push_trust_findings(t: &TrustPages, out: &mut Vec<CommerceFinding>) {
    use CommerceFindingKind::*;
    // German B2C duty pages weigh heaviest; AGB/shipping/payment are strong UX
    // expectations; contact is advisory. Wording stays "not linked", not "absent".
    if !t.impressum {
        out.push(finding(MissingImpressumLink, Severity::High));
    }
    if !t.widerruf {
        out.push(finding(MissingWiderrufLink, Severity::High));
    }
    if !t.agb {
        out.push(finding(MissingAgbLink, Severity::Medium));
    }
    if !t.versand {
        out.push(finding(MissingShippingPageLink, Severity::Medium));
    }
    if !t.zahlungsarten {
        out.push(finding(MissingPaymentLink, Severity::Low));
    }
    if !t.kontakt {
        out.push(finding(MissingContactLink, Severity::Low));
    }
}

/// Detect mandatory/trust page links from anchor texts (case-insensitive).
pub fn detect_trust_pages(anchor_texts: &[String]) -> TrustPages {
    let lower: Vec<String> = anchor_texts.iter().map(|t| t.to_lowercase()).collect();
    let any = |needles: &[&str]| lower.iter().any(|t| needles.iter().any(|n| t.contains(n)));
    TrustPages {
        impressum: any(&["impressum", "imprint", "legal notice"]),
        agb: any(&["agb", "allgemeine geschäftsbedingungen", "terms"]),
        widerruf: any(&["widerruf", "rückgabe", "retoure", "return"]),
        versand: any(&["versand", "lieferung", "shipping", "delivery"]),
        zahlungsarten: any(&["zahlung", "zahlart", "payment"]),
        kontakt: any(&["kontakt", "contact"]),
    }
}

fn finding(kind: CommerceFindingKind, severity: Severity) -> CommerceFinding {
    CommerceFinding {
        kind,
        severity,
        message: commerce_finding_text(kind, true),
    }
}

/// Single source of truth for finding wording (#406).
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
        (MissingImpressumLink, true) => "No imprint (Impressum) link found on this page.".into(),
        (MissingImpressumLink, false) => "Kein Impressum-Link auf dieser Seite gefunden.".into(),
        (MissingWiderrufLink, true) => {
            "No right-of-withdrawal / returns (Widerruf/Retoure) link found on this page.".into()
        }
        (MissingWiderrufLink, false) => {
            "Kein Widerruf-/Retoure-Link auf dieser Seite gefunden.".into()
        }
        (MissingAgbLink, true) => "No terms (AGB) link found on this page.".into(),
        (MissingAgbLink, false) => "Kein AGB-Link auf dieser Seite gefunden.".into(),
        (MissingShippingPageLink, true) => {
            "No shipping-info (Versand/Lieferung) link found on this page.".into()
        }
        (MissingShippingPageLink, false) => {
            "Kein Versand-/Lieferungs-Link auf dieser Seite gefunden.".into()
        }
        (MissingPaymentLink, true) => {
            "No payment-methods (Zahlungsarten) link found on this page.".into()
        }
        (MissingPaymentLink, false) => {
            "Kein Zahlungsarten-Link auf dieser Seite gefunden.".into()
        }
        (MissingContactLink, true) => "No contact (Kontakt) link found on this page.".into(),
        (MissingContactLink, false) => "Kein Kontakt-Link auf dieser Seite gefunden.".into(),
        (NoPaymentMethodsVisible, true) => {
            "No common payment methods named on this cart/checkout page.".into()
        }
        (NoPaymentMethodsVisible, false) => {
            "Keine gängigen Zahlungsarten auf dieser Warenkorb-/Checkout-Seite genannt.".into()
        }
        (NoGuestCheckout, true) => {
            "No guest-checkout option detected on the checkout page (possible forced registration).".into()
        }
        (NoGuestCheckout, false) => {
            "Keine Gast-Bestellung auf der Checkout-Seite erkannt (mögliche Pflichtregistrierung).".into()
        }
    }
}

// ─── JSON-LD navigation helpers ────────────────────────────────────────────

fn schema_is(v: &Value, wanted: &str) -> bool {
    match v.get("@type") {
        Some(Value::String(s)) => s == wanted,
        Some(Value::Array(arr)) => arr.iter().any(|t| t.as_str() == Some(wanted)),
        _ => false,
    }
}

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
    match product.get("aggregateRating") {
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

    fn empty() -> StructuredData {
        StructuredData {
            json_ld: vec![],
            types: vec![],
            has_structured_data: false,
            rich_snippets_potential: vec![],
            schema_issues: vec![],
        }
    }

    #[test]
    fn non_shop_page_returns_none() {
        let sd = structured(serde_json::json!({"@type": "Article", "headline": "x"}));
        assert!(analyze_commerce("https://x.de/blog/post", &sd, &[], &[], false).is_none());
    }

    #[test]
    fn ecommerce_stack_without_product_schema_still_analyzes_trust_pages() {
        let c = analyze_commerce(
            "https://shop.de/kategorie/schuhe",
            &empty(),
            &["Impressum".into(), "Kontakt".into()],
            &[],
            true,
        )
        .expect("ecommerce stack activates commerce");
        assert_eq!(c.page_kind, CommercePageKind::Category);
        assert!(c.product.is_none());
        assert!(c.trust_pages.impressum);
        assert!(c.trust_pages.kontakt);
        assert!(!c.trust_pages.widerruf);
        // Widerruf/AGB/Versand/Zahlung missing → findings.
        assert!(c
            .findings
            .iter()
            .any(|f| f.kind == CommerceFindingKind::MissingWiderrufLink));
    }

    #[test]
    fn full_product_with_all_trust_pages_has_no_findings() {
        let sd = structured(serde_json::json!({
            "@type": "Product",
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
        let anchors: Vec<String> = [
            "Impressum",
            "AGB",
            "Widerruf",
            "Versand",
            "Zahlungsarten",
            "Kontakt",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let c = analyze_commerce("https://shop.de/produkt/widget", &sd, &anchors, &[], false)
            .expect("product");
        assert_eq!(c.page_kind, CommercePageKind::ProductDetail);
        assert_eq!(c.product.unwrap().score, 100);
        assert!(c.findings.is_empty());
    }

    #[test]
    fn conversion_signals_detected_and_gated_by_page_kind() {
        let texts: Vec<String> = [
            "Zahlung per PayPal, Klarna oder Rechnung",
            "Versandkostenfrei ab 50 €",
            "Weiter als Gast",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        // Checkout page: payment + free shipping + guest checkout all detected.
        let co = detect_conversion(&texts, CommercePageKind::Checkout);
        assert!(co.payment_methods.contains(&"PayPal".to_string()));
        assert!(co.payment_methods.contains(&"Klarna".to_string()));
        assert!(co.payment_methods.contains(&"Invoice".to_string()));
        assert!(co.free_shipping_threshold);
        assert_eq!(co.guest_checkout, Some(true));

        // Product page: guest_checkout is not applicable → None.
        let pd = detect_conversion(&texts, CommercePageKind::ProductDetail);
        assert_eq!(pd.guest_checkout, None);

        // Checkout without a guest marker → Some(false) + finding.
        let no_guest = detect_conversion(
            &["Zur Kasse".into(), "PayPal".into()],
            CommercePageKind::Checkout,
        );
        assert_eq!(no_guest.guest_checkout, Some(false));
        let mut findings = Vec::new();
        push_conversion_findings(&no_guest, CommercePageKind::Checkout, &mut findings);
        assert!(findings
            .iter()
            .any(|f| f.kind == CommerceFindingKind::NoGuestCheckout));
    }

    #[test]
    fn page_kind_classification() {
        use CommercePageKind::*;
        assert_eq!(
            detect_page_kind("https://s.de/checkout/step2", false),
            Checkout
        );
        assert_eq!(detect_page_kind("https://s.de/warenkorb", false), Cart);
        assert_eq!(
            detect_page_kind("https://s.de/produkt/abc", false),
            ProductDetail
        );
        // Product schema alone classifies as product detail even on a bare URL.
        assert_eq!(detect_page_kind("https://s.de/x", true), ProductDetail);
        assert_eq!(
            detect_page_kind("https://s.de/kategorie/schuhe", false),
            Category
        );
        assert_eq!(detect_page_kind("https://s.de/ueber-uns", false), Other);
        // Funnel stage wins over embedded product data.
        assert_eq!(detect_page_kind("https://s.de/kasse", true), Checkout);
    }

    #[test]
    fn trust_page_keywords_match_variants() {
        let t = detect_trust_pages(&[
            "Allgemeine Geschäftsbedingungen".into(),
            "Rückgabe & Retoure".into(),
            "Lieferung".into(),
        ]);
        assert!(t.agb);
        assert!(t.widerruf);
        assert!(t.versand);
        assert!(!t.impressum);
    }

    #[test]
    fn english_finding_text_has_no_german_umlauts() {
        for kind in [
            CommerceFindingKind::MissingPrice,
            CommerceFindingKind::MissingAvailability,
            CommerceFindingKind::MissingShippingDetails,
            CommerceFindingKind::MissingReturnPolicy,
            CommerceFindingKind::MissingReviews,
            CommerceFindingKind::MissingImpressumLink,
            CommerceFindingKind::MissingAgbLink,
            CommerceFindingKind::MissingWiderrufLink,
            CommerceFindingKind::MissingShippingPageLink,
            CommerceFindingKind::MissingPaymentLink,
            CommerceFindingKind::MissingContactLink,
        ] {
            let t = commerce_finding_text(kind, true);
            assert!(
                !t.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "EN text has umlaut: {t}"
            );
        }
    }
}
