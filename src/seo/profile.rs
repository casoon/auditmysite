//! SEO Content Profile — technical content and signal profile
//!
//! Analysiert die vorhandenen SEO-Rohdaten (Meta, Headings, JSON-LD, Social, Technical)
//! und erzeugt eine kompakte Zusammenfassung: Was ist die Seite, welche technischen
//! SEO-Signale werden eingesetzt, wie vollständig sind strukturierte Daten.

use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};
use web_checks::meta::LengthClass;

use super::SeoAnalysis;
use crate::seo::schema::SchemaType;

// ─── Maturity & Completeness ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeoMaturityLevel {
    Basic,
    Standard,
    Advanced,
    Professional,
}

impl SeoMaturityLevel {
    /// The `seo-profile-maturity-*` message of `locales/{en,de}/seo-issues.ftl`.
    pub fn label(&self, en: bool) -> String {
        let key = match self {
            Self::Basic => "seo-profile-maturity-basic",
            Self::Standard => "seo-profile-maturity-standard",
            Self::Advanced => "seo-profile-maturity-advanced",
            Self::Professional => "seo-profile-maturity-professional",
        };
        crate::i18n::text(key, en)
    }

    /// The `seo-profile-maturity-description-*` message of
    /// `locales/{en,de}/seo-issues.ftl`.
    pub fn description(&self, en: bool) -> String {
        let key = match self {
            Self::Basic => "seo-profile-maturity-description-basic",
            Self::Standard => "seo-profile-maturity-description-standard",
            Self::Advanced => "seo-profile-maturity-description-advanced",
            Self::Professional => "seo-profile-maturity-description-professional",
        };
        crate::i18n::text(key, en)
    }

    fn from_count(techniques: u32) -> Self {
        match techniques {
            0..=3 => Self::Basic,
            4..=6 => Self::Standard,
            7..=9 => Self::Advanced,
            _ => Self::Professional,
        }
    }
}

// ─── Top-Level Profile ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoContentProfile {
    pub content_identity: ContentIdentity,
    pub page_classification: PageClassification,
    pub schema_inventory: SchemaInventory,
    pub signal_strength: SeoSignalStrength,
    pub maturity: SeoMaturityLevel,
    pub maturity_techniques: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PageType {
    Editorial,
    StructuredContent,
    MarketingLanding,
    MediaHeavy,
    Utility,
    NavigationHub,
    ThinContent,
}

