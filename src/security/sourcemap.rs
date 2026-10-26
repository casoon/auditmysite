//! Public source-map leak check (#538).
//!
//! Extracts `<script src>`/`<link rel="stylesheet" href>` asset URLs from the
//! audited page's HTML, reads each asset's trailing
//! `//# sourceMappingURL=...` (JS) or `/*# sourceMappingURL=... */` (CSS)
//! comment, and probes whether the referenced `.map` file is itself publicly
//! reachable. A reachable source map is a real information leak (original
//! source, comments, internal file paths) — this only reports presence, not
//! the map's contents.

use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

use crate::audit::collection::{reason, Collected};

/// Cap on how many JS/CSS assets get their sourceMappingURL checked, largest
/// pages first in document order — bounds probe time on asset-heavy pages.
const MAX_ASSETS_CHECKED: usize = 20;

/// Result of the public source-map leak probe.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMapLeakAudit {
    /// Number of JS/CSS assets whose sourceMappingURL comment was checked
    pub assets_checked: u32,
    /// Assets whose referenced `.map` file is publicly reachable (HTTP 2xx)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub leaks: Vec<SourceMapLeak>,
}

/// A single publicly reachable source map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapLeak {
    /// The JS/CSS asset that references the map
    pub asset_url: String,
    /// The resolved, publicly reachable source-map URL
    pub map_url: String,
}

/// Probe the page's loaded scripts/stylesheets for publicly reachable source maps.
///
/// Fetches the page HTML itself (independent of any CDP session — this is a
/// pure HTTP check, matching the rest of the security module) to find asset
/// URLs, then probes each asset's own trailing sourceMappingURL comment.
///
/// A page that could not be fetched is no page without leaks; assets that
/// could not be fetched make the check partial (#774).
pub async fn audit_source_maps(page_url: &str) -> Collected<SourceMapLeakAudit> {
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("auditmysite-probe/1.0")
        .build()
    {
        Ok(client) => client,
        Err(_) => return Collected::Failed(reason::HTTP_REQUEST_FAILED),
    };

    let Ok(page_resp) = client.get(page_url).send().await else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };
    let Ok(html) = page_resp.text().await else {
        return Collected::Failed(reason::HTTP_REQUEST_FAILED);
    };

    let asset_urls = extract_asset_urls(page_url, &html);
    if asset_urls.is_empty() {
        return Collected::Empty(SourceMapLeakAudit::default());
    }

    let mut audit = SourceMapLeakAudit::default();
    let mut unanswered = 0usize;
    for asset_url in asset_urls.iter().take(MAX_ASSETS_CHECKED) {
        let Ok(resp) = client.get(asset_url).send().await else {
            unanswered += 1;
            continue;
        };
        if !resp.status().is_success() {
            continue;
        }
        let Ok(body) = resp.text().await else {
            continue;
        };
        audit.assets_checked += 1;

        let Some(map_ref) = extract_source_mapping_url(&body) else {
            continue;
        };
        // Inline (data:) source maps carry the map itself, not a reference —
        // nothing reachable over HTTP to leak.
        if map_ref.starts_with("data:") {
            continue;
        }
        let Some(map_url) = Url::parse(asset_url)
            .ok()
            .and_then(|base| base.join(&map_ref).ok())
        else {
            continue;
        };

        if let Ok(map_resp) = client.head(map_url.as_str()).send().await {
            if map_resp.status().is_success() {
                debug!("Public source map leak: {} -> {}", asset_url, map_url);
                audit.leaks.push(SourceMapLeak {
                    asset_url: asset_url.clone(),
                    map_url: map_url.to_string(),
                });
            }
        }
    }

    sourcemap_outcome(audit, unanswered)
}

#[cfg(test)]
pub(crate) fn sourcemap_outcome_for_tests(
    audit: SourceMapLeakAudit,
    unanswered: usize,
) -> Collected<SourceMapLeakAudit> {
    sourcemap_outcome(audit, unanswered)
}

/// Every asset answered: measured; some: partial; none: failed.
fn sourcemap_outcome(
    audit: SourceMapLeakAudit,
    unanswered: usize,
) -> Collected<SourceMapLeakAudit> {
    if unanswered == 0 {
        Collected::Measured(audit)
    } else if audit.assets_checked == 0 && audit.leaks.is_empty() {
        Collected::Failed(reason::HTTP_REQUEST_FAILED)
    } else {
        Collected::Partial {
            value: audit,
            gaps: vec!["asset_request"],
        }
    }
}

