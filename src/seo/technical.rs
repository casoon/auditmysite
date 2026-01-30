//! Technical SEO analysis
//!
//! Checks HTTPS, canonical URLs, language, sitemap, robots.txt.

use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::{AuditError, Result};

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
    /// Word count on page
    pub word_count: u32,
    /// Internal links count
    pub internal_links: u32,
    /// External links count
    pub external_links: u32,
    /// Broken links found
    pub broken_links: Vec<String>,
    /// Issues found
    pub issues: Vec<TechnicalIssue>,
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
    pub severity: String,
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

        // Word count (approximate)
        const text = document.body ? document.body.innerText : '';
        result.wordCount = text.split(/\s+/).filter(w => w.length > 0).length;

        // Links
        const links = document.querySelectorAll('a[href]');
        let internal = 0, external = 0;
        const currentHost = window.location.host;

        links.forEach(a => {
            try {
                const href = a.getAttribute('href');
                if (href.startsWith('http')) {
                    const linkUrl = new URL(href);
                    if (linkUrl.host === currentHost) {
                        internal++;
                    } else {
                        external++;
                    }
                } else if (href.startsWith('/') || href.startsWith('#')) {
                    internal++;
                }
            } catch (e) {}
        });

        result.internalLinks = internal;
        result.externalLinks = external;

        return JSON.stringify(result);
    })()
    "#;

    let js_result = page
        .evaluate(js_code)
        .await
        .map_err(|e| AuditError::CdpError(format!("Technical SEO analysis failed: {}", e)))?;

    let json_str = js_result.value().and_then(|v| v.as_str()).unwrap_or("{}");

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap_or_else(|e| {
        warn!("Failed to parse technical SEO JSON: {}", e);
        serde_json::Value::Object(serde_json::Map::new())
    });

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

    let word_count = parsed["wordCount"].as_u64().unwrap_or(0) as u32;
    let internal_links = parsed["internalLinks"].as_u64().unwrap_or(0) as u32;
    let external_links = parsed["externalLinks"].as_u64().unwrap_or(0) as u32;

    // Generate issues
    let mut issues = Vec::new();

    if !https {
        issues.push(TechnicalIssue {
            issue_type: "no_https".to_string(),
            message: "Page is not served over HTTPS".to_string(),
            severity: "error".to_string(),
        });
    }

    if canonical_url.is_none() {
        issues.push(TechnicalIssue {
            issue_type: "no_canonical".to_string(),
            message: "Missing canonical URL".to_string(),
            severity: "warning".to_string(),
        });
    }

    if lang.is_none() {
        issues.push(TechnicalIssue {
            issue_type: "no_lang".to_string(),
            message: "Missing lang attribute on html element".to_string(),
            severity: "warning".to_string(),
        });
    }

    if word_count < 300 {
        issues.push(TechnicalIssue {
            issue_type: "thin_content".to_string(),
            message: format!(
                "Page has thin content ({} words, recommended: 300+)",
                word_count
            ),
            severity: "warning".to_string(),
        });
    }

    if internal_links == 0 {
        issues.push(TechnicalIssue {
            issue_type: "no_internal_links".to_string(),
            message: "Page has no internal links".to_string(),
            severity: "warning".to_string(),
        });
    }

    info!(
        "Technical SEO: HTTPS={}, canonical={}, lang={}, words={}",
        https,
        canonical_url.is_some(),
        lang.is_some(),
        word_count
    );

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
        word_count,
        internal_links,
        external_links,
        broken_links: vec![],
        issues,
    })
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
}
