//! Page health analysis
//!
//! HTTP probes and DOM inspections that don't belong in the technical SEO
//! module: soft-404 detection, meta-refresh, frames, URL structure,
//! redirect detection, www/non-www consolidation, and basic HTML validation.

use chromiumoxide::Page;
use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::RcDom;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::{AuditError, Result};

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

    /// Page uses <meta http-equiv="refresh">
    pub has_meta_refresh: bool,
    /// Content attribute of the meta-refresh tag
    pub meta_refresh_content: Option<String>,

    /// Number of <frame> / <frameset> elements (deprecated HTML4)
    pub frame_count: u32,
    /// Number of <iframe> elements
    pub iframe_count: u32,
    /// Number of <iframe> elements pointing to a different host
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

    /// Duplicate ID count across the DOM
    pub duplicate_id_count: u32,
    /// <img> elements missing the alt attribute
    pub images_without_alt: u32,
    /// <table> elements without <th> or <caption>
    pub tables_without_headers: u32,
    /// Empty heading elements (h1–h6)
    pub empty_headings: u32,
    /// Nested interactive elements (button inside button, a inside a)
    pub nested_interactive_count: u32,
    /// Structured list of HTML validation findings
    pub html_issues: Vec<HtmlValidationIssue>,
    /// Status of W3C / Nu HTML validation: "executed", "skipped", or "failed"
    pub html_validator_status: String,
    /// Additional detail about validator execution or skip reason
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

/// Cache-policy summary for static subresources.
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

