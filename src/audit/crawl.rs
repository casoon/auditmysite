//! Minimal same-domain crawler for batch discovery.
//!
//! This is the first step toward a fuller crawl/link-graph system:
//! - start from a seed URL
//! - follow same-domain links
//! - breadth-first with depth and page limits

use std::collections::{HashMap, HashSet, VecDeque};

use reqwest::{redirect::Policy, Client};
use tracing::{debug, info, warn};
use url::Url;

use crate::audit::report::{BrokenLink, BrokenLinkSeverity, CrawlDiagnostics, RedirectChain};
use crate::error::{AuditError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrawlNode {
    pub url: String,
    pub depth: usize,
    pub links_out: Vec<String>,
    pub external_links_out: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrawlResult {
    pub seed_url: String,
    pub pages: Vec<CrawlNode>,
}

impl CrawlResult {
    pub fn urls(&self) -> Vec<String> {
        self.pages.iter().map(|p| p.url.clone()).collect()
    }
}

struct LinkCheckResult {
    final_status: Option<u16>,
    hops: u8,
    final_url: String,
    error: Option<String>,
}

async fn check_link(client: &Client, url: &str) -> LinkCheckResult {
    let mut current = url.to_string();
    let mut hops = 0u8;
    const MAX_HOPS: u8 = 6;

    loop {
        let result = client
            .head(&current)
            .header("User-Agent", "auditmysite-link-checker/1.0")
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                if status.is_redirection() && hops < MAX_HOPS {
                    if let Some(loc) = resp
                        .headers()
                        .get("location")
                        .and_then(|v| v.to_str().ok())
                    {
                        let next = if let Ok(base) = Url::parse(&current) {
                            base.join(loc)
                                .map(|u| u.to_string())
                                .unwrap_or_else(|_| loc.to_string())
                        } else {
                            loc.to_string()
                        };
                        // strip fragment
                        let next = Url::parse(&next)
                            .ok()
                            .map(|mut u| {
                                u.set_fragment(None);
                                u.to_string()
                            })
                            .unwrap_or(next);
                        current = next;
                        hops += 1;
                        continue;
                    }
                }
                return LinkCheckResult {
                    final_status: Some(status.as_u16()),
                    hops,
                    final_url: current,
                    error: None,
                };
            }
            Err(e) => {
                // HEAD not supported by some servers — try GET for the initial request only
                if hops == 0 {
                    match client
                        .get(url)
                        .header("User-Agent", "auditmysite-link-checker/1.0")
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            return LinkCheckResult {
                                final_status: Some(resp.status().as_u16()),
                                hops: 0,
                                final_url: url.to_string(),
                                error: None,
                            };
                        }
                        Err(e2) => {
                            return LinkCheckResult {
                                final_status: None,
                                hops: 0,
                                final_url: url.to_string(),
                                error: Some(e2.to_string()),
                            };
                        }
                    }
                }
                return LinkCheckResult {
                    final_status: None,
                    hops,
                    final_url: current,
                    error: Some(e.to_string()),
                };
            }
        }
    }
}

fn broken_link_severity(
    is_external: bool,
    status: Option<u16>,
    error: &Option<String>,
) -> BrokenLinkSeverity {
    match (is_external, status) {
        (false, Some(s)) if s >= 400 && s < 500 => BrokenLinkSeverity::High,
        (false, Some(s)) if s >= 500 => BrokenLinkSeverity::Medium,
        (false, None) if error.is_some() => BrokenLinkSeverity::High,
        (true, Some(s)) if s >= 400 && s < 500 => BrokenLinkSeverity::Medium,
        (true, Some(s)) if s >= 500 => BrokenLinkSeverity::Low,
        (true, None) if error.is_some() => BrokenLinkSeverity::Low,
        _ => BrokenLinkSeverity::Low,
    }
}

