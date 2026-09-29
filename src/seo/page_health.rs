//! Page health analysis
//!
//! HTTP probes and DOM inspections that don't belong in the technical SEO
//! module: soft-404 detection, meta-refresh, frames, URL structure,
//! redirect detection, www/non-www consolidation, and basic HTML validation.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::Result;

mod dom;
mod issues;
mod probes;

use dom::{run_dom_inspection, run_local_html_validation};
pub use issues::collect_issues;
use probes::run_http_probes;

/// Complete page health analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageHealthAnalysis {
    /// HTTP status returned for a probe URL (soft-404 detection)
    pub soft_404_status: Option<u16>,
    /// True when the server returns 200 for non-existent URLs
    pub is_soft_404: bool,
    /// Detailed custom 404 probe result for a known non-existent URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_404: Option<Custom404Check>,

    /// Page uses `<meta http-equiv="refresh">`
    pub has_meta_refresh: bool,
    /// Content attribute of the meta-refresh tag
    pub meta_refresh_content: Option<String>,

    /// Number of `<frame>` / `<frameset>` elements (deprecated HTML4)
    pub frame_count: u32,
    /// Number of `<iframe>` elements
    pub iframe_count: u32,
    /// Number of `<iframe>` elements pointing to a different host
    pub cross_origin_iframe_count: u32,

    /// Length of the page URL in characters
    pub url_length: usize,
    /// True when URL contains query parameters
    pub url_has_query_params: bool,
    /// True when URL has query parameters (dynamic URL)
    pub url_is_dynamic: bool,
    /// Number of non-empty path segments
    pub url_path_depth: usize,
    /// True when URL length > 115 characters
    pub url_is_too_long: bool,
    /// True when path depth > 5
    pub url_is_too_deep: bool,

    /// True when the browser navigated to a different URL than requested
    pub own_redirect_detected: bool,
    /// Final URL after navigation (when different from requested URL)
    pub own_final_url: Option<String>,
    /// Number of HTTP redirect hops before the final response
    pub redirect_count: u32,
    /// Full redirect chain (status + URL per hop, up to 10)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redirect_chain: Vec<RedirectHop>,

    /// www ↔ non-www redirect configuration
    pub www_consolidation: Option<WwwConsolidation>,
    /// Trailing-slash and http→https redirect consistency (#542)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_canonicalization: Option<UrlCanonicalizationCheck>,

    /// Duplicate ID count across the DOM
    pub duplicate_id_count: u32,
    /// `<img>` elements missing the alt attribute
    pub images_without_alt: u32,
    /// `<table>` elements without `<th>` or `<caption>`
    pub tables_without_headers: u32,
    /// Empty heading elements (h1–h6)
    pub empty_headings: u32,
    /// Nested interactive elements (button inside button, a inside a)
    pub nested_interactive_count: u32,
    /// Structured list of HTML validation findings
    pub html_issues: Vec<HtmlValidationIssue>,
    /// Status of the local HTML5 validation: "executed", "skipped", or "failed"
    pub html_validator_status: String,
    /// Additional detail about validator execution or skip reason. Canonical
    /// English; for `"executed"` the PDF re-derives it via
    /// [`html_validator_executed_text`] (#406).
    #[serde(default, deserialize_with = "deserialize_html_validator_detail")]
    pub html_validator_detail: Option<String>,

    /// True when a valid HTML5 `<!DOCTYPE html>` declaration is present
    pub has_doctype: bool,
    /// Number of inline `<script>` elements that call `document.write()`
    pub document_write_count: u32,
    /// Total DOM element count (`document.querySelectorAll('*').length`)
    pub dom_node_count: u32,
    /// Maximum nesting depth of the DOM tree
    pub dom_max_depth: u32,
    /// `<img>` elements without explicit `width` + `height` attributes (CLS risk)
    pub images_without_dimensions: u32,
    /// `<input type="password">` fields with an inline `onpaste` handler that blocks paste
    pub paste_blocking_password_fields: u32,
    /// `<img>` elements below the initial viewport without `loading="lazy"`
    pub offscreen_images_without_lazy: u32,
    /// `<img>` elements outside `<picture>` without a `srcset` attribute (no responsive variants)
    pub images_without_srcset: u32,
    /// Third-party origins without a matching `<link rel="preconnect">` hint
    pub missing_preconnect_count: u32,
    /// Sample of origins missing preconnect (up to 5)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_preconnect_origins: Vec<String>,

    /// `<a>` elements with non-crawlable hrefs (javascript:, empty, missing)
    pub non_crawlable_links: u32,
    /// `<img>` elements served as JPEG/PNG without a WebP/AVIF alternative
    pub images_without_modern_format: u32,
    /// `<img>` elements whose natural size significantly exceeds their display size
    pub oversized_images: u32,
    /// `<img src="*.gif">` elements (potential animated GIFs to convert to video)
    pub gif_images: u32,
    /// Count of `@font-face` rules with missing or blocking `font-display`
    pub font_display_issues: u32,
    /// Number of distinct web fonts loaded via `@font-face` (#533)
    pub font_face_count: u32,
    /// Web fonts with no matching `<link rel="preload" as="font">` (#533)
    pub fonts_without_preload_count: u32,
    /// Number of `<link rel="preload">` hints
    pub preload_hints: u32,
    /// Number of `<link rel="prefetch">` hints
    pub prefetch_hints: u32,
    /// Number of `<link rel="dns-prefetch">` hints
    pub dns_prefetch_hints: u32,
    /// Preload hints that don't match any loaded resource (orphaned)
    pub orphaned_preload_count: u32,
    /// True when the main page is served over HTTP/2 or HTTP/3
    pub uses_http2: bool,
    /// True when the main page response is compressed (gzip/br/zstd)
    pub has_compression: bool,
    /// Main document decoded body size from Navigation Timing, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_decoded_bytes: Option<u64>,
    /// Main document transfer size from Navigation Timing, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_transfer_bytes: Option<u64>,
    /// Raw Cache-Control header value from the main page response
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<String>,
    /// charset declared in the HTTP Content-Type header, lowercased (#543)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charset_header: Option<String>,
    /// charset declared via `<meta charset>`, as extracted by the SEO meta
    /// module — copied in by `analyze_seo` so this struct alone can compare
    /// the two without a cross-module dependency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_charset: Option<String>,
    /// True when Cache-Control includes a positive max-age or s-maxage
    pub has_efficient_cache: bool,
    /// Static-resource cache policy audit.
    #[serde(default)]
    pub resource_cache: ResourceCacheAudit,
    /// Resource URLs collected from browser timing for cache probing.
    #[serde(skip)]
    pub resource_cache_probe_urls: Vec<String>,
    /// Number of Server-Timing header entries on the main page response
    pub server_timing_count: u32,
    /// hreflang link elements present (count of rel="alternate" hreflang)
    pub hreflang_count: u32,
    /// hreflang entries with invalid language codes
    pub hreflang_invalid_count: u32,
    /// JSON-LD blocks found on the page
    pub jsonld_count: u32,
    /// JSON-LD blocks that are missing @context or @type (invalid)
    pub jsonld_invalid_count: u32,
    /// LCP image candidate (largest visible img) lacks a preload hint
    pub lcp_image_without_preload: bool,
    /// LCP image candidate is missing fetchpriority="high"
    pub lcp_image_without_fetchpriority: bool,
    /// LCP image candidate has loading="lazy" (incorrect — delays LCP)
    pub lcp_image_lazy_loaded: bool,
    /// URL of the heuristic LCP image candidate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lcp_image_url: Option<String>,
    /// Number of deprecated browser API patterns detected in inline scripts
    pub deprecated_api_count: u32,

    /// Synchronous `<script src>` in `<head>` without defer/async/type=module (render-blocking)
    pub sync_head_scripts: u32,
    /// External `<script src>` without Subresource Integrity `integrity` attribute
    pub external_scripts_without_sri: u32,
    /// External `<link rel="stylesheet">` without Subresource Integrity `integrity` attribute
    pub external_styles_without_sri: u32,
    /// `<a href="#fragment">` links where the target ID does not exist on this page
    pub broken_fragment_links: u32,
    /// Sample of broken fragment hrefs (up to 5)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub broken_fragment_samples: Vec<String>,
    /// Links with generic, non-descriptive text ("hier", "mehr", "click here", etc.)
    pub generic_link_text_count: u32,
    /// Sample hrefs of generic-text links (up to 5)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_link_text_samples: Vec<String>,

    /// Aggregated issue list for report rendering
    pub issues: Vec<PageHealthIssue>,
}

