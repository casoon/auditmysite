//! WCAG 1.1.1 - Non-text Content (Text Alternatives)
//!
//! All non-text content has a text alternative that serves the equivalent purpose.
//! This includes images, icons, charts, and other visual content.

use std::collections::HashSet;

use crate::accessibility::{AXTree, NameSource};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 1.1.1
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Non-text Content",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "All non-text content has a text alternative that serves the equivalent purpose",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "image-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

/// Check for missing text alternatives on images
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for images missing alt text
pub fn check_text_alternatives(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Get all image nodes
    let images = tree.images();
    results.nodes_checked = images.len();

    // Node IDs already evaluated as images, so check_icons does not flag the same
    // role="img" node a second time (#487 false-positive double counting).
    let mut flagged_image_ids: HashSet<&str> = HashSet::new();

    for image in images {
        // Skip ignored nodes (they're intentionally hidden from AT)
        if image.ignored {
            continue;
        }

        // Skip explicitly decorative images: an empty name that comes from a name
        // attribute (alt="" / aria-label="") is intentional, not a missing
        // alternative. Lazy-load placeholders (data-URI src) keep the empty alt
        // but are not marked `ignored` in headless Chrome, so they would
        // otherwise be flagged en masse (#487).
        if is_decorative_empty_name(image) {
            results.passes += 1;
            continue;
        }

        // Check if image has an accessible name
        if !image.has_name() {
            flagged_image_ids.insert(image.node_id.as_str());
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                "Image is missing alternative text",
                &image.node_id,
            )
            .with_role(image.role.clone())
            .with_fix(
                "Add an alt attribute describing the image content, or alt=\"\" if decorative",
            )
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }

    // Also check for other non-text content
    check_icons(tree, &flagged_image_ids, &mut results);
    check_svg_elements(tree, &mut results);

    results
}

/// True when the node has no accessible name but the (empty) name was supplied
/// by a name attribute such as `alt=""` or `aria-label=""` — i.e. the author
/// explicitly marked it decorative. A genuinely missing `alt` has no attribute
/// name source, so it stays flagged.
fn is_decorative_empty_name(node: &crate::accessibility::AXNode) -> bool {
    !node.has_name() && node.name.is_some() && node.name_source == Some(NameSource::Attribute)
}

/// Check icon elements for text alternatives
fn check_icons(tree: &AXTree, flagged_image_ids: &HashSet<&str>, results: &mut WcagResults) {
    // Icons might have role="img" but different implementation
    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        // Skip nodes already flagged by the main image loop (#487 dedup) and
        // explicitly-decorative empty-name nodes.
        if flagged_image_ids.contains(node.node_id.as_str()) || is_decorative_empty_name(node) {
            continue;
        }

        // Check for icon patterns
        let is_icon = node.role.as_deref() == Some("img")
            || node
                .name
                .as_ref()
                .is_some_and(|n| n.contains("icon") || n.contains("Icon"));

        if is_icon && !node.has_name() {
            // Only flag if it seems meaningful (not decorative)
            let likely_decorative = node.get_property_str("hidden").is_some();

            if !likely_decorative {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    Severity::Medium,
                    "Icon element may need alternative text",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix(
                    "Add aria-label for meaningful icons, or aria-hidden=\"true\" for decorative",
                )
                .with_help_url(RULE_META.help_url);

                results.add_violation(violation);
            }
        }
    }
}

/// Check SVG elements for text alternatives
fn check_svg_elements(tree: &AXTree, results: &mut WcagResults) {
    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        // SVG elements often appear as graphics role
        if (node.role.as_deref() == Some("graphics-document")
            || node.role.as_deref() == Some("graphics-symbol"))
            && !node.has_name()
        {
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::High,
                "SVG graphic is missing alternative text",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix("Add <title> element inside SVG, or aria-label on the SVG element")
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXNode;

    fn create_image_node(id: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".to_string()),
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
    fn test_image_with_alt() {
        let nodes = vec![create_image_node("1", Some("Company Logo"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_text_alternatives(&tree);

        assert_eq!(results.violations.len(), 0);
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_image_without_alt() {
        let nodes = vec![create_image_node("1", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_text_alternatives(&tree);

        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].rule, "1.1.1");
    }

    #[test]
    fn test_multiple_images() {
        let nodes = vec![
            create_image_node("1", Some("Logo")),
            create_image_node("2", None),
            create_image_node("3", Some("Banner")),
            create_image_node("4", None),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_text_alternatives(&tree);

        assert_eq!(results.violations.len(), 2);
        assert_eq!(results.passes, 2);
    }

    #[test]
    fn test_ignored_image_not_flagged() {
        let mut node = create_image_node("1", None);
        node.ignored = true;

        let tree = AXTree::from_nodes(vec![node]);
        let results = check_text_alternatives(&tree);

        // Ignored nodes should not be flagged
        assert_eq!(results.violations.len(), 0);
    }
}
