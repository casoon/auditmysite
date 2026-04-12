//! WCAG 4.1.2 - ARIA Role Validity
//!
//! Validates that ARIA roles are valid, that required owned elements are present,
//! and that elements appear in the correct parent context.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA role validity
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Role Validity",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA roles must be valid, and elements must appear in the required context",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-roles",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// All valid ARIA roles per the ARIA 1.2 specification
const VALID_ARIA_ROLES: &[&str] = &[
    // Landmark roles
    "banner",
    "complementary",
    "contentinfo",
    "form",
    "main",
    "navigation",
    "region",
    "search",
    // Document structure
    "article",
    "blockquote",
    "caption",
    "cell",
    "columnheader",
    "definition",
    "code",
    "mark",
    "deletion",
    "directory",
    "document",
    "emphasis",
    "feed",
    "figure",
    "generic",
    "group",
    "heading",
    "img",
    "image",
    "insertion",
    "list",
    "listitem",
    "log",
    "marquee",
    "math",
    "meter",
    "none",
    "note",
    "paragraph",
    "presentation",
    "row",
    "rowgroup",
    "rowheader",
    "scrollbar",
    "separator",
    "status",
    "strong",
    "subscript",
    "superscript",
    "table",
    "term",
    "time",
    "toolbar",
    "tooltip",
    // Widget roles
    "button",
    "checkbox",
    "combobox",
    "grid",
    "gridcell",
    "link",
    "listbox",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "progressbar",
    "radio",
    "radiogroup",
    "slider",
    "spinbutton",
    "switch",
    "tab",
    "tablist",
    "tabpanel",
    "textbox",
    "tree",
    "treegrid",
    "treeitem",
    // Live region roles
    "alert",
    "alertdialog",
    "dialog",
    // Additional common roles
    "application",
    "searchbox",
    "spinbutton",
    "term",
    // Browser-native roles that appear in AX trees
    "WebArea",
    "RootWebArea",
    "Iframe",
    "StaticText",
    "InlineTextBox",
    "LineBreak",
    "SVGRoot",
    "SvgRoot",
    "Canvas",
    "EmbeddedObject",
    "LayoutTable",
    "LayoutTableRow",
    "LayoutTableCell",
    "Unknown",
    // Browser-internal implicit roles (not author errors)
    "ListMarker",    // ::marker pseudo-elements
    "sectionheader", // implicit role of <header>
    "sectionfooter", // implicit role of <footer>
];

/// Valid `aria-*` attribute names per the ARIA 1.2 specification
const VALID_ARIA_ATTRIBUTES: &[&str] = &[
    "aria-activedescendant",
    "aria-atomic",
    "aria-autocomplete",
    "aria-braillelabel",
    "aria-brailleroledescription",
    "aria-busy",
    "aria-checked",
    "aria-colcount",
    "aria-colindex",
    "aria-colindextext",
    "aria-colspan",
    "aria-controls",
    "aria-current",
    "aria-describedby",
    "aria-description",
    "aria-details",
    "aria-disabled",
    "aria-dropeffect",
    "aria-errormessage",
    "aria-expanded",
    "aria-flowto",
    "aria-grabbed",
    "aria-haspopup",
    "aria-hidden",
    "aria-invalid",
    "aria-keyshortcuts",
    "aria-label",
    "aria-labelledby",
    "aria-level",
    "aria-live",
    "aria-modal",
    "aria-multiline",
    "aria-multiselectable",
    "aria-orientation",
    "aria-owns",
    "aria-placeholder",
    "aria-posinset",
    "aria-pressed",
    "aria-readonly",
    "aria-relevant",
    "aria-required",
    "aria-roledescription",
    "aria-rowcount",
    "aria-rowindex",
    "aria-rowindextext",
    "aria-rowspan",
    "aria-selected",
    "aria-setsize",
    "aria-sort",
    "aria-valuemax",
    "aria-valuemin",
    "aria-valuenow",
    "aria-valuetext",
];

