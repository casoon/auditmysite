//! WCAG Rule Unit Tests — browser-free
//!
//! These tests exercise the WCAG rule logic directly against in-memory AXTree
//! structures. No Chrome, no CDP, no network required.
//!
//! Run with:
//!   cargo test --test wcag_unit_tests

use auditmysite::accessibility::{AXNode, AXProperty, AXTree, AXValue, NameSource};
use auditmysite::cli::WcagLevel;
use auditmysite::wcag::engine::check_all;
use auditmysite::wcag::rules::{
    check_aria_roles, check_focus_order, check_headings, check_info_relationships,
    check_label_title_only, check_language, check_link_purpose, check_list_structure,
    check_page_titled, check_resize_text, check_text_alternatives, Color, ContrastRule,
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

// ---------------------------------------------------------------------------
// 1.3.1 Info and Relationships — check_info_relationships (table structure)
// ---------------------------------------------------------------------------

fn table_node(id: &str, row_ids: Vec<&str>) -> AXNode {
    node_with_children(id, "table", None, row_ids)
}

fn row_node(id: &str, parent: &str, cell_ids: Vec<&str>) -> AXNode {
    let mut n = node_with_children(id, "row", None, cell_ids);
    n.parent_id = Some(parent.to_string());
    n
}

fn cell_node(id: &str, role: &str, parent: &str, name: Option<&str>) -> AXNode {
    let mut n = node(id, role, name);
    n.parent_id = Some(parent.to_string());
    n
}

#[test]
fn test_131_table_with_columnheaders_passes() {
    let tree = AXTree::from_nodes(vec![
        table_node("t", vec!["r1", "r2"]),
        row_node("r1", "t", vec!["h1", "h2"]),
        cell_node("h1", "columnheader", "r1", Some("Name")),
        cell_node("h2", "columnheader", "r1", Some("Age")),
        row_node("r2", "t", vec!["c1", "c2"]),
        cell_node("c1", "cell", "r2", Some("Alice")),
        cell_node("c2", "cell", "r2", Some("30")),
    ]);
    let results = check_info_relationships(&tree);
    let table_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.message.contains("header") && v.node_id == "t")
        .collect();
    assert!(
        table_violations.is_empty(),
        "Table with column headers should not produce a header violation"
    );
}

#[test]
fn test_131_table_without_headers_flagged() {
    let tree = AXTree::from_nodes(vec![
        table_node("t", vec!["r1"]),
        row_node("r1", "t", vec!["c1", "c2"]),
        cell_node("c1", "cell", "r1", Some("Alice")),
        cell_node("c2", "cell", "r1", Some("30")),
    ]);
    let results = check_info_relationships(&tree);
    assert!(
        results
            .violations
            .iter()
            .any(|v| v.message.contains("header")),
        "Table with data cells but no headers should be flagged"
    );
}

#[test]
fn test_131_table_with_only_headers_no_violation() {
    // A table consisting only of headers (no data cells) is OK
    let tree = AXTree::from_nodes(vec![
        table_node("t", vec!["r1"]),
        row_node("r1", "t", vec!["h1", "h2"]),
        cell_node("h1", "columnheader", "r1", Some("Name")),
        cell_node("h2", "columnheader", "r1", Some("Score")),
    ]);
    let results = check_info_relationships(&tree);
    let table_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.message.contains("header") && v.node_id == "t")
        .collect();
    assert!(
        table_violations.is_empty(),
        "Table with only header cells should not be flagged"
    );
}

// ---------------------------------------------------------------------------
// 1.3.1 Info and Relationships — check_list_structure
// ---------------------------------------------------------------------------

