//! Modal journey: Dialog öffnen, Fokusführung und Schließen belegen.
//!
//! # Was hier gemessen wird
//!
//! `snapshot → klicken → snapshot → Differenz`, dann Tastaturschritte mit je
//! einer Aufnahme. Alle Aussagen beziehen sich auf **den Dialog, den dieser
//! Auslöser geöffnet hat** — bestimmt aus der Differenz, nicht aus
//! `document.querySelector('[role=dialog]')`.
//!
//! # Warum nicht über `querySelector`
//!
//! Der vorherige Stand las den Dialog, die Fokuszugehörigkeit und den
//! Schließzustand per JavaScript aus dem DOM. Dieselben drei blinden Flecken
//! wie bei der Disclosure-Journey (Plan 53, Schritt 1) gelten hier ebenso:
//!
//! - `querySelector('[role="dialog"]')` durchdringt weder Shadow Roots noch
//!   iframe-Grenzen und findet **das Attribut**, nicht die Rolle — ein natives
//!   `<dialog>` ohne `role`-Attribut blieb damit unsichtbar, obwohl Chrome ihm
//!   im Accessibility-Tree die Rolle `dialog` gibt.
//! - Es findet den *ersten* Dialog im Dokument, nicht den geöffneten.
//! - Jeder Lesefehler wurde über `unwrap_or(true)` zu stillem Bestehen.
//!
//! Der Hintergrundtest war zusätzlich ohne Substanz: er prüfte, ob *irgendwo*
//! auf der Seite ein `aria-hidden="true"` steht — ein einzelnes dekoratives
//! Icon genügte.
//!
//! # Modal oder nicht entscheidet, was erwartet werden darf
//!
//! Gelesen wird jetzt `modal` am geöffneten Dialog — `aria-modal="true"` oder
//! ein natives `showModal()`. Davon hängt die ganze Bewertung ab:
//!
//! - **Modal**: der Fokus gehört hinein und soll drin bleiben.
//!   `FocusTrapNotEntered` und `FocusTrapEscaped` gelten.
//! - **Nicht modal**: der Fokus darf den Dialog verlassen, der Hintergrund
//!   bleibt bedienbar. Das ist ein anderes Muster, kein Mangel — und wird
//!   deshalb nicht gemeldet.
//! - **Nicht modal, aber der Fokus kommt trotzdem nicht heraus**: das ist der
//!   Befund, den `FocusTrapBackgroundNotHidden` benennt. Die Tastatur ist
//!   gefangen, während der Hintergrund für assistive Technik weiter lesbar
//!   danebensteht.
//!
//! Die vorherige Fassung meldete jeden nicht-modalen Dialog als High-Befund.

use chromiumoxide::Page;

