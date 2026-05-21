//! WCAG 3.3.1 Error Identification (Level A)
//!
//! When a user input error is automatically detected, the failed element
//! must be identified and the error described in text. The most reliable
//! AXTree signal: form controls marked `aria-invalid="true"` should also
//! reference an error message via `aria-describedby`.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const ERROR_ID_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.1",
    name: "Error Identification",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Detected input errors must be identified and described in text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/error-identification.html",
    axe_id: "aria-invalid-without-describedby",
    tags: &["wcag2a", "wcag331", "cat.forms"],
};

pub fn check_error_identification(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let invalid = node.get_property_bool("invalid").unwrap_or(false);
        if !invalid {
            continue;
        }

        // aria-describedby is a relatedNodes property in the AX tree — use
        // has_property() which matches regardless of value type.
        let has_describedby = node.has_property("describedby")
            || node
                .description
                .as_deref()
                .is_some_and(|d| !d.trim().is_empty());

        if has_describedby {
            results.passes += 1;
            continue;
        }

        let role = node.role.as_deref().unwrap_or("input");
        results.add_violation(
            Violation::new(
                ERROR_ID_RULE.id,
                ERROR_ID_RULE.name,
                ERROR_ID_RULE.level,
                Severity::Medium,
                "Form field is marked aria-invalid=\"true\" but has no accessible error description (no aria-describedby, no description). Screen readers cannot tell users what went wrong.".to_string(),
                &node.node_id,
            )
            .with_role(Some(role.to_string()))
            .with_name(node.name.clone())
            .with_fix(
                "Connect the invalid field to its error message: aria-describedby=\"<error-id>\" on the field, with the message in a visible element (or live region) carrying that id.",
            )
            .with_rule_id(ERROR_ID_RULE.axe_id)
            .with_help_url(ERROR_ID_RULE.help_url),
        );
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn input_node(id: &str, invalid: bool, describedby: Option<&str>) -> AXNode {
        let mut n = AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("textbox".into()),
            name: Some("Email".into()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        };
        if invalid {
            n.properties.push(AXProperty {
                name: "invalid".into(),
                value: AXValue::Bool(true),
            });
        }
        if let Some(target) = describedby {
            n.properties.push(AXProperty {
                name: "describedby".into(),
                value: AXValue::String(target.into()),
            });
        }
        n
    }

    #[test]
    fn test_invalid_without_describedby_flagged() {
        let tree = AXTree::from_nodes(vec![input_node("1", true, None)]);
        let results = check_error_identification(&tree);
        assert!(
            !results.violations.is_empty(),
            "Expected violation for invalid field without describedby"
        );
    }

    #[test]
    fn test_invalid_with_describedby_passes() {
        let tree = AXTree::from_nodes(vec![input_node("1", true, Some("err-1"))]);
        let results = check_error_identification(&tree);
        assert!(results.violations.is_empty());
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_valid_field_not_flagged() {
        let tree = AXTree::from_nodes(vec![input_node("1", false, None)]);
        let results = check_error_identification(&tree);
        assert!(results.violations.is_empty());
    }
}
