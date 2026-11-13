//! Text export of the SEO module detail presentation, slice 11 of #936:
//! every text `build_seo_details` puts into the `SeoPresentation` (meta tags
//! and issues, heading, social, technical and tracking summaries, the content
//! profile with its schema, signal and maturity rows, robots.txt and the
//! technical issue rows) across an enumerated set of pages in both languages,
//! as records in `reports/seo-details-texts.json` (gitignored; the path can be
//! set with `SEO_DETAILS_TEXT_EXPORT`). Each page goes through
//! `normalize_checked` like a real audit. Run it before and after a change of
//! how the texts are stored and compare the two files byte for byte.

use super::build_seo_details;
use crate::audit::{AuditReport, ConsentCookieSignal, ConsentPrivacySnapshot};
use crate::i18n::I18n;
use crate::model::WcagLevel;
use crate::output::report_model::SeoPresentation;
use crate::seo::headings::HeadingIssue;
use crate::seo::meta::{MetaIssueKind, MetaIssueValues, MetaValidation};
use crate::seo::schema::{JsonLdSchema, SchemaIssue, SchemaIssueSeverity};
use crate::seo::schema_rules::{
    ManualReview, SchemaFeature, SchemaFeatureAvailability, SchemaRequirementStatus,
    SchemaRuleAssessment,
};
use crate::seo::social::{OpenGraph, TwitterCard};
use crate::seo::technical::{CookieInventoryItem, StorageItem, TrackingCookie};
use crate::seo::SeoAnalysis;
use crate::taxonomy::Severity;
use crate::wcag::WcagResults;
use serde_json::{json, Value};

const LANGS: [&str; 2] = ["en", "de"];

/// Counts: one, two and one that Fluent could group.
const COUNTS: [usize; 3] = [1, 2, 1_234_567];

type Apply = Box<dyn Fn(&mut SeoAnalysis, &mut Option<ConsentPrivacySnapshot>)>;

/// One page of the export: a label, the SEO analysis and the consent snapshot.
pub(super) struct Scenario {
    pub(super) label: String,
    pub(super) seo: SeoAnalysis,
    pub(super) consent: Option<ConsentPrivacySnapshot>,
}

fn push(out: &mut Vec<Scenario>, label: String, apply: Apply) {
    let mut seo = SeoAnalysis::default();
    let mut consent = None;
    apply(&mut seo, &mut consent);
    out.push(Scenario {
        label,
        seo,
        consent,
    });
}

fn json_ld(schema_type: &str, content: Value) -> JsonLdSchema {
    JsonLdSchema {
        schema_type: schema_type.to_string(),
        schema_types: vec![schema_type.to_string()],
        content,
        is_valid: true,
    }
}

fn cookie(
    name: &str,
    secure: bool,
    same_site: bool,
    category: Option<&str>,
) -> CookieInventoryItem {
    CookieInventoryItem {
        name: name.to_string(),
        domain: "example.com".to_string(),
        path: "/".to_string(),
        secure,
        http_only: false,
        same_site: same_site.then(|| "Lax".to_string()),
        session: false,
        third_party: false,
        provider: None,
        category: category.map(String::from),
    }
}

fn storage(key: &str, category: Option<&str>) -> StorageItem {
    StorageItem {
        area: "localStorage".to_string(),
        key: key.to_string(),
        provider: None,
        category: category.map(String::from),
    }
}

fn consent_cookie(name: &str, category: Option<&str>) -> ConsentCookieSignal {
    ConsentCookieSignal {
        name: name.to_string(),
        domain: "example.com".to_string(),
        secure: true,
        http_only: false,
        same_site: None,
        provider: None,
        category: category.map(String::from),
    }
}