impl PageType {
    /// The `seo-profile-page-type-*` message of `locales/{en,de}/seo-issues.ftl`.
    pub fn label(&self, en: bool) -> String {
        let key = match self {
            Self::Editorial => "seo-profile-page-type-editorial",
            Self::StructuredContent => "seo-profile-page-type-structured-content",
            Self::MarketingLanding => "seo-profile-page-type-marketing-landing",
            Self::MediaHeavy => "seo-profile-page-type-media-heavy",
            Self::Utility => "seo-profile-page-type-utility",
            // Same wording as the journey layer's hub intent, so one page is
            // not "Hub / Portal" in one section and "Navigations-/Hub-Seite"
            // in the next (#902-7).
            Self::NavigationHub => return crate::page_flow::PageIntent::Hub.label(en),
            Self::ThinContent => "seo-profile-page-type-thin-content",
        };
        crate::i18n::text(key, en)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageClassification {
    pub primary_type: PageType,
    pub attributes: Vec<String>,
    pub content_depth_score: u32,
    pub structural_richness_score: u32,
    pub media_text_balance_score: u32,
    pub intent_fit_score: u32,
}

// ─── Content Identity ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentIdentity {
    /// Zusammenfassung aus Title + Description
    pub summary: String,
    /// Seitenname (OG site_name → JSON-LD Org name → Domain)
    pub site_name: Option<String>,
    /// Inhaltstyp (Website/Artikel/E-Commerce etc.)
    pub content_type: String,
    /// Sprache
    pub language: Option<String>,
    /// Alle gefundenen Schema-Typen als Hinweis
    pub category_hints: Vec<String>,
}

// ─── Schema Inventory ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInventory {
    pub schemas: Vec<SchemaDetail>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDetail {
    pub schema_type: String,
    pub fields_present: Vec<String>,
    pub fields_missing: Vec<String>,
    pub completeness_pct: u32,
    pub extracted: SchemaExtracted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SchemaExtracted {
    Organization {
        name: Option<String>,
        url: Option<String>,
        logo: Option<String>,
        address: Option<String>,
        phone: Option<String>,
    },
    LocalBusiness {
        name: Option<String>,
        address: Option<String>,
        postal_code: Option<String>,
        locality: Option<String>,
        country: Option<String>,
        phone: Option<String>,
        url: Option<String>,
        opening_hours: Box<Vec<String>>,
        price_range: Option<String>,
        same_as: Box<Vec<String>>,
        logo: Option<String>,
        image: Option<String>,
        area_served: Box<Vec<String>>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        aggregate_rating: Option<String>,
    },
    Article {
        headline: Option<String>,
        author: Option<String>,
        date_published: Option<String>,
        date_modified: Option<String>,
        publisher: Option<String>,
    },
    FAQPage {
        question_count: usize,
        questions: Vec<String>,
    },
    Product {
        name: Option<String>,
        price: Option<String>,
        currency: Option<String>,
        rating: Option<String>,
        availability: Option<String>,
    },
    WebSite {
        name: Option<String>,
        url: Option<String>,
        has_search_action: bool,
    },
    WebPage {
        name: Option<String>,
        url: Option<String>,
        author: Option<String>,
        in_language: Option<String>,
    },
    Service {
        name: Option<String>,
        address: Option<String>,
        phone: Option<String>,
        price_range: Option<String>,
        area_served_count: usize,
    },
    BreadcrumbList {
        item_count: usize,
    },
    Generic {
        key_fields: Vec<(String, String)>,
    },
}

// ─── Signal Strength ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoSignalStrength {
    pub categories: Vec<SignalCategory>,
    /// Weighted pass rate across the categories. Internal aggregate for the
    /// Search Experience composite — the report shows `totals()` instead,
    /// because a weighted percentage of pass rates is one derivation further
    /// from the evidence than the counts are (plan 29, D4).
    pub overall_pct: u32,
}

impl SeoSignalStrength {
    /// Total signals met and total signals checked, across all categories.
    pub fn totals(&self) -> (u32, u32) {
        self.categories
            .iter()
            .fold((0, 0), |(p, t), c| (p + c.passed, t + c.total))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCategory {
    pub name: String,
    /// How many of this category's checks passed, and how many there are.
    ///
    /// Reported as counts rather than as a percentage (plan 29, D4): a bare
    /// "80 %" hides its denominator, so adding a check that usually passes
    /// silently lifts every site's value. The check list per category is
    /// fixed, so `passed`/`total` is the whole truth about this category.
    pub passed: u32,
    pub total: u32,
    pub checks: Vec<SignalCheck>,
}

impl SignalCategory {
    /// Pass rate, for the aggregate that the Search Experience composite
    /// consumes. Deliberately a method rather than a stored field: the
    /// percentage is a derivation of the counts, not a second measurement.
    pub fn pct(&self) -> u32 {
        if self.total == 0 {
            return 0;
        }
        self.passed * 100 / self.total
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCheck {
    pub label: String,
    pub passed: bool,
    pub detail: Option<String>,
}

// ─── Builder ────────────────────────────────────────────────────────────────

pub fn build_content_profile(seo: &SeoAnalysis, locale: &str) -> SeoContentProfile {
    let en = locale == "en";
    let content_identity = build_identity(seo, en);
    let page_classification = classify_page(seo, en);
    let schema_inventory = build_schema_inventory(seo, en);
    let signal_strength = build_signal_strength(seo, en);
    let maturity_techniques = count_techniques(seo);
    let maturity = SeoMaturityLevel::from_count(maturity_techniques);

    SeoContentProfile {
        content_identity,
        page_classification,
        schema_inventory,
        signal_strength,
        maturity,
        maturity_techniques,
    }
}

fn classify_page(seo: &SeoAnalysis, en: bool) -> PageClassification {
    let word_count = seo.technical.word_count;
    let heading_count = seo.headings.total_count as u32;
    let h2_plus_count = seo
        .headings
        .headings
        .iter()
        .filter(|h| h.level >= 2)
        .count() as u32;
    let internal_links = seo.technical.internal_links;
    let has_faq = seo.structured_data.types.contains(&SchemaType::FAQPage);
    let has_article_schema = seo.structured_data.types.iter().any(|t| {
        matches!(
            t,
            SchemaType::Article | SchemaType::BlogPosting | SchemaType::NewsArticle
        )
    });
    let has_product_schema = seo.structured_data.types.contains(&SchemaType::Product);
    let has_search_schema = seo.structured_data.types.contains(&SchemaType::WebSite);

    let content_depth_score = score_content_depth(word_count, h2_plus_count, heading_count);
    let structural_richness_score = score_structural_richness(
        heading_count,
        h2_plus_count,
        has_faq,
        seo.headings.issues.len(),
    );
    let media_text_balance_score = score_media_text_balance(
        word_count,
        seo.social.open_graph.is_some(),
        has_product_schema,
    );

    let is_utility_title = seo
        .meta
        .title
        .as_deref()
        .map(|t| t.contains("Kontakt") || t.contains("Login"))
        .unwrap_or(false);
    let looks_like_marketing_page = has_product_schema
        || seo.social.open_graph.is_some()
        || (word_count <= 700 && h2_plus_count >= 2);
    let looks_like_hub_page = internal_links >= 40 || has_search_schema;

    let primary_type = if has_article_schema || (word_count >= 1200 && h2_plus_count >= 3) {
        PageType::Editorial
    } else if has_faq || (word_count >= 600 && (h2_plus_count >= 4 || heading_count >= 6)) {
        PageType::StructuredContent
    } else if looks_like_hub_page {
        PageType::NavigationHub
    } else if is_utility_title {
        PageType::Utility
    } else if looks_like_marketing_page {
        PageType::MarketingLanding
    } else if word_count < 300 {
        PageType::ThinContent
    } else {
        PageType::MediaHeavy
    };

    let intent_fit_score =
        score_intent_fit(&primary_type, word_count, h2_plus_count, internal_links);
    let mut attribute_keys = Vec::new();
    if word_count >= 1200 {
        attribute_keys.push("seo-profile-attribute-text-rich");
    } else if word_count < 300 {
        attribute_keys.push("seo-profile-attribute-very-short");
    }
    if h2_plus_count >= 3 {
        attribute_keys.push("seo-profile-attribute-structured");
    }
    if internal_links >= 30 {
        attribute_keys.push("seo-profile-attribute-navigation-heavy");
    }
    if seo.social.open_graph.is_some() {
        attribute_keys.push("seo-profile-attribute-visually-oriented");
    }
    if has_faq {
        attribute_keys.push("seo-profile-attribute-knowledge-oriented");
    }
    if matches!(primary_type, PageType::MarketingLanding) {
        attribute_keys.push("seo-profile-attribute-conversion-oriented");
    }
    if matches!(primary_type, PageType::Utility) {
        attribute_keys.push("seo-profile-attribute-purpose-bound");
    }
    if matches!(primary_type, PageType::ThinContent) {
        attribute_keys.push("seo-profile-attribute-thin");
    }
    let mut attributes: Vec<String> = attribute_keys.into_iter().map(|key| t(key, en)).collect();
    attributes.sort();
    attributes.dedup();

    PageClassification {
        primary_type,
        attributes,
        content_depth_score,
        structural_richness_score,
        media_text_balance_score,
        intent_fit_score,
    }
}

fn score_content_depth(word_count: u32, h2_plus_count: u32, heading_count: u32) -> u32 {
    let word_component = match word_count {
        0..=149 => 10,
        150..=299 => 25,
        300..=599 => 45,
        600..=1199 => 70,
        _ => 90,
    };
    let structure_bonus = (h2_plus_count * 5 + heading_count.min(6) * 2).min(20);
    (word_component + structure_bonus).min(100)
}

fn score_structural_richness(
    heading_count: u32,
    h2_plus_count: u32,
    has_faq: bool,
    heading_issues: usize,
) -> u32 {
    let mut score = (heading_count * 10).min(60) + (h2_plus_count * 8).min(24);
    if has_faq {
        score += 8;
    }
    score = score.saturating_sub((heading_issues as u32) * 6);
    score.min(100)
}

fn score_media_text_balance(word_count: u32, has_og: bool, has_product_schema: bool) -> u32 {
    let mut score: u32 = match word_count {
        0..=199 => 35,
        200..=499 => 55,
        500..=999 => 75,
        _ => 85,
    };
    if has_og {
        score = (score + 5).min(100);
    }
    if has_product_schema {
        score = score.saturating_sub(5);
    }
    score
}

/// Whether `score_intent_fit` reads any measured signal for this page type,
/// or just returns a constant keyed on the type itself.
///
/// `MediaHeavy`, `Utility` and `ThinContent` have a single constant each, so
/// their intent-fit value says nothing about the page in hand — it restates
/// its classification. A reader-facing judgement about *this page* must not be
/// gated on such a value (plan 39).
pub fn intent_fit_is_signal_derived(page_type: &PageType) -> bool {
    match page_type {
        PageType::Editorial
        | PageType::StructuredContent
        | PageType::MarketingLanding
        | PageType::NavigationHub => true,
        PageType::MediaHeavy | PageType::Utility | PageType::ThinContent => false,
    }
}

fn score_intent_fit(
    page_type: &PageType,
    word_count: u32,
    h2_plus_count: u32,
    internal_links: u32,
) -> u32 {
    match page_type {
        PageType::Editorial => {
            if word_count >= 1000 && h2_plus_count >= 3 {
                88
            } else {
                72
            }
        }
        PageType::StructuredContent => {
            if h2_plus_count >= 4 {
                84
            } else {
                68
            }
        }
        PageType::MarketingLanding => {
            if (250..=900).contains(&word_count) {
                80
            } else {
                65
            }
        }
        PageType::MediaHeavy => 62,
        PageType::Utility => 78,
        PageType::NavigationHub => {
            if internal_links >= 30 {
                82
            } else {
                60
            }
        }
        PageType::ThinContent => 28,
    }
}

// ─── Content Identity Builder ───────────────────────────────────────────────

fn build_identity(seo: &SeoAnalysis, en: bool) -> ContentIdentity {
    // Site name: OG site_name → JSON-LD Organization name → None
    let site_name = seo
        .social
        .open_graph
        .as_ref()
        .and_then(|og| og.site_name.clone())
        .or_else(|| find_org_name(&seo.structured_data.json_ld));

    // Content type from OG type → JSON-LD @type → default
    let content_type = derive_content_type(seo, en);

    // Summary from title + description
    let summary = build_summary(seo, en);

    // Category hints from all schema types
    let category_hints: Vec<String> = seo
        .structured_data
        .types
        .iter()
        .map(|t| t.to_string())
        .collect();

    ContentIdentity {
        summary,
        site_name,
        content_type,
        language: seo.technical.lang.clone(),
        category_hints,
    }
}

fn find_org_name(json_ld: &[crate::seo::schema::JsonLdSchema]) -> Option<String> {
    for schema in json_ld {
        if schema
            .schema_types
            .iter()
            .any(|t| t == "Organization" || t == "LocalBusiness" || t == "ProfessionalService")
            || schema.schema_type == "Organization"
            || schema.schema_type == "LocalBusiness"
            || schema.schema_type == "ProfessionalService"
        {
            if let Some(name) = schema.content["name"].as_str() {
                return Some(name.to_string());
            }
        }
        // Check @graph
        if let Some(graph) = schema.content["@graph"].as_array() {
            for item in graph {
                let item_type = item["@type"].as_str().unwrap_or("");
                if item_type == "Organization" || item_type == "LocalBusiness" {
                    if let Some(name) = item["name"].as_str() {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }
    None
}

fn derive_content_type(seo: &SeoAnalysis, en: bool) -> String {
    // Check OG type first
    if let Some(ref og) = seo.social.open_graph {
        if let Some(ref og_type) = og.og_type {
            match og_type.as_str() {
                "article" => return t("seo-profile-content-type-article", en),
                "product" => return t("seo-profile-content-type-product", en),
                "profile" => return t("seo-profile-content-type-profile", en),
                "website" => {} // generic, check schema types
                _ => {}
            }
        }
    }

    // Check JSON-LD types
    for schema_type in &seo.structured_data.types {
        let key = match schema_type {
            SchemaType::Article | SchemaType::BlogPosting | SchemaType::NewsArticle => {
                "seo-profile-content-type-article-blog"
            }
            SchemaType::Product => "seo-profile-content-type-ecommerce-product",
            SchemaType::LocalBusiness => "seo-profile-content-type-local-business",
            SchemaType::Event => "seo-profile-content-type-event",
            SchemaType::Recipe => "seo-profile-content-type-recipe",
            SchemaType::FAQPage => "seo-profile-content-type-faq",
            _ => continue,
        };
        return t(key, en);
    }

    "Website".to_string()
}

fn build_summary(seo: &SeoAnalysis, en: bool) -> String {
    let title = seo.meta.title.as_deref().unwrap_or("");
    let desc = seo.meta.description.as_deref().unwrap_or("");

    if !title.is_empty() && !desc.is_empty() {
        let combined = format!("{} — {}", title, desc);
        if combined.len() > 150 {
            let boundary = combined
                .char_indices()
                .take_while(|(i, _)| *i <= 147)
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            format!("{}...", &combined[..boundary])
        } else {
            combined
        }
    } else if !title.is_empty() {
        title.to_string()
    } else if !desc.is_empty() {
        desc.to_string()
    } else {
        t("seo-profile-summary-none", en)
    }
}

// ─── Schema Inventory Builder ───────────────────────────────────────────────

fn build_schema_inventory(seo: &SeoAnalysis, en: bool) -> SchemaInventory {
    let mut schemas = Vec::new();

    for json_ld in &seo.structured_data.json_ld {
        if !json_ld.is_valid {
            continue;
        }
        let root_types = if json_ld.schema_types.is_empty() {
            vec![json_ld.schema_type.clone()]
        } else {
            json_ld.schema_types.clone()
        };

        for schema_type in root_types {
            if let Some(d) = analyze_schema(&schema_type, &json_ld.content, en) {
                schemas.push(d);
                break;
            }
        }

        // `schema::detect_structured_data` already flattens @graph documents.
        // Re-walking graph members here would duplicate inventory rows.
    }

    let total_count = schemas.len();
    SchemaInventory {
        schemas,
        total_count,
    }
}

fn analyze_schema(
    schema_type: &str,
    content: &serde_json::Value,
    en: bool,
) -> Option<SchemaDetail> {
    let expected = crate::seo::schema_rules::inventory_fields(schema_type);
    let extracted = match schema_type {
        "Organization" => analyze_organization(content),
        "LocalBusiness" => analyze_local_business(content),
        "ProfessionalService"
        | "Service"
        | "WebDesignCompany"
        | "LegalService"
        | "AccountingService"
        | "FinancialService"
        | "HVACBusiness"
        | "Dentist"
        | "Physician"
        | "Attorney" => analyze_service(content),
        "Article" | "BlogPosting" | "NewsArticle" => analyze_article(content),
        "FAQPage" => analyze_faq(content),
        "Product" => analyze_product(content),
        "WebSite" => analyze_website(content),
        "WebPage" => analyze_webpage(content),
        "BreadcrumbList" => analyze_breadcrumb(content),
        _ => analyze_generic(content, en),
    };

    let mut present = Vec::new();
    let mut missing = Vec::new();
    for field in expected {
        if !content[field].is_null() {
            present.push(field.to_string());
        } else {
            missing.push(field.to_string());
        }
    }

    let total = present.len() + missing.len();
    let pct = (present.len() * 100).checked_div(total).unwrap_or(0) as u32;

    Some(SchemaDetail {
        schema_type: schema_type.to_string(),
        fields_present: present,
        fields_missing: missing,
        completeness_pct: pct,
        extracted,
    })
}

fn str_opt(v: &serde_json::Value, key: &str) -> Option<String> {
    v[key].as_str().map(|s| s.to_string())
}

fn analyze_organization(v: &serde_json::Value) -> SchemaExtracted {
    let address = v["address"]["streetAddress"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "address"));

    SchemaExtracted::Organization {
        name: str_opt(v, "name"),
        url: str_opt(v, "url"),
        logo: v["logo"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| str_opt(&v["logo"], "url")),
        address,
        phone: str_opt(v, "telephone"),
    }
}

fn analyze_local_business(v: &serde_json::Value) -> SchemaExtracted {
    let addr = &v["address"];
    let address = addr["streetAddress"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "address"));
    let postal_code = str_opt(addr, "postalCode");
    let locality = str_opt(addr, "addressLocality");
    let country = str_opt(addr, "addressCountry")
        .or_else(|| addr["addressCountry"]["@id"].as_str().map(String::from));

    let opening_hours = v["openingHours"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|h| h.as_str().map(|s| s.to_string()))
                .collect()
        })
        .or_else(|| v["openingHours"].as_str().map(|s| vec![s.to_string()]))
        .unwrap_or_default();

    let same_as = v["sameAs"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .or_else(|| v["sameAs"].as_str().map(|s| vec![s.to_string()]))
        .unwrap_or_default();

    let area_served = v["areaServed"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .or_else(|| v["areaServed"].as_str().map(|s| vec![s.to_string()]))
        .unwrap_or_default();

    let geo = &v["geo"];
    let latitude = geo["latitude"]
        .as_f64()
        .or_else(|| geo["latitude"].as_str().and_then(|s| s.parse().ok()));
    let longitude = geo["longitude"]
        .as_f64()
        .or_else(|| geo["longitude"].as_str().and_then(|s| s.parse().ok()));

    let logo = v["logo"]
        .as_str()
        .map(String::from)
        .or_else(|| str_opt(&v["logo"], "url"));

    let image = v["image"]
        .as_str()
        .map(String::from)
        .or_else(|| str_opt(&v["image"], "url"));

    let aggregate_rating = v["aggregateRating"]["ratingValue"]
        .as_str()
        .map(String::from)
        .or_else(|| {
            v["aggregateRating"]["ratingValue"]
                .as_f64()
                .map(|r| format!("{:.1}", r))
        });

    SchemaExtracted::LocalBusiness {
        name: str_opt(v, "name"),
        address,
        postal_code,
        locality,
        country,
        phone: str_opt(v, "telephone"),
        url: str_opt(v, "url"),
        opening_hours: Box::new(opening_hours),
        price_range: str_opt(v, "priceRange"),
        same_as: Box::new(same_as),
        logo,
        image,
        area_served: Box::new(area_served),
        latitude,
        longitude,
        aggregate_rating,
    }
}

fn analyze_article(v: &serde_json::Value) -> SchemaExtracted {
    let author = v["author"]["name"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "author"));

    let publisher = v["publisher"]["name"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "publisher"));

    SchemaExtracted::Article {
        headline: str_opt(v, "headline"),
        author,
        date_published: str_opt(v, "datePublished"),
        date_modified: str_opt(v, "dateModified"),
        publisher,
    }
}

fn analyze_faq(v: &serde_json::Value) -> SchemaExtracted {
    let mut questions = Vec::new();
    if let Some(entities) = v["mainEntity"].as_array() {
        for entity in entities {
            if let Some(q) = entity["name"].as_str() {
                questions.push(q.to_string());
            }
        }
    }

    SchemaExtracted::FAQPage {
        question_count: questions.len(),
        questions,
    }
}

fn analyze_product(v: &serde_json::Value) -> SchemaExtracted {
    let offers = &v["offers"];
    let price = offers["price"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| offers["price"].as_f64().map(|p| format!("{:.2}", p)));
    let currency = str_opt(offers, "priceCurrency");
    let availability = str_opt(offers, "availability");

    let rating = v["aggregateRating"]["ratingValue"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| {
            v["aggregateRating"]["ratingValue"]
                .as_f64()
                .map(|r| format!("{:.1}", r))
        });

    SchemaExtracted::Product {
        name: str_opt(v, "name"),
        price,
        currency,
        rating,
        availability,
    }
}

fn analyze_website(v: &serde_json::Value) -> SchemaExtracted {
    let has_search_action = v["potentialAction"]["@type"]
        .as_str()
        .map(|t| t == "SearchAction")
        .unwrap_or(false);

    SchemaExtracted::WebSite {
        name: str_opt(v, "name"),
        url: str_opt(v, "url"),
        has_search_action,
    }
}

fn analyze_webpage(v: &serde_json::Value) -> SchemaExtracted {
    let author = v["author"]["name"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "author"));

    SchemaExtracted::WebPage {
        name: str_opt(v, "name"),
        url: str_opt(v, "url"),
        author,
        in_language: str_opt(v, "inLanguage"),
    }
}

fn analyze_service(v: &serde_json::Value) -> SchemaExtracted {
    let address = v["address"]["streetAddress"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "address"));
    let area_served_count = v["areaServed"].as_array().map(|arr| arr.len()).unwrap_or(0);

    SchemaExtracted::Service {
        name: str_opt(v, "name"),
        address,
        phone: str_opt(v, "telephone"),
        price_range: str_opt(v, "priceRange"),
        area_served_count,
    }
}

fn analyze_breadcrumb(v: &serde_json::Value) -> SchemaExtracted {
    let item_count = v["itemListElement"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    SchemaExtracted::BreadcrumbList { item_count }
}

fn analyze_generic(v: &serde_json::Value, en: bool) -> SchemaExtracted {
    let mut key_fields = Vec::new();
    if let Some(obj) = v.as_object() {
        for (key, val) in obj {
            if key.starts_with('@') {
                continue;
            }
            let display = match val {
                serde_json::Value::String(s) => {
                    if s.len() > 60 {
                        let b = s
                            .char_indices()
                            .take_while(|(i, _)| *i <= 57)
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        format!("{}...", &s[..b])
                    } else {
                        s.clone()
                    }
                }
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Array(a) => {
                    count_text("seo-profile-generic-array-entries", a.len(), en)
                }
                serde_json::Value::Object(_) => "{...}".to_string(),
                serde_json::Value::Null => continue,
            };
            key_fields.push((key.clone(), display));
            if key_fields.len() >= 8 {
                break;
            }
        }
    }

    SchemaExtracted::Generic { key_fields }
}

// ─── Signal Strength Builder ────────────────────────────────────────────────

fn build_signal_strength(seo: &SeoAnalysis, en: bool) -> SeoSignalStrength {
    let meta = build_meta_signals(seo, en);
    let headings = build_heading_signals(seo, en);
    let social = build_social_signals(seo, en);
    let technical = build_technical_signals(seo, en);
    let structured = build_structured_data_signals(seo, en);
    let content = build_content_signals(seo, en);

    // Gewichteter Durchschnitt
    let overall_pct = (meta.pct() as f32 * 0.20
        + headings.pct() as f32 * 0.15
        + social.pct() as f32 * 0.15
        + technical.pct() as f32 * 0.20
        + structured.pct() as f32 * 0.15
        + content.pct() as f32 * 0.15) as u32;

    SeoSignalStrength {
        categories: vec![meta, headings, social, technical, structured, content],
        overall_pct,
    }
}

/// Signals met and signals checked for one category.
fn category_counts(checks: &[SignalCheck]) -> (u32, u32) {
    (
        checks.iter().filter(|c| c.passed).count() as u32,
        checks.len() as u32,
    )
}

fn check(label: &str, passed: bool, detail: Option<String>) -> SignalCheck {
    SignalCheck {
        label: label.to_string(),
        passed,
        detail,
    }
}

/// The `seo-profile-*` message `key` of `locales/{en,de}/seo-issues.ftl`.
fn t(key: &str, en: bool) -> String {
    crate::i18n::text(key, en)
}

/// The `seo-profile-*` message `key` with the number `$count`.
fn count_text(key: &str, count: usize, en: bool) -> String {
    crate::i18n::count_text(key, count, en)
}

/// The `seo-profile-*` message `key` with the text `$list`.
fn list_text(key: &str, list: &str, en: bool) -> String {
    crate::i18n::text_args(key, &[("list", FluentValue::from(list.to_string()))], en)
}

fn build_meta_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let title_ok = seo
        .meta
        .title
        .as_ref()
        .map(|t| web_checks::meta::TITLE.classify(t) == LengthClass::Ok)
        .unwrap_or(false);
    let desc_ok = seo
        .meta
        .description
        .as_ref()
        .map(|d| web_checks::meta::DESCRIPTION.classify(d) == LengthClass::Ok)
        .unwrap_or(false);

    let chars = |n: usize| count_text("seo-profile-detail-characters", n, en);
    let missing = || t("seo-profile-detail-missing", en);

    let checks = vec![
        check(
            &t("seo-profile-check-title", en),
            title_ok,
            seo.meta
                .title
                .as_ref()
                .map(|t| chars(web_checks::meta::length(t))),
        ),
        check(
            &t("seo-profile-check-description", en),
            desc_ok,
            seo.meta
                .description
                .as_ref()
                .map(|d| chars(web_checks::meta::length(d))),
        ),
        check(
            &t("seo-profile-check-viewport", en),
            seo.meta.viewport.is_some(),
            Some(seo.meta.viewport.clone().unwrap_or_else(missing)),
        ),
        check(
            &t("seo-profile-check-charset", en),
            seo.meta.charset.is_some(),
            Some(seo.meta.charset.clone().unwrap_or_else(missing)),
        ),
        check(
            &t("seo-profile-check-lang-set", en),
            seo.meta.lang.is_some(),
            seo.meta.lang.clone(),
        ),
    ];
    let (passed, total) = category_counts(&checks);
    SignalCategory {
        name: "Meta-Tags".to_string(),
        passed,
        total,
        checks,
    }
}

fn build_heading_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let h1_ok = seo.headings.h1_count == 1;
    let h1_not_empty = seo
        .headings
        .h1_text
        .as_ref()
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);

