//! Text export of the SERP pass (#936): every text that
//! `serp::build_serp_analysis` writes (category, label, status label and
//! detail of each signal) in both languages across an enumerated set of
//! pages, as records in `reports/seo-serp-texts.json` (gitignored; the path
//! can be set with `SEO_SERP_TEXT_EXPORT`). Run it before and after a change
//! of how the texts are stored and compare the two files byte for byte.

use super::schema::SchemaType;
use super::schema_rules::{
    SchemaFeature, SchemaFeatureAvailability, SchemaRequirementStatus, SchemaRuleAssessment,
};
use super::serp::build_serp_analysis;
use super::technical::HreflangTag;
use super::SeoAnalysis;
use serde_json::{json, Value};

type Apply = Box<dyn Fn(&mut SeoAnalysis)>;

/// The requirement status and missing properties of a breadcrumb scenario.
type BreadcrumbAssessment = Option<(SchemaRequirementStatus, Vec<&'static str>)>;

/// The page URL every scenario is audited under.
const SERP_URL: &str = "https://example.com/page/";

fn breadcrumb(status: SchemaRequirementStatus, missing: &[&str]) -> SchemaRuleAssessment {
    SchemaRuleAssessment {
        node_index: 0,
        schema_type: "BreadcrumbList".to_string(),
        feature: SchemaFeature::Breadcrumb,
        availability: SchemaFeatureAvailability::General,
        requirement_status: status,
        missing_required: missing.iter().map(|s| s.to_string()).collect(),
        missing_recommended: vec![],
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

/// Pages that between them reach every text path of the SERP pass.
pub(crate) fn serp_scenarios() -> Vec<(String, SeoAnalysis)> {
    let mut out = Vec::new();
    push(&mut out, "empty", Box::new(|_| {}));

    // Robots meta: noindex, nosnippet, both, neither.
    for robots in [
        "noindex",
        "NOINDEX, follow",
        "nosnippet",
        "noindex, nosnippet",
        "index, follow",
        "noindex {$var}",
    ] {
        push(
            &mut out,
            &format!("robots {robots}"),
            Box::new(move |s| s.technical.robots_meta = Some(robots.to_string())),
        );
    }

    // Title: missing, empty, short, good, at and past the truncation limit,
    // with umlauts.
    let titles: [(&str, Option<String>); 7] = [
        ("no title", None),
        ("empty title", Some(String::new())),
        ("short title", Some("Kurz".to_string())),
        (
            "good title",
            Some("Ein optimaler Titel der genau passt!!!!".to_string()),
        ),
        ("55 character title", Some("x".repeat(55))),
        ("56 character title", Some("x".repeat(56))),
        (
            "long title with umlauts",
            Some(format!("Überschrift äöü ß {}", "y".repeat(70))),
        ),
    ];
    for (label, title) in titles {
        push(
            &mut out,
            label,
            Box::new(move |s| s.meta.title = title.clone()),
        );
    }

    // Meta description: missing, empty, short, good, long.
    let descriptions: [(&str, Option<String>); 5] = [
        ("no description", None),
        ("empty description", Some(String::new())),
        ("short description", Some("Kurz".to_string())),
        ("good description", Some("d".repeat(140))),
        ("long description", Some("d".repeat(200))),
    ];
    for (label, description) in descriptions {
        push(
            &mut out,
            label,
            Box::new(move |s| s.meta.description = description.clone()),
        );
    }

    // Canonical: tag with and without a URL, self-reference (with and
    // without the trailing slash, other case), a different URL, a URL
    // without a tag flag, an empty URL.
    let canonicals: [(&str, bool, Option<&str>); 8] = [
        ("canonical flag without url", true, None),
        ("canonical self", true, Some("https://example.com/page/")),
        (
            "canonical self without slash",
            true,
            Some("https://example.com/page"),
        ),
        (
            "canonical self other case",
            true,
            Some("HTTPS://EXAMPLE.COM/page"),
        ),
        (
            "canonical other url",
            true,
            Some("https://example.com/other"),
        ),
        (
            "canonical other url {$var}",
            true,
            Some("https://example.com/{$var}"),
        ),
        (
            "canonical url without flag",
            false,
            Some("https://example.com/other"),
        ),
        ("canonical empty url", true, Some("/")),
    ];
    for (label, has_canonical, canonical_url) in canonicals {
        push(
            &mut out,
            label,
            Box::new(move |s| {
                s.technical.has_canonical = has_canonical;
                s.technical.canonical_url = canonical_url.map(String::from);
            }),
        );
    }

    // Favicon.
    push(
        &mut out,
        "favicon",
        Box::new(|s| s.technical.has_favicon = true),
    );

    // Hreflang with and without x-default, one and several tags.
    for (count, x_default) in [(1_usize, false), (3, false), (1, true), (4, true)] {
        push(
            &mut out,
            &format!("{count} hreflang, x-default {x_default}"),
            Box::new(move |s| {
                s.technical.has_hreflang = true;
                s.technical.hreflang = (0..count)
                    .map(|i| HreflangTag {
                        lang: if x_default && i == 0 {
                            "x-default".to_string()
                        } else {
                            format!("l{i}")
                        },
                        url: format!("https://example.com/{i}"),
                    })
                    .collect();
            }),
        );
    }

    // Breadcrumb schema: absent, complete, incomplete (one and two missing
    // properties), an incomplete assessment without the type.
    let breadcrumbs: [(&str, bool, BreadcrumbAssessment); 6] = [
        ("breadcrumb type only", true, None),
        (
            "breadcrumb complete",
            true,
            Some((SchemaRequirementStatus::MeetsRequiredProperties, vec![])),
        ),
        (
            "breadcrumb recommendations only",
            true,
            Some((SchemaRequirementStatus::RecommendationsOnly, vec![])),
        ),
        (
            "breadcrumb missing one",
            true,
            Some((
                SchemaRequirementStatus::MissingRequiredProperties,
                vec!["itemListElement (minimum 2 entries)"],
            )),
        ),
        (
            "breadcrumb missing two",
            true,
            Some((
                SchemaRequirementStatus::MissingRequiredProperties,
                vec!["itemListElement", "name {$var}"],
            )),
        ),
        (
            "incomplete breadcrumb without type",
            false,
            Some((
                SchemaRequirementStatus::MissingRequiredProperties,
                vec!["itemListElement"],
            )),
        ),
    ];
    for (label, has_type, assessment) in breadcrumbs {
        push(
            &mut out,
            label,
            Box::new(move |s| {
                if has_type {
                    s.structured_data.types.push(SchemaType::BreadcrumbList);
                }
                if let Some((status, missing)) = &assessment {
                    s.structured_data
                        .rule_assessments
                        .push(breadcrumb(*status, missing));
                }
            }),
        );
    }

    // A complete page: every signal at once, and rich-result types.
    push(
        &mut out,
        "complete page",
        Box::new(|s| {
            s.technical.robots_meta = Some("noindex, nosnippet".to_string());
            s.meta.title = Some("t".repeat(58));
            s.meta.description = Some("d".repeat(130));
            s.technical.has_canonical = true;
            s.technical.canonical_url = Some("https://example.com/other".to_string());
            s.technical.has_favicon = true;
            s.technical.has_hreflang = true;
            s.technical.hreflang = vec![HreflangTag {
                lang: "x-default".to_string(),
                url: "https://example.com/".to_string(),
            }];
            s.structured_data.types.push(SchemaType::BreadcrumbList);
            s.structured_data.rich_snippets_potential = vec!["FAQ".to_string()];
        }),
    );
    out
}

/// The texts and counts of one SERP pass, in the order the pass holds them.
pub(super) fn serp_texts(seo: &SeoAnalysis, lang: &str) -> Value {
    let analysis = build_serp_analysis(seo, SERP_URL, lang);
    let en = lang == "en";
    let signals: Vec<_> = analysis
        .signals
        .iter()
        .map(|signal| {
            json!([
                signal.category,
                signal.label,
                signal.status.label(en),
                signal.detail
            ])
        })
        .collect();
    json!({
        "signals": signals,
        "score": analysis.score,
        "counts": [analysis.pass_count, analysis.warning_count, analysis.fail_count],
        "rich_result_types": analysis.rich_result_types,
    })
}

#[test]
#[ignore = "writes reports/seo-serp-texts.json; run before and after a change of the text storage"]
fn export_seo_serp_texts() {
    let mut records = Vec::new();
    for (label, seo) in serp_scenarios() {
        for lang in ["en", "de"] {
            records.push(json!({
                "scenario": label,
                "lang": lang,
                "serp": serp_texts(&seo, lang),
            }));
        }
    }
    let path = std::env::var("SEO_SERP_TEXT_EXPORT")
        .unwrap_or_else(|_| format!("{}/reports/seo-serp-texts.json", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!("{} serp records written to {path}", records.len());
}