fn assessment(
    availability: SchemaFeatureAvailability,
    status: SchemaRequirementStatus,
    required: &[&str],
    recommended: &[&str],
    manual_review: Vec<ManualReview>,
) -> SchemaRuleAssessment {
    SchemaRuleAssessment {
        node_index: 0,
        schema_type: "Product".to_string(),
        feature: SchemaFeature::Breadcrumb,
        availability,
        requirement_status: status,
        missing_required: required.iter().map(|s| s.to_string()).collect(),
        missing_recommended: recommended.iter().map(|s| s.to_string()).collect(),
        manual_review,
        quality_score: None,
        source_url: String::new(),
        reviewed_at: String::new(),
    }
}

/// Pages that between them reach every text path of `build_seo_details`.
pub(super) fn scenarios() -> Vec<Scenario> {
    let mut out = Vec::new();
    push(&mut out, "empty".to_string(), Box::new(|_, _| {}));

    // Meta tags.
    for (label, title, description, viewport) in [
        ("title", true, false, false),
        ("description", false, true, false),
        ("viewport", false, false, true),
        ("all meta tags", true, true, true),
    ] {
        push(
            &mut out,
            format!("meta: {label}"),
            Box::new(move |s, _| {
                s.meta.title = title.then(|| "Titel äöü".to_string());
                s.meta.description = description.then(|| "Beschreibung {$var}".to_string());
                s.meta.viewport = viewport.then(|| "width=device-width".to_string());
            }),
        );
    }

    // Meta issues: with and without a cross-module note.
    let meta_issues: [(&str, MetaIssueKind, Severity); 7] = [
        ("lang", MetaIssueKind::MissingLang, Severity::Medium),
        ("lang", MetaIssueKind::MissingLang, Severity::High),
        ("title", MetaIssueKind::MissingTitle, Severity::High),
        ("title", MetaIssueKind::TitleTooShort, Severity::Medium),
        ("viewport", MetaIssueKind::MissingViewport, Severity::High),
        (
            "description",
            MetaIssueKind::MissingDescription,
            Severity::Medium,
        ),
        ("canonical", MetaIssueKind::MissingCanonical, Severity::Low),
    ];
    for (field, kind, severity) in meta_issues {
        push(
            &mut out,
            format!("meta issue: {field} {kind:?} {severity:?}"),
            Box::new(move |s, _| {
                s.meta_issues.push(MetaValidation::new(
                    field,
                    kind,
                    MetaIssueValues {
                        length: Some(12),
                        ..MetaIssueValues::default()
                    },
                    severity,
                    "suggestion",
                ));
            }),
        );
    }
    for field in ["lang", "title", "viewport", "other"] {
        push(
            &mut out,
            format!("meta issue without kind: {field}"),
            Box::new(move |s, _| {
                s.meta_issues.push(MetaValidation {
                    field: field.to_string(),
                    message: "Stored message {$var} äöü".to_string(),
                    severity: Severity::High,
                    suggestion: None,
                    kind: None,
                    values: MetaIssueValues::default(),
                });
            }),
        );
    }

    // Heading summary.
    for h1 in [0, 1, 2, 1_234_567] {
        for total in COUNTS {
            for issues in [0, 1, 2] {
                push(
                    &mut out,
                    format!("headings: h1={h1} total={total} issues={issues}"),
                    Box::new(move |s, _| {
                        s.headings.h1_count = h1;
                        s.headings.total_count = total;
                        s.headings.issues = (0..issues)
                            .map(|_| HeadingIssue {
                                issue_type: "skipped_level".to_string(),
                                message: String::new(),
                                severity: Severity::Medium,
                                long_heading: None,
                            })
                            .collect();
                    }),
                );
            }
        }
    }

    // Social summary.
    for og in [false, true] {
        for twitter in [false, true] {
            for completeness in [0, 45, 100] {
                push(
                    &mut out,
                    format!("social: og={og} twitter={twitter} completeness={completeness}"),
                    Box::new(move |s, _| {
                        s.social.open_graph = og.then(OpenGraph::default);
                        s.social.twitter_card = twitter.then(TwitterCard::default);
                        s.social.completeness = completeness;
                    }),
                );
            }
        }
    }

    // Technical summary: the AMP row and the counts.
    for (label, detected, issues) in [
        ("amp off", false, 0),
        ("amp clean", true, 0),
        ("amp 1 issue", true, 1),
        ("amp 2 issues", true, 2),
        ("amp many issues", true, 12),
    ] {
        push(
            &mut out,
            format!("technical: {label}"),
            Box::new(move |s, _| {
                s.technical.amp.detected = detected;
                s.technical.amp.issues = (0..issues).map(|i| format!("issue {i}")).collect();
                s.technical.https = detected;
                s.technical.has_lang = !detected;
                s.technical.has_canonical = detected;
                s.technical.word_count = 1_234_567;
                s.technical.internal_links = 12;
                s.technical.external_links = 3;
                s.technical.dofollow_links = 10;
                s.technical.nofollow_links = 1;
            }),
        );
    }

    // Tracking summary.
    push(
        &mut out,
        "tracking: google fonts".to_string(),
        Box::new(|s, _| {
            s.technical.uses_remote_google_fonts = true;
            s.technical.google_fonts_sources = vec![
                "https://fonts.googleapis.com/css2?family=Inter".to_string(),
                "https://fonts.gstatic.com/s/inter/v12/very-long-font-file-name.woff2".to_string(),
                "https://fonts.googleapis.com/css?family=Roboto".to_string(),
            ];
        }),
    );
    push(
        &mut out,
        "tracking: cookies, signals, zaraz".to_string(),
        Box::new(|s, _| {
            s.technical.tracking_cookies = ["_ga", "_gid"]
                .iter()
                .map(|name| TrackingCookie {
                    name: name.to_string(),
                    scope: "first-party".to_string(),
                    provider: "Google".to_string(),
                })
                .collect();
            s.technical.tracking_signals = ["gtag", "fbq", "hotjar", "matomo"]
                .iter()
                .map(|t| t.to_string())
                .collect();
            s.technical.zaraz.detected = true;
            s.technical.zaraz.signals = vec!["zaraz.track".into(), "zaraz.set".into(), "x".into()];
        }),
    );
    push(
        &mut out,
        "tracking: zaraz without signals".to_string(),
        Box::new(|s, _| s.technical.zaraz.detected = true),
    );
    push(
        &mut out,
        "tracking: empty cookie inventory".to_string(),
        Box::new(|s, _| s.technical.cookie_inventory = Some(vec![])),
    );
    for (label, cookies) in [
        ("one safe cookie", vec![cookie("a", true, true, None)]),
        (
            "risky cookies",
            vec![
                cookie("a", false, true, None),
                cookie("b", true, false, None),
            ],
        ),
        (
            "categorized cookies",
            vec![
                cookie("a", true, true, Some("analytics")),
                cookie("b", false, false, Some("analytics")),
                cookie("c", true, true, Some("marketing")),
                cookie("d", true, true, None),
            ],
        ),
    ] {
        push(
            &mut out,
            format!("tracking: cookie inventory, {label}"),
            Box::new(move |s, _| s.technical.cookie_inventory = Some(cookies.clone())),
        );
    }
    for (label, items) in [
        ("storage without categories", vec![storage("theme", None)]),
        (
            "storage with categories",
            vec![
                storage("_ga_state", Some("analytics")),
                storage("ab_test", Some("analytics")),
                storage("consent", Some("functional")),
            ],
        ),
    ] {
        push(
            &mut out,
            format!("tracking: {label}"),
            Box::new(move |s, _| s.technical.storage_items = items.clone()),
        );
    }
    for count in COUNTS {
        for categories in [false, true] {
            push(
                &mut out,
                format!("tracking: consent snapshot, {count} new, categories={categories}"),
                Box::new(move |_, consent| {
                    let shown = count.min(3);
                    let category = categories.then_some("marketing");
                    *consent = Some(ConsentPrivacySnapshot {
                        before_interaction: vec![consent_cookie("before", None)],
                        after_interaction: (0..shown + 1)
                            .map(|i| consent_cookie(&format!("after{i}"), category))
                            .collect(),
                        added_after_interaction: (0..shown)
                            .map(|i| consent_cookie(&format!("new{i}"), category))
                            .collect(),
                    });
                }),
            );
        }
    }

    // Schema inventory rows, one per extracted variant and branch.
    let schemas: [(&str, &str, Value); 16] = [
        ("organization", "Organization", json!({"name": "Org"})),
        (
            "local business",
            "LocalBusiness",
            json!({"name": "Shop", "address": {"streetAddress": "Hauptstr. 1"}}),
        ),
        (
            "article",
            "Article",
            json!({"headline": "Headline", "author": {"name": "Ann"}}),
        ),
        (
            "faq one",
            "FAQPage",
            json!({"mainEntity": [{"@type": "Question", "name": "Q1"}]}),
        ),
        (
            "faq two",
            "FAQPage",
            json!({"mainEntity": [{"name": "Q1"}, {"name": "Q2"}]}),
        ),
        (
            "product",
            "Product",
            json!({"name": "Thing", "offers": {"price": "9.99", "priceCurrency": "EUR"}}),
        ),
        ("website", "WebSite", json!({"name": "Site"})),
        (
            "website with search",
            "WebSite",
            json!({"name": "Site", "potentialAction": {"@type": "SearchAction"}}),
        ),
        (
            "webpage",
            "WebPage",
            json!({"name": "Page", "author": "Bob", "inLanguage": "de"}),
        ),
        ("service", "Service", json!({"name": "Service"})),
        (
            "service one region",
            "Service",
            json!({"name": "Service", "address": "Street", "areaServed": ["A"]}),
        ),
        (
            "service two regions",
            "ProfessionalService",
            json!({"name": "Service", "areaServed": ["A", "B"]}),
        ),
        ("breadcrumb empty", "BreadcrumbList", json!({})),
        (
            "breadcrumb one",
            "BreadcrumbList",
            json!({"itemListElement": [{"position": 1}]}),
        ),
        (
            "breadcrumb two",
            "BreadcrumbList",
            json!({"itemListElement": [{"position": 1}, {"position": 2}]}),
        ),
        ("generic", "Thing", json!({"a_list": [1, 2, 3], "b": "x"})),
    ];
    for (label, schema_type, content) in schemas {
        push(
            &mut out,
            format!("schema row: {label}"),
            Box::new(move |s, _| {
                s.structured_data.has_structured_data = true;
                s.structured_data
                    .json_ld
                    .push(json_ld(schema_type, content.clone()));
            }),
        );
    }

    // Schema status: grouped structural issues, non-JSON-LD data.
    push(
        &mut out,
        "schema status: grouped issues".to_string(),
        Box::new(|s, _| {
            s.structured_data.has_structured_data = true;
            for issue_type in [
                "jsonld_parse_error",
                "jsonld_parse_error",
                "jsonld_missing_type",
                "schema_missing_name",
            ] {
                s.structured_data.schema_issues.push(SchemaIssue {
                    schema_type: "Article".to_string(),
                    severity: SchemaIssueSeverity::Required,
                    issue_type: issue_type.to_string(),
                    message: "message".to_string(),
                });
            }
        }),
    );
    push(
        &mut out,
        "schema status: microdata".to_string(),
        Box::new(|s, _| s.structured_data.has_structured_data = true),
    );

    // Schema fit and content parity.
    for status in crate::i18n::detail_texts_export::SCHEMA_FIT_STATUSES {
        push(
            &mut out,
            format!("schema fit: {status:?}"),
            Box::new(move |s, _| {
                s.structured_data.fit_assessment =
                    Some(crate::i18n::detail_texts_export::schema_fit(status));
            }),
        );
    }
    push(
        &mut out,
        "schema fit: no types".to_string(),
        Box::new(|s, _| {
            let mut fit = crate::i18n::detail_texts_export::schema_fit(
                crate::i18n::detail_texts_export::SCHEMA_FIT_STATUSES[0],
            );
            fit.expected_primary_types.clear();
            fit.detected_primary_types.clear();
            s.structured_data.fit_assessment = Some(fit);
        }),
    );
    push(
        &mut out,
        "schema parity".to_string(),
        Box::new(|s, _| {
            s.structured_data.content_parity = crate::i18n::detail_texts_export::content_parities();
        }),
    );

    // Schema rule assessments.
    push(
        &mut out,
        "schema rules".to_string(),
        Box::new(|s, _| {
            for availability in [
                SchemaFeatureAvailability::General,
                SchemaFeatureAvailability::Limited,
                SchemaFeatureAvailability::ContextDependent,
            ] {
                for (status, required, recommended) in [
                    (
                        SchemaRequirementStatus::MissingRequiredProperties,
                        &["name", "offers"][..],
                        &["image"][..],
                    ),
                    (
                        SchemaRequirementStatus::MeetsRequiredProperties,
                        &[][..],
                        &["image", "brand"][..],
                    ),
                    (SchemaRequirementStatus::NotEvaluated, &[][..], &[][..]),
                    (
                        SchemaRequirementStatus::RecommendationsOnly,
                        &[][..],
                        &[][..],
                    ),
                ] {
                    s.structured_data.rule_assessments.push(assessment(
                        availability,
                        status,
                        required,
                        recommended,
                        vec![
                            ManualReview::MerchantListingContext,
                            ManualReview::RecipeCookAndPrepTime,
                        ],
                    ));
                }
            }
        }),
    );

    // Content profile pages: classification, attributes, topic hints,
    // signal ratings and maturity.
    for (label, seo) in crate::seo::profile_texts_export::profile_scenarios() {
        out.push(Scenario {
            label: format!("profile: {label}"),
            seo,
            consent: None,
        });
    }
    push(
        &mut out,
        "rich page".to_string(),
        Box::new(|s, _| {
            s.meta.title = Some("A good page title of a fitting length".to_string());
            s.meta.description = Some(
                "A description of the page that is long enough to count as a real description \
                 of what the page offers to its readers."
                    .to_string(),
            );
            s.meta.viewport = Some("width=device-width".to_string());
            s.meta.canonical = Some("https://example.com/".to_string());
            s.technical.https = true;
            s.technical.has_canonical = true;
            s.technical.has_lang = true;
            s.technical.lang = Some("en".to_string());
            s.technical.word_count = 1_800;
            s.technical.internal_links = 40;
            s.social.open_graph = Some(OpenGraph {
                title: Some("T".into()),
                description: Some("D".into()),
                image: Some("I".into()),
                url: Some("U".into()),
                og_type: Some("article".into()),
                site_name: Some("S".into()),
                locale: Some("L".into()),
            });
            s.social.twitter_card = Some(TwitterCard {
                card: Some("summary".into()),
                title: Some("T".into()),
                description: Some("D".into()),
                image: Some("I".into()),
                site: Some("@s".into()),
                creator: Some("@c".into()),
            });
            s.social.completeness = 100;
            s.structured_data.has_structured_data = true;
            s.structured_data.json_ld.push(json_ld(
                "Article",
                json!({"headline": "H", "author": {"name": "A"}, "datePublished": "2026-01-01",
                       "image": "i.png", "publisher": {"name": "P"}}),
            ));
        }),
    );

    // robots.txt.
    for (label, audit) in crate::seo::label_texts_export::robots_audits() {
        push(
            &mut out,
            format!("robots: {label}"),
            Box::new(move |s, _| s.robots = Some(audit.clone())),
        );
    }

    // Technical issue rows across severities.
    for (label, technical) in crate::seo::issue_texts_export::technical_scenarios() {
        push(
            &mut out,
            format!("technical issues: {label}"),
            Box::new(move |s, _| s.technical = technical.clone()),
        );
    }

    out
}

