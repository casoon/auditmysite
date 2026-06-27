//! WCAG 4.1.2 - ARIA Prohibited Attributes
//!
//! Validates that elements do not use ARIA attributes that are explicitly
//! prohibited for their role.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};
use chromiumoxide::Page;
use tracing::warn;

/// Rule metadata for ARIA prohibited attributes
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "aria-prohibited-attr",
    name: "ARIA Prohibited Attributes",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "ARIA attributes that are prohibited on a role must not be used",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-prohibited-attr",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Roles and the ARIA attributes that are prohibited on them
const PROHIBITED_ATTRS: &[(&str, &[&str])] = &[
    ("presentation", &["aria-label", "aria-labelledby"]),
    ("none", &["aria-label", "aria-labelledby"]),
    (
        "generic",
        &["aria-label", "aria-labelledby", "aria-roledescription"],
    ),
    ("code", &["aria-label", "aria-labelledby"]),
    ("emphasis", &["aria-label", "aria-labelledby"]),
    ("strong", &["aria-label", "aria-labelledby"]),
    ("subscript", &["aria-label", "aria-labelledby"]),
    ("superscript", &["aria-label", "aria-labelledby"]),
    ("deletion", &["aria-label", "aria-labelledby"]),
    ("insertion", &["aria-label", "aria-labelledby"]),
];

/// Check that elements do not use ARIA attributes prohibited for their role
pub fn check_aria_prohibited_attr(tree: &AXTree) -> WcagResults {
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

        let prohibited = match PROHIBITED_ATTRS.iter().find(|(r, _)| *r == role) {
            Some((_, attrs)) => attrs,
            None => continue,
        };

        for prop in &node.properties {
            if !prop.name.starts_with("aria-") {
                continue;
            }

            let attr_name = prop.name.as_str();

            if prohibited.contains(&attr_name) {
                let violation = Violation::new(
                    RULE_META.id,
                    RULE_META.name,
                    RULE_META.level,
                    RULE_META.severity,
                    format!(
                        "ARIA attribute '{}' is prohibited on role '{}'",
                        attr_name, role
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_rule_id(RULE_META.axe_id)
                .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
                .with_fix(format!(
                    "Remove '{}' from this element with role '{}'",
                    attr_name, role
                ))
                .with_help_url(RULE_META.help_url);

                results.add_violation(violation);
            }
        }
    }

    results
}

/// DOM supplement for generic elements that may be omitted or simplified in
/// the AX tree. Mirrors axe-core's `aria-prohibited-attr` check for the common
/// case of naming a generic `div`/`span` without assigning a valid role.
pub async fn check_aria_prohibited_attr_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        var issues = [];
        var selector = [
          'div:not([role])[aria-label]',
          'div:not([role])[aria-labelledby]',
          'div:not([role])[aria-roledescription]',
          'span:not([role])[aria-label]',
          'span:not([role])[aria-labelledby]',
          'span:not([role])[aria-roledescription]'
        ].join(',');
        var elements = document.querySelectorAll(selector);
        for (var i = 0; i < elements.length; i++) {
          var el = elements[i];
          var s = window.getComputedStyle(el);
          if (s.display === 'none' || s.visibility === 'hidden') continue;
          var attrs = [];
          ['aria-label', 'aria-labelledby', 'aria-roledescription'].forEach(function(attr) {
            if (el.hasAttribute(attr)) attrs.push(attr);
          });
          if (attrs.length === 0) continue;
          issues.push({
            selector: __amsCssSelector(el),
            snippet: el.outerHTML.substring(0, 200),
            attrs: attrs.join(', ')
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
            warn!("aria-prohibited-attr DOM JS failed: {}", e);
            return vec![];
        }
    };

    let Some(value) = result.value() else {
        return vec![];
    };
    let Some(issues) = value.as_array() else {
        return vec![];
    };

    issues
        .iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let attrs = issue.get("attrs")?.as_str()?.to_string();
            let mut violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                RULE_META.severity,
                format!(
                    "ARIA attributes '{}' cannot be used on a generic element without a valid role",
                    attrs
                ),
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(RULE_META.axe_id)
            .with_tags(RULE_META.tags.iter().map(|s| s.to_string()).collect())
            .with_fix("Remove the prohibited ARIA attribute or add a valid semantic role.")
            .with_help_url(RULE_META.help_url);

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
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
    fn test_no_prohibited_attrs_passes() {
        let nodes = vec![
            make_node("1", "button", vec![("aria-label", "OK")]),
            make_node("2", "presentation", vec![]),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_prohibited_attr(&tree);
        assert_eq!(results.violations.len(), 0);
    }

    #[test]
    fn test_prohibited_attr_on_presentation_flagged() {
        let nodes = vec![make_node(
            "1",
            "presentation",
            vec![("aria-label", "should not be here")],
        )];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_prohibited_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0].message.contains("aria-label"));
        assert!(results.violations[0].message.contains("presentation"));
    }

    #[test]
    fn test_prohibited_attr_on_generic_flagged() {
        let nodes = vec![make_node(
            "1",
            "generic",
            vec![("aria-roledescription", "fancy thing")],
        )];
        let tree = AXTree::from_nodes(nodes);
        let results = check_aria_prohibited_attr(&tree);
        assert_eq!(results.violations.len(), 1);
        assert!(results.violations[0]
            .message
            .contains("aria-roledescription"));
    }
}
