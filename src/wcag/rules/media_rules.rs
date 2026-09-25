//! WCAG 1.2.1, 1.2.2, 1.2.5, 1.1.1 - Media Rules
//!
//! Checks that media elements, SVGs, and canvas elements have accessible names
//! and that decorative elements are not spuriously named.

use chromiumoxide::Page;
use reqwest::Client;
use std::time::Duration;
use tracing::warn;
use url::Url;

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for media accessibility (1.2.x)
pub const RULE_META_MEDIA: RuleMetadata = RuleMetadata {
    id: "1.2.1",
    name: "Audio-only and Video-only",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Video and audio elements must have accessible alternatives",
    help_url:
        "https://www.w3.org/WAI/WCAG22/Understanding/audio-only-and-video-only-prerecorded.html",
    axe_id: "video-caption",
    tags: &["wcag2a", "wcag121", "cat.media"],
};

/// Rule metadata for captions (1.2.2)
pub const RULE_META_CAPTIONS: RuleMetadata = RuleMetadata {
    id: "1.2.2",
    name: "Captions (Prerecorded)",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Prerecorded audio content in synchronized media has captions",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/captions-prerecorded.html",
    axe_id: "video-caption",
    tags: &["wcag2a", "wcag122", "cat.media"],
};

/// Rule metadata for SVG/image accessibility (1.1.1)
pub const RULE_META_IMAGE: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Non-text Content",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "All non-text content must have a text alternative",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html",
    axe_id: "image-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

/// Rule metadata for iframe accessible names (axe-core `frame-title`).
pub const RULE_META_FRAME_TITLE: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Frame title",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Frames and iframes must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG22/Techniques/html/H64",
    axe_id: "frame-title",
    tags: &["wcag2a", "wcag241", "cat.text-alternatives"],
};

/// Run all media-related WCAG checks
///
/// 1.2.2 (captions) is **not** handled here — the always-"manual review"
/// notice this function used to emit for any `role="Video"` AXTree node was
/// replaced by [`check_video_caption_tracks_with_page`], which inspects the
/// live DOM (`<track kind="captions"|"subtitles">`) and, for a same-origin
/// track file, actually probes whether it resolves before deciding between
/// a confirmed pass and a manual-review notice (#video-caption-checks).
pub fn check_media_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        match role {
            "application" => {
                check_application_has_name(node, &mut results);
            }
            "img" => {
                // SVG images and other img-role elements
                check_img_role_has_name(node, &mut results);
            }
            "presentation" | "none" => {
                check_decorative_has_no_spurious_name(node, &mut results);
            }
            _ => {}
        }
    }

    results
}

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

fn track_kind_is_caption_like(kind: &str) -> bool {
    matches!(kind.to_ascii_lowercase().as_str(), "captions" | "subtitles")
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

/// Same-origin HTTP HEAD probe (mirrors the pattern in
/// `security::sourcemap::audit_source_maps`): only fetches when the track
/// URL's origin (scheme + host + port) matches the audited page's origin, so
/// this never becomes an arbitrary-URL fetch driven by page content.
async fn track_resolves_same_origin(
    client: &Client,
    base_origin: Option<&url::Origin>,
    track_src: &str,
) -> bool {
    let Ok(track_url) = Url::parse(track_src) else {
        return false;
    };
    if base_origin != Some(&track_url.origin()) {
        return false;
    }
    let Ok(resp) = client.head(track_url.as_str()).send().await else {
        return false;
    };
    if !resp.status().is_success() {
        return false;
    }
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());
    looks_like_caption_file(content_type, track_url.as_str())
}

/// Caps how many `<track>` files get an HTTP HEAD probe, bounding worst-case
/// latency on a page with many videos/tracks.
const MAX_TRACKS_CHECKED: usize = 10;

const VIDEO_TRACK_SCAN_JS: &str = r#"
var videos = [];
Array.prototype.forEach.call(document.querySelectorAll('video'), function(v) {
  var tracks = [];
  Array.prototype.forEach.call(v.querySelectorAll('track'), function(t) {
    tracks.push({
      kind: (t.getAttribute('kind') || '').toLowerCase(),
      src: t.src || '',
      srclang: t.getAttribute('srclang') || '',
      label: t.getAttribute('label') || ''
    });
  });
  videos.push({ selector: __amsCssSelector(v), tracks: tracks });
});
var embeds = [];
Array.prototype.forEach.call(document.querySelectorAll('iframe[src]'), function(f) {
  embeds.push({ selector: __amsCssSelector(f), src: f.src || '' });
});
return { videos: videos, embeds: embeds };
"#;

