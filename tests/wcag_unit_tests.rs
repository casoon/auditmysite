//! WCAG Rule Unit Tests — browser-free
//!
//! These tests exercise the WCAG rule logic directly against in-memory AXTree
//! structures. No Chrome, no CDP, no network required.
//!
//! Run with:
//!   cargo test --test wcag_unit_tests

use auditmysite::accessibility::{AXNode, AXProperty, AXTree, AXValue};
use auditmysite::cli::WcagLevel;
use auditmysite::wcag::engine::check_all;
use auditmysite::wcag::rules::{
    check_aria_roles, check_headings, check_language, check_link_purpose, check_page_titled,
    check_resize_text, check_text_alternatives, Color, ContrastRule,
};

// ---------------------------------------------------------------------------
// Helper constructors
// ---------------------------------------------------------------------------

fn node(id: &str, role: &str, name: Option<&str>) -> AXNode {
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

fn node_with_parent(id: &str, role: &str, name: Option<&str>, parent: &str) -> AXNode {
    let mut n = node(id, role, name);
    n.parent_id = Some(parent.to_string());
    n
}

fn node_with_children(id: &str, role: &str, name: Option<&str>, children: Vec<&str>) -> AXNode {
    let mut n = node(id, role, name);
    n.child_ids = children.into_iter().map(String::from).collect();
    n
}

fn heading(id: &str, level: u8, name: Option<&str>) -> AXNode {
    let mut n = node(id, "heading", name);
    n.properties.push(AXProperty {
        name: "level".to_string(),
        value: AXValue::Int(level as i64),
    });
    n
}

fn doc_with_lang(id: &str, lang: Option<&str>) -> AXNode {
    let mut n = node(id, "RootWebArea", Some("Test Page"));
    if let Some(l) = lang {
        n.properties.push(AXProperty {
            name: "lang".to_string(),
            value: AXValue::String(l.to_string()),
        });
    }
    n
}

fn root_with_viewport(viewport: &str) -> AXNode {
    let mut n = node("root", "WebArea", Some("Test Page"));
    n.properties.push(AXProperty {
        name: "viewport".to_string(),
        value: AXValue::String(viewport.to_string()),
    });
    n
}

// ---------------------------------------------------------------------------
// 1.1.1 Non-text Content — check_text_alternatives
// ---------------------------------------------------------------------------

#[test]
fn test_111_image_with_alt_passes() {
    let tree = AXTree::from_nodes(vec![node("1", "image", Some("Company logo"))]);
    let results = check_text_alternatives(&tree);
    assert_eq!(results.violations.len(), 0);
    assert_eq!(results.passes, 1);
}

#[test]
fn test_111_image_without_alt_is_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "image", None)]);
    let results = check_text_alternatives(&tree);
    assert_eq!(results.violations.len(), 1);
    assert_eq!(results.violations[0].rule, "1.1.1");
    assert!(results.violations[0]
        .message
        .contains("missing alternative text"));
}

#[test]
fn test_111_ignored_image_not_flagged() {
    let mut n = node("1", "image", None);
    n.ignored = true;
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_text_alternatives(&tree);
    assert_eq!(results.violations.len(), 0);
}

#[test]
fn test_111_multiple_images_partial_alt() {
    let tree = AXTree::from_nodes(vec![
        node("1", "image", Some("Logo")),
        node("2", "image", None),
        node("3", "image", Some("Banner")),
        node("4", "image", None),
    ]);
    let results = check_text_alternatives(&tree);
    assert_eq!(results.violations.len(), 2);
    assert_eq!(results.passes, 2);
}

#[test]
fn test_111_whitespace_only_name_treated_as_missing() {
    let tree = AXTree::from_nodes(vec![node("1", "image", Some("   "))]);
    let results = check_text_alternatives(&tree);
    // Whitespace-only name is not a valid accessible name
    assert_eq!(results.violations.len(), 1);
}

// ---------------------------------------------------------------------------
// 2.4.2 Page Titled — check_page_titled
// ---------------------------------------------------------------------------

