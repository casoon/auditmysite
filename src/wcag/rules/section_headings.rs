//! WCAG 2.4.10 Section Headings
//!
//! Section headings are used to organize the content.
//! Level AAA - Helps users find content and navigate more easily.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.4.10
pub const SECTION_HEADINGS_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.10",
    name: "Section Headings",
    level: WcagLevel::AAA,
    severity: Severity::Minor,
    description: "Section headings are used to organize the content",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/section-headings.html",
};

/// Check for proper use of section headings
pub fn check_section_headings(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked = tree.len();

    // Count headings and sections
    let heading_count = count_headings(tree);
    let section_count = count_sections(tree);
    let article_count = count_articles(tree);
    let nav_count = count_navigation(tree);

    let total_sections = section_count + article_count + nav_count;

    // Check if there are sections but insufficient headings
    if total_sections > 0 && heading_count < total_sections {
        let violation = Violation::new(
            SECTION_HEADINGS_RULE.id,
            SECTION_HEADINGS_RULE.name,
            SECTION_HEADINGS_RULE.level,
            SECTION_HEADINGS_RULE.severity,
            format!(
                "Found {} sections but only {} headings - sections should have headings",
                total_sections, heading_count
            ),
            "page",
        )
        .with_fix("Add descriptive headings to each section/article element")
        .with_help_url(SECTION_HEADINGS_RULE.help_url);

        results.add_violation(violation);
    } else if total_sections > 0 {
        results.passes += 1;
    }

    // Check for large blocks of text without headings
    let paragraph_count = count_paragraphs(tree);
    if paragraph_count > 10 && heading_count < 3 {
        let violation = Violation::new(
            SECTION_HEADINGS_RULE.id,
            SECTION_HEADINGS_RULE.name,
            SECTION_HEADINGS_RULE.level,
            SECTION_HEADINGS_RULE.severity,
            format!(
                "Large amount of content ({} paragraphs) with insufficient headings ({})",
                paragraph_count, heading_count
            ),
            "page",
        )
        .with_fix("Break up long content with descriptive section headings")
        .with_help_url(SECTION_HEADINGS_RULE.help_url);

        results.add_violation(violation);
    }

    // Check for proper heading hierarchy
    let headings = get_heading_levels(tree);
    if !headings.is_empty() && has_heading_gaps(&headings) {
        let violation = Violation::new(
            SECTION_HEADINGS_RULE.id,
            SECTION_HEADINGS_RULE.name,
            SECTION_HEADINGS_RULE.level,
            Severity::Minor,
            "Heading hierarchy has gaps (e.g., h1 to h3 without h2)",
            "page",
        )
        .with_fix("Use consecutive heading levels (h1, h2, h3) without skipping")
        .with_help_url(SECTION_HEADINGS_RULE.help_url);

        results.add_violation(violation);
    }

    results
}

/// Count headings in the page
fn count_headings(tree: &AXTree) -> usize {
    tree.iter()
        .filter(|node| node.role.as_deref() == Some("heading"))
        .count()
}

/// Count section elements
fn count_sections(tree: &AXTree) -> usize {
    tree.iter()
        .filter(|node| {
            node.role
                .as_deref()
                .map(|r| r.to_lowercase() == "region")
                .unwrap_or(false)
        })
        .count()
}

/// Count article elements
fn count_articles(tree: &AXTree) -> usize {
    tree.iter()
        .filter(|node| {
            node.role
                .as_deref()
                .map(|r| r.to_lowercase() == "article")
                .unwrap_or(false)
        })
        .count()
}

/// Count navigation elements
fn count_navigation(tree: &AXTree) -> usize {
    tree.iter()
        .filter(|node| {
            node.role
                .as_deref()
                .map(|r| r.to_lowercase() == "navigation")
                .unwrap_or(false)
        })
        .count()
}

/// Count paragraphs
fn count_paragraphs(tree: &AXTree) -> usize {
    tree.iter()
        .filter(|node| {
            node.role
                .as_deref()
                .map(|r| r.to_lowercase() == "paragraph")
                .unwrap_or(false)
        })
        .count()
}

/// Get all heading levels from the tree
fn get_heading_levels(tree: &AXTree) -> Vec<u32> {
    tree.iter()
        .filter(|node| node.role.as_deref() == Some("heading"))
        .filter_map(|node| node.get_property_int("level").map(|l| l as u32))
        .collect()
}

/// Check if heading hierarchy has gaps
fn has_heading_gaps(levels: &[u32]) -> bool {
    if levels.is_empty() {
        return false;
    }

    let mut prev_level = 0u32;
    for &level in levels {
        if level > prev_level + 1 && prev_level > 0 {
            return true; // Gap detected (e.g., h1 to h3)
        }
        prev_level = prev_level.max(level);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXValue};

    fn create_node(id: &str, role: &str) -> AXNode {
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some(role.to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    fn create_heading(id: &str, level: i64) -> AXNode {
        let mut node = create_node(id, "heading");
        node.properties.push(AXProperty {
            name: "level".to_string(),
            value: AXValue::Int(level),
        });
        node
    }

    #[test]
    fn test_section_headings_rule_metadata() {
        assert_eq!(SECTION_HEADINGS_RULE.id, "2.4.10");
        assert_eq!(SECTION_HEADINGS_RULE.level, WcagLevel::AAA);
    }

    #[test]
    fn test_sections_with_headings() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "region"),
            create_heading("2", 1),
            create_node("3", "article"),
            create_heading("4", 2),
        ]);

        let results = check_section_headings(&tree);
        assert!(results
            .violations
            .iter()
            .all(|v| !v.message.contains("sections but only")));
    }

    #[test]
    fn test_sections_without_headings() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "region"),
            create_node("2", "region"),
            create_heading("3", 1),
        ]);

        let results = check_section_headings(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("sections but only")));
    }

    #[test]
    fn test_heading_gaps() {
        assert!(!has_heading_gaps(&[]));
        assert!(!has_heading_gaps(&[1, 2, 3]));
        assert!(has_heading_gaps(&[1, 3])); // Gap from h1 to h3
        assert!(has_heading_gaps(&[1, 2, 4])); // Gap from h2 to h4
        assert!(!has_heading_gaps(&[1, 1, 2, 2])); // Multiple same levels OK
    }
}
