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

/// Rule metadata for 1.3.1
pub const INFO_RELATIONSHIPS_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Information, structure, and relationships can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "definition-list",
    tags: &["wcag2a", "wcag131", "cat.structure"],
};

/// Rule metadata for role=presentation/none hiding semantic descendants.
pub const PRESENTATION_SEMANTIC_CHILDREN_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Info and Relationships",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Presentational containers must not hide semantic child structure",
    help_url: "https://www.w3.org/WAI/WCAG21/Techniques/failures/F92",
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

        // Check for tables without proper structure
        if role == "table" {
            check_table_structure(node, tree, &mut results);
        }

        // Check for lists
        if role == "list" {
            check_list_structure(node, tree, &mut results);
        }

        // Check for form fields in fieldsets
        if is_form_control(&role) {
            check_form_grouping(node, tree, &mut results);
        }

        // Check for data cells without headers
        if role == "cell" || role == "gridcell" {
            check_cell_headers(node, tree, &mut results);
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

/// Check table has proper headers
fn check_table_structure(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    // Count header cells in the table's children
    let mut has_headers = false;
    let mut has_data_cells = false;

    for child_id in &node.child_ids {
        if let Some(child) = tree.get_node(child_id) {
            let child_role = child.role.as_deref().unwrap_or("").to_lowercase();

            // Check rowgroup and row children
            if child_role == "rowgroup" || child_role == "row" {
                for grandchild_id in &child.child_ids {
                    if let Some(grandchild) = tree.get_node(grandchild_id) {
                        let gc_role = grandchild.role.as_deref().unwrap_or("").to_lowercase();
                        if gc_role == "columnheader" || gc_role == "rowheader" {
                            has_headers = true;
                        }
                        if gc_role == "cell" || gc_role == "gridcell" {
                            has_data_cells = true;
                        }
                    }
                }
            }

            if child_role == "columnheader" || child_role == "rowheader" {
                has_headers = true;
            }
            if child_role == "cell" || child_role == "gridcell" {
                has_data_cells = true;
            }
        }
    }

    // If table has data cells but no headers, flag it
    if has_data_cells && !has_headers {
        let violation = Violation::new(
            INFO_RELATIONSHIPS_RULE.id,
            INFO_RELATIONSHIPS_RULE.name,
            INFO_RELATIONSHIPS_RULE.level,
            Severity::High,
            "Data table lacks header cells",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Add <th> elements for column and/or row headers in data tables")
        .with_help_url(INFO_RELATIONSHIPS_RULE.help_url);

        results.add_violation(violation);
    } else if has_headers {
        results.passes += 1;
    }
}

/// Check list has proper structure
fn check_list_structure(node: &AXNode, tree: &AXTree, results: &mut WcagResults) {
    let mut has_list_items = false;
    let mut has_non_list_items = false;

    for child_id in &node.child_ids {
        if let Some(child) = tree.get_node(child_id) {
            let child_role = child.role.as_deref().unwrap_or("").to_lowercase();
            if child_role == "listitem" {
                has_list_items = true;
            } else if !child_role.is_empty() && child_role != "presentation" && child_role != "none"
            {
                has_non_list_items = true;
            }
        }
    }

    if has_non_list_items && !has_list_items {
        let violation = Violation::new(
            INFO_RELATIONSHIPS_RULE.id,
            INFO_RELATIONSHIPS_RULE.name,
            INFO_RELATIONSHIPS_RULE.level,
            Severity::Medium,
            "List does not contain proper list item elements",
            &node.node_id,
        )
        .with_role(node.role.clone())
        .with_fix("Use <li> elements as direct children of <ul> or <ol> lists")
        .with_help_url(INFO_RELATIONSHIPS_RULE.help_url);

        results.add_violation(violation);
    } else if has_list_items {
        results.passes += 1;
    }
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
                        INFO_RELATIONSHIPS_RULE.id,
                        INFO_RELATIONSHIPS_RULE.name,
                        INFO_RELATIONSHIPS_RULE.level,
                        Severity::Medium,
                        "Radio button is not contained in a group",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_name(node.name.clone())
                    .with_fix("Group related radio buttons using <fieldset> and <legend> or role=\"radiogroup\"")
                    .with_help_url(INFO_RELATIONSHIPS_RULE.help_url);

                    results.add_violation(violation);
                    return;
                }
            }
        }
    }

    results.passes += 1;
}

/// Check data cells for an explicit header association.
///
/// Whether the enclosing data table exposes header cells at all is validated in
/// `check_table_structure`. Here we only credit an *explicit* cell→header
/// association via the `headers` IDREF attribute (what complex tables need). A
/// content cell without it is governed by the table-level verdict and must not
/// be counted as a pass on its own — previously every content cell passed,
/// inflating the pass count (#444).
fn check_cell_headers(node: &AXNode, _tree: &AXTree, results: &mut WcagResults) {
    let has_content = node
        .name
        .as_ref()
        .map(|n| !n.trim().is_empty())
        .unwrap_or(false);

    if has_content && !node.get_property_idrefs("headers").is_empty() {
        results.passes += 1;
    }
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

    #[test]
    fn test_info_relationships_rule_metadata() {
        assert_eq!(INFO_RELATIONSHIPS_RULE.id, "1.3.1");
        assert_eq!(INFO_RELATIONSHIPS_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_table_with_headers() {
        let table = create_node("1", "table", None, vec!["2", "3"]);
        let header = create_node("2", "columnheader", Some("Name"), vec![]);
        let cell = create_node("3", "cell", Some("John"), vec![]);

        let tree = AXTree::from_nodes(vec![table, header, cell]);
        let results = check_info_relationships(&tree);

        // Should pass - has header and cell
        assert!(results.violations.is_empty() || results.passes > 0);
    }

    #[test]
    fn test_table_without_headers() {
        let table = create_node("1", "table", None, vec!["2"]);
        let cell = create_node("2", "cell", Some("Data"), vec![]);

        let tree = AXTree::from_nodes(vec![table, cell]);
        let results = check_info_relationships(&tree);

        // Should flag - has data cell but no headers
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("header")));
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
