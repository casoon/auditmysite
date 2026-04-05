//! WCAG 1.3.1 - Label Title Only
//!
//! axe-core rule: `label-title-only`
//! Form inputs that derive their accessible name solely from a `title`
//! attribute are flagged because `title` is only exposed on hover and is
//! not reliably announced by all assistive technologies.
//!
//! A visible `<label>`, `aria-label`, or `aria-labelledby` should be used
//! instead of (or in addition to) title.

use crate::accessibility::{AXTree, NameSource};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const RULE_LABEL_TITLE_ONLY: RuleMetadata = RuleMetadata {
    id: "1.3.1",
    name: "Label Title Only",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description:
        "Form elements must not rely solely on the title attribute for their accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html",
    axe_id: "label-title-only",
    tags: &["wcag2a", "wcag131", "cat.forms"],
};

/// Form input roles that should have a proper label (not just title).
const INPUT_ROLES: &[&str] = &[
    "textbox",
    "combobox",
    "listbox",
    "slider",
    "spinbutton",
    "searchbox",
];

/// Check form inputs for title-only labelling.
pub fn check_label_title_only(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.iter() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r.to_lowercase(),
            None => continue,
        };

        if !INPUT_ROLES.contains(&role.as_str()) {
            continue;
        }

        // If the node has a name and its source is Title, flag it.
        // Also use a pragmatic fallback: if name_source is unavailable,
        // check whether the node has a "title" property whose value
        // matches the accessible name, and no aria-label/aria-labelledby.
        let is_title_only = if node.name_source == Some(NameSource::Title) {
            true
        } else if node.has_name() && node.name_source.is_none() {
            // Fallback heuristic
            let title_val = node.get_property_str("title");
            let has_aria_label = node.get_property_str("aria-label").is_some();
            let has_aria_labelledby = node.get_property_str("aria-labelledby").is_some();

            if let (Some(name), Some(title)) = (node.name.as_deref(), title_val) {
                name == title && !has_aria_label && !has_aria_labelledby
            } else {
                false
            }
        } else {
            false
        };

        if is_title_only {
            results.add_violation(
                Violation::new(
                    RULE_LABEL_TITLE_ONLY.id,
                    RULE_LABEL_TITLE_ONLY.name,
                    RULE_LABEL_TITLE_ONLY.level,
                    RULE_LABEL_TITLE_ONLY.severity,
                    format!(
                        "Form element with role=\"{}\" derives its accessible name only from the title attribute",
                        role
                    ),
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix(
                    "Add a visible <label> element or aria-label instead of relying on title attribute",
                )
                .with_rule_id(RULE_LABEL_TITLE_ONLY.axe_id)
                .with_help_url(RULE_LABEL_TITLE_ONLY.help_url),
            );
        } else if node.has_name() {
            results.passes += 1;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue, NameSource};

    fn input_node(
        id: &str,
        role: &str,
        name: Option<&str>,
        name_source: Option<NameSource>,
        props: Vec<(&str, &str)>,
    ) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: name.map(String::from),
            name_source,
            description: None,
            value: None,
            properties: props
                .into_iter()
                .map(|(k, v)| AXProperty {
                    name: k.into(),
                    value: AXValue::String(v.into()),
                })
                .collect(),
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_title_only_name_source_flagged() {
        let node = input_node(
            "1",
            "textbox",
            Some("Search"),
            Some(NameSource::Title),
            vec![],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_label_title_only(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("label-title-only")));
    }

    #[test]
    fn test_aria_label_name_source_passes() {
        let node = input_node(
            "1",
            "textbox",
            Some("Search"),
            Some(NameSource::Attribute),
            vec![],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_label_title_only(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_label_name_source_passes() {
        let node = input_node(
            "1",
            "combobox",
            Some("Country"),
            Some(NameSource::RelatedElement),
            vec![],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_label_title_only(&tree);
        assert!(r.violations.is_empty());
        assert!(r.passes >= 1);
    }

    #[test]
    fn test_fallback_heuristic_title_matches_name_flagged() {
        // No name_source, but title property matches the name
        let node = input_node(
            "1",
            "searchbox",
            Some("Search site"),
            None,
            vec![("title", "Search site")],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_label_title_only(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("label-title-only")));
    }

    #[test]
    fn test_fallback_heuristic_with_aria_label_passes() {
        // Has title matching name, but also has aria-label
        let node = input_node(
            "1",
            "searchbox",
            Some("Search site"),
            None,
            vec![("title", "Search site"), ("aria-label", "Search site")],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_label_title_only(&tree);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_non_input_role_not_flagged() {
        let node = input_node(
            "1",
            "button",
            Some("Submit"),
            Some(NameSource::Title),
            vec![],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_label_title_only(&tree);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_slider_title_only_flagged() {
        let node = input_node(
            "1",
            "slider",
            Some("Volume"),
            Some(NameSource::Title),
            vec![],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_label_title_only(&tree);
        assert!(r
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("label-title-only")));
    }
}