/// A single HTML validation finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlValidationIssue {
    pub check: String,
    pub count: u32,
    pub severity: String,
    pub detail: String,
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

    // W3C / Nu HTML validation (best effort)
    if let Err(e) = run_w3c_html_validation(page, url, &mut analysis).await {
        analysis.html_validator_status = "failed".to_string();
        analysis.html_validator_detail = Some(e.to_string());
        warn!("W3C HTML validation failed: {}", e);
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

// ─── DOM inspection ──────────────────────────────────────────────────────────

async fn run_dom_inspection(page: &Page, url: &str, a: &mut PageHealthAnalysis) -> Result<()> {
    let js = r#"
    (() => {
        const r = {};
        const host = window.location.host;

        // Final URL (redirect detection)
        r.finalUrl = window.location.href;

        // Meta-refresh
        const mr = document.querySelector('meta[http-equiv="refresh"]');
        r.hasMetaRefresh = !!mr;
        r.metaRefreshContent = mr ? mr.getAttribute('content') : null;

        // Frames
        r.frameCount = document.querySelectorAll('frame, frameset').length;
        r.iframeCount = document.querySelectorAll('iframe').length;
        r.crossOriginIframeCount = Array.from(document.querySelectorAll('iframe[src]'))
            .filter(f => {
                try { return new URL(f.src).host !== host; } catch(e) { return false; }
            }).length;

        // Duplicate IDs
        const allIds = Array.from(document.querySelectorAll('[id]')).map(el => el.id);
        const idCounts = {};
        allIds.forEach(id => { idCounts[id] = (idCounts[id] || 0) + 1; });
        const dupIds = Object.entries(idCounts).filter(([_, c]) => c > 1);
        r.duplicateIdCount = dupIds.length;
        r.duplicateIdSamples = dupIds.slice(0, 5).map(([id]) => id);

        // Images without alt
        r.imagesWithoutAlt = document.querySelectorAll('img:not([alt])').length;

        // Tables without headers
        r.tablesWithoutHeaders = Array.from(document.querySelectorAll('table'))
            .filter(t => !t.querySelector('th') && !t.querySelector('caption')).length;

        // Empty headings
        r.emptyHeadings = document.querySelectorAll(
            'h1:empty,h2:empty,h3:empty,h4:empty,h5:empty,h6:empty'
        ).length;

        // Nested interactive elements
        r.nestedInteractive = document.querySelectorAll('button button, a a').length;

        // Doctype detection
        r.hasDoctype = !!(document.doctype && document.doctype.name === 'html');

        // document.write() usage in inline scripts
        const inlineScripts = Array.from(document.querySelectorAll('script:not([src])'));
        r.documentWriteCount = inlineScripts.filter(s => s.textContent.includes('document.write(')).length;

        // Images without explicit width+height (CLS risk)
        r.imagesWithoutDimensions = Array.from(document.querySelectorAll('img'))
            .filter(img => {
                const style = window.getComputedStyle(img);
                const pos = style.position;
                if (pos === 'absolute' || pos === 'fixed') return false;
                const hasAttrs = img.hasAttribute('width') && img.hasAttribute('height');
                const hasAspectRatio = style.aspectRatio && style.aspectRatio !== 'auto';
                return !hasAttrs && !hasAspectRatio;
            }).length;

        // Paste-blocking password fields (inline handler only)
        r.pasteBlockingPasswords = Array.from(document.querySelectorAll('input[type="password"]'))
            .filter(inp => {
                const attr = inp.getAttribute('onpaste') || '';
                return attr.includes('return false') || attr.includes('preventDefault');
            }).length;

        // DOM size
        r.domNodeCount = document.querySelectorAll('*').length;
        let domMaxDepth = 0;
        const domQueue = [[document.documentElement, 0]];
        while (domQueue.length && domQueue.length < 50000) {
            const [el, d] = domQueue.shift();
            if (d > domMaxDepth) domMaxDepth = d;
            const kids = el && el.children ? Array.from(el.children) : [];
            for (const child of kids) domQueue.push([child, d + 1]);
        }
        r.domMaxDepth = domMaxDepth;

        // Offscreen images without lazy loading
        const vh = window.innerHeight;
        r.offscreenWithoutLazy = Array.from(document.querySelectorAll('img'))
            .filter(img => {
                const rect = img.getBoundingClientRect();
                return rect.top > vh && img.getAttribute('loading') !== 'lazy';
            }).length;

        // Images without srcset (no responsive variants)
        r.imagesWithoutSrcset = Array.from(document.querySelectorAll('img'))
            .filter(img => {
                if (img.closest('picture')) return false;
                const src = img.getAttribute('src') || '';
                if (src.endsWith('.svg') || src.startsWith('data:image/svg')) return false;
                if (img.naturalWidth > 0 && img.naturalWidth < 80) return false;
                return !img.hasAttribute('srcset');
            }).length;

        // Missing preconnect hints for third-party origins
        const preconnected = new Set(
            Array.from(document.querySelectorAll('link[rel="preconnect"]'))
                .map(l => { try { return new URL(l.href).origin; } catch(e) { return null; } })
                .filter(Boolean)
        );
        const extOrigins = new Set();
        document.querySelectorAll('script[src],link[rel="stylesheet"][href],img[src],iframe[src]')
            .forEach(el => {
                const src = el.src || el.href;
                try {
                    const o = new URL(src).origin;
                    if (o !== window.location.origin) extOrigins.add(o);
                } catch(e) {}
            });
        const missingPreconnect = [...extOrigins].filter(o => !preconnected.has(o));
        r.missingPreconnectCount = missingPreconnect.length;
        r.missingPreconnectOrigins = missingPreconnect.slice(0, 5);

        // Non-crawlable links
        r.nonCrawlableLinks = Array.from(document.querySelectorAll('a')).filter(a => {
            const href = a.getAttribute('href');
            if (href === null) return a.hasAttribute('onclick');
            return href === '' || href === '#' || href.startsWith('javascript:');
        }).length;

        // Images without modern format (no WebP/AVIF in picture or srcset type)
        r.imagesWithoutModernFormat = Array.from(document.querySelectorAll('img')).filter(img => {
            const src = img.getAttribute('src') || '';
            if (!src.match(/\.(jpe?g|png)(\?|$)/i)) return false;
            const picture = img.closest('picture');
            if (picture && picture.querySelector('source[type="image/webp"], source[type="image/avif"]')) return false;
            const srcset = img.getAttribute('srcset') || '';
            if (srcset.match(/\.(webp|avif)/i)) return false;
            return true;
        }).length;

        // Oversized images (natural > 4x display area)
        r.oversizedImages = Array.from(document.querySelectorAll('img')).filter(img => {
            if (!img.naturalWidth || !img.clientWidth) return false;
            const naturalPixels = img.naturalWidth * img.naturalHeight;
            const displayPixels = img.clientWidth * img.clientHeight;
            return displayPixels > 0 && naturalPixels > displayPixels * 4;
        }).length;

        // GIF images
        r.gifImages = Array.from(document.querySelectorAll('img[src]'))
            .filter(img => (img.getAttribute('src') || '').match(/\.gif(\?|$)/i)).length;

        // Font-display issues in @font-face rules
        let fontDisplayIssues = 0;
        for (const sheet of document.styleSheets) {
            try {
                for (const rule of sheet.cssRules) {
                    if (rule.type === CSSRule.FONT_FACE_RULE) {
                        const display = rule.style.getPropertyValue('font-display');
                        if (!display || display === 'block' || display === 'auto') fontDisplayIssues++;
                    }
                }
            } catch(e) {}
        }
        r.fontDisplayIssues = fontDisplayIssues;

        // Resource hints inventory
        let preloadCount = 0, prefetchCount = 0, dnsPrefetchCount = 0;
        const preloadHrefs = new Set();
        document.querySelectorAll('link[rel]').forEach(link => {
            const rel = link.getAttribute('rel');
            if (rel === 'preload') { preloadCount++; preloadHrefs.add(link.href); }
            else if (rel === 'prefetch') prefetchCount++;
            else if (rel === 'dns-prefetch') dnsPrefetchCount++;
        });
        r.preloadHints = preloadCount;
        r.prefetchHints = prefetchCount;
        r.dnsPrefetchHints = dnsPrefetchCount;
        // Orphaned preloads: preloaded but not in Resource Timing
        const loadedResources = new Set(performance.getEntriesByType('resource').map(e => e.name));
        r.orphanedPreloadCount = [...preloadHrefs].filter(href => href && !loadedResources.has(href)).length;

        const nav = performance.getEntriesByType('navigation')[0];
        r.documentDecodedBytes = nav ? Math.round(nav.decodedBodySize || 0) : 0;
        r.documentTransferBytes = nav ? Math.round(nav.transferSize || 0) : 0;
        r.cacheProbeUrls = Array.from(new Set(
            performance.getEntriesByType('resource')
                .filter(entry => {
                    const type = entry.initiatorType || '';
                    if (['script', 'link', 'css', 'img', 'font'].includes(type)) return true;
                    return /\.(js|css|mjs|woff2?|ttf|otf|png|jpe?g|gif|webp|avif|svg)(\?|$)/i.test(entry.name || '');
                })
                .map(entry => entry.name)
                .filter(Boolean)
        )).slice(0, 30);

        // LCP image candidate: largest visible img by display area
        let lcpImg = null, lcpArea = 0;
        document.querySelectorAll('img[src]').forEach(img => {
            const rect = img.getBoundingClientRect();
            if (rect.top < 0 || rect.top > window.innerHeight) return;
            const area = rect.width * rect.height;
            if (area > lcpArea) { lcpArea = area; lcpImg = img; }
        });
        if (lcpImg) {
            const lcpSrc = lcpImg.src;
            const hasPreload = [...preloadHrefs].some(h => h === lcpSrc);
            r.lcpImageWithoutPreload = !hasPreload;
            r.lcpImageUrl = lcpSrc;
            r.lcpImageWithoutFetchpriority = lcpImg.getAttribute('fetchpriority') !== 'high';
            r.lcpImageLazyLoaded = lcpImg.getAttribute('loading') === 'lazy';
        } else {
            r.lcpImageWithoutPreload = false;
            r.lcpImageWithoutFetchpriority = false;
            r.lcpImageLazyLoaded = false;
        }

        // Deprecated API detection in inline scripts
        const DEPRECATED = ['AppCache', 'document.domain =', 'webkitStorageInfo',
            'webkitIndexedDB', 'navigator.userAgentData', 'importScripts'];
        const inlineText = Array.from(document.querySelectorAll('script:not([src])')).map(s => s.textContent).join('\n');
        r.deprecatedApiCount = DEPRECATED.filter(p => inlineText.includes(p)).length;

        // Hreflang validation
        const hreflangLinks = Array.from(document.querySelectorAll('link[rel="alternate"][hreflang]'));
        r.hreflangCount = hreflangLinks.length;
        const langPattern = /^[a-z]{2,3}(-[A-Z]{2})?$|^x-default$/;
        r.hreflangInvalidCount = hreflangLinks.filter(l => !langPattern.test(l.getAttribute('hreflang') || '')).length;

        // JSON-LD validation
        const jsonldBlocks = Array.from(document.querySelectorAll('script[type="application/ld+json"]'));
        r.jsonldCount = jsonldBlocks.length;
        r.jsonldInvalidCount = jsonldBlocks.filter(s => {
            try {
                const data = JSON.parse(s.textContent);
                return !data['@context'] || !data['@type'];
            } catch(e) { return true; }
        }).length;

        // Render-blocking: sync <script src> in <head> without defer/async/type=module
        r.syncHeadScripts = Array.from(document.querySelectorAll('head script[src]'))
            .filter(s => {
                const t = (s.getAttribute('type') || '').toLowerCase();
                return !s.hasAttribute('async') && !s.hasAttribute('defer') && t !== 'module';
            }).length;

        // SRI: external scripts and stylesheets without integrity attribute
        const pageOrigin = window.location.origin;
        r.externalScriptsWithoutSri = Array.from(document.querySelectorAll('script[src]'))
            .filter(s => {
                try { return new URL(s.src).origin !== pageOrigin && !s.hasAttribute('integrity'); }
                catch(e) { return false; }
            }).length;
        r.externalStylesWithoutSri = Array.from(document.querySelectorAll('link[rel="stylesheet"][href]'))
            .filter(l => {
                try { return new URL(l.href).origin !== pageOrigin && !l.hasAttribute('integrity'); }
                catch(e) { return false; }
            }).length;

        // Fragment anchor validation: #anchor links where target ID does not exist on this page
        const pageIds = new Set(Array.from(document.querySelectorAll('[id]')).map(el => el.id));
        const brokenFragmentEls = Array.from(document.querySelectorAll('a')).filter(a => {
            const href = a.getAttribute('href') || '';
            if (href.charAt(0) !== '#') return false;
            const frag = href.slice(1);
            if (!frag) return false;
            try { return !pageIds.has(frag) && !pageIds.has(decodeURIComponent(frag)); }
            catch(e) { return !pageIds.has(frag); }
        });
        r.brokenFragmentLinks = brokenFragmentEls.length;
        r.brokenFragmentSamples = brokenFragmentEls.slice(0, 5).map(a => a.getAttribute('href'));

        // Generic link text: links with non-descriptive anchor text
        const GENERIC_TEXTS = new Set(['hier', 'mehr', 'weiter', 'klick', 'link', 'details', 'ansehen',
            'click here', 'read more', 'learn more', 'more', 'here']);
        const genericLinkEls = Array.from(document.querySelectorAll('a')).filter(a => {
            const text = (a.textContent || '').trim().toLowerCase();
            return text && GENERIC_TEXTS.has(text);
        });
        r.genericLinkTextCount = genericLinkEls.length;
        r.genericLinkTextSamples = genericLinkEls.slice(0, 5).map(a => a.getAttribute('href') || '(no href)');

        return JSON.stringify(r);
    })()
    "#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Page health JS failed: {}", e)))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("{}");
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

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

    Ok(())
}

