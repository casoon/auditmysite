//! WCAG 1.2.1, 1.2.2, 1.2.5, 1.1.1 - Media Rules
//!
//! Checks that media elements, SVGs, and canvas elements have accessible names
//! and that decorative elements are not spuriously named.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

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

/// Run all media-related WCAG checks
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
