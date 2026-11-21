//! Where the markup of a finding comes from.

use serde::{Deserialize, Serialize};

/// The origin of a finding (#1109): whether the markup it sits in belongs to
/// the audited page itself or to a frame the site embeds. A label for the
/// reader, no verdict: scores, grades and verdicts do not read it.
///
/// `Mixed` exists on a finding only, never on a single occurrence: it marks a
/// finding whose occurrences have more than one origin.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FindingOrigin {
    /// The main document of the audited page.
    #[default]
    FirstParty,
    /// A frame whose site (registrable domain) is the page's own, or one
    /// without a document URL of its own (`about:blank`, `srcdoc`).
    SameSiteFrame,
    /// A frame served from another site than the page.
    CrossOriginFrame,
    /// A finding with occurrences of more than one origin.
    Mixed,
}

impl FindingOrigin {
    /// The JSON value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstParty => "first_party",
            Self::SameSiteFrame => "same_site_frame",
            Self::CrossOriginFrame => "cross_origin_frame",
            Self::Mixed => "mixed",
        }
    }

    pub fn is_first_party(&self) -> bool {
        *self == Self::FirstParty
    }

    /// The origin of a finding from the origins of its occurrences: the one
    /// they share, `Mixed` when they differ, `FirstParty` without any.
    pub fn of_occurrences(origins: impl IntoIterator<Item = FindingOrigin>) -> FindingOrigin {
        let mut iter = origins.into_iter();
        let Some(first) = iter.next() else {
            return Self::FirstParty;
        };
        if iter.all(|o| o == first) {
            first
        } else {
            Self::Mixed
        }
    }
}

/// What a frame's source says about its content (#1109).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FrameOrigin {
    pub origin: FindingOrigin,
    /// Canonical English name of the known provider of a cross-origin frame
    /// (`web-checks` provider list); `None` for an unknown host.
    pub provider: Option<String>,
    /// Host of the frame's http(s) source; `None` for a frame without one.
    pub host: Option<String>,
}

impl FrameOrigin {
    /// A frame origin of a given value (tests and the attribution of a frame
    /// whose source is already resolved).
    pub fn of(origin: FindingOrigin, provider: Option<&str>, host: Option<&str>) -> Self {
        Self {
            origin,
            provider: provider.map(str::to_string),
            host: host.map(str::to_string),
        }
    }
}

/// The origin of a frame's content against its page, and the known provider
/// of a cross-origin frame (#1109).
///
/// `sources` are the frame's candidate URLs in order of preference, the
/// `src` attribute of the iframe element first and the frame document's URL
/// after it: a lazy-loaded frame is still `about:blank` when it is audited
/// and only its `src` says where it comes from. The first one that is an
/// http(s) URL (resolved against the page) is the source. A frame without
/// one (`about:blank`, `srcdoc`, `data:`) is the page's own markup.
/// Same site means the same registrable domain (public suffix list), so
/// `www1.example.com` in a page of `www.example.com` is a same-site frame;
/// a host without a registrable domain (an IP address, `localhost`) is only
/// the same site as itself. The provider is the `web-checks` provider list's
/// answer for the source URL, never an own list, and only a cross-origin
/// frame has one: an unknown host stays without.
pub fn frame_origin(page_url: &str, sources: &[Option<&str>]) -> FrameOrigin {
    let own = |origin| FrameOrigin {
        origin,
        provider: None,
        host: None,
    };
    let Ok(page) = url::Url::parse(page_url) else {
        return own(FindingOrigin::SameSiteFrame);
    };
    let source = sources
        .iter()
        .flatten()
        .map(|u| u.trim())
        .filter(|u| !u.is_empty())
        .filter_map(|u| page.join(u).ok())
        .find(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some());
    let (Some(frame), Some(page_host)) = (source, page.host_str()) else {
        return own(FindingOrigin::SameSiteFrame);
    };
    let frame_host = frame.host_str().unwrap_or_default().to_ascii_lowercase();
    if same_site(page_host, &frame_host) {
        return FrameOrigin {
            origin: FindingOrigin::SameSiteFrame,
            provider: None,
            host: Some(frame_host),
        };
    }
    FrameOrigin {
        origin: FindingOrigin::CrossOriginFrame,
        provider: web_checks::third_party::classify_url(frame.as_str())
            .map(|m| m.provider.name.to_string()),
        host: Some(frame_host),
    }
}

