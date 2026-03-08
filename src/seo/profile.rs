//! SEO Content Profile — Inhaltsanalyse & Strategiebewertung
//!
//! Analysiert die vorhandenen SEO-Rohdaten (Meta, Headings, JSON-LD, Social, Technical)
//! und erzeugt eine kompakte Zusammenfassung: Was ist die Seite, welche SEO-Strategien
//! werden eingesetzt, wie vollständig sind strukturierte Daten.

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
    pub fn label(&self) -> &'static str {
        match self {
            Self::Basic => "Basis",
            Self::Standard => "Standard",
            Self::Advanced => "Fortgeschritten",
            Self::Professional => "Professionell",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Basic => "Grundlegende SEO-Maßnahmen fehlen weitgehend.",
            Self::Standard => "Basis-SEO ist vorhanden, fortgeschrittene Techniken fehlen.",
            Self::Advanced => "Gute SEO-Abdeckung mit strukturierten Daten und Social Tags.",
            Self::Professional => "Umfassende SEO-Strategie mit vollständiger Technik-Abdeckung.",
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
    pub schema_inventory: SchemaInventory,
    pub signal_strength: SeoSignalStrength,
    pub maturity: SeoMaturityLevel,
    pub maturity_techniques: u32,
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
        phone: Option<String>,
        opening_hours: Vec<String>,
        price_range: Option<String>,
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

pub fn build_content_profile(seo: &SeoAnalysis) -> SeoContentProfile {
    let content_identity = build_identity(seo);
    let schema_inventory = build_schema_inventory(seo);
    let signal_strength = build_signal_strength(seo);
    let maturity_techniques = count_techniques(seo);
    let maturity = SeoMaturityLevel::from_count(maturity_techniques);

    SeoContentProfile {
        content_identity,
        schema_inventory,
        signal_strength,
        maturity,
        maturity_techniques,
    }
}

// ─── Content Identity Builder ───────────────────────────────────────────────

fn build_identity(seo: &SeoAnalysis) -> ContentIdentity {
    // Site name: OG site_name → JSON-LD Organization name → None
    let site_name = seo
        .social
        .open_graph
        .as_ref()
        .and_then(|og| og.site_name.clone())
        .or_else(|| find_org_name(&seo.structured_data.json_ld));

    // Content type from OG type → JSON-LD @type → default
    let content_type = derive_content_type(seo);

    // Summary from title + description
    let summary = build_summary(seo);

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
        if schema.schema_type == "Organization" || schema.schema_type == "LocalBusiness" {
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

fn derive_content_type(seo: &SeoAnalysis) -> String {
    // Check OG type first
    if let Some(ref og) = seo.social.open_graph {
        if let Some(ref og_type) = og.og_type {
            match og_type.as_str() {
                "article" => return "Artikel".to_string(),
                "product" => return "Produkt".to_string(),
                "profile" => return "Profil".to_string(),
                "website" => {} // generic, check schema types
                _ => {}
            }
        }
    }

    // Check JSON-LD types
    for t in &seo.structured_data.types {
        match t {
            SchemaType::Article | SchemaType::BlogPosting | SchemaType::NewsArticle => {
                return "Artikel / Blog".to_string()
            }
            SchemaType::Product => return "E-Commerce / Produkt".to_string(),
            SchemaType::LocalBusiness => return "Lokales Unternehmen".to_string(),
            SchemaType::Event => return "Veranstaltung".to_string(),
            SchemaType::Recipe => return "Rezept".to_string(),
            SchemaType::FAQPage => return "FAQ / Informationsseite".to_string(),
            _ => {}
        }
    }

    "Website".to_string()
}

fn build_summary(seo: &SeoAnalysis) -> String {
    let title = seo.meta.title.as_deref().unwrap_or("");
    let desc = seo.meta.description.as_deref().unwrap_or("");

    if !title.is_empty() && !desc.is_empty() {
        let combined = format!("{} — {}", title, desc);
        if combined.len() > 150 {
            format!("{}...", &combined[..147])
        } else {
            combined
        }
    } else if !title.is_empty() {
        title.to_string()
    } else if !desc.is_empty() {
        desc.to_string()
    } else {
        "Keine Meta-Informationen gefunden.".to_string()
    }
}

// ─── Schema Inventory Builder ───────────────────────────────────────────────

fn build_schema_inventory(seo: &SeoAnalysis) -> SchemaInventory {
    let mut schemas = Vec::new();

    for json_ld in &seo.structured_data.json_ld {
        if !json_ld.is_valid {
            continue;
        }
        // Process the root level
        let detail = analyze_schema(&json_ld.schema_type, &json_ld.content);
        if let Some(d) = detail {
            schemas.push(d);
        }

        // Process @graph items
        if let Some(graph) = json_ld.content["@graph"].as_array() {
            for item in graph {
                let item_type = item["@type"].as_str().unwrap_or("Unknown");
                if let Some(d) = analyze_schema(item_type, item) {
                    schemas.push(d);
                }
            }
        }
    }

    let total_count = schemas.len();
    SchemaInventory {
        schemas,
        total_count,
    }
}

fn analyze_schema(schema_type: &str, content: &serde_json::Value) -> Option<SchemaDetail> {
    let (expected, extracted) = match schema_type {
        "Organization" => analyze_organization(content),
        "LocalBusiness" => analyze_local_business(content),
        "Article" | "BlogPosting" | "NewsArticle" => analyze_article(content),
        "FAQPage" => analyze_faq(content),
        "Product" => analyze_product(content),
        "WebSite" => analyze_website(content),
        "BreadcrumbList" => analyze_breadcrumb(content),
        _ => analyze_generic(content),
    };

    let mut present = Vec::new();
    let mut missing = Vec::new();
    for field in &expected {
        if !content[field].is_null() {
            present.push(field.to_string());
        } else {
            missing.push(field.to_string());
        }
    }

    let total = present.len() + missing.len();
    let pct = if total > 0 {
        (present.len() * 100 / total) as u32
    } else {
        0
    };

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

fn analyze_organization(
    v: &serde_json::Value,
) -> (Vec<&'static str>, SchemaExtracted) {
    let expected = vec!["name", "url", "logo", "address", "telephone", "sameAs"];

    let address = v["address"]["streetAddress"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "address"));

    let extracted = SchemaExtracted::Organization {
        name: str_opt(v, "name"),
        url: str_opt(v, "url"),
        logo: v["logo"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| str_opt(&v["logo"], "url")),
        address,
        phone: str_opt(v, "telephone"),
    };
    (expected, extracted)
}

fn analyze_local_business(
    v: &serde_json::Value,
) -> (Vec<&'static str>, SchemaExtracted) {
    let expected = vec![
        "name",
        "address",
        "telephone",
        "openingHours",
        "priceRange",
    ];

    let address = v["address"]["streetAddress"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "address"));

    let opening_hours = v["openingHours"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|h| h.as_str().map(|s| s.to_string()))
                .collect()
        })
        .or_else(|| {
            v["openingHours"]
                .as_str()
                .map(|s| vec![s.to_string()])
        })
        .unwrap_or_default();

    let extracted = SchemaExtracted::LocalBusiness {
        name: str_opt(v, "name"),
        address,
        phone: str_opt(v, "telephone"),
        opening_hours,
        price_range: str_opt(v, "priceRange"),
    };
    (expected, extracted)
}

