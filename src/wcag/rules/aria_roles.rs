//! WCAG 4.1.2 - ARIA Role Validity
//!
//! Validates that ARIA roles are valid and that required owned elements are
//! present. Required-parent-context validation lives in
//! `aria_required_parent.rs` to avoid double-reporting the same violation.
//!
//! Invalid-role detection (`check_invalid_role_with_page`) is a DOM-level
//! rule: Chrome repairs an invalid explicit `role="..."` attribute before
//! exposing the element in the AX tree (falling back to the native/implicit
//! role or `generic`), so a tree-based check can never observe the
//! author's mistake — an earlier implementation only ever false-positived on
//! Chrome-internal AX roles missing from its allowlist (#QA-030).

use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA role validity
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Role Validity",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA roles must be valid and required owned elements must be present",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-roles",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

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

        // Invalid-role detection now runs as a DOM page rule
        // (check_invalid_role_with_page) — Chrome repairs invalid roles
        // before AX extraction, so this tree-based check could only ever
        // false-positive on Chrome-internal roles (#QA-030).

        // Invalid-aria-attribute-name detection now runs as a DOM page rule
        // (check_invalid_aria_attribute_name_with_page) — CDP never exposes
        // aria-*-prefixed property names, so this was dead code (#QA-030).

        // Check for missing required owned elements
        check_required_owned_elements(node, role, tree, &mut results);

        // Required-parent-context is checked exclusively by
        // aria_required_parent.rs — this file used to duplicate it via
        // `check_required_context`, causing every violation to be reported
        // twice (#QA-033).
    }

    results
}

const INVALID_ROLE_CAP: usize = 250;

/// Real ARIA 1.2 role list (abstract roles excluded — authors must not use
/// them). Deliberately does NOT include Chrome-internal AX roles
/// (WebArea, LayoutTable, StaticText, …) since those never appear as an
/// author-set `role="..."` attribute value in the DOM.
const VALID_DOM_ARIA_ROLES_JS: &str = r#"
  var validRoles = [
    'alert','alertdialog','application','article','banner','blockquote','button',
    'caption','cell','checkbox','code','columnheader','combobox','complementary',
    'contentinfo','definition','deletion','dialog','directory','document','emphasis',
    'feed','figure','form','generic','grid','gridcell','group','heading','img',
    'insertion','link','list','listbox','listitem','log','main','mark','marquee',
    'math','menu','menubar','menuitem','menuitemcheckbox','menuitemradio','meter',
    'navigation','none','note','option','paragraph','presentation','progressbar',
    'radio','radiogroup','region','row','rowgroup','rowheader','scrollbar','search',
    'searchbox','separator','slider','spinbutton','status','strong','subscript',
    'superscript','switch','tab','table','tablist','tabpanel','term','textbox',
    'time','timer','toolbar','tooltip','tree','treegrid','treeitem'
  ];
"#;

const INVALID_ROLE_BODY: &str = r#"
  var issues = [];
  var elems = document.querySelectorAll('[role]');
  for (var i = 0; i < elems.length && issues.length < CAP; i++) {
    var el = elems[i];
    var roles = (el.getAttribute('role') || '').trim().toLowerCase().split(/\s+/).filter(Boolean);
    for (var r = 0; r < roles.length; r++) {
      if (validRoles.indexOf(roles[r]) === -1) {
        issues.push({ role: roles[r], selector: __amsCssSelector(el) });
        break;
      }
    }
  }
  return { issues: issues };
"#;

/// Check that explicit `role="..."` attribute values are valid ARIA roles.
pub async fn check_invalid_role_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        VALID_DOM_ARIA_ROLES_JS,
        &INVALID_ROLE_BODY.replace("CAP", &INVALID_ROLE_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("invalid-role JS failed: {}", e);
            return vec![];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let role = issue.get("role")?.as_str()?;
            let selector = issue.get("selector")?.as_str()?.to_string();

            Some(
                Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    Severity::High,
                    format!("Element has invalid ARIA role: '{}'", role),
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix("Use a valid ARIA role from the ARIA specification")
                .with_rule_id(RULE_META.axe_id)
                .with_help_url("https://www.w3.org/TR/wai-aria-1.2/#role_definitions"),
            )
        })
        .collect()
}