    let no_skipped = !seo
        .headings
        .issues
        .iter()
        .any(|i| i.issue_type == "skipped_level");
    // The first heading in document order must be the H1: the same fact the
    // structure signal ("not the first heading") and the screen-reader
    // heading audit report. The skip-level rule alone cannot see it (H2 first
    // is no skipped level) and used to leave "Logical hierarchy: no issues"
    // next to it (#952-1).
    let first_level = seo.headings.first_exposed_level();
    let first_not_h1 = matches!(first_level, Some(level) if level != 1);
    let order_problems = seo.headings.issues.len() + usize::from(first_not_h1);
    let no_issues = order_problems == 0;

    let checks = vec![
        check(
            &t("seo-profile-check-single-h1", en),
            h1_ok,
            Some(count_text(
                "seo-profile-detail-h1-count",
                seo.headings.h1_count,
                en,
            )),
        ),
        check(
            &t("seo-profile-check-h1-text", en),
            h1_not_empty,
            Some(
                seo.headings
                    .h1_text
                    .clone()
                    .unwrap_or_else(|| t("seo-profile-detail-no-h1-text", en)),
            ),
        ),
        check(
            &t("seo-profile-check-no-skipped-levels", en),
            no_skipped,
            Some(t(
                if no_skipped {
                    "seo-profile-detail-no-gaps"
                } else {
                    "seo-profile-detail-levels-skipped"
                },
                en,
            )),
        ),
        check(
            &t("seo-profile-check-logical-hierarchy", en),
            no_issues,
            Some(if no_issues {
                t("seo-profile-detail-no-issues", en)
            } else {
                match (first_not_h1, first_level) {
                    (true, Some(level)) => crate::i18n::text_args(
                        "seo-profile-detail-order-problems-first-heading",
                        &[
                            ("count", FluentValue::from(order_problems as i64)),
                            ("level", FluentValue::from(i64::from(level))),
                        ],
                        en,
                    ),
                    _ => count_text("seo-profile-detail-order-problems", order_problems, en),
                }
            }),
        ),
    ];
    let (passed, total) = category_counts(&checks);
    SignalCategory {
        name: t("seo-profile-category-headings", en),
        passed,
        total,
        checks,
    }
}

fn build_social_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let og_present = seo.social.open_graph.is_some();
    let og_complete = seo
        .social
        .open_graph
        .as_ref()
        .map(|og| og.completeness() >= 80)
        .unwrap_or(false);
    let tw_present = seo.social.twitter_card.is_some();
    let tw_complete = seo
        .social
        .twitter_card
        .as_ref()
        .map(|tc| tc.completeness() >= 75)
        .unwrap_or(false);

