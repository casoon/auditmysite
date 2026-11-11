//! Technical SEO analysis
//!
//! Checks HTTPS, canonical URLs, language, sitemap, robots.txt.

use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use fluent_bundle::FluentValue;
use futures::stream::{self, StreamExt};
use reqwest::{redirect::Policy, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tracing::{info, warn};

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
    /// Basic AMP conformance signals when AMP markup is present.
    #[serde(default)]
    pub amp: AmpAnalysis,
    /// Sensitive form transport, method, and credential-autocomplete checks.
    #[serde(default)]
    pub form_security: FormSecurityAnalysis,
    /// Visible text excerpt for topic analysis and redundancy checks
    pub text_excerpt: String,
    /// Where `text_excerpt` comes from: `main` (a single main landmark with
    /// at least 200 characters) or `body` (fallback, includes header and
    /// navigation). Empty in reports from before plan 90.
    #[serde(default)]
    pub text_excerpt_source: String,
    /// Uses externally hosted Google Fonts assets
    pub uses_remote_google_fonts: bool,
    /// Matching Google Fonts assets or stylesheets
    pub google_fonts_sources: Vec<String>,
    /// Tracking cookies detected on the page
    pub tracking_cookies: Vec<TrackingCookie>,
    /// Cookie metadata from the browser cookie jar. Values are never
    /// collected. `None` when the cookie jar could not be read.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cookie_inventory: Option<Vec<CookieInventoryItem>>,
    /// localStorage/sessionStorage keys detected on the page (values are never collected).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub storage_items: Vec<StorageItem>,
    /// Tracking providers or signals detected from scripts/resources
    pub tracking_signals: Vec<String>,
    /// Cloudflare Zaraz detected
    pub zaraz: ZarazDetection,
    /// Favicon detected (<link rel="icon"> or apple-touch-icon)
    pub has_favicon: bool,
    /// www / non-www redirect check result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub www_redirect: Option<WwwRedirectCheck>,
    /// og:image is present without an og:image:alt description.
    pub og_image_missing_alt: bool,
    /// twitter:image is present without a twitter:image:alt description.
    pub twitter_image_missing_alt: bool,
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

/// Browser cookie metadata. Values are intentionally omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieInventoryItem {
    pub name: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
    pub session: bool,
    pub third_party: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Browser storage key detected on the page. Values are intentionally omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageItem {
    pub area: String,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
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

/// Basic AMP markup signal analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AmpAnalysis {
    pub detected: bool,
    pub canonical_present: bool,
    pub runtime_script_present: bool,
    pub boilerplate_present: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub issues: Vec<String>,
}

/// Basic Progressive Web App signal analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PwaAnalysis {
    pub manifest_url: Option<String>,
    /// Whether the linked manifest carries the required PWA fields; `None`
    /// when no manifest is linked or it could not be fetched.
    #[serde(default)]
    pub manifest_valid: Option<bool>,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub start_url: Option<String>,
    pub display: Option<String>,
    pub theme_color: Option<String>,
    pub icons_count: u32,
    pub has_maskable_icon: bool,
    pub service_worker_supported: bool,
    pub service_worker_controlled: bool,
    /// `None` when the registrations could not be read.
    #[serde(default)]
    pub service_worker_registrations: Option<u32>,
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

const fn technical_collector(
    id: &'static str,
    method: crate::audit::collection::CollectionMethod,
) -> crate::audit::collection::CollectorSpec {
    crate::audit::collection::CollectorSpec {
        id,
        module: "seo",
        method,
        method_version: 1,
        scoring: false,
    }
}

/// Browser cookie jar (`Network.getCookies`); names and flags, never values.
pub(crate) static COOKIE_INVENTORY: crate::audit::collection::CollectorSpec = technical_collector(
    "seo.cookie_inventory",
    crate::audit::collection::CollectionMethod::Cdp,
);
/// Same-origin link targets, checked with HEAD (GET fallback).
pub(crate) static BROKEN_LINKS: crate::audit::collection::CollectorSpec = technical_collector(
    "seo.broken_links",
    crate::audit::collection::CollectionMethod::Http,
);
/// `navigator.serviceWorker.getRegistrations()`.
pub(crate) static SERVICE_WORKERS: crate::audit::collection::CollectorSpec = technical_collector(
    "seo.service_worker_registrations",
    crate::audit::collection::CollectionMethod::Cdp,
);
/// The linked web app manifest.
pub(crate) static WEB_MANIFEST: crate::audit::collection::CollectorSpec = technical_collector(
    "seo.web_manifest",
    crate::audit::collection::CollectionMethod::Http,
);
/// HEAD of the www / non-www alternative host.
pub(crate) static WWW_REDIRECT: crate::audit::collection::CollectorSpec = technical_collector(
    "seo.www_redirect",
    crate::audit::collection::CollectionMethod::Http,
);

/// Technical SEO script (transport, canonical, lang, robots, …); the SEO
/// score reads it.
pub(crate) static TECHNICAL: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "seo.technical",
        module: "seo",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: true,
    };

