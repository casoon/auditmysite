//! WCAG 4.1.2 Extended - Accessible Name Checks
//!
//! Checks interactive elements for empty or inadequate accessible names,
//! icon-only controls, redundant descriptions, and empty ARIA label references.

use crate::accessibility::{AXTree, NameSource};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for accessible name checks
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Accessible Name",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Interactive elements must have non-empty, meaningful accessible names",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-label",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles considered interactive and requiring an accessible name
const INTERACTIVE_ROLES: &[&str] = &[
    "button",
    "link",
    "textbox",
    "checkbox",
    "radio",
    "combobox",
    "listbox",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "tab",
    "treeitem",
    "slider",
    "spinbutton",
    "searchbox",
    "switch",
];

/// Characters considered icon-only / symbol (single non-alphabetic chars or common symbols)
/// We check if the name is a single character that is not a standard alphanumeric letter.
fn is_icon_only_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Single character that is not ASCII alphanumeric
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() == 1 {
        let c = chars[0];
        return !c.is_ascii_alphanumeric();
    }
    false
}

/// Check accessible names across interactive elements
pub fn check_accessible_name(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        if !INTERACTIVE_ROLES.contains(&role) {
            continue;
        }

        results.nodes_checked += 1;

        // 1. Empty accessible name
        if !node.has_name() {
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::High,
                format!(
                    "Interactive element with role '{}' has no accessible name",
                    role
                ),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix("Add aria-label, aria-labelledby, or visible text content")
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
            continue;
        }

        // 2. Icon-only control (name from Attribute but single non-alphanumeric char)
        let name_str = node.name.as_deref().unwrap_or("");
        if matches!(node.name_source, Some(NameSource::Attribute)) && is_icon_only_name(name_str) {
            let violation = Violation::new(
                RULE_META.id,
                "Accessible Name - Icon Only",
                RULE_META.level,
                Severity::Medium,
                format!(
                    "Interactive element with role '{}' appears to have only an icon/symbol as its accessible name: '{}'",
                    role, name_str
                ),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix("Provide a descriptive accessible name using aria-label or visible text")
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
            continue;
        }

        // 3. Name/Description conflict (redundant description)
        if let (Some(name), Some(desc)) = (node.name.as_deref(), node.description.as_deref()) {
            if !name.trim().is_empty() && name.trim() == desc.trim() {
                let violation = Violation::new(
                    RULE_META.id,
                    "Accessible Name - Redundant Description",
                    RULE_META.level,
                    Severity::Low,
                    format!(
                        "Element's accessible name and description are identical: '{}'",
                        name.trim()
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix(
                    "The accessible description should provide additional information beyond the name",
                )
                .with_help_url(RULE_META.help_url);

                results.add_violation(violation);
                continue;
            }
        }

        // 4. aria-labelledby present but value is empty
        if let Some(val) = node.get_property_str("aria-labelledby") {
            if val.trim().is_empty() {
                let violation = Violation::new(
                    RULE_META.id,
                    "Accessible Name - Empty aria-labelledby",
                    RULE_META.level,
                    Severity::High,
                    "Element has an empty aria-labelledby attribute",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix("Provide a valid ID reference in aria-labelledby")
                .with_help_url(RULE_META.help_url);

                results.add_violation(violation);
                continue;
            }
        }

        // 5. aria-describedby present but value is empty
        if let Some(val) = node.get_property_str("aria-describedby") {
            if val.trim().is_empty() {
                let violation = Violation::new(
                    RULE_META.id,
                    "Accessible Name - Empty aria-describedby",
                    RULE_META.level,
                    Severity::Medium,
                    "Element has an empty aria-describedby attribute",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix(
                    "Provide a valid ID reference in aria-describedby or remove the attribute",
                )
                .with_help_url(RULE_META.help_url);

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
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue, NameSource};

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
    fn test_button_with_name_passes() {
        let nodes = vec![make_node("1", "button", Some("Submit"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_accessible_name(&tree);
        assert_eq!(results.violations.len(), 0);
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_button_without_name_fails() {
        let nodes = vec![make_node("1", "button", None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_accessible_name(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("no accessible name"));
    }

    #[test]
    fn test_icon_only_name_flagged() {
        let mut node = make_node("1", "button", Some("×"));
        node.name_source = Some(NameSource::Attribute);
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_accessible_name(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("icon/symbol")));
    }

    #[test]
    fn test_redundant_description_flagged() {
        let mut node = make_node("1", "button", Some("Search"));
        node.description = Some("Search".to_string());
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_accessible_name(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("identical")));
    }

    #[test]
    fn test_empty_aria_labelledby_flagged() {
        let mut node = make_node("1", "textbox", Some("Name"));
        node.properties.push(AXProperty {
            name: "aria-labelledby".to_string(),
            value: AXValue::String(String::new()),
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_accessible_name(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("aria-labelledby")));
    }
}
