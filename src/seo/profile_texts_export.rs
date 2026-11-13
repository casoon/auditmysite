//! Text export of the SEO content profile (#936): every text that
//! `profile::build_content_profile` writes (content type, summary, page
//! attributes, the counts of generic schema fields and the names, labels and
//! details of the signal categories) in both languages across an enumerated
//! set of pages, as records in `reports/seo-profile-texts.json` (gitignored;
//! the path can be set with `SEO_PROFILE_TEXT_EXPORT`). Run it before and
//! after a change of how the texts are stored and compare the two files byte
//! for byte.

use super::headings::{HeadingInfo, HeadingIssue};
use super::profile::{build_content_profile, SchemaExtracted};
use super::schema::{JsonLdSchema, SchemaIssue, SchemaIssueSeverity, SchemaType};
use super::schema_rules::{
    SchemaFeature, SchemaFeatureAvailability, SchemaRequirementStatus, SchemaRuleAssessment,
};
use super::social::{OpenGraph, TwitterCard};
use super::technical::HreflangTag;
use super::SeoAnalysis;
use crate::taxonomy::Severity;
use serde_json::{json, Value};

type Apply = Box<dyn Fn(&mut SeoAnalysis)>;

fn heading(level: u8) -> HeadingInfo {
    HeadingInfo {
        level,
        text: format!("H{level}"),
        length: 2,
        is_question: false,
        in_faq_context: false,
        word_count_after: 0,
        hidden: false,
    }
}

fn heading_issue(issue_type: &str) -> HeadingIssue {
    HeadingIssue {
        issue_type: issue_type.to_string(),
        message: String::new(),
        severity: Severity::Medium,
        long_heading: None,
    }
}

fn json_ld(schema_type: &str, content: Value, is_valid: bool) -> JsonLdSchema {
    JsonLdSchema {
        schema_type: schema_type.to_string(),
        schema_types: if is_valid {
            vec![schema_type.to_string()]
        } else {
            vec![]
        },
        content,
        is_valid,
    }
}

fn full_og(og_type: &str) -> OpenGraph {
    OpenGraph {
        title: Some("T".into()),
        description: Some("D".into()),
        image: Some("I".into()),
        url: Some("U".into()),
        og_type: Some(og_type.into()),
        site_name: Some("S".into()),
        locale: Some("L".into()),
    }
}

fn assessment(
    feature: SchemaFeature,
    status: SchemaRequirementStatus,
    required: &[&str],
    recommended: &[&str],
) -> SchemaRuleAssessment {
    SchemaRuleAssessment {
        node_index: 0,
        schema_type: "Product".to_string(),
        feature,
        availability: SchemaFeatureAvailability::General,
        requirement_status: status,
        missing_required: required.iter().map(|s| s.to_string()).collect(),
        missing_recommended: recommended.iter().map(|s| s.to_string()).collect(),
        manual_review: vec![],
        quality_score: None,
        source_url: String::new(),
        reviewed_at: String::new(),
    }
}

fn push(out: &mut Vec<(String, SeoAnalysis)>, label: &str, apply: Apply) {
    let mut seo = SeoAnalysis::default();
    apply(&mut seo);
    out.push((label.to_string(), seo));
}

