//! Render-blocking resource analysis via CDP.
//!
//! Detects scripts and stylesheets in <head> that block page rendering,
//! and breaks down first-party vs. third-party resource sizes.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;

use crate::error::{AuditError, Result};

/// A resource that blocks initial rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingResource {
    /// Resource URL (truncated to 120 chars)
    pub url: String,
    /// Transfer size in bytes (0 if not yet loaded or not in timing)
    pub transfer_bytes: u64,
    /// "script" or "css"
    pub kind: String,
}

/// Render-blocking analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBlockingAnalysis {
    /// Scripts in <head> without defer/async
    pub blocking_scripts: Vec<BlockingResource>,
    /// CSS <link rel=stylesheet> in <head> for screen/all media
    pub blocking_css: Vec<BlockingResource>,
    /// Total transfer size of all blocking resources in bytes
    pub blocking_transfer_bytes: u64,
    /// First-party transfer size (same hostname as page) in bytes
    pub first_party_bytes: u64,
    /// Third-party transfer size (different hostname) in bytes
    pub third_party_bytes: u64,
    /// Number of distinct third-party origins
    pub third_party_origin_count: u32,
    /// Actionable suggestions
    pub suggestions: Vec<String>,
}

impl RenderBlockingAnalysis {
    /// Total number of blocking resources
    pub fn blocking_count(&self) -> usize {
        self.blocking_scripts.len() + self.blocking_css.len()
    }

    /// True if there are any render-blocking resources
    pub fn has_blocking(&self) -> bool {
        self.blocking_count() > 0
    }
}

