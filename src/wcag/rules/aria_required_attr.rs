//! WCAG 4.1.2 - ARIA Required Attributes
//!
//! Validates that elements with certain ARIA roles have the required ARIA attributes.
//!
//! Reads the AX tree's own property presence (CDP exposes these unprefixed,
//! e.g. `expanded`/`valuenow`, not `aria-expanded`/`aria-valuenow`) — an
//! earlier version of this check matched against the prefixed HTML
//! attribute names, which never appear in the AX tree, and so never fired
//! in production except for the heading/`level` special case (#QA-030).
//! This fix restores detection for combobox/meter/scrollbar/separator/
//! slider/spinbutton (their CDP properties reflect real absence).
//!
//! `checkbox`/`radio`/`switch` are handled separately by
//! `check_checked_state_with_page` below: Chrome synthesizes a default
//! `checked: "false"` AX property for these three roles even when the
//! author never set `aria-checked`, so AX-tree presence-checking can't
//! distinguish "not set" from "explicitly false" — this needed a DOM read
//! (`element.hasAttribute('aria-checked')`) instead, confirmed via a live
//! debug print against real Chrome.

use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for ARIA required attributes
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "ARIA Required Attributes",
    level: WcagLevel::A,
    severity: Severity::Critical,
    description: "Roles that require specific ARIA attributes must have them present",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-required-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles and their required ARIA attributes, by CDP AX property name
/// (unprefixed — see module docs). `checkbox`/`radio`/`switch`'s `checked`
/// requirement is NOT listed here — see `check_checked_state_with_page`.
const REQUIRED_ATTRS: &[(&str, &[&str])] = &[
    ("combobox", &["expanded"]),
    ("heading", &["level"]),
    ("meter", &["valuenow"]),
    ("scrollbar", &["controls", "valuenow"]),
    ("separator", &["valuenow"]),
    ("slider", &["valuenow"]),
    ("spinbutton", &["valuenow"]),
];

/// Check that required ARIA attributes are present for each role
pub fn check_aria_required_attr(tree: &AXTree) -> WcagResults {
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

        let required = match REQUIRED_ATTRS.iter().find(|(r, _)| *r == role) {
            Some((_, attrs)) => attrs,
            None => continue,
        };

        // Non-focusable separators (decorative <hr>) do not require aria-valuenow.
        // Per ARIA spec, only focusable splitter-style separators have value requirements.
        if role == "separator" && !node.is_focusable() {
            continue;
        }

        for &req_attr in *required {
            if !node.has_property(req_attr) {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    RULE_META.severity,
                    format!(
                        "Element with role '{}' is missing required attribute 'aria-{}'",
                        role, req_attr
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_rule_id(RULE_META.axe_id)
                .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
                .with_fix(format!("Add 'aria-{}' attribute to this element", req_attr))
                .with_help_url(RULE_META.help_url);

                results.add_violation(violation);
            }
        }
    }

    results
}

const CHECKED_STATE_CAP: usize = 250;

const CHECKED_STATE_BODY: &str = r#"
  var issues = [];
  var elems = document.querySelectorAll(
    'input[type="checkbox"], input[type="radio"], [role="checkbox"], [role="radio"], [role="switch"]'
  );
  for (var i = 0; i < elems.length && issues.length < CAP; i++) {
    var el = elems[i];
    var isNativeCheckable = el.tagName === 'INPUT' &&
      (el.getAttribute('type') === 'checkbox' || el.getAttribute('type') === 'radio');
    if (isNativeCheckable) continue; // native checked state, no aria-checked required
    if (!el.hasAttribute('aria-checked')) {
      var role = (el.getAttribute('role') || '').toLowerCase();
      issues.push({ role: role, selector: __amsCssSelector(el) });
    }
  }
  return { issues: issues };
"#;

/// Check that `role="checkbox"|"radio"|"switch"` elements expose
/// `aria-checked`. Native `<input type=checkbox|radio>` is exempt (the
/// browser maintains its checked state natively). DOM-level: see module docs
/// for why this can't be an AX-tree presence check.
pub async fn check_checked_state_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &CHECKED_STATE_BODY.replace("CAP", &CHECKED_STATE_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("checked-state JS failed: {}", e);
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
                    RULE_META.severity,
                    format!(
                        "Element with role '{}' is missing required attribute 'aria-checked'",
                        role
                    ),
                    selector.clone(),
                )
                .with_selector(selector)
                .with_rule_id(RULE_META.axe_id)
                .with_fix("Add 'aria-checked' attribute to this element")
                .with_help_url(RULE_META.help_url),
            )
        })
        .collect()
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
    fn test_required_attr_present_passes() {
        // Property names match the real CDP AX tree shape (unprefixed).
        let nodes = vec![
            make_node("1", "combobox", vec![("expanded", "false")]),
            make_node("2", "slider", vec![("valuenow", "50")]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_missing_required_attr_flagged() {
        // slider without a "valuenow" AX property
        let nodes = vec![make_node("1", "slider", vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("aria-valuenow"));
    }

    #[test]
    fn test_prefixed_property_name_does_not_satisfy_check() {
        // Regression check for #QA-030: a node whose property is literally
        // named "aria-valuenow" (the old, wrong shape) must NOT be treated as
        // satisfying the check — only the real CDP name ("valuenow") should.
        let nodes = vec![make_node("1", "slider", vec![("aria-valuenow", "50")])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 1);
    }

    #[test]
    fn test_ignored_node_skipped() {
        let mut node = make_node("1", "slider", vec![]);
        node.ignored = true;
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    // check_checked_state_with_page (checkbox/radio/switch aria-checked) is
    // DOM-based and needs a live Page — not unit-tested here; covered by
    // live verification instead (see plans/quality-audit-backlog.md).
}