/// A single hop in the HTTP redirect chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectHop {
    pub status: u16,
    pub url: String,
}

/// Custom 404 probe result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Custom404Check {
    pub probe_url: String,
    pub status: u16,
    pub proper_status: bool,
    pub custom_page: bool,
}

/// Cache-policy and Content-Type-header summary for static subresources.
/// Both audits share one probe pass over the same already-collected asset
/// URLs (#544) — no extra requests beyond the existing cache audit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceCacheAudit {
    pub checked_resources: u32,
    pub cacheable_resources: u32,
    pub inefficient_resources: u32,
    pub immutable_resources: u32,
    pub etag_resources: u32,
    pub expires_resources: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub samples: Vec<ResourceCacheFinding>,
    /// Static resources (by recognized extension) with no Content-Type header at all.
    #[serde(default)]
    pub missing_content_type: u32,
    /// Static resources whose Content-Type header doesn't match their file extension
    /// (e.g. a `.js` file served as `text/html`).
    #[serde(default)]
    pub incorrect_content_type: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_type_samples: Vec<ContentTypeFinding>,
}

/// Single inefficient static-resource cache finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCacheFinding {
    pub url: String,
    pub cache_control: Option<String>,
    pub has_etag: bool,
    pub has_expires: bool,
    pub reason: String,
}

