//! WCAG 1.3.1, 3.3.1, 3.3.2 - Form Rules
//!
//! Checks grouped form controls for fieldset/legend, required field indication,
//! and error descriptions for invalid fields.

use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for form structure rules (1.3.1)
///
/// Own id: axe-core has no rule for missing fieldset grouping. The id used to
/// be `form-field-multiple-labels`, which in axe-core means a field with more
/// than one `<label>` — a different defect (plan 56).
pub const RULE_META_STRUCTURE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships - Forms",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Grouped form controls must have a fieldset/legend; form structure must be programmatically determinable",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    axe_id: "form-field-group",
    tags: &["wcag2a", "wcag131", "cat.forms"],
};

/// Rule metadata for error identification rules (3.3.1)
pub const RULE_META_ERROR: RuleMetadata = RuleMetadata {
    id: "3.3.1",
    name: "Error Identification",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "If an input error is automatically detected, the item that is in error must be identified and the error described in text",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/error-identification.html",
    axe_id: "input-error-message",
    tags: &["wcag2a", "wcag331", "cat.forms"],
};

/// Rule metadata for labels/instructions (3.3.2)
pub const RULE_META_LABELS: RuleMetadata = RuleMetadata {
    id: "3.3.2",
    name: "Labels or Instructions",
    level: WcagLevel::A,
    severity: Severity::Low,
    description: "Labels or instructions are provided when content requires user input",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions.html",
    axe_id: "label",
    tags: &["wcag2a", "wcag332", "cat.forms"],
};

/// Rule metadata for forms without an explicit submit control (H32).
pub const RULE_META_FORM_NO_SUBMIT: RuleMetadata = RuleMetadata {
    id: "3.2.2",
    name: "On Input",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Forms that collect user input should provide an explicit submit control",
    help_url: "https://www.w3.org/WAI/WCAG22/Techniques/html/H32",
    axe_id: "form-no-submit",
    tags: &["wcag2a", "wcag322", "cat.forms"],
};

/// Run all form-related WCAG checks
pub fn check_form_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    check_grouped_controls(tree, &mut results);
    check_required_field_indication(tree, &mut results);
    check_invalid_field_description(tree, &mut results);

    results
}

