//! WCAG 1.3.1 Info and Relationships
//!
//! Information, structure, and relationships conveyed through presentation
//! can be programmatically determined or are available in text.
//! Level A

use chromiumoxide::Page;
use tracing::warn;

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for a radio button outside a group.
pub const RADIO_GROUP_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Radio buttons must be contained in a group",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/info-and-relationships.html",
    axe_id: "radio-group",
    tags: &["wcag2a", "wcag131", "cat.forms"],
};

/// Rule metadata for role=presentation/none hiding semantic descendants.
pub const PRESENTATION_SEMANTIC_CHILDREN_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Presentational containers must not hide semantic child structure",
    help_url: "https://www.w3.org/WAI/WCAG22/Techniques/failures/F92",
    axe_id: "presentation-semantic-children",
    tags: &["wcag2a", "wcag131", "cat.semantics"],
};

/// Check for proper info and relationships
pub fn check_info_relationships(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }

        results.nodes_checked += 1;
        let role = node.role.as_deref().unwrap_or("").to_lowercase();

        // Check for form fields in fieldsets
        if is_form_control(&role) {
            check_form_grouping(node, tree, &mut results);
        }
    }

    results
}

/// DOM check for presentational containers that include semantic descendants.
pub async fn check_presentation_semantic_children_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        r#"
        var issues = [];
        var semanticSelector = [
          'h1,h2,h3,h4,h5,h6',
          'main,nav,header,footer,article,aside,section[aria-label],section[aria-labelledby]',
          'ul,ol,dl,table,th',
          'button,input:not([type="hidden"]),select,textarea,a[href]',
          '[role]:not([role="presentation"]):not([role="none"]):not([role="generic"])'
        ].join(',');
        var containers = document.querySelectorAll('[role="presentation"], [role="none"]');
        for (var i = 0; i < containers.length; i++) {
          var el = containers[i];
          if (el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true') continue;
          var style = window.getComputedStyle(el);
          if (style && (style.display === 'none' || style.visibility === 'hidden')) continue;

          var child = el.querySelector(semanticSelector);
          if (!child) continue;
          issues.push({
            selector: __amsCssSelector(el),
            child_selector: __amsCssSelector(child),
            child_role: child.getAttribute('role') || child.tagName.toLowerCase(),
            snippet: el.outerHTML.substring(0, 200)
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
            warn!("presentation semantic children DOM JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "presentation-semantic-children",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let Some(value) = result.value() else {
        return vec![crate::wcag::technical_rule_failure_for(
            "presentation-semantic-children",
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
            let child_selector = issue
                .get("child_selector")
                .and_then(|v| v.as_str())
                .unwrap_or("semantic descendant");
            let child_role = issue
                .get("child_role")
                .and_then(|v| v.as_str())
                .unwrap_or("semantic role");
            let mut violation = Violation::new(
                PRESENTATION_SEMANTIC_CHILDREN_RULE.id,
                PRESENTATION_SEMANTIC_CHILDREN_RULE.name,
                PRESENTATION_SEMANTIC_CHILDREN_RULE.level,
                PRESENTATION_SEMANTIC_CHILDREN_RULE.severity,
                format!(
                    "Element with role=\"presentation\"/\"none\" contains semantic child {child_role} ({child_selector})"
                ),
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(PRESENTATION_SEMANTIC_CHILDREN_RULE.axe_id)
            .with_tags(
                PRESENTATION_SEMANTIC_CHILDREN_RULE
                    .tags
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
            .with_fix("Remove role=\"presentation\"/\"none\" from the container, or remove semantic roles from purely decorative descendants.")
            .with_help_url(PRESENTATION_SEMANTIC_CHILDREN_RULE.help_url);

            if let Some(snippet) = issue.get("snippet").and_then(|v| v.as_str()) {
                violation = violation.with_html_snippet(snippet);
            }

            Some(violation)
        })
        .collect()
}

/// Check form controls are properly grouped
fn check_form_grouping(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let role = node.role.as_deref().unwrap_or("").to_lowercase();

    // Radio buttons and checkboxes should be in a group
    if role == "radio" {
        // Check if parent is a radiogroup
        if let Some(ref parent_id) = node.parent_id {
            if let Some(parent) = tree.get_node(parent_id) {
                let parent_role = parent.role.as_deref().unwrap_or("").to_lowercase();
                if parent_role != "radiogroup" && parent_role != "group" {
                    let violation = Violation::new(
                        RADIO_GROUP_RULE.id,
                        RADIO_GROUP_RULE.name,
                        RADIO_GROUP_RULE.level,
                        RADIO_GROUP_RULE.severity,
                        "Radio button is not contained in a group",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_name(node.name.clone())
                    .with_fix("Group related radio buttons using <fieldset> and <legend> or role=\"radiogroup\"")
                    .with_help_url(RADIO_GROUP_RULE.help_url)
                    .with_rule_id(RADIO_GROUP_RULE.axe_id);

                    results.add_violation(violation);
                    return;
                }
            }
        }
    }

    results.passes += 1;
}

/// Check if role is a form control
fn is_form_control(role: &str) -> bool {
    matches!(
        role,
        "textbox"
            | "searchbox"
            | "combobox"
            | "listbox"
            | "spinbutton"
            | "slider"
            | "checkbox"
            | "radio"
            | "switch"
            | "button"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_node(id: &str, role: &str, name: Option<&str>, children: Vec<&str>) -> AXNode {
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
            child_ids: children.iter().map(|s| s.to_string()).collect(),
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    /// Plan 56: the radio check must not report under `definition-list`.
    #[test]
    fn test_radio_outside_group_carries_its_own_rule_id() {
        let paragraph = create_node("1", "paragraph", None, vec!["2"]);
        let mut radio = create_node("2", "radio", Some("Option"), vec![]);
        radio.parent_id = Some("1".to_string());

        let tree = AXTree::from_nodes(vec![paragraph, radio]);
        let results = check_info_relationships(&tree);

        let ids: Vec<_> = results
            .violations
            .iter()
            .filter_map(|v| v.rule_id.as_deref())
            .collect();
        assert_eq!(ids, vec!["radio-group"]);
    }

    #[test]
    fn test_is_form_control() {
        assert!(is_form_control("textbox"));
        assert!(is_form_control("checkbox"));
        assert!(is_form_control("radio"));
        assert!(!is_form_control("link"));
        assert!(!is_form_control("heading"));
    }

    #[test]
    fn test_presentation_semantic_children_metadata() {
        assert_eq!(PRESENTATION_SEMANTIC_CHILDREN_RULE.id, "1.3.1");
        assert_eq!(
            PRESENTATION_SEMANTIC_CHILDREN_RULE.axe_id,
            "presentation-semantic-children"
        );
        assert!(PRESENTATION_SEMANTIC_CHILDREN_RULE
            .tags
            .contains(&"wcag131"));
    }
}
