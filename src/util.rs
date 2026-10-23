//! Shared utility functions

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;

pub const BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Build a browser-like header set that passes basic bot-detection heuristics.
/// Mirrors the Sec-CH-UA / Sec-Fetch-* fingerprint of a real Chrome on macOS.
pub fn browser_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let h = |s: &str| HeaderValue::from_str(s).expect("valid header value");
    headers.insert("accept", h("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"));
    headers.insert("accept-language", h("en-US,en;q=0.9,de;q=0.8"));
    headers.insert("accept-encoding", h("gzip"));
    headers.insert("cache-control", h("no-cache"));
    // Sec-CH-UA omitted intentionally: combining browser client-hints with a
    // non-Chrome TLS fingerprint (rustls ≠ BoringSSL) worsens bot-detection mismatches.
    headers.insert("sec-fetch-dest", h("document"));
    headers.insert("sec-fetch-mode", h("navigate"));
    headers.insert("sec-fetch-site", h("none"));
    headers.insert("sec-fetch-user", h("?1"));
    headers.insert("upgrade-insecure-requests", h("1"));
    headers
}

/// Build an HTTP client that looks like a browser to basic bot-detection filters.
pub fn build_browser_client(timeout_secs: u64) -> reqwest::Result<Client> {
    Client::builder()
        .default_headers(browser_headers())
        .user_agent(BROWSER_UA)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(timeout_secs))
        .gzip(true)
        .build()
}

/// Truncate a string for display purposes (safe for multi-byte UTF-8)
pub fn truncate_url(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let target = max_len.saturating_sub(3);
    // Find the last char boundary at or before target
    let boundary = s
        .char_indices()
        .take_while(|(i, _)| *i <= target)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    format!("{}...", &s[..boundary])
}

/// A URL for a narrow table cell: the scheme dropped and, when still too
/// long, the middle cut instead of the end — the end of a path is what tells
/// sibling pages apart. `truncate_url` turned `/aktuelles/news/hafen` and
/// `/aktuelles/news/bahn` into the same `https://www.example.de/akt...`
/// (plan 92). Safe for multi-byte UTF-8.
pub fn short_url(url: &str, max_len: usize) -> String {
    let bare = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let chars: Vec<char> = bare.chars().collect();
    if chars.len() <= max_len || max_len < 8 {
        return bare.to_string();
    }
    let head = max_len / 3;
    let tail = max_len - head - 1;
    let start: String = chars[..head].iter().collect();
    let end: String = chars[chars.len() - tail..].iter().collect();
    format!("{start}…{end}")
}

/// Truncate a string for display purposes, ellipsizing with `…` (safe for
/// multi-byte UTF-8). Distinct from [`truncate_url`], which appends an ASCII
/// `...` ellipsis instead — this preserves the `…` style the `performance`
/// modules' finding messages already render.
pub fn truncate_ellipsis(s: &str, max: usize) -> String {
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
    fn short_url_keeps_the_distinguishing_end() {
        let a = short_url(
            "https://www.example.de/aktuelles/news/hafen-hamburg-erweiterung",
            40,
        );
        let b = short_url(
            "https://www.example.de/aktuelles/news/bahnstrecke-nord-ausbau",
            40,
        );
        assert_ne!(a, b);
        assert!(a.ends_with("hafen-hamburg-erweiterung"), "{a}");
        assert!(a.starts_with("www.example"), "{a}");
        assert_eq!(a.chars().count(), 40);
        assert_eq!(short_url("https://example.de/kurz", 40), "example.de/kurz");
    }

    #[test]
    fn test_truncate_short_url() {
        assert_eq!(
            truncate_url("https://example.com", 30),
            "https://example.com"
        );
    }

    #[test]
    fn test_truncate_long_url() {
        let url = "https://example.com/very/long/path/that/exceeds/limit";
        let result = truncate_url(url, 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_ellipsis_long_string() {
        let long = "a".repeat(200);
        let result = truncate_ellipsis(&long, 120);
        assert!(result.len() <= 120);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_browser_headers_only_advertise_supported_encoding() {
        let headers = browser_headers();
        assert_eq!(headers["accept-encoding"], "gzip");
    }
}
