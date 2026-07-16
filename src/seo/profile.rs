//! SEO Content Profile — technical content and signal profile
//!
//! Analysiert die vorhandenen SEO-Rohdaten (Meta, Headings, JSON-LD, Social, Technical)
//! und erzeugt eine kompakte Zusammenfassung: Was ist die Seite, welche technischen
//! SEO-Signale werden eingesetzt, wie vollständig sind strukturierte Daten.

use serde::{Deserialize, Serialize};

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
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            Self::Basic => {
                if en {
                    "Basic"
                } else {
                    "Basis"
                }
            }
            Self::Standard => "Standard",
            Self::Advanced => {
                if en {
                    "Advanced"
                } else {
                    "Fortgeschritten"
                }
            }
            Self::Professional => {
                if en {
                    "Professional"
                } else {
                    "Professionell"
                }
            }
        }
    }

    pub fn description(&self, en: bool) -> &'static str {
        match self {
            Self::Basic => {
                if en {
                    "Fundamental SEO measures are largely absent."
                } else {
                    "Grundlegende SEO-Maßnahmen fehlen weitgehend."
                }
            }
            Self::Standard => {
                if en {
                    "Basic SEO is in place; advanced techniques are missing."
                } else {
                    "Basis-SEO ist vorhanden, fortgeschrittene Techniken fehlen."
                }
            }
            Self::Advanced => {
                if en {
                    "Good SEO coverage with structured data and social tags."
                } else {
                    "Gute SEO-Abdeckung mit strukturierten Daten und Social Tags."
                }
            }
            Self::Professional => {
                if en {
                    "Very broad technical SEO coverage; no statement on strategic content quality."
                } else {
                    "Sehr breite technische SEO-Abdeckung; keine Aussage zur strategischen Content-Qualität."
                }
            }
        }
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
    pub fn label(&self, en: bool) -> &'static str {
        match self {
            Self::Editorial => {
                if en {
                    "Editorial / Article"
                } else {
                    "Editorial / Artikel"
                }
            }
            Self::StructuredContent => {
                if en {
                    "Structured knowledge content"
                } else {
                    "Strukturierter Wissensinhalt"
                }
            }
            Self::MarketingLanding => "Marketing / Landing Page",
            Self::MediaHeavy => {
                if en {
                    "Media-oriented page"
                } else {
                    "Medienorientierte Seite"
                }
            }
            Self::Utility => {
                if en {
                    "Transactional / Utility"
                } else {
                    "Transaktional / Utility"
                }
            }
            Self::NavigationHub => {
                if en {
                    "Navigation / Hub page"
                } else {
                    "Navigations- / Hub-Seite"
                }
            }
            Self::ThinContent => "Thin / Minimal Content",
        }
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
    pub overall_pct: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCategory {
    pub name: String,
    pub score_pct: u32,
    pub checks: Vec<SignalCheck>,
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
    let mut attributes = Vec::new();
    if word_count >= 1200 {
        attributes.push(if en { "text-rich" } else { "textstark" }.to_string());
    } else if word_count < 300 {
        attributes.push(if en { "very short" } else { "sehr kurz" }.to_string());
    }
    if h2_plus_count >= 3 {
        attributes.push(if en { "structured" } else { "strukturiert" }.to_string());
    }
    if internal_links >= 30 {
        attributes.push(
            if en {
                "navigation-heavy"
            } else {
                "navigationslastig"
            }
            .to_string(),
        );
    }
    if seo.social.open_graph.is_some() {
        attributes.push(
            if en {
                "visually oriented"
            } else {
                "visuell geprägt"
            }
            .to_string(),
        );
    }
    if has_faq {
        attributes.push(
            if en {
                "knowledge-oriented"
            } else {
                "wissensorientiert"
            }
            .to_string(),
        );
    }
    if matches!(primary_type, PageType::MarketingLanding) {
        attributes.push(
            if en {
                "conversion-oriented"
            } else {
                "conversionorientiert"
            }
            .to_string(),
        );
    }
    if matches!(primary_type, PageType::Utility) {
        attributes.push(if en { "purpose-bound" } else { "zweckgebunden" }.to_string());
    }
    if matches!(primary_type, PageType::ThinContent) {
        attributes.push(if en { "thin" } else { "dünn" }.to_string());
    }
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
        .map(|t| format!("{:?}", t))
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
                "article" => return if en { "Article" } else { "Artikel" }.to_string(),
                "product" => return if en { "Product" } else { "Produkt" }.to_string(),
                "profile" => return if en { "Profile" } else { "Profil" }.to_string(),
                "website" => {} // generic, check schema types
                _ => {}
            }
        }
    }

    // Check JSON-LD types
    for t in &seo.structured_data.types {
        match t {
            SchemaType::Article | SchemaType::BlogPosting | SchemaType::NewsArticle => {
                return if en {
                    "Article / Blog"
                } else {
                    "Artikel / Blog"
                }
                .to_string()
            }
            SchemaType::Product => {
                return if en {
                    "E-Commerce / Product"
                } else {
                    "E-Commerce / Produkt"
                }
                .to_string()
            }
            SchemaType::LocalBusiness => {
                return if en {
                    "Local business"
                } else {
                    "Lokales Unternehmen"
                }
                .to_string()
            }
            SchemaType::Event => return if en { "Event" } else { "Veranstaltung" }.to_string(),
            SchemaType::Recipe => return if en { "Recipe" } else { "Rezept" }.to_string(),
            SchemaType::FAQPage => {
                return if en {
                    "FAQ / Information page"
                } else {
                    "FAQ / Informationsseite"
                }
                .to_string()
            }
            _ => {}
        }
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
    } else if en {
        "No meta information found.".to_string()
    } else {
        "Keine Meta-Informationen gefunden.".to_string()
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
                    if en {
                        format!("[{} entries]", a.len())
                    } else {
                        format!("[{} Einträge]", a.len())
                    }
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
    let overall_pct = (meta.score_pct as f32 * 0.20
        + headings.score_pct as f32 * 0.15
        + social.score_pct as f32 * 0.15
        + technical.score_pct as f32 * 0.20
        + structured.score_pct as f32 * 0.15
        + content.score_pct as f32 * 0.15) as u32;

    SeoSignalStrength {
        categories: vec![meta, headings, social, technical, structured, content],
        overall_pct,
    }
}

