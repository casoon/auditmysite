//! DOM inspection via a single CDP evaluate.

use chromiumoxide::Page;

use crate::audit::collection::{
    evaluate_js, CollectionLog, CollectionMethod, CollectorSpec, JsObject,
};

use super::{HtmlValidationIssue, HtmlValidationKind, PageHealthAnalysis};

pub(crate) static DOM_INSPECTION: CollectorSpec = CollectorSpec {
    id: "seo.page_health.dom",
    module: "seo",
    method: CollectionMethod::JavaScript,
    method_version: 1,
    scoring: false,
};

/// Reads the page-health DOM signals into `a`. Returns `false` when the
/// script result is missing or invalid: then none of the DOM fields is a
/// measurement (#777).
pub(super) async fn run_dom_inspection(
    page: &Page,
    url: &str,
    a: &mut PageHealthAnalysis,
    log: &CollectionLog,
) -> bool {
    let js = include_str!("js/dom_inspection.js");

    let Some(parsed) = log
        .record(&DOM_INSPECTION, evaluate_js::<JsObject>(page, js).await)
        .into_value()
        .map(serde_json::Value::Object)
    else {
        return false;
    };

    // Final URL / redirect detection
    if let Some(final_url) = parsed["finalUrl"].as_str() {
        // Compare canonicalized URLs (ignore trailing slash differences)
        let canonical = |s: &str| s.trim_end_matches('/').to_string();
        if canonical(final_url) != canonical(url) {
            a.own_redirect_detected = true;
            a.own_final_url = Some(final_url.to_string());
        }
    }

    // Meta-refresh
    a.has_meta_refresh = parsed["hasMetaRefresh"].as_bool().unwrap_or(false);
    a.meta_refresh_content = parsed["metaRefreshContent"].as_str().map(String::from);

    // Frames
    a.frame_count = parsed["frameCount"].as_u64().unwrap_or(0) as u32;
    a.iframe_count = parsed["iframeCount"].as_u64().unwrap_or(0) as u32;
    a.cross_origin_iframe_count = parsed["crossOriginIframeCount"].as_u64().unwrap_or(0) as u32;

    // HTML validation
    a.duplicate_id_count = parsed["duplicateIdCount"].as_u64().unwrap_or(0) as u32;
    a.images_without_alt = parsed["imagesWithoutAlt"].as_u64().unwrap_or(0) as u32;
    a.tables_without_headers = parsed["tablesWithoutHeaders"].as_u64().unwrap_or(0) as u32;
    a.empty_headings = parsed["emptyHeadings"].as_u64().unwrap_or(0) as u32;
    a.nested_interactive_count = parsed["nestedInteractive"].as_u64().unwrap_or(0) as u32;
    a.has_doctype = parsed["hasDoctype"].as_bool().unwrap_or(false);
    a.document_write_count = parsed["documentWriteCount"].as_u64().unwrap_or(0) as u32;
    a.dom_node_count = parsed["domNodeCount"].as_u64().unwrap_or(0) as u32;
    a.dom_max_depth = parsed["domMaxDepth"].as_u64().unwrap_or(0) as u32;
    a.images_without_dimensions = parsed["imagesWithoutDimensions"].as_u64().unwrap_or(0) as u32;
    a.paste_blocking_password_fields =
        parsed["pasteBlockingPasswords"].as_u64().unwrap_or(0) as u32;
    a.offscreen_images_without_lazy = parsed["offscreenWithoutLazy"].as_u64().unwrap_or(0) as u32;
    a.images_without_srcset = parsed["imagesWithoutSrcset"].as_u64().unwrap_or(0) as u32;
    a.missing_preconnect_count = parsed["missingPreconnectCount"].as_u64().unwrap_or(0) as u32;
    a.missing_preconnect_origins = parsed["missingPreconnectOrigins"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    a.non_crawlable_links = parsed["nonCrawlableLinks"].as_u64().unwrap_or(0) as u32;
    a.images_without_modern_format =
        parsed["imagesWithoutModernFormat"].as_u64().unwrap_or(0) as u32;
    a.oversized_images = parsed["oversizedImages"].as_u64().unwrap_or(0) as u32;
    a.gif_images = parsed["gifImages"].as_u64().unwrap_or(0) as u32;
    a.font_display_issues = parsed["fontDisplayIssues"].as_u64().unwrap_or(0) as u32;
    a.font_face_count = parsed["fontFaceCount"].as_u64().unwrap_or(0) as u32;
    a.fonts_without_preload_count = parsed["fontsWithoutPreloadCount"].as_u64().unwrap_or(0) as u32;
    a.preload_hints = parsed["preloadHints"].as_u64().unwrap_or(0) as u32;
    a.prefetch_hints = parsed["prefetchHints"].as_u64().unwrap_or(0) as u32;
    a.dns_prefetch_hints = parsed["dnsPrefetchHints"].as_u64().unwrap_or(0) as u32;
    a.orphaned_preload_count = parsed["orphanedPreloadCount"].as_u64().unwrap_or(0) as u32;
    a.document_decoded_bytes = nonzero_u64(&parsed["documentDecodedBytes"]);
    a.document_transfer_bytes = nonzero_u64(&parsed["documentTransferBytes"]);
    a.resource_cache_probe_urls = parsed["cacheProbeUrls"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    a.lcp_image_without_preload = parsed["lcpImageWithoutPreload"].as_bool().unwrap_or(false);
    a.lcp_image_without_fetchpriority = parsed["lcpImageWithoutFetchpriority"]
        .as_bool()
        .unwrap_or(false);
    a.lcp_image_lazy_loaded = parsed["lcpImageLazyLoaded"].as_bool().unwrap_or(false);
    a.lcp_image_url = parsed["lcpImageUrl"].as_str().map(String::from);
    a.deprecated_api_count = parsed["deprecatedApiCount"].as_u64().unwrap_or(0) as u32;
    a.hreflang_count = parsed["hreflangCount"].as_u64().unwrap_or(0) as u32;
    a.hreflang_invalid_count = parsed["hreflangInvalidCount"].as_u64().unwrap_or(0) as u32;
    a.jsonld_count = parsed["jsonldCount"].as_u64().unwrap_or(0) as u32;
    a.jsonld_invalid_count = parsed["jsonldInvalidCount"].as_u64().unwrap_or(0) as u32;
    a.sync_head_scripts = parsed["syncHeadScripts"].as_u64().unwrap_or(0) as u32;
    a.external_scripts_without_sri =
        parsed["externalScriptsWithoutSri"].as_u64().unwrap_or(0) as u32;
    a.external_styles_without_sri = parsed["externalStylesWithoutSri"].as_u64().unwrap_or(0) as u32;
    a.broken_fragment_links = parsed["brokenFragmentLinks"].as_u64().unwrap_or(0) as u32;
    a.broken_fragment_samples = parsed["brokenFragmentSamples"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    a.generic_link_text_count = parsed["genericLinkTextCount"].as_u64().unwrap_or(0) as u32;
    a.generic_link_text_samples = parsed["genericLinkTextSamples"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    a.html_issues = build_html_issues(a, &parsed);

    true
}

pub(super) fn build_html_issues(
    a: &PageHealthAnalysis,
    parsed: &serde_json::Value,
) -> Vec<HtmlValidationIssue> {
    let mut html_issues = Vec::new();

    if a.duplicate_id_count > 0 {
        let samples = parsed["duplicateIdSamples"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        html_issues.push(HtmlValidationIssue::new(
            HtmlValidationKind::DuplicateIds,
            a.duplicate_id_count,
            "medium",
            samples,
        ));
    }

    if a.images_without_alt > 0 {
        html_issues.push(HtmlValidationIssue::new(
            HtmlValidationKind::ImagesWithoutAlt,
            a.images_without_alt,
            "high",
            Vec::new(),
        ));
    }
    if a.tables_without_headers > 0 {
        html_issues.push(HtmlValidationIssue::new(
            HtmlValidationKind::TablesWithoutHeaders,
            a.tables_without_headers,
            "medium",
            Vec::new(),
        ));
    }
    if a.empty_headings > 0 {
        html_issues.push(HtmlValidationIssue::new(
            HtmlValidationKind::EmptyHeadings,
            a.empty_headings,
            "medium",
            Vec::new(),
        ));
    }
    if a.nested_interactive_count > 0 {
        html_issues.push(HtmlValidationIssue::new(
            HtmlValidationKind::NestedInteractive,
            a.nested_interactive_count,
            "high",
            Vec::new(),
        ));
    }

    html_issues
}

fn nonzero_u64(value: &serde_json::Value) -> Option<u64> {
    value.as_u64().filter(|v| *v > 0)
}

/// Counter-cases of [`DOM_INSPECTION`] (#777). Without its result the whole
/// page-health block is withheld (`analyze_page_health` returns `None`).
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_contract::<JsObject>(serde_json::json!({}));
}
