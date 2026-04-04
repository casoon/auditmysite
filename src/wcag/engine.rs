//! WCAG Rule Engine - Executes all WCAG rules against an AXTree
//!
//! The engine loads all rules and runs them against the accessibility tree.

use tracing::{debug, info};

use super::rules::{
    check_accessible_name, check_aria_naming_rules, check_aria_relationships, check_aria_roles,
    check_bypass_blocks, check_dialog_rules, check_focus_order, check_focus_visible,
    check_form_rules, check_headings, check_image_input_rules, check_info_relationships,
    check_input_purpose, check_instructions, check_keyboard, check_label_in_name, check_labels,
    check_landmark_extended, check_landmarks, check_language, check_language_extended,
    check_link_purpose, check_list_structure, check_media_rules, check_non_text_contrast,
    check_on_focus, check_on_input, check_page_titled, check_resize_text, check_section_headings,
    check_svg_rules, check_table_extended, check_table_rules, check_text_alternatives,
    check_wcag22_rules, check_widget_rules,
};
use super::types::WcagResults;
use crate::accessibility::AXTree;
use crate::cli::WcagLevel;

/// Rule filtering configuration (subset of cli::config::RulesConfig for the engine)
#[derive(Debug, Clone, Default)]
pub struct RuleFilterConfig {
    /// axe_ids of rules to disable
    pub disabled_rules: Vec<String>,
    /// If non-empty, only run these rules (by axe_id)
    pub enabled_only_rules: Vec<String>,
}

impl RuleFilterConfig {
    /// Returns true if the rule with the given axe_id should be run
    pub fn should_run(&self, axe_id: &str) -> bool {
        if !self.enabled_only_rules.is_empty() {
            return self.enabled_only_rules.iter().any(|r| r == axe_id);
        }
        !self.disabled_rules.iter().any(|r| r == axe_id)
    }
}

/// Run all WCAG checks against an AXTree
///
/// # Arguments
/// * `tree` - The accessibility tree to check
/// * `level` - The WCAG conformance level to check against
///
/// # Returns
/// Results containing all violations found
pub fn check_all(tree: &AXTree, level: WcagLevel) -> WcagResults {
    check_all_with_config(tree, level, &RuleFilterConfig::default())
}

/// Run all WCAG checks against an AXTree with optional rule filtering
///
/// # Arguments
/// * `tree` - The accessibility tree to check
/// * `level` - The WCAG conformance level to check against
/// * `filter` - Rule filter configuration (disabled/enabled_only rules)
///
/// # Returns
/// Results containing all violations found
pub fn check_all_with_config(
    tree: &AXTree,
    level: WcagLevel,
    filter: &RuleFilterConfig,
) -> WcagResults {
    info!("Running WCAG checks at level {}", level);

    let mut results = WcagResults::new();
    results.nodes_checked = tree.len();

    // Run Level A rules
    debug!("Running Level A rules...");
    run_level_a_rules(tree, &mut results, filter);

    // Run Level AA rules if requested
    if matches!(level, WcagLevel::AA | WcagLevel::AAA) {
        debug!("Running Level AA rules...");
        run_level_aa_rules(tree, &mut results, filter);
    }

    // Run Level AAA rules if requested
    if level == WcagLevel::AAA {
        debug!("Running Level AAA rules...");
        run_level_aaa_rules(tree, &mut results, filter);
    }

    info!(
        "WCAG check complete: {} violations found",
        results.violations.len()
    );

    results
}

/// Merge rule results only if the filter allows the given axe_id
macro_rules! run_if_allowed {
    ($filter:expr, $axe_id:expr, $check_fn:expr, $results:expr, $tree:expr) => {
        if $filter.should_run($axe_id) {
            let rule_results = $check_fn($tree);
            $results.merge(rule_results);
        }
    };
}