/// Single missing/incorrect Content-Type header finding (#544).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTypeFinding {
    pub url: String,
    /// Substring expected in a correct Content-Type for this extension (e.g. "javascript")
    pub expected: String,
    /// Actual Content-Type header value, if any
    pub actual: Option<String>,
}

/// www ↔ non-www redirect configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WwwConsolidation {
    /// HTTP status of the www variant
    pub www_status: Option<u16>,
    /// HTTP status of the non-www variant
    pub non_www_status: Option<u16>,
    /// www redirects to non-www
    pub www_redirects_to_non_www: bool,
    /// non-www redirects to www
    pub non_www_redirects_to_www: bool,
    /// Canonical variant: "www", "non-www", or "inconsistent"
    pub canonical_variant: String,
    /// True when one variant properly redirects to the other
    pub is_consolidated: bool,
}

/// Trailing-slash and http→https redirect consistency (#542).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlCanonicalizationCheck {
    /// HTTP status of the URL variant with the trailing slash toggled
    /// (added if absent, removed if present). `None` for the root path,
    /// where a trailing slash is unambiguous and not checked.
    pub trailing_slash_variant_status: Option<u16>,
    /// True when the original URL and its trailing-slash variant both
    /// answer 200 independently instead of one redirecting to the other —
    /// a duplicate-content risk.
    pub trailing_slash_inconsistent: bool,
    /// HTTP status of the plain `http://` version of an `https://` page.
    /// `None` when the audited URL is not `https://`.
    pub http_status: Option<u16>,
    /// True when the audited URL is `https://` and its `http://` version
    /// does not redirect to `https://`.
    pub http_to_https_missing: bool,
}

/// Canonical kind of an HTML validation finding (#406).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HtmlValidationKind {
    DuplicateIds,
    ImagesWithoutAlt,
    TablesWithoutHeaders,
    EmptyHeadings,
    NestedInteractive,
    ParseErrors,
}

/// A single HTML validation finding. `check` and `detail` are canonical
/// English; the PDF re-derives them from `kind`, `count` and `samples` via
/// [`html_validation_check_text`] / [`html_validation_detail_text`] (#406).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "HtmlValidationIssueWire")]
pub struct HtmlValidationIssue {
    pub kind: HtmlValidationKind,
    pub check: String,
    pub count: u32,
    pub severity: String,
    pub detail: String,
    /// Raw values behind `detail`: duplicate-ID samples or the first parse errors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub samples: Vec<String>,
}

impl HtmlValidationIssue {
    pub fn new(kind: HtmlValidationKind, count: u32, severity: &str, samples: Vec<String>) -> Self {
        Self {
            kind,
            check: html_validation_check_text(kind, true).to_string(),
            count,
            severity: severity.to_string(),
            detail: html_validation_detail_text(kind, count, &samples, true),
            samples,
        }
    }
}

/// Label of an HTML validation check — the only source of this text.
pub fn html_validation_check_text(kind: HtmlValidationKind, en: bool) -> &'static str {
    match (kind, en) {
        (HtmlValidationKind::DuplicateIds, true) => "Duplicate IDs",
        (HtmlValidationKind::DuplicateIds, false) => "Doppelte IDs",
        (HtmlValidationKind::ImagesWithoutAlt, true) => "Images without alt attribute",
        (HtmlValidationKind::ImagesWithoutAlt, false) => "Bilder ohne alt-Attribut",
        (HtmlValidationKind::TablesWithoutHeaders, true) => "Tables without header row",
        (HtmlValidationKind::TablesWithoutHeaders, false) => "Tabellen ohne Kopfzeile",
        (HtmlValidationKind::EmptyHeadings, true) => "Empty headings",
        (HtmlValidationKind::EmptyHeadings, false) => "Leere Überschriften",
        (HtmlValidationKind::NestedInteractive, true) => "Nested interactive elements",
        (HtmlValidationKind::NestedInteractive, false) => "Verschachtelte interaktive Elemente",
        (HtmlValidationKind::ParseErrors, true) => "HTML5 parsing errors",
        (HtmlValidationKind::ParseErrors, false) => "HTML5-Parsing-Fehler",
    }
}

