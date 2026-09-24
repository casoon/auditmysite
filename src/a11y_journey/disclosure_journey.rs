//! Disclosure / accordion journey: Auslöser betätigen, Zustandswechsel belegen.
//!
//! # Was hier gemessen wird
//!
//! `snapshot → klicken → snapshot → Differenz`, zweimal: einmal in die eine
//! Richtung, einmal zurück. Beurteilt wird ausschließlich der Knoten, der
//! tatsächlich betätigt wurde — zugeordnet über die Backend-Node-ID.
//!
//! Die Differenz trennt vier Fälle, die für Screenreader-Nutzende
//! unterschiedlich ausgehen:
//!
//! | Zustand am Auslöser | Inhalt im Baum | Bedeutung |
//! |---|---|---|
//! | wechselt | ändert sich | der Normalfall |
//! | wechselt | unverändert | „ausgeklappt" gehört, nichts zu lesen |
//! | unverändert | ändert sich | etwas öffnet sich, ohne angekündigt zu werden |
//! | unverändert | unverändert | die Betätigung bleibt wirkungslos |
//!
//! # Warum nicht über `aria-expanded` im DOM
//!
//! Bis Plan 53 las diese Journey per JavaScript alle `[aria-expanded]` der
//! Seite in ein Array und verglich positionsweise. Ein Korpuslauf über 133
//! Seiten und 692 Journey-Instanzen hat diesen Weg widerlegt: 87 der 197
//! erzeugten `DisclosureNotOpened` waren falsch. Drei Ursachen, an
//! Einzelseiten nachgewiesen — `document.querySelectorAll` durchdringt weder
//! Shadow Roots noch iframe-Grenzen, und der Positionsvergleich verrutscht,
//! sobald sich die Menge der Elemente ändert. Der Accessibility-Tree hat
//! keine dieser Grenzen. Einzelheiten in `plan/53-journey-diff-umbau.md`.
//!
//! # Richtung
//!
//! Die erwartete Richtung wird aus der Ausgangsaufnahme abgeleitet, nicht
//! unterstellt: ein Auslöser, der beim Start bereits aufgeklappt ist, klappt
//! beim ersten Klick zu. Im Korpus betraf das 30 von 692 Instanzen, die die
//! alte Prüfung samt und sonders als Fehler meldete.

use std::time::Instant;

use chromiumoxide::Page;

use crate::accessibility::{AXSnapshot, AXTreeDiff};
use crate::audit::normalized::{
    InteractiveFinding, InteractiveFindingKind, InteractiveFindingValues, JourneyStep, JourneyTrace,
};
use crate::error::Result;
use crate::interaction::{pointer, stability};
use crate::patterns::JourneyCandidate;
use crate::taxonomy::Severity;

/// Was die Differenz über den bedienten Auslöser sagt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// Zustand gewechselt und Inhalt wurde wahrnehmbar: der Normalfall.
    StateAndContent,
    /// Zustand gewechselt, aber im Baum ändert sich nichts — der Nutzer hört
    /// „ausgeklappt" und bekommt nichts zu lesen.
    StateOnly,
    /// Inhalt erschien, ohne dass der Auslöser seinen Zustand meldet.
    ContentOnly,
    /// Weder Zustand noch Inhalt: die Betätigung blieb wirkungslos.
    NoChange,
    /// Der Zustand wechselte, aber entgegen der erwarteten Richtung.
    ///
    /// Seit die Richtung aus der Ausgangsaufnahme stammt, bleibt dafür genau
    /// ein Fall: der Auslöser trug dort **keinen** Zustand und meldet nach dem
    /// Klick `expanded=false`. Ob er damit zugeklappt hat oder nie aufgeklappt
    /// war, ist ohne Ausgangswert nicht zu entscheiden — im Korpus 9 von 381.
    WrongDirection,
    /// Eine der beiden Aufnahmen fehlt — nicht beurteilbar, kein Bestehen.
    NotObservable,
}

impl Verdict {
    /// Die Kennung im Trace. Stabil gehalten, damit exportierte Snapshots
    /// über den Umbau hinweg vergleichbar bleiben.
    fn as_str(self) -> &'static str {
        match self {
            Verdict::StateAndContent => "diff:state+content",
            Verdict::StateOnly => "diff:state_only",
            Verdict::ContentOnly => "diff:content_only",
            Verdict::NoChange => "diff:no_change",
            Verdict::WrongDirection => "diff:state_wrong_direction",
            Verdict::NotObservable => "diff:not_observable",
        }
    }
}