/// Distinct from `RULE_META.axe_id` ("aria-roles", owned by `check_invalid_role`) —
/// this check validates ARIA attribute *names* (misspelled/non-existent
/// aria-* attributes), a different problem from an invalid role value. Not a
/// real axe-core rule id (axe folds this into `aria-valid-attr`, which this
/// codebase's `aria_relationships.rs` already uses for an unrelated check —
/// see #QA-009 remaining scope), so this is a custom, stable identifier.
const INVALID_ARIA_ATTR_NAME_AXE_ID: &str = "aria-attr-name-invalid";

const INVALID_ATTR_NAME_CAP: usize = 250;

const VALID_ARIA_ATTRIBUTES_JS: &str = r#"
  var validAttrs = [
    'aria-activedescendant', 'aria-atomic', 'aria-autocomplete', 'aria-braillelabel',
    'aria-brailleroledescription', 'aria-busy', 'aria-checked', 'aria-colcount',
    'aria-colindex', 'aria-colindextext', 'aria-colspan', 'aria-controls', 'aria-current',
    'aria-describedby', 'aria-description', 'aria-details', 'aria-disabled',
    'aria-dropeffect', 'aria-errormessage', 'aria-expanded', 'aria-flowto', 'aria-grabbed',
    'aria-haspopup', 'aria-hidden', 'aria-invalid', 'aria-keyshortcuts', 'aria-label',
    'aria-labelledby', 'aria-level', 'aria-live', 'aria-modal', 'aria-multiline',
    'aria-multiselectable', 'aria-orientation', 'aria-owns', 'aria-placeholder',
    'aria-posinset', 'aria-pressed', 'aria-readonly', 'aria-relevant', 'aria-required',
    'aria-roledescription', 'aria-rowcount', 'aria-rowindex', 'aria-rowindextext',
    'aria-rowspan', 'aria-selected', 'aria-setsize', 'aria-sort', 'aria-valuemax',
    'aria-valuemin', 'aria-valuenow', 'aria-valuetext'
  ];
"#;

const INVALID_ATTR_NAME_BODY: &str = r#"
  var issues = [];
  var elems = document.querySelectorAll('*');
  for (var i = 0; i < elems.length && issues.length < CAP; i++) {
    var el = elems[i];
    var attrs = el.attributes;
    for (var a = 0; a < attrs.length; a++) {
      var name = attrs[a].name;
      if (name.indexOf('aria-') !== 0) continue;
      if (validAttrs.indexOf(name) === -1) {
        issues.push({ attr: name, selector: __amsCssSelector(el) });
        if (issues.length >= CAP) break;
      }
    }
  }
  return { issues: issues };
"#;

/// Check that `aria-*` attribute names are recognized ARIA 1.2 attributes
/// (catches misspellings/non-existent attributes). DOM-level: CDP never
/// exposes `aria-`-prefixed AX property names, so a tree-based check here
/// was dead code (#QA-030).
pub async fn check_invalid_aria_attribute_name_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        VALID_ARIA_ATTRIBUTES_JS,
        &INVALID_ATTR_NAME_BODY.replace("CAP", &INVALID_ATTR_NAME_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("invalid-aria-attribute-name JS failed: {}", e);
            return vec![];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let attr = issue.get("attr")?.as_str()?;
            let selector = issue.get("selector")?.as_str()?.to_string();

            Some(
                Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    Severity::High,
                    format!("Element has invalid ARIA attribute: '{}'", attr),
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix("Use a valid aria-* attribute from the ARIA specification")
                .with_rule_id(INVALID_ARIA_ATTR_NAME_AXE_ID)
                .with_help_url("https://www.w3.org/TR/wai-aria-1.2/#state_prop_def"),
            )
        })
        .collect()
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
                "Element with role '{}' is missing required child {}: {}",
                role,
                if required_child_roles.len() == 1 {
                    "role"
                } else {
                    "roles"
                },
                required_child_roles.join(", ")
            ),
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix(format!(
            "Add child elements with {}: {}",
            if required_child_roles.len() == 1 {
                "role"
            } else {
                "roles"
            },
            required_child_roles.join(", ")
        ))
        .with_help_url("https://www.w3.org/TR/wai-aria-1.2/#mustContain");

        results.add_violation(violation);
    }
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

    // Invalid-role detection moved to check_invalid_role_with_page (DOM-based,
    // #QA-030) — not unit-tested here since it needs a live Page.

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

    // Invalid-ARIA-attribute-name detection moved to
    // check_invalid_aria_attribute_name_with_page (DOM-based, #QA-030) —
    // needs a live Page, not unit-tested here.
}
