//! Unminified JavaScript and CSS detection (#111).
//!
//! Uses PerformanceResourceTiming to identify JS/CSS resources that were
//! served without minification, based on the compression ratio heuristic
//! (high decodedBodySize / transferSize ratio → lots of redundant whitespace).
//!
//! Heuristic: a compression ratio ≥ 4.0 on a file larger than 10 KB that
//! does not carry a `.min.` or `-min.` marker in its URL is flagged as
//! likely unminified.  The estimated savings are (decoded_bytes − decoded_bytes
//! / 3) as a conservative minification target (≈ 66 % size reduction typical
//! of production minifiers).

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AuditError, Result};

/// A single JS or CSS resource that appears to be unminified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnminifiedAsset {
    /// Resource URL (truncated to 120 chars)
    pub url: String,
    /// "script" or "css"
    pub kind: String,
    /// Uncompressed size of the resource (bytes)
    pub decoded_bytes: u64,
    /// Compressed transfer size (0 if cached)
    pub transfer_bytes: u64,
    /// Estimated bytes wasted vs. a minified equivalent
    pub savings_bytes: u64,
}

/// JavaScript resource that looks like legacy/polyfill payload for modern browsers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyJavascriptAsset {
    /// Resource URL (truncated to 120 chars)
    pub url: String,
    /// Detected legacy/polyfill family.
    pub signature: String,
    /// Decoded size of the resource (bytes)
    pub decoded_bytes: u64,
    /// Compressed transfer size (0 if cached)
    pub transfer_bytes: u64,
    /// Conservative wasted-byte estimate for modern browser delivery.
    pub wasted_bytes: u64,
}

/// Results of the unminified-asset analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinificationAnalysis {
    /// JS files that appear to be unminified
    pub unminified_scripts: Vec<UnminifiedAsset>,
    /// CSS files that appear to be unminified
    pub unminified_styles: Vec<UnminifiedAsset>,
    /// Total estimated bytes that could be saved by minifying
    pub total_savings_bytes: u64,
    /// Total number of unminified resources detected
    pub total_unminified_count: u32,
    /// Script resources that look like legacy/polyfill payload.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legacy_scripts: Vec<LegacyJavascriptAsset>,
    /// Conservative estimated bytes wasted by legacy/polyfill payload.
    #[serde(default)]
    pub total_legacy_wasted_bytes: u64,
}

