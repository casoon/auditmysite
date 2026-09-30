//! WCAG 2.4.1 Bypass Blocks
//!
//! Provides a mechanism to bypass blocks of content that are repeated.
//! Level A - Important for keyboard users to skip navigation.
//!
//! Sprunglink und main-Landmark prüft seit #690 der geteilte Bestand
//! (`keyboard/skip-link-missing`, `landmarks/main-missing`, siehe
//! `wcag::shared`). Die frühere Sammelmeldung „weder Sprunglink noch main"
//! entfällt damit: Sie war nur die Verknüpfung genau dieser beiden Prüfungen,
//! und fehlen beide, melden die zwei geteilten Kennungen je ihren Teil.
//! Ebenso die Positivsignale „Sprunglink erkannt" und „main erkannt".
//!
//! Hier bleibt, was der geteilte Bestand nicht kennt: eine Seite ganz ohne
//! Überschriften (`headings/h1-missing` feuert nur, wenn es überhaupt
//! Überschriften gibt).

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 2.4.1
pub(super) const BYPASS_BLOCKS_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.1",
    name: "Bypass Blocks",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "A mechanism is available to bypass blocks of content that are repeated",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/bypass-blocks.html",
    axe_id: "bypass",
    tags: &["wcag2a", "wcag241", "cat.keyboard"],
};

/// Check that the page offers headings to navigate by.
pub fn check_bypass_blocks(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked = tree.len();

    let heading_count = count_headings(tree);
    if heading_count == 0 {
        let violation = Violation::new(
            BYPASS_BLOCKS_RULE.id,
            BYPASS_BLOCKS_RULE.name,
            BYPASS_BLOCKS_RULE.level,
            BYPASS_BLOCKS_RULE.severity,
            "No headings found for content navigation",
            "page",
        )
        .with_fix("Add headings (h1-h6) to structure your content")
        .with_help_url(BYPASS_BLOCKS_RULE.help_url)
        .with_rule_id(BYPASS_BLOCKS_RULE.axe_id);

        results.add_violation(violation);
    } else {
        results.passes += 1;
    }

    results
}

/// Count headings in the page
fn count_headings(tree: &AXTree) -> usize {
    tree.iter()
        .filter(|node| node.role.as_deref() == Some("heading"))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::AXNode;

    fn create_node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_bypass_blocks_rule_metadata() {
        assert_eq!(BYPASS_BLOCKS_RULE.id, "2.4.1");
        assert_eq!(BYPASS_BLOCKS_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_page_with_headings_passes() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "main", None),
            create_node("2", "heading", Some("Page Title")),
        ]);

        let results = check_bypass_blocks(&tree);
        assert!(results.violations.is_empty(), "{:?}", results.violations);
    }

    #[test]
    fn test_page_without_headings_flagged() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "generic", None),
            create_node("2", "paragraph", Some("Some text")),
        ]);

        let results = check_bypass_blocks(&tree);
        assert!(results
            .violations
            .iter()
            .any(|v| v.message.contains("No headings found")));
    }

    /// Sprunglink und main-Landmark sind Sache des geteilten Bestands (#690);
    /// diese Regel meldet ihr Fehlen nicht noch einmal.
    #[test]
    fn test_missing_skip_link_and_main_not_reported_here() {
        let tree = AXTree::from_nodes(vec![
            create_node("1", "link", Some("Home")),
            create_node("2", "heading", Some("Page Title")),
        ]);

        let results = check_bypass_blocks(&tree);
        assert!(results.violations.is_empty(), "{:?}", results.violations);
    }
}