/// Run all Level A rules
fn run_level_a_rules(tree: &AXTree, results: &mut WcagResults, filter: &RuleFilterConfig) {
    // 1.1.1 Non-text Content (Level A)
    run_if_allowed!(filter, "image-alt", check_text_alternatives, results, tree);
    // 1.1.1 Area / input[type=image] / object alternatives (Level A)
    run_if_allowed!(filter, "area-alt", check_image_input_rules, results, tree);

    // 1.3.1 Info and Relationships (Level A)
    run_if_allowed!(
        filter,
        "definition-list",
        check_info_relationships,
        results,
        tree
    );

    // 2.1.1 Keyboard (Level A)
    run_if_allowed!(filter, "keyboard", check_keyboard, results, tree);

    // 2.4.1 Bypass Blocks (Level A)
    run_if_allowed!(filter, "bypass", check_bypass_blocks, results, tree);

    // 2.4.2 Page Titled (Level A)
    run_if_allowed!(filter, "document-title", check_page_titled, results, tree);

    // 2.4.4 Link Purpose (In Context) (Level A)
    run_if_allowed!(filter, "link-name", check_link_purpose, results, tree);

    // 3.1.1 Language of Page (Level A)
    run_if_allowed!(filter, "html-has-lang", check_language, results, tree);
    // 3.1.1 Valid lang attribute + xml:lang mismatch (Level A)
    run_if_allowed!(filter, "valid-lang", check_language_extended, results, tree);

    // 3.3.2 Labels or Instructions (Level A)
    run_if_allowed!(filter, "label", check_instructions, results, tree);

    // 2.4.3 Focus Order (Level A)
    run_if_allowed!(
        filter,
        "focus-order-semantics",
        check_focus_order,
        results,
        tree
    );

    // 2.5.3 Label in Name (Level A)
    run_if_allowed!(
        filter,
        "label-content-name-mismatch",
        check_label_in_name,
        results,
        tree
    );

    // 3.2.1 On Focus (Level A)
    run_if_allowed!(
        filter,
        "focus-no-context-change",
        check_on_focus,
        results,
        tree
    );

    // 3.2.2 On Input (Level A)
    run_if_allowed!(
        filter,
        "input-no-context-change",
        check_on_input,
        results,
        tree
    );

    // 4.1.2 Name, Role, Value (Level A)
    run_if_allowed!(filter, "label", check_labels, results, tree);

    // 4.1.2 ARIA Role Validity (Level A)
    run_if_allowed!(filter, "aria-roles", check_aria_roles, results, tree);

    // 4.1.2 Accessible Name Extended (Level A)
    run_if_allowed!(filter, "aria-label", check_accessible_name, results, tree);

    // 4.1.2 ARIA Relationship Attributes (Level A)
    run_if_allowed!(
        filter,
        "aria-valid-attr",
        check_aria_relationships,
        results,
        tree
    );

    // 4.1.2 / 1.1.1 ARIA Role-Specific Naming Rules (Level A)
    run_if_allowed!(
        filter,
        "aria-command-name",
        check_aria_naming_rules,
        results,
        tree
    );

    // 1.3.1 / 4.1.2 Table Rules (Level A) - P1
    run_if_allowed!(
        filter,
        "table-duplicate-name",
        check_table_rules,
        results,
        tree
    );

    // 1.3.1 / 3.3.1 / 3.3.2 Form Rules (Level A) - P1
    run_if_allowed!(
        filter,
        "form-field-multiple-labels",
        check_form_rules,
        results,
        tree
    );

    // 1.3.1 List Structure (Level A) - P1
    run_if_allowed!(filter, "list", check_list_structure, results, tree);

    // 4.1.2 / 2.4.3 Dialog Rules (Level A) - P1
    run_if_allowed!(filter, "dialog-name", check_dialog_rules, results, tree);

    // 4.1.2 / 2.1.1 Widget Rules (Level A) - P2
    run_if_allowed!(
        filter,
        "aria-required-children",
        check_widget_rules,
        results,
        tree
    );

    // 1.2.1 / 1.1.1 Media Rules (Level A) - P2
    run_if_allowed!(filter, "video-caption", check_media_rules, results, tree);

    // 1.1.1 SVG Rules (Level A) - P2
    run_if_allowed!(filter, "svg-img-alt", check_svg_rules, results, tree);

    // 2.4.1 Extended Landmark Rules (Level A) - P1
    run_if_allowed!(
        filter,
        "landmark-no-duplicate-banner",
        check_landmark_extended,
        results,
        tree
    );

    // 1.3.1 Extended Table Header Rules (Level A) - P1
    run_if_allowed!(
        filter,
        "td-headers-attr",
        check_table_extended,
        results,
        tree
    );

    // Best Practice / WCAG 2.2 (Level A subset)
    run_if_allowed!(filter, "empty-heading", check_wcag22_rules, results, tree);
}

/// Run all Level AA rules
fn run_level_aa_rules(tree: &AXTree, results: &mut WcagResults, filter: &RuleFilterConfig) {
    // Note: 1.4.3 Contrast (Minimum) requires CDP page access and is
    // handled separately in the pipeline via ContrastRule::check_with_page

    // 1.3.5 Identify Input Purpose (Level AA)
    run_if_allowed!(
        filter,
        "autocomplete-valid",
        check_input_purpose,
        results,
        tree
    );

    // 1.4.4 Resize Text (Level AA)
    run_if_allowed!(filter, "meta-viewport", check_resize_text, results, tree);

    // 1.4.4 Viewport Large Scale restriction (Level AA)
    run_if_allowed!(
        filter,
        "meta-viewport-large",
        check_wcag22_rules,
        results,
        tree
    );

    // 1.4.11 Non-text Contrast (Level AA)
    run_if_allowed!(
        filter,
        "non-text-contrast",
        check_non_text_contrast,
        results,
        tree
    );

    // 2.4.6 Headings and Labels (Level AA)
    run_if_allowed!(filter, "heading-order", check_headings, results, tree);

    // 2.4.7 Focus Visible (Level AA)
    run_if_allowed!(filter, "focus-visible", check_focus_visible, results, tree);

    // 2.4.1 / 1.3.6 Landmark Regions (Level AA)
    run_if_allowed!(filter, "landmark-one-main", check_landmarks, results, tree);
}

/// Run all Level AAA rules
fn run_level_aaa_rules(tree: &AXTree, results: &mut WcagResults, filter: &RuleFilterConfig) {
    // Note: 1.4.6 Contrast (Enhanced) requires CDP page access and is
    // handled separately in the pipeline via ContrastRule::check_with_page

    // 2.4.10 Section Headings (Level AAA)
    run_if_allowed!(
        filter,
        "heading-order",
        check_section_headings,
        results,
        tree
    );

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
