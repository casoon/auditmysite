//! Third-party script attribution per origin (#138).
//!
//! Groups all page resources by their full hostname and classifies each
//! hostname as first-party or third-party relative to the page host.

use chromiumoxide::cdp::browser_protocol::network::SetBlockedUrLsParams;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use url::Url;

use crate::browser::BrowserManager;
use crate::error::{AuditError, Result};
use crate::util::truncate_ellipsis as truncate;

use super::vitals::{extract_web_vitals, prepare_vitals_collection};

/// Per-origin resource summary for a single third-party domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyOrigin {
    /// Hostname of the third-party origin (e.g. "fonts.googleapis.com")
    pub origin: String,
    /// Total compressed transfer size across all resources from this origin
    pub transfer_bytes: u64,
    /// Number of resources loaded from this origin
    pub request_count: u32,
    /// Distinct resource kinds observed (e.g. "script", "css", "font", "img")
    pub resource_kinds: Vec<String>,
    /// URL of the largest single resource from this origin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub largest_url: Option<String>,
    /// Known provider, when the origin matches a built-in tracker classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Functional tracker category such as analytics, ads, social, or marketing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Transfer size of the largest single resource
    pub largest_bytes: u64,
}

/// Aggregated third-party attribution for the audited page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyAttribution {
    /// Per-origin breakdown, sorted by transfer_bytes descending
    pub origins: Vec<ThirdPartyOrigin>,
    /// Number of distinct third-party origins
    pub total_origins: u32,
    /// Total transfer bytes across all third-party resources
    pub total_bytes: u64,
    /// Total number of third-party requests
    pub total_requests: u32,
    /// Isolated main-thread impact per origin, measured via CDP request
    /// blocking (#531). Only populated when `--isolate-third-party-impact`
    /// is set; empty otherwise (opt-in, requires extra page reloads).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub isolated_impact: Vec<ThirdPartyImpact>,
}

/// Isolated main-thread impact of a single third-party origin (#531).
///
/// Measured by reloading the page once with this origin's requests blocked
/// via CDP `Network.setBlockedURLs` and diffing the resulting Total Blocking
/// Time against the baseline TBT already measured during the page's normal,
/// unblocked audit pass — no separate baseline-only reload is performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyImpact {
    /// Third-party origin this measurement isolates (matches `ThirdPartyOrigin.origin`)
    pub origin: String,
    /// CDP `Network.setBlockedURLs` wildcard pattern used to block this origin
    pub blocked_url_pattern: String,
    /// TBT (ms) measured during the page's normal, unblocked audit pass
    pub baseline_tbt_ms: f64,
    /// TBT (ms) measured with this origin's requests blocked
    pub without_script_tbt_ms: f64,
    /// Estimated main-thread impact attributable to this origin: `max(0, baseline - without)`
    pub estimated_impact_ms: f64,
}

impl ThirdPartyAttribution {
    /// True when third-party resources exceed 20 % of total page transfer bytes.
    pub fn is_significant(&self, page_total_bytes: u64) -> bool {
        page_total_bytes > 0 && self.total_bytes * 100 / page_total_bytes >= 20
    }
}