/// Extract distinct `<script src>` and `<link rel="stylesheet" href>` asset
/// URLs from the page HTML, in document order, resolved against `page_url`.
fn extract_asset_urls(page_url: &str, html: &str) -> Vec<String> {
    let dom: RcDom = parse_document(RcDom::default(), Default::default()).one(html);
    let mut raw = Vec::new();
    collect_asset_hrefs(&dom.document, &mut raw);

    let Ok(base) = Url::parse(page_url) else {
        return Vec::new();
    };
    let mut seen = std::collections::BTreeSet::new();
    raw.into_iter()
        .filter_map(|href| {
            let resolved = base.join(&href).ok()?.to_string();
            seen.insert(resolved.clone()).then_some(resolved)
        })
        .collect()
}

/// Depth-first DOM walk collecting `<script src>` and
/// `<link rel="stylesheet" href>` attribute values.
fn collect_asset_hrefs(handle: &Handle, out: &mut Vec<String>) {
    if let NodeData::Element { name, attrs, .. } = &handle.data {
        let tag = name.local.as_ref();
        let attrs = attrs.borrow();
        let attr = |attr_name: &str| {
            attrs
                .iter()
                .find(|a| a.name.local.as_ref().eq_ignore_ascii_case(attr_name))
                .map(|a| a.value.to_string())
        };

        if tag.eq_ignore_ascii_case("script") {
            if let Some(src) = attr("src") {
                if !src.trim().is_empty() {
                    out.push(src);
                }
            }
        } else if tag.eq_ignore_ascii_case("link") {
            let is_stylesheet = attr("rel")
                .map(|r| {
                    r.to_ascii_lowercase()
                        .split_whitespace()
                        .any(|t| t == "stylesheet")
                })
                .unwrap_or(false);
            if is_stylesheet {
                if let Some(href) = attr("href") {
                    if !href.trim().is_empty() {
                        out.push(href);
                    }
                }
            }
        }
    }

    for child in handle.children.borrow().iter() {
        collect_asset_hrefs(child, out);
    }
}

/// Extract the value of a trailing `//# sourceMappingURL=...` (JS) or
/// `/*# sourceMappingURL=... */` (CSS) comment, if present in the asset's
/// last few lines (the comment is conventionally the last line of the file).
fn extract_source_mapping_url(body: &str) -> Option<String> {
    body.lines().rev().take(5).find_map(|line| {
        let line = line.trim();
        let marker = "sourceMappingURL=";
        let idx = line.find(marker)?;
        let rest = &line[idx + marker.len()..];
        let value = rest.trim().trim_end_matches("*/").trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_asset_urls_finds_scripts_and_stylesheets() {
        let html = r#"
            <html><head>
                <link rel="stylesheet" href="/css/app.css">
                <link rel="icon" href="/favicon.ico">
                <script src="/js/app.js"></script>
                <script>console.log("inline")</script>
            </head></html>
        "#;
        let urls = extract_asset_urls("https://example.com/page", html);
        assert_eq!(
            urls,
            vec![
                "https://example.com/css/app.css",
                "https://example.com/js/app.js",
            ]
        );
    }

    #[test]
    fn extract_asset_urls_dedupes() {
        let html = r#"<script src="/js/app.js"></script><script src="/js/app.js"></script>"#;
        let urls = extract_asset_urls("https://example.com/page", html);
        assert_eq!(urls, vec!["https://example.com/js/app.js"]);
    }

    #[test]
    fn extract_source_mapping_url_finds_js_comment() {
        let body = "console.log(1);\n//# sourceMappingURL=app.js.map\n";
        assert_eq!(
            extract_source_mapping_url(body),
            Some("app.js.map".to_string())
        );
    }

    #[test]
    fn extract_source_mapping_url_finds_css_comment() {
        let body = "body{color:red}\n/*# sourceMappingURL=app.css.map */\n";
        assert_eq!(
            extract_source_mapping_url(body),
            Some("app.css.map".to_string())
        );
    }

    #[test]
    fn extract_source_mapping_url_absent_returns_none() {
        let body = "console.log(1);\n";
        assert_eq!(extract_source_mapping_url(body), None);
    }

    #[test]
    fn extract_source_mapping_url_ignores_inline_data_uri_at_call_site() {
        // The extractor itself doesn't special-case data: — the caller does,
        // so it should still be extracted here.
        let body = "//# sourceMappingURL=data:application/json;base64,abc123\n";
        assert!(extract_source_mapping_url(body)
            .unwrap()
            .starts_with("data:"));
    }
}