#[test]
fn test_242_page_with_good_title_passes() {
    let tree = AXTree::from_nodes(vec![node(
        "1",
        "RootWebArea",
        Some("Shopping Cart - Example Store"),
    )]);
    let results = check_page_titled(&tree);
    assert!(results.violations.is_empty());
    assert_eq!(results.passes, 1);
}

#[test]
fn test_242_page_without_title_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "RootWebArea", None)]);
    let results = check_page_titled(&tree);
    assert!(!results.violations.is_empty());
    assert_eq!(results.violations[0].rule, "2.4.2");
}

#[test]
fn test_242_generic_title_untitled_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "RootWebArea", Some("Untitled"))]);
    let results = check_page_titled(&tree);
    assert!(!results.violations.is_empty());
}

#[test]
fn test_242_generic_title_home_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "RootWebArea", Some("home"))]);
    let results = check_page_titled(&tree);
    assert!(!results.violations.is_empty());
}

#[test]
fn test_242_meaningful_title_passes() {
    let tree = AXTree::from_nodes(vec![node(
        "1",
        "RootWebArea",
        Some("Product Details - My Store"),
    )]);
    let results = check_page_titled(&tree);
    assert!(results.violations.is_empty());
}

// ---------------------------------------------------------------------------
// 2.4.4 Link Purpose — check_link_purpose
// ---------------------------------------------------------------------------

#[test]
fn test_244_empty_link_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "link", None)]);
    let results = check_link_purpose(&tree);
    assert!(!results.violations.is_empty());
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("no accessible text")));
}

#[test]
fn test_244_generic_click_here_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "link", Some("click here"))]);
    let results = check_link_purpose(&tree);
    assert!(!results.violations.is_empty());
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("generic text")));
}

#[test]
fn test_244_generic_read_more_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "link", Some("read more"))]);
    let results = check_link_purpose(&tree);
    assert!(!results.violations.is_empty());
}

#[test]
fn test_244_url_as_link_text_flagged() {
    let tree = AXTree::from_nodes(vec![node(
        "1",
        "link",
        Some("https://example.com/long/path"),
    )]);
    let results = check_link_purpose(&tree);
    assert!(!results.violations.is_empty());
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("raw URL")));
}

#[test]
fn test_244_descriptive_link_passes() {
    let tree = AXTree::from_nodes(vec![node(
        "1",
        "link",
        Some("View our accessibility statement"),
    )]);
    let results = check_link_purpose(&tree);
    assert!(results.violations.is_empty());
    assert_eq!(results.passes, 1);
}

#[test]
fn test_244_multiple_links_mixed() {
    let tree = AXTree::from_nodes(vec![
        node("1", "link", Some("Download the annual report PDF")),
        node("2", "link", None),
        node("3", "link", Some("here")),
    ]);
    let results = check_link_purpose(&tree);
    // "here" and empty both violate; descriptive passes
    assert!(results.violations.len() >= 2);
}

// ---------------------------------------------------------------------------
// 2.4.6 Headings and Labels — check_headings
// ---------------------------------------------------------------------------

#[test]
fn test_246_valid_heading_hierarchy_passes() {
    let tree = AXTree::from_nodes(vec![
        heading("1", 1, Some("Main Title")),
        heading("2", 2, Some("Section")),
        heading("3", 3, Some("Subsection")),
    ]);
    let results = check_headings(&tree);
    let hierarchy_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.message.contains("skipped"))
        .collect();
    assert!(
        hierarchy_violations.is_empty(),
        "Valid hierarchy should not produce skipped-level violations"
    );
}

#[test]
fn test_246_skipped_heading_level_flagged() {
    let tree = AXTree::from_nodes(vec![
        heading("1", 1, Some("Main Title")),
        heading("2", 4, Some("Jumped to h4")),
    ]);
    let results = check_headings(&tree);
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("skipped")));
}

#[test]
fn test_246_empty_heading_flagged() {
    let tree = AXTree::from_nodes(vec![heading("1", 1, None)]);
    let results = check_headings(&tree);
    assert!(!results.violations.is_empty());
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("empty")));
}

