//! WCAG 2.2 and axe-core best-practice rules
//!
//! - `empty-heading`:       heading element with no text content
//! - `label-title-only`:    form element whose only accessible name comes from title
//! - `summary-name`:        <details> element without an accessible <summary> label
//! - `meta-viewport-large`: viewport meta tag restricts zoom / maximum-scale
//! - `target-size`:         interactive targets smaller than 24 × 24 CSS px (WCAG 2.2 AA)
//!                          (informational stub — pixel dimensions not available in AX tree)

use crate::accessibility::{AXTree, NameSource};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

// ── Rule metadata ──────────────────────────────────────────────────────────────

pub const RULE_EMPTY_HEADING: RuleMetadata = RuleMetadata {
    id: "2.4.6",
    name: "Empty Heading",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Heading elements must contain non-empty text",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/headings-and-labels.html",
    axe_id: "empty-heading",
    tags: &["wcag2aa", "wcag246", "cat.name-role-value"],
};

pub const RULE_LABEL_TITLE_ONLY: RuleMetadata = RuleMetadata {
    id: "3.3.2",
    name: "Label Not Title Only",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Form elements must not rely solely on the title attribute for their accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/labels-or-instructions.html",
    axe_id: "label-title-only",
    tags: &["wcag2a", "wcag332", "cat.forms"],
};

pub const RULE_SUMMARY_NAME: RuleMetadata = RuleMetadata {
    id: "4.1.2",
    name: "Summary Accessible Name",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "<details> disclosure widgets should have a descriptive <summary> accessible name",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "summary-name",
    tags: &["wcag2a", "wcag412", "cat.name-role-value"],
};

pub const RULE_META_VIEWPORT_LARGE: RuleMetadata = RuleMetadata {
    id: "1.4.4",
    name: "Meta Viewport Large Scale",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "The viewport meta tag must not restrict users from scaling the page beyond 3× its default size",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/resize-text.html",
    axe_id: "meta-viewport-large",
    tags: &["wcag2aa", "wcag144", "cat.sensory-and-visual-cues"],
};

pub const RULE_TARGET_SIZE: RuleMetadata = RuleMetadata {
    id: "2.5.8",
    name: "Target Size",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Interactive targets should be at least 24 × 24 CSS pixels (WCAG 2.2 AA)",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html",
    axe_id: "target-size",
    tags: &["wcag22aa", "wcag258", "cat.sensory-and-visual-cues"],
};

// ── Heading roles ──────────────────────────────────────────────────────────────

const HEADING_ROLES: &[&str] = &[
    "heading",
    // Chrome CDP sometimes exposes concrete levels as separate roles
    "h1", "h2", "h3", "h4", "h5", "h6",
];

// ── Form input roles that may use title as fallback label ─────────────────────

const LABELED_INPUT_ROLES: &[&str] = &[
    "textbox", "combobox", "listbox", "searchbox", "spinbutton",
    "checkbox", "radio", "switch", "slider",
];

// ── Public check function ──────────────────────────────────────────────────────

