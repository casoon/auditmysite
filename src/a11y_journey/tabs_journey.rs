//! Tabs journey: ersten Tab betätigen, dann mit Pfeil rechts weiterwandern.
//!
//! # Was hier gemessen wird
//!
//! `snapshot → klicken → snapshot → Pfeil rechts → snapshot`. Beurteilt wird
//! die Differenz der letzten beiden Aufnahmen: wechselt `selected` an einem
//! Knoten mit der Rolle `tab`, und steht der Fokus danach auf einem Tab.
//!
//! # Warum nicht über `querySelectorAll('[role="tab"]')`
//!
//! Der vorherige Stand las alle `aria-selected`-Werte der Seite in ein Array
//! und verglich `before != after` — seitenweit, also auch empfindlich für
//! jede unbeteiligte Änderung, und blind für Shadow Roots und iframes
//! (Plan 53, Schritt 1).
//!
//! Die Fokusprüfung war gar keine. Sie lautete
//!
//! ```text
//! !selector.to_lowercase().contains("body")
//! ```
//!
//! mit dem Kommentar „if we can read focus and it's not body, assume it's on
//! a tab". Der Selektor eines Fokusknotens trägt den Pfad, und Pfade der Form
//! `html > body > …` enthalten „body" — an solchen Seiten meldete die Prüfung
//! `TabsFocusNotOnTab`, ohne je eine Rolle angesehen zu haben. Jetzt wird die
//! Rolle des fokussierten Knotens gelesen.
//!
//! # Manuelle Aktivierung ist kein Mangel
//!
//! Die APG kennt zwei Tab-Muster: automatische Aktivierung (Pfeiltaste
//! verschiebt Fokus *und* Auswahl) und manuelle (Pfeiltaste verschiebt nur den
//! Fokus, Enter oder Leertaste aktiviert). Der vorherige Stand meldete das
//! zweite Muster als `TabsSelectionNotMoved` mit Severity High. Gemeldet wird
//! jetzt nur noch der Fall, in dem sich **weder** Auswahl **noch** Fokus
//! bewegt — die Tab-Liste also gar nicht auf die Pfeiltaste reagiert.

use chromiumoxide::Page;