fn build_html_issues(
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
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        html_issues.push(HtmlValidationIssue {
            check: "Doppelte IDs".to_string(),
            count: a.duplicate_id_count,
            severity: "medium".to_string(),
            detail: if samples.is_empty() {
                format!("{} gefunden", a.duplicate_id_count)
            } else {
                format!("{} (z.B. {})", a.duplicate_id_count, samples)
            },
        });
    }

    if a.images_without_alt > 0 {
        html_issues.push(HtmlValidationIssue {
            check: "Bilder ohne alt-Attribut".to_string(),
            count: a.images_without_alt,
            severity: "high".to_string(),
            detail: format!("{} <img> ohne alt", a.images_without_alt),
        });
    }
    if a.tables_without_headers > 0 {
        html_issues.push(HtmlValidationIssue {
            check: "Tabellen ohne Kopfzeile".to_string(),
            count: a.tables_without_headers,
            severity: "medium".to_string(),
            detail: format!(
                "{} <table> ohne <th> oder <caption>",
                a.tables_without_headers
            ),
        });
    }
    if a.empty_headings > 0 {
        html_issues.push(HtmlValidationIssue {
            check: "Leere Überschriften".to_string(),
            count: a.empty_headings,
            severity: "medium".to_string(),
            detail: format!("{} leere h1–h6 Elemente", a.empty_headings),
        });
    }
    if a.nested_interactive_count > 0 {
        html_issues.push(HtmlValidationIssue {
            check: "Verschachtelte interaktive Elemente".to_string(),
            count: a.nested_interactive_count,
            severity: "high".to_string(),
            detail: format!("{} button/a in button/a", a.nested_interactive_count),
        });
    }

    html_issues
}

async fn run_w3c_html_validation(
    page: &Page,
    _url: &str,
    a: &mut PageHealthAnalysis,
) -> Result<()> {
    let html = extract_document_html(page).await?;
    let issues = validate_html_locally(&html);
    a.html_issues.extend(issues);
    a.html_validator_status = "executed".to_string();
    a.html_validator_detail = Some("HTML5-Validierung lokal via html5ever".to_string());
    Ok(())
}

fn validate_html_locally(html: &str) -> Vec<HtmlValidationIssue> {
    let dom: RcDom = parse_document(RcDom::default(), Default::default()).one(html);

    let errors = dom.errors;
    if errors.is_empty() {
        return Vec::new();
    }

    let parts: Vec<String> = errors.iter().take(3).map(|e| e.to_string()).collect();
    let detail = if errors.len() <= 3 {
        parts.join(" | ")
    } else {
        format!("{} | +{} weitere", parts.join(" | "), errors.len() - 3)
    };

    vec![HtmlValidationIssue {
        check: "HTML5-Parsing-Fehler".to_string(),
        count: errors.len() as u32,
        severity: "high".to_string(),
        detail,
    }]
}

async fn extract_document_html(page: &Page) -> Result<String> {
    let js = r#"
    (() => {
        const d = document.doctype;
        const doctype = d
            ? `<!DOCTYPE ${d.name}${d.publicId ? ` PUBLIC "${d.publicId}"` : ''}${d.systemId ? ` "${d.systemId}"` : ''}>`
            : '<!DOCTYPE html>';
        return doctype + '\n' + document.documentElement.outerHTML;
    })()
    "#;

    let result = page.evaluate(js).await.map_err(|e| {
        AuditError::CdpError(format!("HTML extraction for validator failed: {}", e))
    })?;

    result
        .value()
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            AuditError::CdpError("Validator HTML extraction returned no string".to_string())
        })
}

// ─── HTTP probes ─────────────────────────────────────────────────────────────