use crate::accessibility::{AXSnapshot, AXTreeDiff};
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{keyboard, pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

/// Wie oft nach dem Öffnen Tab gedrückt wird, um den Fokus-Einschluss zu prüfen.
const TAB_STEPS: usize = 3;

fn is_dialog(role: Option<&str>) -> bool {
    matches!(role, Some("dialog") | Some("alertdialog"))
}

/// Der Dialog, den dieser Auslöser geöffnet hat.
///
/// Entschieden wird über die Differenz: entweder ist der Knoten neu im Baum,
/// oder er hat seine Sichtbarkeit bzw. Modalität gewechselt. Ein Dialog, der
/// vorher und nachher unverändert dasteht, ist nicht der geöffnete.
fn opened_dialog(after: &AXSnapshot, diff: &AXTreeDiff) -> Option<i64> {
    let candidates: Vec<&crate::accessibility::AXNode> = after
        .tree
        .iter()
        .filter(|n| is_dialog(n.role.as_deref()))
        .collect();

    candidates
        .iter()
        .find(|n| diff.added.contains(&n.node_id))
        .or_else(|| {
            candidates.iter().find(|n| {
                n.backend_dom_node_id.is_some_and(|bid| {
                    diff.property_change_for(bid, "modal").is_some()
                        || diff.property_change_for(bid, "hidden").is_some()
                })
            })
        })
        .and_then(|n| n.backend_dom_node_id)
}

/// Ob der Fokus dieser Aufnahme innerhalb des Dialogs steht.
///
/// `None`, solange kein Fokus aufgenommen werden konnte — das ist keine
/// Aussage über den Einschluss und darf nicht als Bestehen gelten.
fn focus_within(snapshot: &AXSnapshot, dialog: i64) -> Option<bool> {
    let focused = snapshot.focus.active_backend_node_id?;
    Some(snapshot.tree.is_within(focused, dialog))
}

/// Ob der Dialog in dieser Aufnahme noch offen steht.
fn still_open(snapshot: &AXSnapshot, dialog: i64) -> bool {
    match snapshot.tree.node_by_backend_id(dialog) {
        Some(node) => !node.get_property_bool("hidden").unwrap_or(false),
        // Aus dem Baum verschwunden: geschlossen.
        None => false,
    }
}

async fn snapshot(page: &Page, label: &str) -> Option<AXSnapshot> {
    match crate::accessibility::capture_snapshot(page, label, 0).await {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            tracing::warn!("modal: Aufnahme '{label}' fehlgeschlagen: {e}");
            None
        }
    }
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let journey_name = format!("modal_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings: Vec<InteractiveFinding> = Vec::new();

    let finding = |kind: InteractiveFindingKind,
                   category: &str,
                   severity: Severity,
                   after: Option<String>,
                   findings: &mut Vec<InteractiveFinding>| {
        findings.push(InteractiveFinding::new(
            category,
            kind,
            None,
            severity,
            journey_name.clone(),
            None,
            after,
            InteractiveFindingValues::default(),
        ));
    };

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
        tracing::warn!("modal: click on backend node {trigger_id} failed: {e}");
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
    let dialog = opened_dialog(&after_open, &open_diff);

    trace.steps.push(JourneyStep {
        action: "check_dialog_opened".to_string(),
        target: dialog.map(|d| format!("backend_node:{d}")),
        focus: None,
        result: Some(match dialog {
            Some(_) => "dialog_opened".to_string(),
            None => "no_dialog_observed".to_string(),
        }),
        snapshot_label: Some("after_open_click".to_string()),
    });

    let Some(dialog) = dialog else {
        // Der Auslöser führt `aria-haspopup="dialog"`, es erscheint aber
        // keiner. Alles Weitere — Fokuseinschluss, Escape, Rückgabe — hat
        // ohne Dialog keinen Gegenstand.
        finding(
            InteractiveFindingKind::ModalNotOpened,
            "StateTransition",
            Severity::Medium,
            Some("after_open_click".to_string()),
            &mut findings,
        );
        return Ok((trace, findings));
    };

    // 1. Ist der Dialog modal? Davon hängt ab, was überhaupt erwartet werden
    //    darf. Ein *nicht*-modaler Dialog schließt den Fokus nicht ein und
    //    nimmt den Hintergrund nicht aus der Wahrnehmung — das ist kein
    //    Mangel, sondern ein anderes Muster.
    let modal = after_open
        .tree
        .node_by_backend_id(dialog)
        .and_then(|n| n.get_property_bool("modal"))
        .unwrap_or(false);
    trace.steps.push(JourneyStep {
        action: "check_dialog_modal".to_string(),
        target: Some(format!("backend_node:{dialog}")),
        focus: None,
        result: Some(if modal { "modal" } else { "not_modal" }.to_string()),
        snapshot_label: Some("after_open_click".to_string()),
    });

    // 2. Steht der Fokus nach dem Öffnen im Dialog?
    let entered = focus_within(&after_open, dialog);
    trace.steps.push(JourneyStep {
        action: "check_focus_in_dialog".to_string(),
        target: Some(format!("backend_node:{dialog}")),
        focus: after_open.focus.selector.clone(),
        result: Some(
            match entered {
                Some(true) => "focus_inside_dialog",
                Some(false) => "focus_not_in_dialog",
                None => "no_focus_captured",
            }
            .to_string(),
        ),
        snapshot_label: Some("after_open_click".to_string()),
    });
    // Nur ein modaler Dialog muss den Fokus aufnehmen.
    if modal && entered == Some(false) {
        finding(
            InteractiveFindingKind::FocusTrapNotEntered,
            "FocusTrap",
            Severity::High,
            Some("after_open_click".to_string()),
            &mut findings,
        );
    }

    // Steht der Fokus nicht im Dialog, prüfen Tab und Escape nichts, was mit
    // diesem Dialog zu tun hat — dieselbe Lehre wie bei der Tab-Liste, wo 39
    // von 40 Befunden Messartefakte waren. Der Grund steht im Trace.
    if entered != Some(true) {
        trace.steps.push(JourneyStep {
            action: "check_focus_in_dialog".to_string(),
            target: Some(format!("backend_node:{dialog}")),
            focus: None,
            result: Some("keyboard_steps_not_applicable".to_string()),
            snapshot_label: Some("after_open_click".to_string()),
        });
        return Ok((trace, findings));
    }

    // 3. Hält der Fokus über Tab-Schritte im Dialog?
    let mut escaped = false;
    for step in 1..=TAB_STEPS {
        if let Err(e) = keyboard::press_tab(page).await {
            tracing::warn!("modal: Tab {step} fehlgeschlagen: {e}");
            break;
        }
        let _ = stability::settle_after_action(page).await;
        let label = format!("after_tab_{step}");
        let Some(shot) = snapshot(page, &label).await else {
            break;
        };
        let inside = focus_within(&shot, dialog);
        trace.steps.push(JourneyStep {
            action: "tab".to_string(),
            target: None,
            focus: shot.focus.selector.clone(),
            result: Some(
                match inside {
                    Some(true) => "focus_in_dialog",
                    Some(false) => "focus_escaped",
                    None => "no_focus_captured",
                }
                .to_string(),
            ),
            snapshot_label: Some(label),
        });
        if inside == Some(false) {
            escaped = true;
            break;
        }
    }
    // Ein modaler Dialog, den der Fokus verlässt: der Einschluss fehlt.
    if modal && escaped {
        finding(
            InteractiveFindingKind::FocusTrapEscaped,
            "FocusTrap",
            Severity::High,
            None,
            &mut findings,
        );
    }

    // Der umgekehrte Fall, und der eigentliche Sinn dieser Befundart: der
    // Dialog schließt den Fokus ein, meldet sich aber nicht als modal. Dann
    // ist die Tastatur gefangen, während der Hintergrund für assistive
    // Technik weiter lesbar danebensteht.
    //
    // Nicht gemeldet wird der schlichte nicht-modale Dialog, den der Fokus
    // erwartungsgemäß verlässt — das ist ein anderes Muster, kein Mangel.
    if !modal && !escaped && entered == Some(true) {
        finding(
            InteractiveFindingKind::FocusTrapBackgroundNotHidden,
            "FocusTrap",
            Severity::High,
            Some("after_open_click".to_string()),
            &mut findings,
        );
    }

    // 4. Schließt Escape den Dialog?
    if let Err(e) = keyboard::press_escape(page).await {
        tracing::warn!("modal: Escape fehlgeschlagen: {e}");
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
    let open_after_escape = still_open(&after_escape, dialog);

    trace.steps.push(JourneyStep {
        action: "check_dialog_closed".to_string(),
        target: Some(format!("backend_node:{dialog}")),
        focus: None,
        result: Some(
            if open_after_escape {
                "dialog_still_open"
            } else {
                "dialog_closed"
            }
            .to_string(),
        ),
        snapshot_label: Some("after_escape".to_string()),
    });

    if open_after_escape {
        finding(
            InteractiveFindingKind::FocusTrapEscapeNotClosing,
            "FocusTrap",
            Severity::High,
            Some("after_escape".to_string()),
            &mut findings,
        );
        return Ok((trace, findings));
    }

    // 5. Kehrt der Fokus zum Auslöser zurück?
    let restored = after_escape.focus.active_backend_node_id;
    trace.steps.push(JourneyStep {
        action: "check_focus_restored".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: after_escape.focus.selector.clone(),
        result: Some(
            match restored {
                Some(id) if id == trigger_id => "focus_on_trigger",
                Some(_) => "focus_elsewhere",
                None => "focus_lost",
            }
            .to_string(),
        ),
        snapshot_label: Some("after_escape".to_string()),
    });

    // Gemeldet wird nur der eindeutige Fall: der Fokus liegt nirgends mehr.
    // „Woanders, aber nicht am Auslöser" kann eine bewusste Entscheidung sein
    // und steht deshalb im Trace, nicht als Befund.
    //
    // `selector` trägt das Element unabhängig von der Backend-ID. Scheitert
    // nur deren Auflösung, ist das ein Aufnahmefehler und kein Fokusverlust.
    //
    // Und die Voraussetzung: zurückgeben lässt sich nur, was vorher dastand.
    // Die Journey betätigt den Auslöser über `element.click()`, und das
    // verschiebt den Fokus nicht — stand er vorher auf `body`, ist „danach auf
    // `body`" die richtige Wiederherstellung, kein Verlust. Ohne diese Prüfung
    // meldet selbst ein natives `showModal()` einen Befund.
    let had_focus_before =
        before.focus.active_backend_node_id.is_some() || before.focus.selector.is_some();
    let focus_lost = restored.is_none() && after_escape.focus.selector.is_none();
    if !had_focus_before {
        trace.steps.push(JourneyStep {
            action: "check_focus_restored".to_string(),
            target: None,
            focus: None,
            result: Some("no_focus_before_open".to_string()),
            snapshot_label: Some("after_escape".to_string()),
        });
    } else if focus_lost {
        finding(
            InteractiveFindingKind::FocusRestorationLostToBody,
            "FocusRestoration",
            Severity::Medium,
            Some("after_escape".to_string()),
            &mut findings,
        );
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

    /// Der Dialog wird über die Differenz bestimmt, nicht über „der erste im
    /// Dokument". Ein zweiter, unveränderter Dialog darf nicht gewinnen.
    #[test]
    fn der_neu_erschienene_dialog_wird_gewaehlt() {
        let before = snap(vec![node("old", 1, "dialog", None)], None);
        let after = snap(
            vec![
                node("old", 1, "dialog", None),
                node("new", 2, "dialog", None),
            ],
            None,
        );
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(opened_dialog(&after, &diff), Some(2));
    }

    /// Ein Dialog, der schon im Baum stand und nur sichtbar wurde, zählt auch.
    #[test]
    fn sichtbar_gewordener_dialog_wird_erkannt() {
        let before = snap(
            vec![with_bool(node("d", 5, "dialog", None), "hidden", true)],
            None,
        );
        let after = snap(
            vec![with_bool(node("d", 5, "dialog", None), "hidden", false)],
            None,
        );
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(opened_dialog(&after, &diff), Some(5));
    }

    /// Ändert sich an keinem Dialog etwas, ist keiner geöffnet worden.
    #[test]
    fn unveraenderter_dialog_gilt_nicht_als_geoeffnet() {
        let before = snap(vec![node("d", 5, "dialog", None)], None);
        let after = snap(vec![node("d", 5, "dialog", None)], None);
        let diff = AXTreeDiff::between(&before, &after);
        assert_eq!(opened_dialog(&after, &diff), None);
    }

    /// Fokuszugehörigkeit wird über die Elternkette im Baum entschieden —
    /// das trägt über Shadow Roots, die `element.contains()` im DOM nicht sieht.
    #[test]
    fn fokus_im_dialog_wird_ueber_die_elternkette_entschieden() {
        let tree = vec![
            node("d", 10, "dialog", None),
            node("inner", 11, "generic", Some("d")),
            node("btn", 12, "button", Some("inner")),
            node("outside", 20, "button", None),
        ];
        assert_eq!(focus_within(&snap(tree.clone(), Some(12)), 10), Some(true));
        assert_eq!(focus_within(&snap(tree.clone(), Some(10)), 10), Some(true));
        assert_eq!(focus_within(&snap(tree.clone(), Some(20)), 10), Some(false));
    }

    /// Ohne aufgenommenen Fokus ist nichts belegt — weder Einschluss noch Bruch.
    #[test]
    fn fehlender_fokus_ist_keine_aussage() {
        let tree = vec![node("d", 10, "dialog", None)];
        assert_eq!(focus_within(&snap(tree, None), 10), None);
    }

    #[test]
    fn geschlossen_heisst_verschwunden_oder_versteckt() {
        let visible = snap(vec![node("d", 10, "dialog", None)], None);
        assert!(still_open(&visible, 10));

        let hidden = snap(
            vec![with_bool(node("d", 10, "dialog", None), "hidden", true)],
            None,
        );
        assert!(!still_open(&hidden, 10));

        let gone = snap(vec![node("other", 99, "button", None)], None);
        assert!(!still_open(&gone, 10));
    }
}
