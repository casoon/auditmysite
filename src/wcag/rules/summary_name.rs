//! WCAG 4.1.2 - Summary Accessible Name
//!
//! axe-core rule: `summary-name`
//! `<summary>` elements inside `<details>` disclosure widgets must have an
//! accessible name so screen reader users understand the purpose of the toggle.
//! Chrome's AX tree exposes `<details>` via an htmlTag property, and
//! `<summary>` elements may appear with role "DisclosureTriangle" or similar.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const RULE_SUMMARY_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Summary Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Summary elements must have an accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "summary-name",
    tags: &["wcag2a", "wcag412", "cat.name-role-value"],
};

/// Check that disclosure widgets have an accessible summary name.
///
/// Matches nodes that represent `<summary>` / `<details>` elements:
/// - role "DisclosureTriangle" (Chrome's AX role for `<summary>`)
/// - htmlTag "SUMMARY"
/// - htmlTag "DETAILS" (the container — its name typically comes from the summary)
pub fn check_summary_name(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = node.role.as_deref().unwrap_or_default().to_lowercase();

        let is_summary = role == "disclosuretriangle"
            || node
                .get_property_str("htmlTag")
                .is_some_and(|t| t.eq_ignore_ascii_case("SUMMARY"))
            || node
                .get_property_str("htmlTag")
                .is_some_and(|t| t.eq_ignore_ascii_case("DETAILS"));

        if is_summary {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_SUMMARY_NAME.id,
                        RULE_SUMMARY_NAME.name,
                        RULE_SUMMARY_NAME.level,
                        RULE_SUMMARY_NAME.severity,
                        "Disclosure widget has no accessible name (missing or empty <summary> text)",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add text content to the summary element")
                    .with_rule_id(RULE_SUMMARY_NAME.axe_id)
                    .with_help_url(RULE_SUMMARY_NAME.help_url),
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

    fn node_with_role(id: &str, role: &str, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
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

    fn node_with_html_tag(id: &str, role: &str, name: Option<&str>, tag: &str) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "htmlTag".into(),
                value: AXValue::String(tag.into()),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_disclosure_triangle_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_role("1", "DisclosureTriangle", None)]);
        let r = check_summary_name(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("summary-name")));
    }

    #[test]
    fn test_disclosure_triangle_with_name_passes() {
        let tree = AXTree::from_nodes(vec![node_with_role(
            "1",
            "DisclosureTriangle",
            Some("More details"),
        )]);
        let r = check_summary_name(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_details_html_tag_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_html_tag("1", "group", None, "DETAILS")]);
        let r = check_summary_name(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("summary-name")));
    }

    #[test]
    fn test_details_html_tag_with_name_passes() {
        let tree = AXTree::from_nodes(vec![node_with_html_tag(
            "1",
            "group",
            Some("FAQ Section"),
            "DETAILS",
        )]);
        let r = check_summary_name(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_summary_html_tag_without_name_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_html_tag("1", "button", None, "SUMMARY")]);
        let r = check_summary_name(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("summary-name")));
    }

    #[test]
    fn test_summary_html_tag_with_name_passes() {
        let tree = AXTree::from_nodes(vec![node_with_html_tag(
            "1",
            "button",
            Some("Toggle section"),
            "SUMMARY",
        )]);
        let r = check_summary_name(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_unrelated_node_not_flagged() {
        let tree = AXTree::from_nodes(vec![node_with_role("1", "button", None)]);
        let r = check_summary_name(&tree);
        assert!(r.violations.is_empty());
    }
}
