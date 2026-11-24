//! WCAG 1.2.2 caption tracks - Media Rules
//!
//! What stays in the host after #696 (B6) and #1182: whether a `<track>`
//! caption file actually loads (a network probe) and third-party video
//! embeds. A video without a caption track is the shared rule
//! `media/video-captions-missing`. Which frames are cross-origin runs as
//! `frames/cross-origin` in the shared rules (`accessibility::probes`, #1002).
//! Frame names, `<area>`,
//! `<input type="image">`, `<object>`, server-side image maps and autoplaying
//! audio run as shared rules (`wcag::shared`). Not ported and deleted here:
//! the AX-tree checks for an unnamed `role="application"` (no evidence),
//! an unnamed `role="img"` (covered by `svg/name-missing` and
//! `images/alt-missing`) and a named decorative element (covered by
//! `aria/attribute-prohibited`), see the `a11y-rules` 0.18.0 changelog.

use chromiumoxide::Page;
use reqwest::Client;
use std::time::Duration;
use tracing::warn;
use url::Url;

use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};
use crate::wcag::WcagLevel;

/// Rule metadata for captions (1.2.2)
pub(super) const RULE_META_CAPTIONS: RuleMetadata = RuleMetadata {
    id: "1.2.2",
    name: "Captions (Prerecorded)",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Prerecorded audio content in synchronized media has captions",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/captions-prerecorded.html",
    axe_id: "video-caption",
    tags: &["wcag2a", "wcag122", "cat.media"],
};

/// Hosts of known third-party video-embed players. Captions on these are
/// controlled by the platform's own player and are invisible to a DOM/network
/// probe from this tool — pages embedding one of these get a distinct
/// manual-review message rather than being silently ignored.
const VIDEO_EMBED_HOSTS: &[&str] = &[
    "youtube.com",
    "youtube-nocookie.com",
    "vimeo.com",
    "player.vimeo.com",
    "dailymotion.com",
    "wistia.com",
    "wistia.net",
];

fn is_video_embed_host(src: &str) -> bool {
    let Ok(url) = Url::parse(src) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    VIDEO_EMBED_HOSTS
        .iter()
        .any(|h| host == *h || host.ends_with(&format!(".{h}")))
}

/// Whether a `<track>` with this `kind` attribute (`None` when it is missing)
/// is a caption or subtitle track, the same reading as the shared
/// `media/video-captions-missing` (a11y-rules 0.31.1) so the two rules split
/// the videos exactly. HTML: the missing value default is `subtitles`; an
/// invalid value (also `""` or `" captions"`) is `metadata`. Keywords match
/// ASCII case-insensitively, without trimming.
fn track_kind_is_caption_like(kind: Option<&str>) -> bool {
    kind.is_none_or(|k| k.eq_ignore_ascii_case("captions") || k.eq_ignore_ascii_case("subtitles"))
}

/// A `<track>` file counts as a verified caption/subtitle source only when it
/// resolves (checked by the caller) AND looks like a real caption format —
/// by file extension or declared content-type. A bare 2xx alone isn't
/// enough: a misconfigured server can return 200 for any path (e.g. an SPA
/// catch-all), which would otherwise be misread as "captions exist".
fn looks_like_caption_file(content_type: Option<&str>, url: &str) -> bool {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    if path.ends_with(".vtt") || path.ends_with(".srt") {
        return true;
    }
    content_type
        .map(|ct| ct.to_ascii_lowercase().contains("vtt"))
        .unwrap_or(false)
}

/// What the probe learned about one caption or subtitle track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackCheck {
    /// Its file loads from the page origin as a caption file.
    Resolved,
    /// It has no `src`, or the page origin answered for its file with an
    /// error status or something that is no caption file.
    Failed,
    /// Not decided: the file is on another origin, the probe budget is spent,
    /// no HTTP client could be built, or the request did not get an answer.
    Unverified,
}

/// How a video with a caption or subtitle track fares as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VideoCaptions {
    /// One of its caption tracks resolves.
    Captioned,
    /// Every caption track failed on the page origin: no working captions.
    Broken,
    /// Neither: at least one track is undecided.
    Unverified,
}

/// A video counts as broken only when every one of its caption tracks
/// failed; one undecided track (another origin, beyond the budget) could be
/// the working one.
fn video_captions(checks: &[TrackCheck]) -> VideoCaptions {
    if checks.contains(&TrackCheck::Resolved) {
        VideoCaptions::Captioned
    } else if !checks.is_empty() && checks.iter().all(|c| *c == TrackCheck::Failed) {
        VideoCaptions::Broken
    } else {
        VideoCaptions::Unverified
    }
}

