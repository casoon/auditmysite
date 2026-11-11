//! Text export of the page-health issues (#936): every text `collect_issues`
//! and `cache_reason_text` can produce in both languages across an enumerated
//! input space, as records in `reports/seo-health-texts.json` (gitignored; the
//! path can be set with `HEALTH_TEXT_EXPORT`). Run it before and after a change
//! of how the texts are stored and compare the two files byte for byte.

use super::issues::cache_reason_text;
use super::{
    collect_issues, ContentTypeFinding, Custom404Check, PageHealthAnalysis, ResourceCacheAudit,
    ResourceCacheFinding, UrlCanonicalizationCheck, WwwConsolidation,
};
use serde_json::json;

type Apply = Box<dyn Fn(&mut PageHealthAnalysis)>;

fn push(out: &mut Vec<(String, PageHealthAnalysis)>, label: String, apply: Apply) {
    let mut analysis = PageHealthAnalysis::default();
    apply(&mut analysis);
    out.push((label, analysis));
}

/// Counts that cross every threshold of the page-health rules (0, 1, 2,
/// 4/5/6, the 800/1500 DOM limits) and one that Fluent could group.
const COUNTS: [u32; 8] = [0, 1, 2, 4, 5, 6, 1_500, 1_234_567];

const TEXTS: [&str; 6] = [
    "",
    "https://cdn.example.com/a.js",
    " padded value ",
    "{braces} and {$var}",
    "äöüß — „quotes“ \"x\" <b>",
    "https://example.com/a-very-long-path/that-keeps-going/and-going/and-going/and-going/and-going/and-going/and-going/and-going/and-going/file.min.js?v=1234567890",
];

