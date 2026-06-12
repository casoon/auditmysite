//! Technical SEO analysis
//!
//! Checks HTTPS, canonical URLs, language, sitemap, robots.txt.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use futures::stream::{self, StreamExt};
use reqwest::{redirect::Policy, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tracing::info;

use crate::error::{AuditError, Result};
use crate::taxonomy::Severity;

/// Technical SEO analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnicalSeo {
    /// URL uses HTTPS
    pub https: bool,
    /// Has canonical URL
    pub has_canonical: bool,
    /// Canonical URL value
    pub canonical_url: Option<String>,
    /// Has lang attribute
    pub has_lang: bool,
    /// Language value
    pub lang: Option<String>,
    /// Has robots meta
    pub has_robots_meta: bool,
    /// Robots meta content
    pub robots_meta: Option<String>,
    /// Has hreflang tags
    pub has_hreflang: bool,
    /// Hreflang values
    pub hreflang: Vec<HreflangTag>,
    /// x-default hreflang entry is present (only meaningful when has_hreflang is true)
    pub hreflang_has_x_default: bool,
    /// Page URL is not listed in its own hreflang set (only meaningful when has_hreflang is true)
    pub hreflang_missing_self_reference: bool,
    /// Page appears to be part of a paginated archive/listing.
    pub pagination_detected: bool,
    /// `<link rel="prev">` target, if present.
    pub pagination_prev: Option<String>,
    /// `<link rel="next">` target, if present.
    pub pagination_next: Option<String>,
    /// Internal links that contain query parameters (crawl budget dilution risk)
    pub internal_links_with_query_params: u32,
    /// Word count on page
    pub word_count: u32,
    /// Internal links count
    pub internal_links: u32,
    /// External links count
    pub external_links: u32,
    /// Dofollow links (no nofollow/ugc/sponsored rel)
    pub dofollow_links: u32,
    /// Nofollow links (rel=nofollow/ugc/sponsored)
    pub nofollow_links: u32,
    /// Resolved paths of internal links (for inbound link computation in batch mode, capped at 500)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub internal_link_targets: Vec<String>,
    /// Broken links found
    pub broken_links: Vec<String>,
    /// HTTP subresources referenced from an HTTPS page.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub mixed_content: Vec<MixedContentResource>,
    /// Progressive Web App manifest and service-worker signals.
    #[serde(default)]
    pub pwa: PwaAnalysis,
    /// Sensitive form transport, method, and credential-autocomplete checks.
    #[serde(default)]
    pub form_security: FormSecurityAnalysis,
    /// Visible text excerpt for topic analysis and redundancy checks
    pub text_excerpt: String,
    /// Uses externally hosted Google Fonts assets
    pub uses_remote_google_fonts: bool,
    /// Matching Google Fonts assets or stylesheets
    pub google_fonts_sources: Vec<String>,
    /// Tracking cookies detected on the page
    pub tracking_cookies: Vec<TrackingCookie>,
    /// Tracking providers or signals detected from scripts/resources
    pub tracking_signals: Vec<String>,
    /// Cloudflare Zaraz detected
    pub zaraz: ZarazDetection,
    /// Favicon detected (<link rel="icon"> or apple-touch-icon)
    pub has_favicon: bool,
    /// www / non-www redirect check result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub www_redirect: Option<WwwRedirectCheck>,
    /// Issues found
    pub issues: Vec<TechnicalIssue>,
}

/// Result of the www ↔ non-www redirect check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WwwRedirectCheck {
    /// The URL variant that was audited (primary)
    pub primary: String,
    /// The alternative variant (www ↔ non-www)
    pub alternative: String,
    /// HTTP status code returned by the alternative
    pub alternative_status: u16,
    /// Does the alternative redirect (301/302/308) to the primary?
    pub redirects_to_primary: bool,
    /// Does the page canonical point to the primary domain? None if no canonical.
    pub canonical_matches_primary: Option<bool>,
}

/// Tracking cookie detected on the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingCookie {
    pub name: String,
    pub scope: String,
    pub provider: String,
}

/// Zaraz detection summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZarazDetection {
    pub detected: bool,
    pub signals: Vec<String>,
}

/// Hreflang tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HreflangTag {
    pub lang: String,
    pub url: String,
}

/// HTTP subresource referenced by an HTTPS page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MixedContentResource {
    pub url: String,
    pub resource_type: String,
    pub blockable: bool,
}

/// Basic Progressive Web App signal analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PwaAnalysis {
    pub manifest_url: Option<String>,
    pub manifest_valid: bool,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub start_url: Option<String>,
    pub display: Option<String>,
    pub theme_color: Option<String>,
    pub icons_count: u32,
    pub has_maskable_icon: bool,
    pub service_worker_supported: bool,
    pub service_worker_controlled: bool,
    pub service_worker_registrations: u32,
}

/// Sensitive form security analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormSecurityAnalysis {
    pub sensitive_forms: u32,
    pub insecure_transport_forms: u32,
    pub get_sensitive_forms: u32,
    pub password_autocomplete_issues: u32,
}

/// Technical SEO issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIssue {
    pub issue_type: String,
    pub message: String,
    pub severity: Severity,
}