/// Analyze third-party resource attribution for a loaded page.
pub async fn analyze_third_party_attribution(
    page: &Page,
    page_url: &str,
) -> Result<ThirdPartyAttribution> {
    info!("Analyzing third-party attribution...");

    let js = r#"
    (() => {
        var resources = performance.getEntriesByType('resource');
        return JSON.stringify(resources.map(function(r) {
            return {
                url: r.name,
                transferSize: r.transferSize || 0,
                initiatorType: r.initiatorType || 'other'
            };
        }));
    })()
    "#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Third-party attribution JS failed: {e}")))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("[]");
    let entries: Vec<ResourceEntry> = serde_json::from_str(json_str).unwrap_or_default();

    let page_host = Url::parse(page_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default();

    // Aggregate per origin
    let mut origin_map: std::collections::HashMap<String, OriginAccum> =
        std::collections::HashMap::new();

    for entry in &entries {
        let host = match Url::parse(&entry.url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
        {
            Some(h) => h,
            None => continue,
        };

        if host == page_host {
            continue; // first-party — skip
        }

        let kind = classify_kind(&entry.url, &entry.initiator_type);
        let accum = origin_map
            .entry(host.clone())
            .or_insert_with(OriginAccum::new);
        accum.transfer_bytes += entry.transfer_size;
        accum.request_count += 1;
        if !accum.kinds.contains(&kind) {
            accum.kinds.push(kind);
        }
        if entry.transfer_size > accum.largest_bytes {
            accum.largest_bytes = entry.transfer_size;
            accum.largest_url = Some(truncate(&entry.url, 120));
        }
    }

    let total_bytes: u64 = origin_map.values().map(|a| a.transfer_bytes).sum();
    let total_requests: u32 = origin_map.values().map(|a| a.request_count).sum();
    let total_origins = origin_map.len() as u32;

    let mut origins: Vec<ThirdPartyOrigin> = origin_map
        .into_iter()
        .map(|(host, a)| {
            let (provider, category) = classify_origin(&host);
            ThirdPartyOrigin {
                origin: host,
                provider,
                category,
                transfer_bytes: a.transfer_bytes,
                request_count: a.request_count,
                resource_kinds: a.kinds,
                largest_url: a.largest_url,
                largest_bytes: a.largest_bytes,
            }
        })
        .collect();

    origins.sort_by_key(|o| std::cmp::Reverse(o.transfer_bytes));

    info!(
        "Third-party attribution: {} origins, {} requests, {:.1} KB",
        total_origins,
        total_requests,
        total_bytes as f64 / 1024.0
    );

    Ok(ThirdPartyAttribution {
        origins,
        total_origins,
        total_bytes,
        total_requests,
        isolated_impact: Vec::new(),
    })
}

// ── Isolated third-party impact (#531) ─────────────────────────────────────

/// Cap on how many third-party origins get isolated per page.
///
/// Each isolated origin costs one additional full page reload. Five keeps
/// the extra-reload budget in the same order of magnitude as the existing
/// throttled-performance passes (3 reloads, `ThrottleProfile::AUTO_PROFILES`)
/// that already run under `--full`, even on a page with dozens of
/// third-party origins.
pub const MAX_ISOLATED_ORIGINS: usize = 5;

/// Select the origins to isolate: the top `cap` origins from `origins`.
///
/// `origins` is expected to already be sorted by `transfer_bytes` descending
/// (as produced by `analyze_third_party_attribution`), so this is a plain
/// prefix-take — the origins responsible for the most third-party bytes (and,
/// in practice, usually the most requests) are isolated first.
pub fn select_origins_to_isolate(
    origins: &[ThirdPartyOrigin],
    cap: usize,
) -> Vec<&ThirdPartyOrigin> {
    origins.iter().take(cap).collect()
}

/// Compute the isolated-impact delta for one origin's blocked-vs-baseline TBT
/// measurement. Pure so the delta math is unit-testable without a browser.
pub(super) fn compute_isolated_impact(
    origin: &str,
    blocked_url_pattern: &str,
    baseline_tbt_ms: f64,
    without_script_tbt_ms: f64,
) -> ThirdPartyImpact {
    ThirdPartyImpact {
        origin: origin.to_string(),
        blocked_url_pattern: blocked_url_pattern.to_string(),
        baseline_tbt_ms,
        without_script_tbt_ms,
        estimated_impact_ms: (baseline_tbt_ms - without_script_tbt_ms).max(0.0),
    }
}

/// Block (or unblock, when `patterns` is empty) URL patterns via CDP
/// `Network.setBlockedURLs`.
async fn set_blocked_urls(page: &Page, patterns: &[String]) -> Result<()> {
    page.execute(SetBlockedUrLsParams::new(patterns.to_vec()))
        .await
        .map_err(|e| AuditError::CdpError(format!("Network.setBlockedURLs failed: {e}")))?;
    Ok(())
}

/// Measure the isolated main-thread impact of the top third-party origins on
/// a page (#531, opt-in via `--isolate-third-party-impact`).
///
/// For each of the top [`MAX_ISOLATED_ORIGINS`] origins (by transfer bytes),
/// reloads `page` with that origin's requests blocked via CDP
/// `Network.setBlockedURLs`, re-measures TBT using the same
/// `extract_web_vitals` mechanism as the normal audit pass, and diffs it
/// against `baseline_tbt_ms` — the TBT already measured during the page's
/// normal, unblocked audit pass. No separate baseline-only reload is done.
///
/// Reuses the already-loaded `page`/`browser` (same pattern as the
/// throttled-performance multi-pass in `pipeline.rs`) rather than opening a
/// new browser instance. A failed reload or vitals read for one origin is
/// logged and skipped — the origin is simply absent from the result, the
/// remaining origins still get isolated.
///
/// Costly: one full page reload per isolated origin. Callers must only
/// invoke this when both performance checking and the explicit
/// `--isolate-third-party-impact` opt-in are active (see `PipelineConfig`).
pub async fn isolate_third_party_impact(
    page: &Page,
    browser: &BrowserManager,
    url: &str,
    origins: &[ThirdPartyOrigin],
    baseline_tbt_ms: f64,
) -> Vec<ThirdPartyImpact> {
    let mut results = Vec::new();

    for origin in select_origins_to_isolate(origins, MAX_ISOLATED_ORIGINS) {
        let pattern = format!("*{}*", origin.origin);
        info!("Isolating third-party impact for {}", origin.origin);

        if let Err(e) = set_blocked_urls(page, std::slice::from_ref(&pattern)).await {
            warn!("Failed to block {} for isolation: {}", origin.origin, e);
            continue;
        }

        if let Err(e) = prepare_vitals_collection(page).await {
            warn!(
                "Vitals injection failed while isolating {}: {}",
                origin.origin, e
            );
            let _ = set_blocked_urls(page, &[]).await;
            continue;
        }

        if let Err(e) = browser.navigate(page, url).await {
            warn!("Navigation failed while isolating {}: {}", origin.origin, e);
            let _ = set_blocked_urls(page, &[]).await;
            continue;
        }

        let without_tbt = match extract_web_vitals(page).await {
            Ok(vitals) => vitals.tbt.map(|m| m.value),
            Err(e) => {
                warn!(
                    "Vitals extraction failed while isolating {}: {}",
                    origin.origin, e
                );
                None
            }
        };

        // Always clear blocking again, regardless of outcome, before the
        // next origin (or any subsequent pass) navigates.
        let _ = set_blocked_urls(page, &[]).await;

        if let Some(without_tbt) = without_tbt {
            results.push(compute_isolated_impact(
                &origin.origin,
                &pattern,
                baseline_tbt_ms,
                without_tbt,
            ));
        }
    }

    results
}

// ── Helpers ───────────────────────────────────────────────────────────────────

struct OriginAccum {
    transfer_bytes: u64,
    request_count: u32,
    kinds: Vec<String>,
    largest_bytes: u64,
    largest_url: Option<String>,
}

impl OriginAccum {
    fn new() -> Self {
        Self {
            transfer_bytes: 0,
            request_count: 0,
            kinds: vec![],
            largest_bytes: 0,
            largest_url: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ResourceEntry {
    url: String,
    #[serde(rename = "transferSize")]
    transfer_size: u64,
    #[serde(rename = "initiatorType")]
    initiator_type: String,
}

fn classify_kind(url: &str, initiator_type: &str) -> String {
    let url_lower = url.to_lowercase();
    if initiator_type == "script" || url_lower.ends_with(".js") {
        "script"
    } else if initiator_type == "css" || url_lower.ends_with(".css") {
        "css"
    } else if url_lower.contains(".woff")
        || url_lower.contains(".ttf")
        || url_lower.contains(".otf")
        || url_lower.contains(".eot")
    {
        "font"
    } else if initiator_type == "img"
        || url_lower.ends_with(".png")
        || url_lower.ends_with(".jpg")
        || url_lower.ends_with(".jpeg")
        || url_lower.ends_with(".webp")
        || url_lower.ends_with(".svg")
        || url_lower.ends_with(".gif")
    {
        "img"
    } else if url_lower.ends_with(".mp4")
        || url_lower.ends_with(".webm")
        || url_lower.ends_with(".mp3")
    {
        "media"
    } else {
        "other"
    }
    .to_string()
}

fn classify_origin(host: &str) -> (Option<String>, Option<String>) {
    let lower = host.to_ascii_lowercase();
    let pair = if lower.contains("googletagmanager.com")
        || lower.contains("google-analytics.com")
        || lower.contains("analytics.google.com")
    {
        ("Google", "analytics")
    } else if lower.contains("doubleclick.net") || lower.contains("googleadservices.com") {
        ("Google Ads", "ads")
    } else if lower.contains("facebook.com") || lower.contains("connect.facebook.net") {
        ("Meta", "social")
    } else if lower.contains("hotjar.com") {
        ("Hotjar", "analytics")
    } else if lower.contains("hubspot.com") || lower.contains("hs-analytics.net") {
        ("HubSpot", "marketing")
    } else if lower.contains("matomo") {
        ("Matomo", "analytics")
    } else {
        return (None, None);
    };
    (Some(pair.0.to_string()), Some(pair.1.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_kind_script() {
        assert_eq!(
            classify_kind("https://cdn.example.com/app.js", "script"),
            "script"
        );
        assert_eq!(
            classify_kind("https://cdn.example.com/app.js", "other"),
            "script"
        );
    }

    #[test]
    fn test_classify_kind_font() {
        assert_eq!(
            classify_kind("https://fonts.gstatic.com/s/font.woff2", "other"),
            "font"
        );
    }

    #[test]
    fn test_is_significant() {
        let attr = ThirdPartyAttribution {
            origins: vec![],
            total_origins: 0,
            total_bytes: 200_000,
            total_requests: 5,
            isolated_impact: vec![],
        };
        assert!(attr.is_significant(500_000)); // 40 %
        assert!(!attr.is_significant(2_000_000)); // 10 %
    }

    fn test_origin(host: &str, bytes: u64) -> ThirdPartyOrigin {
        ThirdPartyOrigin {
            origin: host.to_string(),
            transfer_bytes: bytes,
            request_count: 1,
            resource_kinds: vec!["script".to_string()],
            largest_url: None,
            provider: None,
            category: None,
            largest_bytes: bytes,
        }
    }

    #[test]
    fn select_origins_to_isolate_caps_at_the_given_number() {
        let origins: Vec<ThirdPartyOrigin> = (0..8)
            .map(|i| test_origin(&format!("host{i}.example.com"), (8 - i) * 1000))
            .collect();

        let selected = select_origins_to_isolate(&origins, MAX_ISOLATED_ORIGINS);

        assert_eq!(selected.len(), MAX_ISOLATED_ORIGINS);
        // Prefix-take preserves the caller's ordering (already sorted by bytes descending).
        assert_eq!(selected[0].origin, "host0.example.com");
        assert_eq!(selected[4].origin, "host4.example.com");
    }

    #[test]
    fn select_origins_to_isolate_returns_all_when_under_the_cap() {
        let origins = vec![
            test_origin("a.example.com", 100),
            test_origin("b.example.com", 50),
        ];
        let selected = select_origins_to_isolate(&origins, MAX_ISOLATED_ORIGINS);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn compute_isolated_impact_is_the_positive_baseline_minus_without_delta() {
        let impact = compute_isolated_impact("tag.example.com", "*tag.example.com*", 450.0, 120.0);
        assert_eq!(impact.origin, "tag.example.com");
        assert_eq!(impact.blocked_url_pattern, "*tag.example.com*");
        assert_eq!(impact.baseline_tbt_ms, 450.0);
        assert_eq!(impact.without_script_tbt_ms, 120.0);
        assert_eq!(impact.estimated_impact_ms, 330.0);
    }

    #[test]
    fn compute_isolated_impact_clamps_negative_delta_to_zero() {
        // Blocking a script should never make the page slower; if noise in
        // measurement makes "without" look higher than baseline, the impact
        // must be reported as 0, not negative.
        let impact =
            compute_isolated_impact("noise.example.com", "*noise.example.com*", 100.0, 140.0);
        assert_eq!(impact.estimated_impact_ms, 0.0);
    }
}