fn same_site(a: &str, b: &str) -> bool {
    let (a, b) = (a.to_ascii_lowercase(), b.to_ascii_lowercase());
    if a == b {
        return true;
    }
    match (psl::domain_str(&a), psl::domain_str(&b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin_of(page: &str, frame: &str) -> (FindingOrigin, Option<String>) {
        let o = frame_origin(page, &[Some(frame)]);
        (o.origin, o.provider)
    }

    #[test]
    fn a_frame_of_another_site_is_cross_origin_with_its_provider() {
        let (origin, provider) = origin_of(
            "https://www.example.com/page",
            "https://www.youtube.com/embed/abc",
        );
        assert_eq!(origin, FindingOrigin::CrossOriginFrame);
        assert!(provider.is_some(), "YouTube is in the web-checks list");

        let unknown = frame_origin(
            "https://www.example.com/",
            &[Some("https://widgets.unknown-host.example.net/x")],
        );
        assert_eq!(unknown.origin, FindingOrigin::CrossOriginFrame);
        assert_eq!(unknown.provider, None);
        assert_eq!(
            unknown.host.as_deref(),
            Some("widgets.unknown-host.example.net")
        );
    }

    /// A lazy-loaded frame is `about:blank` when it is audited; its `src`
    /// attribute says where it comes from and wins over the document URL.
    #[test]
    fn a_blank_frame_with_a_src_takes_its_origin_from_the_src() {
        let page = "https://www.example.com/";
        let cross = frame_origin(
            page,
            &[
                Some("https://ir-api.widgets.example.net/embed"),
                Some("about:blank"),
            ],
        );
        assert_eq!(cross.origin, FindingOrigin::CrossOriginFrame);
        assert_eq!(cross.host.as_deref(), Some("ir-api.widgets.example.net"));

        let same = frame_origin(
            page,
            &[
                Some("https://www1.example.com/newsroom"),
                Some("about:blank"),
            ],
        );
        assert_eq!(same.origin, FindingOrigin::SameSiteFrame);
        assert_eq!(same.host.as_deref(), Some("www1.example.com"));

        // Without a usable src the frame document's URL is the fallback.
        let fallback = frame_origin(page, &[None, Some("https://www.youtube.com/embed/x")]);
        assert_eq!(fallback.origin, FindingOrigin::CrossOriginFrame);
        assert!(fallback.provider.is_some());
        let non_http = frame_origin(
            page,
            &[Some("about:blank"), Some("https://www.youtube.com/embed/x")],
        );
        assert_eq!(non_http.origin, FindingOrigin::CrossOriginFrame);

        // Nothing but about:blank: the page's own markup.
        let blank = frame_origin(page, &[None, Some("about:blank")]);
        assert_eq!(blank.origin, FindingOrigin::SameSiteFrame);
        assert_eq!(blank.host, None);
    }

    #[test]
    fn a_frame_of_the_same_registrable_domain_is_same_site() {
        for frame in [
            "https://www1.example.com/newsroom",
            "https://example.com/",
            "/local/path",
            "about:blank",
            "",
        ] {
            assert_eq!(
                origin_of("https://www.example.com/", frame),
                (FindingOrigin::SameSiteFrame, None),
                "{frame}"
            );
        }
        assert_eq!(
            frame_origin("https://www.example.com/", &[None]).origin,
            FindingOrigin::SameSiteFrame
        );
    }

    #[test]
    fn the_public_suffix_list_separates_sites_below_a_shared_suffix() {
        assert_eq!(
            origin_of("https://a.example.co.uk/", "https://b.example.co.uk/").0,
            FindingOrigin::SameSiteFrame
        );
        assert_eq!(
            origin_of("https://a.example.co.uk/", "https://other.co.uk/").0,
            FindingOrigin::CrossOriginFrame
        );
        assert_eq!(
            origin_of("https://a.github.io/", "https://b.github.io/").0,
            FindingOrigin::CrossOriginFrame
        );
    }

    #[test]
    fn hosts_without_a_registrable_domain_are_only_their_own_site() {
        assert_eq!(
            origin_of("http://localhost:3000/", "http://localhost:4000/").0,
            FindingOrigin::SameSiteFrame
        );
        assert_eq!(
            origin_of("http://localhost:3000/", "http://127.0.0.1:4000/").0,
            FindingOrigin::CrossOriginFrame
        );
    }

    #[test]
    fn a_finding_takes_the_shared_origin_or_is_mixed() {
        use FindingOrigin::*;
        assert_eq!(FindingOrigin::of_occurrences([]), FirstParty);
        assert_eq!(
            FindingOrigin::of_occurrences([CrossOriginFrame, CrossOriginFrame]),
            CrossOriginFrame
        );
        assert_eq!(
            FindingOrigin::of_occurrences([FirstParty, CrossOriginFrame]),
            Mixed
        );
    }

    #[test]
    fn json_values_are_the_snake_case_names() {
        for origin in [
            FindingOrigin::FirstParty,
            FindingOrigin::SameSiteFrame,
            FindingOrigin::CrossOriginFrame,
            FindingOrigin::Mixed,
        ] {
            assert_eq!(
                serde_json::to_value(origin).unwrap(),
                serde_json::json!(origin.as_str())
            );
        }
    }
}