/// Pages that between them reach every text path of the content profile.
pub(super) fn profile_scenarios() -> Vec<(String, SeoAnalysis)> {
    let mut out = Vec::new();
    push(&mut out, "empty", Box::new(|_| {}));

    // Page classification and its attributes.
    for words in [0_u32, 120, 450, 650, 1_800] {
        for h2 in [0_usize, 2, 3, 5] {
            push(
                &mut out,
                &format!("words {words}, {h2} h2"),
                Box::new(move |s| {
                    s.technical.word_count = words;
                    s.headings.headings = std::iter::once(heading(1))
                        .chain((0..h2).map(|_| heading(2)))
                        .collect();
                    s.headings.total_count = s.headings.headings.len();
                }),
            );
        }
    }
    for links in [2_u32, 3, 30, 45] {
        push(
            &mut out,
            &format!("{links} internal links"),
            Box::new(move |s| {
                s.technical.word_count = 400;
                s.technical.internal_links = links;
            }),
        );
    }
    for title in ["Kontakt", "Login", "Home"] {
        push(
            &mut out,
            &format!("title {title}"),
            Box::new(move |s| {
                s.technical.word_count = 400;
                s.meta.title = Some(title.to_string());
            }),
        );
    }

    // Content type: OpenGraph type first, then the schema types.
    for og_type in ["article", "product", "profile", "website", "video.other"] {
        push(
            &mut out,
            &format!("og:type {og_type}"),
            Box::new(move |s| s.social.open_graph = Some(full_og(og_type))),
        );
    }
    let types = [
        SchemaType::Article,
        SchemaType::BlogPosting,
        SchemaType::NewsArticle,
        SchemaType::Product,
        SchemaType::LocalBusiness,
        SchemaType::Event,
        SchemaType::Recipe,
        SchemaType::FAQPage,
        SchemaType::Organization,
        SchemaType::WebSite,
    ];
    for schema_type in types {
        push(
            &mut out,
            &format!("schema type {schema_type}"),
            Box::new(move |s| {
                s.technical.word_count = 400;
                s.structured_data.types.push(schema_type.clone());
            }),
        );
    }

    // Summary.
    let long = "Ein sehr langer Titel mit Umlauten äöü und noch mehr Text dahinter";
    let long_desc = "Eine Beschreibung, die zusammen mit dem Titel deutlich über \
                     hundertfünfzig Zeichen hinausgeht, damit gekürzt wird — mit ß.";
    for (label, title, desc) in [
        ("title only", Some("Titel"), None),
        ("description only", None, Some("Beschreibung")),
        ("title and description", Some("Titel"), Some("Beschreibung")),
        ("long title and description", Some(long), Some(long_desc)),
        ("empty title and description", Some(""), Some("")),
    ] {
        push(
            &mut out,
            label,
            Box::new(move |s| {
                s.meta.title = title.map(String::from);
                s.meta.description = desc.map(String::from);
            }),
        );
    }

    // Generic schema fields (an array is counted).
    push(
        &mut out,
        "generic schema",
        Box::new(|s| {
            s.structured_data.json_ld.push(json_ld(
                "Thing",
                json!({
                    "@type": "Thing",
                    "a_list": [1, 2, 3],
                    "b_one": ["x"],
                    "c_empty": [],
                    "d_number": 5,
                    "e_bool": true,
                    "f_object": {},
                    "g_null": null,
                    "h_long": "x".repeat(80),
                    "i_name": "Name",
                }),
                true,
            ));
        }),
    );

    // Meta signals.
    let good_title = "Ein optimaler Titel der genau passt!!!!";
    let good_desc = "Eine perfekte Beschreibung die zwischen 120 und 160 Zeichen liegt \
                     und alles sagt was man wissen muss über den Inhalt.";
    for (label, title, desc) in [
        ("good title and description", good_title, good_desc),
        ("short title and description", "Kurz", "Kurz"),
    ] {
        push(
            &mut out,
            label,
            Box::new(move |s| {
                s.meta.title = Some(title.to_string());
                s.meta.description = Some(desc.to_string());
                s.meta.viewport = Some("width=device-width".into());
                s.meta.charset = Some("UTF-8".into());
                s.meta.lang = Some("de".into());
            }),
        );
    }

    // Heading signals.
    for h1_count in [0_usize, 1, 2] {
        for h1_text in [None, Some(""), Some("Hauptüberschrift")] {
            push(
                &mut out,
                &format!("{h1_count} h1, text {h1_text:?}"),
                Box::new(move |s| {
                    s.headings.h1_count = h1_count;
                    s.headings.h1_text = h1_text.map(String::from);
                }),
            );
        }
    }
    let orders: [(&str, Vec<u8>); 4] = [
        ("h1 first", vec![1, 2]),
        ("h2 first", vec![2, 1, 2]),
        ("h3 first", vec![3, 1]),
        ("no headings", vec![]),
    ];
    for (order, levels) in orders {
        for issues in [vec![], vec!["skipped_level"], vec!["multiple_h1", "empty"]] {
            let levels = levels.clone();
            push(
                &mut out,
                &format!("{order}, issues {issues:?}"),
                Box::new(move |s| {
                    s.headings.h1_count = 1;
                    s.headings.h1_text = Some("H1".into());
                    s.headings.headings = levels.iter().map(|l| heading(*l)).collect();
                    s.headings.issues = issues.iter().map(|i| heading_issue(i)).collect();
                }),
            );
        }
    }

    // Social signals.
    push(
        &mut out,
        "complete social tags",
        Box::new(|s| {
            s.social.open_graph = Some(full_og("website"));
            s.social.twitter_card = Some(TwitterCard {
                card: Some("summary".into()),
                title: Some("T".into()),
                description: Some("D".into()),
                image: Some("I".into()),
                site: Some("S".into()),
                creator: Some("C".into()),
            });
        }),
    );
    push(
        &mut out,
        "partial social tags",
        Box::new(|s| {
            s.social.open_graph = Some(OpenGraph {
                title: Some("T".into()),
                ..Default::default()
            });
            s.social.twitter_card = Some(TwitterCard {
                card: Some("summary".into()),
                ..Default::default()
            });
        }),
    );

    // Technical signals.
    for hreflang in [0_usize, 1, 3] {
        for robots in [None, Some("noindex, follow"), Some("index, follow")] {
            push(
                &mut out,
                &format!("{hreflang} hreflang, robots {robots:?}"),
                Box::new(move |s| {
                    s.technical.https = hreflang > 0;
                    s.technical.has_canonical = hreflang > 0;
                    s.technical.canonical_url =
                        (hreflang > 0).then(|| "https://example.com/".to_string());
                    s.technical.has_lang = hreflang > 1;
                    s.technical.lang = (hreflang > 1).then(|| "de".to_string());
                    s.technical.has_hreflang = hreflang > 0;
                    s.technical.hreflang = (0..hreflang)
                        .map(|i| HreflangTag {
                            lang: format!("l{i}"),
                            url: format!("https://example.com/{i}"),
                        })
                        .collect();
                    s.technical.robots_meta = robots.map(String::from);
                }),
            );
        }
    }

    // Structured-data signals.
    for valid in [0_usize, 1, 2] {
        for invalid in [false, true] {
            for rich in [vec![], vec!["FAQ"], vec!["FAQ", "Product"]] {
                push(
                    &mut out,
                    &format!("{valid} valid json-ld, invalid {invalid}, rich {rich:?}"),
                    Box::new(move |s| {
                        s.structured_data.has_structured_data = true;
                        for i in 0..valid {
                            s.structured_data.json_ld.push(json_ld(
                                "WebPage",
                                json!({ "@type": "WebPage", "name": format!("P{i}") }),
                                true,
                            ));
                        }
                        if invalid {
                            s.structured_data
                                .json_ld
                                .push(json_ld("Unknown", json!({}), false));
                        }
                        s.structured_data.rich_snippets_potential =
                            rich.iter().map(|r| r.to_string()).collect();
                        s.structured_data.types.push(SchemaType::Organization);
                    }),
                );
            }
        }
    }
    let local_businesses: [(&str, Vec<Value>); 6] = [
        (
            "bare local business",
            vec![json!({ "@type": "LocalBusiness", "name": "A" })],
        ),
        (
            "complete local business",
            vec![json!({
                "@type": "LocalBusiness",
                "name": "A",
                "telephone": "+49",
                "address": {
                    "streetAddress": "Straße 1",
                    "postalCode": "18055",
                    "addressLocality": "Rostock"
                },
                "geo": { "latitude": 54.0 },
                "sameAs": ["https://example.com"],
                "aggregateRating": { "ratingValue": 4.5 }
            })],
        ),
        (
            "local business without locality",
            vec![json!({
                "@type": "LocalBusiness",
                "name": "A",
                "address": { "streetAddress": "Straße 1", "postalCode": "18055" },
                "geo": {}
            })],
        ),
        (
            "two consistent local businesses",
            vec![
                json!({ "@type": "LocalBusiness", "name": "A" }),
                json!({ "@type": "LocalBusiness", "name": "A" }),
            ],
        ),
        (
            "three conflicting local businesses",
            vec![
                json!({ "@type": "LocalBusiness", "name": "A" }),
                json!({ "@type": "LocalBusiness", "name": "B" }),
                json!({ "@type": "LocalBusiness", "name": "A" }),
            ],
        ),
        (
            "auto business",
            vec![json!({ "@type": "AutoBusiness", "name": "A", "telephone": "+49" })],
        ),
    ];
    for (label, schemas) in local_businesses {
        push(
            &mut out,
            label,
            Box::new(move |s| {
                s.structured_data.has_structured_data = true;
                s.structured_data.types.push(SchemaType::LocalBusiness);
                for content in &schemas {
                    let schema_type = content["@type"].as_str().unwrap().to_string();
                    s.structured_data
                        .json_ld
                        .push(json_ld(&schema_type, content.clone(), true));
                }
            }),
        );
    }
    push(
        &mut out,
        "structural json-ld issues",
        Box::new(|s| {
            s.structured_data.has_structured_data = true;
            for issue_type in [
                "jsonld_parse_error",
                "jsonld_missing_context",
                "schema_missing_name",
            ] {
                s.structured_data.schema_issues.push(SchemaIssue {
                    schema_type: "Article".to_string(),
                    severity: SchemaIssueSeverity::Required,
                    issue_type: issue_type.to_string(),
                    message: "fallback".to_string(),
                });
            }
        }),
    );
    let statuses = [
        SchemaRequirementStatus::MeetsRequiredProperties,
        SchemaRequirementStatus::MissingRequiredProperties,
        SchemaRequirementStatus::RecommendationsOnly,
        SchemaRequirementStatus::NotEvaluated,
    ];
    for feature in [SchemaFeature::ProductSnippet, SchemaFeature::Breadcrumb] {
        for status in statuses {
            for (required, recommended) in [
                (vec![], vec![]),
                (vec!["name"], vec![]),
                (vec![], vec!["image", "brand"]),
                (vec!["offers", "name"], vec!["image"]),
            ] {
                push(
                    &mut out,
                    &format!("{feature:?} {status:?} {required:?} {recommended:?}"),
                    Box::new(move |s| {
                        s.structured_data.rule_assessments.push(assessment(
                            feature,
                            status,
                            &required,
                            &recommended,
                        ));
                    }),
                );
            }
        }
    }

    // Content signals.
    for (words, links) in [(0_u32, 0_u32), (250, 2), (1_000, 7), (3_000, 3), (100, 40)] {
        push(
            &mut out,
            &format!("{words} words, {links} internal links"),
            Box::new(move |s| {
                s.technical.word_count = words;
                s.technical.internal_links = links;
            }),
        );
    }
    out
}