fn analyze_article(
    v: &serde_json::Value,
) -> (Vec<&'static str>, SchemaExtracted) {
    let expected = vec![
        "headline",
        "author",
        "datePublished",
        "dateModified",
        "publisher",
        "image",
    ];

    let author = v["author"]["name"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "author"));

    let publisher = v["publisher"]["name"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| str_opt(v, "publisher"));

    let extracted = SchemaExtracted::Article {
        headline: str_opt(v, "headline"),
        author,
        date_published: str_opt(v, "datePublished"),
        date_modified: str_opt(v, "dateModified"),
        publisher,
    };
    (expected, extracted)
}

fn analyze_faq(v: &serde_json::Value) -> (Vec<&'static str>, SchemaExtracted) {
    let expected = vec!["mainEntity"];

    let mut questions = Vec::new();
    if let Some(entities) = v["mainEntity"].as_array() {
        for entity in entities {
            if let Some(q) = entity["name"].as_str() {
                questions.push(q.to_string());
            }
        }
    }

    let extracted = SchemaExtracted::FAQPage {
        question_count: questions.len(),
        questions,
    };
    (expected, extracted)
}

fn analyze_product(
    v: &serde_json::Value,
) -> (Vec<&'static str>, SchemaExtracted) {
    let expected = vec!["name", "offers", "image", "aggregateRating"];

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

    let extracted = SchemaExtracted::Product {
        name: str_opt(v, "name"),
        price,
        currency,
        rating,
        availability,
    };
    (expected, extracted)
}