/// Analyze technical SEO aspects
pub async fn analyze_technical_seo(
    page: &Page,
    url: &str,
    log: &crate::audit::collection::CollectionLog,
) -> Result<TechnicalSeo> {
    info!("Analyzing technical SEO...");

    let js_code = include_str!("js/technical_signals.js");

    let parsed = serde_json::Value::Object(
        log.record(
            &TECHNICAL,
            crate::audit::collection::evaluate_js::<crate::audit::collection::JsObject>(
                page, js_code,
            )
            .await,
        )
        .into_result(&TECHNICAL)?,
    );
    let https = parsed["https"].as_bool().ok_or_else(|| {
        AuditError::CdpError("SEO collector omitted the final transport scheme".into())
    })?;

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
    let text_excerpt_source = parsed["textExcerptSource"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let stylesheet_urls = parse_string_array(&parsed["stylesheetUrls"]);
    let script_urls = parse_string_array(&parsed["scriptUrls"]);
    let resource_urls = parse_string_array(&parsed["resourceUrls"]);
    let mixed_content = parse_mixed_content_resources(&parsed["mixedContentResources"]);
    let cookie_names = parse_string_array(&parsed["cookieNames"]);
    let storage_items = parse_storage_items(&parsed["storageItems"]);
    let has_zaraz_global = parsed["hasZarazGlobal"].as_bool().unwrap_or(false);
    let has_favicon = parsed["hasFavicon"].as_bool().unwrap_or(false);
    let has_og_image = parsed["hasOgImage"].as_bool().unwrap_or(false);
    let has_og_image_alt = parsed["hasOgImageAlt"].as_bool().unwrap_or(false);
    let has_twitter_image = parsed["hasTwitterImage"].as_bool().unwrap_or(false);
    let has_twitter_image_alt = parsed["hasTwitterImageAlt"].as_bool().unwrap_or(false);
    let web_manifest_url = parsed["webManifestUrl"].as_str().map(String::from);
    let theme_color = parsed["themeColor"].as_str().map(String::from);
    let service_worker_supported = parsed["serviceWorkerSupported"].as_bool().unwrap_or(false);
    let service_worker_controlled = parsed["serviceWorkerControlled"].as_bool().unwrap_or(false);
    let form_security = parse_form_security_analysis(&parsed["formSecurity"]);
    let amp = parse_amp_analysis(&parsed["amp"]);

    let google_fonts_sources =
        collect_google_fonts_sources(&stylesheet_urls, &resource_urls, &script_urls);
    let tracking_cookies: Vec<TrackingCookie> = cookie_names
        .iter()
        .filter_map(|name| classify_tracking_cookie(name))
        .collect();
    let cookie_inventory = log
        .record(&COOKIE_INVENTORY, collect_cookie_inventory(page, url).await)
        .into_value();
    let tracking_signals = collect_tracking_signals(
        &script_urls,
        &resource_urls,
        &tracking_cookies,
        cookie_inventory.as_deref().unwrap_or_default(),
        &storage_items,
    );
    let zaraz = detect_zaraz(
        &script_urls,
        &resource_urls,
        &tracking_cookies,
        has_zaraz_global,
    );
    // Links whose request failed are not reported as broken; the record
    // says the check is partial (#777).
    let broken_links = log
        .record(
            &BROKEN_LINKS,
            check_broken_internal_links(url, &internal_link_check_targets).await,
        )
        .into_value()
        .unwrap_or_default();
    let service_worker_registrations = log
        .record(
            &SERVICE_WORKERS,
            read_service_worker_registration_count(page).await,
        )
        .into_value();
    let manifest = log.record(
        &WEB_MANIFEST,
        match web_manifest_url.as_deref() {
            Some(manifest_url) => fetch_web_manifest(manifest_url).await,
            None => crate::audit::collection::Collected::NotApplicable("no_manifest_linked"),
        },
    );
    let pwa = analyze_pwa(
        web_manifest_url,
        manifest,
        theme_color,
        service_worker_supported,
        service_worker_controlled,
        service_worker_registrations,
    );

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
    let www_redirect = log
        .record(
            &WWW_REDIRECT,
            check_www_redirect(url, canonical_url.as_deref()).await,
        )
        .into_value();

    let mut technical = TechnicalSeo {
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
        amp,
        form_security,
        text_excerpt,
        text_excerpt_source,
        uses_remote_google_fonts: !google_fonts_sources.is_empty(),
        google_fonts_sources,
        tracking_cookies,
        cookie_inventory,
        storage_items,
        tracking_signals,
        zaraz,
        has_favicon,
        www_redirect,
        og_image_missing_alt: has_og_image && !has_og_image_alt,
        twitter_image_missing_alt: has_twitter_image && !has_twitter_image_alt,
        issues: Vec::new(),
    };

    // Issues are baked in canonical English (for JSON); the PDF re-derives
    // localized issues via collect_technical_issues at presentation time (#406).
    technical.issues = collect_technical_issues(&technical, true);

    Ok(technical)
}

/// A number as a Fluent argument (plural selectors need a number).
fn num(n: usize) -> FluentValue<'static> {
    FluentValue::from(n as i64)
}

/// Text as a Fluent argument.
fn txt(s: &str) -> FluentValue<'static> {
    FluentValue::from(s.to_string())
}

/// Add the technical issue `issue_type`: its message is `key` of
/// `locales/{en,de}/seo-issues.ftl` with `args`.
fn push_issue(
    issues: &mut Vec<TechnicalIssue>,
    issue_type: &str,
    severity: Severity,
    key: &str,
    args: &[(&str, FluentValue)],
    en: bool,
) {
    issues.push(TechnicalIssue {
        issue_type: issue_type.to_string(),
        message: crate::i18n::text_args(key, args, en),
        severity,
    });
}