/// Analyze technical SEO aspects
pub async fn analyze_technical_seo(page: &Page, url: &str, locale: &str) -> Result<TechnicalSeo> {
    info!("Analyzing technical SEO...");

    let en = locale == "en";
    let https = url.starts_with("https://");

    let js_code = r#"
    (() => {
        const result = {};

        // Canonical
        const canonical = document.querySelector('link[rel="canonical"]');
        result.canonical = canonical ? canonical.getAttribute('href') : null;

        // Language
        result.lang = document.documentElement.getAttribute('lang');

        // Robots meta
        const robots = document.querySelector('meta[name="robots"]');
        result.robots = robots ? robots.getAttribute('content') : null;

        // Hreflang
        result.hreflang = [];
        document.querySelectorAll('link[rel="alternate"][hreflang]').forEach(el => {
            result.hreflang.push({
                lang: el.getAttribute('hreflang'),
                url: el.getAttribute('href')
            });
        });
        result.hreflangHasXDefault = result.hreflang.some(h => h.lang === 'x-default');
        result.hreflangHasSelfReference = result.hreflang.some(h => {
            try {
                const norm = u => new URL(u, window.location.href).href.replace(/\/$/, '');
                return norm(h.url) === norm(window.location.href);
            } catch(e) { return false; }
        });

        // Pagination
        const relHref = rel => {
            const el = Array.from(document.querySelectorAll('link[rel][href]')).find(link => {
                const rels = (link.getAttribute('rel') || '').toLowerCase().split(/\s+/);
                return rels.includes(rel);
            });
            return el ? el.href : null;
        };
        result.paginationPrev = relHref('prev');
        result.paginationNext = relHref('next');
        result.paginationLinkCount = document.querySelectorAll(
            'nav[class*="pag" i], [class*="pagination" i], [aria-label*="pagination" i], [aria-label*="seiten" i]'
        ).length;
        result.paginationAnchorCount = Array.from(document.querySelectorAll('a[href]')).filter(a => {
            const text = (a.textContent || '').trim().toLowerCase();
            const label = (a.getAttribute('aria-label') || '').toLowerCase();
            const rel = (a.getAttribute('rel') || '').toLowerCase();
            const href = a.getAttribute('href') || '';
            return rel.includes('next') || rel.includes('prev') ||
                /^(next|prev|previous|weiter|zurück|vorherige|nächste|[0-9]+)$/.test(text) ||
                label.includes('next') || label.includes('previous') ||
                label.includes('weiter') || label.includes('zurück') ||
                /([?&](page|paged|p)=\d+|\/page\/\d+\/?)/i.test(href);
        }).length;

        // Word count (approximate)
        const text = document.body ? document.body.innerText : '';
        result.wordCount = text.split(/\s+/).filter(w => w.length > 0).length;
        result.textExcerpt = text.slice(0, 4000);

        // Links
        const links = document.querySelectorAll('a[href]');
        let internal = 0, external = 0;
        const currentHost = window.location.host;
        let dofollow = 0, nofollow = 0;

        const internalTargets = [];
        const internalLinkCheckTargets = [];
        links.forEach(a => {
            try {
                const rel = (a.getAttribute('rel') || '').toLowerCase().split(/\s+/);
                const isNofollow = rel.includes('nofollow') || rel.includes('ugc') || rel.includes('sponsored');
                if (isNofollow) { nofollow++; } else { dofollow++; }

                const href = a.getAttribute('href');
                if (!href || /^(mailto|tel|javascript):/i.test(href)) return;
                if (href.startsWith('#')) {
                    internal++;
                    return;
                }
                const linkUrl = new URL(href, window.location.href);
                if (linkUrl.protocol !== 'http:' && linkUrl.protocol !== 'https:') return;
                const isInternal = linkUrl.host === currentHost;
                if (isInternal) {
                    internal++;
                    linkUrl.hash = '';
                    internalLinkCheckTargets.push(linkUrl.href);
                    internalTargets.push(linkUrl.pathname);
                } else {
                    external++;
                }
            } catch (e) {}
        });

        result.internalLinks = internal;
        result.externalLinks = external;
        result.dofollowLinks = dofollow;
        result.nofollowLinks = nofollow;
        result.internalLinkTargets = internalTargets.slice(0, 500);
        result.internalLinkCheckTargets = Array.from(new Set(internalLinkCheckTargets)).slice(0, 50);

        // Crawl budget: internal links with query parameters
        let internalWithQP = 0;
        links.forEach(a => {
            try {
                const href = a.getAttribute('href');
                if (!href || !href.includes('?')) return;
                if (href.startsWith('/')) { internalWithQP++; return; }
                if (href.startsWith('http') && new URL(href).host === currentHost) internalWithQP++;
            } catch(e) {}
        });
        result.internalLinksWithQueryParams = internalWithQP;

        result.stylesheetUrls = Array.from(
            document.querySelectorAll('link[rel=\"stylesheet\"][href]'),
            el => el.href
        );
        result.scriptUrls = Array.from(document.querySelectorAll('script[src]'), el => el.src);
        result.resourceUrls = performance
            .getEntriesByType('resource')
            .map(entry => entry.name)
            .filter(Boolean);
        result.mixedContentResources = [];
        if (window.location.protocol === 'https:') {
            const pushMixed = (url, type, blockable) => {
                try {
                    const resolved = new URL(url, window.location.href);
                    if (resolved.protocol === 'http:') {
                        result.mixedContentResources.push({
                            url: resolved.href,
                            type,
                            blockable
                        });
                    }
                } catch(e) {}
            };
            document.querySelectorAll('script[src]').forEach(el => pushMixed(el.getAttribute('src'), 'script', true));
            document.querySelectorAll('link[rel="stylesheet"][href]').forEach(el => pushMixed(el.getAttribute('href'), 'stylesheet', true));
            document.querySelectorAll('iframe[src], object[data], embed[src]').forEach(el => pushMixed(el.getAttribute('src') || el.getAttribute('data'), 'frame', true));
            document.querySelectorAll('img[src], source[src], video[src], audio[src]').forEach(el => pushMixed(el.getAttribute('src'), 'media', false));
            performance.getEntriesByType('resource').forEach(entry => {
                if (entry.name && entry.name.startsWith('http://')) {
                    const type = entry.initiatorType || 'resource';
                    pushMixed(entry.name, type, ['script', 'link', 'css', 'xmlhttprequest', 'fetch', 'iframe'].includes(type));
                }
            });
        }
        result.cookieNames = document.cookie
            .split(';')
            .map(part => part.trim().split('=')[0])
            .filter(Boolean);
        result.hasZarazGlobal = typeof window.zaraz !== 'undefined';

        // Favicon
        const faviconEl = document.querySelector(
            'link[rel="icon"], link[rel="shortcut icon"], link[rel="apple-touch-icon"]'
        );
        result.hasFavicon = !!faviconEl;

        const manifestEl = document.querySelector('link[rel~="manifest"][href]');
        result.webManifestUrl = manifestEl ? manifestEl.href : null;
        const themeColorEl = document.querySelector('meta[name="theme-color"]');
        result.themeColor = themeColorEl ? themeColorEl.getAttribute('content') : null;
        result.serviceWorkerSupported = 'serviceWorker' in navigator;
        result.serviceWorkerControlled = !!(navigator.serviceWorker && navigator.serviceWorker.controller);

        const sensitiveNamePattern = /(pass|pwd|credit|card|cc-|cc_|cvc|cvv|iban|account|konto|login|email|mail|tel|phone|token|secret)/i;
        const isSensitiveInput = input => {
            const type = (input.getAttribute('type') || 'text').toLowerCase();
            const name = [
                input.getAttribute('name'),
                input.getAttribute('id'),
                input.getAttribute('autocomplete'),
                input.getAttribute('placeholder'),
                input.getAttribute('aria-label')
            ].filter(Boolean).join(' ');
            return ['password', 'email', 'tel'].includes(type) ||
                ['cc-number', 'cc-csc', 'cc-exp', 'one-time-code'].includes((input.getAttribute('autocomplete') || '').toLowerCase()) ||
                sensitiveNamePattern.test(name);
        };
        result.formSecurity = {
            sensitiveForms: 0,
            insecureTransportForms: 0,
            getSensitiveForms: 0,
            passwordAutocompleteIssues: 0
        };
        document.querySelectorAll('form').forEach(form => {
            const inputs = Array.from(form.querySelectorAll('input, textarea, select'));
            const sensitiveInputs = inputs.filter(isSensitiveInput);
            const passwordInputs = inputs.filter(input => (input.getAttribute('type') || '').toLowerCase() === 'password');
            if (sensitiveInputs.length === 0 && passwordInputs.length === 0) return;

            result.formSecurity.sensitiveForms++;

            const method = (form.getAttribute('method') || 'get').toLowerCase();
            if (method === 'get') {
                result.formSecurity.getSensitiveForms++;
            }

            try {
                const action = form.getAttribute('action') || window.location.href;
                const target = new URL(action, window.location.href);
                if (target.protocol === 'http:') {
                    result.formSecurity.insecureTransportForms++;
                }
            } catch(e) {
                if (window.location.protocol === 'http:') {
                    result.formSecurity.insecureTransportForms++;
                }
            }

            passwordInputs.forEach(input => {
                const autocomplete = (input.getAttribute('autocomplete') || '').trim().toLowerCase();
                if (autocomplete === 'off' || (autocomplete && !['current-password', 'new-password', 'one-time-code'].includes(autocomplete))) {
                    result.formSecurity.passwordAutocompleteIssues++;
                }
            });
        });

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Technical SEO analysis failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let canonical_url = parsed["canonical"].as_str().map(String::from);
    let lang = parsed["lang"].as_str().map(String::from);
    let robots_meta = parsed["robots"].as_str().map(String::from);

    let hreflang: Vec<HreflangTag> = parsed["hreflang"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(HreflangTag {
                        lang: v["lang"].as_str()?.to_string(),
                        url: v["url"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let hreflang_has_x_default = parsed["hreflangHasXDefault"].as_bool().unwrap_or(false);
    let hreflang_missing_self_reference = !hreflang.is_empty()
        && !parsed["hreflangHasSelfReference"]
            .as_bool()
            .unwrap_or(false);
    let pagination_prev = parsed["paginationPrev"].as_str().map(String::from);
    let pagination_next = parsed["paginationNext"].as_str().map(String::from);
    let pagination_link_count = parsed["paginationLinkCount"].as_u64().unwrap_or(0) as u32;
    let pagination_anchor_count = parsed["paginationAnchorCount"].as_u64().unwrap_or(0) as u32;
    let pagination_detected =
        detect_pagination(url, pagination_link_count, pagination_anchor_count);
    let internal_links_with_query_params =
        parsed["internalLinksWithQueryParams"].as_u64().unwrap_or(0) as u32;

    let word_count = parsed["wordCount"].as_u64().unwrap_or(0) as u32;
    let internal_links = parsed["internalLinks"].as_u64().unwrap_or(0) as u32;
    let external_links = parsed["externalLinks"].as_u64().unwrap_or(0) as u32;
    let dofollow_links = parsed["dofollowLinks"].as_u64().unwrap_or(0) as u32;
    let nofollow_links = parsed["nofollowLinks"].as_u64().unwrap_or(0) as u32;
    let internal_link_targets = parse_string_array(&parsed["internalLinkTargets"]);
    let internal_link_check_targets = parse_string_array(&parsed["internalLinkCheckTargets"]);
    let text_excerpt = parsed["textExcerpt"].as_str().unwrap_or("").to_string();
    let stylesheet_urls = parse_string_array(&parsed["stylesheetUrls"]);
    let script_urls = parse_string_array(&parsed["scriptUrls"]);
    let resource_urls = parse_string_array(&parsed["resourceUrls"]);
    let mixed_content = parse_mixed_content_resources(&parsed["mixedContentResources"]);
    let cookie_names = parse_string_array(&parsed["cookieNames"]);
    let has_zaraz_global = parsed["hasZarazGlobal"].as_bool().unwrap_or(false);
    let has_favicon = parsed["hasFavicon"].as_bool().unwrap_or(false);
    let web_manifest_url = parsed["webManifestUrl"].as_str().map(String::from);
    let theme_color = parsed["themeColor"].as_str().map(String::from);
    let service_worker_supported = parsed["serviceWorkerSupported"].as_bool().unwrap_or(false);
    let service_worker_controlled = parsed["serviceWorkerControlled"].as_bool().unwrap_or(false);
    let form_security = parse_form_security_analysis(&parsed["formSecurity"]);

    let google_fonts_sources =
        collect_google_fonts_sources(&stylesheet_urls, &resource_urls, &script_urls);
    let tracking_cookies: Vec<TrackingCookie> = cookie_names
        .iter()
        .filter_map(|name| classify_tracking_cookie(name))
        .collect();
    let tracking_signals =
        collect_tracking_signals(&script_urls, &resource_urls, &tracking_cookies);
    let zaraz = detect_zaraz(
        &script_urls,
        &resource_urls,
        &tracking_cookies,
        has_zaraz_global,
    );
    let broken_links = check_broken_internal_links(url, &internal_link_check_targets).await;
    let service_worker_registrations = read_service_worker_registration_count(page).await;
    let pwa = analyze_pwa(
        web_manifest_url,
        theme_color,
        service_worker_supported,
        service_worker_controlled,
        service_worker_registrations,
    )
    .await;

    // Generate issues
    let mut issues = Vec::new();

    if !https {
        issues.push(TechnicalIssue {
            issue_type: "no_https".to_string(),
            message: "Page is not served over HTTPS".to_string(),
            severity: Severity::High,
        });
    }

    if canonical_url.is_none() {
        issues.push(TechnicalIssue {
            issue_type: "no_canonical".to_string(),
            message: "Missing canonical URL".to_string(),
            severity: Severity::Medium,
        });
    }

    if lang.is_none() {
        issues.push(TechnicalIssue {
            issue_type: "no_lang".to_string(),
            message: "Missing lang attribute on html element".to_string(),
            severity: Severity::Medium,
        });
    }

    if word_count < 300 {
        issues.push(TechnicalIssue {
            issue_type: "thin_content".to_string(),
            message: format!(
                "Page has thin content ({} words, recommended: 300+)",
                word_count
            ),
            severity: Severity::Medium,
        });
    }

    if internal_links == 0 {
        issues.push(TechnicalIssue {
            issue_type: "no_internal_links".to_string(),
            message: "Page has no internal links".to_string(),
            severity: Severity::Medium,
        });
    }

    if !google_fonts_sources.is_empty() {
        issues.push(TechnicalIssue {
            issue_type: "remote_google_fonts".to_string(),
            message: if en {
                "Externally hosted Google Fonts detected".to_string()
            } else {
                "Extern gehostete Google Fonts erkannt".to_string()
            },
            severity: Severity::Low,
        });
    }

    if let Some(ref robots) = robots_meta {
        if robots.to_lowercase().contains("noindex") {
            issues.push(TechnicalIssue {
                issue_type: "noindex".to_string(),
                message: if en {
                    "Page has a noindex directive — it is not indexed by search engines".to_string()
                } else {
                    "Seite hat noindex-Direktive — wird von Suchmaschinen nicht indexiert"
                        .to_string()
                },
                severity: Severity::High,
            });
        }
    }

    if !hreflang.is_empty() && !hreflang_has_x_default {
        issues.push(TechnicalIssue {
            issue_type: "hreflang_no_x_default".to_string(),
            message: if en {
                "hreflang annotations without an x-default entry".to_string()
            } else {
                "hreflang-Annotierungen ohne x-default-Eintrag".to_string()
            },
            severity: Severity::Low,
        });
    }

    if hreflang_missing_self_reference {
        issues.push(TechnicalIssue {
            issue_type: "hreflang_no_self_reference".to_string(),
            message: if en {
                "hreflang set contains no self-reference for the current page".to_string()
            } else {
                "hreflang-Set enthält keine Self-Reference für die aktuelle Seite".to_string()
            },
            severity: Severity::Low,
        });
    }

    if internal_links_with_query_params > 0 {
        issues.push(TechnicalIssue {
            issue_type: "internal_links_with_query_params".to_string(),
            message: if en {
                format!(
                    "{} internal link(s) with query parameters — can dilute crawl budget",
                    internal_links_with_query_params
                )
            } else {
                format!(
                    "{} interne Link(s) mit Query-Parametern — können Crawl-Budget verwässern",
                    internal_links_with_query_params
                )
            },
            severity: Severity::Low,
        });
    }

    collect_pagination_issues(
        url,
        pagination_detected,
        pagination_prev.as_deref(),
        pagination_next.as_deref(),
        &mut issues,
    );

    if !broken_links.is_empty() {
        issues.push(TechnicalIssue {
            issue_type: "broken_internal_links".to_string(),
            message: format!(
                "{} broken internal link(s) detected: {}",
                broken_links.len(),
                broken_links
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            severity: Severity::High,
        });
    }

    if !mixed_content.is_empty() {
        let blockable_count = mixed_content
            .iter()
            .filter(|resource| resource.blockable)
            .count();
        issues.push(TechnicalIssue {
            issue_type: "mixed_content".to_string(),
            message: format!(
                "{} HTTP subresource(s) referenced from HTTPS page ({} blockable): {}",
                mixed_content.len(),
                blockable_count,
                mixed_content
                    .iter()
                    .take(3)
                    .map(|resource| resource.url.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            severity: if blockable_count > 0 {
                Severity::High
            } else {
                Severity::Medium
            },
        });
    }

    collect_pwa_issues(&pwa, &mut issues);
    collect_form_security_issues(&form_security, &mut issues);

    info!(
        "Technical SEO: HTTPS={}, canonical={}, lang={}, words={}, favicon={}, google_fonts={}, tracking_cookies={}, zaraz={}, pagination={}",
        https,
        canonical_url.is_some(),
        lang.is_some(),
        word_count,
        has_favicon,
        !google_fonts_sources.is_empty(),
        tracking_cookies.len(),
        zaraz.detected,
        pagination_detected
    );

    // www / non-www redirect check
    let www_redirect = check_www_redirect(url, canonical_url.as_deref()).await;
    if let Some(ref check) = www_redirect {
        if !check.redirects_to_primary && check.alternative_status == 200 {
            let canonical_note = match check.canonical_matches_primary {
                Some(true) => " Canonical tag is set correctly, but a 301 is still recommended — canonicals are hints, not directives.",
                Some(false) => " Additionally, canonical does not point to the primary domain.",
                None => " No canonical tag found.",
            };
            issues.push(TechnicalIssue {
                issue_type: "www_no_redirect".to_string(),
                message: format!(
                    "Both {} and {} serve content (HTTP 200). Add a 301 redirect from the alternative to the primary to prevent duplicate content.{}",
                    check.primary, check.alternative, canonical_note
                ),
                severity: Severity::Medium,
            });
        } else if !check.redirects_to_primary && check.alternative_status == 0 {
            // alternative unreachable — no issue, just informational
        }
        if let Some(false) = check.canonical_matches_primary {
            issues.push(TechnicalIssue {
                issue_type: "canonical_domain_mismatch".to_string(),
                message: format!(
                    "Canonical URL points to a different domain variant than the audited URL ({} vs canonical).",
                    check.primary
                ),
                severity: Severity::Medium,
            });
        }
    }

    Ok(TechnicalSeo {
        https,
        has_canonical: canonical_url.is_some(),
        canonical_url,
        has_lang: lang.is_some(),
        lang,
        has_robots_meta: robots_meta.is_some(),
        robots_meta,
        has_hreflang: !hreflang.is_empty(),
        hreflang,
        hreflang_has_x_default,
        hreflang_missing_self_reference,
        pagination_detected,
        pagination_prev,
        pagination_next,
        internal_links_with_query_params,
        word_count,
        internal_links,
        external_links,
        dofollow_links,
        nofollow_links,
        internal_link_targets,
        broken_links,
        mixed_content,
        pwa,
        form_security,
        text_excerpt,
        uses_remote_google_fonts: !google_fonts_sources.is_empty(),
        google_fonts_sources,
        tracking_cookies,
        tracking_signals,
        zaraz,
        has_favicon,
        www_redirect,
        issues,
    })
}

/// Check whether the www ↔ non-www counterpart of `url` redirects to `url`
/// or serves its own content (duplicate).
async fn check_www_redirect(url: &str, canonical: Option<&str>) -> Option<WwwRedirectCheck> {
    // Build the alternative URL variant
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str()?;

    let (primary_host, alt_host) = if let Some(stripped) = host.strip_prefix("www.") {
        (host.to_string(), stripped.to_string())
    } else {
        (host.to_string(), format!("www.{}", host))
    };

    // Only makes sense for plain domains; skip IPs, localhost, and subdomains
    if primary_host.parse::<std::net::IpAddr>().is_ok() || primary_host == "localhost" {
        return None;
    }
    // Determine the bare host (without www prefix) and skip if it's a subdomain
    let bare_host = host.strip_prefix("www.").unwrap_or(host);
    if bare_host.split('.').count() > 2 {
        return None;
    }

    let mut alt_url = parsed.clone();
    alt_url.set_host(Some(&alt_host)).ok()?;
    let alternative = alt_url.as_str().to_string();
    let primary = url.to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Mozilla/5.0 (compatible; AuditMySite/1.0)")
        .build()
        .ok()?;

    let response = match client.head(&alternative).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("www-redirect check failed for {}: {}", alternative, e);
            return Some(WwwRedirectCheck {
                primary,
                alternative,
                alternative_status: 0,
                redirects_to_primary: false,
                canonical_matches_primary: canonical_matches(&primary_host, canonical),
            });
        }
    };

    let status = response.status().as_u16();
    let redirects_to_primary = if matches!(status, 301 | 302 | 307 | 308) {
        response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(|loc| {
                // Location may be absolute or relative; check if it resolves to primary host
                if let Ok(loc_url) = url::Url::parse(loc) {
                    loc_url.host_str() == Some(&primary_host)
                } else {
                    // relative redirect — stays on same host
                    false
                }
            })
            .unwrap_or(false)
    } else {
        false
    };

    Some(WwwRedirectCheck {
        primary,
        alternative,
        alternative_status: status,
        redirects_to_primary,
        canonical_matches_primary: canonical_matches(&primary_host, canonical),
    })
}

/// Returns Some(true) if canonical points to primary_host, Some(false) if it points elsewhere,
/// None if no canonical.
fn canonical_matches(primary_host: &str, canonical: Option<&str>) -> Option<bool> {
    let canon = canonical?;
    let parsed = url::Url::parse(canon).ok()?;
    Some(parsed.host_str() == Some(primary_host))
}

fn parse_string_array(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_mixed_content_resources(value: &serde_json::Value) -> Vec<MixedContentResource> {
    let mut seen = BTreeSet::new();
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let url = item["url"].as_str()?.trim();
                    if url.is_empty() || !seen.insert(url.to_string()) {
                        return None;
                    }
                    Some(MixedContentResource {
                        url: url.to_string(),
                        resource_type: item["type"]
                            .as_str()
                            .unwrap_or("resource")
                            .trim()
                            .to_string(),
                        blockable: item["blockable"].as_bool().unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_form_security_analysis(value: &serde_json::Value) -> FormSecurityAnalysis {
    FormSecurityAnalysis {
        sensitive_forms: value["sensitiveForms"].as_u64().unwrap_or(0) as u32,
        insecure_transport_forms: value["insecureTransportForms"].as_u64().unwrap_or(0) as u32,
        get_sensitive_forms: value["getSensitiveForms"].as_u64().unwrap_or(0) as u32,
        password_autocomplete_issues: value["passwordAutocompleteIssues"].as_u64().unwrap_or(0)
            as u32,
    }
}

async fn analyze_pwa(
    manifest_url: Option<String>,
    theme_color: Option<String>,
    service_worker_supported: bool,
    service_worker_controlled: bool,
    service_worker_registrations: u32,
) -> PwaAnalysis {
    let Some(ref manifest_url) = manifest_url else {
        return PwaAnalysis {
            manifest_url,
            theme_color,
            service_worker_supported,
            service_worker_controlled,
            service_worker_registrations,
            ..PwaAnalysis::default()
        };
    };

    let manifest = fetch_web_manifest(manifest_url).await;
    let Some(manifest) = manifest else {
        return PwaAnalysis {
            manifest_url: Some(manifest_url.clone()),
            theme_color,
            service_worker_supported,
            service_worker_controlled,
            service_worker_registrations,
            ..PwaAnalysis::default()
        };
    };

    let icons = manifest["icons"].as_array().cloned().unwrap_or_default();
    let has_maskable_icon = icons.iter().any(|icon| {
        icon["purpose"]
            .as_str()
            .map(|purpose| purpose.split_whitespace().any(|part| part == "maskable"))
            .unwrap_or(false)
    });
    let manifest_theme_color = manifest["theme_color"].as_str().map(String::from);
    let display = manifest["display"].as_str().map(String::from);
    let start_url = manifest["start_url"].as_str().map(String::from);
    let name = manifest["name"].as_str().map(String::from);
    let short_name = manifest["short_name"].as_str().map(String::from);
    let manifest_valid = (name.is_some() || short_name.is_some())
        && start_url.is_some()
        && display.is_some()
        && !icons.is_empty()
        && (manifest_theme_color.is_some() || theme_color.is_some());

    PwaAnalysis {
        manifest_url: Some(manifest_url.clone()),
        manifest_valid,
        name,
        short_name,
        start_url,
        display,
        theme_color: manifest_theme_color.or(theme_color),
        icons_count: icons.len() as u32,
        has_maskable_icon,
        service_worker_supported,
        service_worker_controlled,
        service_worker_registrations,
    }
}

async fn fetch_web_manifest(manifest_url: &str) -> Option<serde_json::Value> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(Policy::limited(4))
        .build()
        .ok()?;
    let response = client
        .get(manifest_url)
        .header("User-Agent", "auditmysite-pwa-checker/1.0")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json::<serde_json::Value>().await.ok()
}

async fn read_service_worker_registration_count(page: &Page) -> u32 {
    let expression = r#"
        (async () => {
            if (!('serviceWorker' in navigator) || !navigator.serviceWorker.getRegistrations) {
                return 0;
            }
            try {
                const registrations = await navigator.serviceWorker.getRegistrations();
                return registrations.length || 0;
            } catch (e) {
                return 0;
            }
        })()
    "#;
    let Ok(params) = EvaluateParams::builder()
        .expression(expression.to_string())
        .await_promise(true)
        .build()
    else {
        return 0;
    };
    page.execute(params)
        .await
        .ok()
        .and_then(|result| result.result.result.value)
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as u32
}

fn collect_pwa_issues(pwa: &PwaAnalysis, issues: &mut Vec<TechnicalIssue>) {
    if pwa.manifest_url.is_none() {
        issues.push(TechnicalIssue {
            issue_type: "pwa_missing_manifest".to_string(),
            message: "Missing web app manifest".to_string(),
            severity: Severity::Low,
        });
        return;
    }

    if !pwa.manifest_valid {
        issues.push(TechnicalIssue {
            issue_type: "pwa_invalid_manifest".to_string(),
            message: "Web app manifest is missing required PWA fields".to_string(),
            severity: Severity::Medium,
        });
    }

    if pwa.service_worker_supported && pwa.service_worker_registrations == 0 {
        issues.push(TechnicalIssue {
            issue_type: "pwa_missing_service_worker".to_string(),
            message: "No service worker registration detected".to_string(),
            severity: Severity::Low,
        });
    }
}

fn collect_form_security_issues(
    form_security: &FormSecurityAnalysis,
    issues: &mut Vec<TechnicalIssue>,
) {
    if form_security.insecure_transport_forms > 0 {
        issues.push(TechnicalIssue {
            issue_type: "form_insecure_transport".to_string(),
            message: format!(
                "{} sensitive form(s) submit to HTTP or are hosted on HTTP",
                form_security.insecure_transport_forms
            ),
            severity: Severity::High,
        });
    }

    if form_security.get_sensitive_forms > 0 {
        issues.push(TechnicalIssue {
            issue_type: "form_sensitive_get".to_string(),
            message: format!(
                "{} sensitive form(s) use GET and can leak values into URLs and logs",
                form_security.get_sensitive_forms
            ),
            severity: Severity::High,
        });
    }

    if form_security.password_autocomplete_issues > 0 {
        issues.push(TechnicalIssue {
            issue_type: "form_password_autocomplete".to_string(),
            message: format!(
                "{} password field(s) have autocomplete disabled or misconfigured",
                form_security.password_autocomplete_issues
            ),
            severity: Severity::Medium,
        });
    }
}

async fn check_broken_internal_links(source_url: &str, targets: &[String]) -> Vec<String> {
    let Some(source_origin) = url_origin(source_url) else {
        return Vec::new();
    };
    let client = Client::builder()
        .redirect(Policy::limited(6))
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| Client::new());

    let mut unique = BTreeSet::new();
    for target in targets {
        if url_origin(target).as_deref() == Some(source_origin.as_str()) {
            unique.insert(target.clone());
        }
    }

    stream::iter(unique.into_iter().take(50))
        .map(|target| {
            let client = client.clone();
            async move {
                check_link_status(&client, &target)
                    .await
                    .map(|reason| format!("{target} ({reason})"))
            }
        })
        .buffer_unordered(8)
        .filter_map(|result| async move { result })
        .collect()
        .await
}

async fn check_link_status(client: &Client, target: &str) -> Option<String> {
    match client
        .head(target)
        .header("User-Agent", "auditmysite-link-checker/1.0")
        .send()
        .await
    {
        Ok(response)
            if response.status() == StatusCode::METHOD_NOT_ALLOWED
                || response.status() == StatusCode::NOT_IMPLEMENTED =>
        {
            check_link_status_with_get(client, target).await
        }
        Ok(response) if response.status().as_u16() >= 400 => {
            Some(format!("HTTP {}", response.status().as_u16()))
        }
        Ok(_) => None,
        Err(head_error) => match check_link_status_with_get(client, target).await {
            Some(reason) => Some(reason),
            None => Some(head_error.to_string()),
        },
    }
}

async fn check_link_status_with_get(client: &Client, target: &str) -> Option<String> {
    match client
        .get(target)
        .header("User-Agent", "auditmysite-link-checker/1.0")
        .send()
        .await
    {
        Ok(response) if response.status().as_u16() >= 400 => {
            Some(format!("HTTP {}", response.status().as_u16()))
        }
        Ok(_) => None,
        Err(error) => Some(error.to_string()),
    }
}

fn url_origin(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    let port = parsed
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    Some(format!("{}://{}{}", parsed.scheme(), host, port))
}

fn detect_pagination(
    url: &str,
    pagination_container_count: u32,
    pagination_anchor_count: u32,
) -> bool {
    if pagination_container_count > 0 || pagination_anchor_count >= 2 {
        return true;
    }

    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };

    let has_page_query = parsed.query_pairs().any(|(key, value)| {
        matches!(key.as_ref(), "page" | "paged" | "p")
            && value.parse::<u32>().is_ok_and(|page| page > 1)
    });
    let has_page_path = parsed
        .path_segments()
        .map(|segments| {
            let items: Vec<_> = segments.collect();
            items
                .windows(2)
                .any(|pair| pair[0].eq_ignore_ascii_case("page") && pair[1].parse::<u32>().is_ok())
        })
        .unwrap_or(false);

    has_page_query || has_page_path
}

fn collect_pagination_issues(
    url: &str,
    pagination_detected: bool,
    prev: Option<&str>,
    next: Option<&str>,
    issues: &mut Vec<TechnicalIssue>,
) {
    if !pagination_detected {
        return;
    }

    if prev.is_none() && next.is_none() {
        issues.push(TechnicalIssue {
            issue_type: "pagination_missing_rel_links".to_string(),
            message: "Paginated page detected without rel=\"prev\" or rel=\"next\" link markup"
                .to_string(),
            severity: Severity::Low,
        });
        return;
    }

    if prev.is_some_and(|href| same_url(href, url)) || next.is_some_and(|href| same_url(href, url))
    {
        issues.push(TechnicalIssue {
            issue_type: "pagination_self_referential_rel_link".to_string(),
            message: "Pagination rel link points to the current page instead of an adjacent page"
                .to_string(),
            severity: Severity::Low,
        });
    }

    if let (Some(prev), Some(next)) = (prev, next) {
        if same_url(prev, next) {
            issues.push(TechnicalIssue {
                issue_type: "pagination_duplicate_prev_next".to_string(),
                message: "Pagination rel=\"prev\" and rel=\"next\" point to the same URL"
                    .to_string(),
                severity: Severity::Low,
            });
        }
    }
}

fn same_url(a: &str, b: &str) -> bool {
    let normalize = |input: &str| {
        url::Url::parse(input)
            .map(|mut parsed| {
                parsed.set_fragment(None);
                parsed.to_string().trim_end_matches('/').to_string()
            })
            .unwrap_or_else(|_| input.trim_end_matches('/').to_string())
    };
    normalize(a) == normalize(b)
}

fn collect_google_fonts_sources(
    stylesheet_urls: &[String],
    resource_urls: &[String],
    script_urls: &[String],
) -> Vec<String> {
    let mut urls = BTreeSet::new();
    for url in stylesheet_urls
        .iter()
        .chain(resource_urls.iter())
        .chain(script_urls.iter())
    {
        if is_google_fonts_url(url) {
            urls.insert(url.to_string());
        }
    }
    urls.into_iter().collect()
}

fn is_google_fonts_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("fonts.googleapis.com") || lower.contains("fonts.gstatic.com")
}

fn classify_tracking_cookie(name: &str) -> Option<TrackingCookie> {
    let lower = name.trim().to_ascii_lowercase();
    let (scope, provider) = if lower.starts_with("_ga")
        || lower == "_gid"
        || lower == "_gat"
        || lower.starts_with("_gcl")
    {
        ("extern", "Google")
    } else if lower == "_fbp" || lower == "_fbc" || lower == "fr" {
        ("extern", "Meta")
    } else if lower.starts_with("_hj") {
        ("extern", "Hotjar")
    } else if lower == "hubspotutk" {
        ("extern", "HubSpot")
    } else if lower.starts_with("_pk_") || lower == "mtm_cookie_consent" {
        ("lokal", "Matomo")
    } else if lower.contains("zaraz") {
        ("lokal", "Cloudflare Zaraz")
    } else {
        return None;
    };

    Some(TrackingCookie {
        name: name.to_string(),
        scope: scope.to_string(),
        provider: provider.to_string(),
    })
}

fn collect_tracking_signals(
    script_urls: &[String],
    resource_urls: &[String],
    tracking_cookies: &[TrackingCookie],
) -> Vec<String> {
    let mut signals = BTreeSet::new();
    for url in script_urls.iter().chain(resource_urls.iter()) {
        if let Some(signal) = classify_tracking_url(url) {
            signals.insert(signal.to_string());
        }
    }
    for cookie in tracking_cookies {
        signals.insert(format!("{}-Cookie: {}", cookie.provider, cookie.name));
    }
    signals.into_iter().collect()
}

fn classify_tracking_url(url: &str) -> Option<&'static str> {
    let lower = url.to_ascii_lowercase();
    if lower.contains("/cdn-cgi/zaraz/") || lower.contains("zaraz") {
        Some("Cloudflare Zaraz")
    } else if lower.contains("googletagmanager.com")
        || lower.contains("google-analytics.com")
        || lower.contains("analytics.google.com")
    {
        Some("Google Analytics / Tag Manager")
    } else if lower.contains("connect.facebook.net") || lower.contains("facebook.com/tr") {
        Some("Meta Pixel")
    } else if lower.contains("static.cloudflareinsights.com") {
        Some("Cloudflare Web Analytics")
    } else if lower.contains("plausible.io") {
        Some("Plausible")
    } else if lower.contains("matomo") || lower.contains("/matomo.") || lower.contains("piwik") {
        Some("Matomo")
    } else {
        None
    }
}

fn detect_zaraz(
    script_urls: &[String],
    resource_urls: &[String],
    tracking_cookies: &[TrackingCookie],
    has_zaraz_global: bool,
) -> ZarazDetection {
    let mut signals = BTreeSet::new();
    if has_zaraz_global {
        signals.insert("window.zaraz".to_string());
    }
    for url in script_urls.iter().chain(resource_urls.iter()) {
        if url.to_ascii_lowercase().contains("/cdn-cgi/zaraz/")
            || url.to_ascii_lowercase().contains("zaraz")
        {
            signals.insert(url.clone());
        }
    }
    for cookie in tracking_cookies {
        if cookie.provider == "Cloudflare Zaraz" {
            signals.insert(format!("Cookie: {}", cookie.name));
        }
    }
    ZarazDetection {
        detected: !signals.is_empty(),
        signals: signals.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technical_seo_default() {
        let tech = TechnicalSeo::default();
        assert!(!tech.https);
        assert!(!tech.has_canonical);
    }

    #[test]
    fn test_google_fonts_detection() {
        assert!(is_google_fonts_url(
            "https://fonts.googleapis.com/css2?family=Inter:wght@400;700&display=swap"
        ));
        assert!(is_google_fonts_url(
            "https://fonts.gstatic.com/s/inter/v20/font.woff2"
        ));
        assert!(!is_google_fonts_url("https://example.com/app.css"));
    }

    #[test]
    fn test_tracking_cookie_classification() {
        let google = classify_tracking_cookie("_ga").unwrap();
        assert_eq!(google.scope, "extern");
        assert_eq!(google.provider, "Google");

        let zaraz = classify_tracking_cookie("zarazExample").unwrap();
        assert_eq!(zaraz.scope, "lokal");
        assert_eq!(zaraz.provider, "Cloudflare Zaraz");

        assert!(classify_tracking_cookie("sessionid").is_none());
    }

    #[test]
    fn test_zaraz_detection() {
        let zaraz = detect_zaraz(
            &["https://www.example.com/cdn-cgi/zaraz/s.js".to_string()],
            &[],
            &[],
            false,
        );
        assert!(zaraz.detected);
        assert_eq!(zaraz.signals.len(), 1);
    }

    #[test]
    fn test_url_origin_preserves_scheme_host_and_port() {
        assert_eq!(
            url_origin("https://example.com:8443/path?x=1"),
            Some("https://example.com:8443".to_string())
        );
        assert_eq!(
            url_origin("http://localhost:3000/about"),
            Some("http://localhost:3000".to_string())
        );
        assert_eq!(url_origin("/relative"), None);
    }

    #[test]
    fn test_parse_mixed_content_resources_deduplicates() {
        let value = serde_json::json!([
            {"url": "http://cdn.example.com/app.js", "type": "script", "blockable": true},
            {"url": "http://cdn.example.com/app.js", "type": "script", "blockable": true},
            {"url": "http://cdn.example.com/photo.jpg", "type": "media", "blockable": false}
        ]);

        let resources = parse_mixed_content_resources(&value);

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].resource_type, "script");
        assert!(resources[0].blockable);
        assert!(!resources[1].blockable);
    }

    #[test]
    fn test_collect_pwa_issues_missing_manifest() {
        let mut issues = Vec::new();
        collect_pwa_issues(&PwaAnalysis::default(), &mut issues);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "pwa_missing_manifest");
        assert_eq!(issues[0].severity, Severity::Low);
    }

    #[test]
    fn test_collect_pwa_issues_invalid_manifest_and_no_service_worker() {
        let pwa = PwaAnalysis {
            manifest_url: Some("https://example.com/manifest.webmanifest".to_string()),
            manifest_valid: false,
            service_worker_supported: true,
            service_worker_registrations: 0,
            ..PwaAnalysis::default()
        };
        let mut issues = Vec::new();
        collect_pwa_issues(&pwa, &mut issues);

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "pwa_invalid_manifest"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "pwa_missing_service_worker"));
    }

    #[test]
    fn test_collect_form_security_issues_flags_sensitive_get_and_http() {
        let form_security = FormSecurityAnalysis {
            sensitive_forms: 2,
            insecure_transport_forms: 1,
            get_sensitive_forms: 1,
            password_autocomplete_issues: 0,
        };
        let mut issues = Vec::new();

        collect_form_security_issues(&form_security, &mut issues);

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "form_insecure_transport"
                && issue.severity == Severity::High));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "form_sensitive_get"
                && issue.severity == Severity::High));
    }

    #[test]
    fn test_collect_form_security_issues_flags_password_autocomplete() {
        let form_security = FormSecurityAnalysis {
            password_autocomplete_issues: 2,
            ..FormSecurityAnalysis::default()
        };
        let mut issues = Vec::new();

        collect_form_security_issues(&form_security, &mut issues);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "form_password_autocomplete");
        assert_eq!(issues[0].severity, Severity::Medium);
    }

    #[test]
    fn test_detect_pagination_from_url_patterns() {
        assert!(detect_pagination("https://example.com/blog/page/2/", 0, 0));
        assert!(detect_pagination("https://example.com/blog?page=3", 0, 0));
        assert!(detect_pagination("https://example.com/blog", 1, 0));
        assert!(detect_pagination("https://example.com/blog", 0, 2));
        assert!(!detect_pagination("https://example.com/blog", 0, 1));
        assert!(!detect_pagination("https://example.com/blog?page=1", 0, 0));
    }

    #[test]
    fn test_collect_pagination_issues_missing_rel_links() {
        let mut issues = Vec::new();
        collect_pagination_issues(
            "https://example.com/blog/page/2/",
            true,
            None,
            None,
            &mut issues,
        );

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "pagination_missing_rel_links");
        assert_eq!(issues[0].severity, Severity::Low);
    }

    #[test]
    fn test_collect_pagination_issues_self_reference_and_duplicate() {
        let mut issues = Vec::new();
        collect_pagination_issues(
            "https://example.com/blog/page/2/",
            true,
            Some("https://example.com/blog/page/2/#top"),
            Some("https://example.com/blog/page/2/"),
            &mut issues,
        );

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "pagination_self_referential_rel_link"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "pagination_duplicate_prev_next"));
    }
}
