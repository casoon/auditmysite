//! WCAG 1.1.1 - SVG Accessibility Rules
//!
//! Dedicated rules for SVG elements exposed with role="img" in the accessibility tree.
//! Focuses on SVG-specific patterns distinct from general media checks.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for SVG accessibility (1.1.1)
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Non-text Content - SVG",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "SVG images must have meaningful accessible names; decorative SVGs must be hidden from assistive technology",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "svg-img-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

/// Run all SVG-specific WCAG checks
///
/// This rule focuses on SVG elements that appear as role="img" in the AX tree,
/// which is distinct from the broader media_rules checks.
pub fn check_svg_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        // SVGs most commonly appear as role="img" or "SVGRoot"/"SvgRoot" in the AX tree
        let is_svg = role == "img"
            || role == "SVGRoot"
            || role == "SvgRoot"
            // Some browsers expose SVG with an explicit SVG-related name source
            || node
                .name_source
                .map(|ns| {
                    // If it's an img with no name_source hint, it might be SVG
                    matches!(
                        ns,
                        crate::accessibility::NameSource::Attribute
                            | crate::accessibility::NameSource::RelatedElement
                    )
                })
                .unwrap_or(false);

        if !is_svg {
            continue;
        }

        results.nodes_checked += 1;

        match role {
            "SVGRoot" | "SvgRoot" => {
                // SVG root nodes that are focusable or visible need an accessible name
                check_svg_root_has_name(node, &mut results);
            }
            "img" => {
                // role=img on an SVG element: check name and whitespace-only names
                check_svg_img_name(node, &mut results);
            }
            _ => {}
        }
    }

    results
}

/// SVG root elements that are not hidden should have a title or aria-label
fn check_svg_root_has_name(node: &AXNode, results: &mut WcagResults) {
    if !node.has_name() {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Medium,
            "SVG root element has no accessible name — consider adding a <title> or aria-hidden=\"true\" if decorative",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix(
            "Add a <title> as the first child of the <svg>, use aria-label, or hide decorative SVGs with aria-hidden=\"true\"",
        )
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// SVG images exposed as role="img" need non-empty, non-whitespace accessible names
fn check_svg_img_name(node: &AXNode, results: &mut WcagResults) {
    match node.name.as_deref() {
        None => {
            // No name at all
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::High,
                "SVG image is missing an accessible name",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix(
                "Add a <title> element as the first child of the SVG, or use aria-label/aria-labelledby",
            )
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        }
        Some(name) if name.trim().is_empty() => {
            // Whitespace-only name
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::Medium,
                "SVG image has empty accessible name",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Replace the empty accessible name with a meaningful description, or mark the SVG as decorative with role=\"presentation\" and aria-hidden=\"true\"",
            )
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        }
        Some(_) => {
            // Valid name present — check for non-interactivity suggesting it could be decorative
            // This is informational only (Low severity)
            let is_interactive = node.is_interactive() || node.is_focusable();
            let _ = is_interactive; // informational, not flagged
            results.passes += 1;
        }
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
        let results = check_svg_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("missing an accessible name")));
    }

    #[test]
    fn test_svg_img_with_whitespace_name_flagged() {
        let nodes = vec![make_node("1", "img", Some("   "))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_svg_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("empty accessible name")));
    }

    #[test]
    fn test_svg_img_with_name_passes() {
        let nodes = vec![make_node("1", "img", Some("Company logo"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_svg_rules(&tree);
        assert!(results.violations.is_empty());
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_svg_root_without_name_flagged() {
        let nodes = vec![make_node("1", "SVGRoot", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_svg_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible name")));
    }

    #[test]
    fn test_svg_root_with_name_passes() {
        let nodes = vec![make_node("1", "SVGRoot", Some("Chart: Monthly Sales"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_svg_rules(&tree);
        assert!(results.violations.is_empty());
    }
}