async fn run_http_probes(url: &str, a: &mut PageHealthAnalysis) {
    let Ok(parsed) = url::Url::parse(url) else {
        return;
    };
    let origin = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));

    // Run all probes concurrently
    let probe_url = format!("{}/auditmysite-404-probe-xyz123", origin);
    let cache_probe_urls = a.resource_cache_probe_urls.clone();
    let (custom_404_result, www_result, header_result, redirect_chain, resource_cache) = tokio::join!(
        probe_custom_404(&probe_url),
        check_www_consolidation(url),
        probe_headers(url),
        follow_redirect_chain(url),
        audit_resource_cache(&cache_probe_urls)
    );

    // Soft 404: only record the probe status when the server actually returns
    // 200 for a non-existent URL (= soft 404 confirmed). A proper 404/301/etc.
    // response means everything is fine — leave soft_404_status as None so the
    // JSON field stays absent rather than showing a confusing non-200 code.
    if let Some(check) = custom_404_result {
        a.is_soft_404 = check.status == 200;
        if a.is_soft_404 {
            a.soft_404_status = Some(check.status);
        }
        debug!("Custom-404 probe: {} → {}", probe_url, check.status);
        a.custom_404 = Some(check);
    }

    // www consolidation
    a.www_consolidation = www_result;

    // Redirect chain
    let hops: Vec<RedirectHop> = redirect_chain
        .into_iter()
        .filter(|(status, _)| *status >= 300 && *status < 400)
        .map(|(status, url)| RedirectHop { status, url })
        .collect();
    a.redirect_count = hops.len() as u32;
    a.redirect_chain = hops;

    // HTTP headers
    if let Some((uses_http2, compression, cache_control, server_timing_count)) = header_result {
        a.uses_http2 = uses_http2;
        a.has_compression = compression || document_timing_indicates_compression(a);
        a.has_efficient_cache = cache_control
            .as_deref()
            .map(is_cache_policy_efficient)
            .unwrap_or(false);
        a.cache_control = cache_control;
        a.server_timing_count = server_timing_count;
    }
    if document_timing_indicates_compression(a) {
        a.has_compression = true;
    }
    a.resource_cache = resource_cache;
}

/// Probe main page headers: HTTP version, compression, Cache-Control, Server-Timing.
async fn probe_headers(url: &str) -> Option<(bool, bool, Option<String>, u32)> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("auditmysite-probe/1.0")
        .build()
        .ok()?;

    let resp = client.get(url).send().await.ok()?;
    let uses_http2 = matches!(
        resp.version(),
        reqwest::Version::HTTP_2 | reqwest::Version::HTTP_3
    );
    let headers = resp.headers();

    let compression = headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("gzip") || v.contains("br") || v.contains("zstd"))
        .unwrap_or(false);

    let cache_control = headers
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let server_timing_count = headers
        .get("server-timing")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').count() as u32)
        .unwrap_or(0);

    Some((uses_http2, compression, cache_control, server_timing_count))
}

/// Follow redirect chain manually, returning (status, url) pairs for each hop.
async fn follow_redirect_chain(url: &str) -> Vec<(u16, String)> {
    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("auditmysite-probe/1.0")
        .build()
    else {
        return Vec::new();
    };

    let mut chain = Vec::new();
    let mut current = url.to_string();

    for _ in 0..10 {
        let Ok(resp) = client.head(&current).send().await else {
            break;
        };
        let status = resp.status().as_u16();
        chain.push((status, current.clone()));
        if !(300..400).contains(&status) {
            break;
        }
        let Some(location) = resp.headers().get("location").and_then(|v| v.to_str().ok()) else {
            break;
        };
        current = if location.starts_with("http") {
            location.to_string()
        } else if let Ok(base) = url::Url::parse(&current) {
            match base.join(location) {
                Ok(u) => u.to_string(),
                Err(_) => break,
            }
        } else {
            break;
        };
    }
    chain
}

/// Probe a known non-existent URL and classify status plus page customization.
async fn probe_custom_404(url: &str) -> Option<Custom404Check> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
        .ok()?;

    let response = client.get(url).send().await.ok()?;
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();

    Some(Custom404Check {
        probe_url: url.to_string(),
        status,
        proper_status: matches!(status, 404 | 410),
        custom_page: looks_like_custom_404_page(&body),
    })
}

async fn audit_resource_cache(urls: &[String]) -> ResourceCacheAudit {
    if urls.is_empty() {
        return ResourceCacheAudit::default();
    }

    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(3))
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("auditmysite-cache-probe/1.0")
        .build()
    else {
        return ResourceCacheAudit::default();
    };

    let mut audit = ResourceCacheAudit::default();
    let mut seen = std::collections::BTreeSet::new();

    for url in urls
        .iter()
        .filter(|url| seen.insert((*url).clone()))
        .take(30)
    {
        let Ok(response) = client.head(url).send().await else {
            continue;
        };
        if !response.status().is_success() {
            continue;
        }

        audit.checked_resources += 1;
        let headers = response.headers();
        let cache_control = headers
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let has_etag = headers.get("etag").is_some();
        let has_expires = headers.get("expires").is_some();
        let immutable = cache_control
            .as_deref()
            .map(|cc| cc.to_ascii_lowercase().contains("immutable"))
            .unwrap_or(false);

        if has_etag {
            audit.etag_resources += 1;
        }
        if has_expires {
            audit.expires_resources += 1;
        }
        if immutable {
            audit.immutable_resources += 1;
        }

        if !is_static_cache_candidate(url) {
            continue;
        }

        audit.cacheable_resources += 1;
        let efficient = cache_control
            .as_deref()
            .map(is_cache_policy_efficient)
            .unwrap_or(false)
            || (has_expires && cache_control.is_none());

        if !efficient {
            audit.inefficient_resources += 1;
            if audit.samples.len() < 5 {
                audit.samples.push(ResourceCacheFinding {
                    url: url.clone(),
                    cache_control,
                    has_etag,
                    has_expires,
                    reason: cache_policy_reason(headers),
                });
            }
        }
    }

    audit
}

fn looks_like_custom_404_page(body: &str) -> bool {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.len() < 400 {
        return false;
    }

    let lower = normalized.to_ascii_lowercase();
    let generic_markers = [
        "nginx",
        "apache",
        "iis",
        "not found",
        "the requested url was not found",
        "404 not found",
    ];
    let looks_generic =
        normalized.len() < 800 && generic_markers.iter().any(|marker| lower.contains(marker));

    !looks_generic
}

fn is_static_cache_candidate(url: &str) -> bool {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next_back().map(str::to_ascii_lowercase))
        })
        .map(|filename| {
            filename.ends_with(".js")
                || filename.ends_with(".mjs")
                || filename.ends_with(".css")
                || filename.ends_with(".woff")
                || filename.ends_with(".woff2")
                || filename.ends_with(".ttf")
                || filename.ends_with(".otf")
                || filename.ends_with(".png")
                || filename.ends_with(".jpg")
                || filename.ends_with(".jpeg")
                || filename.ends_with(".gif")
                || filename.ends_with(".webp")
                || filename.ends_with(".avif")
                || filename.ends_with(".svg")
        })
        .unwrap_or(false)
}