fn scenarios() -> Vec<(String, PageHealthAnalysis)> {
    let mut v = Vec::new();

    for doctype in [false, true] {
        for nodes in [0, 1, 799, 800, 1_499, 1_500, 123_456] {
            for depth in [0, 7, 33] {
                push(
                    &mut v,
                    format!("doctype={doctype} dom_node_count={nodes} dom_max_depth={depth}"),
                    Box::new(move |a| {
                        a.has_doctype = doctype;
                        a.dom_node_count = nodes;
                        a.dom_max_depth = depth;
                    }),
                );
            }
        }
    }

    macro_rules! counts {
        ($($field:ident),* $(,)?) => {$(
            for n in COUNTS {
                push(
                    &mut v,
                    format!("{}={n}", stringify!($field)),
                    Box::new(move |a| a.$field = n),
                );
            }
        )*};
    }
    counts!(
        document_write_count,
        images_without_dimensions,
        paste_blocking_password_fields,
        offscreen_images_without_lazy,
        images_without_srcset,
        non_crawlable_links,
        images_without_modern_format,
        oversized_images,
        gif_images,
        font_display_issues,
        orphaned_preload_count,
        deprecated_api_count,
        hreflang_invalid_count,
        jsonld_invalid_count,
        sync_head_scripts,
        generic_link_text_count,
        frame_count,
    );

    for count in COUNTS {
        let lists: [Vec<String>; 3] = [
            vec![],
            vec![TEXTS[1].to_string()],
            TEXTS.iter().map(|t| t.to_string()).collect(),
        ];
        for (i, list) in lists.iter().enumerate() {
            let origins = list.clone();
            push(
                &mut v,
                format!("missing_preconnect_count={count} origins#{i}"),
                Box::new(move |a| {
                    a.missing_preconnect_count = count;
                    a.missing_preconnect_origins = origins.clone();
                }),
            );
            let samples = list.clone();
            push(
                &mut v,
                format!("broken_fragment_links={count} samples#{i}"),
                Box::new(move |a| {
                    a.broken_fragment_links = count;
                    a.broken_fragment_samples = samples.clone();
                }),
            );
        }
    }

    for faces in [0, 1, 2, 10] {
        for without in COUNTS {
            push(
                &mut v,
                format!("font_face_count={faces} fonts_without_preload_count={without}"),
                Box::new(move |a| {
                    a.font_face_count = faces;
                    a.fonts_without_preload_count = without;
                }),
            );
        }
    }

    for lazy in [false, true] {
        for no_preload in [false, true] {
            for no_priority in [false, true] {
                for url in std::iter::once(None).chain(TEXTS.iter().map(|t| Some(*t))) {
                    push(
                        &mut v,
                        format!(
                            "lcp lazy={lazy} no_preload={no_preload} no_priority={no_priority} url={url:?}"
                        ),
                        Box::new(move |a| {
                            a.lcp_image_lazy_loaded = lazy;
                            a.lcp_image_without_preload = no_preload;
                            a.lcp_image_without_fetchpriority = no_priority;
                            a.lcp_image_url = url.map(String::from);
                        }),
                    );
                }
            }
        }
    }

    for http2 in [None, Some(false), Some(true)] {
        for compression in [None, Some(false), Some(true)] {
            push(
                &mut v,
                format!("uses_http2={http2:?} has_compression={compression:?}"),
                Box::new(move |a| {
                    a.uses_http2 = http2;
                    a.has_compression = compression;
                }),
            );
        }
    }

    for efficient in [false, true] {
        for control in std::iter::once(None).chain(TEXTS.iter().map(|t| Some(*t))) {
            push(
                &mut v,
                format!("has_efficient_cache={efficient} cache_control={control:?}"),
                Box::new(move |a| {
                    a.has_efficient_cache = efficient;
                    a.cache_control = control.map(String::from);
                }),
            );
        }
    }

    const REASONS: [&str; 5] = [
        "missing Cache-Control",
        "explicitly disables caching",
        "short or missing max-age/s-maxage",
        "something the probe never says",
        "",
    ];
    for inefficient in COUNTS {
        for cacheable in [0, 1, 2, 1_234_567] {
            for (i, reason) in REASONS.iter().enumerate() {
                for url in [TEXTS[1], TEXTS[5]] {
                    push(
                        &mut v,
                        format!(
                            "resource_cache inefficient={inefficient} cacheable={cacheable} reason#{i} url={url}"
                        ),
                        Box::new(move |a| {
                            a.resource_cache.inefficient_resources = inefficient;
                            a.resource_cache.cacheable_resources = cacheable;
                            a.resource_cache.samples = vec![ResourceCacheFinding {
                                url: url.to_string(),
                                cache_control: None,
                                has_etag: false,
                                has_expires: false,
                                reason: reason.to_string(),
                            }];
                        }),
                    );
                }
            }
        }
        push(
            &mut v,
            format!("resource_cache inefficient={inefficient} no samples"),
            Box::new(move |a| a.resource_cache.inefficient_resources = inefficient),
        );
    }

    for missing in [0, 1, 2, 1_234_567] {
        for incorrect in [0, 1, 2, 1_234_567] {
            let actuals = [
                None,
                Some(""),
                Some("text/html; charset=utf-8"),
                Some("{x}"),
            ];
            for (i, actual) in actuals.into_iter().enumerate() {
                for expected in ["javascript", ""] {
                    push(
                        &mut v,
                        format!(
                            "content_type missing={missing} incorrect={incorrect} actual#{i} expected={expected:?}"
                        ),
                        Box::new(move |a| {
                            a.resource_cache.missing_content_type = missing;
                            a.resource_cache.incorrect_content_type = incorrect;
                            a.resource_cache.content_type_samples = vec![ContentTypeFinding {
                                url: TEXTS[1].to_string(),
                                expected: expected.to_string(),
                                actual: actual.map(String::from),
                            }];
                        }),
                    );
                }
            }
            push(
                &mut v,
                format!("content_type missing={missing} incorrect={incorrect} no samples"),
                Box::new(move |a| {
                    a.resource_cache.missing_content_type = missing;
                    a.resource_cache.incorrect_content_type = incorrect;
                }),
            );
        }
    }

    for header in [None, Some("utf-8"), Some("iso-8859-1"), Some("")] {
        for declared in [
            None,
            Some("utf-8"),
            Some("UTF-8"),
            Some("ISO-8859-1"),
            Some(""),
        ] {
            push(
                &mut v,
                format!("charset header={header:?} declared={declared:?}"),
                Box::new(move |a| {
                    a.charset_header = header.map(String::from);
                    a.declared_charset = declared.map(String::from);
                }),
            );
        }
    }

    for scripts in [0, 1, 2, 5] {
        for styles in [0, 1, 2, 5] {
            push(
                &mut v,
                format!("sri scripts={scripts} styles={styles}"),
                Box::new(move |a| {
                    a.external_scripts_without_sri = scripts;
                    a.external_styles_without_sri = styles;
                }),
            );
        }
    }

    for soft in [false, true] {
        for status in [None, Some(0), Some(200), Some(503)] {
            push(
                &mut v,
                format!("soft_404={soft} status={status:?}"),
                Box::new(move |a| {
                    a.is_soft_404 = soft;
                    a.soft_404_status = status;
                }),
            );
        }
    }

    push(
        &mut v,
        "custom_404=None".to_string(),
        Box::new(|a| a.custom_404 = None),
    );
    for status in [0, 200, 404, 500] {
        for proper in [false, true] {
            for custom_page in [false, true] {
                push(
                    &mut v,
                    format!("custom_404 status={status} proper={proper} custom_page={custom_page}"),
                    Box::new(move |a| {
                        a.custom_404 = Some(Custom404Check {
                            probe_url: TEXTS[1].to_string(),
                            status,
                            proper_status: proper,
                            custom_page,
                        });
                    }),
                );
            }
        }
    }

    for refresh in [false, true] {
        for content in [
            None,
            Some(""),
            Some("0"),
            Some("0; url=https://example.com"),
            Some("5; url=https://example.com"),
            Some("  3 ;x"),
            Some("abc"),
            Some("-1"),
            Some("4294967296"),
        ] {
            push(
                &mut v,
                format!("meta_refresh={refresh} content={content:?}"),
                Box::new(move |a| {
                    a.has_meta_refresh = refresh;
                    a.meta_refresh_content = content.map(String::from);
                }),
            );
        }
    }

    for flag in [false, true] {
        for length in [0, 115, 116, 2_048, 1_234_567] {
            push(
                &mut v,
                format!("url_is_too_long={flag} url_length={length}"),
                Box::new(move |a| {
                    a.url_is_too_long = flag;
                    a.url_length = length;
                }),
            );
        }
        for depth in [0, 5, 6, 8, 1_234_567] {
            push(
                &mut v,
                format!("url_is_too_deep={flag} url_path_depth={depth}"),
                Box::new(move |a| {
                    a.url_is_too_deep = flag;
                    a.url_path_depth = depth;
                }),
            );
        }
        push(
            &mut v,
            format!("url_has_query_params={flag}"),
            Box::new(move |a| a.url_has_query_params = flag),
        );
    }

    for hops in [0, 1, 2, 3, 10, 1_234_567] {
        for detected in [false, true] {
            for target in [None, Some(""), Some(TEXTS[1]), Some(TEXTS[3])] {
                push(
                    &mut v,
                    format!("redirect_count={hops} detected={detected} final={target:?}"),
                    Box::new(move |a| {
                        a.redirect_count = hops;
                        a.own_redirect_detected = detected;
                        a.own_final_url = target.map(String::from);
                    }),
                );
            }
        }
    }

    push(
        &mut v,
        "www=None".to_string(),
        Box::new(|a| a.www_consolidation = None),
    );
    for consolidated in [false, true] {
        push(
            &mut v,
            format!("www is_consolidated={consolidated}"),
            Box::new(move |a| {
                a.www_consolidation = Some(WwwConsolidation {
                    www_status: None,
                    non_www_status: None,
                    www_redirects_to_non_www: false,
                    non_www_redirects_to_www: false,
                    canonical_variant: "inconsistent".to_string(),
                    is_consolidated: consolidated,
                });
            }),
        );
    }

    push(
        &mut v,
        "canonicalization=None".to_string(),
        Box::new(|a| a.url_canonicalization = None),
    );
    for slash in [None, Some(false), Some(true)] {
        for https in [None, Some(false), Some(true)] {
            for status in [None, Some(0), Some(200), Some(503)] {
                push(
                    &mut v,
                    format!(
                        "canonicalization slash={slash:?} https_missing={https:?} http_status={status:?}"
                    ),
                    Box::new(move |a| {
                        a.url_canonicalization = Some(UrlCanonicalizationCheck {
                            trailing_slash_variant_status: None,
                            trailing_slash_inconsistent: slash,
                            http_status: status,
                            http_to_https_missing: https,
                        });
                    }),
                );
            }
        }
    }

    // Everything at once: the order of the issues and their interplay.
    push(
        &mut v,
        "everything".to_string(),
        Box::new(|a| {
            a.has_doctype = false;
            a.dom_node_count = 1_600;
            a.dom_max_depth = 20;
            a.document_write_count = 2;
            a.images_without_dimensions = 4;
            a.paste_blocking_password_fields = 1;
            a.offscreen_images_without_lazy = 6;
            a.images_without_srcset = 7;
            a.missing_preconnect_count = 2;
            a.missing_preconnect_origins = vec![TEXTS[1].to_string(), TEXTS[2].to_string()];
            a.non_crawlable_links = 6;
            a.images_without_modern_format = 6;
            a.oversized_images = 3;
            a.gif_images = 2;
            a.font_display_issues = 3;
            a.font_face_count = 2;
            a.fonts_without_preload_count = 1;
            a.orphaned_preload_count = 1;
            a.lcp_image_without_preload = true;
            a.lcp_image_without_fetchpriority = true;
            a.lcp_image_url = Some(TEXTS[1].to_string());
            a.deprecated_api_count = 2;
            a.uses_http2 = Some(false);
            a.has_compression = Some(false);
            a.cache_control = Some("max-age=60".to_string());
            a.resource_cache = ResourceCacheAudit {
                cacheable_resources: 3,
                inefficient_resources: 2,
                missing_content_type: 1,
                incorrect_content_type: 1,
                samples: vec![ResourceCacheFinding {
                    url: TEXTS[1].to_string(),
                    cache_control: None,
                    has_etag: false,
                    has_expires: false,
                    reason: "missing Cache-Control".to_string(),
                }],
                content_type_samples: vec![ContentTypeFinding {
                    url: TEXTS[1].to_string(),
                    expected: "javascript".to_string(),
                    actual: None,
                }],
                ..Default::default()
            };
            a.hreflang_invalid_count = 1;
            a.jsonld_invalid_count = 1;
            a.sync_head_scripts = 2;
            a.external_scripts_without_sri = 1;
            a.external_styles_without_sri = 1;
            a.broken_fragment_links = 2;
            a.broken_fragment_samples = vec!["#a".to_string(), "#b".to_string()];
            a.generic_link_text_count = 3;
            a.is_soft_404 = true;
            a.custom_404 = Some(Custom404Check {
                probe_url: TEXTS[1].to_string(),
                status: 200,
                proper_status: false,
                custom_page: false,
            });
            a.has_meta_refresh = true;
            a.meta_refresh_content = Some("5; url=x".to_string());
            a.frame_count = 1;
            a.url_is_too_long = true;
            a.url_length = 200;
            a.url_is_too_deep = true;
            a.url_path_depth = 8;
            a.url_has_query_params = true;
            a.redirect_count = 3;
            a.charset_header = Some("iso-8859-1".to_string());
            a.declared_charset = Some("UTF-8".to_string());
            a.www_consolidation = Some(WwwConsolidation {
                www_status: None,
                non_www_status: None,
                www_redirects_to_non_www: false,
                non_www_redirects_to_www: false,
                canonical_variant: "inconsistent".to_string(),
                is_consolidated: false,
            });
            a.url_canonicalization = Some(UrlCanonicalizationCheck {
                trailing_slash_variant_status: Some(200),
                trailing_slash_inconsistent: Some(true),
                http_status: None,
                http_to_https_missing: Some(true),
            });
        }),
    );

    v
}