/// Same-origin HTTP probe (mirrors the pattern in
/// `security::sourcemap::audit_source_maps`): only fetches when the track
/// URL's origin (scheme + host + port) matches the audited page's origin, so
/// this never becomes an arbitrary-URL fetch driven by page content. A server
/// that refuses `HEAD` (405, 501) gets a `GET`, so a refused method is never
/// read as a missing file.
async fn probe_track(
    client: &Client,
    base_origin: Option<&url::Origin>,
    track_src: &str,
) -> TrackCheck {
    let Ok(track_url) = Url::parse(track_src) else {
        return TrackCheck::Unverified;
    };
    if base_origin != Some(&track_url.origin()) {
        return TrackCheck::Unverified;
    }
    let resp = match client.head(track_url.as_str()).send().await {
        Ok(r)
            if r.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED
                || r.status() == reqwest::StatusCode::NOT_IMPLEMENTED =>
        {
            client.get(track_url.as_str()).send().await
        }
        other => other,
    };
    let Ok(resp) = resp else {
        return TrackCheck::Unverified;
    };
    if !resp.status().is_success() {
        return TrackCheck::Failed;
    }
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());
    if looks_like_caption_file(content_type, track_url.as_str()) {
        TrackCheck::Resolved
    } else {
        TrackCheck::Failed
    }
}

/// Caps how many `<track>` files get an HTTP HEAD probe, bounding worst-case
/// latency on a page with many videos/tracks.
const MAX_TRACKS_CHECKED: usize = 10;

const VIDEO_TRACK_SCAN_JS: &str = include_str!("js/video_track_scan.js");

/// 1.2.2 Captions (Prerecorded) — what needs the browser (#1182).
///
/// A rendered `<video>` without a `<track kind="captions"|"subtitles">` child
/// is the shared rule `media/video-captions-missing` (one review per video).
/// This rule judges the rest, which only a running browser can see:
///
/// - videos that do carry such a track: does one of its files load
///   same-origin as a real caption file? Emits a confirmed pass
///   (`Outcome::Pass`) only when every one of them does and the page embeds
///   no third-party player;
/// - third-party video embeds (YouTube, Vimeo, …) whose captions this tool
///   cannot see.
///
/// A video whose caption tracks all failed on the page origin (no `src`, an
/// error status, no caption file) has no working captions, the same as a
/// missing track: one page-level `Review` naming those videos. Anything else
/// short of the pass (a track on another origin or beyond the probe budget,
/// an embed) is one page-level `NotTestable` manual-review notice. Its
/// applicability matches the shared rule: a muted video without `controls`
/// and a video under `aria-hidden="true"` are left out, so the two rules
/// split the videos by track presence and no video is reported twice.
pub async fn check_video_caption_tracks_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        VIDEO_TRACK_SCAN_JS,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("video-caption-track DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                RULE_META_CAPTIONS.axe_id,
                RULE_META_CAPTIONS.level,
                "page_evaluation_failed",
            )];
        }
    };
    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            RULE_META_CAPTIONS.axe_id,
            RULE_META_CAPTIONS.level,
            "missing_evaluation_value",
        )];
    };

    // Videos without a caption-like track belong to the shared rule.
    let videos: Vec<(String, Vec<serde_json::Value>)> = value
        .get("videos")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .map(|video| {
            let selector = video
                .get("selector")
                .and_then(|s| s.as_str())
                .unwrap_or("video")
                .to_string();
            let tracks = video
                .get("tracks")
                .and_then(|t| t.as_array())
                .into_iter()
                .flatten()
                .filter(|track| {
                    track_kind_is_caption_like(track.get("kind").and_then(|k| k.as_str()))
                })
                .cloned()
                .collect::<Vec<_>>();
            (selector, tracks)
        })
        .filter(|(_, tracks)| !tracks.is_empty())
        .collect();
    let embed_count = value
        .get("embeds")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.get("src").and_then(|s| s.as_str()))
                .filter(|src| is_video_embed_host(src))
                .count()
        })
        .unwrap_or(0);

    if videos.is_empty() && embed_count == 0 {
        return Vec::new();
    }

    let base_origin = page
        .url()
        .await
        .ok()
        .flatten()
        .and_then(|u| Url::parse(&u).ok())
        .map(|u| u.origin());

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("auditmysite-probe/1.0")
        .build()
        .ok();

    let mut checked = 0usize;
    let mut states = Vec::with_capacity(videos.len());
    for (selector, tracks) in &videos {
        let mut checks = Vec::with_capacity(tracks.len());
        for track in tracks {
            let src = track.get("src").and_then(|s| s.as_str()).unwrap_or("");
            if src.is_empty() {
                checks.push(TrackCheck::Failed);
                continue;
            }
            if checked >= MAX_TRACKS_CHECKED {
                checks.push(TrackCheck::Unverified);
                continue;
            }
            checked += 1;
            let Some(client) = client.as_ref() else {
                checks.push(TrackCheck::Unverified);
                continue;
            };
            checks.push(probe_track(client, base_origin.as_ref(), src).await);
        }
        states.push((selector.as_str(), video_captions(&checks)));
    }

    caption_findings(&states, embed_count)
}

