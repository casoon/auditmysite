//! WCAG 4.1.2 - Name, Role, Value
//!
//! For all user interface components, the name and role can be programmatically determined.
//! This rule checks that form controls have accessible names.
//!
//! Links und Buttons ohne Namen laufen seit #690 als `links/name-missing`
//! bzw. `buttons/name-missing` im geteilten Bestand (siehe `wcag::shared`).

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 4.1.2
pub(super) const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value",
    level: WcagLevel::A,
    severity: Severity::High,
    description:
        "For all user interface components, the name and role can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    // Deliberately not "label": that axe_id is already used by
    // `instructions.rs`/`form_rules.rs`'s WCAG 3.3.2 label/instructions
    // family. Both used to share the literal string "label", which is
    // harmless today (their grouping key falls back to the raw WCAG
    // criterion, "4.1.2" vs "3.3.2", so they never actually merged) but
    // would have silently merged this file's 4.1.2 findings into the 3.3.2
    // bucket (or vice versa) the moment either got its own
    // `LEGACY_WCAG_MAP` entry keyed by axe_id (plan/1
    // root-cause-title-occurrence-mismatch.md, "Mechanism 2").
    axe_id: "control-missing-label",
    tags: &["wcag2a", "wcag412", "cat.forms"],
};

/// Check that form controls have accessible names
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for form controls missing accessible names
pub fn check_labels(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Check form controls
    let form_controls = tree.form_controls();
    results.nodes_checked += form_controls.len();

    for control in form_controls {
        if control.ignored {
            continue;
        }

        check_form_control(control, &mut results);
    }

    results
}

/// Check a single form control
fn check_form_control(node: &crate::accessibility::AXNode, results: &mut WcagResults) {
    let role = node.role.as_deref().unwrap_or("unknown");

    // Check for accessible name
    if !node.has_name() {
        let message = match role {
            "textbox" => "Text input field is missing a label",
            "checkbox" => "Checkbox is missing a label",
            "radio" => "Radio button is missing a label",
            "combobox" => "Dropdown/select is missing a label",
            "listbox" => "List box is missing a label",
            "spinbutton" => "Spin button is missing a label",
            "slider" => "Slider is missing a label",
            "searchbox" => "Search field is missing a label",
            _ => "Form control is missing a label",
        };

        let fix = match role {
            "textbox" | "searchbox" => {
                "Add a <label> element with 'for' attribute, or use aria-label/aria-labelledby"
            }
            "checkbox" | "radio" => "Wrap in a <label> element, or use aria-label",
            _ => "Add aria-label or aria-labelledby attribute",
        };

        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            RULE_META.severity,
            message,
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix(fix)
        .with_help_url(RULE_META.help_url)
        .with_rule_id(RULE_META.axe_id);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn create_control_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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
    fn test_labeled_textbox() {
        let nodes = vec![create_control_node("1", "textbox", Some("Email Address"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_labels(&tree);

        assert_eq!(results.violations.len(), 0);
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_unlabeled_textbox() {
        let nodes = vec![create_control_node("1", "textbox", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_labels(&tree);

        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].rule, "4.1.2");
        assert!(results.violations[0].message.contains("label"));
    }

    #[test]
    fn test_generic_link_text_is_left_to_link_purpose() {
        let nodes = vec![create_control_node("1", "link", Some("click here"))];
        let tree = AXTree::from_nodes(nodes);

        // One rule per defect: labels checks form controls only, 2.4.4 is
        // link_purpose's.
        assert!(check_labels(&tree).violations.is_empty());
        let purpose = crate::wcag::rules::link_purpose::check_link_purpose(&tree);
        assert!(purpose
            .violations
            .iter()
            .chain(&purpose.warnings)
            .any(|v| v.rule == "2.4.4"));
    }
}
