//! WCAG 1.2.1, 1.2.2, 1.2.5, 1.1.1 - Media Rules
//!
//! Checks that media elements, SVGs, and canvas elements have accessible names
//! and that decorative elements are not spuriously named.

use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for media accessibility (1.2.x)
pub const RULE_META_MEDIA: RuleMetadata = RuleMetadata {
    id: "1.2.1",
    name: "Audio-only and Video-only",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Video and audio elements must have accessible alternatives",
    help_url:
        "https://www.w3.org/WAI/WCAG21/Understanding/audio-only-and-video-only-prerecorded.html",
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
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/captions-prerecorded.html",
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
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "image-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

/// Rule metadata for iframe accessible names (axe-core `frame-title`).
pub const RULE_META_FRAME_TITLE: RuleMetadata = RuleMetadata {
    id: "frame-title",
    name: "Frame title",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Frames and iframes must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "frame-title",
    tags: &["wcag2a", "wcag412", "cat.text-alternatives"],
};

/// Run all media-related WCAG checks
pub fn check_media_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    let mut video_element_count = 0usize;

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
                video_element_count += 1;
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

    // 1.2.2 Caption quality cannot be verified automatically — the AXTree
    // reveals that a media element is present, but whether captions are
    // accurate, complete, and synchronized requires human review.
    if video_element_count > 0 {
        results.add_violation(
            Violation::new(
                RULE_META_CAPTIONS.id,
                RULE_META_CAPTIONS.name,
                RULE_META_CAPTIONS.level,
                Severity::High,
                format!(
                    "{video_element_count} media element(s) detected. \
                     Caption presence and accuracy cannot be verified automatically — \
                     review each video for correct, synchronized captions."
                ),
                "page",
            )
            .with_fix(
                "Ensure all prerecorded video with audio has synchronized captions. \
                 Use the <track kind=\"captions\"> element or a captioning service.",
            )
            .with_help_url(RULE_META_CAPTIONS.help_url)
            .with_kind(FindingKind::NotTestable),
        );
    }

    results
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
            return vec![];
        }
    };

    let Some(value) = result.value() else {
        return vec![];
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
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
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
            return vec![];
        }
    };

    let Some(value) = result.value() else {
        return vec![];
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
            .with_kind(FindingKind::NotTestable);

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
        .with_help_url(RULE_META_MEDIA.help_url);

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
        .with_help_url(RULE_META_IMAGE.help_url);

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
        .with_help_url(RULE_META_IMAGE.help_url);

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
}