fn unique_internal_targets(crawl: &CrawlResult) -> Vec<String> {
    let mut unique = HashSet::new();
    for page in &crawl.pages {
        for target in &page.links_out {
            unique.insert(target.clone());
        }
    }
    let mut targets: Vec<String> = unique.into_iter().collect();
    targets.sort();
    targets
}

fn unique_external_targets(crawl: &CrawlResult) -> Vec<String> {
    let mut unique = HashSet::new();
    for page in &crawl.pages {
        for target in &page.external_links_out {
            unique.insert(target.clone());
        }
    }
    let mut targets: Vec<String> = unique.into_iter().collect();
    targets.sort();
    // Cap at 100 external checks to keep runtime reasonable
    targets.truncate(100);
    targets
}

pub async fn analyze_crawl_links(crawl: &CrawlResult) -> CrawlDiagnostics {
    let client = Client::builder()
        .redirect(Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| Client::new());

    // ── Internal links ──────────────────────────────────────────────────────
    let unique_internal = unique_internal_targets(crawl);
    let mut internal_results: HashMap<String, LinkCheckResult> = HashMap::new();
    for target in &unique_internal {
        internal_results.insert(target.clone(), check_link(&client, target).await);
    }

    let mut broken_internal_links = Vec::new();
    let mut redirect_chains: Vec<RedirectChain> = Vec::new();

    for page in &crawl.pages {
        for target in &page.links_out {
            let Some(res) = internal_results.get(target) else {
                continue;
            };

            // Redirect chain (> 1 hop)
            if res.hops > 1
                && !redirect_chains
                    .iter()
                    .any(|r: &RedirectChain| &r.target_url == target)
            {
                redirect_chains.push(RedirectChain {
                    source_url: page.url.clone(),
                    target_url: target.clone(),
                    final_url: res.final_url.clone(),
                    hops: res.hops,
                    is_external: false,
                });
            }

            let is_broken = matches!(res.final_status, Some(s) if s >= 400)
                || (res.final_status.is_none() && res.error.is_some());
            if is_broken {
                let severity = broken_link_severity(false, res.final_status, &res.error);
                broken_internal_links.push(BrokenLink {
                    source_url: page.url.clone(),
                    target_url: target.clone(),
                    status_code: res.final_status,
                    error: res.error.clone(),
                    is_external: false,
                    redirect_hops: res.hops,
                    severity,
                });
            }
        }
    }

    // ── External links ──────────────────────────────────────────────────────
    let unique_external = unique_external_targets(crawl);
    let mut external_results: HashMap<String, LinkCheckResult> = HashMap::new();
    for target in &unique_external {
        external_results.insert(target.clone(), check_link(&client, target).await);
    }

    let mut broken_external_links = Vec::new();

    for page in &crawl.pages {
        for target in &page.external_links_out {
            let Some(res) = external_results.get(target) else {
                continue;
            };

            // External redirect chain
            if res.hops > 1
                && !redirect_chains
                    .iter()
                    .any(|r: &RedirectChain| &r.target_url == target)
            {
                redirect_chains.push(RedirectChain {
                    source_url: page.url.clone(),
                    target_url: target.clone(),
                    final_url: res.final_url.clone(),
                    hops: res.hops,
                    is_external: true,
                });
            }

            let is_broken = matches!(res.final_status, Some(s) if s >= 400)
                || (res.final_status.is_none() && res.error.is_some());
            if is_broken {
                let severity = broken_link_severity(true, res.final_status, &res.error);
                broken_external_links.push(BrokenLink {
                    source_url: page.url.clone(),
                    target_url: target.clone(),
                    status_code: res.final_status,
                    error: res.error.clone(),
                    is_external: true,
                    redirect_hops: res.hops,
                    severity,
                });
            }
        }
    }

    // Sort by severity (High first)
    broken_internal_links.sort_by_key(|b| match b.severity {
        BrokenLinkSeverity::High => 0,
        BrokenLinkSeverity::Medium => 1,
        BrokenLinkSeverity::Low => 2,
    });
    broken_external_links.sort_by_key(|b| match b.severity {
        BrokenLinkSeverity::High => 0,
        BrokenLinkSeverity::Medium => 1,
        BrokenLinkSeverity::Low => 2,
    });
    // Sort redirect chains by hop count descending
    redirect_chains.sort_by(|a, b| b.hops.cmp(&a.hops));

    CrawlDiagnostics {
        seed_url: crawl.seed_url.clone(),
        discovered_urls: crawl.pages.len(),
        checked_internal_links: unique_internal.len(),
        broken_internal_links,
        checked_external_links: unique_external.len(),
        broken_external_links,
        redirect_chains,
    }
}

