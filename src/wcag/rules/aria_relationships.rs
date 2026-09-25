//! WCAG 4.1.2 - ARIA Relationship Attributes
//!
//! Checks for empty ARIA relationship attributes, which break programmatic
//! relationships between elements.
//!
//! This file used to also detect duplicate `id` attributes via
//! `node.get_property_str("id")` — dead code, since CDP never exposes a
//! generic `id` AX property, and duplicate-ID detection already has a
//! working, dedicated implementation in `parsing.rs` (axe ids `duplicate-id`
//! / `duplicate-id-aria`). Removed as part of #QA-009's cleanup.
//!
//! The empty-relationship check itself was also dead code (#QA-030): it read
//! CDP AX properties by their `aria-`-prefixed HTML attribute names, but CDP
//! exposes them unprefixed (`controls`, `owns`, `activedescendant`), and via
//! `get_property_str` which returns `None` for the `AXValue::Node` values real
//! relationship properties carry. Fixed to read the correct property names
//! via `get_property_idrefs`, which handles both value shapes.
//!
//! `check_aria_relationships` (the AX-tree check above) still misses the
//! case it was written for: Chrome's `Accessibility.getFullAXTree` exposes
//! no `controls`/`owns`/`activedescendant` AX property at all when the
//! attribute value is empty or references a non-existent id (checked live,
//! #567) — `node.has_property(prop_name)` is always `false` for exactly
//! those cases. `check_aria_relationships_with_page` below is a DOM-based
//! supplement (same `_with_page` pattern as `on_focus.rs`/`on_input.rs`)
//! that reads the raw attribute values directly and resolves each id
//! against `document.getElementById`, closing that gap. The AX-tree check
//! stays as-is: it correctly catches a different, real shape (a
//! relationship property present with an empty `related_nodes: []`) that
//! the DOM check's id-existence test wouldn't flag (the DOM element the AX
//! tree pruned still exists in the DOM).