use crate::accessibility::{AXSnapshot, AXTreeDiff};
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{keyboard, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

/// Ob die Differenz einen Auswahlwechsel an einem Tab zeigt.
///
/// Verlangt wird beides: die Eigenschaft `selected` und die Rolle `tab` am
/// selben Knoten. Ein `selected` an einer Option oder einem Baumknoten
/// irgendwo auf der Seite zählt nicht.
fn selection_moved(after: &AXSnapshot, diff: &AXTreeDiff) -> bool {
    diff.property_changes
        .iter()
        .filter(|c| c.property == "selected")
        .any(|c| {
            c.backend_node_id
                .and_then(|bid| after.tree.node_by_backend_id(bid))
                .and_then(|n| n.role.as_deref())
                == Some("tab")
        })
}

/// Der fokussierte Knoten, sofern er eine Tab-Rolle trägt.
fn focused_tab(snapshot: &AXSnapshot) -> Option<i64> {
    let focused = snapshot.focus.active_backend_node_id?;
    let node = snapshot.tree.node_by_backend_id(focused)?;
    (node.role.as_deref() == Some("tab")).then_some(focused)
}

async fn snapshot(page: &Page, label: &str) -> Option<AXSnapshot> {
    match AXSnapshot::capture(page, label, 0).await {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            tracing::warn!("tabs: Aufnahme '{label}' fehlgeschlagen: {e}");
            None
        }
    }
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("tabs_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings)),
    };

    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("tabs: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_first_tab_click".to_string()),
    });

    stability::settle(page).await?;

    let Some(after_click) = snapshot(page, "after_first_tab_click").await else {
        return Ok((trace, findings));
    };

    if let Err(e) = keyboard::press_arrow(page, "Right").await {
        tracing::warn!("tabs: ArrowRight failed: {e}");
        return Ok((trace, findings));
    }
    trace.steps.push(JourneyStep {
        action: "arrow_right".to_string(),
        target: None,
        focus: None,
        result: None,
        snapshot_label: Some("after_arrow_right".to_string()),
    });

    stability::settle(page).await?;

    let Some(after_arrow) = snapshot(page, "after_arrow_right").await else {
        return Ok((trace, findings));
    };
    let diff = AXTreeDiff::between(&after_click, &after_arrow);

    let moved = selection_moved(&after_arrow, &diff);
    let focus_before = focused_tab(&after_click);
    let focus_after = focused_tab(&after_arrow);
    let focus_moved_to_other_tab = focus_after.is_some() && focus_after != focus_before;

    trace.steps.push(JourneyStep {
        action: "check_selection".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: after_arrow.focus.selector.clone(),
        result: Some(
            match (moved, focus_moved_to_other_tab) {
                (true, _) => "selection_moved",
                // Fokus wandert, Auswahl folgt erst auf Enter: das zweite
                // APG-Muster, kein Mangel.
                (false, true) => "focus_moved_manual_activation",
                (false, false) => "selection_unchanged",
            }
            .to_string(),
        ),
        snapshot_label: Some("after_arrow_right".to_string()),
    });

    if !moved && !focus_moved_to_other_tab {
        findings.push(InteractiveFinding::new(
            "TabsJourney",
            InteractiveFindingKind::TabsSelectionNotMoved,
            None,
            Severity::High,
            journey_name.clone(),
            Some("after_first_tab_click".to_string()),
            Some("after_arrow_right".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    // Nach der Pfeiltaste gehört der Fokus auf einen Tab. Ohne aufgenommenen
    // Fokus ist nichts belegt — dann bleibt es beim Trace-Eintrag.
    let focus_captured = after_arrow.focus.active_backend_node_id.is_some();
    trace.steps.push(JourneyStep {
        action: "check_focus_on_tab".to_string(),
        target: None,
        focus: after_arrow.focus.selector.clone(),
        result: Some(
            match (focus_captured, focus_after.is_some()) {
                (_, true) => "focus_on_tab",
                (true, false) => "focus_not_on_tab",
                (false, false) => "no_focus_captured",
            }
            .to_string(),
        ),
        snapshot_label: Some("after_arrow_right".to_string()),
    });

    if focus_captured && focus_after.is_none() {
        findings.push(InteractiveFinding::new(
            "TabsJourney",
            InteractiveFindingKind::TabsFocusNotOnTab,
            None,
            Severity::Medium,
            journey_name,
            None,
            Some("after_arrow_right".to_string()),
            InteractiveFindingValues::default(),
        ));
    }

    Ok((trace, findings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue, FocusSnapshot};

    fn node(ax_id: &str, backend: i64, role: &str, selected: Option<bool>) -> AXNode {
        AXNode {
            node_id: ax_id.to_string(),
            ignored: false,
            ignored_reasons: Vec::new(),
            role: Some(role.to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: selected
                .map(|v| {
                    vec![AXProperty {
                        name: "selected".to_string(),
                        value: AXValue::Bool(v),
                    }]
                })
                .unwrap_or_default(),
            child_ids: Vec::new(),
            parent_id: None,
            backend_dom_node_id: Some(backend),
        }
    }

    fn snap(nodes: Vec<AXNode>, focus: Option<i64>) -> AXSnapshot {
        AXSnapshot::new(
            "s",
            "https://x",
            "T",
            0,
            AXTree::from_nodes(nodes),
            FocusSnapshot {
                active_backend_node_id: focus,
                ..Default::default()
            },
        )
    }

    #[test]
    fn auswahlwechsel_an_einem_tab_zaehlt() {
        let before = snap(
            vec![
                node("a", 1, "tab", Some(true)),
                node("b", 2, "tab", Some(false)),
            ],
            None,
        );
        let after = snap(
            vec![
                node("a", 1, "tab", Some(false)),
                node("b", 2, "tab", Some(true)),
            ],
            None,
        );
        let diff = AXTreeDiff::between(&before, &after);
        assert!(selection_moved(&after, &diff));
    }

    /// `selected` gibt es auch an Optionen und Baumknoten. Nur die Tab-Rolle
    /// zählt — sonst bewertet die Journey eine fremde Liste.
    #[test]
    fn auswahlwechsel_an_fremder_rolle_zaehlt_nicht() {
        let before = snap(vec![node("o", 5, "option", Some(false))], None);
        let after = snap(vec![node("o", 5, "option", Some(true))], None);
        let diff = AXTreeDiff::between(&before, &after);
        assert!(!selection_moved(&after, &diff));
    }

    /// Die Rolle entscheidet, nicht der Selektortext. Der alte Test
    /// `!selector.contains("body")` schlug bei Pfaden wie `html > body > a` an.
    #[test]
    fn fokus_auf_tab_wird_ueber_die_rolle_entschieden() {
        let tree = vec![node("t", 1, "tab", None), node("l", 2, "link", None)];
        assert_eq!(focused_tab(&snap(tree.clone(), Some(1))), Some(1));
        assert_eq!(focused_tab(&snap(tree.clone(), Some(2))), None);
        assert_eq!(focused_tab(&snap(tree, None)), None);
    }
}