#[test]
fn test_131_listitem_inside_list_passes() {
    let list = node_with_children("list", "list", None, vec!["i1", "i2"]);
    let mut i1 = node("i1", "listitem", Some("First"));
    i1.parent_id = Some("list".to_string());
    let mut i2 = node("i2", "listitem", Some("Second"));
    i2.parent_id = Some("list".to_string());
    let tree = AXTree::from_nodes(vec![list, i1, i2]);
    let results = check_list_structure(&tree);
    let orphan_violations: Vec<_> = results
        .violations
        .iter()
        .filter(|v| v.message.contains("List item is not contained"))
        .collect();
    assert!(
        orphan_violations.is_empty(),
        "List items inside a list should not be flagged"
    );
}

#[test]
fn test_131_listitem_without_list_parent_flagged() {
    // listitem with no parent_id → no list ancestor
    let item = node("i1", "listitem", Some("Orphan item"));
    let tree = AXTree::from_nodes(vec![item]);
    let results = check_list_structure(&tree);
    assert!(
        results
            .violations
            .iter()
            .any(|v| v.message.contains("List item is not contained")),
        "Listitem without a list parent should be flagged"
    );
}

#[test]
fn test_131_empty_list_flagged() {
    // A list with no visible children
    let list = node("list", "list", None); // no child_ids
    let tree = AXTree::from_nodes(vec![list]);
    let results = check_list_structure(&tree);
    assert!(
        results
            .violations
            .iter()
            .any(|v| v.message.contains("no list items")),
        "Empty list should be flagged"
    );
}

// ---------------------------------------------------------------------------
// 2.4.3 Focus Order — check_focus_order
// ---------------------------------------------------------------------------

#[test]
fn test_243_positive_tabindex_flagged() {
    let mut n = node("1", "link", Some("Home"));
    n.properties.push(AXProperty {
        name: "tabindex".to_string(),
        value: AXValue::Int(5),
    });
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_focus_order(&tree);
    assert!(
        results
            .violations
            .iter()
            .any(|v| v.message.contains("tabindex=5")),
        "Positive tabindex should be flagged"
    );
}

#[test]
fn test_243_zero_tabindex_passes() {
    let mut n = node("1", "link", Some("Home"));
    n.properties.push(AXProperty {
        name: "tabindex".to_string(),
        value: AXValue::Int(0),
    });
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_focus_order(&tree);
    assert_eq!(
        results.violations.len(),
        0,
        "tabindex=0 should not produce focus-order violations"
    );
}

#[test]
fn test_243_negative_tabindex_passes() {
    let mut n = node("1", "link", Some("Home"));
    n.properties.push(AXProperty {
        name: "tabindex".to_string(),
        value: AXValue::Int(-1),
    });
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_focus_order(&tree);
    assert_eq!(
        results.violations.len(),
        0,
        "tabindex=-1 should not produce focus-order violations"
    );
}

#[test]
fn test_243_focusable_in_aria_hidden_flagged() {
    let mut n = node("1", "button", Some("Hidden button"));
    n.properties.push(AXProperty {
        name: "aria-hidden".to_string(),
        value: AXValue::String("true".to_string()),
    });
    n.properties.push(AXProperty {
        name: "focusable".to_string(),
        value: AXValue::Bool(true),
    });
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_focus_order(&tree);
    assert!(
        results
            .violations
            .iter()
            .any(|v| v.message.contains("aria-hidden")),
        "Focusable element inside aria-hidden should be flagged"
    );
}

#[test]
fn test_243_clean_interactive_tree_passes() {
    let tree = AXTree::from_nodes(vec![
        node("1", "link", Some("Home")),
        node("2", "button", Some("Submit")),
        node("3", "link", Some("About")),
    ]);
    let results = check_focus_order(&tree);
    assert_eq!(
        results.violations.len(),
        0,
        "Clean interactive tree should have no focus-order violations"
    );
}

// ---------------------------------------------------------------------------
// 1.3.1 Label Title Only — check_label_title_only
// ---------------------------------------------------------------------------