use chromiumoxide::Page;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{evaluate_or_fail, RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA relationship attribute checks
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Relationships",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "ARIA relationship attributes must reference valid, non-empty targets",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "aria-valid-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// ARIA relationship attributes that must not be empty when present.
/// (CDP AX property name, HTML attribute name for messages)
const EMPTY_RELATIONSHIP_ATTRS: &[(&str, &str)] = &[
    ("controls", "aria-controls"),
    ("owns", "aria-owns"),
    ("activedescendant", "aria-activedescendant"),
];

/// Check ARIA relationship attributes for empty values
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for empty ARIA relationships
pub fn check_aria_relationships(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        // Check each relationship attribute for empty values
        for (prop_name, attr_name) in EMPTY_RELATIONSHIP_ATTRS {
            if !node.has_property(prop_name) {
                continue;
            }
            if node.get_property_idrefs(prop_name).is_empty() {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    Severity::Medium,
                    format!("{} references a target but the value is empty", attr_name),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix(format!(
                    "Either provide a valid ID reference for {} or remove the attribute",
                    attr_name
                ))
                .with_rule_id(RULE_META.axe_id)
                .with_help_url(RULE_META.help_url);

                results.add_violation(violation);
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

const ARIA_RELATIONSHIPS_CAP: usize = 250;

const ARIA_RELATIONSHIPS_BODY: &str = r#"
  var attrs = ['aria-controls', 'aria-owns', 'aria-activedescendant'];
  var selector = attrs.map(function(a) { return '[' + a + ']'; }).join(', ');
  var els = document.querySelectorAll(selector);
  var issues = [];
  for (var i = 0; i < els.length && issues.length < CAP; i++) {
    var el = els[i];
    for (var a = 0; a < attrs.length; a++) {
      var attr = attrs[a];
      if (!el.hasAttribute(attr)) continue;
      var raw = (el.getAttribute(attr) || '').trim();
      var kind = null;
      if (!raw) {
        kind = 'empty';
      } else {
        var broken = raw.split(/\s+/).some(function(id) { return !document.getElementById(id); });
        if (broken) kind = 'broken';
      }
      if (kind) {
        issues.push({ attr: attr, kind: kind, selector: __amsCssSelector(el) });
      }
    }
  }
  return issues;
"#;

/// DOM supplement for `aria-valid-attr` (see module docs / #567): catches
/// empty or dangling `aria-controls`/`aria-owns`/`aria-activedescendant`
/// values, which the AX-tree-based `check_aria_relationships` above cannot
/// see because Chrome omits the AX property entirely for these cases.
pub async fn check_aria_relationships_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &ARIA_RELATIONSHIPS_BODY.replace("CAP", &ARIA_RELATIONSHIPS_CAP.to_string()),
        "})()",
    ]
    .concat();

    let items = match evaluate_or_fail(page, &RULE_META, js.as_str()).await {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let Some(items) = items.as_array() else {
        return vec![];
    };

    items
        .iter()
        .filter_map(|item| {
            let attr = item.get("attr")?.as_str()?;
            let kind = item.get("kind")?.as_str()?;
            let selector = item.get("selector")?.as_str()?.to_string();

            let message = if kind == "empty" {
                format!("{} references a target but the value is empty", attr)
            } else {
                format!(
                    "{} references a target that does not exist in the page",
                    attr
                )
            };

            Some(
                Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    RULE_META.severity,
                    message,
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix(format!(
                    "Either provide a valid ID reference for {} or remove the attribute",
                    attr
                ))
                .with_rule_id(RULE_META.axe_id)
                .with_help_url(RULE_META.help_url),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn make_node(id: &str, role: &str) -> AXNode {
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
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    fn make_node_with_prop(id: &str, role: &str, prop_name: &str, prop_value: &str) -> AXNode {
        let mut node = make_node(id, role);
        node.properties.push(AXProperty {
            name: prop_name.to_string(),
            value: AXValue::String(prop_value.to_string()),
        });
        node
    }

    #[test]
    fn test_empty_aria_controls_flagged() {
        let node = make_node_with_prop("1", "button", "controls", "");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-controls")),
            "Empty aria-controls should be flagged"
        );
    }

    #[test]
    fn test_empty_aria_owns_flagged() {
        let node = make_node_with_prop("1", "combobox", "owns", "");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-owns")),
            "Empty aria-owns should be flagged"
        );
    }

    #[test]
    fn test_empty_aria_activedescendant_flagged() {
        let node = make_node_with_prop("1", "listbox", "activedescendant", "");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-activedescendant")),
            "Empty aria-activedescendant should be flagged"
        );
    }

    #[test]
    fn test_valid_aria_controls_passes() {
        let node = make_node_with_prop("1", "button", "controls", "my-panel");
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .all(|v| !v.message.contains("aria-controls")),
            "Valid aria-controls should not be flagged"
        );
    }

    #[test]
    fn test_empty_node_relationship_flagged() {
        // Real CDP traffic carries relationship properties as AXValue::Node,
        // not AXValue::String — an empty related_nodes list must still be
        // flagged (#QA-030 regression check for the get_property_str bug).
        let mut node = make_node("1", "button");
        node.properties.push(AXProperty {
            name: "controls".to_string(),
            value: AXValue::Node {
                related_nodes: vec![],
            },
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.message.contains("aria-controls")),
            "Empty related_nodes (AXValue::Node shape) should be flagged"
        );
    }

    #[test]
    fn test_valid_node_relationship_passes() {
        let mut node = make_node("1", "button");
        node.properties.push(AXProperty {
            name: "controls".to_string(),
            value: AXValue::Node {
                related_nodes: vec![crate::accessibility::RelatedNode {
                    backend_dom_node_id: None,
                    idref: Some("panel-1".to_string()),
                    text: None,
                }],
            },
        });
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_relationships(&tree);
        assert!(
            results
                .violations
                .iter()
                .all(|v| !v.message.contains("aria-controls")),
            "Non-empty related_nodes should not be flagged"
        );
    }
}