/// Detect unminified JS and CSS resources on the loaded page.
pub async fn analyze_minification(page: &Page) -> Result<MinificationAnalysis> {
    info!("Analyzing unminified JS/CSS assets...");

    let js = r#"
    (() => {
        var resources = performance.getEntriesByType('resource');
        return JSON.stringify(resources.map(function(r) {
            return {
                url: r.name,
                initiatorType: r.initiatorType || 'other',
                decodedBodySize: r.decodedBodySize || 0,
                transferSize: r.transferSize || 0
            };
        }));
    })()
    "#;

    let result = page
        .evaluate(js)
        .await
        .map_err(|e| AuditError::CdpError(format!("Minification analysis JS failed: {e}")))?;

    let json_str = result.value().and_then(|v| v.as_str()).unwrap_or("[]");
    let entries: Vec<RawEntry> = serde_json::from_str(json_str).unwrap_or_default();

    let mut unminified_scripts: Vec<UnminifiedAsset> = Vec::new();
    let mut unminified_styles: Vec<UnminifiedAsset> = Vec::new();
    let mut legacy_scripts: Vec<LegacyJavascriptAsset> = Vec::new();

    for entry in entries {
        let kind = classify_kind(&entry.url, &entry.initiator_type);
        let kind = match kind.as_str() {
            "script" | "css" => kind,
            _ => continue,
        };

        // Skip data: URIs, blobs, and very small resources
        if entry.url.starts_with("data:") || entry.url.starts_with("blob:") {
            continue;
        }
        if entry.decoded_body_size < 10_000 {
            continue;
        }

        if kind == "script" {
            if let Some(signature) = legacy_javascript_signature(&entry.url) {
                let wasted_bytes =
                    legacy_wasted_bytes(entry.decoded_body_size, entry.transfer_size);
                legacy_scripts.push(LegacyJavascriptAsset {
                    url: truncate(&entry.url, 120),
                    signature: signature.to_string(),
                    decoded_bytes: entry.decoded_body_size,
                    transfer_bytes: entry.transfer_size,
                    wasted_bytes,
                });
            }
        }

        // Skip explicitly named minified files
        let url_lower = entry.url.to_lowercase();
        if url_lower.contains(".min.") || url_lower.contains("-min.") {
            continue;
        }

        // Heuristic: high compression ratio = unminified source
        let is_likely_unminified = if entry.transfer_size > 0 {
            let ratio = entry.decoded_body_size as f64 / entry.transfer_size as f64;
            ratio >= 4.0
        } else {
            // Cached: rely on file size alone — large files without .min. are suspicious
            entry.decoded_body_size >= 100_000
        };

        if !is_likely_unminified {
            continue;
        }

        // Conservative savings estimate: minification typically achieves ~66 %
        let savings_bytes = entry
            .decoded_body_size
            .saturating_sub(entry.decoded_body_size / 3);

        let asset = UnminifiedAsset {
            url: truncate(&entry.url, 120),
            kind: kind.clone(),
            decoded_bytes: entry.decoded_body_size,
            transfer_bytes: entry.transfer_size,
            savings_bytes,
        };

        if kind == "script" {
            unminified_scripts.push(asset);
        } else {
            unminified_styles.push(asset);
        }
    }

    // Sort largest savings first
    unminified_scripts.sort_by_key(|a| std::cmp::Reverse(a.savings_bytes));
    unminified_styles.sort_by_key(|a| std::cmp::Reverse(a.savings_bytes));
    legacy_scripts.sort_by_key(|a| std::cmp::Reverse(a.wasted_bytes));

    let total_savings_bytes: u64 = unminified_scripts
        .iter()
        .chain(unminified_styles.iter())
        .map(|a| a.savings_bytes)
        .sum();
    let total_unminified_count = (unminified_scripts.len() + unminified_styles.len()) as u32;
    let total_legacy_wasted_bytes = legacy_scripts.iter().map(|a| a.wasted_bytes).sum();

    info!(
        "Minification: {} unminified resources, {:.1} KB estimated savings, {} legacy/polyfill script(s)",
        total_unminified_count,
        total_savings_bytes as f64 / 1024.0,
        legacy_scripts.len()
    );

    Ok(MinificationAnalysis {
        unminified_scripts,
        unminified_styles,
        total_savings_bytes,
        total_unminified_count,
        legacy_scripts,
        total_legacy_wasted_bytes,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct RawEntry {
    url: String,
    #[serde(rename = "initiatorType")]
    initiator_type: String,
    #[serde(rename = "decodedBodySize")]
    decoded_body_size: u64,
    #[serde(rename = "transferSize")]
    transfer_size: u64,
}

fn classify_kind(url: &str, initiator_type: &str) -> String {
    let url_lower = url.to_lowercase();
    // Font files loaded via CSS @font-face carry initiatorType="css" in
    // PerformanceResourceTiming, but they are not CSS — exclude them first.
    if url_lower.ends_with(".woff2")
        || url_lower.ends_with(".woff")
        || url_lower.ends_with(".ttf")
        || url_lower.ends_with(".eot")
        || url_lower.ends_with(".otf")
    {
        return "other".to_string();
    }
    if initiator_type == "script" || url_lower.ends_with(".js") || url_lower.contains(".js?") {
        "script"
    } else if initiator_type == "css" || url_lower.ends_with(".css") || url_lower.contains(".css?")
    {
        "css"
    } else {
        "other"
    }
    .to_string()
}

fn legacy_javascript_signature(url: &str) -> Option<&'static str> {
    let lower = url.to_ascii_lowercase();
    if lower.contains("core-js") || lower.contains("core.min.js") {
        Some("core-js")
    } else if lower.contains("regenerator-runtime") || lower.contains("regeneratorruntime") {
        Some("regenerator-runtime")
    } else if lower.contains("@babel/runtime")
        || lower.contains("babel-polyfill")
        || lower.contains("babel_runtime")
        || lower.contains("babel-runtime")
    {
        Some("babel-runtime")
    } else if lower.contains("polyfill.io")
        || lower.contains("/polyfill")
        || lower.contains("polyfills.")
        || lower.contains("polyfills-")
    {
        Some("polyfill-bundle")
    } else if lower.contains("es5-shim") || lower.contains("es6-shim") {
        Some("es-shim")
    } else {
        None
    }
}

fn legacy_wasted_bytes(decoded_bytes: u64, transfer_bytes: u64) -> u64 {
    if decoded_bytes > 0 {
        decoded_bytes
    } else {
        transfer_bytes
    }
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
    fn test_classify_kind_css() {
        assert_eq!(
            classify_kind("https://cdn.example.com/style.css", "css"),
            "css"
        );
        assert_eq!(
            classify_kind("https://cdn.example.com/style.css", "other"),
            "css"
        );
    }

    #[test]
    fn test_classify_kind_other() {
        assert_eq!(
            classify_kind("https://cdn.example.com/image.png", "img"),
            "other"
        );
    }

    #[test]
    fn test_legacy_javascript_signature() {
        assert_eq!(
            legacy_javascript_signature("https://cdn.example.com/core-js/3.37/index.min.js"),
            Some("core-js")
        );
        assert_eq!(
            legacy_javascript_signature("https://cdn.example.com/regenerator-runtime/runtime.js"),
            Some("regenerator-runtime")
        );
        assert_eq!(
            legacy_javascript_signature("https://polyfill.io/v3/polyfill.min.js"),
            Some("polyfill-bundle")
        );
        assert_eq!(
            legacy_javascript_signature("https://cdn.example.com/app.modern.js"),
            None
        );
    }

    #[test]
    fn test_legacy_wasted_bytes_prefers_decoded_size() {
        assert_eq!(legacy_wasted_bytes(120_000, 30_000), 120_000);
        assert_eq!(legacy_wasted_bytes(0, 30_000), 30_000);
    }

    #[test]
    fn test_truncate() {
        let long = "a".repeat(200);
        let result = truncate(&long, 120);
        assert!(result.len() <= 123); // 120 chars + "…"
    }
}