fn category_score(checks: &[SignalCheck]) -> u32 {
    if checks.is_empty() {
        return 0;
    }
    let passed = checks.iter().filter(|c| c.passed).count();
    (passed * 100 / checks.len()) as u32
}

fn check(label: &str, passed: bool, detail: Option<String>) -> SignalCheck {
    SignalCheck {
        label: label.to_string(),
        passed,
        detail,
    }
}

fn build_meta_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let title_ok = seo
        .meta
        .title
        .as_ref()
        .map(|t| t.len() >= 30 && t.len() <= 60)
        .unwrap_or(false);
    let desc_ok = seo
        .meta
        .description
        .as_ref()
        .map(|d| d.len() >= 120 && d.len() <= 160)
        .unwrap_or(false);

    let chars = |n: usize| {
        if en {
            format!("{} characters", n)
        } else {
            format!("{} Zeichen", n)
        }
    };
    let missing = || if en { "missing" } else { "fehlt" }.to_string();

    let checks = vec![
        check(
            if en {
                "Title present & length optimal"
            } else {
                "Title vorhanden & Länge optimal"
            },
            title_ok,
            seo.meta.title.as_ref().map(|t| chars(t.len())),
        ),
        check(
            if en {
                "Description present & length optimal"
            } else {
                "Description vorhanden & Länge optimal"
            },
            desc_ok,
            seo.meta.description.as_ref().map(|d| chars(d.len())),
        ),
        check(
            if en {
                "Viewport configured"
            } else {
                "Viewport konfiguriert"
            },
            seo.meta.viewport.is_some(),
            Some(seo.meta.viewport.clone().unwrap_or_else(missing)),
        ),
        check(
            if en {
                "Charset defined"
            } else {
                "Charset definiert"
            },
            seo.meta.charset.is_some(),
            Some(seo.meta.charset.clone().unwrap_or_else(missing)),
        ),
        check(
            if en {
                "Language (lang) set"
            } else {
                "Sprache (lang) gesetzt"
            },
            seo.meta.lang.is_some(),
            seo.meta.lang.clone(),
        ),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: "Meta-Tags".to_string(),
        score_pct,
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
    let no_issues = seo.headings.issues.is_empty();

    let checks = vec![
        check(
            if en { "Exactly one H1" } else { "Genau ein H1" },
            h1_ok,
            Some(if en {
                format!("{} H1 tags found", seo.headings.h1_count)
            } else {
                format!("{} H1-Tags gefunden", seo.headings.h1_count)
            }),
        ),
        check(
            if en {
                "H1 text present"
            } else {
                "H1-Text vorhanden"
            },
            h1_not_empty,
            Some(
                seo.headings
                    .h1_text
                    .clone()
                    .unwrap_or_else(|| if en { "no H1 text" } else { "kein H1-Text" }.to_string()),
            ),
        ),
        check(
            if en {
                "No skipped levels"
            } else {
                "Keine übersprungenen Ebenen"
            },
            no_skipped,
            Some(
                if no_skipped {
                    if en {
                        "no gaps"
                    } else {
                        "keine Lücken"
                    }
                } else if en {
                    "levels skipped"
                } else {
                    "Ebenen übersprungen"
                }
                .to_string(),
            ),
        ),
        check(
            if en {
                "Logical hierarchy"
            } else {
                "Logische Hierarchie"
            },
            no_issues,
            Some(if no_issues {
                if en { "no issues" } else { "keine Probleme" }.to_string()
            } else if en {
                format!("{} issues", seo.headings.issues.len())
            } else {
                format!("{} Probleme", seo.headings.issues.len())
            }),
        ),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: if en { "Headings" } else { "Überschriften" }.to_string(),
        score_pct,
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
        if present {
            if en {
                "present"
            } else {
                "vorhanden"
            }
        } else if en {
            "missing"
        } else {
            "fehlt"
        }
        .to_string()
    };

    let checks = vec![
        check(
            if en {
                "OpenGraph tags present"
            } else {
                "OpenGraph-Tags vorhanden"
            },
            og_present,
            Some(present_or_missing(og_present)),
        ),
        check(
            if en {
                "OpenGraph ≥ 80% complete"
            } else {
                "OpenGraph ≥ 80% vollständig"
            },
            og_complete,
            seo.social
                .open_graph
                .as_ref()
                .map(|og| format!("{}%", og.completeness())),
        ),
        check(
            if en {
                "Twitter Card present"
            } else {
                "Twitter Card vorhanden"
            },
            tw_present,
            Some(present_or_missing(tw_present)),
        ),
        check(
            if en {
                "Twitter Card ≥ 75% complete"
            } else {
                "Twitter Card ≥ 75% vollständig"
            },
            tw_complete,
            seo.social
                .twitter_card
                .as_ref()
                .map(|tc| format!("{}%", tc.completeness())),
        ),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: "Social Media".to_string(),
        score_pct,
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

    let missing = || if en { "missing" } else { "fehlt" }.to_string();

    let checks = vec![
        check(
            "HTTPS",
            seo.technical.https,
            Some(
                if seo.technical.https {
                    if en {
                        "active"
                    } else {
                        "aktiv"
                    }
                } else if en {
                    "missing"
                } else {
                    "fehlt"
                }
                .to_string(),
            ),
        ),
        check(
            "Canonical URL",
            seo.technical.has_canonical,
            Some(seo.technical.canonical_url.clone().unwrap_or_else(missing)),
        ),
        check(
            if en {
                "Language (lang)"
            } else {
                "Sprache (lang)"
            },
            seo.technical.has_lang,
            Some(seo.technical.lang.clone().unwrap_or_else(missing)),
        ),
        check(
            if en {
                "Hreflang (multilingual)"
            } else {
                "Hreflang (Mehrsprachigkeit)"
            },
            true,
            Some(if seo.technical.has_hreflang {
                if en {
                    format!("{} languages", seo.technical.hreflang.len())
                } else {
                    format!("{} Sprachen", seo.technical.hreflang.len())
                }
            } else if en {
                "no hreflang tags".to_string()
            } else {
                "keine Hreflang-Tags".to_string()
            }),
        ),
        check(
            if en {
                "Robots allows indexing"
            } else {
                "Robots erlaubt Indexierung"
            },
            robots_ok,
            seo.technical.robots_meta.clone(),
        ),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: if en { "Technical" } else { "Technisch" }.to_string(),
        score_pct,
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
            if en {
                "Structured data present"
            } else {
                "Strukturierte Daten vorhanden"
            },
            has_any,
            Some(if !has_any {
                if en {
                    "no structured data".to_string()
                } else {
                    "keine strukturierten Daten".to_string()
                }
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
                    if en {
                        "JSON-LD detected but not evaluable".to_string()
                    } else {
                        "JSON-LD erkannt, aber nicht auswertbar".to_string()
                    }
                } else if en {
                    // has_structured_data is true via microdata/RDFa, but
                    // json_ld is empty — the count only tracks JSON-LD, so
                    // "0 Schemas" would contradict the passing checkmark above.
                    "structured data detected (microdata/RDFa, no JSON-LD)".to_string()
                } else {
                    "strukturierte Daten erkannt (Microdata/RDFa, kein JSON-LD)".to_string()
                }
            }),
        ),
        check(
            if en {
                "Rich-result type detected"
            } else {
                "Rich-Result-Typ erkannt"
            },
            has_rich,
            Some(if has_rich {
                if en {
                    format!(
                        "{} (type detection only; eligibility also requires complete, matching content)",
                        seo.structured_data.rich_snippets_potential.join(", ")
                    )
                } else {
                    format!(
                        "{} (nur Typ-Erkennung; Eignung erfordert zusätzlich vollständige, passende Inhalte)",
                        seo.structured_data.rich_snippets_potential.join(", ")
                    )
                }
            } else if en {
                "no rich-result types detected".to_string()
            } else {
                "keine Rich-Result-Typen erkannt".to_string()
            }),
        ),
        check(
            "Organization/WebSite Schema",
            has_org,
            Some(
                if has_org {
                    if en {
                        "present"
                    } else {
                        "vorhanden"
                    }
                } else if en {
                    "missing"
                } else {
                    "fehlt"
                }
                .to_string(),
            ),
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
            Some(if en {
                format!("Missing: {}", missing.join(", "))
            } else {
                format!("Fehlt: {}", missing.join(", "))
            })
        };
        checks.push(check(
            if en {
                "LocalBusiness NAP complete"
            } else {
                "LocalBusiness NAP vollständig"
            },
            nap_complete,
            nap_detail,
        ));

        // Geo coordinates
        let has_geo = !lb["geo"].is_null()
            && (!lb["geo"]["latitude"].is_null() || !lb["geo"]["longitude"].is_null());
        checks.push(check(
            if en {
                "LocalBusiness geo coordinates"
            } else {
                "LocalBusiness Geo-Koordinaten"
            },
            has_geo,
            if has_geo {
                None
            } else if en {
                Some("geo.latitude/longitude missing".to_string())
            } else {
                Some("geo.latitude/longitude fehlt".to_string())
            },
        ));

        // Trust signals: sameAs (authority links to social profiles / Wikidata)
        let has_same_as = !lb["sameAs"].is_null();
        checks.push(check(
            if en {
                "LocalBusiness sameAs (authority links)"
            } else {
                "LocalBusiness sameAs (Autoritätslinks)"
            },
            has_same_as,
            if has_same_as {
                None
            } else if en {
                Some("sameAs missing — prevents a knowledge panel".to_string())
            } else {
                Some("sameAs fehlt — verhindert Knowledge-Panel".to_string())
            },
        ));

        // aggregateRating
        let has_rating = !lb["aggregateRating"].is_null();
        checks.push(check(
            "LocalBusiness aggregateRating",
            has_rating,
            if has_rating {
                None
            } else if en {
                Some("aggregateRating missing — no stars in the SERP".to_string())
            } else {
                Some("aggregateRating fehlt — keine Sterne im SERP".to_string())
            },
        ));

        // Contradiction check: multiple conflicting LocalBusiness schemas
        if local_businesses.len() > 1 {
            let first_name = lb["name"].as_str().unwrap_or("");
            let consistent = local_businesses
                .iter()
                .all(|s| s.content["name"].as_str().unwrap_or("") == first_name);
            checks.push(check(
                if en {
                    "LocalBusiness schemas consistent"
                } else {
                    "LocalBusiness-Schemas konsistent"
                },
                consistent,
                if consistent {
                    if en {
                        Some(format!(
                            "{} schemas, name consistent",
                            local_businesses.len()
                        ))
                    } else {
                        Some(format!(
                            "{} Schemas, Name einheitlich",
                            local_businesses.len()
                        ))
                    }
                } else if en {
                    Some(format!(
                        "{} schemas with conflicting name",
                        local_businesses.len()
                    ))
                } else {
                    Some(format!(
                        "{} Schemas mit widersprüchlichem Namen",
                        local_businesses.len()
                    ))
                },
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
            if en {
                "JSON-LD syntax and structure"
            } else {
                "JSON-LD-Syntax und -Struktur"
            },
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
            if en {
                format!(
                    "Missing required: {}",
                    assessment.missing_required.join(", ")
                )
            } else {
                format!(
                    "Pflichtangaben fehlen: {}",
                    assessment.missing_required.join(", ")
                )
            }
        } else if !assessment.missing_recommended.is_empty() {
            if en {
                format!(
                    "Recommended properties not present: {}",
                    assessment.missing_recommended.join(", ")
                )
            } else {
                format!(
                    "Empfohlene Angaben nicht vorhanden: {}",
                    assessment.missing_recommended.join(", ")
                )
            }
        } else {
            assessment.status_text(en).to_string()
        };
        checks.push(check(
            &format!(
                "{}: {}",
                assessment.schema_type,
                assessment.feature.label(en)
            ),
            passed,
            Some(detail),
        ));
    }

    let score_pct = category_score(&checks);
    SignalCategory {
        name: if en {
            "Structured data"
        } else {
            "Strukturierte Daten"
        }
        .to_string(),
        score_pct,
        checks,
    }
}

fn build_content_signals(seo: &SeoAnalysis, en: bool) -> SignalCategory {
    let wc = seo.technical.word_count;
    let il = seo.technical.internal_links;
    let link_density = if wc > 0 { il as f32 / wc as f32 } else { 0.0 };

    let checks = vec![
        check(
            if en {
                "Readable text volume (≥ 300 words, guideline)"
            } else {
                "Lesbarer Textumfang (≥ 300 Wörter, Richtwert)"
            },
            wc >= 300,
            Some(if en {
                format!("{} words", wc)
            } else {
                format!("{} Wörter", wc)
            }),
        ),
        check(
            if en {
                "Internal linking (≥ 3 links)"
            } else {
                "Interne Verlinkung (≥ 3 Links)"
            },
            il >= 3,
            Some(if en {
                format!("{} internal links", il)
            } else {
                format!("{} interne Links", il)
            }),
        ),
        check(
            if en {
                "Link density > 0.5%"
            } else {
                "Linkdichte > 0,5%"
            },
            link_density > 0.005,
            Some(format!("{:.2}%", link_density * 100.0)),
        ),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: if en {
            "Technical content base"
        } else {
            "Technische Inhaltsbasis"
        }
        .to_string(),
        score_pct,
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
        .map(|t| t.len() >= 30 && t.len() <= 60)
        .unwrap_or(false)
    {
        count += 1;
    }
    // 2. Description optimiert (120-160 Zeichen)
    if seo
        .meta
        .description
        .as_ref()
        .map(|d| d.len() >= 120 && d.len() <= 160)
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
            assert!(cat.score_pct >= 75, "{} only {}%", cat.name, cat.score_pct);
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
            },
            HeadingInfo {
                level: 2,
                text: "A".into(),
                length: 1,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
            },
            HeadingInfo {
                level: 2,
                text: "B".into(),
                length: 1,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
            },
            HeadingInfo {
                level: 3,
                text: "C".into(),
                length: 1,
                is_question: false,
                in_faq_context: false,
                word_count_after: 0,
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
                !has_german_chars(level.label(true)),
                "German chars in maturity label: {}",
                level.label(true)
            );
            assert!(
                !has_german_chars(level.description(true)),
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
                !has_german_chars(pt.label(true)),
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