fn is_cache_policy_efficient(cache_control: &str) -> bool {
    let lower = cache_control.to_ascii_lowercase();
    if lower.contains("no-store") || lower.contains("no-cache") || lower.contains("max-age=0") {
        return false;
    }
    lower.contains("immutable")
        || max_cache_age_seconds(&lower).is_some_and(|seconds| seconds >= 86_400)
}

fn max_cache_age_seconds(cache_control: &str) -> Option<u64> {
    cache_control
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            let value = part
                .strip_prefix("max-age=")
                .or_else(|| part.strip_prefix("s-maxage="))?;
            value.parse::<u64>().ok()
        })
        .max()
}

fn cache_policy_reason(headers: &reqwest::header::HeaderMap) -> String {
    let cache_control = headers
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if cache_control.is_empty() {
        return "missing Cache-Control".to_string();
    }
    if cache_control.to_ascii_lowercase().contains("no-store")
        || cache_control.to_ascii_lowercase().contains("no-cache")
    {
        return "explicitly disables caching".to_string();
    }
    "short or missing max-age/s-maxage".to_string()
}

/// Check www ↔ non-www redirect configuration.
async fn check_www_consolidation(url: &str) -> Option<WwwConsolidation> {
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str()?;

    // Skip IPs, localhost, and subdomains that aren't www
    if host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    let (www_url, non_www_url) = if host.starts_with("www.") {
        let non_www_host = host.strip_prefix("www.")?;
        let non_www = url.replacen(host, non_www_host, 1);
        (url.to_string(), non_www)
    } else {
        // Only handle apex domains (e.g. example.com), skip subdomains like api.example.com
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() != 2 {
            return None;
        }
        let www_host = format!("www.{}", host);
        let www = url.replacen(host, &www_host, 1);
        (www, url.to_string())
    };

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
        .ok()?;

    let (www_resp, non_www_resp) = tokio::join!(
        client.head(&www_url).send(),
        client.head(&non_www_url).send()
    );

    let www_status = www_resp.as_ref().ok().map(|r| r.status().as_u16());
    let non_www_status = non_www_resp.as_ref().ok().map(|r| r.status().as_u16());

    let www_location = www_resp
        .ok()
        .and_then(|r| r.headers().get("location").cloned())
        .and_then(|v| v.to_str().ok().map(String::from))
        .unwrap_or_default();
    let non_www_location = non_www_resp
        .ok()
        .and_then(|r| r.headers().get("location").cloned())
        .and_then(|v| v.to_str().ok().map(String::from))
        .unwrap_or_default();

    let www_redirects = matches!(www_status, Some(301) | Some(302) | Some(307) | Some(308))
        && !www_location.contains("www.");
    let non_www_redirects = matches!(
        non_www_status,
        Some(301) | Some(302) | Some(307) | Some(308)
    ) && non_www_location.contains("www.");

    let is_consolidated = www_redirects || non_www_redirects;
    let canonical_variant = if www_redirects {
        "non-www".to_string()
    } else if non_www_redirects {
        "www".to_string()
    } else if is_consolidated {
        "consistent".to_string()
    } else {
        "inconsistent".to_string()
    };

    Some(WwwConsolidation {
        www_status,
        non_www_status,
        www_redirects_to_non_www: www_redirects,
        non_www_redirects_to_www: non_www_redirects,
        canonical_variant,
        is_consolidated,
    })
}

// ─── Issue aggregation ───────────────────────────────────────────────────────