fn analyze_website(
    v: &serde_json::Value,
) -> (Vec<&'static str>, SchemaExtracted) {
    let expected = vec!["name", "url", "potentialAction"];

    let has_search_action = v["potentialAction"]["@type"]
        .as_str()
        .map(|t| t == "SearchAction")
        .unwrap_or(false);

    let extracted = SchemaExtracted::WebSite {
        name: str_opt(v, "name"),
        url: str_opt(v, "url"),
        has_search_action,
    };
    (expected, extracted)
}

fn analyze_breadcrumb(
    v: &serde_json::Value,
) -> (Vec<&'static str>, SchemaExtracted) {
    let expected = vec!["itemListElement"];

    let item_count = v["itemListElement"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    let extracted = SchemaExtracted::BreadcrumbList { item_count };
    (expected, extracted)
}

fn analyze_generic(
    v: &serde_json::Value,
) -> (Vec<&'static str>, SchemaExtracted) {
    let mut key_fields = Vec::new();
    if let Some(obj) = v.as_object() {
        for (key, val) in obj {
            if key.starts_with('@') {
                continue;
            }
            let display = match val {
                serde_json::Value::String(s) => {
                    if s.len() > 60 {
                        format!("{}...", &s[..57])
                    } else {
                        s.clone()
                    }
                }
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Array(a) => format!("[{} Einträge]", a.len()),
                serde_json::Value::Object(_) => "{...}".to_string(),
                serde_json::Value::Null => continue,
            };
            key_fields.push((key.clone(), display));
            if key_fields.len() >= 8 {
                break;
            }
        }
    }

    (vec![], SchemaExtracted::Generic { key_fields })
}

// ─── Signal Strength Builder ────────────────────────────────────────────────

fn build_signal_strength(seo: &SeoAnalysis) -> SeoSignalStrength {
    let meta = build_meta_signals(seo);
    let headings = build_heading_signals(seo);
    let social = build_social_signals(seo);
    let technical = build_technical_signals(seo);
    let structured = build_structured_data_signals(seo);
    let content = build_content_signals(seo);

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

fn build_meta_signals(seo: &SeoAnalysis) -> SignalCategory {
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

    let checks = vec![
        check(
            "Title vorhanden & Länge optimal",
            title_ok,
            seo.meta.title.as_ref().map(|t| format!("{} Zeichen", t.len())),
        ),
        check(
            "Description vorhanden & Länge optimal",
            desc_ok,
            seo.meta
                .description
                .as_ref()
                .map(|d| format!("{} Zeichen", d.len())),
        ),
        check("Keywords vorhanden", seo.meta.keywords.is_some(), None),
        check("Viewport konfiguriert", seo.meta.viewport.is_some(), None),
        check("Charset definiert", seo.meta.charset.is_some(), None),
        check(
            "Sprache (lang) gesetzt",
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

fn build_heading_signals(seo: &SeoAnalysis) -> SignalCategory {
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
            "Genau ein H1",
            h1_ok,
            Some(format!("{} H1-Tags gefunden", seo.headings.h1_count)),
        ),
        check(
            "H1-Text vorhanden",
            h1_not_empty,
            seo.headings.h1_text.clone(),
        ),
        check("Keine übersprungenen Ebenen", no_skipped, None),
        check(
            "Logische Hierarchie",
            no_issues,
            if no_issues {
                None
            } else {
                Some(format!("{} Probleme", seo.headings.issues.len()))
            },
        ),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: "Überschriften".to_string(),
        score_pct,
        checks,
    }
}

fn build_social_signals(seo: &SeoAnalysis) -> SignalCategory {
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

    let checks = vec![
        check("OpenGraph-Tags vorhanden", og_present, None),
        check(
            "OpenGraph ≥ 80% vollständig",
            og_complete,
            seo.social
                .open_graph
                .as_ref()
                .map(|og| format!("{}%", og.completeness())),
        ),
        check("Twitter Card vorhanden", tw_present, None),
        check(
            "Twitter Card ≥ 75% vollständig",
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

fn build_technical_signals(seo: &SeoAnalysis) -> SignalCategory {
    let robots_ok = seo
        .technical
        .robots_meta
        .as_ref()
        .map(|r| !r.contains("noindex"))
        .unwrap_or(true); // No robots meta = ok (default is index)

    let checks = vec![
        check("HTTPS", seo.technical.https, None),
        check(
            "Canonical URL",
            seo.technical.has_canonical,
            seo.technical.canonical_url.clone(),
        ),
        check(
            "Sprache (lang)",
            seo.technical.has_lang,
            seo.technical.lang.clone(),
        ),
        check(
            "Hreflang (Mehrsprachigkeit)",
            seo.technical.has_hreflang || !seo.technical.has_lang,
            if seo.technical.has_hreflang {
                Some(format!("{} Sprachen", seo.technical.hreflang.len()))
            } else {
                None
            },
        ),
        check("Robots erlaubt Indexierung", robots_ok, seo.technical.robots_meta.clone()),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: "Technisch".to_string(),
        score_pct,
        checks,
    }
}

fn build_structured_data_signals(seo: &SeoAnalysis) -> SignalCategory {
    let has_any = seo.structured_data.has_structured_data;
    let has_rich = !seo.structured_data.rich_snippets_potential.is_empty();
    let has_org = seo.structured_data.types.iter().any(|t| {
        matches!(
            t,
            SchemaType::Organization | SchemaType::WebSite | SchemaType::LocalBusiness
        )
    });

    let checks = vec![
        check(
            "Strukturierte Daten vorhanden",
            has_any,
            if has_any {
                Some(format!(
                    "{} Schema(s)",
                    seo.structured_data.json_ld.len()
                ))
            } else {
                None
            },
        ),
        check(
            "Rich-Snippet-fähig",
            has_rich,
            if has_rich {
                Some(seo.structured_data.rich_snippets_potential.join(", "))
            } else {
                None
            },
        ),
        check("Organization/WebSite Schema", has_org, None),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: "Strukturierte Daten".to_string(),
        score_pct,
        checks,
    }
}

fn build_content_signals(seo: &SeoAnalysis) -> SignalCategory {
    let wc = seo.technical.word_count;
    let il = seo.technical.internal_links;
    let link_density = if wc > 0 {
        il as f32 / wc as f32
    } else {
        0.0
    };

    let checks = vec![
        check(
            "Inhaltsstärke (≥ 300 Wörter)",
            wc >= 300,
            Some(format!("{} Wörter", wc)),
        ),
        check(
            "Interne Verlinkung (≥ 3 Links)",
            il >= 3,
            Some(format!("{} interne Links", il)),
        ),
        check(
            "Linkdichte > 0,5%",
            link_density > 0.005,
            Some(format!("{:.2}%", link_density * 100.0)),
        ),
    ];
    let score_pct = category_score(&checks);
    SignalCategory {
        name: "Inhaltsqualität".to_string(),
        score_pct,
        checks,
    }
}

// ─── Maturity Techniques Counter ────────────────────────────────────────────

fn count_techniques(seo: &SeoAnalysis) -> u32 {
    let mut count = 0u32;

    // 1. Title optimiert (30-60 Zeichen)
    if seo.meta.title.as_ref().map(|t| t.len() >= 30 && t.len() <= 60).unwrap_or(false) {
        count += 1;
    }
    // 2. Description optimiert (120-160 Zeichen)
    if seo.meta.description.as_ref().map(|d| d.len() >= 120 && d.len() <= 160).unwrap_or(false) {
        count += 1;
    }
    // 3. OpenGraph komplett (≥80%)
    if seo.social.open_graph.as_ref().map(|og| og.completeness() >= 80).unwrap_or(false) {
        count += 1;
    }
    // 4. Twitter Card komplett (≥75%)
    if seo.social.twitter_card.as_ref().map(|tc| tc.completeness() >= 75).unwrap_or(false) {
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
    // 12. Rich-Snippet-fähig
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
        }
    }

    #[test]
    fn test_maturity_basic_with_no_data() {
        let seo = minimal_seo();
        let profile = build_content_profile(&seo);
        assert_eq!(profile.maturity, SeoMaturityLevel::Basic);
        assert_eq!(profile.maturity_techniques, 0);
    }

    #[test]
    fn test_content_identity_from_meta() {
        let mut seo = minimal_seo();
        seo.meta.title = Some("Casoon - CSS Framework & Design System".to_string());
        seo.meta.description = Some("Casoon bietet ein modernes CSS Framework.".to_string());
        seo.technical.lang = Some("de".to_string());

        let profile = build_content_profile(&seo);
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
            content: faq_content,
            is_valid: true,
        });
        seo.structured_data.types.push(SchemaType::FAQPage);
        seo.structured_data.has_structured_data = true;

        let profile = build_content_profile(&seo);
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

        let profile = build_content_profile(&seo);

        // All signal categories should be high
        for cat in &profile.signal_strength.categories {
            assert!(
                cat.score_pct >= 75,
                "{} only {}%",
                cat.name,
                cat.score_pct
            );
        }
        assert!(profile.signal_strength.overall_pct >= 80);
        assert!(matches!(
            profile.maturity,
            SeoMaturityLevel::Advanced | SeoMaturityLevel::Professional
        ));
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
}
