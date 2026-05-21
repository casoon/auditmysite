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
        r#"
        var issues = [];
        var frames = document.querySelectorAll('iframe, frame');
        for (var i = 0; i < frames.length; i++) {
          var el = frames[i];
          var role = (el.getAttribute('role') || '').toLowerCase();
          if (role === 'none' || role === 'presentation') continue;
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