#[test]
fn test_246_missing_h1_flagged() {
    let tree = AXTree::from_nodes(vec![
        heading("1", 2, Some("Section")),
        heading("2", 3, Some("Subsection")),
    ]);
    let results = check_headings(&tree);
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("missing an h1")));
}

#[test]
fn test_246_multiple_h1_flagged() {
    let tree = AXTree::from_nodes(vec![
        heading("1", 1, Some("First Title")),
        heading("2", 1, Some("Second Title")),
    ]);
    let results = check_headings(&tree);
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("Multiple h1")));
}

// ---------------------------------------------------------------------------
// 3.1.1 Language of Page — check_language
// ---------------------------------------------------------------------------

#[test]
fn test_311_page_with_valid_lang_passes() {
    let tree = AXTree::from_nodes(vec![doc_with_lang("1", Some("en"))]);
    let results = check_language(&tree);
    assert!(results.violations.is_empty());
    assert_eq!(results.passes, 1);
}

#[test]
fn test_311_page_with_lang_de_passes() {
    let tree = AXTree::from_nodes(vec![doc_with_lang("1", Some("de"))]);
    let results = check_language(&tree);
    assert!(results.violations.is_empty());
}

#[test]
fn test_311_page_with_region_subtag_passes() {
    let tree = AXTree::from_nodes(vec![doc_with_lang("1", Some("en-US"))]);
    let results = check_language(&tree);
    assert!(results.violations.is_empty());
}

#[test]
fn test_311_page_without_lang_flagged() {
    let tree = AXTree::from_nodes(vec![doc_with_lang("1", None)]);
    let results = check_language(&tree);
    assert!(!results.violations.is_empty());
    assert_eq!(results.violations[0].rule, "3.1.1");
}

#[test]
fn test_311_page_with_empty_lang_flagged() {
    let tree = AXTree::from_nodes(vec![doc_with_lang("1", Some(""))]);
    let results = check_language(&tree);
    assert!(!results.violations.is_empty());
}

// ---------------------------------------------------------------------------
// 4.1.2 ARIA Role Validity — check_aria_roles
// ---------------------------------------------------------------------------

#[test]
fn test_412_valid_role_passes() {
    let tree = AXTree::from_nodes(vec![
        node_with_children("1", "WebArea", Some("Page"), vec!["2"]),
        node_with_parent("2", "button", Some("Submit"), "1"),
    ]);
    let results = check_aria_roles(&tree);
    assert_eq!(results.violations.len(), 0);
}

#[test]
fn test_412_invalid_role_flagged() {
    let tree = AXTree::from_nodes(vec![node("1", "superbutton", Some("Fake Button"))]);
    let results = check_aria_roles(&tree);
    assert!(!results.violations.is_empty());
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("invalid ARIA role")));
}

#[test]
fn test_412_invalid_aria_attribute_flagged() {
    let mut n = node("1", "button", Some("Click me"));
    n.properties.push(AXProperty {
        name: "aria-notvalid".to_string(),
        value: AXValue::String("true".to_string()),
    });
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_aria_roles(&tree);
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("invalid ARIA attribute")));
}

#[test]
fn test_412_listitem_without_list_context_flagged() {
    let tree = AXTree::from_nodes(vec![
        node_with_children("1", "WebArea", Some("Page"), vec!["2"]),
        node_with_parent("2", "listitem", Some("Item"), "1"),
    ]);
    let results = check_aria_roles(&tree);
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.contains("required parent context")));
}

#[test]
fn test_412_listitem_with_list_parent_passes() {
    let tree = AXTree::from_nodes(vec![
        {
            let mut n = node_with_children("1", "WebArea", Some("Page"), vec!["2"]);
            n.child_ids = vec!["2".to_string()];
            n
        },
        {
            let mut n = node("2", "list", Some("My List"));
            n.child_ids = vec!["3".to_string()];
            n.parent_id = Some("1".to_string());
            n
        },
        {
            let mut n = node("3", "listitem", Some("An item"));
            n.parent_id = Some("2".to_string());
            n
        },
    ]);
    let results = check_aria_roles(&tree);
    let context_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.message.contains("required parent context") && v.node_id == "3")
        .collect();
    assert!(
        context_violations.is_empty(),
        "listitem inside list should not be flagged for context"
    );
}

