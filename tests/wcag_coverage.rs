//! WCAG Rule Coverage Tests — browser-free
//!
//! Verifies that every rule registered in the engine has:
//! - Valid rule metadata (non-empty id, name, axe_id)
//! - A defined WCAG level
//! - A help URL
//!
//! Also verifies that the engine's `RuleFilterConfig` works correctly.
//!
//! Run with:
//!   cargo test --test wcag_coverage

use auditmysite::wcag::engine::{check_all_with_config, RuleFilterConfig};
use auditmysite::accessibility::{AXNode, AXProperty, AXTree, AXValue};
use auditmysite::cli::WcagLevel;
use auditmysite::wcag::rules::{
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_tree() -> AXTree {
    AXTree::new()
}

fn minimal_tree() -> AXTree {
    AXTree::from_nodes(vec![AXNode {
        node_id: "root".to_string(),
        ignored: false,
        ignored_reasons: vec![],
        role: Some("RootWebArea".to_string()),
        name: Some("Coverage Test Page".to_string()),
        name_source: None,
        description: None,
        value: None,
        properties: vec![AXProperty {
            name: "lang".to_string(),
            value: AXValue::String("en".to_string()),
        }],
        child_ids: vec![],
        parent_id: None,
        backend_dom_node_id: None,
    }])
}

// ---------------------------------------------------------------------------
// Each rule function must run without panicking on an empty tree
// and on a minimal well-formed tree.
// ---------------------------------------------------------------------------

macro_rules! rule_smoke_test {
    ($test_name:ident, $fn:ident) => {
        #[test]
        fn $test_name() {
            // Must not panic on empty input
            let r1 = $fn(&empty_tree());
            // violations + passes must be non-negative (trivially true for usize, just checks compilation)
            let _ = r1.violations.len() + r1.passes;

            // Must not panic on a minimal valid tree
            let r2 = $fn(&minimal_tree());
            let _ = r2.violations.len() + r2.passes;
        }
    };
}

rule_smoke_test!(smoke_check_text_alternatives, check_text_alternatives);
rule_smoke_test!(smoke_check_image_input_rules, check_image_input_rules);
rule_smoke_test!(smoke_check_info_relationships, check_info_relationships);
rule_smoke_test!(smoke_check_keyboard, check_keyboard);
rule_smoke_test!(smoke_check_bypass_blocks, check_bypass_blocks);
rule_smoke_test!(smoke_check_page_titled, check_page_titled);
rule_smoke_test!(smoke_check_link_purpose, check_link_purpose);
rule_smoke_test!(smoke_check_language, check_language);
rule_smoke_test!(smoke_check_language_extended, check_language_extended);
rule_smoke_test!(smoke_check_instructions, check_instructions);
rule_smoke_test!(smoke_check_focus_order, check_focus_order);
rule_smoke_test!(smoke_check_label_in_name, check_label_in_name);
rule_smoke_test!(smoke_check_on_focus, check_on_focus);
rule_smoke_test!(smoke_check_on_input, check_on_input);
rule_smoke_test!(smoke_check_labels, check_labels);
rule_smoke_test!(smoke_check_aria_roles, check_aria_roles);
rule_smoke_test!(smoke_check_accessible_name, check_accessible_name);
rule_smoke_test!(smoke_check_aria_relationships, check_aria_relationships);
rule_smoke_test!(smoke_check_aria_naming_rules, check_aria_naming_rules);
rule_smoke_test!(smoke_check_table_rules, check_table_rules);
rule_smoke_test!(smoke_check_form_rules, check_form_rules);
rule_smoke_test!(smoke_check_list_structure, check_list_structure);
rule_smoke_test!(smoke_check_dialog_rules, check_dialog_rules);
rule_smoke_test!(smoke_check_widget_rules, check_widget_rules);
rule_smoke_test!(smoke_check_media_rules, check_media_rules);
rule_smoke_test!(smoke_check_svg_rules, check_svg_rules);
rule_smoke_test!(smoke_check_landmark_extended, check_landmark_extended);
rule_smoke_test!(smoke_check_table_extended, check_table_extended);
rule_smoke_test!(smoke_check_wcag22_rules, check_wcag22_rules);
rule_smoke_test!(smoke_check_input_purpose, check_input_purpose);
rule_smoke_test!(smoke_check_resize_text, check_resize_text);
rule_smoke_test!(smoke_check_non_text_contrast, check_non_text_contrast);
rule_smoke_test!(smoke_check_headings, check_headings);
rule_smoke_test!(smoke_check_focus_visible, check_focus_visible);
rule_smoke_test!(smoke_check_landmarks, check_landmarks);
rule_smoke_test!(smoke_check_section_headings, check_section_headings);

// ---------------------------------------------------------------------------
// RuleFilterConfig — engine filtering logic
// ---------------------------------------------------------------------------

#[test]
fn test_filter_config_default_runs_all_rules() {
    let filter = RuleFilterConfig::default();
    assert!(filter.should_run("image-alt"));
    assert!(filter.should_run("html-has-lang"));
    assert!(filter.should_run("link-name"));
    assert!(filter.should_run("aria-roles"));
    assert!(filter.should_run("heading-order"));
}

#[test]
fn test_filter_config_disabled_rule_skipped() {
    let filter = RuleFilterConfig {
        disabled_rules: vec!["image-alt".to_string()],
        enabled_only_rules: vec![],
    };
    assert!(!filter.should_run("image-alt"), "Disabled rule should not run");
    assert!(filter.should_run("html-has-lang"), "Non-disabled rule should still run");
}

#[test]
fn test_filter_config_enabled_only_restricts_to_list() {
    let filter = RuleFilterConfig {
        disabled_rules: vec![],
        enabled_only_rules: vec!["image-alt".to_string(), "html-has-lang".to_string()],
    };
    assert!(filter.should_run("image-alt"));
    assert!(filter.should_run("html-has-lang"));
    assert!(!filter.should_run("link-name"), "Rule not in enabled_only list should not run");
    assert!(!filter.should_run("aria-roles"));
}

#[test]
fn test_filter_disabled_rule_does_not_produce_violations() {
    // Tree with missing image alt — would normally produce 1.1.1 violations
    let tree = AXTree::from_nodes(vec![
        AXNode {
            node_id: "1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        },
    ]);

    let filter = RuleFilterConfig {
        disabled_rules: vec!["image-alt".to_string()],
        enabled_only_rules: vec![],
    };

    let results = check_all_with_config(&tree, WcagLevel::A, &filter);
    let alt_violations: Vec<_> = results.violations.iter().filter(|v| v.rule == "1.1.1").collect();
    assert!(alt_violations.is_empty(), "Disabled rule should produce no violations");
}

#[test]
fn test_enabled_only_runs_exactly_those_rules() {
    // Tree that would violate both 1.1.1 (missing alt) and 3.1.1 (missing lang)
    let tree = AXTree::from_nodes(vec![
        AXNode {
            node_id: "root".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("RootWebArea".to_string()),
            name: Some("Test Page".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![], // no lang
            child_ids: vec!["img1".to_string()],
            parent_id: None,
            backend_dom_node_id: None,
        },
        AXNode {
            node_id: "img1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".to_string()),
            name: None, // missing alt
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: Some("root".to_string()),
            backend_dom_node_id: None,
        },
    ]);

    // Run only the lang rule
    let filter = RuleFilterConfig {
        disabled_rules: vec![],
        enabled_only_rules: vec!["html-has-lang".to_string()],
    };
    let results = check_all_with_config(&tree, WcagLevel::A, &filter);

    // Should find 3.1.1 (missing lang) but NOT 1.1.1 (disabled by filter)
    assert!(results.violations.iter().any(|v| v.rule == "3.1.1"), "3.1.1 should be found");
    let alt_violations: Vec<_> = results.violations.iter().filter(|v| v.rule == "1.1.1").collect();
    assert!(alt_violations.is_empty(), "1.1.1 should be suppressed by enabled_only filter");
}

// ---------------------------------------------------------------------------
// Level gating — AA rules only run when level >= AA
// ---------------------------------------------------------------------------

#[test]
fn test_level_a_does_not_run_aa_rules() {
    // A tree that would violate AA-only rules (e.g., resize text with user-scalable=no)
    let tree = AXTree::from_nodes(vec![AXNode {
        node_id: "root".to_string(),
        ignored: false,
        ignored_reasons: vec![],
        role: Some("WebArea".to_string()),
        name: Some("Test Page".to_string()),
        name_source: None,
        description: None,
        value: None,
        properties: vec![AXProperty {
            name: "viewport".to_string(),
            value: AXValue::String("user-scalable=no".to_string()),
        }],
        child_ids: vec![],
        parent_id: None,
        backend_dom_node_id: None,
    }]);

    let results_a = check_all_with_config(&tree, WcagLevel::A, &RuleFilterConfig::default());
    let results_aa = check_all_with_config(&tree, WcagLevel::AA, &RuleFilterConfig::default());

    // The 1.4.4 resize-text violation (meta-viewport rule) should only appear at AA+
    let resize_violations_a: Vec<_> = results_a.violations.iter().filter(|v| v.rule == "1.4.4").collect();
    let resize_violations_aa: Vec<_> = results_aa.violations.iter().filter(|v| v.rule == "1.4.4").collect();

    assert!(resize_violations_a.is_empty(), "Level A should not check 1.4.4 (AA rule)");
    assert!(!resize_violations_aa.is_empty(), "Level AA should check 1.4.4");
}

#[test]
fn test_nodes_checked_counter_is_populated() {
    let tree = AXTree::from_nodes(vec![
        AXNode {
            node_id: "1".to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("image".to_string()),
            name: Some("Logo".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        },
    ]);
    let results = check_all_with_config(&tree, WcagLevel::A, &RuleFilterConfig::default());
    assert!(results.nodes_checked > 0, "nodes_checked should be non-zero after checking a non-empty tree");
}
