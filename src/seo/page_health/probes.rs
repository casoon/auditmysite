//! HTTP probes: redirects, custom 404, resource caching, content types,
//! www/non-www consolidation and URL canonicalization.

use tracing::debug;

use crate::audit::collection::{reason, Collected, CollectionLog, CollectionMethod, CollectorSpec};

use super::{
    ContentTypeFinding, Custom404Check, PageHealthAnalysis, RedirectHop, ResourceCacheAudit,
    ResourceCacheFinding, UrlCanonicalizationCheck, WwwConsolidation,
};

const fn http_probe(id: &'static str) -> CollectorSpec {
    CollectorSpec {
        id,
        module: "seo",
        method: CollectionMethod::Http,
        method_version: 1,
        scoring: false,
    }
}

pub(crate) static HEADERS: CollectorSpec = http_probe("seo.page_health.headers");
pub(crate) static REDIRECT_CHAIN: CollectorSpec = http_probe("seo.page_health.redirect_chain");
pub(crate) static CUSTOM_404: CollectorSpec = http_probe("seo.page_health.custom_404");
pub(crate) static RESOURCE_CACHE: CollectorSpec = http_probe("seo.page_health.resource_cache");
pub(crate) static WWW_CONSOLIDATION: CollectorSpec =
    http_probe("seo.page_health.www_consolidation");
pub(crate) static URL_CANONICALIZATION: CollectorSpec =
    http_probe("seo.page_health.url_canonicalization");

/// Main-document response headers, as far as page health reads them.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct HeaderProbe {
    pub uses_http2: bool,
    pub compression: bool,
    pub cache_control: Option<String>,
    pub server_timing_count: u32,
    pub charset_header: Option<String>,
}

pub(super) async fn run_http_probes(url: &str, a: &mut PageHealthAnalysis, log: &CollectionLog) {
    let Ok(parsed) = url::Url::parse(url) else {
        return;
    };
    let origin = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));

    // Run all probes concurrently
    let probe_url = format!("{}/auditmysite-404-probe-xyz123", origin);
    let cache_probe_urls = a.resource_cache_probe_urls.clone();
    let (
        custom_404_result,
        www_result,
        url_canonicalization_result,
        header_result,
        redirect_chain,
        resource_cache,
    ) = tokio::join!(
        probe_custom_404(&probe_url),
        check_www_consolidation(url),
        check_url_canonicalization(url),
        probe_headers(url),
        follow_redirect_chain(url),
        audit_resource_cache(&cache_probe_urls)
    );

    apply_http_probes(
        a,
        log.record(&CUSTOM_404, custom_404_result),
        log.record(&WWW_CONSOLIDATION, www_result),
        log.record(&URL_CANONICALIZATION, url_canonicalization_result),
        log.record(&HEADERS, header_result),
        log.record(&REDIRECT_CHAIN, redirect_chain),
        log.record(&RESOURCE_CACHE, resource_cache),
    );
    debug!("Page-health probes for {} done", probe_url);
}