/// Every text of a presentation, in the order the presentation holds them.
pub(super) fn presentation_texts(p: &SeoPresentation) -> Value {
    let profile = p.profile.as_ref().map(|cp| {
        json!({
            "identity_summary": cp.identity_summary,
            "site_name": cp.site_name,
            "content_type": cp.content_type,
            "language": cp.language,
            "category_hints": cp.category_hints,
            "identity_facts": cp.identity_facts,
            "page_type": cp.page_type,
            "page_attributes": cp.page_attributes,
            "page_profile_summary": cp.page_profile_summary,
            "optimization_note": cp.optimization_note,
            "page_profile_facts": cp.page_profile_facts,
            "schema_rows": cp.schema_rows,
            "schema_status_summary": cp.schema_status_summary,
            "schema_status_rows": cp.schema_status_rows,
            "schema_fit_summary": cp.schema_fit_summary,
            "schema_fit_facts": cp.schema_fit_facts,
            "schema_rule_rows": cp.schema_rule_rows,
            "schema_manual_review_rows": cp.schema_manual_review_rows,
            "schema_parity_rows": cp.schema_parity_rows,
            "signal_rows": cp.signal_rows,
            "signal_details": cp.signal_details,
            "maturity_level": cp.maturity_level,
            "maturity_description": cp.maturity_description,
        })
    });
    let robots = p.robots.as_ref().map(|r| {
        json!({
            "inferred_policy": r.inferred_policy,
            "bot_rows": r.bot_rows,
            "blocked_ai_bots": r.blocked_ai_bots,
        })
    });
    let meta_issues: Vec<_> = p
        .meta_issues
        .iter()
        .map(|(field, severity, message)| json!([field, format!("{severity:?}"), message]))
        .collect();
    json!({
        "interpretation": p.interpretation,
        "meta_tags": p.meta_tags,
        "meta_issues": meta_issues,
        "heading_summary": p.heading_summary,
        "social_summary": p.social_summary,
        "technical_summary": p.technical_summary,
        "tracking_summary": p.tracking_summary,
        "tracking_summary_text": p.tracking_summary_text,
        "profile": profile,
        "robots": robots,
        "technical_issues": p.technical_issues,
    })
}