#[test]
fn test_131_title_only_source_flagged() {
    let mut n = node("1", "textbox", Some("Search"));
    n.name_source = Some(NameSource::Title);
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_label_title_only(&tree);
    assert!(
        !results.violations.is_empty(),
        "Input with title-only accessible name should be flagged"
    );
    assert_eq!(results.violations[0].rule, "1.3.1");
}

#[test]
fn test_131_attribute_source_passes() {
    let mut n = node("1", "textbox", Some("Search"));
    n.name_source = Some(NameSource::Attribute); // aria-label
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_label_title_only(&tree);
    assert!(
        results.violations.is_empty(),
        "Input with aria-label source should not be flagged"
    );
}

#[test]
fn test_131_title_heuristic_flagged() {
    // No name_source but title property matches accessible name and no aria-label
    let mut n = node("1", "textbox", Some("Enter email"));
    n.properties.push(AXProperty {
        name: "title".to_string(),
        value: AXValue::String("Enter email".to_string()),
    });
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_label_title_only(&tree);
    assert!(
        !results.violations.is_empty(),
        "Input whose name matches its title attribute (no aria-label) should be flagged"
    );
}

#[test]
fn test_131_title_heuristic_with_aria_label_passes() {
    let mut n = node("1", "textbox", Some("Enter email"));
    n.properties.push(AXProperty {
        name: "title".to_string(),
        value: AXValue::String("Enter email".to_string()),
    });
    n.properties.push(AXProperty {
        name: "aria-label".to_string(),
        value: AXValue::String("Enter email".to_string()),
    });
    let tree = AXTree::from_nodes(vec![n]);
    let results = check_label_title_only(&tree);
    assert!(
        results.violations.is_empty(),
        "Input with aria-label in addition to title should not be flagged"
    );
}

// ---------------------------------------------------------------------------
// Scenario tests — determinism guard for complete page mocks
//
// Each test models a realistic page with exactly the accessibility issue named
// in the scenario, then asserts the exact set of WCAG rule IDs that fire.
// If a rule change silently expands or shrinks violation output, these tests
// catch it immediately.
//
// The "clean page" baseline includes a proper landmark structure (main) and
// focusable=true on interactive elements so region/keyboard rules don't fire.
// All violation scenarios build on top of this baseline.
// ---------------------------------------------------------------------------

/// Build the sorted list of distinct rule IDs that produced violations.
fn fired_rules(tree: &AXTree, level: WcagLevel) -> Vec<String> {
    let mut ids: Vec<String> = check_all(tree, level)
        .violations
        .into_iter()
        .map(|v| v.rule)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    ids.sort();
    ids
}

/// Helper: AXNode for a focusable interactive element.
fn focusable(id: &str, role: &str, name: Option<&str>) -> AXNode {
    let mut n = node(id, role, name);
    n.properties.push(AXProperty {
        name: "focusable".to_string(),
        value: AXValue::Bool(true),
    });
    n
}