/// Analyze render-blocking resources on a loaded page via CDP.
pub async fn analyze_render_blocking(
    page: &Page,
    page_url: &str,
) -> Result<RenderBlockingAnalysis> {
    info!("Analyzing render-blocking resources...");

    let js = r#"
    (() => {
        const result = {};

        // Scripts in <head> without defer or async (render-blocking)
        result.blockingScriptUrls = Array.from(
            document.head ? document.head.querySelectorAll('script[src]') : []
        )
        .filter(s => !s.defer && !s.async && !(s.type || '').includes('module'))
        .map(s => s.src)
        .filter(Boolean);

        // CSS <link rel=stylesheet> in <head> for screen/all media (render-blocking)
        result.blockingCssUrls = Array.from(
            document.head ? document.head.querySelectorAll('link[rel="stylesheet"]') : []
        )
        .filter(l => {
            const m = (l.media || 'all').trim().toLowerCase();
            return m === '' || m === 'all' || m === 'screen';
        })
        .map(l => l.href)
        .filter(Boolean);

        // Resource timing: build url → transferSize map
        const timing = {};
        for (const r of performance.getEntriesByType('resource')) {
            timing[r.name] = r.transferSize || 0;
        }
        result.resourceTiming = timing;

        // All resource URLs and sizes for first/third party split
        result.allResources = Array.from(performance.getEntriesByType('resource')).map(r => ({
            url: r.name,
            transferSize: r.transferSize || 0,
        }));

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Render-blocking analysis failed: {e}")))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();

    let timing = parsed["resourceTiming"]
        .as_object()
        .cloned()
        .unwrap_or_default();

    let lookup_size = |url: &str| -> u64 { timing.get(url).and_then(|v| v.as_u64()).unwrap_or(0) };

    let blocking_scripts: Vec<BlockingResource> = parsed["blockingScriptUrls"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .map(|url| BlockingResource {
            url: truncate(url, 120),
            transfer_bytes: lookup_size(url),
            kind: "script".to_string(),
        })
        .collect();

    let blocking_css: Vec<BlockingResource> = parsed["blockingCssUrls"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .map(|url| BlockingResource {
            url: truncate(url, 120),
            transfer_bytes: lookup_size(url),
            kind: "css".to_string(),
        })
        .collect();

    let blocking_transfer_bytes: u64 = blocking_scripts
        .iter()
        .chain(blocking_css.iter())
        .map(|r| r.transfer_bytes)
        .sum();

    // First vs. third party split
    let page_host = Url::parse(page_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default();

    let mut first_party_bytes = 0u64;
    let mut third_party_bytes = 0u64;
    let mut third_party_origins = std::collections::HashSet::new();

    if let Some(resources) = parsed["allResources"].as_array() {
        for r in resources {
            let url_str = r["url"].as_str().unwrap_or("");
            let size = r["transferSize"].as_u64().unwrap_or(0);
            let is_first_party = Url::parse(url_str)
                .ok()
                .and_then(|u| u.host_str().map(|h| h == page_host))
                .unwrap_or(true);
            if is_first_party {
                first_party_bytes += size;
            } else {
                third_party_bytes += size;
                if let Ok(u) = Url::parse(url_str) {
                    if let Some(h) = u.host_str() {
                        third_party_origins.insert(h.to_string());
                    }
                }
            }
        }
    }

    let suggestions = build_suggestions(
        &blocking_scripts,
        &blocking_css,
        third_party_bytes,
        third_party_origins.len() as u32,
    );

    info!(
        "Render-blocking: {} scripts, {} CSS, {} KB blocking",
        blocking_scripts.len(),
        blocking_css.len(),
        blocking_transfer_bytes / 1024,
    );

    Ok(RenderBlockingAnalysis {
        blocking_scripts,
        blocking_css,
        blocking_transfer_bytes,
        first_party_bytes,
        third_party_bytes,
        third_party_origin_count: third_party_origins.len() as u32,
        suggestions,
    })
}

fn build_suggestions(
    scripts: &[BlockingResource],
    css: &[BlockingResource],
    third_party_bytes: u64,
    third_party_origins: u32,
) -> Vec<String> {
    let mut s = Vec::new();

    if !scripts.is_empty() {
        s.push(format!(
            "{} Script{} im <head> ohne defer/async blockier{} das Rendering. \
             Füge defer oder async hinzu.",
            scripts.len(),
            if scripts.len() == 1 { "" } else { "s" },
            if scripts.len() == 1 { "t" } else { "en" },
        ));
    }

    if !css.is_empty() {
        s.push(format!(
            "{} CSS-Datei{} blockier{} das Rendering. \
             Inline kritisches CSS oder nutze <link rel=\"preload\">.",
            css.len(),
            if css.len() == 1 { "" } else { "en" },
            if css.len() == 1 { "t" } else { "en" },
        ));
    }

    if third_party_bytes > 100_000 {
        s.push(format!(
            "Third-Party-Ressourcen: {:.0} KB von {} Domain{}. \
             Prüfe, ob alle Skripte wirklich notwendig sind.",
            third_party_bytes as f64 / 1024.0,
            third_party_origins,
            if third_party_origins == 1 { "" } else { "s" },
        ));
    }

    s
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_suggestions_single_script() {
        let script = BlockingResource {
            url: "https://example.com/app.js".to_string(),
            transfer_bytes: 50_000,
            kind: "script".to_string(),
        };
        let s = build_suggestions(&[script], &[], 0, 0);
        assert_eq!(s.len(), 1);
        assert!(s[0].contains("defer"));
    }

    #[test]
    fn test_build_suggestions_third_party() {
        let s = build_suggestions(&[], &[], 200_000, 3);
        assert_eq!(s.len(), 1);
        assert!(s[0].contains("Third-Party"));
    }

    #[test]
    fn test_build_suggestions_empty() {
        let s = build_suggestions(&[], &[], 0, 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_blocking_count() {
        let analysis = RenderBlockingAnalysis {
            blocking_scripts: vec![BlockingResource {
                url: "a.js".into(),
                transfer_bytes: 1000,
                kind: "script".into(),
            }],
            blocking_css: vec![
                BlockingResource {
                    url: "b.css".into(),
                    transfer_bytes: 500,
                    kind: "css".into(),
                },
                BlockingResource {
                    url: "c.css".into(),
                    transfer_bytes: 300,
                    kind: "css".into(),
                },
            ],
            blocking_transfer_bytes: 1800,
            first_party_bytes: 10000,
            third_party_bytes: 5000,
            third_party_origin_count: 2,
            suggestions: vec![],
        };
        assert_eq!(analysis.blocking_count(), 3);
        assert!(analysis.has_blocking());
    }
}
