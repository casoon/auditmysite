//! WCAG 4.1.2 - ARIA Allowed Attributes
//!
//! Validates that ARIA attributes used on elements are allowed for their role.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA allowed attributes
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Allowed Attributes",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA attributes must be allowed for the element's role",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-allowed-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Global ARIA attributes allowed on ALL roles
const GLOBAL_ARIA_ATTRS: &[&str] = &[
    "aria-atomic",
    "aria-busy",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-live",
    "aria-owns",
    "aria-relevant",
    "aria-roledescription",
];

/// Role-specific allowed ARIA attributes (beyond globals)
const ROLE_SPECIFIC_ATTRS: &[(&str, &[&str])] = &[
    ("button", &["aria-expanded", "aria-pressed"]),
    ("link", &["aria-expanded"]),
    (
        "checkbox",
        &["aria-checked", "aria-readonly", "aria-required"],
    ),
    ("radio", &["aria-checked", "aria-posinset", "aria-setsize"]),
    (
        "textbox",
        &[
            "aria-activedescendant",
            "aria-autocomplete",
            "aria-multiline",
            "aria-placeholder",
            "aria-readonly",
            "aria-required",
        ],
    ),
    (
        "combobox",
        &[
            "aria-activedescendant",
            "aria-autocomplete",
            "aria-expanded",
            "aria-required",
        ],
    ),
    (
        "listbox",
        &[
            "aria-activedescendant",
            "aria-expanded",
            "aria-multiselectable",
            "aria-orientation",
            "aria-readonly",
            "aria-required",
        ],
    ),
    (
        "slider",
        &[
            "aria-orientation",
            "aria-readonly",
            "aria-valuemax",
            "aria-valuemin",
            "aria-valuenow",
            "aria-valuetext",
        ],
    ),
    (
        "tab",
        &[
            "aria-expanded",
            "aria-posinset",
            "aria-selected",
            "aria-setsize",
        ],
    ),
    ("tabpanel", &[]),
    ("dialog", &["aria-modal"]),
    ("alert", &[]),
    ("img", &[]),
    ("heading", &["aria-level"]),
    ("list", &[]),
    ("listitem", &["aria-level", "aria-posinset", "aria-setsize"]),
    ("navigation", &[]),
    ("main", &[]),
    ("banner", &[]),
    ("contentinfo", &[]),
    ("complementary", &[]),
    ("form", &[]),
    ("search", &[]),
    ("menu", &["aria-activedescendant", "aria-orientation"]),
    (
        "menuitem",
        &["aria-expanded", "aria-posinset", "aria-setsize"],
    ),
    (
        "menuitemcheckbox",
        &["aria-checked", "aria-posinset", "aria-setsize"],
    ),
    (
        "menuitemradio",
        &["aria-checked", "aria-posinset", "aria-setsize"],
    ),
    (
        "tree",
        &[
            "aria-activedescendant",
            "aria-multiselectable",
            "aria-orientation",
            "aria-required",
        ],
    ),
    (
        "treeitem",
        &[
            "aria-checked",
            "aria-expanded",
            "aria-level",
            "aria-posinset",
            "aria-selected",
            "aria-setsize",
        ],
    ),
    (
        "grid",
        &[
            "aria-activedescendant",
            "aria-colcount",
            "aria-multiselectable",
            "aria-readonly",
            "aria-rowcount",
        ],
    ),
    (
        "gridcell",
        &[
            "aria-colindex",
            "aria-colspan",
            "aria-expanded",
            "aria-readonly",
            "aria-required",
            "aria-rowindex",
            "aria-rowspan",
            "aria-selected",
        ],
    ),
    (
        "row",
        &[
            "aria-activedescendant",
            "aria-colindex",
            "aria-expanded",
            "aria-level",
            "aria-posinset",
            "aria-rowindex",
            "aria-selected",
            "aria-setsize",
        ],
    ),
    (
        "columnheader",
        &[
            "aria-colindex",
            "aria-colspan",
            "aria-expanded",
            "aria-readonly",
            "aria-required",
            "aria-rowindex",
            "aria-rowspan",
            "aria-selected",
            "aria-sort",
        ],
    ),
    (
        "rowheader",
        &[
            "aria-colindex",
            "aria-colspan",
            "aria-expanded",
            "aria-readonly",
            "aria-required",
            "aria-rowindex",
            "aria-rowspan",
            "aria-selected",
            "aria-sort",
        ],
    ),
    (
        "progressbar",
        &[
            "aria-valuemax",
            "aria-valuemin",
            "aria-valuenow",
            "aria-valuetext",
        ],
    ),
    (
        "scrollbar",
        &[
            "aria-orientation",
            "aria-valuemax",
            "aria-valuemin",
            "aria-valuenow",
        ],
    ),
    (
        "spinbutton",
        &[
            "aria-readonly",
            "aria-required",
            "aria-valuemax",
            "aria-valuemin",
            "aria-valuenow",
            "aria-valuetext",
        ],
    ),
    (
        "switch",
        &["aria-checked", "aria-readonly", "aria-required"],
    ),
    (
        "separator",
        &[
            "aria-orientation",
            "aria-valuemax",
            "aria-valuemin",
            "aria-valuenow",
            "aria-valuetext",
        ],
    ),
    ("toolbar", &["aria-activedescendant", "aria-orientation"]),
];

/// Check that ARIA attributes are allowed for each element's role
pub fn check_aria_allowed_attr(tree: &AXTree) -> WcagResults {
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

        // Only check roles we have a mapping for
        let role_specific = ROLE_SPECIFIC_ATTRS.iter().find(|(r, _)| *r == role);
        let role_specific = match role_specific {
            Some((_, attrs)) => attrs,
            None => continue, // Unknown/unmapped role, skip
        };

        for prop in &node.properties {
            if !prop.name.starts_with("aria-") {
                continue;
            }

            let attr_name = prop.name.as_str();

            // Global attrs are always allowed
            if GLOBAL_ARIA_ATTRS.contains(&attr_name) {
                continue;
            }

            // Check role-specific attrs
            if !role_specific.contains(&attr_name) {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    RULE_META.severity,
                    format!(
                        "ARIA attribute '{}' is not allowed on role '{}'",
                        attr_name, role
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_rule_id(RULE_META.axe_id)
                .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
                .with_fix(format!(
                    "Remove '{}' from this element or change its role",
                    attr_name
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
    fn test_allowed_attr_passes() {
        let nodes = vec![
            make_node("1", "button", vec![("aria-expanded", "true")]),
            make_node("2", "checkbox", vec![("aria-checked", "true")]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_allowed_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_disallowed_attr_flagged() {
        // aria-checked is not allowed on button
        let nodes = vec![make_node("1", "button", vec![("aria-checked", "true")])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_allowed_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("aria-checked"));
    }

    #[test]
    fn test_global_attrs_always_allowed() {
        let nodes = vec![make_node(
            "1",
            "button",
            vec![("aria-label", "Click me"), ("aria-describedby", "desc1")],
        )];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_allowed_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