/// The findings for the judged videos (selector and state) and the number of
/// third-party embeds: a pass, a review for the broken videos, an untested
/// note for the undecided ones and the embeds — or nothing.
fn caption_findings(states: &[(&str, VideoCaptions)], embed_count: usize) -> Vec<Violation> {
    let of = |wanted: VideoCaptions| -> Vec<&str> {
        states
            .iter()
            .filter(|(_, s)| *s == wanted)
            .map(|(sel, _)| *sel)
            .collect()
    };
    let broken = of(VideoCaptions::Broken);
    let unverified = of(VideoCaptions::Unverified);
    let track_count = states.len();
    let plural = |n: usize| if n == 1 { "" } else { "s" };

    if track_count > 0 && broken.is_empty() && unverified.is_empty() && embed_count == 0 {
        return vec![Violation::new(
            RULE_META_CAPTIONS.id,
            RULE_META_CAPTIONS.name,
            RULE_META_CAPTIONS.level,
            Severity::Low,
            format!(
                "{track_count} video element{} with a <track kind=\"captions\"> or \
                 <track kind=\"subtitles\">, each with a resolving caption file. Caption \
                 presence and format are technically confirmed — a manual check of \
                 synchronization and transcription accuracy is still recommended.",
                plural(track_count)
            ),
            "page",
        )
        .with_fix(
            "Verify caption timing and transcription accuracy against the video's audio track.",
        )
        .with_help_url(RULE_META_CAPTIONS.help_url)
        .with_rule_id(RULE_META_CAPTIONS.axe_id)
        .as_positive()];
    }

    let mut findings = Vec::new();
    if !broken.is_empty() {
        findings.push(
            Violation::new(
                RULE_META_CAPTIONS.id,
                RULE_META_CAPTIONS.name,
                RULE_META_CAPTIONS.level,
                Severity::Medium,
                format!(
                    "{} video element{} ({}) {} a <track kind=\"captions\"> or \
                     <track kind=\"subtitles\">, but no track file loads from the page origin \
                     as a caption file (error status, no caption format, or no src). Without a \
                     working file the video has no captions — check the track URLs and files.",
                    broken.len(),
                    plural(broken.len()),
                    broken.join(", "),
                    if broken.len() == 1 { "has" } else { "have" },
                ),
                "page",
            )
            .with_fix(
                "Point each <track kind=\"captions\"> at a WebVTT (.vtt) or SRT file that the \
                 server delivers, or caption the video another way.",
            )
            .with_help_url(RULE_META_CAPTIONS.help_url)
            .with_rule_id(RULE_META_CAPTIONS.axe_id)
            .with_kind(Outcome::Review),
        );
    }

    let mut parts = Vec::new();
    if !unverified.is_empty() {
        parts.push(format!(
            "{} video element{} with a <track kind=\"captions\"> or <track kind=\"subtitles\"> \
             could not be confirmed (track file on another origin or beyond the check budget) \
             — review {} for working, synchronized captions.",
            unverified.len(),
            plural(unverified.len()),
            if unverified.len() == 1 {
                "it"
            } else {
                "each of them"
            },
        ));
    }
    if embed_count > 0 {
        parts.push(format!(
            "{embed_count} embedded video player{} detected (e.g. a YouTube/Vimeo-style \
             iframe). Caption availability depends on the platform's own player and cannot be \
             verified automatically — check the embed's caption/subtitle settings manually.",
            plural(embed_count)
        ));
    }
    if !parts.is_empty() {
        findings.push(
            Violation::new(
                RULE_META_CAPTIONS.id,
                RULE_META_CAPTIONS.name,
                RULE_META_CAPTIONS.level,
                Severity::High,
                parts.join(" "),
                "page",
            )
            .with_fix(
                "Ensure all prerecorded video with audio has synchronized captions. \
                 Use the <track kind=\"captions\"> element or a captioning service.",
            )
            .with_help_url(RULE_META_CAPTIONS.help_url)
            .with_rule_id(RULE_META_CAPTIONS.axe_id)
            .with_kind(Outcome::Untested),
        );
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_video_embed_host_matches_known_platforms() {
        assert!(is_video_embed_host("https://www.youtube.com/embed/abc123"));
        assert!(is_video_embed_host(
            "https://www.youtube-nocookie.com/embed/abc123"
        ));
        assert!(is_video_embed_host("https://player.vimeo.com/video/42"));
        assert!(!is_video_embed_host("https://example.com/video.html"));
        assert!(!is_video_embed_host("not a url"));
    }

    #[test]
    fn test_looks_like_caption_file_by_extension() {
        assert!(looks_like_caption_file(
            None,
            "https://example.com/captions.vtt"
        ));
        assert!(looks_like_caption_file(
            None,
            "https://example.com/captions.srt?v=2"
        ));
        assert!(!looks_like_caption_file(
            None,
            "https://example.com/index.html"
        ));
    }

    #[test]
    fn test_looks_like_caption_file_by_content_type() {
        assert!(looks_like_caption_file(
            Some("text/vtt; charset=utf-8"),
            "https://example.com/dynamic-captions"
        ));
        assert!(!looks_like_caption_file(
            Some("text/html"),
            "https://example.com/dynamic-captions"
        ));
    }

    #[test]
    fn a_video_is_broken_only_when_every_caption_track_failed() {
        use TrackCheck::*;
        assert_eq!(video_captions(&[Failed]), VideoCaptions::Broken);
        assert_eq!(video_captions(&[Failed, Failed]), VideoCaptions::Broken);
        assert_eq!(
            video_captions(&[Failed, Resolved]),
            VideoCaptions::Captioned
        );
        // An undecided track could be the working one.
        assert_eq!(
            video_captions(&[Failed, Unverified]),
            VideoCaptions::Unverified
        );
        assert_eq!(video_captions(&[Unverified]), VideoCaptions::Unverified);
        assert_eq!(video_captions(&[]), VideoCaptions::Unverified);
    }

    #[test]
    fn a_broken_track_is_a_review_and_undecided_ones_stay_untested() {
        let findings = caption_findings(
            &[
                ("video#ok", VideoCaptions::Captioned),
                ("video#broken", VideoCaptions::Broken),
                ("video#remote", VideoCaptions::Unverified),
            ],
            1,
        );
        assert_eq!(findings.len(), 2, "{findings:?}");
        assert_eq!(findings[0].kind, Outcome::Review);
        assert!(findings[0].message.contains("video#broken"));
        assert!(!findings[0].message.contains("video#remote"));
        assert_eq!(findings[1].kind, Outcome::Untested);
        assert!(findings[1].message.contains("could not be confirmed"));
        assert!(findings[1].message.contains("embedded video player"));
    }

    #[test]
    fn only_broken_tracks_give_a_review_alone() {
        let findings = caption_findings(&[("video#broken", VideoCaptions::Broken)], 0);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].kind, Outcome::Review);
    }

    #[test]
    fn an_embed_alone_stays_untested() {
        let findings = caption_findings(&[], 1);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].kind, Outcome::Untested);
        assert!(findings[0].message.contains("embedded video player"));
    }

    #[test]
    fn all_resolving_tracks_without_embed_pass() {
        let findings = caption_findings(&[("video#ok", VideoCaptions::Captioned)], 0);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, Outcome::Pass);
        assert!(caption_findings(&[], 0).is_empty());
    }

    #[test]
    fn test_track_kind_is_caption_like() {
        assert!(track_kind_is_caption_like(Some("captions")));
        assert!(track_kind_is_caption_like(Some("Subtitles")));
        assert!(track_kind_is_caption_like(Some("CAPTIONS")));
        // Missing value default: subtitles.
        assert!(track_kind_is_caption_like(None));
        // Invalid values are metadata: no trimming, the empty value too.
        assert!(!track_kind_is_caption_like(Some(" captions")));
        assert!(!track_kind_is_caption_like(Some("")));
        assert!(!track_kind_is_caption_like(Some("chapters")));
        assert!(!track_kind_is_caption_like(Some("descriptions")));
    }
}
