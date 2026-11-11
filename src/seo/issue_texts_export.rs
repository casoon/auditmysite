//! Text export of the SEO issue texts (#936): every text of
//! `technical::collect_technical_issues`, `meta::meta_issue_text` and
//! `schema::schema_issue_text` in both languages across an enumerated input
//! space, as records in `reports/seo-issue-texts.json` (gitignored; the path
//! can be set with `SEO_ISSUE_TEXT_EXPORT`). Run it before and after a change
//! of how the texts are stored and compare the two files byte for byte.

use super::meta::{meta_issue_text, MetaIssueKind, MetaIssueValues};
use super::schema::{schema_issue_text, SchemaIssue, SchemaIssueSeverity};
use super::technical::{
    collect_technical_issues, AmpAnalysis, FormSecurityAnalysis, HreflangTag, MixedContentResource,
    PwaAnalysis, TechnicalSeo, WwwRedirectCheck,
};
use serde_json::json;

type Apply = Box<dyn Fn(&mut TechnicalSeo)>;

/// A page with none of the technical issues; every scenario changes from it.
fn quiet() -> TechnicalSeo {
    TechnicalSeo {
        https: true,
        canonical_url: Some("https://example.com/".to_string()),
        lang: Some("en".to_string()),
        word_count: 1_000,
        internal_links: 5,
        pwa: PwaAnalysis {
            manifest_url: Some("https://example.com/manifest.json".to_string()),
            manifest_valid: Some(true),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn push(out: &mut Vec<(String, TechnicalSeo)>, label: String, apply: Apply) {
    let mut t = quiet();
    apply(&mut t);
    out.push((label, t));
}

const COUNTS: [u32; 6] = [0, 1, 2, 5, 1_500, 1_234_567];

const TEXTS: [&str; 6] = [
    "",
    "https://example.com/a",
    " padded value ",
    "{braces} and {$var}",
    "äöüß — „quotes“ \"x\" <b>; semi",
    "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/and-going/and-going/and-going/and-going/file.min.js?v=1234567890",
];

pub(super) fn technical_scenarios() -> Vec<(String, TechnicalSeo)> {
    let mut v = Vec::new();

    push(&mut v, "quiet".to_string(), Box::new(|_| {}));
    v.push(("default".to_string(), TechnicalSeo::default()));

    for https in [false, true] {
        push(
            &mut v,
            format!("https={https}"),
            Box::new(move |t| t.https = https),
        );
    }
    push(
        &mut v,
        "no canonical".to_string(),
        Box::new(|t| t.canonical_url = None),
    );
    push(&mut v, "no lang".to_string(), Box::new(|t| t.lang = None));
    for words in [0, 1, 299, 300, 301, 1_234_567] {
        push(
            &mut v,
            format!("word_count={words}"),
            Box::new(move |t| t.word_count = words),
        );
    }
    for links in [0, 1, 5] {
        push(
            &mut v,
            format!("internal_links={links}"),
            Box::new(move |t| t.internal_links = links),
        );
    }
    push(
        &mut v,
        "google fonts".to_string(),
        Box::new(|t| t.google_fonts_sources = vec!["https://fonts.googleapis.com/css".to_string()]),
    );
    for robots in [
        None,
        Some("index, follow"),
        Some("noindex"),
        Some("NoIndex, nofollow"),
    ] {
        push(
            &mut v,
            format!("robots={robots:?}"),
            Box::new(move |t| t.robots_meta = robots.map(String::from)),
        );
    }
    for hreflang in [false, true] {
        for x_default in [false, true] {
            for self_reference in [false, true] {
                push(
                    &mut v,
                    format!(
                        "hreflang={hreflang} x_default={x_default} missing_self={self_reference}"
                    ),
                    Box::new(move |t| {
                        if hreflang {
                            t.hreflang = vec![HreflangTag {
                                lang: "de".to_string(),
                                url: "https://example.com/de".to_string(),
                            }];
                        }
                        t.hreflang_has_x_default = x_default;
                        t.hreflang_missing_self_reference = self_reference;
                    }),
                );
            }
        }
    }
    for n in COUNTS {
        push(
            &mut v,
            format!("internal_links_with_query_params={n}"),
            Box::new(move |t| t.internal_links_with_query_params = n),
        );
    }

    let primary = "https://example.com/blog/";
    let pages = [
        None,
        Some("https://example.com/blog"),
        Some("https://example.com/blog?page=2"),
        Some("https://example.com/blog/#top"),
    ];
    for detected in [false, true] {
        for prev in pages {
            for next in pages {
                for with_www in [false, true] {
                    push(
                        &mut v,
                        format!("pagination detected={detected} prev={prev:?} next={next:?} www={with_www}"),
                        Box::new(move |t| {
                            t.pagination_detected = detected;
                            t.pagination_prev = prev.map(String::from);
                            t.pagination_next = next.map(String::from);
                            if with_www {
                                t.www_redirect = Some(WwwRedirectCheck {
                                    primary: primary.to_string(),
                                    alternative: "https://www.example.com/blog/".to_string(),
                                    alternative_status: 301,
                                    redirects_to_primary: true,
                                    canonical_matches_primary: Some(true),
                                });
                            }
                        }),
                    );
                }
            }
        }
    }

    for n in 0..=5usize {
        for name_set in 0..3usize {
            push(
                &mut v,
                format!("broken_links n={n} names#{name_set}"),
                Box::new(move |t| {
                    t.broken_links = (0..n)
                        .map(|i| {
                            format!("{} (HTTP {})", TEXTS[(i + name_set) % TEXTS.len()], 404 + i)
                        })
                        .collect();
                }),
            );
        }
    }

    for n in 0..=4usize {
        for blockable in 0..=4usize {
            push(
                &mut v,
                format!("mixed_content n={n} blockable={blockable}"),
                Box::new(move |t| {
                    t.mixed_content = (0..n)
                        .map(|i| MixedContentResource {
                            url: format!("http://example.com/{}", TEXTS[i % TEXTS.len()]),
                            resource_type: "script".to_string(),
                            blockable: i < blockable,
                        })
                        .collect();
                }),
            );
        }
    }

    for og in [false, true] {
        for twitter in [false, true] {
            push(
                &mut v,
                format!("og_alt_missing={og} twitter_alt_missing={twitter}"),
                Box::new(move |t| {
                    t.og_image_missing_alt = og;
                    t.twitter_image_missing_alt = twitter;
                }),
            );
        }
    }

    push(
        &mut v,
        "pwa no manifest".to_string(),
        Box::new(|t| t.pwa = PwaAnalysis::default()),
    );
    for valid in [None, Some(false), Some(true)] {
        push(
            &mut v,
            format!("pwa manifest valid={valid:?}"),
            Box::new(move |t| t.pwa.manifest_valid = valid),
        );
    }

    let amp_issues = [
        "missing_html_amp_attribute",
        "missing_canonical",
        "missing_amp_runtime",
        "missing_amp_boilerplate",
        "custom_script_not_allowed",
        "something_unknown",
        "",
    ];
    for detected in [false, true] {
        for issue in amp_issues {
            push(
                &mut v,
                format!("amp detected={detected} issue={issue:?}"),
                Box::new(move |t| {
                    t.amp = AmpAnalysis {
                        detected,
                        issues: vec![issue.to_string()],
                        ..Default::default()
                    };
                }),
            );
        }
        push(
            &mut v,
            format!("amp detected={detected} all issues"),
            Box::new(move |t| {
                t.amp = AmpAnalysis {
                    detected,
                    issues: amp_issues.iter().map(|s| s.to_string()).collect(),
                    ..Default::default()
                };
            }),
        );
        push(
            &mut v,
            format!("amp detected={detected} no issues"),
            Box::new(move |t| {
                t.amp = AmpAnalysis {
                    detected,
                    ..Default::default()
                };
            }),
        );
    }

    for insecure in COUNTS {
        for get in COUNTS {
            for password in COUNTS {
                push(
                    &mut v,
                    format!("forms insecure={insecure} get={get} password={password}"),
                    Box::new(move |t| {
                        t.form_security = FormSecurityAnalysis {
                            sensitive_forms: 0,
                            insecure_transport_forms: insecure,
                            get_sensitive_forms: get,
                            password_autocomplete_issues: password,
                        };
                    }),
                );
            }
        }
    }

    for primary in [TEXTS[0], TEXTS[1], TEXTS[3], TEXTS[4]] {
        for alternative in [TEXTS[1], TEXTS[2]] {
            for status in [200, 301, 404] {
                for redirects in [false, true] {
                    for canonical in [None, Some(true), Some(false)] {
                        push(
                            &mut v,
                            format!(
                                "www primary={primary:?} alt={alternative:?} status={status} redirects={redirects} canonical={canonical:?}"
                            ),
                            Box::new(move |t| {
                                t.www_redirect = Some(WwwRedirectCheck {
                                    primary: primary.to_string(),
                                    alternative: alternative.to_string(),
                                    alternative_status: status,
                                    redirects_to_primary: redirects,
                                    canonical_matches_primary: canonical,
                                });
                            }),
                        );
                    }
                }
            }
        }
    }

    push(
        &mut v,
        "everything".to_string(),
        Box::new(|t| {
            t.https = false;
            t.canonical_url = None;
            t.lang = None;
            t.word_count = 120;
            t.internal_links = 0;
            t.google_fonts_sources = vec!["https://fonts.gstatic.com/x".to_string()];
            t.robots_meta = Some("noindex".to_string());
            t.hreflang = vec![HreflangTag {
                lang: "de".to_string(),
                url: "https://example.com/de".to_string(),
            }];
            t.hreflang_missing_self_reference = true;
            t.internal_links_with_query_params = 3;
            t.pagination_detected = true;
            t.pagination_prev = Some("https://example.com/x/".to_string());
            t.pagination_next = Some("https://example.com/x/".to_string());
            t.broken_links = vec!["https://example.com/a (HTTP 404)".to_string(); 4];
            t.mixed_content = vec![MixedContentResource {
                url: "http://example.com/a.js".to_string(),
                resource_type: "script".to_string(),
                blockable: true,
            }];
            t.og_image_missing_alt = true;
            t.twitter_image_missing_alt = true;
            t.pwa = PwaAnalysis {
                manifest_url: Some("https://example.com/m.json".to_string()),
                manifest_valid: Some(false),
                ..Default::default()
            };
            t.amp = AmpAnalysis {
                detected: true,
                issues: vec![
                    "missing_canonical".to_string(),
                    "missing_amp_runtime".to_string(),
                ],
                ..Default::default()
            };
            t.form_security = FormSecurityAnalysis {
                sensitive_forms: 3,
                insecure_transport_forms: 2,
                get_sensitive_forms: 1,
                password_autocomplete_issues: 2,
            };
            t.www_redirect = Some(WwwRedirectCheck {
                primary: "https://example.com/x/".to_string(),
                alternative: "https://www.example.com/x/".to_string(),
                alternative_status: 200,
                redirects_to_primary: false,
                canonical_matches_primary: Some(false),
            });
        }),
    );

    v
}

pub(super) const META_KINDS: [MetaIssueKind; 11] = [
    MetaIssueKind::MissingTitle,
    MetaIssueKind::TitleTooShort,
    MetaIssueKind::TitleTooLong,
    MetaIssueKind::MissingDescription,
    MetaIssueKind::DescriptionTooShort,
    MetaIssueKind::DescriptionTooLong,
    MetaIssueKind::MissingViewport,
    MetaIssueKind::MissingLang,
    MetaIssueKind::MissingCanonical,
    MetaIssueKind::InvalidTwitterCard,
    MetaIssueKind::RelativeOgImage,
];

pub(super) fn meta_values() -> Vec<MetaIssueValues> {
    let mut out = Vec::new();
    let lengths = [
        None,
        Some(0),
        Some(1),
        Some(29),
        Some(30),
        Some(60),
        Some(61),
        Some(160),
        Some(161),
        Some(1_234_567),
    ];
    let values = [
        None,
        Some(""),
        Some("summary"),
        Some("\"quoted\""),
        Some("{x} {$y}"),
        Some("äöüß"),
        Some(TEXTS[5]),
    ];
    let alloweds = [
        None,
        Some(""),
        Some("summary, summary_large_image, app, player"),
    ];
    for length in lengths {
        for value in values {
            for allowed in alloweds {
                out.push(MetaIssueValues {
                    length,
                    value: value.map(String::from),
                    allowed: allowed.map(String::from),
                });
            }
        }
    }
    out
}

pub(super) fn schema_issues() -> Vec<SchemaIssue> {
    let mut out = Vec::new();
    let issue_types = [
        "jsonld_parse_error",
        "jsonld_invalid_root",
        "jsonld_missing_context",
        "jsonld_missing_type",
        "jsonld_graph_not_array",
        "jsonld_empty_document",
        "schema_missing_name",
        "schema_recommended_missing_image",
        "schema_missing_",
        "schema_recommended_missing_",
        "schema_missing_offers.price {x}",
        "schema_recommended_missing_schema_missing_x",
        "schema_missing_äö",
        "some_future_issue",
        "jsonld_parse_error ",
        "",
    ];
    let schema_types = ["", "Article", "Local Business {x}", "FAQPage", "Äö"];
    let messages = ["", "fallback message", "{y} äö — text"];
    for issue_type in issue_types {
        for schema_type in schema_types {
            for message in messages {
                for severity in [
                    SchemaIssueSeverity::Required,
                    SchemaIssueSeverity::Recommended,
                ] {
                    out.push(SchemaIssue {
                        schema_type: schema_type.to_string(),
                        severity,
                        issue_type: issue_type.to_string(),
                        message: message.to_string(),
                    });
                }
            }
        }
    }
    out
}

#[test]
#[ignore = "writes reports/seo-issue-texts.json; run before and after a change of the text storage"]
fn export_seo_issue_texts() {
    let mut technical = Vec::new();
    for (label, t) in technical_scenarios() {
        for (lang, en) in [("en", true), ("de", false)] {
            let issues: Vec<_> = collect_technical_issues(&t, en)
                .into_iter()
                .map(|i| json!([i.issue_type, format!("{:?}", i.severity), i.message]))
                .collect();
            technical.push(json!({ "scenario": label, "lang": lang, "issues": issues }));
        }
    }
    let mut meta = Vec::new();
    for kind in META_KINDS {
        for values in meta_values() {
            for (lang, en) in [("en", true), ("de", false)] {
                meta.push(json!([
                    format!("{kind:?}"),
                    values.length,
                    values.value,
                    values.allowed,
                    lang,
                    meta_issue_text(kind, &values, en)
                ]));
            }
        }
    }
    let mut schema = Vec::new();
    for issue in schema_issues() {
        for (lang, en) in [("en", true), ("de", false)] {
            schema.push(json!([
                issue.issue_type,
                issue.schema_type,
                issue.message,
                lang,
                schema_issue_text(&issue, en)
            ]));
        }
    }
    let path = std::env::var("SEO_ISSUE_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/seo-issue-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    let export = json!({ "technical": technical, "meta": meta, "schema": schema });
    std::fs::write(&path, serde_json::to_string_pretty(&export).unwrap()).unwrap();
    println!(
        "{} technical, {} meta, {} schema records written to {path}",
        technical.len(),
        meta.len(),
        schema.len()
    );
}
