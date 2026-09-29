//! HTTP probes: redirects, custom 404, resource caching, content types,
//! www/non-www consolidation and URL canonicalization.

use tracing::debug;

use super::{
    ContentTypeFinding, Custom404Check, PageHealthAnalysis, RedirectHop, ResourceCacheAudit,
    ResourceCacheFinding, UrlCanonicalizationCheck, WwwConsolidation,
};

pub(super) async fn run_http_probes(url: &str, a: &mut PageHealthAnalysis) {
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

    // Trailing-slash and http→https consistency (#542)
    a.url_canonicalization = url_canonicalization_result;

    // Redirect chain
    let hops: Vec<RedirectHop> = redirect_chain
        .into_iter()
        .filter(|(status, _)| *status >= 300 && *status < 400)
        .map(|(status, url)| RedirectHop { status, url })
        .collect();
    a.redirect_count = hops.len() as u32;
    a.redirect_chain = hops;

    // HTTP headers
    if let Some((uses_http2, compression, cache_control, server_timing_count, charset_header)) =
        header_result
    {
        a.uses_http2 = uses_http2;
        a.has_compression = compression || document_timing_indicates_compression(a);
        a.has_efficient_cache = cache_control
            .as_deref()
            .map(is_cache_policy_efficient)
            .unwrap_or(false);
        a.cache_control = cache_control;
        a.server_timing_count = server_timing_count;
        a.charset_header = charset_header;
    }
    if document_timing_indicates_compression(a) {
        a.has_compression = true;
    }
    a.resource_cache = resource_cache;
}

/// Probe main page headers: HTTP version, compression, Cache-Control, Server-Timing, charset.
async fn probe_headers(url: &str) -> Option<(bool, bool, Option<String>, u32, Option<String>)> {
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

    let charset_header = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split(';')
                .skip(1)
                .find_map(|part| part.trim().strip_prefix("charset="))
                .map(|c| c.trim_matches('"').to_ascii_lowercase())
        });

    Some((
        uses_http2,
        compression,
        cache_control,
        server_timing_count,
        charset_header,
    ))
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

    audit
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

/// Check trailing-slash and http→https redirect consistency (#542).
async fn check_url_canonicalization(url: &str) -> Option<UrlCanonicalizationCheck> {
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str()?;

    // Skip IPs and localhost — canonicalization redirects aren't meaningful there.
    if host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
        .ok()?;

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
            let orig_status = orig_resp.ok().map(|r| r.status().as_u16());
            let variant_status = variant_resp.ok().map(|r| r.status().as_u16());
            // Both variants answering 200 independently means neither redirects to a
            // single canonical form — a duplicate-content risk. A 30x on either side
            // (redirecting to the other) is the expected, consistent case.
            let inconsistent = orig_status == Some(200) && variant_status == Some(200);
            (variant_status, inconsistent)
        } else {
            (None, false)
        };

    let (http_status, http_to_https_missing) = if parsed.scheme() == "https" {
        let mut http_variant = parsed.clone();
        if http_variant.set_scheme("http").is_ok() {
            let resp = client.head(http_variant.as_str()).send().await.ok();
            let status = resp.as_ref().map(|r| r.status().as_u16());
            let location = resp
                .and_then(|r| r.headers().get("location").cloned())
                .and_then(|v| v.to_str().ok().map(String::from))
                .unwrap_or_default();
            let redirects_to_https =
                matches!(status, Some(301) | Some(302) | Some(307) | Some(308))
                    && location.starts_with("https://");
            (status, !redirects_to_https)
        } else {
            // Scheme swap failed (should not happen for https → http, both
            // are special schemes with defined ports) — skip rather than
            // probe the unchanged https URL and misreport it as a missing
            // HTTP→HTTPS redirect.
            (None, false)
        }
    } else {
        (None, false)
    };

    Some(UrlCanonicalizationCheck {
        trailing_slash_variant_status,
        trailing_slash_inconsistent,
        http_status,
        http_to_https_missing,
    })
}

pub(super) fn document_timing_indicates_compression(a: &PageHealthAnalysis) -> bool {
    let (Some(transfer), Some(decoded)) = (a.document_transfer_bytes, a.document_decoded_bytes)
    else {
        return false;
    };
    decoded > 0 && transfer > 0 && (transfer as f64) < (decoded as f64 * 0.8)
}