    let present_or_missing = |present: bool| {
        t(
            if present {
                "seo-profile-detail-present"
            } else {
                "seo-profile-detail-missing"
            },
            en,
        )
    };

    let checks = vec![
        check(
            &t("seo-profile-check-og-present", en),
            og_present,
            Some(present_or_missing(og_present)),
        ),
        check(
            &t("seo-profile-check-og-complete", en),
            og_complete,
            seo.social
                .open_graph
                .as_ref()
                .map(|og| format!("{}%", og.completeness()))
                .or_else(|| Some(present_or_missing(false))),
        ),
        check(
            &t("seo-profile-check-twitter-present", en),
            tw_present,
            Some(present_or_missing(tw_present)),
        ),
        check(
            &t("seo-profile-check-twitter-complete", en),
            tw_complete,
            seo.social
                .twitter_card
                .as_ref()
                .map(|tc| format!("{}%", tc.completeness()))
                .or_else(|| Some(present_or_missing(false))),
        ),
    ];
    let (passed, total) = category_counts(&checks);
    SignalCategory {
        name: "Social Media".to_string(),
        passed,
        total,
        checks,
    }
}

fn build_technical_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let robots_ok = seo
        .technical
        .robots_meta
        .as_ref()
        .map(|r| !r.contains("noindex"))
        .unwrap_or(true); // No robots meta = ok (default is index)

    let missing = || t("seo-profile-detail-missing", en);

    let checks = vec![
        check(
            "HTTPS",
            seo.technical.https,
            Some(t(
                if seo.technical.https {
                    "seo-profile-detail-active"
                } else {
                    "seo-profile-detail-missing"
                },
                en,
            )),
        ),
        check(
            "Canonical URL",
            seo.technical.has_canonical,
            Some(seo.technical.canonical_url.clone().unwrap_or_else(missing)),
        ),
        check(
            &t("seo-profile-check-lang", en),
            seo.technical.has_lang,
            Some(seo.technical.lang.clone().unwrap_or_else(missing)),
        ),
        check(
            &t("seo-profile-check-hreflang", en),
            true,
            Some(if seo.technical.has_hreflang {
                count_text(
                    "seo-profile-detail-hreflang-languages",
                    seo.technical.hreflang.len(),
                    en,
                )
            } else {
                t("seo-profile-detail-no-hreflang", en)
            }),
        ),
        check(
            &t("seo-profile-check-robots-indexing", en),
            robots_ok,
            Some(
                seo.technical
                    .robots_meta
                    .clone()
                    .unwrap_or_else(|| t("seo-profile-detail-no-robots-meta", en)),
            ),
        ),
    ];
    let (passed, total) = category_counts(&checks);
    SignalCategory {
        name: t("seo-profile-category-technical", en),
        passed,
        total,
        checks,
    }
}