/// Fold the probe results into `a`. A probe that did not measure leaves its
/// fields unset (`None`): a failed request is no evidence of a missing
/// header, redirect or consolidation (#777).
pub(super) fn apply_http_probes(
    a: &mut PageHealthAnalysis,
    custom_404_result: Collected<Custom404Check>,
    www_result: Collected<WwwConsolidation>,
    url_canonicalization_result: Collected<UrlCanonicalizationCheck>,
    header_result: Collected<HeaderProbe>,
    redirect_chain: Collected<Vec<(u16, String)>>,
    resource_cache: Collected<ResourceCacheAudit>,
) {
    // Soft 404: only record the probe status when the server actually returns
    // 200 for a non-existent URL (= soft 404 confirmed). A proper 404/301/etc.
    // response means everything is fine — leave soft_404_status as None so the
    // JSON field stays absent rather than showing a confusing non-200 code.
    if let Some(check) = custom_404_result.into_value() {
        a.is_soft_404 = check.status == 200;
        if a.is_soft_404 {
            a.soft_404_status = Some(check.status);
        }
        a.custom_404 = Some(check);
    }

    // www consolidation
    a.www_consolidation = www_result.into_value();

    // Trailing-slash and http→https consistency (#542)
    a.url_canonicalization = url_canonicalization_result.into_value();

    // Redirect chain. A chain cut short by a failed hop is a lower bound.
    let hops: Vec<RedirectHop> = redirect_chain
        .into_value()
        .unwrap_or_default()
        .into_iter()
        .filter(|(status, _)| *status >= 300 && *status < 400)
        .map(|(status, url)| RedirectHop { status, url })
        .collect();
    a.redirect_count = hops.len() as u32;
    a.redirect_chain = hops;

    // HTTP headers
    if let Some(headers) = header_result.into_value() {
        a.uses_http2 = Some(headers.uses_http2);
        a.has_compression = Some(headers.compression);
        a.has_efficient_cache = headers
            .cache_control
            .as_deref()
            .map(is_cache_policy_efficient)
            .unwrap_or(false);
        a.cache_control = headers.cache_control;
        a.server_timing_count = headers.server_timing_count;
        a.charset_header = headers.charset_header;
    }
    if document_timing_indicates_compression(a) {
        a.has_compression = Some(true);
    }
    // A failed cache probe checked nothing; `checked_resources` says so.
    a.resource_cache = resource_cache.into_value().unwrap_or_default();
}

/// Probe main page headers: HTTP version, compression, Cache-Control, Server-Timing, charset.
async fn probe_headers(url: &str) -> Collected<HeaderProbe> {
    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("auditmysite-probe/1.0")
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let Ok(resp) = client.get(url).send().await else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };
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

    let charset_header = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split(';')
                .skip(1)
                .find_map(|part| part.trim().strip_prefix("charset="))
                .map(|c| c.trim_matches('"').to_ascii_lowercase())
        });

    Collected::Measured(HeaderProbe {
        uses_http2,
        compression,
        cache_control,
        server_timing_count,
        charset_header,
    })
}