/// Run WCAG 2.2 and axe best-practice checks.
pub fn check_wcag22_rules(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        results.nodes_checked += 1;

        let role = match node.role.as_deref() {
            Some(r) => r.to_lowercase(),
            None => continue,
        };
        let role = role.as_str();

        // ── empty-heading ──────────────────────────────────────────────────
        if HEADING_ROLES.contains(&role) {
            if !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_EMPTY_HEADING.id,
                        RULE_EMPTY_HEADING.name,
                        RULE_EMPTY_HEADING.level,
                        RULE_EMPTY_HEADING.severity,
                        "Heading element is empty — it has no accessible text",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add descriptive text content to the heading, or remove the empty heading element")
                    .with_rule_id(RULE_EMPTY_HEADING.axe_id).with_help_url(RULE_EMPTY_HEADING.help_url),
                );
            } else {
                results.passes += 1;
            }
        }

        // ── label-title-only ──────────────────────────────────────────────
        // Form elements whose accessible name is sourced from the title attribute
        // provide a poor experience: the title is only shown on hover and is
        // invisible to many AT users.
        if LABELED_INPUT_ROLES.contains(&role) {
            if node.name_source == Some(NameSource::Title) {
                results.add_violation(
                    Violation::new(
                        RULE_LABEL_TITLE_ONLY.id,
                        RULE_LABEL_TITLE_ONLY.name,
                        RULE_LABEL_TITLE_ONLY.level,
                        RULE_LABEL_TITLE_ONLY.severity,
                        format!(
                            "Form element with role=\"{}\" uses only the title attribute as its accessible name",
                            role
                        ),
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Replace the title-only label with a visible <label> element or add an aria-label / aria-labelledby")
                    .with_rule_id(RULE_LABEL_TITLE_ONLY.axe_id).with_help_url(RULE_LABEL_TITLE_ONLY.help_url),
                );
            } else if node.has_name() {
                results.passes += 1;
            }
        }

        // ── summary-name ──────────────────────────────────────────────────
        // Chrome CDP exposes <details>/<summary> via htmlTag property.
        // A disclosure widget without a meaningful summary label is unclear.
        if let Some(tag) = node.get_property_str("htmlTag") {
            if tag.eq_ignore_ascii_case("DETAILS") && !node.has_name() {
                results.add_violation(
                    Violation::new(
                        RULE_SUMMARY_NAME.id,
                        RULE_SUMMARY_NAME.name,
                        RULE_SUMMARY_NAME.level,
                        RULE_SUMMARY_NAME.severity,
                        "<details> disclosure widget has no accessible name (missing or empty <summary>)",
                        &node.node_id,
                    )
                    .with_role(node.role.clone())
                    .with_fix("Add a <summary> element with descriptive text inside the <details> element")
                    .with_rule_id(RULE_SUMMARY_NAME.axe_id).with_help_url(RULE_SUMMARY_NAME.help_url),
                );
            }
        }

        // ── meta-viewport-large ───────────────────────────────────────────
        // The root document node sometimes carries viewport meta info as a
        // property.  Check that maximum-scale is >= 3 (or absent).
        if matches!(role, "rootwebarea" | "document") {
            if let Some(viewport) = node.get_property_str("viewport") {
                if is_viewport_zoom_restricted_large(viewport) {
                    results.add_violation(
                        Violation::new(
                            RULE_META_VIEWPORT_LARGE.id,
                            RULE_META_VIEWPORT_LARGE.name,
                            RULE_META_VIEWPORT_LARGE.level,
                            RULE_META_VIEWPORT_LARGE.severity,
                            "Viewport meta tag limits zoom to less than 3x (maximum-scale < 3 or user-scalable=no)",
                            &node.node_id,
                        )
                        .with_fix("Set maximum-scale to at least 3 and do not set user-scalable=no in the viewport meta tag")
                        .with_rule_id(RULE_META_VIEWPORT_LARGE.axe_id).with_help_url(RULE_META_VIEWPORT_LARGE.help_url),
                    );
                } else {
                    results.passes += 1;
                }
            }
        }

        // ── target-size ───────────────────────────────────────────────────
        // Pixel dimensions are not available in the Chrome AX tree.
        // This rule is registered for filter/reporting compatibility but
        // cannot produce violations from tree data alone.
        // (A future CDP layout pass could supply bounding rects.)
    }

    results
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Returns true if the viewport content string restricts zoom below 3×.
/// Covers: `maximum-scale=1`, `maximum-scale=2.9`, `user-scalable=no`, `user-scalable=0`.
fn is_viewport_zoom_restricted_large(viewport: &str) -> bool {
    let content = viewport.to_lowercase();

    // user-scalable=no or user-scalable=0 always restricts
    if content.contains("user-scalable=no") || content.contains("user-scalable=0") {
        return true;
    }

    // Parse maximum-scale=<value>
    if let Some(pos) = content.find("maximum-scale=") {
        let after = &content[pos + "maximum-scale=".len()..];
        let value_str: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(val) = value_str.parse::<f64>() {
            return val < 3.0;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue, NameSource};

    fn node_with_props(
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

    fn simple_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
        node_with_props(id, role, name, None, vec![])
    }

    #[test]
    fn test_empty_heading_flagged() {
        let tree = AXTree::from_nodes(vec![simple_node("1", "heading", None)]);
        let r = check_wcag22_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("empty-heading")));
    }

    #[test]
    fn test_heading_with_text_passes() {
        let tree = AXTree::from_nodes(vec![simple_node("1", "heading", Some("About Us"))]);
        let r = check_wcag22_rules(&tree);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_label_title_only_flagged() {
        let node = node_with_props(
            "1", "textbox",
            Some("Search"),
            Some(NameSource::Title),
            vec![],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_wcag22_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("label-title-only")));
    }

    #[test]
    fn test_label_from_attribute_passes() {
        let node = node_with_props(
            "1", "textbox",
            Some("Search"),
            Some(NameSource::Attribute),
            vec![],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_wcag22_rules(&tree);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_details_without_summary_flagged() {
        let node = node_with_props("1", "group", None, None, vec![("htmlTag", "DETAILS")]);
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_wcag22_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("summary-name")));
    }

    #[test]
    fn test_viewport_maximum_scale_1_flagged() {
        let node = node_with_props(
            "root", "RootWebArea",
            Some("Page"),
            None,
            vec![("viewport", "width=device-width, maximum-scale=1")],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_wcag22_rules(&tree);
        assert!(r.violations.iter().any(|v| v.rule_id.as_deref() == Some("meta-viewport-large")));
    }

    #[test]
    fn test_viewport_no_restriction_passes() {
        let node = node_with_props(
            "root", "RootWebArea",
            Some("Page"),
            None,
            vec![("viewport", "width=device-width, initial-scale=1")],
        );
        let tree = AXTree::from_nodes(vec![node]);
        let r = check_wcag22_rules(&tree);
        assert!(!r.violations.iter().any(|v| v.rule_id.as_deref() == Some("meta-viewport-large")));
    }

    #[test]
    fn test_is_viewport_zoom_restricted_large() {
        assert!(is_viewport_zoom_restricted_large("width=device-width, maximum-scale=1"));
        assert!(is_viewport_zoom_restricted_large("maximum-scale=2.9"));
        assert!(is_viewport_zoom_restricted_large("user-scalable=no"));
        assert!(!is_viewport_zoom_restricted_large("width=device-width, initial-scale=1"));
        assert!(!is_viewport_zoom_restricted_large("maximum-scale=5"));
        assert!(!is_viewport_zoom_restricted_large("maximum-scale=3"));
    }
}