fn build_structured_data_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let has_any = seo.structured_data.has_structured_data;
    let has_rich = !seo.structured_data.rich_snippets_potential.is_empty();
    let has_org = seo.structured_data.types.iter().any(|t| {
        matches!(
            t,
            SchemaType::Organization | SchemaType::WebSite | SchemaType::LocalBusiness
        )
    });

    let mut checks = vec![
        check(
            &t("seo-profile-check-structured-data", en),
            has_any,
            Some(if !has_any {
                t("seo-profile-detail-no-structured-data", en)
            } else {
                let n = seo
                    .structured_data
                    .json_ld
                    .iter()
                    .filter(|schema| schema.is_valid && !schema.schema_types.is_empty())
                    .count();
                if n > 0 {
                    let noun = if n == 1 { "Schema" } else { "Schemas" };
                    format!("{} {}", n, noun)
                } else if seo
                    .structured_data
                    .json_ld
                    .iter()
                    .any(|schema| !schema.is_valid)
                {
                    t("seo-profile-detail-jsonld-not-evaluable", en)
                } else {
                    // has_structured_data is true via microdata/RDFa, but
                    // json_ld is empty — the count only tracks JSON-LD, so
                    // "0 Schemas" would contradict the passing checkmark above.
                    t("seo-profile-detail-microdata-only", en)
                }
            }),
        ),
        check(
            &t("seo-profile-check-rich-result", en),
            has_rich,
            Some(if has_rich {
                crate::i18n::text_args(
                    "seo-profile-detail-rich-result-types",
                    &[(
                        "types",
                        FluentValue::from(seo.structured_data.rich_snippets_potential.join(", ")),
                    )],
                    en,
                )
            } else {
                t("seo-profile-detail-no-rich-result-types", en)
            }),
        ),
        check(
            "Organization/WebSite Schema",
            has_org,
            Some(t(
                if has_org {
                    "seo-profile-detail-present"
                } else {
                    "seo-profile-detail-missing"
                },
                en,
            )),
        ),
    ];

    // LocalBusiness-specific quality checks
    let local_businesses: Vec<_> = seo
        .structured_data
        .json_ld
        .iter()
        .filter(|s| {
            s.schema_type == "LocalBusiness"
                || s.schema_types
                    .iter()
                    .any(|t| t == "LocalBusiness" || t.ends_with("Business"))
        })
        .collect();

    if !local_businesses.is_empty() {
        let lb = &local_businesses[0].content;

        // NAP completeness: name + street address + postal code + locality + phone
        let has_street = !lb["address"]["streetAddress"].is_null();
        let has_postal = !lb["address"]["postalCode"].is_null();
        let has_locality = !lb["address"]["addressLocality"].is_null();
        let has_phone = !lb["telephone"].is_null();
        let nap_complete = has_street && has_postal && has_locality && has_phone;
        let nap_detail = if nap_complete {
            None
        } else {
            let missing: Vec<&str> = [
                (!has_street).then_some("streetAddress"),
                (!has_postal).then_some("postalCode"),
                (!has_locality).then_some("addressLocality"),
                (!has_phone).then_some("telephone"),
            ]
            .into_iter()
            .flatten()
            .collect();
            Some(list_text(
                "seo-profile-detail-nap-missing",
                &missing.join(", "),
                en,
            ))
        };
        checks.push(check(
            &t("seo-profile-check-local-business-nap", en),
            nap_complete,
            nap_detail,
        ));

        // Geo coordinates
        let has_geo = !lb["geo"].is_null()
            && (!lb["geo"]["latitude"].is_null() || !lb["geo"]["longitude"].is_null());
        checks.push(check(
            &t("seo-profile-check-local-business-geo", en),
            has_geo,
            (!has_geo).then(|| t("seo-profile-detail-geo-missing", en)),
        ));

        // Trust signals: sameAs (authority links to social profiles / Wikidata)
        let has_same_as = !lb["sameAs"].is_null();
        checks.push(check(
            &t("seo-profile-check-local-business-same-as", en),
            has_same_as,
            (!has_same_as).then(|| t("seo-profile-detail-same-as-missing", en)),
        ));

        // aggregateRating
        let has_rating = !lb["aggregateRating"].is_null();
        checks.push(check(
            "LocalBusiness aggregateRating",
            has_rating,
            (!has_rating).then(|| t("seo-profile-detail-rating-missing", en)),
        ));

        // Contradiction check: multiple conflicting LocalBusiness schemas
        if local_businesses.len() > 1 {
            let first_name = lb["name"].as_str().unwrap_or("");
            let consistent = local_businesses
                .iter()
                .all(|s| s.content["name"].as_str().unwrap_or("") == first_name);
            checks.push(check(
                &t("seo-profile-check-local-business-consistent", en),
                consistent,
                Some(count_text(
                    if consistent {
                        "seo-profile-detail-local-business-consistent"
                    } else {
                        "seo-profile-detail-local-business-conflicting"
                    },
                    local_businesses.len(),
                    en,
                )),
            ));
        }
    }

    let structural_issues: Vec<_> = seo
        .structured_data
        .schema_issues
        .iter()
        .filter(|issue| issue.issue_type.starts_with("jsonld_"))
        .collect();
    if !structural_issues.is_empty() {
        checks.push(check(
            &t("seo-profile-check-jsonld-structure", en),
            false,
            Some(
                structural_issues
                    .iter()
                    .map(|issue| crate::seo::schema::schema_issue_text(issue, en))
                    .collect::<Vec<_>>()
                    .join("; "),
            ),
        ));
    }

    for assessment in &seo.structured_data.rule_assessments {
        use crate::seo::schema_rules::SchemaRequirementStatus;

        if matches!(
            assessment.requirement_status,
            SchemaRequirementStatus::NotEvaluated | SchemaRequirementStatus::RecommendationsOnly
        ) {
            continue;
        }
        let passed =
            assessment.requirement_status != SchemaRequirementStatus::MissingRequiredProperties;
        let detail = if !assessment.missing_required.is_empty() {
            list_text(
                "seo-profile-detail-missing-required",
                &assessment.missing_required.join(", "),
                en,
            )
        } else if !assessment.missing_recommended.is_empty() {
            list_text(
                "seo-profile-detail-missing-recommended",
                &assessment.missing_recommended.join(", "),
                en,
            )
        } else {
            crate::seo::schema_rules::status_text(assessment, en)
        };
        checks.push(check(
            &format!(
                "{}: {}",
                assessment.schema_type,
                crate::seo::schema_rules::feature_label(assessment.feature, en)
            ),
            passed,
            Some(detail),
        ));
    }

    let (passed, total) = category_counts(&checks);
    SignalCategory {
        name: t("seo-profile-category-structured-data", en),
        passed,
        total,
        checks,
    }
}

