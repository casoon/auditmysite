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
// 1.1.1 Non-text Content
// ---------------------------------------------------------------------------
// Laeuft seit #1181 nur noch im geteilten Bestand gegen den DOM: `<img>` als
// `images/alt-missing`, SVG als `svg/name-missing`, Elemente mit
// `role="img"` als `names/required-missing` (ACT 23a2a8). Die Baumregel
// `image-alt` ist entfernt; im AX-Baum war ein `<img>` ohne `src` vom
// `<div role="img">` nicht zu unterscheiden.

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