/// Build the technical-SEO issue list in the requested language.
///
/// Pure function of the analysis struct — called with English at analysis time
/// (canonical, for JSON) and re-called with the report locale by the PDF
/// presentation builder (#406). It decides which issues exist, their severity
/// and the arguments of their `seo-technical-*` messages.
pub fn collect_technical_issues(t: &TechnicalSeo, en: bool) -> Vec<TechnicalIssue> {
    let mut issues = Vec::new();

    if !t.https {
        push_issue(
            &mut issues,
            "no_https",
            Severity::High,
            "seo-technical-no-https",
            &[],
            en,
        );
    }

    if t.canonical_url.is_none() {
        push_issue(
            &mut issues,
            "no_canonical",
            Severity::Medium,
            "seo-technical-no-canonical",
            &[],
            en,
        );
    }

    if t.lang.is_none() {
        push_issue(
            &mut issues,
            "no_lang",
            Severity::Medium,
            "seo-technical-no-lang",
            &[],
            en,
        );
    }

    if t.word_count < 300 {
        push_issue(
            &mut issues,
            "thin_content",
            Severity::Medium,
            "seo-technical-thin-content",
            &[("words", num(t.word_count as usize))],
            en,
        );
    }

    if t.internal_links == 0 {
        push_issue(
            &mut issues,
            "no_internal_links",
            Severity::Medium,
            "seo-technical-no-internal-links",
            &[],
            en,
        );
    }

    if !t.google_fonts_sources.is_empty() {
        push_issue(
            &mut issues,
            "remote_google_fonts",
            Severity::Low,
            "seo-technical-remote-google-fonts",
            &[],
            en,
        );
    }

    if let Some(ref robots) = t.robots_meta {
        if robots.to_lowercase().contains("noindex") {
            push_issue(
                &mut issues,
                "noindex",
                Severity::High,
                "seo-technical-noindex",
                &[],
                en,
            );
        }
    }

    if !t.hreflang.is_empty() && !t.hreflang_has_x_default {
        push_issue(
            &mut issues,
            "hreflang_no_x_default",
            Severity::Low,
            "seo-technical-hreflang-no-x-default",
            &[],
            en,
        );
    }

    if t.hreflang_missing_self_reference {
        push_issue(
            &mut issues,
            "hreflang_no_self_reference",
            Severity::Low,
            "seo-technical-hreflang-no-self-reference",
            &[],
            en,
        );
    }

    if t.internal_links_with_query_params > 0 {
        push_issue(
            &mut issues,
            "internal_links_with_query_params",
            Severity::Low,
            "seo-technical-internal-links-with-query-params",
            &[("count", num(t.internal_links_with_query_params as usize))],
            en,
        );
    }

    collect_pagination_issues(
        t.pagination_detected,
        t.pagination_prev.as_deref(),
        t.pagination_next.as_deref(),
        t.www_redirect.as_ref().map(|c| c.primary.as_str()),
        en,
        &mut issues,
    );

    if !t.broken_links.is_empty() {
        let list = t
            .broken_links
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("; ");
        push_issue(
            &mut issues,
            "broken_internal_links",
            Severity::High,
            "seo-technical-broken-internal-links",
            &[("count", num(t.broken_links.len())), ("list", txt(&list))],
            en,
        );
    }

    if !t.mixed_content.is_empty() {
        let blockable_count = t
            .mixed_content
            .iter()
            .filter(|resource| resource.blockable)
            .count();
        let samples = t
            .mixed_content
            .iter()
            .take(3)
            .map(|resource| resource.url.clone())
            .collect::<Vec<_>>()
            .join("; ");
        push_issue(
            &mut issues,
            "mixed_content",
            if blockable_count > 0 {
                Severity::High
            } else {
                Severity::Medium
            },
            "seo-technical-mixed-content",
            &[
                ("count", num(t.mixed_content.len())),
                ("blockable", num(blockable_count)),
                ("samples", txt(&samples)),
            ],
            en,
        );
    }

    if t.og_image_missing_alt {
        push_issue(
            &mut issues,
            "og_image_missing_alt",
            Severity::Low,
            "seo-technical-og-image-missing-alt",
            &[],
            en,
        );
    }

    if t.twitter_image_missing_alt {
        push_issue(
            &mut issues,
            "twitter_image_missing_alt",
            Severity::Low,
            "seo-technical-twitter-image-missing-alt",
            &[],
            en,
        );
    }

    collect_pwa_issues(&t.pwa, en, &mut issues);
    collect_amp_issues(&t.amp, en, &mut issues);
    collect_form_security_issues(&t.form_security, en, &mut issues);

    if let Some(ref check) = t.www_redirect {
        if !check.redirects_to_primary && check.alternative_status == 200 {
            let canonical_note = match check.canonical_matches_primary {
                Some(true) => "seo-technical-www-note-canonical-ok",
                Some(false) => "seo-technical-www-note-canonical-other",
                None => "seo-technical-www-note-canonical-none",
            };
            push_issue(
                &mut issues,
                "www_no_redirect",
                Severity::Medium,
                "seo-technical-www-no-redirect",
                &[
                    ("primary", txt(&check.primary)),
                    ("alternative", txt(&check.alternative)),
                    ("note", txt(&crate::i18n::text(canonical_note, en))),
                ],
                en,
            );
        }
        if let Some(false) = check.canonical_matches_primary {
            push_issue(
                &mut issues,
                "canonical_domain_mismatch",
                Severity::Medium,
                "seo-technical-canonical-domain-mismatch",
                &[("primary", txt(&check.primary))],
                en,
            );
        }
    }

    issues
}

