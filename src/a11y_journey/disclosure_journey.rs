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

use crate::accessibility::{AXSnapshot, AXTree, AXTreeDiff, AXValue};
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

/// Der Bereich, den der Auslöser steuert, als Backend-ID.
///
/// `aria-controls` (im Baum die Beziehung `controls`), sonst beim `<summary>`
/// — Rolle `DisclosureTriangle` — das umgebende `<details>`. Mehr gibt der
/// Baum nicht verlässlich her; ohne Bereich bleibt die Inhaltsaussage
/// seitenweit, und ein nachgeladenes Bild am Seitenende zählt mit.
fn controlled_region(snapshot: &AXSnapshot, trigger: i64) -> Option<i64> {
    let node = snapshot.tree.node_by_backend_id(trigger)?;
    let controls = node
        .properties
        .iter()
        .find_map(|p| match (&p.name[..], &p.value) {
            ("controls", AXValue::Node { related_nodes }) => {
                related_nodes.iter().find_map(|r| r.backend_dom_node_id)
            }
            _ => None,
        });
    if controls.is_some() {
        return controls;
    }
    if node.role.as_deref() == Some("DisclosureTriangle") {
        let parent = node.parent_id.as_deref()?;
        return snapshot.tree.get_node(parent)?.backend_dom_node_id;
    }
    None
}

/// Ob der Knoten `node_id` in `tree` unterhalb des Bereichs `region` liegt
/// (oder der Bereich selbst ist). Über die Elternkette der Knoten-IDs, damit
/// auch Knoten ohne eigene Backend-ID — Text, Pseudo-Inhalt — zählen.
fn within_region(tree: &AXTree, node_id: &str, region: i64) -> bool {
    let mut current = tree.get_node(node_id);
    // Die Kette kommt aus einem fremden Prozess; begrenzt, falls sie im Kreis zeigt.
    for _ in 0..1024 {
        let Some(node) = current else {
            return false;
        };
        if node.backend_dom_node_id == Some(region) {
            return true;
        }
        current = node.parent_id.as_deref().and_then(|id| tree.get_node(id));
    }
    false
}

/// Ob Inhalt wahrnehmbar geworden (`want` = aufklappen) bzw. verschwunden ist
/// — im gesteuerten Bereich, falls bekannt, sonst seitenweit.
fn content_changed(
    diff: &AXTreeDiff,
    before: &AXSnapshot,
    after: &AXSnapshot,
    region: Option<i64>,
    want: bool,
) -> bool {
    match (region, want) {
        (None, true) => diff.has_additions(),
        (None, false) => diff.has_removals(),
        // `added` trägt Kennungen aus dem späteren, `removed` aus dem früheren Baum.
        (Some(r), true) => diff
            .added
            .iter()
            .any(|id| within_region(&after.tree, id, r)),
        (Some(r), false) => diff
            .removed
            .iter()
            .any(|id| within_region(&before.tree, id, r)),
    }
}