pub async fn crawl_site(seed_url: &str, max_urls: usize, max_depth: usize) -> Result<CrawlResult> {
    let seed = Url::parse(seed_url)?;
    let host = seed
        .host_str()
        .ok_or_else(|| AuditError::InvalidUrl {
            url: seed_url.to_string(),
            reason: "Missing host".to_string(),
        })?
        .to_string();

    let effective_max_urls = if max_urls == 0 { usize::MAX } else { max_urls };

    info!(
        "Starting crawl from {} (max_urls={}, max_depth={})",
        seed_url, effective_max_urls, max_depth
    );

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut pages = Vec::new();

    queue.push_back((normalize_discovered_url(seed.clone())?, 0usize));

    while let Some((current, depth)) = queue.pop_front() {
        if pages.len() >= effective_max_urls {
            break;
        }
        if !visited.insert(current.as_str().to_string()) {
            continue;
        }

        let html = match fetch_html(current.as_str()).await {
            Ok(html) => html,
            Err(err) => {
                warn!("Failed to fetch crawl page {}: {}", current, err);
                continue;
            }
        };

        let (same_domain_links, external_links) = extract_links(&current, &host, &html);
        let mut links_out = same_domain_links;
        links_out.sort();
        links_out.dedup();
        let mut external_links_out = external_links;
        external_links_out.sort();
        external_links_out.dedup();

        debug!(
            "Crawled {} at depth {} ({} same-domain links, {} external links)",
            current,
            depth,
            links_out.len(),
            external_links_out.len()
        );

        if depth < max_depth {
            for link in &links_out {
                if !visited.contains(link) {
                    queue.push_back((link.clone(), depth + 1));
                }
            }
        }

        pages.push(CrawlNode {
            url: current,
            depth,
            links_out,
            external_links_out,
        });
    }

    Ok(CrawlResult {
        seed_url: seed_url.to_string(),
        pages,
    })
}

async fn fetch_html(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !content_type.is_empty() && !content_type.contains("text/html") {
        return Err(AuditError::OutputError {
            reason: format!("URL did not return HTML content: {url}"),
        });
    }

    Ok(response.text().await?)
}

fn extract_links(
    base_url: &str,
    expected_host: &str,
    html: &str,
) -> (Vec<String>, Vec<String>) {
    let mut internal_links = Vec::new();
    let mut external_links = Vec::new();
    let bytes = html.as_bytes();
    let mut idx = 0;

    while let Some(pos) = find_case_insensitive(bytes, idx, b"href") {
        idx = pos + 4;
        let mut cursor = idx;

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }

        let quote = bytes[cursor];
        let (start, end) = if quote == b'"' || quote == b'\'' {
            let start = cursor + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != quote {
                end += 1;
            }
            (start, end)
        } else {
            let start = cursor;
            let mut end = start;
            while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'>' {
                end += 1;
            }
            (start, end)
        };

        if start >= bytes.len() || end <= start || end > bytes.len() {
            continue;
        }

        let href = &html[start..end];
        if let Some(url) = normalize_link(base_url, expected_host, href) {
            internal_links.push(url);
        } else if let Some(url) = normalize_external_link(base_url, expected_host, href) {
            if external_links.len() < 50 {
                external_links.push(url);
            }
        }
        idx = end;
    }

    (internal_links, external_links)
}

