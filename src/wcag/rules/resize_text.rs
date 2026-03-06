//! WCAG 1.4.4 Resize Text
//!
//! Except for captions and images of text, text can be resized without
//! assistive technology up to 200 percent without loss of content or functionality.
//! Level AA
//!
//! Note: Full resize testing requires viewport manipulation via CDP.
//! This rule checks for common patterns that prevent text resizing.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const RESIZE_TEXT_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.4",
    name: "Resize Text",
    level: WcagLevel::AA,
    severity: Severity::Serious,
    description: "Text can be resized up to 200% without loss of content or functionality",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/resize-text.html",
};

pub fn check_resize_text(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Check the root WebArea for viewport meta that prevents zooming
    if let Some(root) = tree.root() {
        // Check for user-scalable=no or maximum-scale < 2 in viewport meta
        // This info may be in properties or the tree structure
        let viewport_info = root.get_property_str("viewport");
        if let Some(viewport) = viewport_info {
            let vp_lower = viewport.to_lowercase();
            if vp_lower.contains("user-scalable=no") || vp_lower.contains("user-scalable=0") {
                let violation = Violation::new(
                    RESIZE_TEXT_RULE.id,
                    RESIZE_TEXT_RULE.name,
                    RESIZE_TEXT_RULE.level,
                    Severity::Serious,
                    "Viewport meta tag prevents user scaling (user-scalable=no)",
                    root.node_id.clone(),
                )
                .with_fix("Remove user-scalable=no from viewport meta tag to allow text resizing")
                .with_help_url(RESIZE_TEXT_RULE.help_url);

                results.add_violation(violation);
            }

            // Check maximum-scale
            if let Some(max_scale_pos) = vp_lower.find("maximum-scale=") {
                let value_str = &vp_lower[max_scale_pos + 14..];
                let value_end = value_str
                    .find(|c: char| !c.is_ascii_digit() && c != '.')
                    .unwrap_or(value_str.len());
                if let Ok(max_scale) = value_str[..value_end].parse::<f32>() {
                    if max_scale < 2.0 {
                        let violation = Violation::new(
                            RESIZE_TEXT_RULE.id,
                            RESIZE_TEXT_RULE.name,
                            RESIZE_TEXT_RULE.level,
                            Severity::Moderate,
                            format!(
                                "Viewport maximum-scale={:.1} is less than 2.0, limiting text resize",
                                max_scale
                            ),
                            root.node_id.clone(),
                        )
                        .with_fix("Set maximum-scale to at least 2.0 or remove it entirely")
                        .with_help_url(RESIZE_TEXT_RULE.help_url);

                        results.add_violation(violation);
                    }
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn root_with_viewport(viewport: &str) -> AXNode {
        AXNode {
            node_id: "root".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("WebArea".to_string()),
            name: Some("Test Page".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "viewport".to_string(),
                value: AXValue::String(viewport.to_string()),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_no_viewport_restriction() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, initial-scale=1",
        )]);
        let results = check_resize_text(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_user_scalable_no() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, user-scalable=no",
        )]);
        let results = check_resize_text(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("user-scalable=no"));
    }

    #[test]
    fn test_maximum_scale_too_low() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, maximum-scale=1.0",
        )]);
        let results = check_resize_text(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("maximum-scale"));
    }
}