/// DOM check for forms that collect user input but have no submit button.
pub async fn check_form_no_submit_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        var issues = [];
        var forms = document.querySelectorAll('form');
        for (var i = 0; i < forms.length; i++) {
          var form = forms[i];
          if (form.hasAttribute('hidden') || form.getAttribute('aria-hidden') === 'true') continue;
          var style = window.getComputedStyle(form);
          if (style && (style.display === 'none' || style.visibility === 'hidden')) continue;

          var controls = form.querySelectorAll(
            'input:not([type="hidden"]):not([disabled]), select:not([disabled]), ' +
            'textarea:not([disabled]), button:not([disabled])'
          );
          if (controls.length === 0) continue;

          var submit = form.querySelector(
            'button:not([type]):not([disabled]), button[type="submit"]:not([disabled]), ' +
            'input[type="submit"]:not([disabled]), input[type="image"]:not([disabled]), ' +
            '[role="button"][data-submit], [role="button"][aria-label*="submit" i], ' +
            '[role="button"][aria-label*="send" i], [role="button"][aria-label*="search" i]'
          );
          if (submit) continue;

          issues.push({
            selector: __amsCssSelector(form),
            snippet: form.outerHTML.substring(0, 200),
            controls: controls.length
          });
        }
        return issues;
        "#,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("form-no-submit DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "form-no-submit",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "form-no-submit",
            crate::cli::WcagLevel::A,
            "missing_evaluation_value",
        )];
    };
    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let controls = issue.get("controls").and_then(|v| v.as_u64()).unwrap_or(0);
            let mut violation = Violation::new(
                RULE_META_FORM_NO_SUBMIT.id,
                RULE_META_FORM_NO_SUBMIT.name,
                RULE_META_FORM_NO_SUBMIT.level,
                RULE_META_FORM_NO_SUBMIT.severity,
                format!(
                    "Form has {controls} input {} but no explicit submit button",
                    if controls == 1 { "control" } else { "controls" }
                ),
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(RULE_META_FORM_NO_SUBMIT.axe_id)
            .with_tags(
                RULE_META_FORM_NO_SUBMIT
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix(
                "Add a visible <button type=\"submit\"> or equivalent explicit submit control.",
            )
            .with_help_url(RULE_META_FORM_NO_SUBMIT.help_url);

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}

/// DOM check for related checkboxes without a group ancestor (1.3.1).
///
/// A single checkbox ("I accept the terms") is not a group. Checkboxes are
/// related when they share a `name` within the same form owner — the HTML
/// signal for "several answers to one question". Only such a set of two or
/// more rendered checkboxes needs a `<fieldset>` or `role="group"` ancestor
/// carrying the question (#643; the tree-only check used to demand one for
/// every checkbox as soon as the page had two radio/checkbox controls
/// anywhere).
pub async fn check_checkbox_group_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        function rendered(el) {
          if (el.closest('[hidden], [aria-hidden="true"]')) return false;
          if (el.getClientRects().length === 0) return false;
          var style = window.getComputedStyle(el);
          return !(style && style.visibility === 'hidden');
        }
        var forms = Array.prototype.slice.call(document.forms);
        var sets = new Map();
        var boxes = document.querySelectorAll('input[type="checkbox"][name]');
        for (var i = 0; i < boxes.length; i++) {
          var box = boxes[i];
          var name = box.name.trim();
          if (!name || !rendered(box)) continue;
          var key = forms.indexOf(box.form) + '\u0000' + name;
          if (!sets.has(key)) sets.set(key, []);
          sets.get(key).push(box);
        }
        var issues = [];
        sets.forEach(function(set) {
          if (set.length < 2) return;
          for (var j = 0; j < set.length; j++) {
            if (set[j].closest('fieldset, [role="group"], [role="radiogroup"]')) continue;
            issues.push({
              selector: __amsCssSelector(set[j]),
              snippet: set[j].outerHTML.substring(0, 200)
            });
          }
        });
        return issues;
        "#,
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("form-field-group DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                RULE_META_STRUCTURE.axe_id,
                RULE_META_STRUCTURE.level,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            RULE_META_STRUCTURE.axe_id,
            RULE_META_STRUCTURE.level,
            "missing_evaluation_value",
        )];
    };
    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let mut violation = Violation::new(
                RULE_META_STRUCTURE.id,
                RULE_META_STRUCTURE.name,
                RULE_META_STRUCTURE.level,
                Severity::Medium,
                "Grouped form controls may be missing a fieldset/legend",
                &selector,
            )
            .with_selector(&selector)
            .with_role(Some("checkbox".to_string()))
            .with_fix(
                "Wrap related radio buttons or checkboxes in a <fieldset> with a <legend>, or use role=\"group\" with aria-labelledby",
            )
            .with_help_url(RULE_META_STRUCTURE.help_url)
            .with_rule_id(RULE_META_STRUCTURE.axe_id);

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}

