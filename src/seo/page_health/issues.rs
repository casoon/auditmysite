//! Issue aggregation for the page-health analysis.

use fluent_bundle::FluentValue;

use super::{PageHealthAnalysis, PageHealthIssue};

/// A count or number as a Fluent argument (plural selectors need a number).
fn num(n: u32) -> FluentValue<'static> {
    FluentValue::from(i64::from(n))
}

/// Text as a Fluent argument.
fn txt(s: &str) -> FluentValue<'static> {
    FluentValue::from(s.to_string())
}

/// `medium` from five findings on, `low` below.
fn medium_from_five(n: u32) -> &'static str {
    if n >= 5 {
        "medium"
    } else {
        "low"
    }
}

/// The reason a resource's cache policy is inefficient, in the requested
/// language. The probe stores canonical English; unknown text is returned as is.
pub(crate) fn cache_reason_text(reason: &str, en: bool) -> String {
    let key = match reason {
        "missing Cache-Control" => "page-health-cache-reason-missing",
        "explicitly disables caching" => "page-health-cache-reason-disabled",
        "short or missing max-age/s-maxage" => "page-health-cache-reason-short",
        other => return other.to_string(),
    };
    crate::i18n::text(key, en)
}

/// A sample list appended to a sentence: nothing for an empty list, otherwise
/// the "(e.g. …)" part.
fn examples_text(samples: &[String], en: bool) -> String {
    if samples.is_empty() {
        return String::new();
    }
    crate::i18n::text_args(
        "page-health-sample-examples",
        &[("list", txt(&samples.join(", ")))],
        en,
    )
}