/// 1.2.2 Captions (Prerecorded) — DOM + network deepening (#video-caption-checks).
///
/// Emits a confirmed pass (`Outcome::Pass`) only when every native
/// `<video>` element on the page has a `<track kind="captions"|"subtitles">`
/// whose `src` actually resolves same-origin as a real caption file.
/// Anything short of that (no track, an unresolving track, or only
/// third-party video embeds whose captions this tool cannot see) stays a
/// `NotTestable` manual-review notice, same as before — this only narrows
/// when a Pass is claimed, it never invents a stronger negative verdict.
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

    let videos = value
        .get("videos")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
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

    let native_count = videos.len();
    let mut captioned_count = 0usize;
    let mut any_track_found = false;
    let mut any_track_unresolved = false;
    let mut checked = 0usize;

    for video in &videos {
        let tracks = video
            .get("tracks")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();
        let mut this_video_captioned = false;
        for track in &tracks {
            let kind = track.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            if !track_kind_is_caption_like(kind) {
                continue;
            }
            let src = track.get("src").and_then(|s| s.as_str()).unwrap_or("");
            if src.is_empty() {
                continue;
            }
            any_track_found = true;
            if checked >= MAX_TRACKS_CHECKED {
                continue;
            }
            checked += 1;
            let Some(client) = client.as_ref() else {
                continue;
            };
            if track_resolves_same_origin(client, base_origin.as_ref(), src).await {
                this_video_captioned = true;
            } else {
                any_track_unresolved = true;
            }
        }
        if this_video_captioned {
            captioned_count += 1;
        }
    }

    if native_count > 0 && captioned_count == native_count {
        return vec![Violation::new(
            RULE_META_CAPTIONS.id,
            RULE_META_CAPTIONS.name,
            RULE_META_CAPTIONS.level,
            Severity::Low,
            format!(
                "{native_count} video element{} with a resolving <track kind=\"captions\"> or \
                 <track kind=\"subtitles\"> file detected. Caption presence and format are \
                 technically confirmed — a manual check of synchronization and transcription \
                 accuracy is still recommended.",
                if native_count == 1 { "" } else { "s" }
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

    let message =
        if native_count > 0 && any_track_found {
            format!(
                "{captioned_count} of {native_count} video elements have a verified, resolving \
             caption/subtitle track. The rest could not be automatically confirmed{} — review \
             each remaining video for correct, synchronized captions.",
                if any_track_unresolved {
                    " (a <track> src was found but did not resolve)"
                } else {
                    ""
                }
            )
        } else if native_count > 0 {
            format!(
            "{native_count} video {} detected without a resolving <track kind=\"captions\"> or \
             <track kind=\"subtitles\"> element. Caption presence and accuracy cannot be \
             verified automatically — review each video for correct, synchronized captions.",
            if native_count == 1 { "element" } else { "elements" }
        )
        } else {
            format!(
                "{embed_count} embedded video player{} detected (e.g. a YouTube/Vimeo-style \
             iframe). Caption availability depends on the platform's own player and cannot be \
             verified automatically — check the embed's caption/subtitle settings manually.",
                if embed_count == 1 { "" } else { "s" }
            )
        };

    vec![Violation::new(
        RULE_META_CAPTIONS.id,
        RULE_META_CAPTIONS.name,
        RULE_META_CAPTIONS.level,
        Severity::High,
        message,
        "page",
    )
    .with_fix(
        "Ensure all prerecorded video with audio has synchronized captions. \
         Use the <track kind=\"captions\"> element or a captioning service.",
    )
    .with_help_url(RULE_META_CAPTIONS.help_url)
    .with_rule_id(RULE_META_CAPTIONS.axe_id)
    .with_kind(Outcome::Untested)]
}

/// DOM check for iframe accessible names. Iframes are not always represented
/// with enough detail in the AX tree, so inspect the live DOM.
pub async fn check_frame_title_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        crate::accessibility::js_helpers::IS_VISUALLY_HIDDEN_JS,
        r#"
        var issues = [];
        var frames = document.querySelectorAll('iframe, frame');
        for (var i = 0; i < frames.length; i++) {
          var el = frames[i];
          var role = (el.getAttribute('role') || '').toLowerCase();
          if (role === 'none' || role === 'presentation') continue;

          // Skip non-perceivable / hidden elements
          if (el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true') continue;
          if (typeof __amsIsVisuallyHidden === 'function' && __amsIsVisuallyHidden(el)) continue;
          
          var parent = el.parentElement;
          var isAriaHiddenAncestor = false;
          while (parent) {
            if (parent.getAttribute('aria-hidden') === 'true') {
              isAriaHiddenAncestor = true;
              break;
            }
            parent = parent.parentElement;
          }
          if (isAriaHiddenAncestor) continue;

          var style = window.getComputedStyle(el);
          if (style && (style.display === 'none' || style.visibility === 'hidden' || style.visibility === 'collapse')) continue;

          parent = el.parentElement;
          var isHiddenAncestor = false;
          while (parent) {
            var parentStyle = window.getComputedStyle(parent);
            if (parentStyle && (parentStyle.display === 'none' || parentStyle.visibility === 'hidden')) {
              isHiddenAncestor = true;
              break;
            }
            parent = parent.parentElement;
          }
          if (isHiddenAncestor) continue;

          var rect = el.getBoundingClientRect();
          if (rect.width <= 1 || rect.height <= 1) continue;

          var wAttr = el.getAttribute('width');
          var hAttr = el.getAttribute('height');
          if (wAttr !== null && hAttr !== null) {
            var wVal = parseInt(wAttr, 10);
            var hVal = parseInt(hAttr, 10);
            if ((wVal === 0 || wVal === 1) && (hVal === 0 || hVal === 1)) continue;
          }

          var title = (el.getAttribute('title') || '').trim();
          var label = (el.getAttribute('aria-label') || '').trim();
          var labelledBy = (el.getAttribute('aria-labelledby') || '').trim();
          var labelledByText = '';
          if (labelledBy) {
            labelledByText = labelledBy.split(/\s+/).map(function(id) {
              var ref = document.getElementById(id);
              return ref ? ref.textContent.trim() : '';
            }).join(' ').trim();
          }
          if (title || label || labelledByText) continue;
          issues.push({
            selector: __amsCssSelector(el),
            snippet: el.outerHTML.substring(0, 200)
          });
        }
        return issues;
        "#,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("frame-title DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "frame-title",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "frame-title",
            crate::cli::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let mut violation = Violation::new(
                RULE_META_FRAME_TITLE.id,
                RULE_META_FRAME_TITLE.name,
                RULE_META_FRAME_TITLE.level,
                RULE_META_FRAME_TITLE.severity,
                "Iframe is missing an accessible name",
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(RULE_META_FRAME_TITLE.axe_id)
            .with_tags(
                RULE_META_FRAME_TITLE
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix("Add a non-empty title, aria-label, or aria-labelledby to the iframe.")
            .with_help_url(RULE_META_FRAME_TITLE.help_url);

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}

/// Rule metadata for frame-tested (axe-core `frame-tested`).
pub const RULE_META_FRAME_TESTED: RuleMetadata = RuleMetadata {
    id: "frame-tested",
    name: "Frame tested",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Identifies cross-origin iframes that cannot be analyzed automatically",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "frame-tested",
    tags: &["wcag2a", "wcag412", "cat.text-alternatives"],
};

/// DOM check for iframes that cannot be analyzed via CDP (cross-origin).
///
/// Chrome's `getFullAXTree` returns the accessibility tree of same-origin
/// iframes as part of the main page tree, so existing WCAG rules already
/// cover those. Cross-origin iframes are inaccessible — one `NotTestable`
/// finding is emitted per cross-origin frame so auditors know manual review
/// is required (mirrors axe-core's `frame-tested` rule).
pub async fn check_frame_tested_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        var results = [];
        var frames = document.querySelectorAll('iframe, frame');
        for (var i = 0; i < frames.length; i++) {
          var el = frames[i];
          var role = (el.getAttribute('role') || '').toLowerCase();
          if (role === 'none' || role === 'presentation') continue;
          if (el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true') continue;
          var style = window.getComputedStyle(el);
          if (style && (style.display === 'none' || style.visibility === 'hidden' || style.visibility === 'collapse')) continue;
          var rect = el.getBoundingClientRect();
          if (rect.width <= 1 || rect.height <= 1) continue;

          var crossOrigin = false;
          try {
            crossOrigin = (el.contentDocument === null);
          } catch(e) {
            crossOrigin = true;
          }
          if (!crossOrigin) continue;

          results.push({
            selector: __amsCssSelector(el),
            snippet: el.outerHTML.substring(0, 200),
            src: el.src || ''
          });
        }
        return results;
        "#,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("frame-tested DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "frame-tested",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "frame-tested",
            crate::cli::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(frames) = value.as_array() else {
        return vec![];
    };

    frames
        .iter()
        .filter_map(|frame| {
            let selector = frame.get("selector")?.as_str()?.to_string();
            let mut violation = Violation::new(
                RULE_META_FRAME_TESTED.id,
                RULE_META_FRAME_TESTED.name,
                RULE_META_FRAME_TESTED.level,
                RULE_META_FRAME_TESTED.severity,
                "Cross-origin iframe cannot be analyzed automatically — manual review required",
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(RULE_META_FRAME_TESTED.axe_id)
            .with_tags(
                RULE_META_FRAME_TESTED
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix(
                "Verify that the embedded content meets WCAG requirements independently. \
                 Ensure the iframe source provides an accessible experience for screen reader users.",
            )
            .with_help_url(RULE_META_FRAME_TESTED.help_url)
            .with_kind(Outcome::Untested);

            if let Some(snippet) = frame.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}

/// Elements with role="application" (often video/canvas wrappers) need an accessible name
fn check_application_has_name(node: &AXNode, results: &mut WcagResults) {
    if !node.has_name() {
        let violation = Violation::new(
            RULE_META_MEDIA.id,
            RULE_META_MEDIA.name,
            RULE_META_MEDIA.level,
            Severity::Medium,
            "Video element may lack accessible name or caption alternative",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix(
            "Add aria-label or aria-labelledby to the application/video element, and provide a transcript or captions",
        )
        .with_help_url(RULE_META_MEDIA.help_url)
        .with_rule_id(RULE_META_MEDIA.axe_id);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Elements with role="img" (SVG, canvas mapped to img) must have an accessible name
fn check_img_role_has_name(node: &AXNode, results: &mut WcagResults) {
    if !node.has_name() {
        let violation = Violation::new(
            RULE_META_IMAGE.id,
            RULE_META_IMAGE.name,
            RULE_META_IMAGE.level,
            Severity::High,
            "SVG image is missing an accessible name",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix(
            "Add a <title> element inside the SVG, or use aria-label/aria-labelledby on the SVG element",
        )
        .with_help_url(RULE_META_IMAGE.help_url)
        .with_rule_id(RULE_META_IMAGE.axe_id);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Decorative elements (presentation/none) should not have an accessible name
/// as this causes confusion for assistive technology users
fn check_decorative_has_no_spurious_name(node: &AXNode, results: &mut WcagResults) {
    if node.has_name() {
        let violation = Violation::new(
            RULE_META_IMAGE.id,
            RULE_META_IMAGE.name,
            RULE_META_IMAGE.level,
            Severity::Low,
            "Decorative element has an accessible name (may be unnecessary)",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix(
            "Remove the accessible name (alt, aria-label) from decorative elements, or change the role to convey meaningful content",
        )
        .with_help_url(RULE_META_IMAGE.help_url)
        .with_rule_id(RULE_META_IMAGE.axe_id);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn make_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_svg_img_without_name_flagged() {
        let nodes = vec![make_node("1", "img", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_media_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("SVG image is missing")));
    }

    #[test]
    fn test_svg_img_with_name_passes() {
        let nodes = vec![make_node("1", "img", Some("Company logo"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_media_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("SVG image is missing")));
    }

    #[test]
    fn test_decorative_with_name_flagged() {
        let nodes = vec![make_node("1", "presentation", Some("decorative star"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_media_rules(&tree);
        assert!(results.violations.iter().any(|v| v
            .message
            .contains("Decorative element has an accessible name")));
    }

    #[test]
    fn test_decorative_without_name_passes() {
        let nodes = vec![make_node("1", "presentation", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_media_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("Decorative element")));
    }

    #[test]
    fn test_application_without_name_flagged() {
        let nodes = vec![make_node("1", "application", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_media_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("Video element may lack")));
    }

    #[test]
    fn test_role_video_no_longer_triggers_check_media_rules() {
        // 1.2.2 caption detection moved to check_video_caption_tracks_with_page
        // (DOM + network, #video-caption-checks) — check_media_rules (AXTree-only)
        // no longer treats a role="Video" node specially at all.
        let nodes = vec![make_node("1", "Video", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_media_rules(&tree);
        assert!(results.not_testables.is_empty());
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("Video element may lack")));
    }

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
    fn test_track_kind_is_caption_like() {
        assert!(track_kind_is_caption_like("captions"));
        assert!(track_kind_is_caption_like("Subtitles"));
        assert!(!track_kind_is_caption_like("chapters"));
        assert!(!track_kind_is_caption_like("descriptions"));
    }

    #[test]
    fn test_frame_title_metadata_uses_wcag_241_with_axe_id() {
        assert_eq!(RULE_META_FRAME_TITLE.id, "2.4.1");
        assert_eq!(RULE_META_FRAME_TITLE.axe_id, "frame-title");
        assert!(RULE_META_FRAME_TITLE.tags.contains(&"wcag241"));
    }
}