/// Was aus einem Urteil folgt: welcher Befund, und ob der nächste Klick noch
/// auf einem bekannten Ausgangszustand aufsetzt.
///
/// Der Korpuslauf hat hier einen Doppelbefund aufgedeckt: `ContentOnly` ist
/// bereits die genaue Aussage („der Auslöser meldet seinen Zustand nicht").
/// Sie zusätzlich als „klappt nicht auf" zu melden, ist derselbe Mangel ein
/// zweites Mal — im Korpus 144 überzählige Befunde.
fn outcome(
    verdict: Verdict,
    want_open: bool,
) -> (Option<(InteractiveFindingKind, Severity)>, bool) {
    match verdict {
        Verdict::StateAndContent => (None, true),
        // Der Wechsel kam an, nur wahrnehmbarer Inhalt fehlt. Der Folgeklick
        // setzt weiterhin auf einem bekannten Zustand auf.
        Verdict::StateOnly => (
            Some((
                InteractiveFindingKind::DisclosureStateWithoutContent,
                Severity::Medium,
            )),
            true,
        ),
        Verdict::ContentOnly => (
            Some((
                InteractiveFindingKind::DisclosureContentWithoutState,
                Severity::Medium,
            )),
            false,
        ),
        Verdict::NoChange => (Some(missed_state_kind(want_open)), false),
        // Nichts belegt: kein Befund, der Grund steht im Trace.
        Verdict::WrongDirection | Verdict::NotObservable => (None, false),
    }
}

/// `want` ist der erwartete neue Zustand: `true` beim Aufklappen,
/// `false` beim Zuklappen.
fn diff_verdict(diff: Option<&AXTreeDiff>, trigger: i64, want: bool) -> Verdict {
    let Some(diff) = diff else {
        return Verdict::NotObservable;
    };
    let state = diff
        .property_change_for(trigger, "expanded")
        .map(|c| c.after == want.to_string());
    let content = if want {
        diff.has_additions()
    } else {
        diff.has_removals()
    };

    match (state, content) {
        (Some(true), true) => Verdict::StateAndContent,
        (Some(true), false) => Verdict::StateOnly,
        (Some(false), _) => Verdict::WrongDirection,
        (None, true) => Verdict::ContentOnly,
        (None, false) => Verdict::NoChange,
    }
}

/// Was die Ausgangsaufnahme über den Auslöser sagt.
///
/// Die drei Fälle auseinanderzuhalten ist nicht akademisch: der Korpuslauf
/// fand 89 von 387 Auslösern ohne Aufklappzustand in der Ausgangsaufnahme,
/// obwohl die Mustererkennung sie ausgewählt hatte, *weil* sie einen trugen.
/// Dazwischen liegen die vorangegangenen Journeys, die die Seite verändern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Start {
    /// Steht im Baum und meldet seinen Aufklappzustand.
    State(bool),
    /// Steht im Baum, führt aber keinen Aufklappzustand.
    NoState,
    /// Steht nicht mehr im Baum. Die Mustererkennung hat ihn an einer
    /// früheren Aufnahme gefunden; seitdem hat sich die Seite verändert.
    Gone,
}

impl Start {
    fn as_str(self) -> &'static str {
        match self {
            Start::State(true) => "expanded",
            Start::State(false) => "collapsed",
            Start::NoState => "no_expanded_state",
            Start::Gone => "trigger_not_in_tree",
        }
    }
}

fn start_state(snapshot: &AXSnapshot, trigger: i64) -> Start {
    match snapshot.tree.node_by_backend_id(trigger) {
        Some(node) => match node.get_property_bool("expanded") {
            Some(v) => Start::State(v),
            None => Start::NoState,
        },
        None => Start::Gone,
    }
}

/// Nimmt auf und meldet Misserfolg als solchen, statt still weiterzulaufen.
async fn snapshot(page: &Page, label: &str, started: Instant) -> Option<AXSnapshot> {
    match crate::accessibility::capture_snapshot(page, label, started.elapsed().as_millis() as u64)
        .await
    {
        Ok(snapshot) => Some(snapshot),
        Err(e) => {
            tracing::warn!("disclosure: Aufnahme '{label}' fehlgeschlagen: {e}");
            None
        }
    }
}