/// Build the page-health issue list in the requested language.
///
/// Pure function of the analysis struct — called with English at analysis time
/// (canonical, for JSON) and re-called with the report locale by the PDF
/// presentation builder (#406). The texts are the `page-health-*` messages of
/// `locales/{en,de}/page-health.ftl`; this function decides which issues
/// exist, their severity and the arguments of their messages.
pub fn collect_issues(a: &PageHealthAnalysis, en: bool) -> Vec<PageHealthIssue> {
    let mut issues = Vec::new();
    let mut add = |issue_type: &str, severity: &str, key: &str, args: &[(&str, FluentValue)]| {
        issues.push(PageHealthIssue {
            issue_type: issue_type.to_string(),
            message: crate::i18n::text_args(key, args, en),
            severity: severity.to_string(),
        });
    };

    if !a.has_doctype && a.dom_node_count > 0 {
        add(
            "missing_doctype",
            "high",
            "page-health-missing-doctype",
            &[],
        );
    }

    if a.document_write_count > 0 {
        add(
            "document_write",
            "medium",
            "page-health-document-write",
            &[("count", num(a.document_write_count))],
        );
    }

    if a.dom_node_count >= 1500 {
        add(
            "excessive_dom",
            "high",
            "page-health-excessive-dom",
            &[
                ("nodes", num(a.dom_node_count)),
                ("depth", num(a.dom_max_depth)),
            ],
        );
    } else if a.dom_node_count >= 800 {
        add(
            "large_dom",
            "medium",
            "page-health-large-dom",
            &[
                ("nodes", num(a.dom_node_count)),
                ("depth", num(a.dom_max_depth)),
            ],
        );
    }

    if a.images_without_dimensions > 0 {
        add(
            "images_without_dimensions",
            "medium",
            "page-health-images-without-dimensions",
            &[("count", num(a.images_without_dimensions))],
        );
    }

    if a.paste_blocking_password_fields > 0 {
        add(
            "paste_blocked_password",
            "medium",
            "page-health-paste-blocked-password",
            &[("count", num(a.paste_blocking_password_fields))],
        );
    }

    if a.offscreen_images_without_lazy > 0 {
        add(
            "offscreen_images_without_lazy",
            medium_from_five(a.offscreen_images_without_lazy),
            "page-health-offscreen-images-without-lazy",
            &[("count", num(a.offscreen_images_without_lazy))],
        );
    }

    if a.images_without_srcset > 0 {
        add(
            "images_without_srcset",
            medium_from_five(a.images_without_srcset),
            "page-health-images-without-srcset",
            &[("count", num(a.images_without_srcset))],
        );
    }

    if a.missing_preconnect_count > 0 {
        add(
            "missing_preconnect",
            "low",
            "page-health-missing-preconnect",
            &[
                ("count", num(a.missing_preconnect_count)),
                (
                    "sample",
                    txt(&examples_text(&a.missing_preconnect_origins, en)),
                ),
            ],
        );
    }

    if a.non_crawlable_links > 0 {
        add(
            "non_crawlable_links",
            medium_from_five(a.non_crawlable_links),
            "page-health-non-crawlable-links",
            &[("count", num(a.non_crawlable_links))],
        );
    }

    if a.images_without_modern_format > 0 {
        add(
            "images_without_modern_format",
            medium_from_five(a.images_without_modern_format),
            "page-health-images-without-modern-format",
            &[("count", num(a.images_without_modern_format))],
        );
    }

    if a.oversized_images > 0 {
        add(
            "oversized_images",
            "medium",
            "page-health-oversized-images",
            &[("count", num(a.oversized_images))],
        );
    }

    if a.gif_images > 0 {
        add(
            "gif_images",
            "low",
            "page-health-gif-images",
            &[("count", num(a.gif_images))],
        );
    }

    if a.font_display_issues > 0 {
        add(
            "font_display_missing",
            "medium",
            "page-health-font-display-missing",
            &[("count", num(a.font_display_issues))],
        );
    }

    if a.fonts_without_preload_count > 0 {
        add(
            "fonts_not_preloaded",
            "low",
            "page-health-fonts-not-preloaded",
            &[
                ("count", num(a.fonts_without_preload_count)),
                ("total", num(a.font_face_count)),
            ],
        );
    }

    if a.orphaned_preload_count > 0 {
        add(
            "orphaned_preload",
            "low",
            "page-health-orphaned-preload",
            &[("count", num(a.orphaned_preload_count))],
        );
    }

    if a.lcp_image_lazy_loaded {
        add(
            "lcp_image_lazy_loaded",
            "high",
            "page-health-lcp-image-lazy-loaded",
            &[],
        );
    }

    if a.lcp_image_without_preload {
        let url_hint = a
            .lcp_image_url
            .as_deref()
            .map(|u| format!(" ({u})"))
            .unwrap_or_default();
        add(
            "lcp_image_without_preload",
            "medium",
            "page-health-lcp-image-without-preload",
            &[("url", txt(&url_hint))],
        );
    }

    if a.lcp_image_without_fetchpriority && !a.lcp_image_lazy_loaded {
        add(
            "lcp_image_without_fetchpriority",
            "low",
            "page-health-lcp-image-without-fetchpriority",
            &[],
        );
    }

    if a.deprecated_api_count > 0 {
        add(
            "deprecated_apis",
            "medium",
            "page-health-deprecated-apis",
            &[("count", num(a.deprecated_api_count))],
        );
    }

    if a.uses_http2 == Some(false) {
        add("http1_only", "medium", "page-health-http1-only", &[]);
    }

    if a.has_compression == Some(false) {
        add(
            "missing_compression",
            "high",
            "page-health-missing-compression",
            &[],
        );
    }

    if !a.has_efficient_cache && a.cache_control.is_some() {
        add(
            "inefficient_cache",
            "medium",
            "page-health-inefficient-cache",
            &[("value", txt(a.cache_control.as_deref().unwrap_or("")))],
        );
    }

    if a.resource_cache.inefficient_resources > 0 {
        let sample = a
            .resource_cache
            .samples
            .first()
            .map(|finding| {
                crate::i18n::text_args(
                    "page-health-sample-cache-resource",
                    &[
                        ("url", txt(&finding.url)),
                        ("reason", txt(&cache_reason_text(&finding.reason, en))),
                    ],
                    en,
                )
            })
            .unwrap_or_default();
        add(
            "inefficient_resource_cache",
            medium_from_five(a.resource_cache.inefficient_resources),
            "page-health-inefficient-resource-cache",
            &[
                ("inefficient", num(a.resource_cache.inefficient_resources)),
                ("total", num(a.resource_cache.cacheable_resources)),
                ("sample", txt(&sample)),
            ],
        );
    }

    if a.resource_cache.missing_content_type > 0 || a.resource_cache.incorrect_content_type > 0 {
        let sample = a
            .resource_cache
            .content_type_samples
            .first()
            .map(|finding| {
                let actual = finding
                    .actual
                    .clone()
                    .unwrap_or_else(|| crate::i18n::text("page-health-none", en));
                crate::i18n::text_args(
                    "page-health-sample-content-type",
                    &[
                        ("url", txt(&finding.url)),
                        ("expected", txt(&finding.expected)),
                        ("actual", txt(&actual)),
                    ],
                    en,
                )
            })
            .unwrap_or_default();
        add(
            "incorrect_content_type",
            "medium",
            "page-health-incorrect-content-type",
            &[
                (
                    "total",
                    num(a.resource_cache.missing_content_type
                        + a.resource_cache.incorrect_content_type),
                ),
                ("sample", txt(&sample)),
            ],
        );
    }

    // Charset consistency between the HTTP Content-Type header and the
    // declared <meta charset> (#543). Only meaningful when both are present —
    // a page missing one or the other is covered by its own separate check
    // elsewhere (meta validation), not here.
    if let (Some(header), Some(declared)) = (&a.charset_header, &a.declared_charset) {
        let declared_normalized = declared.to_ascii_lowercase();
        if header != &declared_normalized {
            add(
                "charset_mismatch",
                "medium",
                "page-health-charset-mismatch",
                &[("header", txt(header)), ("declared", txt(declared))],
            );
        }
    }

    if a.hreflang_invalid_count > 0 {
        add(
            "hreflang_invalid",
            "medium",
            "page-health-hreflang-invalid",
            &[("count", num(a.hreflang_invalid_count))],
        );
    }

    if a.jsonld_invalid_count > 0 {
        add(
            "jsonld_invalid",
            "medium",
            "page-health-jsonld-invalid",
            &[("count", num(a.jsonld_invalid_count))],
        );
    }

    if a.sync_head_scripts > 0 {
        add(
            "sync_head_scripts",
            "medium",
            "page-health-sync-head-scripts",
            &[("count", num(a.sync_head_scripts))],
        );
    }

    let sri_total = a.external_scripts_without_sri + a.external_styles_without_sri;
    if sri_total > 0 {
        add(
            "missing_sri",
            "medium",
            "page-health-missing-sri",
            &[("count", num(sri_total))],
        );
    }

    if a.broken_fragment_links > 0 {
        add(
            "broken_fragment_links",
            "low",
            "page-health-broken-fragment-links",
            &[
                ("count", num(a.broken_fragment_links)),
                (
                    "sample",
                    txt(&examples_text(&a.broken_fragment_samples, en)),
                ),
            ],
        );
    }

    if a.generic_link_text_count > 0 {
        add(
            "generic_link_text",
            "low",
            "page-health-generic-link-text",
            &[("count", num(a.generic_link_text_count))],
        );
    }

    if a.is_soft_404 {
        add(
            "soft_404",
            "high",
            "page-health-soft-404",
            &[("status", num(u32::from(a.soft_404_status.unwrap_or(200))))],
        );
    }

    if let Some(custom_404) = &a.custom_404 {
        if !custom_404.proper_status {
            add(
                "custom_404_invalid_status",
                "high",
                "page-health-custom-404-invalid-status",
                &[("status", num(u32::from(custom_404.status)))],
            );
        } else if !custom_404.custom_page {
            add(
                "generic_404_page",
                "low",
                "page-health-generic-404-page",
                &[],
            );
        }
    }

    if a.has_meta_refresh {
        let delay = a
            .meta_refresh_content
            .as_deref()
            .and_then(|c| c.split(';').next())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        if delay == 0 {
            add(
                "meta_refresh",
                "high",
                "page-health-meta-refresh-immediate",
                &[],
            );
        } else {
            add(
                "meta_refresh",
                "medium",
                "page-health-meta-refresh-delayed",
                &[("delay", num(delay))],
            );
        }
    }

    if a.frame_count > 0 {
        add(
            "frames",
            "high",
            "page-health-frames",
            &[("count", num(a.frame_count))],
        );
    }

    if a.url_is_too_long {
        add(
            "url_too_long",
            "low",
            "page-health-url-too-long",
            &[("length", FluentValue::from(a.url_length as i64))],
        );
    }

    if a.url_is_too_deep {
        add(
            "url_too_deep",
            "low",
            "page-health-url-too-deep",
            &[("depth", FluentValue::from(a.url_path_depth as i64))],
        );
    }

    if a.url_has_query_params {
        add("dynamic_url", "low", "page-health-dynamic-url", &[]);
    }

    if a.redirect_count >= 2 {
        add(
            "multiple_redirects",
            if a.redirect_count >= 3 {
                "high"
            } else {
                "medium"
            },
            "page-health-multiple-redirects",
            &[("count", num(a.redirect_count))],
        );
    } else if a.own_redirect_detected {
        let target = a
            .own_final_url
            .clone()
            .unwrap_or_else(|| crate::i18n::text("page-health-unknown-url", en));
        add(
            "redirect",
            "medium",
            "page-health-redirect",
            &[("url", txt(&target))],
        );
    }

    if let Some(ref www) = a.www_consolidation {
        if !www.is_consolidated {
            add(
                "www_not_consolidated",
                "medium",
                "page-health-www-not-consolidated",
                &[],
            );
        }
    }

    if let Some(ref canon) = a.url_canonicalization {
        if canon.trailing_slash_inconsistent == Some(true) {
            add(
                "trailing_slash_inconsistent",
                "medium",
                "page-health-trailing-slash-inconsistent",
                &[],
            );
        }
        if canon.http_to_https_missing == Some(true) {
            let status_text = canon
                .http_status
                .map(|s| s.to_string())
                .unwrap_or_else(|| crate::i18n::text("page-health-unreachable", en));
            add(
                "http_not_redirected_to_https",
                "high",
                "page-health-http-not-redirected-to-https",
                &[("status", txt(&status_text))],
            );
        }
    }

    issues
}

#[cfg(test)]
mod cache_reason_tests {
    use super::cache_reason_text;

    #[test]
    fn german_reasons_are_german_and_english_is_unchanged() {
        for reason in [
            "missing Cache-Control",
            "explicitly disables caching",
            "short or missing max-age/s-maxage",
        ] {
            assert_eq!(cache_reason_text(reason, true), reason);
            let de = cache_reason_text(reason, false);
            assert_ne!(de, reason);
            assert!(!de.contains("missing") && !de.contains("short"));
        }
    }
}