/// Roles that require specific child roles to be present
/// (role, required_child_roles)
const REQUIRED_OWNED_ELEMENTS: &[(&str, &[&str])] = &[
    ("list", &["listitem"]),
    ("table", &["row", "rowgroup"]),
    ("grid", &["row", "rowgroup"]),
    ("treegrid", &["row", "rowgroup"]),
    ("menu", &["menuitem", "menuitemcheckbox", "menuitemradio"]),
    (
        "menubar",
        &["menuitem", "menuitemcheckbox", "menuitemradio"],
    ),
    ("tree", &["treeitem"]),
    ("listbox", &["option"]),
    ("radiogroup", &["radio"]),
    ("tablist", &["tab"]),
];

/// Roles that require a specific parent role
/// (role, required_parent_roles)
const REQUIRED_CONTEXT: &[(&str, &[&str])] = &[
    ("listitem", &["list", "group"]),
    ("option", &["listbox", "combobox"]),
    ("menuitem", &["menu", "menubar", "group"]),
    ("menuitemcheckbox", &["menu", "menubar", "group"]),
    ("menuitemradio", &["menu", "menubar", "group", "radiogroup"]),
    ("tab", &["tablist"]),
    ("treeitem", &["tree", "group"]),
    ("row", &["table", "grid", "treegrid", "rowgroup"]),
    ("cell", &["row"]),
    ("columnheader", &["row"]),
    ("rowheader", &["row"]),
    ("gridcell", &["row"]),
];

/// Check ARIA role validity across the accessibility tree
pub fn check_aria_roles(tree: &AXTree) -> WcagResults {
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

        // Check for invalid roles
        check_invalid_role(node, role, &mut results);

        // Check for invalid aria-* attributes
        check_invalid_aria_attributes(node, &mut results);

        // Check for missing required owned elements
        check_required_owned_elements(node, role, tree, &mut results);

        // Check for missing required context (parent)
        check_required_context(node, role, tree, &mut results);
    }

    results
}

