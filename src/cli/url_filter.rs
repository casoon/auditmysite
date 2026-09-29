//! URL selection for batch inputs by path glob (`--include-path` /
//! `--exclude-path`, plan 67).
//!
//! Patterns are matched against the whole, percent-decoded URL path — no
//! scheme, host or query:
//! - `*` matches any run of characters within one path segment (not `/`),
//! - `?` matches exactly one character other than `/`,
//! - `**` matches any run of characters, including `/`,
//! - `/**/` additionally matches a single `/`, so `/blog/**/feed` also
//!   matches `/blog/feed`,
//! - every other character matches itself.
//!
//! A URL is selected when no include pattern is given or at least one
//! matches, and no exclude pattern matches.

/// Keep the URLs whose path passes the include/exclude patterns, in order.
///
/// With neither kind of pattern every URL is kept unchanged.
pub fn select_urls(urls: Vec<String>, include: &[String], exclude: &[String]) -> Vec<String> {
    if include.is_empty() && exclude.is_empty() {
        return urls;
    }
    urls.into_iter()
        .filter(|url| {
            let path = url_path(url);
            (include.is_empty() || include.iter().any(|p| path_matches(p, &path)))
                && !exclude.iter().any(|p| path_matches(p, &path))
        })
        .collect()
}

/// Whether `path` matches the glob `pattern` (see the module docs).
pub fn path_matches(pattern: &str, path: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let path: Vec<char> = path.chars().collect();
    glob_match(&pattern, &path)
}

fn glob_match(pattern: &[char], text: &[char]) -> bool {
    match pattern.first() {
        None => text.is_empty(),
        Some('*') if pattern.get(1) == Some(&'*') => {
            let rest = &pattern[2..];
            // `**/` may also stand for no segment at all: `/a/**/b` ~ `/a/b`.
            if rest.first() == Some(&'/') && glob_match(&rest[1..], text) {
                return true;
            }
            (0..=text.len()).any(|i| glob_match(rest, &text[i..]))
        }
        Some('*') => {
            let rest = &pattern[1..];
            for i in 0..=text.len() {
                if glob_match(rest, &text[i..]) {
                    return true;
                }
                if text.get(i) == Some(&'/') {
                    return false;
                }
            }
            false
        }
        Some('?') => {
            matches!(text.first(), Some(c) if *c != '/') && glob_match(&pattern[1..], &text[1..])
        }
        Some(c) => text.first() == Some(c) && glob_match(&pattern[1..], &text[1..]),
    }
}

/// The percent-decoded path of `url`; empty for an unparseable URL.
fn url_path(url: &str) -> String {
    url::Url::parse(url)
        .map(|u| percent_decode(u.path()))
        .unwrap_or_default()
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && bytes[i + 1].is_ascii_hexdigit()
            && bytes[i + 2].is_ascii_hexdigit()
        {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).expect("ASCII hex digits");
            out.push(u8::from_str_radix(hex, 16).expect("two hex digits"));
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls(paths: &[&str]) -> Vec<String> {
        paths
            .iter()
            .map(|p| format!("https://example.com{p}"))
            .collect()
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn single_star_stays_within_one_segment() {
        assert!(path_matches("/blog/*", "/blog/post-1"));
        assert!(path_matches("/blog/*", "/blog/"));
        assert!(!path_matches("/blog/*", "/blog/2026/post-1"));
        assert!(!path_matches("/blog/*", "/blog"));
        assert!(path_matches("/*/kontakt", "/de/kontakt"));
        assert!(!path_matches("/*/kontakt", "/de/at/kontakt"));
    }

    #[test]
    fn double_star_crosses_segments() {
        assert!(path_matches("/blog/**", "/blog/2026/post-1/"));
        assert!(path_matches("/blog/**", "/blog/"));
        assert!(!path_matches("/blog/**", "/blog"));
        assert!(!path_matches("/blog/**", "/blogroll/x"));
        assert!(path_matches("**/feed", "/a/b/feed"));
        assert!(path_matches("/**", "/"));
    }

    #[test]
    fn double_star_slash_also_matches_no_segment() {
        assert!(path_matches("/blog/**/feed", "/blog/feed"));
        assert!(path_matches("/blog/**/feed", "/blog/2026/09/feed"));
        assert!(!path_matches("/blog/**/feed", "/blog/feedback"));
    }

    #[test]
    fn question_mark_matches_one_non_slash_character() {
        assert!(path_matches("/page/?", "/page/2"));
        assert!(!path_matches("/page/?", "/page/12"));
        assert!(!path_matches("/a?b", "/a/b"));
    }

    #[test]
    fn literal_pattern_must_match_whole_path() {
        assert!(path_matches("/", "/"));
        assert!(!path_matches("/", "/about"));
        assert!(path_matches("/about", "/about"));
        assert!(!path_matches("/about", "/about/team"));
    }

    #[test]
    fn matches_the_decoded_path_without_query() {
        let selected = select_urls(
            vec![
                "https://example.com/%C3%BCber-uns?ref=x".to_string(),
                "https://example.com/kontakt".to_string(),
            ],
            &strings(&["/über-uns"]),
            &[],
        );
        assert_eq!(selected, vec!["https://example.com/%C3%BCber-uns?ref=x"]);
    }

    #[test]
    fn no_patterns_keep_everything() {
        let input = urls(&["/", "/a", "/b/c"]);
        assert_eq!(select_urls(input.clone(), &[], &[]), input);
    }

    #[test]
    fn include_keeps_any_match_and_preserves_order() {
        let selected = select_urls(
            urls(&["/", "/blog/a", "/shop/x", "/blog/b", "/about"]),
            &strings(&["/blog/**", "/about"]),
            &[],
        );
        assert_eq!(selected, urls(&["/blog/a", "/blog/b", "/about"]));
    }

    #[test]
    fn exclude_wins_over_include() {
        let selected = select_urls(
            urls(&["/blog/a", "/blog/tag/x", "/blog/b"]),
            &strings(&["/blog/**"]),
            &strings(&["/blog/tag/**"]),
        );
        assert_eq!(selected, urls(&["/blog/a", "/blog/b"]));
    }

    #[test]
    fn exclude_alone_keeps_the_rest() {
        let selected = select_urls(
            urls(&["/", "/tag/a", "/about"]),
            &[],
            &strings(&["/tag/**"]),
        );
        assert_eq!(selected, urls(&["/", "/about"]));
    }

    #[test]
    fn unparseable_url_only_survives_without_include() {
        let input = vec!["not a url".to_string()];
        assert!(select_urls(input.clone(), &strings(&["/**"]), &[]).is_empty());
        assert_eq!(select_urls(input.clone(), &[], &strings(&["/x"])), input);
    }

    #[test]
    fn percent_decode_leaves_malformed_escapes() {
        assert_eq!(percent_decode("/a%2"), "/a%2");
        assert_eq!(percent_decode("/a%zz"), "/a%zz");
        assert_eq!(percent_decode("/a%20b"), "/a b");
    }
}