/// Check whether the www ↔ non-www counterpart of `url` redirects to `url`
/// or serves its own content (duplicate).
async fn check_www_redirect(
    url: &str,
    canonical: Option<&str>,
) -> crate::audit::collection::Collected<WwwRedirectCheck> {
    use crate::audit::collection::{reason, Collected};
    const NOT_APEX_OR_WWW: &str = "host_is_not_apex_or_www";
    // Build the alternative URL variant
    let Ok(parsed) = url::Url::parse(url) else {
        return Collected::NotApplicable(NOT_APEX_OR_WWW);
    };
    let Some(host) = parsed.host_str() else {
        return Collected::NotApplicable(NOT_APEX_OR_WWW);
    };

    let (primary_host, alt_host) = if let Some(stripped) = host.strip_prefix("www.") {
        (host.to_string(), stripped.to_string())
    } else {
        (host.to_string(), format!("www.{}", host))
    };

    // Only makes sense for plain domains; skip IPs, localhost, and subdomains
    if primary_host.parse::<std::net::IpAddr>().is_ok() || primary_host == "localhost" {
        return Collected::NotApplicable(NOT_APEX_OR_WWW);
    }
    // Determine the bare host (without www prefix) and skip if it's a subdomain
    let bare_host = host.strip_prefix("www.").unwrap_or(host);
    if bare_host.split('.').count() > 2 {
        return Collected::NotApplicable(NOT_APEX_OR_WWW);
    }

    let mut alt_url = parsed.clone();
    if alt_url.set_host(Some(&alt_host)).is_err() {
        return Collected::NotApplicable(NOT_APEX_OR_WWW);
    }
    let alternative = alt_url.as_str().to_string();
    let primary = url.to_string();

    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Mozilla/5.0 (compatible; AuditMySite/1.0)")
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let response = match client.head(&alternative).send().await {
        Ok(r) => r,
        Err(e) => {
            // The alternative host not answering is no measured status.
            tracing::debug!("www-redirect check failed for {}: {}", alternative, e);
            return Collected::Failed(reason::ALTERNATE_HOST_UNREACHABLE);
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

    Collected::Measured(WwwRedirectCheck {
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

fn parse_storage_items(value: &serde_json::Value) -> Vec<StorageItem> {
    let mut seen = BTreeSet::new();
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let area = item["area"].as_str()?.trim();
                    let key = item["key"].as_str()?.trim();
                    if area.is_empty() || key.is_empty() {
                        return None;
                    }
                    let dedupe_key = format!("{area}:{key}");
                    if !seen.insert(dedupe_key) {
                        return None;
                    }
                    let classified = classify_tracking_cookie(key);
                    let provider = classified.as_ref().map(|cookie| cookie.provider.clone());
                    Some(StorageItem {
                        area: area.to_string(),
                        key: key.to_string(),
                        category: provider
                            .as_deref()
                            .and_then(classify_tracking_provider_category),
                        provider,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_amp_analysis(value: &serde_json::Value) -> AmpAnalysis {
    let issues = value
        .get("issues")
        .and_then(|items| items.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();

    AmpAnalysis {
        detected: value
            .get("detected")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        canonical_present: value
            .get("canonicalPresent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        runtime_script_present: value
            .get("runtimeScriptPresent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        boilerplate_present: value
            .get("boilerplatePresent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        issues,
    }
}

async fn collect_cookie_inventory(
    page: &Page,
    page_url: &str,
) -> crate::audit::collection::Collected<Vec<CookieInventoryItem>> {
    let page_host = url::Url::parse(page_url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_default();

    let cookies = match page.get_cookies().await {
        Ok(cookies) => cookies,
        Err(e) => {
            warn!("cookie inventory collection failed: {}", e);
            return crate::audit::collection::Collected::Failed(
                crate::audit::collection::reason::CDP_COMMAND_FAILED,
            );
        }
    };

    let inventory = cookies
        .into_iter()
        .map(|cookie| {
            let domain = cookie.domain.trim_start_matches('.').to_ascii_lowercase();
            let third_party = !page_host.is_empty()
                && domain != page_host
                && !page_host.ends_with(&format!(".{domain}"));
            let classified = classify_tracking_cookie(&cookie.name);
            let provider = classified.as_ref().map(|cookie| cookie.provider.clone());
            CookieInventoryItem {
                category: provider
                    .as_deref()
                    .and_then(classify_tracking_provider_category),
                provider,
                name: cookie.name,
                domain: cookie.domain,
                path: cookie.path,
                secure: cookie.secure,
                http_only: cookie.http_only,
                same_site: cookie
                    .same_site
                    .as_ref()
                    .map(|same_site| same_site.as_ref().to_string()),
                session: cookie.session,
                third_party,
            }
        })
        .collect();
    crate::audit::collection::Collected::Measured(inventory).empty_if(Vec::is_empty)
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

/// `manifest` is the fetched manifest: a document that is not a JSON
/// object (or was answered with an error status) is a measured, invalid
/// manifest (`Value::Null`); a request that failed is no measurement and
/// leaves `manifest_valid` unset (#777).
fn analyze_pwa(
    manifest_url: Option<String>,
    manifest: crate::audit::collection::Collected<serde_json::Value>,
    theme_color: Option<String>,
    service_worker_supported: bool,
    service_worker_controlled: bool,
    service_worker_registrations: Option<u32>,
) -> PwaAnalysis {
    let (Some(manifest_url), Some(manifest)) = (manifest_url.as_ref(), manifest.into_value())
    else {
        return PwaAnalysis {
            manifest_url,
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
        manifest_valid: Some(manifest_valid),
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

/// The manifest document. An error status or a body that is not JSON is a
/// measured result (the linked manifest is unusable: `Value::Null`); only a
/// request that did not complete is a failure.
async fn fetch_web_manifest(
    manifest_url: &str,
) -> crate::audit::collection::Collected<serde_json::Value> {
    use crate::audit::collection::{reason, Collected};
    let Ok(client) = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(Policy::limited(4))
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };
    let Ok(response) = client
        .get(manifest_url)
        .header("User-Agent", "auditmysite-pwa-checker/1.0")
        .send()
        .await
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };
    if !response.status().is_success() {
        return Collected::Measured(serde_json::Value::Null);
    }
    match response.text().await {
        Ok(body) => Collected::Measured(
            serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .filter(serde_json::Value::is_object)
                .unwrap_or(serde_json::Value::Null),
        ),
        Err(_) => Collected::Failed(reason::HTTP_REQUEST_FAILED),
    }
}

async fn read_service_worker_registration_count(
    page: &Page,
) -> crate::audit::collection::Collected<u32> {
    use crate::audit::collection::{reason, Collected};
    let expression = include_str!("js/service_worker_registrations.js");
    let Ok(params) = EvaluateParams::builder()
        .expression(expression.to_string())
        .await_promise(true)
        .build()
    else {
        return Collected::Failed(reason::CDP_COMMAND_FAILED);
    };
    match page
        .execute(params)
        .await
        .ok()
        .and_then(|result| result.result.result.value)
        .and_then(|value| value.as_u64())
    {
        Some(count) => Collected::Measured(count as u32),
        None => Collected::Failed(reason::EVALUATION_FAILED),
    }
}

fn collect_pwa_issues(pwa: &PwaAnalysis, en: bool, issues: &mut Vec<TechnicalIssue>) {
    if pwa.manifest_url.is_none() {
        push_issue(
            issues,
            "pwa_missing_manifest",
            Severity::Low,
            "seo-technical-pwa-missing-manifest",
            &[],
            en,
        );
        return;
    }

    if pwa.manifest_valid == Some(false) {
        push_issue(
            issues,
            "pwa_invalid_manifest",
            Severity::Medium,
            "seo-technical-pwa-invalid-manifest",
            &[],
            en,
        );
    }

    // Deliberately no "missing service worker" issue: a web app manifest is
    // routinely shipped for icons and theme color alone, so its presence does
    // not establish that the site intends to be an installable, offline-capable
    // PWA. `PwaAnalysis.service_worker_*` stays in the JSON as information.
}

fn collect_amp_issues(amp: &AmpAnalysis, en: bool, issues: &mut Vec<TechnicalIssue>) {
    if !amp.detected || amp.issues.is_empty() {
        return;
    }

    for issue in &amp.issues {
        let (key, severity) = match issue.as_str() {
            "missing_html_amp_attribute" => (
                "seo-technical-amp-missing-html-amp-attribute",
                Severity::Medium,
            ),
            "missing_canonical" => ("seo-technical-amp-missing-canonical", Severity::Medium),
            "missing_amp_runtime" => ("seo-technical-amp-missing-amp-runtime", Severity::Medium),
            "missing_amp_boilerplate" => {
                ("seo-technical-amp-missing-amp-boilerplate", Severity::Low)
            }
            "custom_script_not_allowed" => (
                "seo-technical-amp-custom-script-not-allowed",
                Severity::Medium,
            ),
            _ => continue,
        };

        push_issue(issues, &format!("amp_{issue}"), severity, key, &[], en);
    }
}

fn collect_form_security_issues(
    form_security: &FormSecurityAnalysis,
    en: bool,
    issues: &mut Vec<TechnicalIssue>,
) {
    if form_security.insecure_transport_forms > 0 {
        push_issue(
            issues,
            "form_insecure_transport",
            Severity::High,
            "seo-technical-form-insecure-transport",
            &[(
                "count",
                num(form_security.insecure_transport_forms as usize),
            )],
            en,
        );
    }

    if form_security.get_sensitive_forms > 0 {
        push_issue(
            issues,
            "form_sensitive_get",
            Severity::High,
            "seo-technical-form-sensitive-get",
            &[("count", num(form_security.get_sensitive_forms as usize))],
            en,
        );
    }

    if form_security.password_autocomplete_issues > 0 {
        push_issue(
            issues,
            "form_password_autocomplete",
            Severity::Medium,
            "seo-technical-form-password-autocomplete",
            &[(
                "count",
                num(form_security.password_autocomplete_issues as usize),
            )],
            en,
        );
    }
}

/// Same-origin link targets that answer with an error status. A target
/// whose request did not complete is no broken link; it makes the check
/// partial (#777).
async fn check_broken_internal_links(
    source_url: &str,
    targets: &[String],
) -> crate::audit::collection::Collected<Vec<String>> {
    check_broken_internal_links_within(source_url, targets, LINK_CHECK_BUDGET).await
}

/// Wall-clock budget of the whole broken-link check. Per request the
/// timeout is 5 s for `HEAD` and again for `GET`; with up to 50 targets, 8 at
/// a time, a host that accepts nothing held the audit for over a minute
/// (7 waves x 10 s). Targets still open at the deadline count as unanswered.
const LINK_CHECK_BUDGET: std::time::Duration = std::time::Duration::from_secs(20);

async fn check_broken_internal_links_within(
    source_url: &str,
    targets: &[String],
    budget: std::time::Duration,
) -> crate::audit::collection::Collected<Vec<String>> {
    use crate::audit::collection::{reason, Collected};
    let Some(source_origin) = url_origin(source_url) else {
        return Collected::NotApplicable("no_http_origin");
    };
    let Ok(client) = Client::builder()
        .redirect(Policy::limited(6))
        .timeout(std::time::Duration::from_secs(5))
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let mut unique = BTreeSet::new();
    for target in targets {
        if url_origin(target).as_deref() == Some(source_origin.as_str()) {
            unique.insert(target.clone());
        }
    }
    if unique.is_empty() {
        return Collected::Empty(Vec::new());
    }

    let selected: Vec<String> = unique.into_iter().take(50).collect();
    let selected_count = selected.len();
    let mut pending = stream::iter(selected)
        .map(|target| {
            let client = client.clone();
            async move {
                let status = check_link_status(&client, &target).await;
                (target, status)
            }
        })
        .buffer_unordered(8);
    let deadline = tokio::time::Instant::now() + budget;
    let mut statuses: Vec<(String, LinkStatus)> = Vec::new();
    while let Ok(Some(result)) = tokio::time::timeout_at(deadline, pending.next()).await {
        statuses.push(result);
    }
    // Requests cut off by the deadline never answered.
    let cut_off = selected_count - statuses.len();
    let checked = selected_count;
    let unanswered = cut_off
        + statuses
            .iter()
            .filter(|(_, status)| matches!(status, LinkStatus::Unanswered))
            .count();
    let broken: Vec<String> = statuses
        .into_iter()
        .filter_map(|(target, status)| match status {
            LinkStatus::Broken(reason) => Some(format!("{target} ({reason})")),
            LinkStatus::Ok | LinkStatus::Unanswered => None,
        })
        .collect();
    if unanswered == 0 {
        Collected::Measured(broken)
    } else if unanswered == checked {
        Collected::Failed(reason::HTTP_REQUEST_FAILED)
    } else {
        Collected::Partial {
            value: broken,
            gaps: vec!["link_request"],
        }
    }
}

enum LinkStatus {
    Ok,
    Broken(String),
    /// Neither HEAD nor GET completed: nothing is known about the target.
    Unanswered,
}

async fn check_link_status(client: &Client, target: &str) -> LinkStatus {
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
            LinkStatus::Broken(format!("HTTP {}", response.status().as_u16()))
        }
        Ok(_) => LinkStatus::Ok,
        Err(_) => check_link_status_with_get(client, target).await,
    }
}

async fn check_link_status_with_get(client: &Client, target: &str) -> LinkStatus {
    match client
        .get(target)
        .header("User-Agent", "auditmysite-link-checker/1.0")
        .send()
        .await
    {
        Ok(response) if response.status().as_u16() >= 400 => {
            LinkStatus::Broken(format!("HTTP {}", response.status().as_u16()))
        }
        Ok(_) => LinkStatus::Ok,
        Err(_) => LinkStatus::Unanswered,
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
    pagination_detected: bool,
    prev: Option<&str>,
    next: Option<&str>,
    url: Option<&str>,
    en: bool,
    issues: &mut Vec<TechnicalIssue>,
) {
    if !pagination_detected {
        return;
    }

    // Deliberately no "missing rel=prev/next" issue: Google dropped the pair
    // as an indexing signal in 2019, so reporting its absence as an SEO problem
    // is misleading. The checks below stay — they only fire when rel links are
    // actually present and wrong, which is broken markup either way.
    if prev.is_none() && next.is_none() {
        return;
    }

    if let Some(url) = url {
        if prev.is_some_and(|href| same_url(href, url))
            || next.is_some_and(|href| same_url(href, url))
        {
            push_issue(
                issues,
                "pagination_self_referential_rel_link",
                Severity::Low,
                "seo-technical-pagination-self-referential-rel-link",
                &[],
                en,
            );
        }
    }

    if let (Some(prev), Some(next)) = (prev, next) {
        if same_url(prev, next) {
            push_issue(
                issues,
                "pagination_duplicate_prev_next",
                Severity::Low,
                "seo-technical-pagination-duplicate-prev-next",
                &[],
                en,
            );
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

pub(crate) fn classify_tracking_cookie(name: &str) -> Option<TrackingCookie> {
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

pub(crate) fn classify_tracking_provider_category(provider: &str) -> Option<String> {
    let category = match provider {
        "Google" | "Matomo" | "Cloudflare Zaraz" => "analytics",
        "Meta" => "social",
        "Hotjar" | "HubSpot" => "marketing",
        _ => return None,
    };
    Some(category.to_string())
}

fn collect_tracking_signals(
    script_urls: &[String],
    resource_urls: &[String],
    tracking_cookies: &[TrackingCookie],
    cookie_inventory: &[CookieInventoryItem],
    storage_items: &[StorageItem],
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
    for cookie in cookie_inventory {
        if let Some(provider) = &cookie.provider {
            signals.insert(format_tracking_signal(
                provider,
                cookie.category.as_deref(),
                "Cookie",
                &cookie.name,
            ));
        }
    }
    for item in storage_items {
        if let Some(provider) = &item.provider {
            signals.insert(format!(
                "{} ({})",
                format_tracking_signal(provider, item.category.as_deref(), "Storage", &item.key),
                item.area
            ));
        }
    }
    signals.into_iter().collect()
}

fn format_tracking_signal(
    provider: &str,
    category: Option<&str>,
    signal_type: &str,
    name: &str,
) -> String {
    match category {
        Some(category) => format!("{provider} [{category}]-{signal_type}: {name}"),
        None => format!("{provider}-{signal_type}: {name}"),
    }
}

fn classify_tracking_url(url: &str) -> Option<&'static str> {
    let lower = url.to_ascii_lowercase();
    if lower.contains("/cdn-cgi/zaraz/") {
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
        if url.to_ascii_lowercase().contains("/cdn-cgi/zaraz/") {
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
    fn test_storage_items_are_inventory_only_and_feed_tracking_signals() {
        let value = serde_json::json!([
            {"area": "localStorage", "key": "_ga"},
            {"area": "localStorage", "key": "_ga"},
            {"area": "sessionStorage", "key": "app_state"}
        ]);

        let storage_items = parse_storage_items(&value);
        assert_eq!(storage_items.len(), 2);
        assert_eq!(storage_items[0].provider.as_deref(), Some("Google"));
        assert_eq!(storage_items[0].category.as_deref(), Some("analytics"));
        assert_eq!(storage_items[1].provider, None);

        let signals = collect_tracking_signals(&[], &[], &[], &[], &storage_items);
        assert!(signals
            .iter()
            .any(|signal| signal == "Google [analytics]-Storage: _ga (localStorage)"));
    }

    #[test]
    fn test_cookie_inventory_feeds_tracking_signals() {
        let cookie_inventory = vec![CookieInventoryItem {
            name: "_fbp".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            secure: true,
            http_only: true,
            same_site: Some("Lax".to_string()),
            session: false,
            third_party: false,
            provider: Some("Meta".to_string()),
            category: Some("social".to_string()),
        }];

        let signals = collect_tracking_signals(&[], &[], &[], &cookie_inventory, &[]);
        assert!(signals
            .iter()
            .any(|signal| signal == "Meta [social]-Cookie: _fbp"));
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
    fn test_zaraz_detection_does_not_false_positive_on_polish_word() {
        // Regression: "zaraz" ("right away" in Polish) must not be treated as
        // a Cloudflare Zaraz signal unless it appears on the actual
        // /cdn-cgi/zaraz/ path.
        let zaraz = detect_zaraz(
            &["https://example.pl/blog/zaraz-wracam-do-pracy".to_string()],
            &[],
            &[],
            false,
        );
        assert!(!zaraz.detected);
        assert!(zaraz.signals.is_empty());
    }

    #[test]
    fn test_classify_tracking_url_does_not_false_positive_on_polish_word() {
        assert_eq!(
            classify_tracking_url("https://example.pl/blog/zaraz-wracam-do-pracy"),
            None
        );
        assert_eq!(
            classify_tracking_url("https://example.com/cdn-cgi/zaraz/s.js"),
            Some("Cloudflare Zaraz")
        );
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
        collect_pwa_issues(&PwaAnalysis::default(), true, &mut issues);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "pwa_missing_manifest");
        assert_eq!(issues[0].severity, Severity::Low);
    }

    #[test]
    fn test_collect_pwa_issues_invalid_manifest() {
        let pwa = PwaAnalysis {
            manifest_url: Some("https://example.com/manifest.webmanifest".to_string()),
            manifest_valid: Some(false),
            service_worker_supported: true,
            service_worker_registrations: Some(0),
            ..PwaAnalysis::default()
        };
        let mut issues = Vec::new();
        collect_pwa_issues(&pwa, true, &mut issues);

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "pwa_invalid_manifest"));
    }

    /// A manifest without a service worker is not reported: shipping a
    /// manifest for icons/theme color is ordinary, and does not make a site a
    /// PWA that owes anyone offline support.
    #[test]
    fn test_manifest_without_service_worker_is_not_an_issue() {
        let pwa = PwaAnalysis {
            manifest_url: Some("https://example.com/manifest.webmanifest".to_string()),
            manifest_valid: Some(true),
            service_worker_supported: true,
            service_worker_registrations: Some(0),
            ..PwaAnalysis::default()
        };
        let mut issues = Vec::new();
        collect_pwa_issues(&pwa, true, &mut issues);

        assert!(issues.is_empty(), "{issues:#?}");
    }

    #[test]
    fn test_parse_amp_analysis_and_collect_issues() {
        let value = serde_json::json!({
            "detected": true,
            "canonicalPresent": false,
            "runtimeScriptPresent": false,
            "boilerplatePresent": true,
            "issues": ["missing_canonical", "missing_amp_runtime"]
        });

        let amp = parse_amp_analysis(&value);
        assert!(amp.detected);
        assert!(!amp.canonical_present);
        assert_eq!(amp.issues.len(), 2);

        let mut issues = Vec::new();
        collect_amp_issues(&amp, true, &mut issues);
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "amp_missing_canonical"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "amp_missing_amp_runtime"));
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

        collect_form_security_issues(&form_security, true, &mut issues);

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

        collect_form_security_issues(&form_security, true, &mut issues);

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

    /// Absent rel=prev/next is not reported: Google stopped using the pair for
    /// indexing in 2019, so its absence is no longer an SEO defect.
    #[test]
    fn test_missing_pagination_rel_links_is_not_an_issue() {
        let mut issues = Vec::new();
        collect_pagination_issues(
            true,
            None,
            None,
            Some("https://example.com/blog/page/2/"),
            true,
            &mut issues,
        );

        assert!(issues.is_empty(), "{issues:#?}");
    }

    #[test]
    fn test_collect_pagination_issues_self_reference_and_duplicate() {
        let mut issues = Vec::new();
        collect_pagination_issues(
            true,
            Some("https://example.com/blog/page/2/#top"),
            Some("https://example.com/blog/page/2/"),
            Some("https://example.com/blog/page/2/"),
            true,
            &mut issues,
        );

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "pagination_self_referential_rel_link"));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "pagination_duplicate_prev_next"));
    }

    #[test]
    fn test_collect_technical_issues_flags_missing_social_image_alt() {
        let technical = TechnicalSeo {
            og_image_missing_alt: true,
            twitter_image_missing_alt: true,
            https: true,
            has_lang: true,
            canonical_url: Some("https://example.com/".to_string()),
            word_count: 500,
            internal_links: 1,
            ..TechnicalSeo::default()
        };

        let issues = collect_technical_issues(&technical, true);

        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "og_image_missing_alt"
                && issue.severity == Severity::Low));
        assert!(issues
            .iter()
            .any(|issue| issue.issue_type == "twitter_image_missing_alt"
                && issue.severity == Severity::Low));
    }

    #[test]
    fn test_collect_technical_issues_no_finding_when_social_image_absent_or_alt_present() {
        let technical = TechnicalSeo {
            og_image_missing_alt: false,
            twitter_image_missing_alt: false,
            https: true,
            has_lang: true,
            canonical_url: Some("https://example.com/".to_string()),
            word_count: 500,
            internal_links: 1,
            ..TechnicalSeo::default()
        };

        let issues = collect_technical_issues(&technical, true);

        assert!(!issues
            .iter()
            .any(|issue| issue.issue_type == "og_image_missing_alt"));
        assert!(!issues
            .iter()
            .any(|issue| issue.issue_type == "twitter_image_missing_alt"));
    }
}

/// Counter-cases of [`TECHNICAL`] (#777).
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_contract::<crate::audit::collection::JsObject>(
        serde_json::json!({"https": true}),
    );
    // A failed collection stops the module instead of yielding a default.
    assert!(
        crate::audit::collection::Collected::<crate::audit::collection::JsObject>::Failed("x")
            .into_result(&TECHNICAL)
            .is_err()
    );
}

/// Counter-cases of the technical SEO collectors (#777).
#[cfg(test)]
pub(crate) mod counter_cases {
    use super::*;
    use crate::audit::collection::{reason, Collected, CollectionState};

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(f)
    }

    fn pwa_issue_types(manifest: Collected<serde_json::Value>, linked: bool) -> Vec<String> {
        let pwa = analyze_pwa(
            linked.then(|| "https://example.com/site.webmanifest".to_string()),
            manifest,
            None,
            true,
            false,
            Some(0),
        );
        let mut issues = Vec::new();
        collect_pwa_issues(&pwa, true, &mut issues);
        issues.into_iter().map(|i| i.issue_type).collect()
    }

    /// A manifest request that failed proves nothing about the manifest; an
    /// error status or invalid document is a measured invalid manifest.
    pub(crate) fn web_manifest() {
        assert!(pwa_issue_types(Collected::Failed(reason::HTTP_REQUEST_FAILED), true).is_empty());
        assert_eq!(
            pwa_issue_types(Collected::Measured(serde_json::Value::Null), true),
            vec!["pwa_invalid_manifest"]
        );
        assert_eq!(
            pwa_issue_types(Collected::NotApplicable("no_manifest_linked"), false),
            vec!["pwa_missing_manifest"]
        );
    }

    /// Unanswered link requests are no broken links; no same-origin target
    /// is a valid empty check.
    pub(crate) fn broken_links() {
        let refused = block_on(check_broken_internal_links(
            "http://127.0.0.1:9/",
            &[
                "http://127.0.0.1:9/a".to_string(),
                "http://127.0.0.1:9/b".to_string(),
            ],
        ));
        assert_eq!(refused.state(), CollectionState::Failed);
        assert!(refused.value().is_none());
        let empty = block_on(check_broken_internal_links(
            "https://example.com/",
            &["https://other.example/x".to_string()],
        ));
        assert_eq!(empty.state(), CollectionState::Empty);
        broken_links_budget();
    }

    /// A host that accepts connections and never answers ends the check at
    /// its budget, not after every wave of per-request timeouts (7 waves x
    /// 10 s for 50 targets). Nothing answered: failed, not "no broken links".
    fn broken_links_budget() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let held = std::thread::spawn(move || {
            let mut streams = Vec::new();
            while let Ok((stream, _)) = listener.accept() {
                streams.push(stream);
                if streams.len() >= 64 {
                    break;
                }
            }
        });
        let source = format!("http://127.0.0.1:{port}/");
        let targets: Vec<String> = (0..24).map(|i| format!("{source}page-{i}")).collect();
        let started = std::time::Instant::now();
        let checked = block_on(check_broken_internal_links_within(
            &source,
            &targets,
            std::time::Duration::from_millis(800),
        ));
        assert!(
            started.elapsed() < std::time::Duration::from_secs(4),
            "the check outlived its budget: {:?}",
            started.elapsed()
        );
        assert_eq!(checked.state(), CollectionState::Failed);
        assert!(checked.value().is_none());
        drop(held);
    }

    /// Local hosts have no www variant to compare.
    pub(crate) fn www_redirect() {
        for url in [
            "http://localhost/",
            "https://127.0.0.1/",
            "https://api.example.com/",
        ] {
            assert_eq!(
                block_on(check_www_redirect(url, None)).state(),
                CollectionState::NotApplicable,
                "{url}"
            );
        }
    }

    /// Unread browser state stays unset instead of reading "none".
    pub(crate) fn browser_state() {
        let cookies: Collected<Vec<CookieInventoryItem>> =
            Collected::Failed(reason::CDP_COMMAND_FAILED);
        assert!(cookies.into_value().is_none());
        let none: Collected<Vec<CookieInventoryItem>> =
            Collected::Measured(Vec::new()).empty_if(Vec::is_empty);
        assert_eq!(none.state(), CollectionState::Empty);
        let registrations: Collected<u32> = Collected::Failed(reason::EVALUATION_FAILED);
        let pwa = analyze_pwa(
            None,
            Collected::NotApplicable("no_manifest_linked"),
            None,
            true,
            false,
            registrations.into_value(),
        );
        assert_eq!(pwa.service_worker_registrations, None);
    }

    #[test]
    fn technical_collector_counter_cases() {
        web_manifest();
        broken_links();
        www_redirect();
        browser_state();
    }
}