/// Build a minimal clean page tree that passes all Level AA rules.
///
/// Includes all required Level AA landmarks (banner, navigation, main, contentinfo)
/// and focusable interactive elements to satisfy keyboard/region rules.
///
/// Structure:
///   RootWebArea [lang=en, children=[banner, nav, main, footer]]
///     ├── banner (header) [children=[logo-link]]
///     │    └── link "Home" (focusable)
///     ├── navigation [name="Main"] [children=[nav-link]]
///     │    └── link "About" (focusable)
///     ├── main [children=[h1, img1, link1]]
///     │    ├── heading h1 "Our Products"
///     │    ├── image "A red bicycle"
///     │    └── link "View product catalogue" (focusable)
///     └── contentinfo (footer) []
fn clean_page_tree() -> AXTree {
    let root = {
        let mut n = node_with_children(
            "root",
            "RootWebArea",
            Some("Products - My Shop"),
            vec!["skip", "banner", "nav", "main", "footer"],
        );
        n.properties.push(AXProperty {
            name: "lang".to_string(),
            value: AXValue::String("en".to_string()),
        });
        n
    };
    // Skip link before first landmark (satisfies 2.4.1 skip-link check)
    let skip_link = focusable("skip", "link", Some("Skip to main content"));
    let banner = node_with_children("banner", "banner", None, vec!["logo-link"]);
    let logo_link = {
        let mut n = focusable("logo-link", "link", Some("Home"));
        n.parent_id = Some("banner".to_string());
        n
    };
    let nav = {
        let mut n = node_with_children("nav", "navigation", Some("Main"), vec!["nav-link"]);
        n.parent_id = Some("root".to_string());
        n
    };
    let nav_link = {
        let mut n = focusable("nav-link", "link", Some("About us"));
        n.parent_id = Some("nav".to_string());
        n
    };
    let main_node = node_with_children("main", "main", None, vec!["h1", "img1", "link1"]);
    let h1 = {
        let mut n = heading("h1", 1, Some("Our Products"));
        n.parent_id = Some("main".to_string());
        n
    };
    let img = {
        let mut n = node("img1", "image", Some("A red bicycle"));
        n.parent_id = Some("main".to_string());
        n
    };
    let link = {
        let mut n = focusable("link1", "link", Some("View product catalogue"));
        n.parent_id = Some("main".to_string());
        n
    };
    let footer = {
        let mut n = node("footer", "contentinfo", None);
        n.parent_id = Some("root".to_string());
        n
    };
    AXTree::from_nodes(vec![
        root, skip_link, banner, logo_link, nav, nav_link, main_node, h1, img, link, footer,
    ])
}

/// The clean page baseline must produce zero violations at Level AA.
/// This acts as a guard: if this fails, a new rule broke with no matching fix.
#[test]
fn scenario_clean_page_no_violations() {
    let tree = clean_page_tree();
    let rules = fired_rules(&tree, WcagLevel::AA);
    assert!(
        rules.is_empty(),
        "Clean page should produce zero violations; got: {rules:?}"
    );
}

/// Build a complete clean page with custom main content nodes.
///
/// Replaces h1/img1/link1 in the main landmark with `main_children`.
/// All landmark structure (banner, nav, contentinfo, skip link) is preserved.
/// Call this from violation scenarios so each test adds exactly one bad element.
fn page_with_main_content(main_children: Vec<(&str, AXNode)>) -> AXTree {
    let child_ids: Vec<&str> = main_children.iter().map(|(id, _)| *id).collect();
    let root = {
        let mut n = node_with_children(
            "root",
            "RootWebArea",
            Some("Test Page - My Site"),
            vec!["skip", "banner", "nav", "main", "footer"],
        );
        n.properties.push(AXProperty {
            name: "lang".to_string(),
            value: AXValue::String("en".to_string()),
        });
        n
    };
    let skip_link = focusable("skip", "link", Some("Skip to main content"));
    let banner = node_with_children("banner", "banner", None, vec!["logo-link"]);
    let logo_link = {
        let mut n = focusable("logo-link", "link", Some("Home"));
        n.parent_id = Some("banner".to_string());
        n
    };
    let nav = {
        let mut n = node_with_children("nav", "navigation", Some("Main"), vec!["nav-link"]);
        n.parent_id = Some("root".to_string());
        n
    };
    let nav_link = {
        let mut n = focusable("nav-link", "link", Some("About us"));
        n.parent_id = Some("nav".to_string());
        n
    };
    let main_node = node_with_children("main", "main", None, child_ids);
    let footer = {
        let mut n = node("footer", "contentinfo", None);
        n.parent_id = Some("root".to_string());
        n
    };
    let mut nodes = vec![
        root, skip_link, banner, logo_link, nav, nav_link, main_node, footer,
    ];
    for (_, mut n) in main_children {
        n.parent_id = Some("main".to_string());
        nodes.push(n);
    }
    AXTree::from_nodes(nodes)
}

