//! WCAG 4.1.2 - ARIA Required Attributes
//!
//! Validates that elements with certain ARIA roles have the required ARIA attributes.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA required attributes
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Required Attributes",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "Roles that require specific ARIA attributes must have them present",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-required-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles and their required ARIA attributes
const REQUIRED_ATTRS: &[(&str, &[&str])] = &[
    ("checkbox", &["aria-checked"]),
    ("combobox", &["aria-expanded"]),
    ("heading", &["aria-level"]),
    ("meter", &["aria-valuenow"]),
    ("radio", &["aria-checked"]),
    ("scrollbar", &["aria-controls", "aria-valuenow"]),
    ("separator", &["aria-valuenow"]),
    ("slider", &["aria-valuenow"]),
    ("spinbutton", &["aria-valuenow"]),
    ("switch", &["aria-checked"]),
];

/// Check that required ARIA attributes are present for each role
pub fn check_aria_required_attr(tree: &AXTree) -> WcagResults {
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

        let required = match REQUIRED_ATTRS.iter().find(|(r, _)| *r == role) {
            Some((_, attrs)) => attrs,
            None => continue,
        };

        // Collect property names on this node
        let prop_names: Vec<&str> = node.properties.iter().map(|p| p.name.as_str()).collect();

        for &req_attr in *required {
            // Native headings (h1-h6) have implicit aria-level via the "level" property
            if req_attr == "aria-level" && role == "heading" && prop_names.contains(&"level") {
                continue;
            }
            if !prop_names.contains(&req_attr) {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    RULE_META.severity,
                    format!(
                        "Element with role '{}' is missing required attribute '{}'",
                        role, req_attr
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_rule_id(RULE_META.axe_id)
                .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
                .with_fix(format!("Add '{}' attribute to this element", req_attr))
                .with_help_url(RULE_META.help_url);

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

    fn make_node(id: &str, role: &str, props: Vec<(&str, &str)>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some(format!("Node {}", id)),
            name_source: None,
            description: None,
            value: None,
            properties: props
                .into_iter()
                .map(|(n, v)| AXProperty {
                    name: n.to_string(),
                    value: AXValue::String(v.to_string()),
                })
                .collect(),
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_required_attr_present_passes() {
        let nodes = vec![
            make_node("1", "checkbox", vec![("aria-checked", "false")]),
            make_node("2", "slider", vec![("aria-valuenow", "50")]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_missing_required_attr_flagged() {
        // checkbox without aria-checked
        let nodes = vec![make_node("1", "checkbox", vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("aria-checked"));
    }

    #[test]
    fn test_ignored_node_skipped() {
        let mut node = make_node("1", "checkbox", vec![]);
        node.ignored = true;
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