/// Detail text of an HTML validation finding — the only source of this text.
pub fn html_validation_detail_text(
    kind: HtmlValidationKind,
    count: u32,
    samples: &[String],
    en: bool,
) -> String {
    match kind {
        HtmlValidationKind::DuplicateIds => match (samples.is_empty(), en) {
            (true, true) => format!("{count} found"),
            (true, false) => format!("{count} gefunden"),
            (false, true) => format!("{count} (e.g. {})", samples.join(", ")),
            (false, false) => format!("{count} (z.B. {})", samples.join(", ")),
        },
        HtmlValidationKind::ImagesWithoutAlt if en => format!("{count} <img> without alt"),
        HtmlValidationKind::ImagesWithoutAlt => format!("{count} <img> ohne alt"),
        HtmlValidationKind::TablesWithoutHeaders if en => {
            format!("{count} <table> without <th> or <caption>")
        }
        HtmlValidationKind::TablesWithoutHeaders => {
            format!("{count} <table> ohne <th> oder <caption>")
        }
        HtmlValidationKind::EmptyHeadings if en => format!("{count} empty h1–h6 elements"),
        HtmlValidationKind::EmptyHeadings => format!("{count} leere h1–h6 Elemente"),
        HtmlValidationKind::NestedInteractive => format!("{count} button/a in button/a"),
        HtmlValidationKind::ParseErrors => {
            let joined = samples.join(" | ");
            let more = (count as usize).saturating_sub(samples.len());
            match (more, en) {
                (0, _) => joined,
                (n, true) => format!("{joined} | +{n} more"),
                (n, false) => format!("{joined} | +{n} weitere"),
            }
        }
    }
}

/// Detail of an executed local HTML validation — the only source of this text.
pub fn html_validator_executed_text(en: bool) -> &'static str {
    if en {
        "HTML5 validation run locally via html5ever"
    } else {
        "HTML5-Validierung lokal via html5ever"
    }
}

/// Serialized shape of [`HtmlValidationIssue`]. Cache entries written before
/// the #406 fix (same `v1.6.0` cache directory) carry German `check`/`detail`
/// and no `kind`; they are mapped back to kind + samples here.
#[derive(Deserialize)]
struct HtmlValidationIssueWire {
    #[serde(default)]
    kind: Option<HtmlValidationKind>,
    check: String,
    count: u32,
    severity: String,
    detail: String,
    #[serde(default)]
    samples: Vec<String>,
}

impl TryFrom<HtmlValidationIssueWire> for HtmlValidationIssue {
    type Error = String;

    fn try_from(w: HtmlValidationIssueWire) -> std::result::Result<Self, Self::Error> {
        if let Some(kind) = w.kind {
            return Ok(Self {
                kind,
                check: w.check,
                count: w.count,
                severity: w.severity,
                detail: w.detail,
                samples: w.samples,
            });
        }
        let kind = [
            HtmlValidationKind::DuplicateIds,
            HtmlValidationKind::ImagesWithoutAlt,
            HtmlValidationKind::TablesWithoutHeaders,
            HtmlValidationKind::EmptyHeadings,
            HtmlValidationKind::NestedInteractive,
            HtmlValidationKind::ParseErrors,
        ]
        .into_iter()
        .find(|k| html_validation_check_text(*k, false) == w.check)
        .ok_or_else(|| format!("unknown legacy HTML validation check: {}", w.check))?;
        let samples = match kind {
            HtmlValidationKind::DuplicateIds => w
                .detail
                .split_once(" (z.B. ")
                .and_then(|(_, rest)| rest.strip_suffix(')'))
                .map(|list| list.split(", ").map(str::to_string).collect())
                .unwrap_or_default(),
            HtmlValidationKind::ParseErrors => w
                .detail
                .split(" | ")
                .filter(|part| !(part.starts_with('+') && part.ends_with(" weitere")))
                .map(str::to_string)
                .collect(),
            _ => Vec::new(),
        };
        Ok(Self::new(kind, w.count, &w.severity, samples))
    }
}

/// Maps the pre-#406 German "executed" detail of cached reports to English.
fn deserialize_html_validator_detail<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let detail = Option::<String>::deserialize(deserializer)?;
    Ok(detail.map(|d| {
        if d == html_validator_executed_text(false) {
            html_validator_executed_text(true).to_string()
        } else {
            d
        }
    }))
}

/// A resolved page health issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageHealthIssue {
    pub issue_type: String,
    pub message: String,
    pub severity: String,
}