/// Page with a single image that has no alt text.
/// Exactly rule 1.1.1 must fire — nothing else (baseline is clean).
#[test]
fn scenario_missing_alt_fires_only_111() {
    let tree = page_with_main_content(vec![
        ("h1", heading("h1", 1, Some("Catalogue"))),
        ("img1", node("img1", "image", None)), // missing alt
        (
            "link1",
            focusable("link1", "link", Some("Browse all items")),
        ),
    ]);
    let rules = fired_rules(&tree, WcagLevel::AA);
    assert_eq!(
        rules,
        vec!["1.1.1"],
        "Missing alt should fire exactly 1.1.1; got: {rules:?}"
    );
}

/// Page with an `<html>` element that carries no lang attribute.
/// Exactly rule 3.1.1 must fire — the baseline otherwise passes.
#[test]
fn scenario_no_lang_fires_only_311() {
    // Build clean tree, then remove the lang property from root
    let mut tree = clean_page_tree();
    let root = tree.nodes.get_mut("root").expect("root node");
    root.properties.retain(|p| p.name != "lang");

    let rules = fired_rules(&tree, WcagLevel::AA);
    assert_eq!(
        rules,
        vec!["3.1.1"],
        "Missing lang should fire exactly 3.1.1; got: {rules:?}"
    );
}

/// Heading hierarchy that jumps from h1 directly to h3 (skips h2).
/// Rule 1.3.1 must fire (the hierarchy violation is classified under Info and
/// Relationships, not 2.4.6 Headings and Labels).  No other rule fires.
#[test]
fn scenario_heading_skip_fires_only_131() {
    let tree = page_with_main_content(vec![
        ("h1", heading("h1", 1, Some("My Blog Post"))),
        ("h3", heading("h3", 3, Some("Section One"))), // skipped h2 → 1.3.1
        (
            "link1",
            focusable("link1", "link", Some("Read the full post")),
        ),
    ]);
    let rules = fired_rules(&tree, WcagLevel::AA);
    assert_eq!(
        rules,
        vec!["1.3.1"],
        "Skipped heading level should fire exactly 1.3.1; got: {rules:?}"
    );
}

/// Page with an invalid ARIA role and a focusable element inside aria-hidden.
/// Rules 4.1.2 (invalid role) and 2.4.3 (focusable+aria-hidden) must both fire.
/// No other rules should fire (baseline is otherwise clean).
#[test]
fn scenario_broken_aria_fires_412_and_243() {
    let hidden_focusable = {
        let mut n = node("btn2", "button", Some("Hidden action"));
        n.properties.push(AXProperty {
            name: "aria-hidden".to_string(),
            value: AXValue::String("true".to_string()),
        });
        n.properties.push(AXProperty {
            name: "focusable".to_string(),
            value: AXValue::Bool(true),
        });
        n
    };
    let tree = page_with_main_content(vec![
        ("h1", heading("h1", 1, Some("Dashboard"))),
        (
            "btn1",
            focusable("btn1", "superwidget", Some("Custom Button")),
        ), // invalid role → 4.1.2
        ("btn2", hidden_focusable), // focusable + aria-hidden → 2.4.3
    ]);
    let rules = fired_rules(&tree, WcagLevel::AA);
    assert!(
        rules.contains(&"4.1.2".to_string()),
        "Invalid ARIA role should fire 4.1.2; got: {rules:?}"
    );
    assert!(
        rules.contains(&"2.4.3".to_string()),
        "Focusable element in aria-hidden should fire 2.4.3; got: {rules:?}"
    );
    // Only the two expected rules should fire — no regressions
    let unexpected: Vec<_> = rules
        .iter()
        .filter(|r| *r != "4.1.2" && *r != "2.4.3")
        .collect();
    assert!(
        unexpected.is_empty(),
        "Unexpected rules fired in broken-aria scenario: {unexpected:?}"
    );
}
