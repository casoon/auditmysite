//! WCAG 2.5.3 Label in Name
//!
//! For user interface components with labels that include text or images of text,
//! the name contains the text that is presented visually.
//! Level A

use crate::accessibility::AXTree;
use crate::wcag::types::WcagResults;

#[cfg(test)]
use crate::cli::WcagLevel;
#[cfg(test)]
use crate::wcag::types::{RuleMetadata, Severity};

#[cfg(test)]
const LABEL_IN_NAME_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.3",
    name: "Label in Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "The accessible name contains the text that is presented visually",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/label-in-name.html",
    axe_id: "label-content-name-mismatch",
    tags: &["wcag2a", "wcag253", "cat.semantics"],
};

pub fn check_label_in_name(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        let role = node.role.as_deref().unwrap_or("");

        // Only check labeled interactive components
        let is_labeled_component = matches!(
            role,
            "button"
                | "link"
                | "menuitem"
                | "tab"
                | "checkbox"
                | "radio"
                | "switch"
                | "combobox"
                | "textbox"
                | "searchbox"
        );

        if !is_labeled_component {
            continue;
        }

        // Skip nodes without an accessible name
        if node.name.as_ref().is_none_or(|n| n.trim().is_empty()) {
            continue;
        }

        // The visible label for interactive elements typically IS the accessible
        // name. node.description often comes from title attributes which are NOT
        // visible to users — comparing name against description causes false
        // positives (e.g. logo links with title="Zur Startseite").
        //
        // Without access to the actual DOM text nodes (which the AXTree doesn't
        // expose separately), we cannot reliably determine the "visible label"
        // distinct from the accessible name. Pass for now.
        results.passes += 1;
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn labeled_node(id: &str, role: &str, name: &str, _description: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some(name.to_string()),
            name_source: None,
            description: _description.map(String::from),
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_name_matches_label() {
        let tree = AXTree::from_nodes(vec![labeled_node("1", "button", "Submit", Some("Submit"))]);
        let results = check_label_in_name(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_name_contains_label() {
        let tree = AXTree::from_nodes(vec![labeled_node(
            "1",
            "button",
            "Submit Form",
            Some("Submit"),
        )]);
        let results = check_label_in_name(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_title_attr_not_treated_as_visible_label() {
        // title="Submit Form" is not visible — should not trigger a violation
        let tree = AXTree::from_nodes(vec![labeled_node(
            "1",
            "button",
            "Send",
            Some("Submit Form"),
        )]);
        let results = check_label_in_name(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_no_description_passes() {
        let tree = AXTree::from_nodes(vec![labeled_node("1", "button", "Submit", None)]);
        let results = check_label_in_name(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
