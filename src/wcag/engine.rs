//! WCAG Rule Engine - Executes all WCAG rules against an AXTree
//!
//! The engine loads all rules and runs them against the accessibility tree.

use tracing::{debug, info};

use super::rules::{
    check_bypass_blocks, check_headings, check_info_relationships, check_instructions,
    check_keyboard, check_labels, check_language, check_link_purpose, check_page_titled,
    check_section_headings, check_text_alternatives, ContrastRule,
};
use super::types::WcagResults;
use crate::accessibility::AXTree;
use crate::cli::WcagLevel;

/// Run all WCAG checks against an AXTree
///
/// # Arguments
/// * `tree` - The accessibility tree to check
/// * `level` - The WCAG conformance level to check against
///
/// # Returns
/// Results containing all violations found
pub fn check_all(tree: &AXTree, level: WcagLevel) -> WcagResults {
    info!("Running WCAG checks at level {}", level);

    let mut results = WcagResults::new();
    results.nodes_checked = tree.len();

    // Run Level A rules
    debug!("Running Level A rules...");
    run_level_a_rules(tree, &mut results);

    // Run Level AA rules if requested
    if matches!(level, WcagLevel::AA | WcagLevel::AAA) {
        debug!("Running Level AA rules...");
        run_level_aa_rules(tree, &mut results);
    }

    // Run Level AAA rules if requested
    if level == WcagLevel::AAA {
        debug!("Running Level AAA rules...");
        run_level_aaa_rules(tree, &mut results);
    }

    info!(
        "WCAG check complete: {} violations found, score: {}",
        results.violations.len(),
        results.calculate_score()
    );

    results
}

/// Run all Level A rules
fn run_level_a_rules(tree: &AXTree, results: &mut WcagResults) {
    // 1.1.1 Non-text Content (Level A)
    let alt_results = check_text_alternatives(tree);
    results.merge(alt_results);

    // 1.3.1 Info and Relationships (Level A)
    let info_results = check_info_relationships(tree);
    results.merge(info_results);

    // 2.1.1 Keyboard (Level A)
    let keyboard_results = check_keyboard(tree);
    results.merge(keyboard_results);

    // 2.4.1 Bypass Blocks (Level A)
    let bypass_results = check_bypass_blocks(tree);
    results.merge(bypass_results);

    // 2.4.2 Page Titled (Level A)
    let page_titled_results = check_page_titled(tree);
    results.merge(page_titled_results);

    // 2.4.4 Link Purpose (In Context) (Level A)
    let link_results = check_link_purpose(tree);
    results.merge(link_results);

    // 3.1.1 Language of Page (Level A)
    let language_results = check_language(tree);
    results.merge(language_results);

    // 3.3.2 Labels or Instructions (Level A)
    let instructions_results = check_instructions(tree);
    results.merge(instructions_results);

    // 4.1.2 Name, Role, Value (Level A)
    let label_results = check_labels(tree);
    results.merge(label_results);
}

/// Run all Level AA rules
fn run_level_aa_rules(tree: &AXTree, results: &mut WcagResults) {
    // 1.4.3 Contrast (Minimum) (Level AA)
    let contrast_violations = ContrastRule::check(tree, WcagLevel::AA);
    results.violations.extend(contrast_violations);

    // 2.4.6 Headings and Labels (Level AA)
    let heading_results = check_headings(tree);
    results.merge(heading_results);
}

/// Run all Level AAA rules
fn run_level_aaa_rules(tree: &AXTree, results: &mut WcagResults) {
    // 1.4.6 Contrast (Enhanced) - would need 7:1 ratio
    // This is handled by ContrastRule with WcagLevel::AAA parameter
    let contrast_violations = ContrastRule::check(tree, WcagLevel::AAA);
    results.violations.extend(contrast_violations);

    // 2.4.10 Section Headings (Level AAA)
    let section_results = check_section_headings(tree);
    results.merge(section_results);

    debug!("Level AAA rules executed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXTree};

    fn create_test_tree() -> AXTree {
        let nodes = vec![
            AXNode {
                node_id: "1".to_string(),
                ignored: false,
                ignored_reasons: vec![],
                role: Some("WebArea".to_string()),
                name: Some("Test Page".to_string()),
                name_source: None,
                description: None,
                value: None,
                properties: vec![],
                child_ids: vec!["2".to_string()],
                parent_id: None,
                backend_dom_node_id: None,
            },
            AXNode {
                node_id: "2".to_string(),
                ignored: false,
                ignored_reasons: vec![],
                role: Some("image".to_string()),
                name: None, // Missing alt text!
                name_source: None,
                description: None,
                value: None,
                properties: vec![],
                child_ids: vec![],
                parent_id: Some("1".to_string()),
                backend_dom_node_id: None,
            },
        ];

        AXTree::from_nodes(nodes)
    }

    #[test]
    fn test_check_all_level_a() {
        let tree = create_test_tree();
        let results = check_all(&tree, WcagLevel::A);

        // Should find the missing alt text
        assert!(!results.violations.is_empty());
        assert!(results.violations.iter().any(|v| v.rule == "1.1.1"));
    }

    #[test]
    fn test_check_all_level_aa() {
        let tree = create_test_tree();
        let results = check_all(&tree, WcagLevel::AA);

        // Should run both A and AA rules
        assert!(!results.violations.is_empty());
    }
}
