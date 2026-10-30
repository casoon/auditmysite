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

/// One stratum of a structural URL sample: how many candidates it held and
/// how many the draw took from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StratumDraw {
    pub key: String,
    pub discovered: usize,
    pub selected: usize,
}

/// Structural stratum of a URL: `[lang/]section[.ext]`.
///
/// `lang` is a leading path segment shaped like a language tag (`de`,
/// `pt-br`; any two-letter segment is taken as one); `section` is the next segment (`/` for the root and for a
/// single top-level file); `ext` a non-HTML file extension of the last
/// segment (`.pdf`). Needs no browser: the key is derived from the URL alone.
pub fn stratum_key(url: &str) -> String {
    let path = url_path(url);
    let mut segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let lang = match segments.first() {
        Some(first) if is_language_segment(first) => Some(segments.remove(0).to_lowercase()),
        _ => None,
    };
    let ext = segments
        .last()
        .and_then(|last| last.rsplit_once('.'))
        .map(|(_, ext)| ext.to_lowercase())
        .filter(|ext| !matches!(ext.as_str(), "html" | "htm" | "php") && ext.len() <= 5);
    let section = match segments.as_slice() {
        [] => "/".to_string(),
        [only] if only.contains('.') => "/".to_string(),
        [first, ..] => first.to_lowercase(),
    };
    let mut key = lang.map(|l| format!("{l}/")).unwrap_or_default();
    if !(section == "/" && !key.is_empty()) {
        key.push_str(&section);
    }
    if let Some(ext) = ext {
        key.push('.');
        key.push_str(&ext);
    }
    key
}

fn is_language_segment(segment: &str) -> bool {
    let (primary, region) = match segment.split_once(['-', '_']) {
        Some((p, r)) => (p, Some(r)),
        None => (segment, None),
    };
    primary.len() == 2
        && primary.chars().all(|c| c.is_ascii_alphabetic())
        && region.is_none_or(|r| {
            (2..=3).contains(&r.len()) && r.chars().all(|c| c.is_ascii_alphabetic())
        })
}

/// Draw at most `budget` URLs, spread over the structural strata.
///
/// Deterministic, no randomness: strata are served round-robin (largest
/// first, ties by key) so every stratum gets a URL before any gets a second;
/// within a stratum the picks are evenly spaced through its input order
/// instead of taking its head. With fewer strata than `budget` all strata are
/// covered; with more, the largest ones are. The selection keeps input order.
/// A zero `budget` or one covering every URL returns the input unchanged.
pub fn stratified_sample(urls: Vec<String>, budget: usize) -> (Vec<String>, Vec<StratumDraw>) {
    use std::collections::BTreeMap;

    if budget == 0 || budget >= urls.len() {
        return (urls, Vec::new());
    }
    let mut strata: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, url) in urls.iter().enumerate() {
        strata.entry(stratum_key(url)).or_default().push(i);
    }
    let mut order: Vec<(&String, &Vec<usize>)> = strata.iter().collect();
    order.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(b.0)));

    let mut quota = vec![0usize; order.len()];
    let mut remaining = budget;
    while remaining > 0 {
        let mut progressed = false;
        for (q, (_, members)) in quota.iter_mut().zip(&order) {
            if remaining > 0 && *q < members.len() {
                *q += 1;
                remaining -= 1;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }

    let mut chosen: Vec<usize> = Vec::with_capacity(budget);
    let mut draws = Vec::with_capacity(order.len());
    for ((key, members), q) in order.iter().zip(&quota) {
        chosen.extend((0..*q).map(|i| members[i * members.len() / q]));
        draws.push(StratumDraw {
            key: (*key).clone(),
            discovered: members.len(),
            selected: *q,
        });
    }
    chosen.sort_unstable();
    let selected = chosen.into_iter().map(|i| urls[i].clone()).collect();
    (selected, draws)
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

    #[test]
    fn stratum_key_reads_language_section_and_extension() {
        assert_eq!(stratum_key("https://e.com/"), "/");
        assert_eq!(stratum_key("https://e.com/impressum.html"), "/");
        assert_eq!(stratum_key("https://e.com/blog/a/b"), "blog");
        assert_eq!(stratum_key("https://e.com/de/blog/a"), "de/blog");
        assert_eq!(stratum_key("https://e.com/de/"), "de/");
        assert_eq!(stratum_key("https://e.com/pt-BR/shop/x"), "pt-br/shop");
        assert_eq!(stratum_key("https://e.com/docs/a.pdf"), "docs.pdf");
        assert_eq!(stratum_key("https://e.com/api/x"), "api");
    }

    fn site() -> Vec<String> {
        let mut v = Vec::new();
        for i in 0..40 {
            v.push(format!("https://e.com/blog/post-{i}"));
        }
        for i in 0..6 {
            v.push(format!("https://e.com/shop/item-{i}"));
        }
        v.push("https://e.com/".to_string());
        v.push("https://e.com/en/about".to_string());
        v
    }

    #[test]
    fn stratified_covers_every_stratum_within_budget() {
        let input = site();
        // first_n would take only blog posts.
        let (sel, draws) = stratified_sample(input.clone(), 6);
        assert_eq!(sel.len(), 6);
        assert_eq!(draws.len(), 4);
        assert!(draws.iter().all(|d| d.selected >= 1));
        assert_eq!(draws.iter().map(|d| d.selected).sum::<usize>(), 6);
        assert_eq!(
            draws.iter().map(|d| d.discovered).sum::<usize>(),
            input.len()
        );
        let keys: Vec<String> = sel.iter().map(|u| stratum_key(u)).collect();
        for want in ["blog", "shop", "/", "en/about"] {
            assert!(keys.iter().any(|k| k == want), "missing {want}: {keys:?}");
        }
    }

    #[test]
    fn stratified_is_deterministic_and_keeps_input_order() {
        let (a, da) = stratified_sample(site(), 10);
        let (b, db) = stratified_sample(site(), 10);
        assert_eq!(a, b);
        assert_eq!(da, db);
        let pos: Vec<usize> = a
            .iter()
            .map(|u| site().iter().position(|x| x == u).unwrap())
            .collect();
        assert!(pos.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn stratified_spreads_within_a_stratum_instead_of_taking_the_head() {
        let (sel, _) = stratified_sample(site(), 8);
        let blog: Vec<&String> = sel.iter().filter(|u| u.contains("/blog/")).collect();
        assert!(blog.len() > 1);
        assert!(blog
            .iter()
            .any(|u| !u.ends_with("post-0") && !u.ends_with("post-1")));
        assert!(blog
            .iter()
            .any(|u| u.contains("post-2") || u.contains("post-3")));
    }

    #[test]
    fn stratified_budget_below_strata_takes_the_largest_and_never_exceeds_budget() {
        let (sel, draws) = stratified_sample(site(), 2);
        assert_eq!(sel.len(), 2);
        assert_eq!(draws.iter().map(|d| d.selected).sum::<usize>(), 2);
        assert_eq!(draws[0].key, "blog");
        assert_eq!(draws[1].key, "shop");
    }

    #[test]
    fn stratified_without_effective_budget_returns_input_unchanged() {
        let input = site();
        assert_eq!(stratified_sample(input.clone(), 0), (input.clone(), vec![]));
        assert_eq!(stratified_sample(input.clone(), 500), (input, vec![]));
    }
}