#[test]
#[ignore = "writes reports/seo-health-texts.json; run before and after a change of the text storage"]
fn export_page_health_texts() {
    let mut records = Vec::new();
    for (label, analysis) in scenarios() {
        for (lang, en) in [("en", true), ("de", false)] {
            let issues: Vec<_> = collect_issues(&analysis, en)
                .into_iter()
                .map(|i| json!([i.issue_type, i.severity, i.message]))
                .collect();
            records.push(json!({ "scenario": label, "lang": lang, "issues": issues }));
        }
    }
    let mut reasons = Vec::new();
    for reason in [
        "missing Cache-Control",
        "explicitly disables caching",
        "short or missing max-age/s-maxage",
        "something the probe never says",
        "",
        " missing Cache-Control",
    ] {
        for (lang, en) in [("en", true), ("de", false)] {
            reasons.push(json!([reason, lang, cache_reason_text(reason, en)]));
        }
    }
    let path = std::env::var("HEALTH_TEXT_EXPORT").unwrap_or_else(|_| {
        format!(
            "{}/reports/seo-health-texts.json",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
    let export = json!({ "cache_reasons": reasons, "scenarios": records });
    std::fs::write(&path, serde_json::to_string_pretty(&export).unwrap()).unwrap();
    println!("{} scenario records written to {path}", records.len());
}