/// Check that radio buttons have a group ancestor.
///
/// A radio button always needs one: it only makes sense as one option of a
/// set, and the question the set answers lives in the group's name.
/// Checkboxes are checked by [`check_checkbox_group_with_page`]: whether two
/// checkboxes belong together is decided by their shared `name` attribute,
/// which the AX tree does not carry (#643).
///
/// Also covers what `info_relationships` checked as `radio-group` (radio whose
/// *direct* parent is not a group) — that check flagged the same radio a
/// second time, and falsely whenever a `<label>` sat between radio and
/// `<fieldset>` (plan 56).
fn check_grouped_controls(tree: &AXTree, results: &mut WcagResults) {
    let grouped_nodes: Vec<&AXNode> = tree
        .nodes
        .values()
        .filter(|n| !n.ignored && n.role.as_deref() == Some("radio"))
        .collect();

    results.nodes_checked += grouped_nodes.len();

    for node in &grouped_nodes {
        let has_group = has_ancestor_with_role(node, &["group", "radiogroup"], tree);
        if !has_group {
            let violation = Violation::new(
                RULE_META_STRUCTURE.id,
                RULE_META_STRUCTURE.name,
                RULE_META_STRUCTURE.level,
                Severity::Medium,
                "Grouped form controls may be missing a fieldset/legend",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Wrap related radio buttons or checkboxes in a <fieldset> with a <legend>, or use role=\"group\" with aria-labelledby",
            )
            .with_help_url(RULE_META_STRUCTURE.help_url)
            .with_rule_id(RULE_META_STRUCTURE.axe_id);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }
}

/// Check that required fields have some indication of being required in their label
fn check_required_field_indication(tree: &AXTree, results: &mut WcagResults) {
    let input_roles = ["textbox", "combobox", "spinbutton", "listbox"];

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        let role = match node.role.as_deref() {
            Some(r) if input_roles.contains(&r) => r,
            _ => continue,
        };

        results.nodes_checked += 1;

        if !node.is_required() {
            results.passes += 1;
            continue;
        }

        let name_lower = node.name.as_deref().unwrap_or("").to_lowercase();
        let has_indication = name_lower.contains('*')
            || name_lower.contains("required")
            || name_lower.contains("pflicht")
            || name_lower.contains("obligatoire");

        if !has_indication {
            let violation = Violation::new(
                RULE_META_LABELS.id,
                RULE_META_LABELS.name,
                RULE_META_LABELS.level,
                Severity::Low,
                format!(
                    "Required {} field may not indicate required status in its label",
                    role
                ),
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Include an asterisk (*) or the word \"required\" in the label, or use aria-required and explain the convention",
            )
            .with_help_url(RULE_META_LABELS.help_url)
            .with_rule_id(RULE_META_LABELS.axe_id);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }
}

/// Check that invalid fields have an accessible error description
fn check_invalid_field_description(tree: &AXTree, results: &mut WcagResults) {
    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        if node.role.is_none() {
            continue;
        }

        results.nodes_checked += 1;

        if !node.is_invalid() {
            results.passes += 1;
            continue;
        }

        let has_description = node
            .description
            .as_ref()
            .is_some_and(|d| !d.trim().is_empty());

        if !has_description {
            let violation = Violation::new(
                RULE_META_ERROR.id,
                RULE_META_ERROR.name,
                RULE_META_ERROR.level,
                Severity::Medium,
                "Invalid field has no accessible error description",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_name(node.name.clone())
            .with_fix(
                "Add aria-describedby pointing to an element containing the error message, or use aria-errormessage",
            )
            .with_help_url(RULE_META_ERROR.help_url)
            .with_rule_id(RULE_META_ERROR.axe_id);

            results.add_violation(violation);
        } else {
            results.passes += 1;
        }
    }
}

/// Walk up the ancestor chain to check whether any ancestor has one of the given roles
fn has_ancestor_with_role(node: &AXNode, roles: &[&str], tree: &AXTree) -> bool {
    let mut current_parent_id = node.parent_id.as_deref();

    while let Some(parent_id) = current_parent_id {
        if let Some(parent) = tree.get_node(parent_id) {
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

    fn make_node(id: &str, role: &str, name: Option<&str>, parent_id: Option<&str>) -> AXNode {
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
            parent_id: parent_id.map(String::from),
            backend_dom_node_id: None,
        }
    }

    fn make_node_required(id: &str, role: &str, name: Option<&str>) -> AXNode {
        let mut node = make_node(id, role, name, None);
        node.properties.push(AXProperty {
            name: "required".to_string(),
            value: AXValue::Bool(true),
        });
        node
    }

    fn make_node_invalid(id: &str, role: &str) -> AXNode {
        let mut node = make_node(id, role, Some("Field"), None);
        node.properties.push(AXProperty {
            // Real CDP traffic carries aria-invalid as an AXValue::String
            // token ("true"/"false"/"grammar"/"spelling"), never a Bool (#566).
            name: "invalid".to_string(),
            value: AXValue::String("true".to_string()),
        });
        node
    }

    #[test]
    fn test_radio_without_group_flagged() {
        let nodes = vec![
            make_node("1", "radio", Some("Option A"), None),
            make_node("2", "radio", Some("Option B"), None),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("fieldset/legend")));
    }

    #[test]
    fn test_radio_with_group_passes() {
        let nodes = vec![
            make_node("g", "group", Some("Options"), None),
            make_node("1", "radio", Some("Option A"), Some("g")),
            make_node("2", "radio", Some("Option B"), Some("g")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("fieldset/legend")));
    }

    /// A lone radio button still needs a group (it used to be flagged only by
    /// `info_relationships`' `radio-group` check, plan 56); a lone checkbox
    /// does not.
    #[test]
    fn test_single_radio_without_group_flagged_single_checkbox_not() {
        let radio = AXTree::from_nodes(vec![make_node("1", "radio", Some("Option"), None)]);
        let ids: Vec<_> = check_form_rules(&radio)
            .violations
            .iter()
            .filter_map(|v| v.rule_id.clone())
            .collect();
        assert_eq!(ids, vec!["form-field-group".to_string()]);

        let checkbox = AXTree::from_nodes(vec![make_node("1", "checkbox", Some("Accept"), None)]);
        assert!(check_form_rules(&checkbox).violations.is_empty());
    }

    /// #643: radios elsewhere on the page do not turn a lone checkbox into a
    /// group. Related checkboxes are identified by their shared `name` in the
    /// DOM rule (`check_checkbox_group_with_page`), not here.
    #[test]
    fn test_lone_checkbox_next_to_grouped_radios_not_flagged() {
        let nodes = vec![
            make_node("g", "group", Some("Mode"), None),
            make_node("1", "radio", Some("Broken"), Some("g")),
            make_node("2", "radio", Some("Fixed"), Some("g")),
            make_node("3", "checkbox", Some("Screen reader view"), None),
        ];
        let tree = AXTree::from_nodes(nodes);
        assert!(check_form_rules(&tree).violations.is_empty());
    }

    /// The group may be any ancestor, not just the direct parent: a `<label>`
    /// between radio and `<fieldset>` is the common markup.
    #[test]
    fn test_radio_with_group_grandparent_passes() {
        let nodes = vec![
            make_node("g", "group", Some("Options"), None),
            make_node("l", "LabelText", None, Some("g")),
            make_node("1", "radio", Some("Option A"), Some("l")),
            make_node("2", "radio", Some("Option B"), Some("g")),
        ];
        let tree = AXTree::from_nodes(nodes);
        assert!(check_form_rules(&tree).violations.is_empty());
    }

    #[test]
    fn test_required_field_without_indication_flagged() {
        let nodes = vec![make_node_required("1", "textbox", Some("Email"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("required status")));
    }

    #[test]
    fn test_required_field_with_asterisk_passes() {
        let nodes = vec![make_node_required("1", "textbox", Some("Email *"))];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("required status")));
    }

    #[test]
    fn test_invalid_field_without_description_flagged() {
        let nodes = vec![make_node_invalid("1", "textbox")];
        let tree = AXTree::from_nodes(nodes);
        let results = check_form_rules(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible error description")));
    }

    #[test]
    fn test_invalid_field_with_description_passes() {
        let mut node = make_node_invalid("1", "textbox");
        node.description = Some("Please enter a valid email address".to_string());
        let tree = AXTree::from_nodes(vec![node]);
        let results = check_form_rules(&tree);
        assert!(!results
            .violations
            .iter()
            .any(|v| v.message.contains("no accessible error description")));
    }

    #[test]
    fn test_form_no_submit_metadata() {
        assert_eq!(RULE_META_FORM_NO_SUBMIT.id, "3.2.2");
        assert_eq!(RULE_META_FORM_NO_SUBMIT.axe_id, "form-no-submit");
        assert!(RULE_META_FORM_NO_SUBMIT.tags.contains(&"wcag322"));
    }
}