/// Analyse page health: runs DOM inspection, URL analysis, and HTTP probes.
pub async fn analyze_page_health(page: &Page, url: &str) -> Result<PageHealthAnalysis> {
    let mut analysis = PageHealthAnalysis {
        html_validator_status: "skipped".to_string(),
        ..Default::default()
    };

    // URL analysis (pure Rust, no CDP)
    analyze_url(url, &mut analysis);

    // DOM inspection via single JS evaluate
    if let Err(e) = run_dom_inspection(page, url, &mut analysis).await {
        warn!("Page health DOM inspection failed: {}", e);
    }

    // HTTP probes (reqwest, concurrent)
    run_http_probes(url, &mut analysis).await;

    // Local HTML5 validation via html5ever (best effort)
    if let Err(e) = run_local_html_validation(page, url, &mut analysis).await {
        analysis.html_validator_status = "failed".to_string();
        analysis.html_validator_detail = Some(e.to_string());
        warn!("Local HTML validation failed: {}", e);
    }

    // Aggregate issues — the stored report (and thus JSON) is always canonical
    // English; the PDF re-derives localized issues at presentation time (#406).
    analysis.issues = collect_issues(&analysis, true);

    Ok(analysis)
}

// ─── URL analysis ────────────────────────────────────────────────────────────

fn analyze_url(url: &str, a: &mut PageHealthAnalysis) {
    a.url_length = url.len();
    a.url_is_too_long = url.len() > 115;

    if let Ok(parsed) = url::Url::parse(url) {
        a.url_has_query_params = parsed.query().is_some();
        a.url_is_dynamic = a.url_has_query_params;
        let depth = parsed
            .path_segments()
            .map(|segs| segs.filter(|s| !s.is_empty()).count())
            .unwrap_or(0);
        a.url_path_depth = depth;
        a.url_is_too_deep = depth > 5;
    }
}

#[cfg(test)]
mod tests {
    use super::dom::{build_html_issues, validate_html_locally};
    use super::probes::{
        document_timing_indicates_compression, expected_content_type_substring,
        is_cache_policy_efficient, is_static_cache_candidate, looks_like_custom_404_page,
    };
    use super::*;
    use serde_json::json;

    #[test]
    fn test_analyze_url_basic() {
        let mut a = PageHealthAnalysis::default();
        analyze_url("https://example.com/foo/bar?q=1", &mut a);
        assert_eq!(a.url_length, 31);
        assert!(a.url_has_query_params);
        assert!(a.url_is_dynamic);
        assert_eq!(a.url_path_depth, 2);
        assert!(!a.url_is_too_long);
        assert!(!a.url_is_too_deep);
    }

    #[test]
    fn test_analyze_url_long() {
        let long_url = format!("https://example.com/{}", "a".repeat(100));
        let mut a = PageHealthAnalysis::default();
        analyze_url(&long_url, &mut a);
        assert!(a.url_is_too_long);
    }

    #[test]
    fn test_analyze_url_deep() {
        let mut a = PageHealthAnalysis::default();
        analyze_url("https://example.com/a/b/c/d/e/f", &mut a);
        assert_eq!(a.url_path_depth, 6);
        assert!(a.url_is_too_deep);
    }

