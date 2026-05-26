//! Technical SEO analysis
//!
//! Checks HTTPS, canonical URLs, language, sitemap, robots.txt.

use chromiumoxide::Page;
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

/// Technical SEO issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIssue {
    pub issue_type: String,
    pub message: String,
    pub severity: Severity,
}

/// Analyze technical SEO aspects
pub async fn analyze_technical_seo(page: &Page, url: &str) -> Result<TechnicalSeo> {
    info!("Analyzing technical SEO...");

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
        links.forEach(a => {
            try {
                const rel = (a.getAttribute('rel') || '').toLowerCase().split(/\s+/);
                const isNofollow = rel.includes('nofollow') || rel.includes('ugc') || rel.includes('sponsored');
                if (isNofollow) { nofollow++; } else { dofollow++; }

                const href = a.getAttribute('href');
                if (href.startsWith('http')) {
                    const linkUrl = new URL(href);
                    if (linkUrl.host === currentHost) {
                        internal++;
                        internalTargets.push(linkUrl.pathname);
                    } else {
                        external++;
                    }
                } else if (href.startsWith('/')) {
                    internal++;
                    internalTargets.push(href.split('?')[0].split('#')[0]);
                } else if (href.startsWith('#')) {
                    internal++;
                }
            } catch (e) {}
        });

        result.internalLinks = internal;
        result.externalLinks = external;
        result.dofollowLinks = dofollow;
        result.nofollowLinks = nofollow;
        result.internalLinkTargets = internalTargets.slice(0, 500);

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
    let internal_links_with_query_params =
        parsed["internalLinksWithQueryParams"].as_u64().unwrap_or(0) as u32;

    let word_count = parsed["wordCount"].as_u64().unwrap_or(0) as u32;
    let internal_links = parsed["internalLinks"].as_u64().unwrap_or(0) as u32;
    let external_links = parsed["externalLinks"].as_u64().unwrap_or(0) as u32;
    let dofollow_links = parsed["dofollowLinks"].as_u64().unwrap_or(0) as u32;
    let nofollow_links = parsed["nofollowLinks"].as_u64().unwrap_or(0) as u32;
    let internal_link_targets = parse_string_array(&parsed["internalLinkTargets"]);
    let text_excerpt = parsed["textExcerpt"].as_str().unwrap_or("").to_string();
    let stylesheet_urls = parse_string_array(&parsed["stylesheetUrls"]);
    let script_urls = parse_string_array(&parsed["scriptUrls"]);
    let resource_urls = parse_string_array(&parsed["resourceUrls"]);
    let cookie_names = parse_string_array(&parsed["cookieNames"]);
    let has_zaraz_global = parsed["hasZarazGlobal"].as_bool().unwrap_or(false);
    let has_favicon = parsed["hasFavicon"].as_bool().unwrap_or(false);

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
            message: "Extern gehostete Google Fonts erkannt".to_string(),
            severity: Severity::Low,
        });
    }

    if let Some(ref robots) = robots_meta {
        if robots.to_lowercase().contains("noindex") {
            issues.push(TechnicalIssue {
                issue_type: "noindex".to_string(),
                message: "Seite hat noindex-Direktive — wird von Suchmaschinen nicht indexiert"
                    .to_string(),
                severity: Severity::High,
            });
        }
    }

    if !hreflang.is_empty() && !hreflang_has_x_default {
        issues.push(TechnicalIssue {
            issue_type: "hreflang_no_x_default".to_string(),
            message: "hreflang-Annotierungen ohne x-default-Eintrag".to_string(),
            severity: Severity::Low,
        });
    }

    if hreflang_missing_self_reference {
        issues.push(TechnicalIssue {
            issue_type: "hreflang_no_self_reference".to_string(),
            message: "hreflang-Set enthält keine Self-Reference für die aktuelle Seite".to_string(),
            severity: Severity::Low,
        });
    }

    if internal_links_with_query_params > 0 {
        issues.push(TechnicalIssue {
            issue_type: "internal_links_with_query_params".to_string(),
            message: format!(
                "{} interne Link(s) mit Query-Parametern — können Crawl-Budget verwässern",
                internal_links_with_query_params
            ),
            severity: Severity::Low,
        });
    }

    info!(
        "Technical SEO: HTTPS={}, canonical={}, lang={}, words={}, favicon={}, google_fonts={}, tracking_cookies={}, zaraz={}",
        https,
        canonical_url.is_some(),
        lang.is_some(),
        word_count,
        has_favicon,
        !google_fonts_sources.is_empty(),
        tracking_cookies.len(),
        zaraz.detected
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
        internal_links_with_query_params,
        word_count,
        internal_links,
        external_links,
        dofollow_links,
        nofollow_links,
        internal_link_targets,
        broken_links: vec![],
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
}
