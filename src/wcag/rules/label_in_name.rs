//! WCAG 2.5.3 Label in Name
//!
//! For user interface components with labels that include text or images of text,
//! the name contains the text that is presented visually.
//! Level A

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const LABEL_IN_NAME_RULE: RuleMetadata = RuleMetadata {
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

        // Get accessible name and visible label (description often contains visible text)
        let accessible_name = match &node.name {
            Some(name) if !name.trim().is_empty() => name.trim().to_lowercase(),
            _ => continue, // No accessible name to check
        };

        // If the node has a description that differs significantly from the name,
        // check if the name at least contains the visible label text
        if let Some(desc) = &node.description {
            let visible_text = desc.trim().to_lowercase();
            if !visible_text.is_empty()
                && visible_text != accessible_name
                && !accessible_name.contains(&visible_text)
            {
                let violation = Violation::new(
                    LABEL_IN_NAME_RULE.id,
                    LABEL_IN_NAME_RULE.name,
                    LABEL_IN_NAME_RULE.level,
                    Severity::High,
                    format!(
                        "Accessible name '{}' does not contain visible label '{}'",
                        node.name.as_deref().unwrap_or(""),
                        desc.trim()
                    ),
                    node.node_id.clone(),
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix(
                    "Ensure the accessible name starts with or contains the visible text label",
                )
                .with_help_url(LABEL_IN_NAME_RULE.help_url);

                results.add_violation(violation);
                continue;
            }
        }

        results.passes += 1;
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn labeled_node(id: &str, role: &str, name: &str, description: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some(name.to_string()),
            name_source: None,
            description: description.map(String::from),
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
    fn test_name_doesnt_contain_label() {
        let tree = AXTree::from_nodes(vec![labeled_node(
            "1",
            "button",
            "Send",
            Some("Submit Form"),
        )]);
        let results = check_label_in_name(&tree);
        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].rule, "2.5.3");
    }

    #[test]
    fn test_no_description_passes() {
        let tree = AXTree::from_nodes(vec![labeled_node("1", "button", "Submit", None)]);
        let results = check_label_in_name(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
