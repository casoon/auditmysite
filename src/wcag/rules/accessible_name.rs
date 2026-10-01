//! WCAG 4.1.2 - Redundant Accessible Description
//!
//! Flags interactive elements whose accessible description repeats their
//! accessible name (#713).
//!
//! Die übrigen Namensprüfungen dieser Datei -- kein Name, nur ein Symbol --
//! laufen seit #692 als `names/required-missing` und `names/symbol-only` im
//! geteilten Bestand (siehe `wcag::shared`). Hier bleibt die doppelte
//! Beschreibung: `a11y_dom::Semantics` liefert keine Accessible Description.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Beschreibung wiederholt den Namen (#713): eigene Best-Practice-Regel statt
/// `aria-label`, weil das Element einen Namen *hat* -- die doppelte Ansage ist
/// redundant, aber kein fehlender Name. Verankert an 4.1.2, dem Kriterium fuer
/// Name und Beschreibung, ohne es zu verletzen.
pub(super) const DESCRIPTION_DUPLICATES_NAME_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Accessible Name - Redundant Description",
    level: WcagLevel::A,
    severity: Severity::Low,
    description: "An accessible description should add information beyond the accessible name",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "description-duplicates-name",
    tags: &["best-practice", "wcag412", "cat.aria"],
};

/// Roles considered interactive
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

/// Check interactive elements for a description that repeats the name
pub fn check_accessible_name(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
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

        if let (Some(name), Some(desc)) = (node.name.as_deref(), node.description.as_deref()) {
            if !name.trim().is_empty() && name.trim() == desc.trim() {
                let violation = Violation::new(
                    DESCRIPTION_DUPLICATES_NAME_META.id,
                    DESCRIPTION_DUPLICATES_NAME_META.name,
                    DESCRIPTION_DUPLICATES_NAME_META.level,
                    DESCRIPTION_DUPLICATES_NAME_META.severity,
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
                .with_help_url(DESCRIPTION_DUPLICATES_NAME_META.help_url)
                .with_rule_id(DESCRIPTION_DUPLICATES_NAME_META.axe_id)
                .with_tags(
                    DESCRIPTION_DUPLICATES_NAME_META
                        .tags
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                );

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

    /// Ein fehlender Name ist `names/required-missing` bzw.
    /// `buttons/name-missing` im geteilten Bestand (#692), nicht diese Regel.
    #[test]
    fn test_button_without_name_is_not_reported_here() {
        let tree = AXTree::from_nodes(vec![make_node("1", "button", None)]);
        let results = check_accessible_name(&tree);
        assert!(results.violations.is_empty());
        assert!(results.warnings.is_empty());
    }

    #[test]
    fn test_redundant_description_flagged() {
        let mut node = make_node("1", "button", Some("Search"));
        node.description = Some("Search".to_string());
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_accessible_name(&tree);
        let redundant: Vec<_> = results
            .violations
            .iter()
            .filter(|v| v.message.contains("identical"))
            .collect();
        assert_eq!(redundant.len(), 1);
        // Eigene Kennung, nicht `aria-label` (#713): das Element hat einen Namen.
        assert_eq!(
            redundant[0].rule_id.as_deref(),
            Some("description-duplicates-name")
        );
        assert!(redundant[0].tags.iter().any(|t| t == "best-practice"));
        assert_eq!(results.violations.len(), 1);
    }
}