// ---------------------------------------------------------------------------
// 1.4.4 Resize Text — check_resize_text
// ---------------------------------------------------------------------------

#[test]
fn test_144_normal_viewport_passes() {
    let tree = AXTree::from_nodes(vec![root_with_viewport(
        "width=device-width, initial-scale=1",
    )]);
    let results = check_resize_text(&tree);
    assert_eq!(results.violations.len(), 0);
}

#[test]
fn test_144_user_scalable_no_flagged() {
    let tree = AXTree::from_nodes(vec![root_with_viewport(
        "width=device-width, user-scalable=no",
    )]);
    let results = check_resize_text(&tree);
    assert_eq!(results.violations.len(), 1);
    assert!(results.violations[0].message.contains("user-scalable=no"));
}

#[test]
fn test_144_maximum_scale_too_low_flagged() {
    let tree = AXTree::from_nodes(vec![root_with_viewport(
        "width=device-width, maximum-scale=1.0",
    )]);
    let results = check_resize_text(&tree);
    assert_eq!(results.violations.len(), 1);
    assert!(results.violations[0].message.contains("maximum-scale"));
}

#[test]
fn test_144_maximum_scale_sufficient_passes() {
    let tree = AXTree::from_nodes(vec![root_with_viewport(
        "width=device-width, maximum-scale=2.0",
    )]);
    let results = check_resize_text(&tree);
    assert_eq!(results.violations.len(), 0);
}

// ---------------------------------------------------------------------------
// 1.4.3 Contrast helpers — Color parsing and contrast ratio calculation
// (ContrastRule::check_with_page requires CDP; these test the pure functions)
// ---------------------------------------------------------------------------

#[test]
fn test_color_parse_rgb() {
    let c = Color::from_css("rgb(255, 0, 0)").expect("should parse");
    assert_eq!(c.r, 255);
    assert_eq!(c.g, 0);
    assert_eq!(c.b, 0);
}

#[test]
fn test_color_parse_rgba() {
    let c = Color::from_css("rgba(0, 128, 255, 0.5)").expect("should parse");
    assert_eq!(c.r, 0);
    assert_eq!(c.g, 128);
    assert_eq!(c.b, 255);
}

#[test]
fn test_color_parse_hex6() {
    let c = Color::from_css("#FF0000").expect("should parse");
    assert_eq!(c.r, 255);
    assert_eq!(c.g, 0);
    assert_eq!(c.b, 0);
}

#[test]
fn test_color_parse_hex3() {
    let c = Color::from_css("#F00").expect("should parse");
    assert_eq!(c.r, 255);
    assert_eq!(c.g, 0);
    assert_eq!(c.b, 0);
}

#[test]
fn test_color_invalid_input_returns_none() {
    assert!(Color::from_css("not-a-color").is_none());
    assert!(Color::from_css("").is_none());
    assert!(Color::from_css("hsl(120, 100%, 50%)").is_none());
}

#[test]
fn test_relative_luminance_white() {
    let white = Color::new(255, 255, 255);
    let lum = white.relative_luminance();
    assert!(
        (lum - 1.0).abs() < 0.01,
        "White luminance should be ~1.0, got {}",
        lum
    );
}

#[test]
fn test_relative_luminance_black() {
    let black = Color::new(0, 0, 0);
    let lum = black.relative_luminance();
    assert!(lum < 0.01, "Black luminance should be ~0.0, got {}", lum);
}

#[test]
fn test_contrast_ratio_black_on_white() {
    let black = Color::new(0, 0, 0);
    let white = Color::new(255, 255, 255);
    let ratio = ContrastRule::calculate_contrast_ratio(&black, &white);
    assert!(
        (ratio - 21.0).abs() < 0.1,
        "Black/white contrast ratio should be ~21:1, got {}",
        ratio
    );
}

#[test]
fn test_contrast_ratio_identical_colors() {
    let red = Color::new(255, 0, 0);
    let ratio = ContrastRule::calculate_contrast_ratio(&red, &red);
    assert!(
        (ratio - 1.0).abs() < 0.01,
        "Same color should yield 1:1 ratio, got {}",
        ratio
    );
}

