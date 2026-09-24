//! Menu journey: Menü öffnen, Fokus prüfen, mit Escape schließen.
//!
//! # Was hier gemessen wird
//!
//! `snapshot → klicken → snapshot → Differenz`, dann Escape mit einer
//! weiteren Aufnahme. Der Zustandswechsel wird **dem betätigten Auslöser**
//! zugeordnet, das Menü über die Differenz bestimmt.
//!
//! # Warum nicht über `querySelector`
//!
//! Der vorherige Stand fragte für „ist das Menü offen?"
//!
//! ```js
//! document.querySelector('[role="menu"]') !== null ||
//!   [...document.querySelectorAll('[aria-expanded]')].some(e => e.getAttribute('aria-expanded') === 'true')
//! ```
//!
//! Beide Hälften sind seitenweit: ein statisches `role="menu"` irgendwo im
//! Dokument ließ die Prüfung immer bestehen, und die Gegenprobe beim
//! Schließen — *kein* `[aria-expanded="true"]` mehr auf der ganzen Seite —
//! schlug fehl, sobald ein beliebiges anderes Element offen stand. Das
//! erzeugte `MenuEscapeNotClosing` mit Severity High an Seiten, deren Menü
//! einwandfrei schließt.
//!
//! Dazu die drei Grenzen aus Plan 53, Schritt 1: `querySelector` durchdringt
//! weder Shadow Roots noch iframes, und jeder Lesefehler wurde über
//! `unwrap_or` still zu „bestanden".

use chromiumoxide::Page;

