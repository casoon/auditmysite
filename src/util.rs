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
/// long, whole path segments in the middle replaced by `…` — the host and the
/// last path segment always stay readable, since the end of a path is what
/// tells sibling pages apart. `truncate_url` turned `/aktuelles/news/hafen`
/// and `/aktuelles/news/bahn` into the same `https://www.example.de/akt...`
/// (plan 92); cutting at a character count split host and slug mid-word
/// (`www.casoon.…o-entwicklung-rostock/`, #1039). A URL with no segment to
/// drop is returned whole, even above `max_len`. Safe for multi-byte UTF-8.
pub fn short_url(url: &str, max_len: usize) -> String {
    let bare = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    if bare.chars().count() <= max_len {
        return bare.to_string();
    }
    let Some(slash) = bare.find('/') else {
        return bare.to_string();
    };
    let (host, path) = bare.split_at(slash);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let Some((last, leading)) = segments.split_last() else {
        return bare.to_string();
    };
    if leading.is_empty() {
        return bare.to_string();
    }
    let tail = format!("/…/{last}{}", if path.ends_with('/') { "/" } else { "" });
    let tail_len = tail.chars().count();
    let mut head = host.to_string();
    for segment in leading {
        let candidate = format!("{head}/{segment}");
        if candidate.chars().count() + tail_len > max_len {
            break;
        }
        head = candidate;
    }
    format!("{head}{tail}")
}

/// Text for a narrow table cell, cut at the last word boundary that fits and
/// ended with `…` — never in the middle of a word or of the text (#1039).
/// A first word longer than `max_chars` is cut at the character limit.
/// Safe for multi-byte UTF-8.
pub fn truncate_words(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let room: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    let cut = room
        .rfind(char::is_whitespace)
        .filter(|&at| at > 0)
        .map_or(room.as_str(), |at| &room[..at]);
    let cut = cut.trim_end_matches(|c: char| c.is_whitespace() || ",;:–—-".contains(c));
    format!("{cut}…")
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
        assert_eq!(a, "www.example.de/…/hafen-hamburg-erweiterung");
        assert_eq!(short_url("https://example.de/kurz", 40), "example.de/kurz");
        // Whole leading segments are kept while they fit.
        assert_eq!(
            short_url("https://www.example.de/aktuelles/news/hafen/", 34),
            "www.example.de/aktuelles/…/hafen/"
        );
        // Host and one segment: nothing to drop, nothing cut mid-word.
        assert_eq!(
            short_url("https://www.casoon.de/astro-entwicklung-rostock/", 34),
            "www.casoon.de/astro-entwicklung-rostock/"
        );
    }

    #[test]
    fn truncate_words_ends_at_a_word() {
        assert_eq!(
            truncate_words("Allgemeine Geschäftsbedingungen", 36),
            "Allgemeine Geschäftsbedingungen"
        );
        assert_eq!(
            truncate_words("White-Label Webentwicklung für Agenturen – CASOON", 36),
            "White-Label Webentwicklung für…"
        );
        assert_eq!(
            truncate_words("Astro im Unternehmen – Enterprise Webentwicklung", 36),
            "Astro im Unternehmen – Enterprise…"
        );
        assert_eq!(
            truncate_words("Unbreakablewordthatislong", 10),
            "Unbreakab…"
        );
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