/// The texts of one profile, in the order the profile holds them.
pub(super) fn profile_texts(seo: &SeoAnalysis, lang: &str) -> Value {
    let profile = build_content_profile(seo, lang);
    let generic: Vec<_> = profile
        .schema_inventory
        .schemas
        .iter()
        .filter_map(|schema| match &schema.extracted {
            SchemaExtracted::Generic { key_fields } => Some(key_fields.clone()),
            _ => None,
        })
        .collect();
    let categories: Vec<_> = profile
        .signal_strength
        .categories
        .iter()
        .map(|category| {
            let checks: Vec<_> = category
                .checks
                .iter()
                .map(|check| json!([check.label, check.passed, check.detail]))
                .collect();
            json!({ "name": category.name, "checks": checks })
        })
        .collect();
    json!({
        "content_type": profile.content_identity.content_type,
        "summary": profile.content_identity.summary,
        "attributes": profile.page_classification.attributes,
        "generic": generic,
        "categories": categories,
    })
}

#[test]
#[ignore = "writes reports/seo-profile-texts.json; run before and after a change of the text storage"]
fn export_seo_profile_texts() {
    let mut records = Vec::new();
    for (label, seo) in profile_scenarios() {
        for lang in ["en", "de"] {
            records.push(json!({
                "scenario": label,
                "lang": lang,
                "profile": profile_texts(&seo, lang),
            }));
        }
    }
    let path = std::env::var("SEO_PROFILE_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/seo-profile-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} profile records written to {path}", records.len());
}