use crate::accessibility::{AXSnapshot, AXTreeDiff};
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{keyboard, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

fn is_menu(role: Option<&str>) -> bool {
    matches!(role, Some("menu") | Some("menubar"))
}

/// Das Menü, das dieser Klick sichtbar gemacht hat — neu im Baum oder von
/// `hidden` befreit. Ein Menü, das unverändert dasteht, ist nicht das geöffnete.
fn opened_menu(after: &AXSnapshot, diff: &AXTreeDiff) -> Option<i64> {
    let candidates: Vec<&crate::accessibility::AXNode> = after
        .tree
        .iter()
        .filter(|n| is_menu(n.role.as_deref()))
        .collect();

    candidates
        .iter()
        .find(|n| diff.added.contains(&n.node_id))
        .or_else(|| {
            candidates.iter().find(|n| {
                n.backend_dom_node_id
                    .is_some_and(|bid| diff.property_change_for(bid, "hidden").is_some())
            })
        })
        .and_then(|n| n.backend_dom_node_id)
}

/// Ob der Auslöser selbst den Wechsel meldet.
fn trigger_expanded(diff: &AXTreeDiff, trigger: i64, want: bool) -> bool {
    diff.property_change_for(trigger, "expanded")
        .is_some_and(|c| c.after == want.to_string())
}

async fn snapshot(page: &Page, label: &str) -> Option<AXSnapshot> {
    match crate::accessibility::capture_snapshot(page, label, 0).await {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            tracing::warn!("menu: Aufnahme '{label}' fehlgeschlagen: {e}");
            None
        }
    }
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("menu_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    let Some(before) = snapshot(page, "initial").await else {
        return Ok((trace, findings));
    };

    // Der Auslöser stammt aus der Mustererkennung einer früheren Aufnahme.
    // Steht er nicht mehr im Baum, hat die Seite sich seither verändert —
    // dann wird hier nichts bewertet statt etwas Fremdes.
    if before.tree.node_by_backend_id(trigger_id).is_none() {
        trace.steps.push(JourneyStep {
            action: "initial_state".to_string(),
            target: Some(format!("backend_node:{trigger_id}")),
            focus: None,
            result: Some("trigger_not_in_tree".to_string()),
            snapshot_label: Some("initial".to_string()),
        });
        return Ok((trace, findings));
    }

    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("menu: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_open_click".to_string()),
    });

    let _ = stability::settle_after_action(page).await;

    let Some(after_open) = snapshot(page, "after_open_click").await else {
        return Ok((trace, findings));
    };
    let open_diff = AXTreeDiff::between(&before, &after_open);
    let announced = trigger_expanded(&open_diff, trigger_id, true);
    let menu = opened_menu(&after_open, &open_diff);

    trace.steps.push(JourneyStep {
        action: "check_menu_open".to_string(),
        target: menu.map(|m| format!("backend_node:{m}")),
        focus: None,
        result: Some(
            match (announced, menu.is_some()) {
                (true, true) => "menu_open",
                // Der Auslöser meldet den Wechsel, aber es erscheint kein
                // Knoten mit Menürolle. Häufig eine Navigationsliste statt
                // eines APG-Menüs — kein Mangel, nur nicht weiter prüfbar.
                (true, false) => "expanded_without_menu_role",
                // Ein Menü erscheint, ohne dass der Auslöser das meldet.
                (false, true) => "menu_without_expanded_state",
                (false, false) => "menu_not_open",
            }
            .to_string(),
        ),
        snapshot_label: Some("after_open_click".to_string()),
    });

    if !announced && menu.is_none() {
        findings.push(InteractiveFinding::new(
            "MenuJourney",
            InteractiveFindingKind::MenuNotOpened,
            None,
            Severity::Medium,
            journey_name.clone(),
            Some("initial".to_string()),
            Some("after_open_click".to_string()),
            InteractiveFindingValues::default(),
        ));
        return Ok((trace, findings));
    }

    // Fokus im Menü: nur beurteilbar, wenn ein Menüknoten feststeht.
    let mut focus_inside = None;
    if let Some(menu) = menu {
        let inside = after_open
            .focus
            .active_backend_node_id
            .map(|f| after_open.tree.is_within(f, menu));
        focus_inside = inside;
        trace.steps.push(JourneyStep {
            action: "check_focus_in_menu".to_string(),
            target: Some(format!("backend_node:{menu}")),
            focus: after_open.focus.selector.clone(),
            result: Some(
                match inside {
                    Some(true) => "focus_in_menu",
                    Some(false) => "focus_not_in_menu",
                    None => "no_focus_captured",
                }
                .to_string(),
            ),
            snapshot_label: Some("after_open_click".to_string()),
        });
        if inside == Some(false) {
            findings.push(InteractiveFinding::new(
                "MenuJourney",
                InteractiveFindingKind::MenuFocusNotMoved,
                None,
                Severity::Low,
                journey_name.clone(),
                None,
                Some("after_open_click".to_string()),
                InteractiveFindingValues::default(),
            ));
        }
    }

    // Escape wird nur bewertet, wenn der Fokus tatsächlich im Menü steht.
    // Sonst trifft die Taste die Seite, nicht dieses Menü — und ein
    // `MenuEscapeNotClosing` (High) wäre ein Messartefakt, kein Befund.
    // An der Tab-Liste waren 39 von 40 Befunden genau das.
    if focus_inside != Some(true) {
        trace.steps.push(JourneyStep {
            action: "check_menu_closed".to_string(),
            target: menu.map(|m| format!("backend_node:{m}")),
            focus: None,
            result: Some("escape_not_applicable".to_string()),
            snapshot_label: Some("after_open_click".to_string()),
        });
        return Ok((trace, findings));
    }

    if let Err(e) = keyboard::press_escape(page).await {
        tracing::warn!("menu: Escape press failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "escape".to_string(),
        target: None,
        focus: None,
        result: None,
        snapshot_label: Some("after_escape".to_string()),
    });

    let _ = stability::settle_after_action(page).await;

    let Some(after_escape) = snapshot(page, "after_escape").await else {
        return Ok((trace, findings));
    };
    let close_diff = AXTreeDiff::between(&after_open, &after_escape);

    // Geschlossen heißt: der Auslöser nimmt seinen Zustand zurück, oder der
    // Menüknoten ist weg bzw. versteckt. Beides bezogen auf *dieses* Menü.
    let retracted = trigger_expanded(&close_diff, trigger_id, false);
    let menu_gone = menu.is_none_or(|m| match after_escape.tree.node_by_backend_id(m) {
        Some(node) => node.get_property_bool("hidden").unwrap_or(false),
        None => true,
    });
    let closed = retracted || menu_gone;

    trace.steps.push(JourneyStep {
        action: "check_menu_closed".to_string(),
        target: menu.map(|m| format!("backend_node:{m}")),
        focus: None,
        result: Some(
            if closed {
                "menu_closed"
            } else {
                "menu_still_open"
            }
            .to_string(),
        ),
        snapshot_label: Some("after_escape".to_string()),
    });

    if !closed {
        findings.push(InteractiveFinding::new(
            "MenuJourney",
            InteractiveFindingKind::MenuEscapeNotClosing,
            None,
            Severity::High,
            journey_name,
            Some("after_open_click".to_string()),
            Some("after_escape".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue, FocusSnapshot};

    fn node(ax_id: &str, backend: i64, role: &str, parent: Option<&str>) -> AXNode {
        AXNode {
            node_id: ax_id.to_string(),
            ignored: false,
            ignored_reasons: Vec::new(),
            role: Some(role.to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: Vec::new(),
            child_ids: Vec::new(),
            parent_id: parent.map(String::from),
            backend_dom_node_id: Some(backend),
        }
    }

    fn with_bool(mut n: AXNode, name: &str, value: bool) -> AXNode {
        n.properties.push(AXProperty {
            name: name.to_string(),
            value: AXValue::Bool(value),
        });
        n
    }

    fn snap(nodes: Vec<AXNode>) -> AXSnapshot {
        AXSnapshot::new(
            "s",
            "https://x",
            "T",
            0,
            AXTree::from_nodes(nodes),
            FocusSnapshot::default(),
        )
    }

    /// Ein statisches `role="menu"` irgendwo im Dokument darf nicht als
    /// „geöffnet" zählen — genau daran bestand die alte Prüfung immer.
    #[test]
    fn unveraendertes_menue_gilt_nicht_als_geoeffnet() {
        let before = snap(vec![node("m", 1, "menu", None)]);
        let after = snap(vec![node("m", 1, "menu", None)]);
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(opened_menu(&after, &diff), None);
    }

    #[test]
    fn neu_erschienenes_menue_wird_gefunden() {
        let before = snap(vec![node("t", 9, "button", None)]);
        let after = snap(vec![
            node("t", 9, "button", None),
            node("m", 1, "menu", None),
        ]);
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(opened_menu(&after, &diff), Some(1));
    }

    #[test]
    fn sichtbar_gewordenes_menue_wird_gefunden() {
        let before = snap(vec![with_bool(
            node("m", 1, "menubar", None),
            "hidden",
            true,
        )]);
        let after = snap(vec![with_bool(
            node("m", 1, "menubar", None),
            "hidden",
            false,
        )]);
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(opened_menu(&after, &diff), Some(1));
    }

    /// Der Zustandswechsel zählt nur am betätigten Auslöser, nicht irgendwo.
    #[test]
    fn zustand_zaehlt_nur_am_ausloeser() {
        let before = snap(vec![
            with_bool(node("t", 9, "button", None), "expanded", false),
            with_bool(node("o", 8, "button", None), "expanded", false),
        ]);
        let after = snap(vec![
            with_bool(node("t", 9, "button", None), "expanded", false),
            with_bool(node("o", 8, "button", None), "expanded", true),
        ]);
        let diff = AXTreeDiff::between(&before, &after);
        assert!(!trigger_expanded(&diff, 9, true));
        assert!(trigger_expanded(&diff, 8, true));
    }
}
