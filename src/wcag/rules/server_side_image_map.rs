//! WCAG 1.1.1 - Server-side Image Map
//!
//! axe-core rule: `server-side-image-map`
//! Server-side image maps are inaccessible because the coordinates are sent
//! to the server and keyboard users cannot interact with them meaningfully.
//! Recommends replacing with client-side image maps or text links.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const RULE_SERVER_SIDE_IMAGE_MAP: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Server-side Image Map",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description:
        "Server-side image maps must not be used; use client-side image maps or text links instead",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "server-side-image-map",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

/// Check for server-side image maps (`<img ismap>`).
pub fn check_server_side_image_map(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r.to_lowercase(),
            None => continue,
        };

        // <img> elements appear as "image" or "img" in the AX tree
        if role == "image" || role == "img" {
            // Check for ismap property being true
            let has_ismap = node.get_property_bool("ismap").unwrap_or(false)
                || node
                    .get_property_str("ismap")
                    .is_some_and(|v| v.eq_ignore_ascii_case("true") || v == "1");

            if has_ismap {
                results.add_violation(
                    Violation::new(
                        RULE_SERVER_SIDE_IMAGE_MAP.id,
                        RULE_SERVER_SIDE_IMAGE_MAP.name,
                        RULE_SERVER_SIDE_IMAGE_MAP.level,
                        RULE_SERVER_SIDE_IMAGE_MAP.severity,
                        "Image uses a server-side image map (ismap attribute), which is inaccessible to keyboard and screen reader users",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Replace server-side image map with client-side image map or text links")
                    .with_rule_id(RULE_SERVER_SIDE_IMAGE_MAP.axe_id)
                    .with_help_url(RULE_SERVER_SIDE_IMAGE_MAP.help_url),
                );
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn img_node(id: &str, name: Option<&str>, ismap: Option<bool>) -> AXNode {
        let mut props = vec![];
        if let Some(val) = ismap {
            props.push(AXProperty {
                name: "ismap".into(),
                value: AXValue::Bool(val),
            });
        }
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".into()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: props,
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_server_side_image_map_flagged() {
        let tree = AXTree::from_nodes(vec![img_node("1", Some("Map"), Some(true))]);
        let r = check_server_side_image_map(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("server-side-image-map")));
    }

    #[test]
    fn test_image_without_ismap_passes() {
        let tree = AXTree::from_nodes(vec![img_node("1", Some("Photo"), None)]);
        let r = check_server_side_image_map(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_image_with_ismap_false_passes() {
        let tree = AXTree::from_nodes(vec![img_node("1", Some("Photo"), Some(false))]);
        let r = check_server_side_image_map(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_ismap_string_true_flagged() {
        let node = AXNode {
            node_id: "1".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".into()),
            name: Some("Map".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "ismap".into(),
                value: AXValue::String("true".into()),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        };
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_server_side_image_map(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("server-side-image-map")));
    }

    #[test]
    fn test_non_image_role_ignored() {
        let node = AXNode {
            node_id: "1".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("button".into()),
            name: Some("Click".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "ismap".into(),
                value: AXValue::Bool(true),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        };
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_server_side_image_map(&tree);
        assert!(r.violations.is_empty());
    }
}