/// Die Befundart, wenn der Auslöser den erwarteten Zustand nicht erreicht.
/// Richtet sich nach der Richtung, nicht nach der Reihenfolge der Klicks.
fn missed_state_kind(want_open: bool) -> (InteractiveFindingKind, Severity) {
    if want_open {
        (InteractiveFindingKind::DisclosureNotOpened, Severity::High)
    } else {
        (
            InteractiveFindingKind::DisclosureNotClosed,
            Severity::Medium,
        )
    }
}

/// Sammelt die Befunde einer Journey und hält jede Art nur einmal fest —
/// beide Klickrichtungen zeigen denselben Mangel, das ist ein Befund.
struct Findings {
    journey: String,
    items: Vec<InteractiveFinding>,
}

impl Findings {
    fn push(
        &mut self,
        kind: InteractiveFindingKind,
        severity: Severity,
        before: Option<String>,
        after: String,
    ) {
        if self.items.iter().any(|f| f.kind == kind) {
            return;
        }
        self.items.push(InteractiveFinding::new(
            "StateTransition",
            kind,
            None,
            severity,
            self.journey.clone(),
            before,
            Some(after),
            InteractiveFindingValues::default(),
        ));
    }
}

pub async fn test(
    page: &Page,
    candidate: &JourneyCandidate,
    index: usize,
) -> Result<(JourneyTrace, Vec<InteractiveFinding>)> {
    let started = Instant::now();
    let journey_name = format!("disclosure_{index}");
    let mut trace = JourneyTrace {
        journey: journey_name.clone(),
        steps: Vec::new(),
    };
    let mut findings = Findings {
        journey: journey_name,
        items: Vec::new(),
    };

    let trigger_id = match candidate.trigger_backend_id {
        Some(id) => id,
        None => return Ok((trace, findings.items)),
    };

    let before_snapshot = snapshot(page, "initial", started).await;

    // Die Richtung folgt dem Ausgangszustand: ein bereits aufgeklappter
    // Auslöser klappt beim ersten Klick zu. Ohne Aufnahme bleibt die
    // herkömmliche Annahme „zugeklappt".
    let start = before_snapshot
        .as_ref()
        .map(|s| start_state(s, trigger_id))
        .unwrap_or(Start::NoState);
    let want_open = start != Start::State(true);
    trace.steps.push(JourneyStep {
        action: "initial_state".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: Some(start.as_str().to_string()),
        snapshot_label: Some("initial".to_string()),
    });

    // Der Auslöser aus der Mustererkennung existiert nicht mehr. Ihn
    // trotzdem anzuklicken hieße, ein fremdes Element zu bewerten.
    if start == Start::Gone {
        return Ok((trace, findings.items));
    }

    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("disclosure: click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings.items));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_first_click".to_string()),
    });

    let _ = stability::settle_after_action(page).await;

    let after_first = snapshot(page, "after_first_click", started).await;
    let first_diff = match (&before_snapshot, &after_first) {
        (Some(before), Some(after)) => Some(AXTreeDiff::between(before, after)),
        _ => None,
    };
    let first = diff_verdict(first_diff.as_ref(), trigger_id, want_open);

    trace.steps.push(JourneyStep {
        action: if want_open {
            "check_expanded".to_string()
        } else {
            "check_collapsed".to_string()
        },
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: Some(first.as_str().to_string()),
        snapshot_label: Some("after_first_click".to_string()),
    });

    let (finding, proceed) = outcome(first, want_open);
    if let Some((kind, severity)) = finding {
        findings.push(
            kind,
            severity,
            Some("initial".to_string()),
            "after_first_click".to_string(),
        );
    }
    // Steht der Zustand nach dem ersten Klick nicht fest, hat der zweite
    // keinen bekannten Ausgangspunkt mehr.
    if !proceed {
        return Ok((trace, findings.items));
    }

    if let Err(e) = pointer::synthetic_click_backend(page, trigger_id).await {
        tracing::warn!("disclosure: second click on backend node {trigger_id} failed: {e}");
        return Ok((trace, findings.items));
    }
    trace.steps.push(JourneyStep {
        action: "synthetic_click".to_string(),
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: None,
        snapshot_label: Some("after_second_click".to_string()),
    });

    let _ = stability::settle_after_action(page).await;

    let after_second = snapshot(page, "after_second_click", started).await;
    let second_diff = match (&after_first, &after_second) {
        (Some(before), Some(after)) => Some(AXTreeDiff::between(before, after)),
        _ => None,
    };
    let second = diff_verdict(second_diff.as_ref(), trigger_id, !want_open);

    trace.steps.push(JourneyStep {
        action: if want_open {
            "check_collapsed".to_string()
        } else {
            "check_expanded".to_string()
        },
        target: Some(format!("backend_node:{trigger_id}")),
        focus: None,
        result: Some(second.as_str().to_string()),
        snapshot_label: Some("after_second_click".to_string()),
    });

    if let Some((kind, severity)) = outcome(second, !want_open).0 {
        findings.push(
            kind,
            severity,
            Some("after_first_click".to_string()),
            "after_second_click".to_string(),
        );
    }

    Ok((trace, findings.items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue, FocusSnapshot};

    fn node(ax_id: &str, backend: i64, expanded: Option<bool>) -> AXNode {
        AXNode {
            node_id: ax_id.to_string(),
            ignored: false,
            ignored_reasons: Vec::new(),
            role: Some("button".to_string()),
            name: None,
            name_source: None,
            description: None,
            value: None,
            properties: expanded
                .map(|v| {
                    vec![AXProperty {
                        name: "expanded".to_string(),
                        value: AXValue::Bool(v),
                    }]
                })
                .unwrap_or_default(),
            child_ids: Vec::new(),
            parent_id: None,
            backend_dom_node_id: Some(backend),
        }
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

    fn verdict(before: Vec<AXNode>, after: Vec<AXNode>, want: bool) -> Verdict {
        let diff = AXTreeDiff::between(&snap(before), &snap(after));
        diff_verdict(Some(&diff), 42, want)
    }

    #[test]
    fn aufklappen_mit_neuem_inhalt_ist_der_normalfall() {
        assert_eq!(
            verdict(
                vec![node("a", 42, Some(false))],
                vec![node("a", 42, Some(true)), node("b", 43, None)],
                true
            ),
            Verdict::StateAndContent
        );
    }

    /// Der Nutzer hört „ausgeklappt", bekommt aber nichts zu lesen — ein
    /// Befund, den die attributbasierte Prüfung gar nicht sehen konnte.
    #[test]
    fn zustand_ohne_inhalt_wird_unterschieden() {
        assert_eq!(
            verdict(
                vec![node("a", 42, Some(false))],
                vec![node("a", 42, Some(true))],
                true
            ),
            Verdict::StateOnly
        );
    }

    /// Inhalt erscheint, der Auslöser meldet seinen Zustand aber nicht.
    #[test]
    fn inhalt_ohne_zustandsmeldung_wird_unterschieden() {
        assert_eq!(
            verdict(
                vec![node("a", 42, None)],
                vec![node("a", 42, None), node("b", 43, None)],
                true
            ),
            Verdict::ContentOnly
        );
    }

    #[test]
    fn keine_wahrnehmbare_wirkung() {
        assert_eq!(
            verdict(
                vec![node("a", 42, Some(false))],
                vec![node("a", 42, Some(false))],
                true
            ),
            Verdict::NoChange
        );
    }

    /// Eine Zustandsänderung an einem *anderen* Knoten zählt nicht. Genau
    /// diesen Fall verwechselte die positionsweise Attributprüfung.
    #[test]
    fn fremder_knoten_zaehlt_nicht_als_erfolg() {
        assert_eq!(
            verdict(
                vec![node("a", 42, Some(false)), node("b", 99, Some(false))],
                vec![node("a", 42, Some(false)), node("b", 99, Some(true))],
                true
            ),
            Verdict::NoChange
        );
    }

    /// Eine fehlgeschlagene Aufnahme ist kein Bestehen.
    #[test]
    fn fehlende_aufnahme_ist_nicht_beobachtbar() {
        assert_eq!(diff_verdict(None, 42, true), Verdict::NotObservable);
        assert_eq!(outcome(Verdict::NotObservable, true), (None, false));
    }

    /// Der Ausgangszustand bestimmt die erwartete Richtung. Ein bereits
    /// aufgeklappter Auslöser klappt beim ersten Klick zu — im Korpus 30 von
    /// 692 Instanzen, die die alte Prüfung als Fehler meldete.
    #[test]
    fn richtung_folgt_dem_ausgangszustand() {
        assert_eq!(
            start_state(&snap(vec![node("a", 42, Some(true))]), 42),
            Start::State(true)
        );
        assert_eq!(
            start_state(&snap(vec![node("a", 42, Some(false))]), 42),
            Start::State(false)
        );
    }

    /// „Kein Zustand am Auslöser" und „Auslöser nicht mehr im Baum" sind
    /// verschiedene Dinge. Das zweite heißt: die Seite hat sich seit der
    /// Mustererkennung verändert, und ein Klick träfe ein fremdes Element.
    #[test]
    fn fehlender_zustand_und_fehlender_knoten_sind_verschieden() {
        assert_eq!(
            start_state(&snap(vec![node("a", 42, None)]), 42),
            Start::NoState
        );
        assert_eq!(
            start_state(&snap(vec![node("a", 42, Some(true))]), 7),
            Start::Gone
        );
    }

    /// Ohne Ausgangswert ist eine Zustandsmeldung entgegen der Erwartung
    /// keine Aussage — und darf keinen Befund erzeugen.
    #[test]
    fn falsche_richtung_ohne_ausgangswert_ist_keine_aussage() {
        assert_eq!(
            verdict(
                vec![node("a", 42, None)],
                vec![node("a", 42, Some(false))],
                true
            ),
            Verdict::WrongDirection
        );
        assert_eq!(outcome(Verdict::WrongDirection, true), (None, false));
    }

    /// Zuklappen wird an verschwundenen Knoten gemessen, nicht an
    /// hinzugekommenen.
    #[test]
    fn zuklappen_misst_verschwundenen_inhalt() {
        assert_eq!(
            verdict(
                vec![node("a", 42, Some(true)), node("b", 43, None)],
                vec![node("a", 42, Some(false))],
                false
            ),
            Verdict::StateAndContent
        );
    }

    /// Ein `content_only` ist bereits die genaue Aussage. Zusätzlich „klappt
    /// nicht auf" zu melden, wäre derselbe Mangel ein zweites Mal — im
    /// Korpuslauf 144 überzählige Befunde.
    #[test]
    fn inhalt_ohne_zustand_erzeugt_genau_einen_befund() {
        let (finding, proceed) = outcome(Verdict::ContentOnly, true);
        assert_eq!(
            finding.map(|f| f.0),
            Some(InteractiveFindingKind::DisclosureContentWithoutState)
        );
        assert!(
            !proceed,
            "ohne bekannten Zustand ist der zweite Klick blind"
        );
    }

    /// Zustand ohne Inhalt ist ein Befund, hält den Ablauf aber nicht an:
    /// der Wechsel ist angekommen, der Folgeklick setzt darauf auf.
    #[test]
    fn zustand_ohne_inhalt_laeuft_weiter() {
        let (finding, proceed) = outcome(Verdict::StateOnly, true);
        assert_eq!(
            finding.map(|f| f.0),
            Some(InteractiveFindingKind::DisclosureStateWithoutContent)
        );
        assert!(proceed);
    }

    #[test]
    fn wirkungslose_betaetigung_meldet_die_richtung() {
        assert_eq!(
            outcome(Verdict::NoChange, true).0.map(|f| f.0),
            Some(InteractiveFindingKind::DisclosureNotOpened)
        );
        assert_eq!(
            outcome(Verdict::NoChange, false).0.map(|f| f.0),
            Some(InteractiveFindingKind::DisclosureNotClosed)
        );
    }

    #[test]
    fn befundart_richtet_sich_nach_der_richtung() {
        assert_eq!(
            missed_state_kind(true).0,
            InteractiveFindingKind::DisclosureNotOpened
        );
        assert_eq!(
            missed_state_kind(false).0,
            InteractiveFindingKind::DisclosureNotClosed
        );
    }

    /// Beide Klickrichtungen zeigen denselben Mangel — das bleibt ein Befund.
    #[test]
    fn gleiche_befundart_wird_nicht_doppelt_gemeldet() {
        let mut f = Findings {
            journey: "disclosure_0".to_string(),
            items: Vec::new(),
        };
        f.push(
            InteractiveFindingKind::DisclosureContentWithoutState,
            Severity::Medium,
            None,
            "a".to_string(),
        );
        f.push(
            InteractiveFindingKind::DisclosureContentWithoutState,
            Severity::Medium,
            None,
            "b".to_string(),
        );
        assert_eq!(f.items.len(), 1);
        assert_eq!(f.items[0].after_snapshot_label.as_deref(), Some("a"));
    }
}