fn normalize_link(base_url: &str, expected_host: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
        || href.starts_with("data:")
    {
        return None;
    }

    let base = Url::parse(base_url).ok()?;
    let url = base.join(href).ok()?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return None;
    }
    if url.host_str()? != expected_host {
        return None;
    }
    normalize_discovered_url(url).ok()
}

fn normalize_external_link(base_url: &str, expected_host: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
        || href.starts_with("data:")
    {
        return None;
    }
    let base = Url::parse(base_url).ok()?;
    let url = base.join(href).ok()?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return None;
    }
    let host = url.host_str()?;
    if host == expected_host {
        return None; // same domain = internal
    }
    let mut u = url.clone();
    u.set_fragment(None);
    Some(u.to_string())
}

fn normalize_discovered_url(mut url: Url) -> Result<String> {
    url.set_fragment(None);

    let path = url.path().to_string();
    if path != "/" && path.ends_with('/') {
        url.set_path(path.trim_end_matches('/'));
    }

    Ok(url.to_string())
}

fn find_case_insensitive(haystack: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    haystack[start..]
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
        .map(|offset| start + offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links_filters_same_domain_and_noise() {
        let html = r##"
            <a href="/about">About</a>
            <a href="https://www.casoon.de/services/">Services</a>
            <a href="mailto:hello@example.com">Mail</a>
            <a href="#contact">Fragment</a>
            <a href="https://external.example.org/">External</a>
        "##;

        let (internal_links, external_links) =
            extract_links("https://www.casoon.de", "www.casoon.de", html);
        assert_eq!(
            internal_links,
            vec![
                "https://www.casoon.de/about".to_string(),
                "https://www.casoon.de/services".to_string()
            ]
        );
        assert_eq!(
            external_links,
            vec!["https://external.example.org/".to_string()]
        );
    }

    #[test]
    fn test_normalize_link_keeps_query_and_strips_fragment() {
        let url = normalize_link(
            "https://www.casoon.de",
            "www.casoon.de",
            "/seo-marketing/?ref=nav#hero",
        )
        .unwrap();
        assert_eq!(url, "https://www.casoon.de/seo-marketing?ref=nav");
    }

    #[test]
    fn test_normalize_link_rejects_other_hosts() {
        let url = normalize_link(
            "https://www.casoon.de",
            "www.casoon.de",
            "https://example.org/path",
        );
        assert!(url.is_none());
    }

    #[test]
    fn test_normalize_external_link_captures_external() {
        let url = normalize_external_link(
            "https://www.casoon.de",
            "www.casoon.de",
            "https://example.org/path#section",
        )
        .unwrap();
        assert_eq!(url, "https://example.org/path");
    }

    #[test]
    fn test_normalize_external_link_rejects_same_domain() {
        let url = normalize_external_link(
            "https://www.casoon.de",
            "www.casoon.de",
            "https://www.casoon.de/about",
        );
        assert!(url.is_none());
    }

    #[test]
    fn test_unique_internal_targets_deduplicates_across_pages() {
        let crawl = CrawlResult {
            seed_url: "https://www.casoon.de".to_string(),
            pages: vec![
                CrawlNode {
                    url: "https://www.casoon.de".to_string(),
                    depth: 0,
                    links_out: vec![
                        "https://www.casoon.de/about".to_string(),
                        "https://www.casoon.de/contact".to_string(),
                    ],
                    external_links_out: vec![],
                },
                CrawlNode {
                    url: "https://www.casoon.de/about".to_string(),
                    depth: 1,
                    links_out: vec!["https://www.casoon.de/contact".to_string()],
                    external_links_out: vec![],
                },
            ],
        };

        let targets = unique_internal_targets(&crawl);
        assert_eq!(
            targets,
            vec![
                "https://www.casoon.de/about".to_string(),
                "https://www.casoon.de/contact".to_string()
            ]
        );
    }
}