#[test]
fn test_contrast_meets_requirement_aa_normal_pass() {
    assert!(ContrastRule::meets_requirement(4.5, false, WcagLevel::AA));
    assert!(ContrastRule::meets_requirement(5.0, false, WcagLevel::AA));
}

#[test]
fn test_contrast_meets_requirement_aa_normal_fail() {
    assert!(!ContrastRule::meets_requirement(4.0, false, WcagLevel::AA));
    assert!(!ContrastRule::meets_requirement(1.0, false, WcagLevel::AA));
}

#[test]
fn test_contrast_meets_requirement_aa_large_text() {
    // Large text needs 3:1 at AA
    assert!(ContrastRule::meets_requirement(3.0, true, WcagLevel::AA));
    assert!(!ContrastRule::meets_requirement(2.9, true, WcagLevel::AA));
}

#[test]
fn test_contrast_meets_requirement_aaa_normal() {
    // Normal text needs 7:1 at AAA
    assert!(ContrastRule::meets_requirement(7.0, false, WcagLevel::AAA));
    assert!(!ContrastRule::meets_requirement(6.9, false, WcagLevel::AAA));
}

#[test]
fn test_contrast_level_a_always_passes() {
    // Level A has no contrast requirement
    assert!(ContrastRule::meets_requirement(1.0, false, WcagLevel::A));
    assert!(ContrastRule::meets_requirement(1.1, true, WcagLevel::A));
}

#[test]
fn test_is_transparent_various_inputs() {
    assert!(Color::is_transparent("transparent"));
    assert!(Color::is_transparent("rgba(0, 0, 0, 0)"));
    assert!(Color::is_transparent("rgba(255, 255, 255, 0.0)"));
    assert!(!Color::is_transparent("rgba(0, 0, 0, 0.5)"));
    assert!(!Color::is_transparent("rgba(0, 0, 0, 1)"));
    assert!(!Color::is_transparent("rgb(255, 255, 255)"));
    assert!(!Color::is_transparent("#ffffff"));
}

// ---------------------------------------------------------------------------
// Engine integration: check_all runs rules on a minimal tree
// ---------------------------------------------------------------------------

#[test]
fn test_engine_detects_missing_alt_at_level_a() {
    let tree = AXTree::from_nodes(vec![
        node("1", "RootWebArea", Some("Test Page")),
        node("2", "image", None),
    ]);
    let results = check_all(&tree, WcagLevel::A);
    assert!(!results.violations.is_empty());
    assert!(results.violations.iter().any(|v| v.rule == "1.1.1"));
}

#[test]
fn test_engine_level_aa_includes_level_a_violations() {
    let tree = AXTree::from_nodes(vec![
        node("1", "RootWebArea", Some("Test Page")),
        node("2", "image", None),
    ]);
    let results_a = check_all(&tree, WcagLevel::A);
    let results_aa = check_all(&tree, WcagLevel::AA);
    // AA should catch at least as many violations as A
    assert!(results_aa.violations.len() >= results_a.violations.len());
}

#[test]
fn test_engine_clean_tree_has_zero_image_alt_violations() {
    let tree = AXTree::from_nodes(vec![
        node("1", "RootWebArea", Some("My Store - Product Page")),
        node("2", "image", Some("Product photo of a red bicycle")),
        node("3", "image", Some("Company logo")),
    ]);
    let results = check_all(&tree, WcagLevel::A);
    let alt_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.rule == "1.1.1")
        .collect();
    assert!(
        alt_violations.is_empty(),
        "Clean tree should have no 1.1.1 violations"
    );
}

#[test]
fn test_engine_missing_lang_detected_at_level_a() {
    // No lang property on the document node
    let tree = AXTree::from_nodes(vec![node("1", "RootWebArea", Some("Page Title"))]);
    let results = check_all(&tree, WcagLevel::A);
    assert!(
        results.violations.iter().any(|v| v.rule == "3.1.1"),
        "Missing lang should produce a 3.1.1 violation"
    );
}
