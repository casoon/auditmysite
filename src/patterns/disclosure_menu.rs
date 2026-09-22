//! DisclosureMenu pattern (issue #28).
//!
//! Detects collapsible menu triggers (hamburger menus, dropdown triggers):
//! a button (or button-role element) with `aria-expanded` is the canonical
//! disclosure pattern. Flags missing `aria-expanded` when the structure
//! suggests a disclosure but the attribute is absent.
//!
//! # Natives `<details>`/`<summary>`
//!
//! Bis Plan 53 bot dieses Modul nur Elemente mit `aria-expanded` als
//! Journey-Kandidat an — natives `<details>` erreichte die Disclosure-Journey
//! deshalb nie, obwohl der Diff-Pfad es lesen kann.
//!
//! Zur Größenordnung, gemessen am lokalen Artefakt-Cache (238 Domains):
//! natives `<details>` steht auf **22 Domains (9 %)**, ARIA-Disclosure auf 137
//! (58 %). Auf 7 Domains ist das native Element die *einzige* aufklappbare
//! Struktur — davon sind 4 eigene Testhosts, es bleiben zwei unabhängige
//! Seiten. Die Lücke war also keine große Menge, sondern eine ganze
//! Elementklasse, die der Prüfpfad nie erreichte.

use crate::accessibility::{AXNode, AXTree};
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

use super::{JourneyCandidate, JourneyKind, PatternAnalysis, PatternConfidence, PatternKind};

/// Die Rollen, unter denen Chrome ein natives `<summary>` im
/// Accessibility-Tree führt. `DisclosureTriangleGrouped` ist die Variante in
/// einer `<details name>`-Gruppe, also einem exklusiven Akkordeon.
const NATIVE_DISCLOSURE_ROLES: &[&str] = &["DisclosureTriangle", "DisclosureTriangleGrouped"];

/// Auslöser mit Aufklappzustand, getrennt nach Herkunft der Semantik.
fn triggers(tree: &AXTree) -> (Vec<&AXNode>, Vec<&AXNode>) {
    let aria = tree
        .nodes_with_role("button")
        .into_iter()
        .filter(|n| n.get_property_bool("expanded").is_some())
        .collect();
    let native = NATIVE_DISCLOSURE_ROLES
        .iter()
        .flat_map(|role| tree.nodes_with_role(role))
        .filter(|n| n.get_property_bool("expanded").is_some())
        .collect();
    (aria, native)
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        "trigger"
    } else {
        "triggers"
    }
}

pub fn detect(tree: &AXTree, out: &mut PatternAnalysis) {
    let (aria, native) = triggers(tree);
    if !aria.is_empty() || !native.is_empty() {
        recognize(&aria, &native, out);
        emit_candidates(&aria, &native, out);
    }

    // Die Heuristik unten bleibt an die ARIA-Variante gebunden: ein natives
    // `<details>` deklariert seinen Zustand selbst, es kann ihn nicht vergessen.
    if !aria.is_empty() {
        flag_undeclared_menu_triggers(tree, out);
    }
}

/// Was von dem Muster erkannt wurde — Text und Konfidenz.
fn recognize(aria: &[&AXNode], native: &[&AXNode], out: &mut PatternAnalysis) {
    let disclosure_count = aria.len();
    let with_controls = aria.iter().filter(|n| n.has_property("controls")).count();

    if disclosure_count == 0 {
        // Nur natives Markup: die Semantik steckt im Element, es gibt keine
        // Attributbeziehung, die fehlen könnte.
        out.add_recognized(
            "DisclosureMenu",
            format!(
                "{} native disclosure {} (<details>/<summary>) — semantics come from the element, no ARIA needed.",
                native.len(),
                plural(native.len())
            ),
            PatternConfidence::Strong,
        );
        return;
    }

    let confidence = if with_controls == disclosure_count {
        PatternConfidence::Strong
    } else {
        PatternConfidence::Partial
    };
    let mut detail = if with_controls == disclosure_count {
        format!(
            "{} disclosure {} with aria-expanded and aria-controls — well-formed pattern.",
            disclosure_count,
            plural(disclosure_count)
        )
    } else {
        format!(
            "{} disclosure {} with aria-expanded ({} with aria-controls). Controls relationship strengthens screen-reader announcements.",
            disclosure_count,
            plural(disclosure_count),
            with_controls
        )
    };
    if !native.is_empty() {
        detail.push_str(&format!(
            " Plus {} native <details>/<summary> {}.",
            native.len(),
            plural(native.len())
        ));
    }
    out.add_recognized("DisclosureMenu", detail, confidence);
}

/// Journey-Kandidaten für die interaktive Prüfung.
fn emit_candidates(aria: &[&AXNode], native: &[&AXNode], out: &mut PatternAnalysis) {
    for btn in aria {
        let haspopup = btn.haspopup();
        let is_menu = matches!(haspopup, Some("menu") | Some("true"));
        let has_controls = btn.has_property("controls");
        if let Some(bid) = btn.backend_dom_node_id {
            out.journey_candidates.push(JourneyCandidate {
                pattern_kind: if is_menu {
                    PatternKind::Menu
                } else {
                    PatternKind::Disclosure
                },
                trigger_backend_id: Some(bid),
                controlled_backend_id: None,
                confidence: if has_controls { 0.8 } else { 0.7 },
                required_journey: if is_menu {
                    JourneyKind::MenuOpen
                } else {
                    JourneyKind::DisclosureToggle
                },
            });
        }
    }

    // Natives `<summary>`: kein Attribut, das fehlen oder falsch stehen kann,
    // daher die höhere Konfidenz. `aria-controls` gibt es hier nicht — die
    // Beziehung steckt in der Verschachtelung.
    for summary in native {
        if let Some(bid) = summary.backend_dom_node_id {
            out.journey_candidates.push(JourneyCandidate {
                pattern_kind: PatternKind::Disclosure,
                trigger_backend_id: Some(bid),
                controlled_backend_id: None,
                confidence: 0.9,
                required_journey: JourneyKind::DisclosureToggle,
            });
        }
    }
}

