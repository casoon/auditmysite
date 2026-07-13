//! WCAG 4.1.2 - ARIA Relationship Attributes
//!
//! Checks for empty ARIA relationship attributes, which break programmatic
//! relationships between elements.
//!
//! This file used to also detect duplicate `id` attributes via
//! `node.get_property_str("id")` — dead code, since CDP never exposes a
//! generic `id` AX property, and duplicate-ID detection already has a
//! working, dedicated implementation in `parsing.rs` (axe ids `duplicate-id`
//! / `duplicate-id-aria`). Removed as part of #QA-009's cleanup.
//!
//! The empty-relationship check itself was also dead code (#QA-030): it read
//! CDP AX properties by their `aria-`-prefixed HTML attribute names, but CDP
//! exposes them unprefixed (`controls`, `owns`, `activedescendant`), and via
//! `get_property_str` which returns `None` for the `AXValue::Node` values real
//! relationship properties carry. Fixed to read the correct property names
//! via `get_property_idrefs`, which handles both value shapes.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA relationship attribute checks
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Relationships",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "ARIA relationship attributes must reference valid, non-empty targets",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-valid-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// ARIA relationship attributes that must not be empty when present.
/// (CDP AX property name, HTML attribute name for messages)
const EMPTY_RELATIONSHIP_ATTRS: &[(&str, &str)] = &[
    ("controls", "aria-controls"),
    ("owns", "aria-owns"),
    ("activedescendant", "aria-activedescendant"),
];

/// Check ARIA relationship attributes for empty values
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for empty ARIA relationships
pub fn check_aria_relationships(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        // Check each relationship attribute for empty values
        for (prop_name, attr_name) in EMPTY_RELATIONSHIP_ATTRS {
            if !node.has_property(prop_name) {
                continue;
            }
            if node.get_property_idrefs(prop_name).is_empty() {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    Severity::Medium,
                    format!("{} references a target but the value is empty", attr_name),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix(format!(
                    "Either provide a valid ID reference for {} or remove the attribute",
                    attr_name
                ))
                .with_rule_id(RULE_META.axe_id)
                .with_help_url(RULE_META.help_url);

                results.add_violation(violation);
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

    fn make_node(id: &str, role: &str) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some(format!("Node {}", id)),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    fn make_node_with_prop(id: &str, role: &str, prop_name: &str, prop_value: &str) -> AXNode {
        let mut node = make_node(id, role);
        node.properties.push(AXProperty {
            name: prop_name.to_string(),
            value: AXValue::String(prop_value.to_string()),
        });
        node
    }

    #[test]
    fn test_empty_aria_controls_flagged() {
        let node = make_node_with_prop("1", "button", "controls", "");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-controls")),
            "Empty aria-controls should be flagged"
        );
    }

    #[test]
    fn test_empty_aria_owns_flagged() {
        let node = make_node_with_prop("1", "combobox", "owns", "");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-owns")),
            "Empty aria-owns should be flagged"
        );
    }

    #[test]
    fn test_empty_aria_activedescendant_flagged() {
        let node = make_node_with_prop("1", "listbox", "activedescendant", "");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-activedescendant")),
            "Empty aria-activedescendant should be flagged"
        );
    }

    #[test]
    fn test_valid_aria_controls_passes() {
        let node = make_node_with_prop("1", "button", "controls", "my-panel");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .all(|v| !v.message.contains("aria-controls")),
            "Valid aria-controls should not be flagged"
        );
    }

    #[test]
    fn test_empty_node_relationship_flagged() {
        // Real CDP traffic carries relationship properties as AXValue::Node,
        // not AXValue::String — an empty related_nodes list must still be
        // flagged (#QA-030 regression check for the get_property_str bug).
        let mut node = make_node("1", "button");
        node.properties.push(AXProperty {
            name: "controls".to_string(),
            value: AXValue::Node {
                related_nodes: vec![],
            },
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-controls")),
            "Empty related_nodes (AXValue::Node shape) should be flagged"
        );
    }

    #[test]
    fn test_valid_node_relationship_passes() {
        let mut node = make_node("1", "button");
        node.properties.push(AXProperty {
            name: "controls".to_string(),
            value: AXValue::Node {
                related_nodes: vec![crate::accessibility::RelatedNode {
                    backend_dom_node_id: None,
                    idref: Some("panel-1".to_string()),
                    text: None,
                }],
            },
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .all(|v| !v.message.contains("aria-controls")),
            "Non-empty related_nodes should not be flagged"
        );
    }
}