/// Follow redirect chain manually, returning (status, url) pairs for each hop.
async fn follow_redirect_chain(url: &str) -> Collected<Vec<(u16, String)>> {
    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("auditmysite-probe/1.0")
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let mut chain = Vec::new();
    let mut current = url.to_string();

    for _ in 0..10 {
        let Ok(resp) = client.head(&current).send().await else {
            // The first request failing measured nothing; a later hop
            // failing leaves the chain as a lower bound.
            return if chain.is_empty() {
                Collected::Failed(reason::HTTP_REQUEST_FAILED)
            } else {
                Collected::Partial {
                    value: chain,
                    gaps: vec!["redirect_hop"],
                }
            };
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
    Collected::Measured(chain)
}

/// Probe a known non-existent URL and classify status plus page customization.
async fn probe_custom_404(url: &str) -> Collected<Custom404Check> {
    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let Ok(response) = client.get(url).send().await else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();

    Collected::Measured(Custom404Check {
        probe_url: url.to_string(),
        status,
        proper_status: matches!(status, 404 | 410),
        custom_page: looks_like_custom_404_page(&body),
    })
}

async fn audit_resource_cache(urls: &[String]) -> Collected<ResourceCacheAudit> {
    if urls.is_empty() {
        return Collected::Empty(ResourceCacheAudit::default());
    }

    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(3))
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("auditmysite-cache-probe/1.0")
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let mut audit = ResourceCacheAudit::default();
    let mut seen = std::collections::BTreeSet::new();
    let (mut attempted, mut failed) = (0usize, 0usize);

    for url in urls
        .iter()
        .filter(|url| seen.insert((*url).clone()))
        .take(30)
    {
        attempted += 1;
        let Ok(response) = client.head(url).send().await else {
            failed += 1;
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

        if let Some(expected) = expected_content_type_substring(url) {
            let actual = headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            let matches_expected = actual
                .as_deref()
                .map(|ct| ct.to_ascii_lowercase().contains(expected))
                .unwrap_or(false);
            if !matches_expected {
                if actual.is_none() {
                    audit.missing_content_type += 1;
                } else {
                    audit.incorrect_content_type += 1;
                }
                if audit.content_type_samples.len() < 5 {
                    audit.content_type_samples.push(ContentTypeFinding {
                        url: url.clone(),
                        expected: expected.to_string(),
                        actual,
                    });
                }
            }
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

    // Requests that failed checked nothing: all of them failing is no
    // measurement, some of them failing a partial one.
    if failed == attempted {
        Collected::Failed(reason::HTTP_REQUEST_FAILED)
    } else if failed > 0 {
        Collected::Partial {
            value: audit,
            gaps: vec!["resource_request"],
        }
    } else {
        Collected::Measured(audit)
    }
}

pub(super) fn looks_like_custom_404_page(body: &str) -> bool {
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

pub(super) fn is_static_cache_candidate(url: &str) -> bool {
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

/// Substring expected in a correct Content-Type header for this URL's file
/// extension, or `None` when the extension isn't one we have an opinion on.
pub(super) fn expected_content_type_substring(url: &str) -> Option<&'static str> {
    let filename = url::Url::parse(url)
        .ok()?
        .path_segments()
        .and_then(|mut segments| segments.next_back().map(str::to_ascii_lowercase))?;

    if filename.ends_with(".js") || filename.ends_with(".mjs") {
        Some("javascript")
    } else if filename.ends_with(".css") {
        Some("css")
    } else if filename.ends_with(".json") {
        Some("json")
    } else if filename.ends_with(".svg") {
        Some("svg")
    } else if filename.ends_with(".png") {
        Some("png")
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        Some("jpeg")
    } else if filename.ends_with(".gif") {
        Some("gif")
    } else if filename.ends_with(".webp") {
        Some("webp")
    } else if filename.ends_with(".avif") {
        Some("avif")
    } else if filename.ends_with(".woff") || filename.ends_with(".woff2") {
        Some("font")
    } else {
        None
    }
}

pub(super) fn is_cache_policy_efficient(cache_control: &str) -> bool {
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
async fn check_www_consolidation(url: &str) -> Collected<WwwConsolidation> {
    const NOT_APEX_OR_WWW: &str = "host_is_not_apex_or_www";
    let Some(host) = url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
    else {
        return Collected::NotApplicable(NOT_APEX_OR_WWW);
    };
    let host = host.as_str();

    // Skip IPs, localhost, and subdomains that aren't www
    if host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
        return Collected::NotApplicable(NOT_APEX_OR_WWW);
    }

    let (www_url, non_www_url) = if let Some(non_www_host) = host.strip_prefix("www.") {
        let non_www = url.replacen(host, non_www_host, 1);
        (url.to_string(), non_www)
    } else {
        // Only handle apex domains (e.g. example.com), skip subdomains like api.example.com
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() != 2 {
            return Collected::NotApplicable(NOT_APEX_OR_WWW);
        }
        let www_host = format!("www.{}", host);
        let www = url.replacen(host, &www_host, 1);
        (www, url.to_string())
    };

    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let (www_resp, non_www_resp) = tokio::join!(
        client.head(&www_url).send(),
        client.head(&non_www_url).send()
    );
    // Consolidation compares both variants: without both answers there is
    // nothing to compare, and "not consolidated" would be invented.
    let (Ok(www_resp), Ok(non_www_resp)) = (www_resp, non_www_resp) else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let www_status = Some(www_resp.status().as_u16());
    let non_www_status = Some(non_www_resp.status().as_u16());

    let location = |resp: &reqwest::Response| {
        resp.headers()
            .get("location")
            .and_then(|v| v.to_str().ok().map(String::from))
            .unwrap_or_default()
    };
    let www_location = location(&www_resp);
    let non_www_location = location(&non_www_resp);

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

    Collected::Measured(WwwConsolidation {
        www_status,
        non_www_status,
        www_redirects_to_non_www: www_redirects,
        non_www_redirects_to_www: non_www_redirects,
        canonical_variant,
        is_consolidated,
    })
}

/// Check trailing-slash and http→https redirect consistency (#542).
async fn check_url_canonicalization(url: &str) -> Collected<UrlCanonicalizationCheck> {
    const LOCAL_HOST: &str = "host_is_local_or_ip";
    let Some(parsed) = url::Url::parse(url).ok().filter(|p| p.host_str().is_some()) else {
        return Collected::NotApplicable(LOCAL_HOST);
    };
    let host = parsed.host_str().unwrap_or_default();

    // Skip IPs and localhost — canonicalization redirects aren't meaningful there.
    if host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
        return Collected::NotApplicable(LOCAL_HOST);
    }

    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
    else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };
    let mut gaps = Vec::new();

    let path = parsed.path();
    let trailing_slash_variant_url = if path.is_empty() || path == "/" {
        // Root path: a trailing slash is unambiguous, nothing to compare.
        None
    } else {
        let mut variant = parsed.clone();
        if let Some(stripped) = path.strip_suffix('/') {
            variant.set_path(stripped);
        } else {
            variant.set_path(&format!("{path}/"));
        }
        Some(variant)
    };

    let (trailing_slash_variant_status, trailing_slash_inconsistent) =
        if let Some(variant_url) = &trailing_slash_variant_url {
            let (orig_resp, variant_resp) = tokio::join!(
                client.head(url).send(),
                client.head(variant_url.as_str()).send()
            );
            match (orig_resp, variant_resp) {
                (Ok(orig), Ok(variant)) => {
                    let variant_status = variant.status().as_u16();
                    // Both variants answering 200 independently means neither
                    // redirects to a single canonical form — a duplicate-content
                    // risk. A 30x on either side (redirecting to the other) is
                    // the expected, consistent case.
                    let inconsistent = orig.status().as_u16() == 200 && variant_status == 200;
                    (Some(variant_status), Some(inconsistent))
                }
                _ => {
                    gaps.push("trailing_slash");
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

    let (http_status, http_to_https_missing) = if parsed.scheme() == "https" {
        let mut http_variant = parsed.clone();
        if http_variant.set_scheme("http").is_ok() {
            // An http:// variant that does not answer is no evidence of a
            // missing redirect: nothing was measured (#777).
            match client.head(http_variant.as_str()).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let location = resp
                        .headers()
                        .get("location")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default();
                    let redirects_to_https =
                        matches!(status, 301 | 302 | 307 | 308) && location.starts_with("https://");
                    (Some(status), Some(!redirects_to_https))
                }
                Err(_) => {
                    gaps.push("http_variant");
                    (None, None)
                }
            }
        } else {
            // Scheme swap failed (should not happen for https → http, both
            // are special schemes with defined ports) — skip rather than
            // probe the unchanged https URL and misreport it as a missing
            // HTTP→HTTPS redirect.
            (None, None)
        }
    } else {
        (None, None)
    };

    let check = UrlCanonicalizationCheck {
        trailing_slash_variant_status,
        trailing_slash_inconsistent,
        http_status,
        http_to_https_missing,
    };
    let attempted =
        usize::from(trailing_slash_variant_url.is_some()) + usize::from(parsed.scheme() == "https");
    if gaps.is_empty() {
        Collected::Measured(check)
    } else if gaps.len() == attempted {
        Collected::Failed(reason::HTTP_REQUEST_FAILED)
    } else {
        Collected::Partial { value: check, gaps }
    }
}

pub(super) fn document_timing_indicates_compression(a: &PageHealthAnalysis) -> bool {
    let (Some(transfer), Some(decoded)) = (a.document_transfer_bytes, a.document_decoded_bytes)
    else {
        return false;
    };
    decoded > 0 && transfer > 0 && (transfer as f64) < (decoded as f64 * 0.8)
}

/// Counter-cases of the page-health HTTP probes (#777): a probe that did not
/// measure produces no issue; the same probe measuring the defect does.
#[cfg(test)]
pub(crate) mod counter_cases {
    use super::*;
    use crate::seo::page_health::collect_issues;

    fn issue_types(a: &PageHealthAnalysis) -> Vec<String> {
        collect_issues(a, true)
            .into_iter()
            .map(|i| i.issue_type)
            .collect()
    }

    fn applied(
        headers: Collected<HeaderProbe>,
        www: Collected<WwwConsolidation>,
        canonical: Collected<UrlCanonicalizationCheck>,
    ) -> PageHealthAnalysis {
        let mut a = PageHealthAnalysis {
            dom_node_count: 10,
            has_doctype: true,
            ..Default::default()
        };
        apply_http_probes(
            &mut a,
            Collected::Failed(reason::HTTP_REQUEST_FAILED),
            www,
            canonical,
            headers,
            Collected::Failed(reason::HTTP_REQUEST_FAILED),
            Collected::Failed(reason::HTTP_REQUEST_FAILED),
        );
        a
    }

    const PROBE_ISSUES: [&str; 4] = [
        "http1_only",
        "missing_compression",
        "www_not_consolidated",
        "http_not_redirected_to_https",
    ];

    fn failed<T>() -> Collected<T> {
        Collected::Failed(reason::HTTP_REQUEST_FAILED)
    }

    /// Every probe failing: none of the transport issues appears, and the
    /// fields stay unset instead of reading "no".
    pub(crate) fn transport_failure() {
        let a = applied(failed(), failed(), failed());
        let issues = issue_types(&a);
        for issue in PROBE_ISSUES {
            assert!(
                !issues.iter().any(|i| i == issue),
                "{issue} from a failed probe"
            );
        }
        assert_eq!(a.uses_http2, None);
        assert_eq!(a.has_compression, None);
        assert!(a.www_consolidation.is_none());
        assert!(a.url_canonicalization.is_none());
        assert_eq!(a.redirect_count, 0);
        assert_eq!(a.resource_cache.checked_resources, 0);
    }

    /// Measured defects are reported.
    pub(crate) fn measured_defects() {
        let a = applied(
            Collected::Measured(HeaderProbe {
                uses_http2: false,
                compression: false,
                cache_control: None,
                server_timing_count: 0,
                charset_header: None,
            }),
            Collected::Measured(WwwConsolidation {
                www_status: Some(200),
                non_www_status: Some(200),
                www_redirects_to_non_www: false,
                non_www_redirects_to_www: false,
                canonical_variant: "inconsistent".into(),
                is_consolidated: false,
            }),
            Collected::Measured(UrlCanonicalizationCheck {
                trailing_slash_variant_status: None,
                trailing_slash_inconsistent: None,
                http_status: Some(200),
                http_to_https_missing: Some(true),
            }),
        );
        let issues = issue_types(&a);
        for issue in PROBE_ISSUES {
            assert!(
                issues.iter().any(|i| i == issue),
                "{issue} missing: {issues:?}"
            );
        }
    }

    /// A partial canonicalization check reports only what it measured.
    pub(crate) fn partial_result() {
        let a = applied(
            failed(),
            failed(),
            Collected::Partial {
                value: UrlCanonicalizationCheck {
                    trailing_slash_variant_status: Some(200),
                    trailing_slash_inconsistent: Some(true),
                    http_status: None,
                    http_to_https_missing: None,
                },
                gaps: vec!["http_variant"],
            },
        );
        let issues = issue_types(&a);
        assert!(issues.iter().any(|i| i == "trailing_slash_inconsistent"));
        assert!(!issues.iter().any(|i| i == "http_not_redirected_to_https"));
    }

    /// No resources to probe is a valid empty measurement.
    pub(crate) fn valid_empty() {
        let audit = tokio_test_block_on(audit_resource_cache(&[]));
        assert_eq!(
            audit.state(),
            crate::audit::collection::CollectionState::Empty
        );
    }

    fn tokio_test_block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(f)
    }

    /// Non-applicable hosts are no failure.
    pub(crate) fn not_applicable() {
        for url in [
            "http://localhost:8080/",
            "https://127.0.0.1/",
            "https://api.example.com/",
        ] {
            let www = tokio_test_block_on(check_www_consolidation(url));
            assert_eq!(
                www.state(),
                crate::audit::collection::CollectionState::NotApplicable,
                "{url}"
            );
        }
        let canonical = tokio_test_block_on(check_url_canonicalization("http://localhost/"));
        assert_eq!(
            canonical.state(),
            crate::audit::collection::CollectionState::NotApplicable
        );
    }

    #[test]
    fn page_health_probe_counter_cases() {
        transport_failure();
        measured_defects();
        partial_result();
        valid_empty();
        not_applicable();
    }
}