/// Flag toggle-like nodes that look like menus but lack aria-expanded.
/// Heuristic: a `generic` or `link` node whose name contains "menu" /
/// "menü" and has an expanded child group is a likely disclosure that
/// failed to declare `aria-expanded`.
fn flag_undeclared_menu_triggers(tree: &AXTree, out: &mut PatternAnalysis) {
    for node in tree.iter() {
        let role = node.role.as_deref().unwrap_or("");
        if role != "generic" && role != "link" {
            continue;
        }
        let name = node.name.as_deref().unwrap_or("").to_lowercase();
        let looks_like_menu = name.contains("menu") || name.contains("menü");
        if !looks_like_menu {
            continue;
        }
        // Skip if it already has aria-expanded
        if node.get_property_bool("expanded").is_some() {
            continue;
        }
        // Has a focusable descendant suggesting an expandable region?
        let has_focusable_descendant = node
            .child_ids
            .iter()
            .filter_map(|id| tree.get_node(id))
            .any(|c| c.get_property_bool("focusable").unwrap_or(false));
        if !has_focusable_descendant {
            continue;
        }

        out.violations.push(
            Violation::new(
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::Medium,
                format!(
                    "Likely disclosure menu trigger (\"{}\") lacks aria-expanded — screen readers cannot announce open/closed state.",
                    node.name.as_deref().unwrap_or("(menu)")
                ),
                &node.node_id,
            )
            .with_fix(
                "Use a native <button> with aria-expanded=\"true|false\" toggled by the click handler.",
            )
            .with_rule_id("aria-expanded-required")
            .with_help_url("https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn node_with_prop(id: &str, role: &str, prop: AXProperty) -> AXNode {
        let mut n = AXNode {
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
        };
        n.properties.push(prop);
        n
    }

    #[test]
    fn test_button_with_expanded_recognized() {
        let tree = AXTree::from_nodes(vec![node_with_prop(
            "1",
            "button",
            AXProperty {
                name: "expanded".into(),
                value: AXValue::Bool(false),
            },
        )]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized.len(), 1);
        assert_eq!(a.recognized[0].pattern, "DisclosureMenu");
    }

    #[test]
    fn test_button_with_expanded_and_controls_strong() {
        let mut n = node_with_prop(
            "1",
            "button",
            AXProperty {
                name: "expanded".into(),
                value: AXValue::Bool(false),
            },
        );
        n.properties.push(AXProperty {
            name: "controls".into(),
            value: AXValue::String("menu-1".into()),
        });
        let tree = AXTree::from_nodes(vec![n]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
    }

    #[test]
    fn test_nothing_without_buttons() {
        let tree = AXTree::from_nodes(vec![]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.recognized.is_empty());
        assert!(a.journey_candidates.is_empty());
    }

    fn expanded(id: &str, role: &str, backend: i64) -> AXNode {
        let mut n = node_with_prop(
            id,
            role,
            AXProperty {
                name: "expanded".into(),
                value: AXValue::Bool(false),
            },
        );
        n.backend_dom_node_id = Some(backend);
        n
    }

    /// Eine Seite, deren einzige aufklappbare Struktur nativ ist, erreichte die
    /// Disclosure-Journey vor Plan 53 gar nicht.
    #[test]
    fn natives_summary_wird_zum_journey_kandidaten() {
        let tree = AXTree::from_nodes(vec![expanded("1", "DisclosureTriangle", 11)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.recognized.len(), 1);
        assert_eq!(a.recognized[0].confidence, PatternConfidence::Strong);
        assert_eq!(a.journey_candidates.len(), 1);
        let c = &a.journey_candidates[0];
        assert_eq!(c.trigger_backend_id, Some(11));
        assert_eq!(c.required_journey, JourneyKind::DisclosureToggle);
        assert_eq!(c.pattern_kind, PatternKind::Disclosure);
    }

    /// `<details name>` — die Gruppenvariante trägt eine eigene Rolle.
    #[test]
    fn gruppiertes_details_zaehlt_ebenso() {
        let tree = AXTree::from_nodes(vec![expanded("1", "DisclosureTriangleGrouped", 12)]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 1);
    }

    /// Ohne Aufklappzustand ist ein Dreieck kein Auslöser.
    #[test]
    fn summary_ohne_zustand_ist_kein_kandidat() {
        let mut n = node_with_prop(
            "1",
            "DisclosureTriangle",
            AXProperty {
                name: "focusable".into(),
                value: AXValue::Bool(true),
            },
        );
        n.backend_dom_node_id = Some(13);
        let tree = AXTree::from_nodes(vec![n]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert!(a.recognized.is_empty());
        assert!(a.journey_candidates.is_empty());
    }

    #[test]
    fn beide_herkuenfte_nebeneinander() {
        let tree = AXTree::from_nodes(vec![
            expanded("1", "button", 21),
            expanded("2", "DisclosureTriangle", 22),
        ]);
        let mut a = PatternAnalysis::default();
        detect(&tree, &mut a);
        assert_eq!(a.journey_candidates.len(), 2);
        assert!(a.recognized[0]
            .message
            .contains("native <details>/<summary>"));
    }
}