fn check_invalid_role(node: &crate::accessibility::AXNode, role: &str, results: &mut WcagResults) {
    if !VALID_ARIA_ROLES.contains(&role) {
        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::High,
            format!("Element has invalid ARIA role: '{}'", role),
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_name(node.name.clone())
        .with_fix("Use a valid ARIA role from the ARIA specification")
        .with_help_url("https://www.w3.org/TR/wai-aria-1.2/#role_definitions");

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

fn check_invalid_aria_attributes(node: &crate::accessibility::AXNode, results: &mut WcagResults) {
    for prop in &node.properties {
        if prop.name.starts_with("aria-") && !VALID_ARIA_ATTRIBUTES.contains(&prop.name.as_str()) {
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::High,
                format!("Element has invalid ARIA attribute: '{}'", prop.name),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix("Use a valid aria-* attribute from the ARIA specification")
            .with_help_url("https://www.w3.org/TR/wai-aria-1.2/#state_prop_def");

            results.add_violation(violation);
        }
    }
}

fn check_required_owned_elements(
    node: &crate::accessibility::AXNode,
    role: &str,
    tree: &AXTree,
    results: &mut WcagResults,
) {
    let required = REQUIRED_OWNED_ELEMENTS.iter().find(|(r, _)| *r == role);

    let (_, required_child_roles) = match required {
        Some(r) => r,
        None => return,
    };

    // Collapsed containers (aria-expanded="false") legitimately have no
    // visible children in the AX tree — don't flag them.
    let is_collapsed = node
        .get_property_str("expanded")
        .map(|v| v == "false")
        .unwrap_or(false)
        || node.get_property_bool("expanded") == Some(false);
    if is_collapsed {
        return;
    }

    // Check if any direct or shallow children have the required role
    let has_required_child = node.child_ids.iter().any(|child_id| {
        tree.nodes
            .get(child_id)
            .and_then(|child| child.role.as_deref())
            .map(|child_role| required_child_roles.contains(&child_role))
            .unwrap_or(false)
    });

    if !has_required_child && !node.child_ids.is_empty() {
        let violation = Violation::new(
            RULE_META.id,
            "ARIA Required Owned Elements",
            RULE_META.level,
            Severity::High,
            format!(
                "Element with role '{}' is missing required child role(s): {}",
                role,
                required_child_roles.join(", ")
            ),
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix(format!(
            "Add child elements with role(s): {}",
            required_child_roles.join(", ")
        ))
        .with_help_url("https://www.w3.org/TR/wai-aria-1.2/#mustContain");

        results.add_violation(violation);
    }
}

fn check_required_context(
    node: &crate::accessibility::AXNode,
    role: &str,
    tree: &AXTree,
    results: &mut WcagResults,
) {
    let required = REQUIRED_CONTEXT.iter().find(|(r, _)| *r == role);

    let (_, required_parent_roles) = match required {
        Some(r) => r,
        None => return,
    };

    // Walk up the tree to find a matching parent role
    let has_valid_context = has_ancestor_with_role(node, required_parent_roles, tree);

    if !has_valid_context {
        let violation = Violation::new(
            RULE_META.id,
            "ARIA Required Context Role",
            RULE_META.level,
            Severity::High,
            format!(
                "Element with role '{}' is not in required parent context: {}",
                role,
                required_parent_roles.join(", ")
            ),
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix(format!(
            "Place this element inside a parent with role: {}",
            required_parent_roles.join(", ")
        ))
        .with_help_url("https://www.w3.org/TR/wai-aria-1.2/#scope");

        results.add_violation(violation);
    }
}

/// Walk up the tree to check if any ancestor has one of the specified roles
fn has_ancestor_with_role(
    node: &crate::accessibility::AXNode,
    roles: &[&str],
    tree: &AXTree,
) -> bool {
    let mut current_parent_id = node.parent_id.as_deref();

    while let Some(parent_id) = current_parent_id {
        if let Some(parent) = tree.nodes.get(parent_id) {
            if let Some(parent_role) = parent.role.as_deref() {
                if roles.contains(&parent_role) {
                    return true;
                }
            }
            current_parent_id = parent.parent_id.as_deref();
        } else {
            break;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn make_node(id: &str, role: &str, parent_id: Option<&str>, child_ids: Vec<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: Some(format!("Node {}", id)),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: child_ids.into_iter().map(String::from).collect(),
            parent_id: parent_id.map(String::from),
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_valid_role_passes() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "button", Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_roles(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_invalid_role_flagged() {
        let nodes = vec![make_node("1", "not-a-real-role", None, vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_roles(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("invalid ARIA role")));
    }

    #[test]
    fn test_list_without_listitem_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "list", Some("1"), vec!["3"]),
            // child has wrong role
            make_node("3", "paragraph", Some("2"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_roles(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("listitem")));
    }

    #[test]
    fn test_collapsed_menu_not_flagged() {
        let mut menu = make_node("2", "menu", Some("1"), vec![]);
        menu.properties.push(AXProperty {
            name: "expanded".to_string(),
            value: AXValue::Bool(false),
        });
        let nodes = vec![make_node("1", "WebArea", None, vec!["2"]), menu];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_roles(&tree);
        assert!(
            !results
                .violations
                .iter()
                .any(|v| v.message.contains("missing required child")),
            "Collapsed menu should not be flagged for missing children"
        );
    }

    #[test]
    fn test_listitem_without_list_parent_flagged() {
        let nodes = vec![
            make_node("1", "WebArea", None, vec!["2"]),
            make_node("2", "listitem", Some("1"), vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_roles(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("required parent context")));
    }

    #[test]
    fn test_invalid_aria_attribute_flagged() {
        let mut node = make_node("1", "button", None, vec![]);
        node.properties.push(AXProperty {
            name: "aria-notavalidattr".to_string(),
            value: AXValue::String("true".to_string()),
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_roles(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("invalid ARIA attribute")));
    }
}
