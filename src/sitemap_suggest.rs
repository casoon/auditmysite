//! Sitemap discovery helpers.
//!
//! Pure async utilities for finding and probing sitemap candidates for a base
//! URL. No interactive prompting here — that lives in runners.rs where the
//! mode context is known.

use colored::Colorize;

use auditmysite::audit::count_sitemap_entries_shallow;
use auditmysite::error::{AuditError, Result};
use auditmysite::util::build_browser_client;

pub async fn discover_populated_sitemap(base_url: &str) -> Result<Option<(String, usize)>> {
    let mut candidates = sitemap_candidates(base_url)?;
    for robots_sitemap in sitemap_candidates_from_robots(base_url).await {
        if !candidates.contains(&robots_sitemap) {
            candidates.push(robots_sitemap);
        }
    }

    for candidate in candidates {
        if let Some(count) = count_sitemap_entries_shallow(&candidate).await {
            return Ok(Some((candidate, count)));
        }
    }

    Ok(None)
}

pub fn sitemap_candidates(base_url: &str) -> Result<Vec<String>> {
    let parsed = url::Url::parse(base_url).map_err(|e| AuditError::ConfigError(e.to_string()))?;
    let base = parsed
        .join("/")
        .map_err(|e| AuditError::ConfigError(e.to_string()))?;

    let usual_suspects = [
        "sitemap.xml",
        "sitemap_index.xml",
        "sitemap-index.xml",
        "sitemaps.xml",
        "post-sitemap.xml",
        "page-sitemap.xml",
    ];

    let mut urls = Vec::new();
    for path in usual_suspects {
        if let Ok(candidate) = base.join(path) {
            urls.push(candidate.to_string());
        }
    }
    Ok(urls)
}

async fn sitemap_candidates_from_robots(base_url: &str) -> Vec<String> {
    let Ok(parsed) = url::Url::parse(base_url) else {
        return Vec::new();
    };
    let Ok(robots_url) = parsed.join("/robots.txt") else {
        return Vec::new();
    };

    let client = build_browser_client(10).unwrap_or_default();
    let Ok(response) = client.get(robots_url.clone()).send().await else {
        return Vec::new();
    };
    let Ok(body) = response.text().await else {
        return Vec::new();
    };

    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let (key, value) = trimmed.split_once(':')?;
            if key.trim().eq_ignore_ascii_case("sitemap") {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Check whether a URL is reachable before launching Chrome.
/// Only fails on network-level errors (DNS, timeout, connection refused).
/// Any HTTP response — including 4xx/5xx from bot-protection like Cloudflare —
/// is treated as "server reachable"; Chrome handles auth and bot challenges itself.
pub async fn check_url_reachable(url: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("{} {}", "Checking:".dimmed(), url);
    }

    let client = build_browser_client(10).map_err(|e| AuditError::ConfigError(e.to_string()))?;

    // Connection-level errors (TLS reset by Cloudflare, refused) are silently ignored —
    // Chrome uses a different TLS stack and often succeeds where reqwest fails.
    // Only abort on timeout: that means the host is genuinely unreachable.
    match client.head(url).send().await {
        Ok(_) => {}
        Err(e) if e.is_timeout() => {
            return Err(AuditError::ConfigError(format!(
                "Domain unreachable (timeout): {}\n  Please check your internet connection and URL.",
                url
            )));
        }
        Err(e) => {
            tracing::debug!("Preflight HEAD failed ({}); proceeding with Chrome", e);
        }
    }

    Ok(())
}

pub fn looks_like_base_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    (parsed.path().is_empty() || parsed.path() == "/")
        && parsed.query().is_none()
        && parsed.fragment().is_none()
}
