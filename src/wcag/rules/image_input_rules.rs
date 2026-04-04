//! WCAG 1.1.1 - Non-text Content: additional input/object rules
//!
//! Covers axe-core rules that extend image-alt to other non-text elements:
//! - `area-alt`:        <area> elements in image maps must have alt text
//! - `input-image-alt`: <input type="image"> must have alt text
//! - `object-alt`:      <object> elements must have a text alternative

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const RULE_AREA_ALT: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Area Alternative Text",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Active <area> elements in image maps must have alternative text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "area-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

pub const RULE_INPUT_IMAGE_ALT: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Image Button Alternative Text",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "<input type=\"image\"> elements must have alternative text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "input-image-alt",
    tags: &["wcag2a", "wcag111", "cat.images"],
};

pub const RULE_OBJECT_ALT: RuleMetadata = RuleMetadata {
    id: "1.1.1",
    name: "Object Alternative Text",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "<object> elements must have a text alternative",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html",
    axe_id: "object-alt",
    tags: &["wcag2a", "wcag111", "cat.text-alternatives"],
};

/// Run all image-input/object text-alternative checks.
pub fn check_image_input_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };

        // area elements appear as "link" in the AX tree; identify by axe-hint or description
        if role == "link" && is_area_element(node) && !node.has_name() {
            let v = Violation::new(
                RULE_AREA_ALT.id,
                RULE_AREA_ALT.name,
                RULE_AREA_ALT.level,
                RULE_AREA_ALT.severity,
                "Active <area> element is missing alternative text",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix("Add an alt attribute to the <area> element describing its destination")
            .with_help_url(RULE_AREA_ALT.help_url);
            results.add_violation(v);
        }
        // input[type=image] typically surfaces as a button role with "type" property "image"
        else if is_input_image(node) && !node.has_name() {
            let v = Violation::new(
                RULE_INPUT_IMAGE_ALT.id,
                RULE_INPUT_IMAGE_ALT.name,
                RULE_INPUT_IMAGE_ALT.level,
                RULE_INPUT_IMAGE_ALT.severity,
                "Image submit button is missing alternative text",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix(
                "Add an alt attribute to the <input type=\"image\"> element \
                 describing the button action",
            )
            .with_help_url(RULE_INPUT_IMAGE_ALT.help_url);
            results.add_violation(v);
        }
        // object elements exposed as various roles; check via description/name conventions
        else if is_object_element(node) && !node.has_name() {
            let v = Violation::new(
                RULE_OBJECT_ALT.id,
                RULE_OBJECT_ALT.name,
                RULE_OBJECT_ALT.level,
                RULE_OBJECT_ALT.severity,
                "<object> element is missing a text alternative",
                &node.node_id,
            )
            .with_role(node.role.clone())
            .with_fix("Provide a text alternative inside the <object> element or via aria-label")
            .with_help_url(RULE_OBJECT_ALT.help_url);
            results.add_violation(v);
        }
    }

    results
}

/// Heuristic: an AX link node that is an area element.
/// Chrome CDP sometimes sets a "type" or "htmlTag" property, or the description contains "area".
fn is_area_element(node: &AXNode) -> bool {
    // Check property "htmlTag" = "AREA" (set by some CDP versions)
    if let Some(tag) = node.get_property_str("htmlTag") {
        return tag.eq_ignore_ascii_case("area");
    }
    // Fallback: description contains "area" hint
    if let Some(ref desc) = node.description {
        return desc.to_lowercase().contains("area");
    }
    false
}

/// Heuristic: input[type=image] — role "button" with type property "image".
fn is_input_image(node: &AXNode) -> bool {
    if !matches!(node.role.as_deref(), Some("button")) {
        return false;
    }
    if let Some(t) = node.get_property_str("type") {
        return t.eq_ignore_ascii_case("image");
    }
    false
}

/// Heuristic: <object> element — exposed as "group" or no particular role,
/// with a "htmlTag" = "OBJECT" or "EMBED" property.
fn is_object_element(node: &AXNode) -> bool {
    if let Some(tag) = node.get_property_str("htmlTag") {
        let tag = tag.to_uppercase();
        return tag == "OBJECT" || tag == "EMBED";
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXValue};

    fn node(id: &str, role: &str, name: Option<&str>, props: Vec<(&str, &str)>) -> AXNode {
        AXNode {
            node_id: id.into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.into()),
            name: name.map(String::from),
            name_source: None,
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
    fn test_input_image_without_alt_flagged() {
        let n = node("1", "button", None, vec![("type", "image")]);
        let tree = AXTree::from_nodes(vec![n]);
        let results = check_image_input_rules(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.rule_id.as_deref() == Some("input-image-alt")
                    || v.rule_name == "Image Button Alternative Text"),
            "input[type=image] without name should be flagged"
        );
    }

    #[test]
    fn test_input_image_with_alt_passes() {
        let n = node("1", "button", Some("Submit form"), vec![("type", "image")]);
        let tree = AXTree::from_nodes(vec![n]);
        let results = check_image_input_rules(&tree);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_object_without_alt_flagged() {
        let n = node("1", "group", None, vec![("htmlTag", "OBJECT")]);
        let tree = AXTree::from_nodes(vec![n]);
        let results = check_image_input_rules(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.rule_name == "Object Alternative Text"),
            "object without name should be flagged"
        );
    }

    #[test]
    fn test_object_with_name_passes() {
        let n = node(
            "1",
            "group",
            Some("Product demo video"),
            vec![("htmlTag", "OBJECT")],
        );
        let tree = AXTree::from_nodes(vec![n]);
        let results = check_image_input_rules(&tree);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_area_without_alt_flagged() {
        let n = node("1", "link", None, vec![("htmlTag", "AREA")]);
        let tree = AXTree::from_nodes(vec![n]);
        let results = check_image_input_rules(&tree);
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.rule_name == "Area Alternative Text"),
            "area without alt should be flagged"
        );
    }

    #[test]
    fn test_area_with_alt_passes() {
        let n = node(
            "1",
            "link",
            Some("Go to contact page"),
            vec![("htmlTag", "AREA")],
        );
        let tree = AXTree::from_nodes(vec![n]);
        let results = check_image_input_rules(&tree);
        assert!(results.violations.is_empty());
    }
}