    #[test]
    fn test_collect_issues_soft_404() {
        let a = PageHealthAnalysis {
            is_soft_404: true,
            soft_404_status: Some(200),
            custom_404: Some(Custom404Check {
                probe_url: "https://example.com/auditmysite-404-probe-xyz123".to_string(),
                status: 200,
                proper_status: false,
                custom_page: true,
            }),
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(issues.iter().any(|i| i.issue_type == "soft_404"));
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "custom_404_invalid_status"));
    }

    #[test]
    fn test_collect_issues_url_canonicalization() {
        let a = PageHealthAnalysis {
            url_canonicalization: Some(UrlCanonicalizationCheck {
                trailing_slash_variant_status: Some(200),
                trailing_slash_inconsistent: true,
                http_status: Some(200),
                http_to_https_missing: true,
            }),
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "trailing_slash_inconsistent"));
        assert!(issues
            .iter()
            .any(|i| i.issue_type == "http_not_redirected_to_https"));
    }

    #[test]
    fn test_collect_issues_url_canonicalization_consistent_is_silent() {
        let a = PageHealthAnalysis {
            url_canonicalization: Some(UrlCanonicalizationCheck {
                trailing_slash_variant_status: Some(301),
                trailing_slash_inconsistent: false,
                http_status: Some(301),
                http_to_https_missing: false,
            }),
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(!issues
            .iter()
            .any(|i| i.issue_type == "trailing_slash_inconsistent"));
        assert!(!issues
            .iter()
            .any(|i| i.issue_type == "http_not_redirected_to_https"));
    }

    #[test]
    fn test_collect_issues_charset_mismatch() {
        let a = PageHealthAnalysis {
            charset_header: Some("iso-8859-1".to_string()),
            declared_charset: Some("UTF-8".to_string()),
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(issues.iter().any(|i| i.issue_type == "charset_mismatch"));
    }

    #[test]
    fn test_collect_issues_charset_match_is_case_insensitive_and_silent() {
        let a = PageHealthAnalysis {
            charset_header: Some("utf-8".to_string()),
            declared_charset: Some("UTF-8".to_string()),
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(!issues.iter().any(|i| i.issue_type == "charset_mismatch"));
    }

    #[test]
    fn test_collect_issues_charset_missing_one_side_is_silent() {
        let a = PageHealthAnalysis {
            charset_header: Some("utf-8".to_string()),
            declared_charset: None,
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(!issues.iter().any(|i| i.issue_type == "charset_mismatch"));
    }

    #[test]
    fn test_collect_issues_fonts_not_preloaded() {
        let a = PageHealthAnalysis {
            font_face_count: 2,
            fonts_without_preload_count: 1,
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(issues.iter().any(|i| i.issue_type == "fonts_not_preloaded"));
    }

    #[test]
    fn test_collect_issues_all_fonts_preloaded_is_silent() {
        let a = PageHealthAnalysis {
            font_face_count: 2,
            fonts_without_preload_count: 0,
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(!issues.iter().any(|i| i.issue_type == "fonts_not_preloaded"));
    }

    #[test]
    fn collect_issues_english_messages_have_no_german_characters() {
        let a = PageHealthAnalysis {
            has_doctype: false,
            dom_node_count: 1600,
            dom_max_depth: 20,
            document_write_count: 2,
            images_without_dimensions: 4,
            paste_blocking_password_fields: 1,
            offscreen_images_without_lazy: 6,
            images_without_srcset: 7,
            missing_preconnect_count: 2,
            missing_preconnect_origins: vec![
                "https://cdn.example.com".to_string(),
                "https://fonts.example.com".to_string(),
            ],
            non_crawlable_links: 6,
            images_without_modern_format: 6,
            oversized_images: 3,
            gif_images: 2,
            font_display_issues: 3,
            font_face_count: 2,
            fonts_without_preload_count: 1,
            orphaned_preload_count: 1,
            lcp_image_lazy_loaded: true,
            lcp_image_without_preload: true,
            lcp_image_url: Some("https://example.com/hero.jpg".to_string()),
            deprecated_api_count: 2,
            uses_http2: false,
            has_compression: false,
            has_efficient_cache: false,
            cache_control: Some("max-age=60".to_string()),
            resource_cache: ResourceCacheAudit {
                checked_resources: 3,
                cacheable_resources: 3,
                inefficient_resources: 2,
                samples: vec![ResourceCacheFinding {
                    url: "https://example.com/app.js".to_string(),
                    cache_control: Some("max-age=60".to_string()),
                    has_etag: true,
                    has_expires: false,
                    reason: "short or missing max-age/s-maxage".to_string(),
                }],
                ..Default::default()
            },
            hreflang_invalid_count: 1,
            jsonld_invalid_count: 1,
            sync_head_scripts: 2,
            external_scripts_without_sri: 1,
            external_styles_without_sri: 1,
            broken_fragment_links: 2,
            broken_fragment_samples: vec!["#missing".to_string()],
            generic_link_text_count: 3,
            is_soft_404: true,
            soft_404_status: Some(200),
            has_meta_refresh: true,
            meta_refresh_content: Some("0; url=https://example.com".to_string()),
            frame_count: 1,
            url_is_too_long: true,
            url_length: 200,
            url_is_too_deep: true,
            url_path_depth: 8,
            url_has_query_params: true,
            redirect_count: 3,
            www_consolidation: Some(WwwConsolidation {
                www_status: None,
                non_www_status: None,
                www_redirects_to_non_www: false,
                non_www_redirects_to_www: false,
                canonical_variant: "inconsistent".to_string(),
                is_consolidated: false,
            }),
            url_canonicalization: Some(UrlCanonicalizationCheck {
                trailing_slash_variant_status: Some(200),
                trailing_slash_inconsistent: true,
                http_status: Some(200),
                http_to_https_missing: true,
            }),
            charset_header: Some("iso-8859-1".to_string()),
            declared_charset: Some("UTF-8".to_string()),
            custom_404: Some(Custom404Check {
                probe_url: "https://example.com/auditmysite-404-probe-xyz123".to_string(),
                status: 200,
                proper_status: false,
                custom_page: false,
            }),
            ..Default::default()
        };

        let issues = collect_issues(&a, true);
        assert!(
            issues.len() >= 10,
            "expected many issues, got {}",
            issues.len()
        );
        for issue in &issues {
            assert!(
                !issue.message.chars().any(|c| "äöüÄÖÜß".contains(c)),
                "English message contains German characters: {}",
                issue.message
            );
        }
    }

    #[test]
    fn test_collect_issues_meta_refresh() {
        let a = PageHealthAnalysis {
            has_meta_refresh: true,
            meta_refresh_content: Some("0; url=https://example.com".to_string()),
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(issues.iter().any(|i| i.issue_type == "meta_refresh"));
        assert_eq!(
            issues
                .iter()
                .find(|i| i.issue_type == "meta_refresh")
                .map(|i| i.severity.as_str()),
            Some("high")
        );
    }

    #[test]
    fn compression_inferred_from_navigation_timing_ratio() {
        let a = PageHealthAnalysis {
            document_transfer_bytes: Some(21_814),
            document_decoded_bytes: Some(1_233_280),
            ..Default::default()
        };
        assert!(document_timing_indicates_compression(&a));
    }

    #[test]
    fn missing_compression_issue_suppressed_when_timing_shows_compression() {
        let a = PageHealthAnalysis {
            has_compression: true,
            document_transfer_bytes: Some(21_814),
            document_decoded_bytes: Some(1_233_280),
            ..Default::default()
        };
        let issues = collect_issues(&a, false);
        assert!(!issues.iter().any(|i| i.issue_type == "missing_compression"));
    }

    #[test]
    fn cache_policy_requires_meaningful_lifetime_or_immutable() {
        assert!(is_cache_policy_efficient(
            "public, max-age=31536000, immutable"
        ));
        assert!(is_cache_policy_efficient("public, s-maxage=86400"));
        assert!(!is_cache_policy_efficient("public, max-age=60"));
        assert!(!is_cache_policy_efficient("no-cache, max-age=31536000"));
    }

    #[test]
    fn static_cache_candidate_detects_asset_extensions() {
        assert!(is_static_cache_candidate("https://example.com/app.css"));
        assert!(is_static_cache_candidate(
            "https://example.com/fonts/inter.woff2?v=1"
        ));
        assert!(!is_static_cache_candidate("https://example.com/page/"));
    }

    #[test]
    fn custom_404_heuristic_rejects_generic_short_pages() {
        assert!(!looks_like_custom_404_page(
            "<html><title>404 Not Found</title><body>nginx 404 not found</body></html>"
        ));
        assert!(looks_like_custom_404_page(&format!(
            "<html><body><nav>Home</nav><main>{}</main></body></html>",
            "Helpful not-found guidance. ".repeat(30)
        )));
    }

    #[test]
    fn collect_issues_reports_resource_cache_findings() {
        let a = PageHealthAnalysis {
            resource_cache: ResourceCacheAudit {
                checked_resources: 3,
                cacheable_resources: 3,
                inefficient_resources: 2,
                samples: vec![ResourceCacheFinding {
                    url: "https://example.com/app.js".to_string(),
                    cache_control: Some("max-age=60".to_string()),
                    has_etag: true,
                    has_expires: false,
                    reason: "short or missing max-age/s-maxage".to_string(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        let issues = collect_issues(&a, false);

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "inefficient_resource_cache"));
    }

    #[test]
    fn collect_issues_reports_incorrect_content_type() {
        let a = PageHealthAnalysis {
            resource_cache: ResourceCacheAudit {
                missing_content_type: 1,
                incorrect_content_type: 1,
                content_type_samples: vec![ContentTypeFinding {
                    url: "https://example.com/app.js".to_string(),
                    expected: "javascript".to_string(),
                    actual: Some("text/html".to_string()),
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        let issues = collect_issues(&a, false);

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "incorrect_content_type"));
    }

    #[test]
    fn expected_content_type_substring_recognizes_common_extensions() {
        assert_eq!(
            expected_content_type_substring("https://example.com/app.js"),
            Some("javascript")
        );
        assert_eq!(
            expected_content_type_substring("https://example.com/style.css"),
            Some("css")
        );
        assert_eq!(
            expected_content_type_substring("https://example.com/page.html"),
            None
        );
    }

    #[test]
    fn collect_issues_reports_generic_custom_404() {
        let a = PageHealthAnalysis {
            custom_404: Some(Custom404Check {
                probe_url: "https://example.com/auditmysite-404-probe-xyz123".to_string(),
                status: 404,
                proper_status: true,
                custom_page: false,
            }),
            ..Default::default()
        };

        let issues = collect_issues(&a, false);

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "generic_404_page"));
    }

    #[test]
    fn test_build_html_issues_for_classic_validation_findings() {
        let analysis = PageHealthAnalysis {
            duplicate_id_count: 2,
            images_without_alt: 3,
            tables_without_headers: 1,
            empty_headings: 2,
            nested_interactive_count: 1,
            ..Default::default()
        };
        let parsed = json!({
            "duplicateIdSamples": ["hero", "cta-button"]
        });

        let issues = build_html_issues(&analysis, &parsed);

        assert!(issues.iter().any(|i| {
            i.kind == HtmlValidationKind::DuplicateIds
                && i.count == 2
                && i.samples == ["hero", "cta-button"]
                && i.detail == "2 (e.g. hero, cta-button)"
        }));
        assert!(issues
            .iter()
            .any(|i| i.kind == HtmlValidationKind::ImagesWithoutAlt && i.count == 3));
        assert!(issues
            .iter()
            .any(|i| i.kind == HtmlValidationKind::TablesWithoutHeaders && i.count == 1));
        assert!(issues
            .iter()
            .any(|i| i.kind == HtmlValidationKind::EmptyHeadings && i.count == 2));
        assert!(issues
            .iter()
            .any(|i| i.kind == HtmlValidationKind::NestedInteractive && i.count == 1));
    }

    const ALL_HTML_VALIDATION_KINDS: [HtmlValidationKind; 6] = [
        HtmlValidationKind::DuplicateIds,
        HtmlValidationKind::ImagesWithoutAlt,
        HtmlValidationKind::TablesWithoutHeaders,
        HtmlValidationKind::EmptyHeadings,
        HtmlValidationKind::NestedInteractive,
        HtmlValidationKind::ParseErrors,
    ];

    #[test]
    fn english_html_validation_text_has_no_german_chars() {
        let samples = vec!["a".to_string(), "b".to_string()];
        let mut texts = vec![html_validator_executed_text(true).to_string()];
        for kind in ALL_HTML_VALIDATION_KINDS {
            texts.push(html_validation_check_text(kind, true).to_string());
            texts.push(html_validation_detail_text(kind, 5, &samples, true));
            texts.push(html_validation_detail_text(kind, 5, &[], true));
        }
        for text in texts {
            assert!(
                !text.contains(['ä', 'ö', 'ü', 'Ä', 'Ö', 'Ü', 'ß']),
                "German char in EN text: {text}"
            );
            for word in [
                "gefunden",
                "z.B.",
                "weitere",
                "ohne",
                "Fehler",
                "Validierung",
            ] {
                assert!(!text.contains(word), "German word in EN text: {text}");
            }
        }
    }

    #[test]
    fn legacy_german_html_issues_deserialize_to_canonical_english() {
        let legacy = json!([
            {"check": "Doppelte IDs", "count": 2, "severity": "medium",
             "detail": "2 (z.B. hero-swiper, webshop-widget)"},
            {"check": "Doppelte IDs", "count": 1, "severity": "medium", "detail": "1 gefunden"},
            {"check": "HTML5-Parsing-Fehler", "count": 40, "severity": "high",
             "detail": "No <p> tag to close | Unexpected token | No <p> tag to close | +37 weitere"},
            {"check": "Leere Überschriften", "count": 10, "severity": "medium",
             "detail": "10 leere h1–h6 Elemente"}
        ]);
        let issues: Vec<HtmlValidationIssue> = serde_json::from_value(legacy.clone()).unwrap();

        assert_eq!(issues[0].kind, HtmlValidationKind::DuplicateIds);
        assert_eq!(issues[0].check, "Duplicate IDs");
        assert_eq!(issues[0].detail, "2 (e.g. hero-swiper, webshop-widget)");
        assert_eq!(issues[1].detail, "1 found");
        assert_eq!(issues[2].kind, HtmlValidationKind::ParseErrors);
        assert_eq!(issues[2].samples.len(), 3);
        assert_eq!(issues[3].check, "Empty headings");
        // The German presentation reproduces the legacy text exactly.
        for (issue, old) in issues.iter().zip(legacy.as_array().unwrap()) {
            assert_eq!(html_validation_check_text(issue.kind, false), old["check"]);
            assert_eq!(
                html_validation_detail_text(issue.kind, issue.count, &issue.samples, false),
                old["detail"]
            );
        }
        // Round trip through the new shape is lossless.
        let again: Vec<HtmlValidationIssue> =
            serde_json::from_value(serde_json::to_value(&issues).unwrap()).unwrap();
        assert_eq!(again[2].samples, issues[2].samples);
        assert_eq!(again[0].detail, issues[0].detail);
    }

    #[test]
    fn legacy_german_validator_detail_deserializes_to_english() {
        let mut value = serde_json::to_value(PageHealthAnalysis::default()).unwrap();
        value["html_validator_detail"] = json!("HTML5-Validierung lokal via html5ever");
        let analysis: PageHealthAnalysis = serde_json::from_value(value).unwrap();
        assert_eq!(
            analysis.html_validator_detail.as_deref(),
            Some(html_validator_executed_text(true))
        );
    }

    #[test]
    fn html5ever_parse_errors_are_structured_findings() {
        let issues = validate_html_locally("<!doctype html><html><head><p></head></html>");

        assert!(issues.iter().any(|issue| {
            issue.kind == HtmlValidationKind::ParseErrors
                && issue.count > 0
                && issue.severity == "high"
                && !issue.detail.is_empty()
        }));
    }
}
