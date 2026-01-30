//! WCAG 2.4.6 - Headings and Labels
//!
//! Headings and labels describe topic or purpose.
//! Also checks heading hierarchy (h1-h6 should not skip levels).

use crate::accessibility::{AXNode, AXTree, AXValue};
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.4.6
pub const RULE_META: RuleMetadata = RuleMetadata {
    id: "2.4.6",
    name: "Headings and Labels",
    level: WcagLevel::AA,
    severity: Severity::Moderate,
    description: "Headings and labels describe topic or purpose",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/headings-and-labels.html",
};

/// Check heading structure and labels
///
/// # Arguments
/// * `tree` - The accessibility tree to check
///
/// # Returns
/// Results with violations for heading issues
pub fn check_headings(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    // Get all headings
    let headings = tree.headings();
    results.nodes_checked = headings.len();

    // Check for empty headings
    for heading in &headings {
        if heading.ignored {
            continue;
        }

        check_empty_heading(heading, &mut results);
    }

    // Check heading hierarchy
    check_heading_hierarchy(&headings, &mut results);

    // Check for multiple h1s
    check_multiple_h1(&headings, &mut results);

    // Check for missing h1
    check_missing_h1(&headings, &mut results);

    results
}

/// Check for empty headings
fn check_empty_heading(heading: &AXNode, results: &mut WcagResults) {
    if !heading.has_name() {
        let level = get_heading_level(heading).unwrap_or(0);

        let violation = Violation::new(
            RULE_META.id,
            RULE_META.name,
            RULE_META.level,
            Severity::Serious,
            format!("Heading level {} is empty", level),
            &heading.node_id,
        )
        .with_role(heading.role.clone())
        .with_fix("Add descriptive text content to the heading")
        .with_help_url(RULE_META.help_url);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }
}

/// Check heading hierarchy (no skipped levels)
fn check_heading_hierarchy(headings: &[&AXNode], results: &mut WcagResults) {
    // Sort headings by their order in the tree (by node_id as approximation)
    let mut sorted_headings: Vec<_> = headings
        .iter()
        .filter(|h| !h.ignored)
        .filter_map(|h| {
            get_heading_level(h).map(|level| (level, *h))
        })
        .collect();

    // Simple ordering by node_id (numeric part)
    sorted_headings.sort_by(|a, b| {
        a.1.node_id.cmp(&b.1.node_id)
    });

    let mut prev_level: Option<u8> = None;

    for (level, heading) in sorted_headings {
        if let Some(prev) = prev_level {
            // Check if we skipped a level (e.g., h2 -> h4)
            if level > prev + 1 {
                let violation = Violation::new(
                    "1.3.1", // Info and Relationships
                    "Heading Hierarchy",
                    WcagLevel::A,
                    Severity::Moderate,
                    format!(
                        "Heading level skipped from h{} to h{} (should not skip levels)",
                        prev, level
                    ),
                    &heading.node_id,
                )
                .with_role(heading.role.clone())
                .with_name(heading.name.clone())
                .with_fix(format!(
                    "Use h{} instead of h{}, or add intermediate headings",
                    prev + 1,
                    level
                ))
                .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships.html");

                results.add_violation(violation);
            }
        }

        prev_level = Some(level);
    }
}

/// Check for multiple h1 elements
fn check_multiple_h1(headings: &[&AXNode], results: &mut WcagResults) {
    let h1_headings: Vec<_> = headings
        .iter()
        .filter(|h| !h.ignored)
        .filter(|h| get_heading_level(h) == Some(1))
        .collect();

    if h1_headings.len() > 1 {
        // Flag all but the first h1
        for (i, heading) in h1_headings.iter().enumerate().skip(1) {
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::Minor,
                format!(
                    "Multiple h1 elements found (this is h1 #{}, best practice is to have only one)",
                    i + 1
                ),
                &heading.node_id,
            )
            .with_role(heading.role.clone())
            .with_name(heading.name.clone())
            .with_fix("Consider using h2 or lower for secondary main headings")
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        }
    }
}

/// Check for missing h1
fn check_missing_h1(headings: &[&AXNode], results: &mut WcagResults) {
    let has_h1 = headings
        .iter()
        .any(|h| !h.ignored && get_heading_level(h) == Some(1));

    if !has_h1 && !headings.is_empty() {
        // We have headings but no h1 - this is a minor issue
        // We don't have a specific node to attach to, so we use the first heading
        if let Some(first_heading) = headings.first() {
            let violation = Violation::new(
                RULE_META.id,
                RULE_META.name,
                RULE_META.level,
                Severity::Minor,
                "Page is missing an h1 element (main heading)",
                &first_heading.node_id,
            )
            .with_fix("Add an h1 element as the main page heading")
            .with_help_url(RULE_META.help_url);

            results.add_violation(violation);
        }
    }
}

/// Get heading level from an AXNode
fn get_heading_level(node: &AXNode) -> Option<u8> {
    if node.role.as_deref() != Some("heading") {
        return None;
    }

    // Try to get level from properties
    node.properties
        .iter()
        .find(|p| p.name == "level")
        .and_then(|p| match &p.value {
            AXValue::Int(i) => Some((*i).clamp(1, 6) as u8),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXProperty;

    fn create_heading_node(id: &str, level: u8, name: Option<&str>) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("heading".to_string()),
            name: name.map(String::from),
            name_source: None,
            description: None,
            value: None,
            properties: vec![AXProperty {
                name: "level".to_string(),
                value: AXValue::Int(level as i64),
            }],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_valid_heading_hierarchy() {
        let nodes = vec![
            create_heading_node("1", 1, Some("Main Title")),
            create_heading_node("2", 2, Some("Section")),
            create_heading_node("3", 3, Some("Subsection")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_headings(&tree);

        // Should pass - no violations
        let hierarchy_violations: Vec<_> = results
            .violations
            .iter()
            .filter(|v| v.message.contains("skipped"))
            .collect();
        assert!(hierarchy_violations.is_empty());
    }

    #[test]
    fn test_skipped_heading_level() {
        let nodes = vec![
            create_heading_node("1", 1, Some("Main Title")),
            create_heading_node("2", 4, Some("Skipped to h4")), // Skipped h2, h3!
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_headings(&tree);

        assert!(results.violations.iter().any(|v| v.message.contains("skipped")));
    }

    #[test]
    fn test_empty_heading() {
        let nodes = vec![create_heading_node("1", 1, None)];
        let tree = AXTree::from_nodes(nodes);
        let results = check_headings(&tree);

        assert!(results.violations.iter().any(|v| v.message.contains("empty")));
    }

    #[test]
    fn test_multiple_h1() {
        let nodes = vec![
            create_heading_node("1", 1, Some("First H1")),
            create_heading_node("2", 1, Some("Second H1")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_headings(&tree);

        assert!(results.violations.iter().any(|v| v.message.contains("Multiple h1")));
    }

    #[test]
    fn test_missing_h1() {
        let nodes = vec![
            create_heading_node("1", 2, Some("Section")),
            create_heading_node("2", 3, Some("Subsection")),
        ];
        let tree = AXTree::from_nodes(nodes);
        let results = check_headings(&tree);

        assert!(results.violations.iter().any(|v| v.message.contains("missing an h1")));
    }
}
