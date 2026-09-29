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
//!
//! The `aria-valuenow` requirement of `meter`/`scrollbar`/`separator`/
//! `slider`/`spinbutton` is DOM-level as well (`check_value_now_with_page`):
//! CDP has no `valuenow` AX property at all — the current value is the
//! node's `value`, and Chrome synthesizes one (50 for a slider, 0 for a
//! spinbutton) when the author set none. The AX-tree presence check
//! therefore flagged every such widget, native `<input type=range|number>`
//! and the internal day/month/year fields of `<input type=date>` included
//! (#656).

use chromiumoxide::Page;

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
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html",
    axe_id: "aria-required-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles and their required ARIA attributes, by CDP AX property name
/// (unprefixed — see module docs). `checkbox`/`radio`/`switch`'s `checked`
/// requirement is NOT listed here — see `check_checked_state_with_page`;
/// neither is `aria-valuenow` — see `check_value_now_with_page`.
const REQUIRED_ATTRS: &[(&str, &[&str])] = &[
    ("combobox", &["expanded"]),
    ("heading", &["level"]),
    ("scrollbar", &["controls"]),
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

    let val = match crate::wcag::types::evaluate_or_fail_for(
        page,
        "checked-state",
        crate::cli::WcagLevel::A,
        js.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
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

const VALUE_NOW_CAP: usize = 250;

/// Walks the document and every open shadow root: web components render
/// their widgets there. A user-agent shadow root — the day/month/year
/// spinbuttons of `<input type=date>` — is not reachable from script, and
/// those fields are the browser's, not the author's.
const VALUE_NOW_BODY: &str = r#"
  var issues = [];
  var roles = { meter: 1, scrollbar: 1, separator: 1, slider: 1, spinbutton: 1 };
  var visit = function(root) {
    var elems = root.querySelectorAll('*');
    for (var i = 0; i < elems.length && issues.length < CAP; i++) {
      var el = elems[i];
      if (el.shadowRoot) visit(el.shadowRoot);
      var role = (el.getAttribute('role') || '').trim().toLowerCase().split(/\s+/)[0];
      if (!roles[role]) continue;
      // Only a focusable (splitter) separator has a value (ARIA 1.2).
      if (role === 'separator' && !el.hasAttribute('tabindex')) continue;
      // Native range/number inputs and <meter> expose their value natively.
      var type = (el.getAttribute('type') || '').toLowerCase();
      if (el.tagName === 'INPUT' && (type === 'range' || type === 'number')) continue;
      if (el.tagName === 'METER') continue;
      if (el.hasAttribute('aria-valuenow')) continue;
      if (__amsIsAriaHidden(el) || (el.checkVisibility && !el.checkVisibility())) continue;
      issues.push({ role: role, selector: __amsCssSelector(el) });
    }
  };
  visit(document);
  return { issues: issues };
"#;

/// Check that custom `meter`/`scrollbar`/`separator` (focusable)/`slider`/
/// `spinbutton` widgets set `aria-valuenow`. Native `<input type=range>`,
/// `<input type=number>` and `<meter>` are exempt, and the internal fields
/// of native date/time inputs are out of reach. DOM-level: see module docs
/// for why this can't be an AX-tree presence check.
pub async fn check_value_now_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        crate::accessibility::js_helpers::IS_ARIA_HIDDEN_JS,
        &VALUE_NOW_BODY.replace("CAP", &VALUE_NOW_CAP.to_string()),
        "})()",
    ]
    .concat();

    let val = match crate::wcag::types::evaluate_or_fail_for(
        page,
        "value-now",
        crate::cli::WcagLevel::A,
        js.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
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
                        "Element with role '{}' is missing required attribute 'aria-valuenow'",
                        role
                    ),
                    selector.clone(),
                )
                .with_role(Some(role.to_string()))
                .with_selector(selector)
                .with_rule_id(RULE_META.axe_id)
                .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
                .with_fix("Add 'aria-valuenow' attribute to this element")
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
            make_node("2", "heading", vec![("level", "2")]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_missing_required_attr_flagged() {
        // combobox without an "expanded" AX property
        let nodes = vec![make_node("1", "combobox", vec![])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("aria-expanded"));
    }

    #[test]
    fn test_prefixed_property_name_does_not_satisfy_check() {
        // Regression check for #QA-030: a node whose property is literally
        // named "aria-expanded" (the old, wrong shape) must NOT be treated as
        // satisfying the check — only the real CDP name ("expanded") should.
        let nodes = vec![make_node("1", "combobox", vec![("aria-expanded", "false")])];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 1);
    }

    /// CDP exposes no `valuenow` property, so the tree check must not judge
    /// value widgets at all — `check_value_now_with_page` does (#656).
    #[test]
    fn value_widgets_are_left_to_the_dom_check() {
        let nodes = ["meter", "separator", "slider", "spinbutton"]
            .iter()
            .enumerate()
            .map(|(i, role)| make_node(&i.to_string(), role, vec![]))
            .collect();
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_required_attr(&tree);
        assert!(results.violations.is_empty(), "{:?}", results.violations);
    }

    #[test]
    fn test_ignored_node_skipped() {
        let mut node = make_node("1", "combobox", vec![]);
        node.ignored = true;
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_aria_required_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    // check_checked_state_with_page (checkbox/radio/switch aria-checked) and
    // check_value_now_with_page (aria-valuenow) are DOM-based and need a live
    // Page — not unit-tested here; check_value_now_with_page is covered by
    // the detection corpus (value_widgets_native / value_widgets_custom).
}