fn build_content_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let wc = seo.technical.word_count;
    let il = seo.technical.internal_links;
    let link_density = if wc > 0 { il as f32 / wc as f32 } else { 0.0 };

    let checks = vec![
        check(
            &t("seo-profile-check-text-volume", en),
            wc >= 300,
            Some(count_text("seo-profile-detail-words", wc as usize, en)),
        ),
        check(
            &t("seo-profile-check-internal-linking", en),
            il >= 3,
            Some(count_text(
                "seo-profile-detail-internal-links",
                il as usize,
                en,
            )),
        ),
        check(
            &t("seo-profile-check-link-density", en),
            link_density > 0.005,
            Some(format!(
                "{}%",
                crate::i18n::number::decimal(f64::from(link_density * 100.0), 2, en)
            )),
        ),
    ];
    let (passed, total) = category_counts(&checks);
    SignalCategory {
        name: t("seo-profile-category-content-base", en),
        passed,
        total,
        checks,
    }
}

// ─── Maturity Techniques Counter ────────────────────────────────────────────

fn count_techniques(seo: &SeoAnalysis) -> u32 {
    let mut count = 0u32;

    // 1. Title optimiert (30-60 Zeichen)
    if seo
        .meta
        .title
        .as_ref()
        .map(|t| web_checks::meta::TITLE.classify(t) == LengthClass::Ok)
        .unwrap_or(false)
    {
        count += 1;
    }
    // 2. Description optimiert (120-160 Zeichen)
    if seo
        .meta
        .description
        .as_ref()
        .map(|d| web_checks::meta::DESCRIPTION.classify(d) == LengthClass::Ok)
        .unwrap_or(false)
    {
        count += 1;
    }
    // 3. OpenGraph komplett (≥80%)
    if seo
        .social
        .open_graph
        .as_ref()
        .map(|og| og.completeness() >= 80)
        .unwrap_or(false)
    {
        count += 1;
    }
    // 4. Twitter Card komplett (≥75%)
    if seo
        .social
        .twitter_card
        .as_ref()
        .map(|tc| tc.completeness() >= 75)
        .unwrap_or(false)
    {
        count += 1;
    }
    // 5. JSON-LD vorhanden
    if seo.structured_data.has_structured_data {
        count += 1;
    }
    // 6. HTTPS
    if seo.technical.https {
        count += 1;
    }
    // 7. Canonical URL
    if seo.technical.has_canonical {
        count += 1;
    }
    // 8. Sprache gesetzt
    if seo.technical.has_lang {
        count += 1;
    }
    // 9. Korrekte Überschriften-Hierarchie
    if seo.headings.h1_count == 1 && seo.headings.issues.is_empty() {
        count += 1;
    }
    // 10. Inhaltstiefe (≥300 Wörter)
    if seo.technical.word_count >= 300 {
        count += 1;
    }
    // 11. Interne Verlinkung (≥3)
    if seo.technical.internal_links >= 3 {
        count += 1;
    }
    // 12. Rich-result-related schema type detected (not an eligibility claim)
    if !seo.structured_data.rich_snippets_potential.is_empty() {
        count += 1;
    }
    // 13. Hreflang für Mehrsprachigkeit
    if seo.technical.has_hreflang {
        count += 1;
    }

    count
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seo::headings::HeadingInfo;
    use crate::seo::*;

    fn minimal_seo() -> SeoAnalysis {
        SeoAnalysis {
            indexability: Default::default(),
            page_intent: None,
            meta: MetaTags::default(),
            meta_issues: vec![],
            headings: HeadingStructure {
                h1_count: 0,
                h1_text: None,
                headings: vec![],
                issues: vec![],
                total_count: 0,
            },
            social: SocialTags {
                open_graph: None,
                twitter_card: None,
                completeness: 0,
            },
            technical: technical::TechnicalSeo::default(),
            structured_data: StructuredData::default(),
            score: 0,
            content_profile: None,
            robots: None,
            page_health: None,
            serp: None,
            image_efficiency: None,
        }
    }

    /// #952-6/8: a missing Twitter Card is "fehlt", never an empty detail, and
    /// one ordering problem is "1 Problem".
    #[test]
    fn social_and_heading_details_are_never_empty_or_plural_wrong() {
        let seo = minimal_seo();
        let social = build_social_signals(&seo, false);
        assert!(social
            .checks
            .iter()
            .all(|c| c.detail.as_deref().is_some_and(|d| !d.is_empty())));

        let heading = |level: u8| HeadingInfo {
            level,
            text: format!("H{level}"),
            length: 2,
            is_question: false,
            in_faq_context: false,
            word_count_after: 0,
            hidden: false,
        };
        let mut seo = minimal_seo();
        seo.headings.h1_count = 1;
        seo.headings.h1_text = Some("H1".into());
        seo.headings.headings = vec![heading(2), heading(1), heading(2)];
        let de = build_heading_signals(&seo, false);
        let detail = de
            .checks
            .iter()
            .find(|c| c.label == "Logische Hierarchie")
            .and_then(|c| c.detail.clone())
            .expect("detail");
        assert!(detail.starts_with("1 Problem,"), "{detail}");
    }

    /// #952-1: a lone H1 that is not the first heading is no "logical
    /// hierarchy", whatever the skip-level rule says.
    #[test]
    fn heading_signals_flag_a_first_heading_that_is_not_the_h1() {
        let heading = |level: u8| HeadingInfo {
            level,
            text: format!("H{level}"),
            length: 2,
            is_question: false,
            in_faq_context: false,
            word_count_after: 0,
            hidden: false,
        };
        let mut seo = minimal_seo();
        seo.headings.h1_count = 1;
        seo.headings.h1_text = Some("H1".into());
        seo.headings.headings = vec![heading(2), heading(1), heading(2)];
        let category = build_heading_signals(&seo, true);
        let hierarchy = category
            .checks
            .iter()
            .find(|c| c.label == "Logical hierarchy")
            .expect("hierarchy check");
        assert!(!hierarchy.passed, "{hierarchy:?}");
        assert!(
            hierarchy
                .detail
                .as_deref()
                .is_some_and(|d| d.contains("first heading is H2")),
            "{hierarchy:?}"
        );

        // A hidden heading in front (a closed mega menu) is met by nobody: the
        // first heading a reader gets is the H1 (casoon.de, #952-1).
        let mut hidden_first = heading(2);
        hidden_first.hidden = true;
        seo.headings.headings = vec![hidden_first, heading(1), heading(2)];
        assert_eq!(seo.headings.first_exposed_level(), Some(1));
        let ok = build_heading_signals(&seo, true);
        assert!(ok
            .checks
            .iter()
            .all(|c| c.label != "Logical hierarchy" || c.passed));

        seo.headings.headings = vec![heading(1), heading(2)];
        let ok = build_heading_signals(&seo, true);
        assert!(ok
            .checks
            .iter()
            .all(|c| c.label != "Logical hierarchy" || c.passed));
    }

    #[test]
    fn test_maturity_basic_with_no_data() {
        let seo = minimal_seo();
        let profile = build_content_profile(&seo, "de");
        assert_eq!(profile.maturity, SeoMaturityLevel::Basic);
        assert_eq!(profile.maturity_techniques, 0);
    }

    #[test]
    fn test_content_identity_from_meta() {
        let mut seo = minimal_seo();
        seo.meta.title = Some("Casoon - CSS Framework & Design System".to_string());
        seo.meta.description = Some("Casoon bietet ein modernes CSS Framework.".to_string());
        seo.technical.lang = Some("de".to_string());

        let profile = build_content_profile(&seo, "de");
        assert!(profile.content_identity.summary.contains("Casoon"));
        assert_eq!(profile.content_identity.language, Some("de".to_string()));
        assert_eq!(profile.content_identity.content_type, "Website");
    }

    #[test]
    fn test_schema_inventory_faq() {
        let mut seo = minimal_seo();
        let faq_content = serde_json::json!({
            "@type": "FAQPage",
            "mainEntity": [
                {"@type": "Question", "name": "Was ist Casoon?", "acceptedAnswer": {"text": "Ein Framework."}},
                {"@type": "Question", "name": "Wie installiere ich es?", "acceptedAnswer": {"text": "Via npm."}},
            ]
        });
        seo.structured_data.json_ld.push(schema::JsonLdSchema {
            schema_type: "FAQPage".to_string(),
            schema_types: vec!["FAQPage".to_string()],
            content: faq_content,
            is_valid: true,
        });
        seo.structured_data.types.push(SchemaType::FAQPage);
        seo.structured_data.has_structured_data = true;

        let profile = build_content_profile(&seo, "de");
        assert_eq!(profile.schema_inventory.total_count, 1);

        let faq = &profile.schema_inventory.schemas[0];
        assert_eq!(faq.schema_type, "FAQPage");
        if let SchemaExtracted::FAQPage {
            question_count,
            questions,
        } = &faq.extracted
        {
            assert_eq!(*question_count, 2);
            assert_eq!(questions[0], "Was ist Casoon?");
        } else {
            panic!("Expected FAQPage extraction");
        }
    }

    #[test]
    fn test_schema_inventory_uses_flattened_graph_nodes_once() {
        let mut seo = minimal_seo();
        seo.structured_data = crate::seo::schema::analyze_structured_data_payloads(
            &[serde_json::json!({
                "@context": "https://schema.org",
                "@graph": [
                    {"@type": "WebSite", "name": "Example", "url": "https://example.com"},
                    {"@type": "WebPage", "name": "About", "url": "https://example.com/about"}
                ]
            })
            .to_string()],
            false,
            false,
        );

        let profile = build_content_profile(&seo, "en");

        assert_eq!(profile.schema_inventory.total_count, 2);
        assert_eq!(profile.schema_inventory.schemas[0].schema_type, "WebSite");
        assert_eq!(profile.schema_inventory.schemas[1].schema_type, "WebPage");
    }

    #[test]
    fn test_signal_strength_perfect() {
        let mut seo = minimal_seo();
        seo.meta.title = Some("Ein optimaler Titel der genau passt!!!!".to_string()); // 40 chars
        seo.meta.description = Some("Eine perfekte Beschreibung die zwischen 120 und 160 Zeichen liegt und alles sagt was man wissen muss über den Inhalt.".to_string());
        seo.meta.keywords = Some("test".to_string());
        seo.meta.viewport = Some("width=device-width".to_string());
        seo.meta.charset = Some("UTF-8".to_string());
        seo.meta.lang = Some("de".to_string());
        seo.headings.h1_count = 1;
        seo.headings.h1_text = Some("Hauptüberschrift".to_string());
        seo.social.open_graph = Some(OpenGraph {
            title: Some("T".into()),
            description: Some("D".into()),
            image: Some("I".into()),
            url: Some("U".into()),
            og_type: Some("website".into()),
            site_name: Some("S".into()),
            locale: Some("L".into()),
        });
        seo.social.twitter_card = Some(TwitterCard {
            card: Some("summary".into()),
            title: Some("T".into()),
            description: Some("D".into()),
            image: Some("I".into()),
            site: Some("S".into()),
            creator: Some("C".into()),
        });
        seo.technical.https = true;
        seo.technical.has_canonical = true;
        seo.technical.has_lang = true;
        seo.technical.word_count = 500;
        seo.technical.internal_links = 10;
        seo.structured_data.has_structured_data = true;
        seo.structured_data.types.push(SchemaType::Organization);
        seo.structured_data
            .rich_snippets_potential
            .push("Test".to_string());

        let profile = build_content_profile(&seo, "de");

        // All signal categories should be high
        for cat in &profile.signal_strength.categories {
            assert!(
                cat.pct() >= 75,
                "{} only {} of {}",
                cat.name,
                cat.passed,
                cat.total
            );
        }
        assert!(profile.signal_strength.overall_pct >= 80);
        assert!(matches!(
            profile.maturity,
            SeoMaturityLevel::Advanced | SeoMaturityLevel::Professional
        ));
    }

    #[test]
    fn test_schema_inventory_handles_webpage_and_professional_service() {
        let mut seo = minimal_seo();
        let webpage = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": "CASOON Startseite",
            "description": "Webentwicklung und digitale Systeme.",
            "url": "https://www.casoon.de/",
            "image": "https://www.casoon.de/og.webp",
            "inLanguage": "de-DE",
            "author": {"@type": "Person", "name": "Jörn Seidel"},
            "publisher": {"@type": "Organization", "name": "CASOON"}
        });
        let service = serde_json::json!({
            "@context": "https://schema.org",
            "@type": ["ProfessionalService", "LocalBusiness"],
            "name": "CASOON",
            "description": "Webentwicklung und digitale Systeme.",
            "url": "https://www.casoon.de/",
            "telephone": "+49 381 448840",
            "address": {"@type": "PostalAddress", "streetAddress": "Zur Häusler-Reihe 10"},
            "serviceType": ["Webentwicklung", "SEO"],
            "areaServed": [{"@type": "City", "name": "Rostock"}],
            "priceRange": "$$"
        });

        seo.structured_data.json_ld.push(schema::JsonLdSchema {
            schema_type: "WebPage".to_string(),
            schema_types: vec!["WebPage".to_string()],
            content: webpage,
            is_valid: true,
        });
        seo.structured_data.json_ld.push(schema::JsonLdSchema {
            schema_type: "ProfessionalService".to_string(),
            schema_types: vec![
                "ProfessionalService".to_string(),
                "LocalBusiness".to_string(),
            ],
            content: service,
            is_valid: true,
        });
        seo.structured_data.types.push(SchemaType::WebPage);
        seo.structured_data
            .types
            .push(SchemaType::Other("ProfessionalService".to_string()));
        seo.structured_data.types.push(SchemaType::LocalBusiness);
        seo.structured_data.has_structured_data = true;

        let profile = build_content_profile(&seo, "de");
        assert_eq!(profile.schema_inventory.total_count, 2);

        let webpage_detail = profile
            .schema_inventory
            .schemas
            .iter()
            .find(|s| s.schema_type == "WebPage")
            .expect("WebPage schema missing");
        assert!(webpage_detail.completeness_pct >= 85);

        let service_detail = profile
            .schema_inventory
            .schemas
            .iter()
            .find(|s| s.schema_type == "ProfessionalService")
            .expect("ProfessionalService schema missing");
        assert!(service_detail.completeness_pct >= 75);
    }

    #[test]
    fn test_maturity_levels() {
        assert_eq!(SeoMaturityLevel::from_count(0), SeoMaturityLevel::Basic);
        assert_eq!(SeoMaturityLevel::from_count(3), SeoMaturityLevel::Basic);
        assert_eq!(SeoMaturityLevel::from_count(4), SeoMaturityLevel::Standard);
        assert_eq!(SeoMaturityLevel::from_count(7), SeoMaturityLevel::Advanced);
        assert_eq!(
            SeoMaturityLevel::from_count(10),
            SeoMaturityLevel::Professional
        );
    }

    #[test]
    fn test_classify_editorial_page() {
        let mut seo = minimal_seo();
        seo.technical.word_count = 1800;
        seo.headings.headings = vec![
            HeadingInfo {
                level: 1,
                text: "Titel".into(),
                length: 5,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
                hidden: false,
            },
            HeadingInfo {
                level: 2,
                text: "A".into(),
                length: 1,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
                hidden: false,
            },
            HeadingInfo {
                level: 2,
                text: "B".into(),
                length: 1,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
                hidden: false,
            },
            HeadingInfo {
                level: 3,
                text: "C".into(),
                length: 1,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
                hidden: false,
            },
        ];
        seo.headings.total_count = seo.headings.headings.len();
        seo.structured_data.types.push(SchemaType::Article);

        let profile = build_content_profile(&seo, "de");
        assert_eq!(
            profile.page_classification.primary_type,
            PageType::Editorial
        );
        assert!(profile
            .page_classification
            .attributes
            .contains(&"textstark".to_string()));
    }

    #[test]
    fn test_classify_thin_page() {
        let mut seo = minimal_seo();
        seo.technical.word_count = 120;
        let profile = build_content_profile(&seo, "de");
        assert_eq!(
            profile.page_classification.primary_type,
            PageType::ThinContent
        );
        assert!(profile.page_classification.content_depth_score < 40);
    }

    fn has_german_chars(s: &str) -> bool {
        s.chars().any(|c| "äöüÄÖÜß".contains(c))
    }

    #[test]
    fn test_enum_labels_have_no_german_chars_in_en() {
        for level in [
            SeoMaturityLevel::Basic,
            SeoMaturityLevel::Standard,
            SeoMaturityLevel::Advanced,
            SeoMaturityLevel::Professional,
        ] {
            assert!(
                !has_german_chars(&level.label(true)),
                "German chars in maturity label: {}",
                level.label(true)
            );
            assert!(
                !has_german_chars(&level.description(true)),
                "German chars in maturity description: {}",
                level.description(true)
            );
        }
        for pt in [
            PageType::Editorial,
            PageType::StructuredContent,
            PageType::MarketingLanding,
            PageType::MediaHeavy,
            PageType::Utility,
            PageType::NavigationHub,
            PageType::ThinContent,
        ] {
            assert!(
                !has_german_chars(&pt.label(true)),
                "German chars in page type label: {}",
                pt.label(true)
            );
        }
    }

    #[test]
    fn test_profile_strings_have_no_german_chars_in_en() {
        // Populate enough signals to exercise most string-producing branches,
        // including the LocalBusiness-specific checks and a thin/short page so
        // both passed and failed details are covered.
        let mut seo = minimal_seo();
        seo.technical.word_count = 120; // thin → "very short", failed content checks
        seo.technical.https = false;
        seo.technical.has_hreflang = false;
        let lb = serde_json::json!({
            "@type": "LocalBusiness",
            "name": "Example",
        });
        seo.structured_data.json_ld.push(schema::JsonLdSchema {
            schema_type: "LocalBusiness".to_string(),
            schema_types: vec!["LocalBusiness".to_string()],
            content: lb,
            is_valid: true,
        });
        seo.structured_data.types.push(SchemaType::LocalBusiness);
        seo.structured_data.has_structured_data = true;

        let profile = build_content_profile(&seo, "en");

        assert!(
            !has_german_chars(&profile.content_identity.content_type),
            "German chars in content_type: {}",
            profile.content_identity.content_type
        );
        assert!(
            !has_german_chars(&profile.content_identity.summary),
            "German chars in summary: {}",
            profile.content_identity.summary
        );
        for attr in &profile.page_classification.attributes {
            assert!(
                !has_german_chars(attr),
                "German chars in attribute: {}",
                attr
            );
        }
        for category in &profile.signal_strength.categories {
            assert!(
                !has_german_chars(&category.name),
                "German chars in category name: {}",
                category.name
            );
            for chk in &category.checks {
                assert!(
                    !has_german_chars(&chk.label),
                    "German chars in check label: {}",
                    chk.label
                );
                if let Some(detail) = &chk.detail {
                    assert!(
                        !has_german_chars(detail),
                        "German chars in check detail: {}",
                        detail
                    );
                }
            }
        }
    }
}
