//! WCAG 1.4.4 - Meta Viewport Large Scale
//!
//! axe-core rule: `meta-viewport-large`
//! Users should be able to zoom to at least 500%. If the viewport meta tag
//! sets `maximum-scale` < 5 or `user-scalable=no`, this rule fires.
//!
//! This is a PAGE-level rule that inspects the viewport property on the
//! root document node (role "RootWebArea" or "document").

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const RULE_META_VIEWPORT_LARGE: RuleMetadata = RuleMetadata {
    id: "1.4.4",
    name: "Meta Viewport Large Scale",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "The viewport meta tag must allow users to scale the page to at least 500%",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/resize-text.html",
    axe_id: "meta-viewport-large",
    tags: &["wcag2aa", "wcag144", "cat.sensory-and-visual-cues"],
};

/// Check that viewport meta allows zoom to at least 500% (maximum-scale >= 5).
pub fn check_meta_viewport_large(tree: &AXTree) -> WcagResults {
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

        // Only check root-level document nodes
        if !matches!(role.as_str(), "rootwebarea" | "document" | "webarea") {
            continue;
        }

        let viewport = match node.get_property_str("viewport") {
            Some(v) => v,
            None => continue,
        };

        if is_viewport_restricted(viewport) {
            results.add_violation(
                Violation::new(
                    RULE_META_VIEWPORT_LARGE.id,
                    RULE_META_VIEWPORT_LARGE.name,
                    RULE_META_VIEWPORT_LARGE.level,
                    RULE_META_VIEWPORT_LARGE.severity,
                    "Viewport meta tag restricts zoom below 500% (maximum-scale < 5 or user-scalable=no)",
                    &node.node_id,
                )
                .with_fix(
                    "Set maximum-scale to at least 5 or remove maximum-scale restriction",
                )
                .with_rule_id(RULE_META_VIEWPORT_LARGE.axe_id)
                .with_help_url(RULE_META_VIEWPORT_LARGE.help_url),
            );
        } else {
            results.passes += 1;
        }
    }

    results
}

/// Returns true if the viewport content string restricts zoom below 500%.
fn is_viewport_restricted(viewport: &str) -> bool {
    let content = viewport.to_lowercase();

    // user-scalable=no or user-scalable=0 always restricts
    if content.contains("user-scalable=no") || content.contains("user-scalable=0") {
        return true;
    }

    // Parse maximum-scale=<value>
    if let Some(pos) = content.find("maximum-scale=") {
        let after = &content[pos + "maximum-scale=".len()..];
        let value_str: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(val) = value_str.parse::<f64>() {
            return val < 5.0;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn root_with_viewport(viewport: &str) -> AXNode {
        AXNode {
            node_id: "root".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("RootWebArea".into()),
            name: Some("Test Page".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "viewport".into(),
                value: AXValue::String(viewport.into()),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_maximum_scale_below_5_flagged() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, maximum-scale=3",
        )]);
        let r = check_meta_viewport_large(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("meta-viewport-large")));
    }

    #[test]
    fn test_user_scalable_no_flagged() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, user-scalable=no",
        )]);
        let r = check_meta_viewport_large(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("meta-viewport-large")));
    }

    #[test]
    fn test_maximum_scale_5_passes() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, maximum-scale=5",
        )]);
        let r = check_meta_viewport_large(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_no_maximum_scale_passes() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, initial-scale=1",
        )]);
        let r = check_meta_viewport_large(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_maximum_scale_10_passes() {
        let tree = AXTree::from_nodes(vec![root_with_viewport(
            "width=device-width, maximum-scale=10",
        )]);
        let r = check_meta_viewport_large(&tree);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_is_viewport_restricted() {
        assert!(is_viewport_restricted("maximum-scale=1"));
        assert!(is_viewport_restricted("maximum-scale=4.9"));
        assert!(is_viewport_restricted("user-scalable=no"));
        assert!(is_viewport_restricted("user-scalable=0"));
        assert!(!is_viewport_restricted(
            "width=device-width, initial-scale=1"
        ));
        assert!(!is_viewport_restricted("maximum-scale=5"));
        assert!(!is_viewport_restricted("maximum-scale=10"));
    }
}