/// Build the page-health issue list in the requested language.
///
/// Pure function of the analysis struct — called with English at analysis time
/// (canonical, for JSON) and re-called with the report locale by the PDF
/// presentation builder (#406).
pub fn collect_issues(a: &PageHealthAnalysis, en: bool) -> Vec<PageHealthIssue> {
    let mut issues = Vec::new();

    if !a.has_doctype && a.dom_node_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "missing_doctype".to_string(),
            message: if en {
                "Missing HTML5 doctype declaration — browser renders in quirks mode".to_string()
            } else {
                "Fehlende HTML5-Doctype-Deklaration — Browser rendert im Quirks-Mode".to_string()
            },
            severity: "high".to_string(),
        });
    }

    if a.document_write_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "document_write".to_string(),
            message: if en {
                format!(
                    "{} inline {} use document.write() — blocks HTML parsing",
                    a.document_write_count,
                    if a.document_write_count == 1 {
                        "script"
                    } else {
                        "scripts"
                    }
                )
            } else {
                format!(
                    "{} Inline-{} verwenden document.write() — blockiert HTML-Parsing",
                    a.document_write_count,
                    if a.document_write_count == 1 {
                        "Script"
                    } else {
                        "Scripts"
                    }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.dom_node_count >= 1500 {
        issues.push(PageHealthIssue {
            issue_type: "excessive_dom".to_string(),
            message: if en {
                format!(
                    "DOM size critical: {} elements (recommended: <1500, max depth: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            } else {
                format!(
                    "DOM-Größe kritisch: {} Elemente (Empfehlung: <1500, max Tiefe: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            },
            severity: "high".to_string(),
        });
    } else if a.dom_node_count >= 800 {
        issues.push(PageHealthIssue {
            issue_type: "large_dom".to_string(),
            message: if en {
                format!(
                    "DOM size elevated: {} elements (recommended: <800, max depth: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            } else {
                format!(
                    "DOM-Größe erhöht: {} Elemente (Empfehlung: <800, max Tiefe: {})",
                    a.dom_node_count, a.dom_max_depth
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.images_without_dimensions > 0 {
        issues.push(PageHealthIssue {
            issue_type: "images_without_dimensions".to_string(),
            message: if en {
                format!(
                    "{} <img> elements without explicit width/height — CLS risk",
                    a.images_without_dimensions
                )
            } else {
                format!(
                    "{} <img>-Elemente ohne explizite width/height — CLS-Risiko",
                    a.images_without_dimensions
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.paste_blocking_password_fields > 0 {
        issues.push(PageHealthIssue {
            issue_type: "paste_blocked_password".to_string(),
            message: if en {
                format!(
                    "{} password {} block pasting (onpaste handler) — interferes with password managers",
                    a.paste_blocking_password_fields,
                    if a.paste_blocking_password_fields == 1 { "field" } else { "fields" }
                )
            } else {
                format!(
                    "{} {} blockieren Einfügen (onpaste-Handler) — beeinträchtigt Passwort-Manager",
                    a.paste_blocking_password_fields,
                    if a.paste_blocking_password_fields == 1 { "Passwortfeld" } else { "Passwortfelder" }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.offscreen_images_without_lazy > 0 {
        issues.push(PageHealthIssue {
            issue_type: "offscreen_images_without_lazy".to_string(),
            message: if en {
                format!(
                    "{} images below the viewport without loading=\"lazy\" — delay initial page render",
                    a.offscreen_images_without_lazy
                )
            } else {
                format!(
                    "{} Bilder unterhalb des Viewports ohne loading=\"lazy\" — verzögern ersten Seitenaufbau",
                    a.offscreen_images_without_lazy
                )
            },
            severity: if a.offscreen_images_without_lazy >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.images_without_srcset > 0 {
        issues.push(PageHealthIssue {
            issue_type: "images_without_srcset".to_string(),
            message: if en {
                format!(
                    "{} <img> elements without srcset — no responsive image variants for different resolutions",
                    a.images_without_srcset
                )
            } else {
                format!(
                    "{} <img>-Elemente ohne srcset — keine responsiven Bildvarianten für verschiedene Auflösungen",
                    a.images_without_srcset
                )
            },
            severity: if a.images_without_srcset >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.missing_preconnect_count > 0 {
        let sample = if a.missing_preconnect_origins.is_empty() {
            String::new()
        } else if en {
            format!(" (e.g. {})", a.missing_preconnect_origins.join(", "))
        } else {
            format!(" (z.B. {})", a.missing_preconnect_origins.join(", "))
        };
        issues.push(PageHealthIssue {
            issue_type: "missing_preconnect".to_string(),
            message: if en {
                format!(
                    "{} external origins without <link rel=\"preconnect\">{}",
                    a.missing_preconnect_count, sample
                )
            } else {
                format!(
                    "{} externe Origins ohne <link rel=\"preconnect\">{}",
                    a.missing_preconnect_count, sample
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.non_crawlable_links > 0 {
        issues.push(PageHealthIssue {
            issue_type: "non_crawlable_links".to_string(),
            message: if en {
                format!(
                    "{} links not crawlable (javascript:, empty, no href) — loss of PageRank",
                    a.non_crawlable_links
                )
            } else {
                format!(
                    "{} Links nicht crawlbar (javascript:, leer, kein href) — PageRank-Verlust",
                    a.non_crawlable_links
                )
            },
            severity: if a.non_crawlable_links >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.images_without_modern_format > 0 {
        issues.push(PageHealthIssue {
            issue_type: "images_without_modern_format".to_string(),
            message: if en {
                format!(
                    "{} images as JPEG/PNG without a WebP/AVIF alternative — increased load time",
                    a.images_without_modern_format
                )
            } else {
                format!(
                    "{} Bilder als JPEG/PNG ohne WebP/AVIF-Alternative — erhöhte Ladezeit",
                    a.images_without_modern_format
                )
            },
            severity: if a.images_without_modern_format >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.oversized_images > 0 {
        issues.push(PageHealthIssue {
            issue_type: "oversized_images".to_string(),
            message: if en {
                format!(
                    "{} images are displayed significantly smaller than their natural resolution",
                    a.oversized_images
                )
            } else {
                format!(
                    "{} Bilder werden deutlich kleiner dargestellt als ihre natürliche Auflösung",
                    a.oversized_images
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.gif_images > 0 {
        issues.push(PageHealthIssue {
            issue_type: "gif_images".to_string(),
            message: if en {
                format!(
                    "{} GIF {} found — consider replacing with MP4/WebM (80–95% smaller)",
                    a.gif_images,
                    if a.gif_images == 1 { "image" } else { "images" }
                )
            } else {
                format!(
                    "{} {} gefunden — ggf. als MP4/WebM ersetzen (80–95 % kleiner)",
                    a.gif_images,
                    if a.gif_images == 1 {
                        "GIF-Bild"
                    } else {
                        "GIF-Bilder"
                    }
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.font_display_issues > 0 {
        issues.push(PageHealthIssue {
            issue_type: "font_display_missing".to_string(),
            message: if en {
                format!(
                    "{} @font-face rules without font-display: swap/fallback/optional — FOIT risk",
                    a.font_display_issues
                )
            } else {
                format!(
                    "{} @font-face-Regeln ohne font-display: swap/fallback/optional — FOIT-Risiko",
                    a.font_display_issues
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.orphaned_preload_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "orphaned_preload".to_string(),
            message: if en {
                format!(
                    "{} <link rel=\"preload\"> hints point to resources that were never loaded (orphaned)",
                    a.orphaned_preload_count
                )
            } else {
                format!(
                    "{} <link rel=\"preload\">-Hinweise auf nicht geladene Ressourcen (orphaned)",
                    a.orphaned_preload_count
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.lcp_image_lazy_loaded {
        issues.push(PageHealthIssue {
            issue_type: "lcp_image_lazy_loaded".to_string(),
            message: if en {
                "LCP image candidate has loading=\"lazy\" — delays the Largest Contentful Paint"
                    .to_string()
            } else {
                "LCP-Bildkandidat hat loading=\"lazy\" — verzögert den Largest Contentful Paint"
                    .to_string()
            },
            severity: "high".to_string(),
        });
    }

    if a.lcp_image_without_preload {
        let url_hint = a
            .lcp_image_url
            .as_deref()
            .map(|u| format!(" ({})", u))
            .unwrap_or_default();
        issues.push(PageHealthIssue {
            issue_type: "lcp_image_without_preload".to_string(),
            message: if en {
                format!(
                    "Largest visible image{} has no <link rel=\"preload\" as=\"image\"> hint",
                    url_hint
                )
            } else {
                format!(
                    "Größtes sichtbares Bild{} hat keinen <link rel=\"preload\" as=\"image\">-Hint",
                    url_hint
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.lcp_image_without_fetchpriority && !a.lcp_image_lazy_loaded {
        issues.push(PageHealthIssue {
            issue_type: "lcp_image_without_fetchpriority".to_string(),
            message: if en {
                "LCP image candidate is missing fetchpriority=\"high\" — load priority not signaled"
                    .to_string()
            } else {
                "LCP-Bildkandidat fehlt fetchpriority=\"high\" — Ladepriorität nicht signalisiert"
                    .to_string()
            },
            severity: "low".to_string(),
        });
    }

    if a.deprecated_api_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "deprecated_apis".to_string(),
            message: if en {
                format!(
                    "{} deprecated browser {} detected in inline scripts",
                    a.deprecated_api_count,
                    if a.deprecated_api_count == 1 {
                        "API"
                    } else {
                        "APIs"
                    }
                )
            } else {
                format!(
                    "{} veraltete {} in Inline-Scripts erkannt",
                    a.deprecated_api_count,
                    if a.deprecated_api_count == 1 {
                        "Browser-API"
                    } else {
                        "Browser-APIs"
                    }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if !a.uses_http2 {
        issues.push(PageHealthIssue {
            issue_type: "http1_only".to_string(),
            message: if en {
                "Page is served over HTTP/1.1 — HTTP/2 enables multiplexing and header compression".to_string()
            } else {
                "Seite wird über HTTP/1.1 ausgeliefert — HTTP/2 ermöglicht Multiplexing und Header-Komprimierung".to_string()
            },
            severity: "medium".to_string(),
        });
    }

    if !a.has_compression {
        issues.push(PageHealthIssue {
            issue_type: "missing_compression".to_string(),
            message: if en {
                "Page content is transferred without compression (no gzip/brotli)".to_string()
            } else {
                "Seiteninhalt wird ohne Komprimierung übertragen (kein gzip/brotli)".to_string()
            },
            severity: "high".to_string(),
        });
    }

    if !a.has_efficient_cache && a.cache_control.is_some() {
        issues.push(PageHealthIssue {
            issue_type: "inefficient_cache".to_string(),
            message: if en {
                format!(
                    "Cache-Control without an effective caching lifetime: {}",
                    a.cache_control.as_deref().unwrap_or("")
                )
            } else {
                format!(
                    "Cache-Control ohne effektive Caching-Dauer: {}",
                    a.cache_control.as_deref().unwrap_or("")
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.resource_cache.inefficient_resources > 0 {
        let sample = a
            .resource_cache
            .samples
            .first()
            .map(|finding| {
                if en {
                    format!(" (e.g. {}: {})", finding.url, finding.reason)
                } else {
                    format!(" (z.B. {}: {})", finding.url, finding.reason)
                }
            })
            .unwrap_or_default();
        issues.push(PageHealthIssue {
            issue_type: "inefficient_resource_cache".to_string(),
            message: if en {
                format!(
                    "{} of {} static {} without an efficient cache policy{}",
                    a.resource_cache.inefficient_resources,
                    a.resource_cache.cacheable_resources,
                    if a.resource_cache.cacheable_resources == 1 {
                        "resource"
                    } else {
                        "resources"
                    },
                    sample
                )
            } else {
                format!(
                    "{} von {} statischen {} ohne effiziente Cache-Policy{}",
                    a.resource_cache.inefficient_resources,
                    a.resource_cache.cacheable_resources,
                    if a.resource_cache.cacheable_resources == 1 {
                        "Ressource"
                    } else {
                        "Ressourcen"
                    },
                    sample
                )
            },
            severity: if a.resource_cache.inefficient_resources >= 5 {
                "medium"
            } else {
                "low"
            }
            .to_string(),
        });
    }

    if a.hreflang_invalid_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "hreflang_invalid".to_string(),
            message: if en {
                format!(
                    "{} hreflang entries with an invalid language code",
                    a.hreflang_invalid_count
                )
            } else {
                format!(
                    "{} hreflang-Einträge mit ungültigem Sprachcode",
                    a.hreflang_invalid_count
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.jsonld_invalid_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "jsonld_invalid".to_string(),
            message: if en {
                format!(
                    "{} JSON-LD blocks without @context or @type",
                    a.jsonld_invalid_count
                )
            } else {
                format!(
                    "{} JSON-LD-Blöcke ohne @context oder @type",
                    a.jsonld_invalid_count
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.sync_head_scripts > 0 {
        issues.push(PageHealthIssue {
            issue_type: "sync_head_scripts".to_string(),
            message: if en {
                format!(
                    "{} {} in <head> without defer/async/module are syntactically render-blocking; whether they delay measurably is shown by the render-blocking analysis.",
                    a.sync_head_scripts,
                    if a.sync_head_scripts == 1 { "script" } else { "scripts" }
                )
            } else {
                format!(
                    "{} {} im <head> ohne defer/async/module sind syntaktisch render-blockierend; ob sie messbar verzögern, zeigt die Render-Blocking-Analyse.",
                    a.sync_head_scripts,
                    if a.sync_head_scripts == 1 { "Script" } else { "Scripts" }
                )
            },
            severity: "medium".to_string(),
        });
    }

    let sri_total = a.external_scripts_without_sri + a.external_styles_without_sri;
    if sri_total > 0 {
        issues.push(PageHealthIssue {
            issue_type: "missing_sri".to_string(),
            message: if en {
                format!(
                    "{} external {} without Subresource Integrity (integrity attribute missing)",
                    sri_total,
                    if sri_total == 1 {
                        "resource"
                    } else {
                        "resources"
                    }
                )
            } else {
                format!(
                    "{} externe {} ohne Subresource Integrity (integrity-Attribut fehlt)",
                    sri_total,
                    if sri_total == 1 {
                        "Ressource"
                    } else {
                        "Ressourcen"
                    }
                )
            },
            severity: "medium".to_string(),
        });
    }

    if a.broken_fragment_links > 0 {
        let sample = if a.broken_fragment_samples.is_empty() {
            String::new()
        } else if en {
            format!(" (e.g. {})", a.broken_fragment_samples.join(", "))
        } else {
            format!(" (z.B. {})", a.broken_fragment_samples.join(", "))
        };
        issues.push(PageHealthIssue {
            issue_type: "broken_fragment_links".to_string(),
            message: if en {
                format!(
                    "{} anchor {} point to non-existent IDs{}",
                    a.broken_fragment_links,
                    if a.broken_fragment_links == 1 {
                        "link"
                    } else {
                        "links"
                    },
                    sample
                )
            } else {
                format!(
                    "{} Anker-{} verweisen auf nicht existierende IDs{}",
                    a.broken_fragment_links,
                    if a.broken_fragment_links == 1 {
                        "Link"
                    } else {
                        "Links"
                    },
                    sample
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.generic_link_text_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "generic_link_text".to_string(),
            message: if en {
                format!(
                    "{} {} with non-descriptive text (\"here\", \"more\", \"click here\" and similar)",
                    a.generic_link_text_count,
                    if a.generic_link_text_count == 1 { "link" } else { "links" }
                )
            } else {
                format!(
                    "{} {} mit nicht-beschreibendem Text (\"hier\", \"mehr\", \"click here\" u.ä.)",
                    a.generic_link_text_count,
                    if a.generic_link_text_count == 1 { "Link" } else { "Links" }
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.is_soft_404 {
        issues.push(PageHealthIssue {
            issue_type: "soft_404".to_string(),
            message: if en {
                format!(
                    "Server returns HTTP {} for non-existent URLs (soft 404)",
                    a.soft_404_status.unwrap_or(200)
                )
            } else {
                format!(
                    "Server gibt HTTP {} für nicht-existierende URLs zurück (Soft 404)",
                    a.soft_404_status.unwrap_or(200)
                )
            },
            severity: "high".to_string(),
        });
    }

    if let Some(custom_404) = &a.custom_404 {
        if !custom_404.proper_status {
            issues.push(PageHealthIssue {
                issue_type: "custom_404_invalid_status".to_string(),
                message: if en {
                    format!(
                        "Non-existent URL returns HTTP {} instead of 404/410",
                        custom_404.status
                    )
                } else {
                    format!(
                        "Nicht-existente URL liefert HTTP {} statt 404/410",
                        custom_404.status
                    )
                },
                severity: "high".to_string(),
            });
        } else if !custom_404.custom_page {
            issues.push(PageHealthIssue {
                issue_type: "generic_404_page".to_string(),
                message: if en {
                    "404 page looks generic or very sparse; a helpful custom 404 page improves orientation and crawling signals".to_string()
                } else {
                    "404-Seite wirkt generisch oder sehr knapp; eine hilfreiche Custom-404-Seite verbessert Orientierung und Crawling-Signale".to_string()
                },
                severity: "low".to_string(),
            });
        }
    }

    if a.has_meta_refresh {
        let delay = a
            .meta_refresh_content
            .as_deref()
            .and_then(|c| c.split(';').next())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        issues.push(PageHealthIssue {
            issue_type: "meta_refresh".to_string(),
            message: match (en, delay == 0) {
                (true, true) => {
                    "Immediate redirect via meta-refresh (harmful for SEO, use a 301 redirect)"
                        .to_string()
                }
                (true, false) => format!("meta-refresh with {}s delay found", delay),
                (false, true) => {
                    "Sofort-Weiterleitung via meta-refresh (SEO-schädlich, nutze 301-Redirect)"
                        .to_string()
                }
                (false, false) => format!("meta-refresh mit {}s Verzögerung gefunden", delay),
            },
            severity: if delay == 0 { "high" } else { "medium" }.to_string(),
        });
    }

    if a.frame_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "frames".to_string(),
            message: if en {
                format!("{} deprecated <frame> elements found", a.frame_count)
            } else {
                format!("{} veraltete <frame>-Elemente gefunden", a.frame_count)
            },
            severity: "high".to_string(),
        });
    }

    if a.url_is_too_long {
        issues.push(PageHealthIssue {
            issue_type: "url_too_long".to_string(),
            message: if en {
                format!(
                    "URL with {} characters exceeds the recommendation (>115)",
                    a.url_length
                )
            } else {
                format!(
                    "URL mit {} Zeichen überschreitet Empfehlung (>115)",
                    a.url_length
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.url_is_too_deep {
        issues.push(PageHealthIssue {
            issue_type: "url_too_deep".to_string(),
            message: if en {
                format!(
                    "URL path depth {} exceeds the recommendation (>5 levels)",
                    a.url_path_depth
                )
            } else {
                format!(
                    "URL-Pfadtiefe {} überschreitet Empfehlung (>5 Ebenen)",
                    a.url_path_depth
                )
            },
            severity: "low".to_string(),
        });
    }

    if a.url_has_query_params {
        issues.push(PageHealthIssue {
            issue_type: "dynamic_url".to_string(),
            message: if en {
                "URL contains query parameters (dynamic URL)".to_string()
            } else {
                "URL enthält Query-Parameter (dynamische URL)".to_string()
            },
            severity: "low".to_string(),
        });
    }

    if a.redirect_count >= 2 {
        issues.push(PageHealthIssue {
            issue_type: "multiple_redirects".to_string(),
            message: if en {
                format!(
                    "{} HTTP redirects before the final page — each hop costs ~100–300 ms",
                    a.redirect_count
                )
            } else {
                format!(
                    "{} HTTP-Weiterleitungen vor der finalen Seite — jeder Hop kostet ~100–300 ms",
                    a.redirect_count
                )
            },
            severity: if a.redirect_count >= 3 {
                "high"
            } else {
                "medium"
            }
            .to_string(),
        });
    } else if a.own_redirect_detected {
        issues.push(PageHealthIssue {
            issue_type: "redirect".to_string(),
            message: if en {
                format!(
                    "Page redirects to: {}",
                    a.own_final_url.as_deref().unwrap_or("(unknown)")
                )
            } else {
                format!(
                    "Seite leitet weiter zu: {}",
                    a.own_final_url.as_deref().unwrap_or("(unbekannt)")
                )
            },
            severity: "medium".to_string(),
        });
    }

    if let Some(ref www) = a.www_consolidation {
        if !www.is_consolidated {
            issues.push(PageHealthIssue {
                issue_type: "www_not_consolidated".to_string(),
                message: if en {
                    "www and non-www versions are not consolidated (no 301 redirect)".to_string()
                } else {
                    "www und non-www Version sind nicht konsolidiert (kein 301-Redirect)"
                        .to_string()
                },
                severity: "medium".to_string(),
            });
        }
    }

    issues
}

fn nonzero_u64(value: &serde_json::Value) -> Option<u64> {
    value.as_u64().filter(|v| *v > 0)
}

fn document_timing_indicates_compression(a: &PageHealthAnalysis) -> bool {
    let (Some(transfer), Some(decoded)) = (a.document_transfer_bytes, a.document_decoded_bytes)
    else {
        return false;
    };
    decoded > 0 && transfer > 0 && (transfer as f64) < (decoded as f64 * 0.8)
}

#[cfg(test)]
mod tests {
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
            i.check == "Doppelte IDs"
                && i.count == 2
                && i.detail.contains("hero")
                && i.detail.contains("cta-button")
        }));
        assert!(issues
            .iter()
            .any(|i| i.check == "Bilder ohne alt-Attribut" && i.count == 3));
        assert!(issues
            .iter()
            .any(|i| i.check == "Tabellen ohne Kopfzeile" && i.count == 1));
        assert!(issues
            .iter()
            .any(|i| i.check == "Leere Überschriften" && i.count == 2));
        assert!(issues
            .iter()
            .any(|i| i.check == "Verschachtelte interaktive Elemente" && i.count == 1));
    }

    #[test]
    fn html5ever_parse_errors_are_structured_findings() {
        let issues = validate_html_locally("<!doctype html><html><head><p></head></html>");

        assert!(issues.iter().any(|issue| {
            issue.check == "HTML5-Parsing-Fehler"
                && issue.count > 0
                && issue.severity == "high"
                && !issue.detail.is_empty()
        }));
    }
}