/// The presentation of one scenario in one language, through
/// `normalize_checked` like a real audit.
pub(super) fn presentation(scenario: &Scenario, i18n: &I18n) -> Value {
    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    report.discoverability.seo = Some(scenario.seo.clone());
    report.consent_privacy = scenario.consent.clone();
    let normalized = crate::audit::validation::normalize_checked(&report)
        .expect("fixture report passes the report contract");
    let presentation = build_seo_details(&normalized, i18n).expect("the page has an SEO analysis");
    presentation_texts(&presentation)
}

/// Every record of the export, in both languages.
pub(super) fn records() -> Vec<Value> {
    let scenarios = scenarios();
    let mut records = Vec::new();
    for lang in LANGS {
        let i18n = I18n::new(lang).unwrap();
        for scenario in &scenarios {
            records.push(json!({
                "scenario": scenario.label,
                "lang": lang,
                "seo": presentation(scenario, &i18n),
            }));
        }
    }
    records
}

#[test]
#[ignore = "writes reports/seo-details-texts.json; run before and after a change of the text storage"]
fn export_seo_details_texts() {
    let records = records();
    let path = std::env::var("SEO_DETAILS_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/seo-details-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&records).unwrap()).unwrap();
    println!(
        "{} SEO detail text records written to {path}",
        records.len()
    );
}
