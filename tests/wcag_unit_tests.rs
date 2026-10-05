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
use auditmysite::wcag::rules::{check_link_purpose, check_text_alternatives, Color};

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
// 2.4.4 Link Purpose — check_link_purpose
// ---------------------------------------------------------------------------

#[test]
fn test_244_empty_link_flagged() {
    // Empty links are handled by a11y.name_role.missing (4.1.2).
    // This rule (2.4.4) skips them to avoid double-reporting.
    let tree = AXTree::from_nodes(vec![node("1", "link", None)]);
    let results = check_link_purpose(&tree);
    assert!(
        results.violations.is_empty(),
        "Empty links must not be reported by link_purpose — covered by name_role.missing"
    );
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
    assert!(!results.warnings.is_empty());
    assert!(results
        .warnings
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
    // "here" violates; descriptive passes; empty is handled by
    // name_role.missing (skipped here to avoid double-reporting).
    assert!(!results.violations.is_empty());
    assert!(results
        .violations
        .iter()
        .any(|v| v.message.to_lowercase().contains("generic")));
}

// ---------------------------------------------------------------------------
// 2.4.6 Headings and Labels
// ---------------------------------------------------------------------------
// Laeuft seit der Umstellung als geteilte Regel gegen den DOM
// (`headings/empty`, `headings/skip-level`, `headings/h1-missing`,
// `headings/h1-multiple`). Die frueheren Tests hier stellten AX-Knoten her und
// sortierten sie nach `node_id` als Text -- dieselbe Annahme, an der die
// AX-Fassung in langen Seiten scheiterte ("h10" vor "h2"). Die geteilte
// Fassung laeuft in Dokumentreihenfolge; ihre Tests stehen in `a11y-rules`.

// ---------------------------------------------------------------------------
// 3.1.1 Language of Page
// ---------------------------------------------------------------------------
// Laeuft seit der Umstellung als geteilte Regel gegen den DOM
// (`document/lang-missing`, `document/lang-invalid`). Die Tests dazu stehen in
// `wcag::shared` und pruefen den authoritativen Weg: Die frueheren Tests hier
// stellten AX-Knoten mit einer `lang`-Eigenschaft her, die Chrome in der
// Praxis auch ohne Autoren-`lang` synthetisiert -- sie konnten den haeufigsten
// Verstoss also gar nicht abbilden.

// ---------------------------------------------------------------------------
// 4.1.2 ARIA-Rollen und erforderlicher Kontext
// ---------------------------------------------------------------------------
// Laufen seit #691 als `aria/role-invalid`, `aria/required-parent-missing`
// und `aria/required-children-missing` im geteilten Bestand, gegen den DOM.
// Die Tests dazu stehen in `wcag::shared` und in `a11y-rules`.

// 1.4.4 Resize Text: der Viewport laeuft seit #690 als `zoom/*` im geteilten
// Bestand (siehe `wcag::shared`) -- der AX-Baum hat keine
// `viewport`-Eigenschaft (#QA-030).

// ---------------------------------------------------------------------------
// Color parsing and contrast ratio — the helpers behind the CSS-level 1.4.11
// check. Text contrast (1.4.3/1.4.6) runs as `contrast/text-*` in the shared
// rule set since #698; its tests live in `a11y-rules`.
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
    let ratio = black.contrast_ratio(&white);
    assert!(
        (ratio - 21.0).abs() < 0.1,
        "Black/white contrast ratio should be ~21:1, got {}",
        ratio
    );
}

#[test]
fn test_contrast_ratio_identical_colors() {
    let red = Color::new(255, 0, 0);
    let ratio = red.contrast_ratio(&red);
    assert!(
        (ratio - 1.0).abs() < 0.01,
        "Same color should yield 1:1 ratio, got {}",
        ratio
    );
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

// ---------------------------------------------------------------------------

// 2.4.3/4.1.2 fokussierbar unter aria-hidden: laeuft seit #690 als
// `keyboard/hidden-focusable` im geteilten Bestand (siehe `wcag::shared`),
// die AX-Regel `check_focus_order` ist geloescht.

// 1.3.1 Beschriftung nur per `title`: laeuft seit #693 als
// `forms/title-only-label` im geteilten Bestand (siehe `wcag::shared`),
// `check_label_title_only` ist geloescht.

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
    let skip_link = {
        // Chrome exposes the link target as `url`; region recognises the
        // skip link by that same-page target, not by its text (#642).
        let mut n = focusable("skip", "link", Some("Skip to main content"));
        n.properties.push(AXProperty {
            name: "url".to_string(),
            value: AXValue::String("#main".to_string()),
        });
        n
    };
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
    let skip_link = {
        // Chrome exposes the link target as `url`; region recognises the
        // skip link by that same-page target, not by its text (#642).
        let mut n = focusable("skip", "link", Some("Skip to main content"));
        n.properties.push(AXProperty {
            name: "url".to_string(),
            value: AXValue::String("#main".to_string()),
        });
        n
    };
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

// Das Gegenstueck zu den uebrigen Szenarien -- eine Seite ohne
// lang-Attribut -- steht nicht mehr hier: 3.1.1 laeuft seit der Umstellung
// als geteilte Regel gegen den DOM (`document/lang-missing`). Ein
// AX-Baum-Szenario koennte es gar nicht mehr ausloesen, denn die
// AX-Eigenschaft `language` synthetisiert Chrome auch ohne Autoren-`lang`.
// Die Faelle stehen als DOM-Szenarien in `wcag::shared`.

// Das Szenario zur uebersprungenen Ueberschriftenebene steht nicht mehr hier.
// Die Gliederung wird seit der Umstellung geteilt geprueft
// (`headings/skip-level`), und zwar in Dokumentreihenfolge ueber den DOM. Ein
// AX-Baum-Szenario koennte sie gar nicht mehr ausloesen -- und die alte
// Fassung sortierte nach `node_id` als Text, was in laengeren Seiten "h10" vor
// "h2" einsortierte. Die Faelle stehen als DOM-Szenarien in `a11y-rules`.

// Das Szenario mit kaputtem ARIA (Menue ohne menuitem) steht nicht mehr hier:
// Rollen, Kontext und Bestandteile laufen seit #691 als `aria/*` im geteilten
// Bestand gegen den DOM, kein AX-Baum-Szenario loest sie mehr aus. Die Faelle
// stehen als DOM-Szenarien in `wcag::shared` und `a11y-rules`.
