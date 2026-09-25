//! WCAG 4.1.2 Name, Role, Value -- mehrdeutige ARIA-Beziehungen
//!
//! Beanspruchen mehrere Knoten per `aria-owns` dasselbe Kind, ist die
//! Beziehung nicht mehr eindeutig aufloesbar: Assistierende Technik kann Name,
//! Rolle und Wert des betroffenen Elements nicht zuverlaessig bestimmen.
//! Level A
//!
//! Die Pruefung auf doppelte IDs laeuft seit der Umstellung auf die geteilten
//! Crates als `ids/duplicate` im `a11y-rules`-Bestand (siehe `wcag::shared`)
//! -- die vormalige DOM-Regel mit der axe-Kennung `duplicate-id` ist deshalb
//! geloescht.
//!
//! Hier bleibt die AX-baumbasierte Pruefung `duplicate-id-aria`. Das ist ein
//! anderer Befund als eine doppelte ID -- er bricht die Aufloesung von
//! ARIA-Beziehungen und hat im geteilten Bestand kein Gegenstueck.

use std::collections::HashMap;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

// WCAG 4.1.1 (Parsing) was removed in WCAG 2.2 (Oct 2023) — it "always passes"
// for HTML since browsers recover from malformed markup. An ambiguous
// aria-owns relationship stays a real failure, but of 4.1.2: AT cannot resolve
// which element the relationship names, so name, role and value are no longer
// programmatically determinable. That is also where axe-core maps its
// `duplicate-id-aria` rule (plan 54 §2). It does not cause complete
// inaccessibility, so this is High, not Critical.
pub const PARSING_RULE: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Name, Role, Value",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA relationships must resolve to exactly one element; a duplicate ID makes aria-owns/aria-labelledby ambiguous (WCAG 4.1.2)",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "duplicate-id-aria",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

pub fn check_parsing(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Collect aria-owns targets: target_id → list of owning node IDs
    let mut owners: HashMap<String, Vec<String>> = HashMap::new();
    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        for target_id in node.get_property_idrefs("owns") {
            owners
                .entry(target_id.to_string())
                .or_default()
                .push(node.node_id.clone());
        }
    }

    for (target_id, owning_nodes) in &owners {
        if owning_nodes.len() > 1 {
            results.add_violation(
                Violation::new(
                    PARSING_RULE.id,
                    PARSING_RULE.name,
                    PARSING_RULE.level,
                    PARSING_RULE.severity,
                    format!(
                        "aria-owns target '{}' is claimed by {} nodes — likely caused by duplicate ID in the DOM",
                        target_id,
                        owning_nodes.len()
                    ),
                    owning_nodes[0].clone(),
                )
                .with_fix(format!(
                    "Ensure the element with id='{}' is unique in the DOM. Each element may only be owned by one aria-owns attribute.",
                    target_id
                ))
                .with_rule_id(PARSING_RULE.axe_id)
                .with_help_url(PARSING_RULE.help_url),
            );
        } else {
            results.passes += 1;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_aria_owns(id: &str, role: &str, owns: &str) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "owns".to_string(),
                value: AXValue::String(owns.to_string()),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_no_duplicate_owns_passes() {
        let nodes = vec![
            node_with_aria_owns("1", "listbox", "opt1 opt2"),
            node_with_aria_owns("2", "listbox", "opt3 opt4"),
        ];
        let tree = AXTree::from_nodes(nodes);
        let r = check_parsing(&tree);
        assert_eq!(r.violations.len(), 0);
    }

    #[test]
    fn test_duplicate_aria_owns_target_flagged() {
        // Two nodes both claiming ownership of "dropdown-list"
        let nodes = vec![
            node_with_aria_owns("1", "combobox", "dropdown-list"),
            node_with_aria_owns("2", "combobox", "dropdown-list"),
        ];
        let tree = AXTree::from_nodes(nodes);
        let r = check_parsing(&tree);
        assert_eq!(r.violations.len(), 1);
        assert!(r.violations[0].message.contains("dropdown-list"));
        assert!(r.violations[0].message.contains("2 nodes"));
    }

    #[test]
    fn test_unique_owns_targets_pass() {
        let nodes = vec![
            node_with_aria_owns("1", "combobox", "list-a"),
            node_with_aria_owns("2", "combobox", "list-b"),
        ];
        let tree = AXTree::from_nodes(nodes);
        let r = check_parsing(&tree);
        assert_eq!(r.violations.len(), 0);
    }

    #[test]
    fn test_multiple_targets_in_single_owns() {
        // One node owns two targets — both unique, should pass
        let nodes = vec![node_with_aria_owns("1", "group", "child-a child-b")];
        let tree = AXTree::from_nodes(nodes);
        let r = check_parsing(&tree);
        assert_eq!(r.violations.len(), 0);
    }
}