/// `want` ist der erwartete neue Zustand: `true` beim Aufklappen,
/// `false` beim Zuklappen.
fn verdict_between(
    before: Option<&AXSnapshot>,
    after: Option<&AXSnapshot>,
    trigger: i64,
    region: Option<i64>,
    want: bool,
) -> Verdict {
    let (Some(before), Some(after)) = (before, after) else {
        return Verdict::NotObservable;
    };
    let diff = AXTreeDiff::between(before, after);
    let state = diff
        .property_change_for(trigger, "expanded")
        .map(|c| c.after == want.to_string());
    let content = content_changed(&diff, before, after, region, want);

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
    let region = before_snapshot
        .as_ref()
        .and_then(|s| controlled_region(s, trigger_id));
    trace.steps.push(JourneyStep {
        action: "controlled_region".to_string(),
        target: region.map(|r| format!("backend_node:{r}")),
        focus: None,
        result: Some(
            if region.is_some() {
                "scoped"
            } else {
                "page_wide"
            }
            .to_string(),
        ),
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
    let first = verdict_between(
        before_snapshot.as_ref(),
        after_first.as_ref(),
        trigger_id,
        region,
        want_open,
    );

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
    let second = verdict_between(
        after_first.as_ref(),
        after_second.as_ref(),
        trigger_id,
        region,
        !want_open,
    );

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
        verdict_between(Some(&snap(before)), Some(&snap(after)), 42, None, want)
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
        assert_eq!(
            verdict_between(None, None, 42, None, true),
            Verdict::NotObservable
        );
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

    fn child(ax_id: &str, backend: i64, parent: &str) -> AXNode {
        AXNode {
            parent_id: Some(parent.to_string()),
            role: Some("StaticText".to_string()),
            ..node(ax_id, backend, None)
        }
    }

    fn controls(mut trigger: AXNode, region: i64) -> AXNode {
        trigger.properties.push(AXProperty {
            name: "controls".to_string(),
            value: AXValue::Node {
                related_nodes: vec![crate::accessibility::RelatedNode {
                    backend_dom_node_id: Some(region),
                    idref: None,
                    text: None,
                }],
            },
        });
        trigger
    }

    /// Mit bekanntem Bereich zählt nur, was darin erscheint. Vorher meldete ein
    /// nachgeladenes Bild am Seitenende „Inhalt erschienen" — und machte aus
    /// einem `StateOnly` („ausgeklappt" gehört, nichts zu lesen) ein Bestehen.
    #[test]
    fn inhalt_ausserhalb_des_gesteuerten_bereichs_zaehlt_nicht() {
        let region = node("panel", 50, None);
        let before = vec![controls(node("a", 42, Some(false)), 50), region.clone()];
        let after_elsewhere = vec![
            controls(node("a", 42, Some(true)), 50),
            region.clone(),
            node("footer-img", 77, None),
        ];
        let after_inside = vec![
            controls(node("a", 42, Some(true)), 50),
            region,
            child("text", 51, "panel"),
        ];
        let b = snap(before);
        assert_eq!(controlled_region(&b, 42), Some(50));
        assert_eq!(
            verdict_between(Some(&b), Some(&snap(after_elsewhere)), 42, Some(50), true),
            Verdict::StateOnly
        );
        assert_eq!(
            verdict_between(Some(&b), Some(&snap(after_inside)), 42, Some(50), true),
            Verdict::StateAndContent
        );
    }

    /// Beim Zuklappen zählt, was *im Bereich* verschwindet — gesucht im
    /// früheren Baum, aus dem die Kennungen in `removed` stammen.
    #[test]
    fn zuklappen_prueft_verschwundenes_im_frueheren_baum() {
        let region = node("panel", 50, None);
        let before = vec![
            controls(node("a", 42, Some(true)), 50),
            region.clone(),
            child("text", 51, "panel"),
        ];
        let after = vec![controls(node("a", 42, Some(false)), 50), region];
        assert_eq!(
            verdict_between(Some(&snap(before)), Some(&snap(after)), 42, Some(50), false),
            Verdict::StateAndContent
        );
    }

    /// `<summary>` trägt kein `aria-controls`; der Bereich ist das umgebende
    /// `<details>`.
    #[test]
    fn summary_steuert_das_umgebende_details() {
        let details = node("details", 60, None);
        let summary = AXNode {
            parent_id: Some("details".to_string()),
            role: Some("DisclosureTriangle".to_string()),
            ..node("summary", 42, Some(false))
        };
        assert_eq!(
            controlled_region(&snap(vec![details, summary]), 42),
            Some(60)
        );
    }

    /// Ohne `aria-controls` und ohne `<details>` gibt es keinen Bereich.
    #[test]
    fn ohne_beziehung_bleibt_die_aussage_seitenweit() {
        assert_eq!(
            controlled_region(&snap(vec![node("a", 42, Some(false))]), 42),
            None
        );
    }
}
