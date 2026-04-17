//! Page health analysis
//!
//! HTTP probes and DOM inspections that don't belong in the technical SEO
//! module: soft-404 detection, meta-refresh, frames, URL structure,
//! redirect detection, www/non-www consolidation, and basic HTML validation.

use chromiumoxide::Page;
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

    /// Aggregated issue list for report rendering
    pub issues: Vec<PageHealthIssue>,
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
    let mut analysis = PageHealthAnalysis::default();

    // URL analysis (pure Rust, no CDP)
    analyze_url(url, &mut analysis);

    // DOM inspection via single JS evaluate
    if let Err(e) = run_dom_inspection(page, url, &mut analysis).await {
        warn!("Page health DOM inspection failed: {}", e);
    }

    // HTTP probes (reqwest, concurrent)
    run_http_probes(url, &mut analysis).await;

    // Aggregate issues
    analysis.issues = collect_issues(&analysis);

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

    // Collect HTML issues
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
    a.html_issues = html_issues;

    Ok(())
}

// ─── HTTP probes ─────────────────────────────────────────────────────────────

async fn run_http_probes(url: &str, a: &mut PageHealthAnalysis) {
    let Ok(parsed) = url::Url::parse(url) else {
        return;
    };
    let origin = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or(""));

    // Run soft-404 probe and www-consolidation check concurrently
    let probe_url = format!("{}/auditmysite-404-probe-xyz123", origin);
    let (soft_404_result, www_result) =
        tokio::join!(probe_status(&probe_url), check_www_consolidation(url));

    // Soft 404
    if let Some(status) = soft_404_result {
        a.soft_404_status = Some(status);
        a.is_soft_404 = status == 200;
        debug!("Soft-404 probe: {} → {}", probe_url, status);
    }

    // www consolidation
    a.www_consolidation = www_result;
}

/// Probe a URL and return its HTTP status (no redirect following).
async fn probe_status(url: &str) -> Option<u16> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
        .ok()?;

    client
        .get(url)
        .send()
        .await
        .ok()
        .map(|r| r.status().as_u16())
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

fn collect_issues(a: &PageHealthAnalysis) -> Vec<PageHealthIssue> {
    let mut issues = Vec::new();

    if a.is_soft_404 {
        issues.push(PageHealthIssue {
            issue_type: "soft_404".to_string(),
            message: format!(
                "Server gibt HTTP {} für nicht-existierende URLs zurück (Soft 404)",
                a.soft_404_status.unwrap_or(200)
            ),
            severity: "high".to_string(),
        });
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
            message: if delay == 0 {
                "Sofort-Weiterleitung via meta-refresh (SEO-schädlich, nutze 301-Redirect)"
                    .to_string()
            } else {
                format!("meta-refresh mit {}s Verzögerung gefunden", delay)
            },
            severity: if delay == 0 { "high" } else { "medium" }.to_string(),
        });
    }

    if a.frame_count > 0 {
        issues.push(PageHealthIssue {
            issue_type: "frames".to_string(),
            message: format!("{} veraltete <frame>-Elemente gefunden", a.frame_count),
            severity: "high".to_string(),
        });
    }

    if a.url_is_too_long {
        issues.push(PageHealthIssue {
            issue_type: "url_too_long".to_string(),
            message: format!(
                "URL mit {} Zeichen überschreitet Empfehlung (>115)",
                a.url_length
            ),
            severity: "low".to_string(),
        });
    }

    if a.url_is_too_deep {
        issues.push(PageHealthIssue {
            issue_type: "url_too_deep".to_string(),
            message: format!(
                "URL-Pfadtiefe {} überschreitet Empfehlung (>5 Ebenen)",
                a.url_path_depth
            ),
            severity: "low".to_string(),
        });
    }

    if a.url_has_query_params {
        issues.push(PageHealthIssue {
            issue_type: "dynamic_url".to_string(),
            message: "URL enthält Query-Parameter (dynamische URL)".to_string(),
            severity: "low".to_string(),
        });
    }

    if a.own_redirect_detected {
        issues.push(PageHealthIssue {
            issue_type: "redirect".to_string(),
            message: format!(
                "Seite leitet weiter zu: {}",
                a.own_final_url.as_deref().unwrap_or("(unbekannt)")
            ),
            severity: "medium".to_string(),
        });
    }

    if let Some(ref www) = a.www_consolidation {
        if !www.is_consolidated {
            issues.push(PageHealthIssue {
                issue_type: "www_not_consolidated".to_string(),
                message: "www und non-www Version sind nicht konsolidiert (kein 301-Redirect)"
                    .to_string(),
                severity: "medium".to_string(),
            });
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

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
            ..Default::default()
        };
        let issues = collect_issues(&a);
        assert!(issues.iter().any(|i| i.issue_type == "soft_404"));
    }

    #[test]
    fn test_collect_issues_meta_refresh() {
        let a = PageHealthAnalysis {
            has_meta_refresh: true,
            meta_refresh_content: Some("0; url=https://example.com".to_string()),
            ..Default::default()
        };
        let issues = collect_issues(&a);
        assert!(issues.iter().any(|i| i.issue_type == "meta_refresh"));
        assert_eq!(
            issues
                .iter()
                .find(|i| i.issue_type == "meta_refresh")
                .map(|i| i.severity.as_str()),
            Some("high")
        );
    }
}
