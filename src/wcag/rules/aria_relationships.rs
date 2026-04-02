//! WCAG 4.1.2 - ARIA Relationship Attributes
//!
//! Checks for empty ARIA relationship attributes and duplicate node IDs,
//! which can break programmatic relationships between elements.

use std::collections::HashMap;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA relationship attribute checks
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Relationships",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "ARIA relationship attributes must reference valid, non-empty targets and IDs must be unique",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-valid-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// ARIA relationship attributes that must not be empty when present
const EMPTY_RELATIONSHIP_ATTRS: &[(&str, &str)] = &[
    (
        "aria-controls",
        "aria-controls references a controlled element but the value is empty",
    ),
    (
        "aria-owns",
        "aria-owns references owned elements but the value is empty",
    ),
    (
        "aria-activedescendant",
        "aria-activedescendant references an active element but the value is empty",
    ),
];

/// Check ARIA relationship attributes for empty values and duplicate IDs
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for empty ARIA relationships and duplicate IDs
pub fn check_aria_relationships(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Track IDs for duplicate detection: id value -> list of node_ids that have it
    let mut id_map: HashMap<String, Vec<String>> = HashMap::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        // Check each relationship attribute for empty values
        for (attr_name, message) in EMPTY_RELATIONSHIP_ATTRS {
            if let Some(val) = node.get_property_str(attr_name) {
                if val.trim().is_empty() {
                    let violation = Violation::new(
                        RULE_META.id,
                        RULE_META.name,
                        RULE_META.level,
                        Severity::Medium,
                        *message,
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix(format!(
                        "Either provide a valid ID reference for {} or remove the attribute",
                        attr_name
                    ))
                    .with_help_url(RULE_META.help_url);

                    results.add_violation(violation);
                } else {
                    results.passes += 1;
                }
            }
        }

        // Collect `id` property values for duplicate detection
        if let Some(id_val) = node.get_property_str("id") {
            if !id_val.trim().is_empty() {
                id_map
                    .entry(id_val.to_string())
                    .or_default()
                    .push(node.node_id.clone());
            }
        }
    }

    // Report duplicate IDs
    for (id_val, node_ids) in &id_map {
        if node_ids.len() > 1 {
            for ax_node_id in node_ids {
                let violation = Violation::new(
                    RULE_META.id,
                    "Duplicate ID",
                    RULE_META.level,
                    Severity::High,
                    format!(
                        "Duplicate id attribute found: '{}' (appears {} times)",
                        id_val,
                        node_ids.len()
                    ),
                    ax_node_id,
                )
                .with_fix("Ensure each id attribute value is unique within the page")
                .with_help_url("https://www.w3.org/TR/WCAG21/#parsing");

                results.add_violation(violation);
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
        let node = make_node_with_prop("1", "button", "aria-controls", "");
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
        let node = make_node_with_prop("1", "combobox", "aria-owns", "");
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
        let node = make_node_with_prop("1", "listbox", "aria-activedescendant", "");
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
        let node = make_node_with_prop("1", "button", "aria-controls", "my-panel");
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
    fn test_duplicate_ids_flagged() {
        let node1 = make_node_with_prop("1", "button", "id", "my-id");
        let node2 = make_node_with_prop("2", "link", "id", "my-id");
        let tree = AXTree::from_nodes(vec![node1, node2]);
        let results = check_aria_relationships(&tree);
        let dup_violations: Vec<_> = results
            .violations
            .iter()
            .filter(|v| v.message.contains("Duplicate id"))
            .collect();
        assert_eq!(
            dup_violations.len(),
            2,
            "Both nodes with duplicate ID should be flagged"
        );
    }

    #[test]
    fn test_unique_ids_pass() {
        let node1 = make_node_with_prop("1", "button", "id", "btn-1");
        let node2 = make_node_with_prop("2", "link", "id", "link-1");
        let tree = AXTree::from_nodes(vec![node1, node2]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .all(|v| !v.message.contains("Duplicate id")),
            "Unique IDs should not produce violations"
        );
    }
}
