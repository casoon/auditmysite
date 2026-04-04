//! WCAG 4.1.2 - ARIA Prohibited Attributes
//!
//! Validates that elements do not use ARIA attributes that are explicitly
//! prohibited for their role.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA prohibited attributes
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Prohibited Attributes",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA attributes that are prohibited on a role must not be used",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-prohibited-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles and the ARIA attributes that are prohibited on them
const PROHIBITED_ATTRS: &[(&str, &[&str])] = &[
    ("presentation", &["aria-label", "aria-labelledby"]),
    ("none", &["aria-label", "aria-labelledby"]),
    (
        "generic",
        &["aria-label", "aria-labelledby", "aria-roledescription"],
    ),
    ("code", &["aria-label", "aria-labelledby"]),
    ("emphasis", &["aria-label", "aria-labelledby"]),
    ("strong", &["aria-label", "aria-labelledby"]),
    ("subscript", &["aria-label", "aria-labelledby"]),
    ("superscript", &["aria-label", "aria-labelledby"]),
    ("deletion", &["aria-label", "aria-labelledby"]),
    ("insertion", &["aria-label", "aria-labelledby"]),
];

/// Check that elements do not use ARIA attributes prohibited for their role
pub fn check_aria_prohibited_attr(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        let prohibited = match PROHIBITED_ATTRS.iter().find(|(r, _)| *r == role) {
            Some((_, attrs)) => attrs,
            None => continue,
        };

        for prop in &node.properties {
            if !prop.name.starts_with("aria-") {
                continue;
            }

            let attr_name = prop.name.as_str();

            if prohibited.contains(&attr_name) {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    RULE_META.severity,
                    format!(
                        "ARIA attribute '{}' is prohibited on role '{}'",
                        attr_name, role
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_rule_id(RULE_META.axe_id)
                .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
                .with_fix(format!(
                    "Remove '{}' from this element with role '{}'",
                    attr_name, role
                ))
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
    fn test_no_prohibited_attrs_passes() {
        let nodes = vec![
            make_node("1", "button", vec![("aria-label", "OK")]),
            make_node("2", "presentation", vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_prohibited_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_prohibited_attr_on_presentation_flagged() {
        let nodes = vec![make_node(
            "1",
            "presentation",
            vec![("aria-label", "should not be here")],
        )];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_prohibited_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("aria-label"));
        assert!(results.violations[0].message.contains("presentation"));
    }

    #[test]
    fn test_prohibited_attr_on_generic_flagged() {
        let nodes = vec![make_node(
            "1",
            "generic",
            vec![("aria-roledescription", "fancy thing")],
        )];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_prohibited_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0]
            .message
            .contains("aria-roledescription"));
    }
}
