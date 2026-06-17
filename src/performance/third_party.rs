//! Third-party script attribution per origin (#138).
//!
//! Groups all page resources by their full hostname and classifies each
//! hostname as first-party or third-party relative to the page host.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;

use crate::error::{AuditError, Result};

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
    })
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let boundary = s
        .char_indices()
        .take_while(|(i, _)| *i <= max.saturating_sub(3))
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}…", &s[..boundary])
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
        };
        assert!(attr.is_significant(500_000)); // 40 %
        assert!(!attr.is_significant(2_000_000)); // 10 %
    }
}
