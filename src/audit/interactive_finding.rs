//! Accessibility-journey layer: journey/trace types and the interactive
//! (journey) finding model with its #406 en/de text source.
//!
//! Moved out of `audit::normalized`; every item stays reachable at its old
//! `auditmysite::audit::normalized::…` path via `pub use`.

use serde::{Deserialize, Serialize};

use crate::taxonomy::Severity;

// ---------------------------------------------------------------------------
// Accessibility-Journey-Layer (Phase 1: Foundation only — types live here so
// `NormalizedReport` is schema-stable for all future phases.)
// ---------------------------------------------------------------------------

/// Bundle of accessibility-journey results for one page.
/// Populated only when `--interactive != off`; otherwise the report's
/// `accessibility_journey` field stays `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessibilityJourney {
    /// Execution coverage for the interactive layer. This is separate from
    /// findings so "nothing found" remains distinguishable from "not run".
    #[serde(default)]
    pub execution: JourneyExecution,
    /// Reproducible step sequences (one per journey: tab walk, modal open, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traces: Vec<JourneyTrace>,
    /// Compact per-step focus evidence from the tab walk. This deliberately
    /// excludes AXTree snapshots while retaining the visual/focus facts needed
    /// to reproduce and review the automated conclusion.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub focus_evidence: Vec<crate::accessibility::FocusSnapshot>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JourneyExecution {
    pub mode: String,
    pub budget_ms: u64,
    #[serde(default)]
    pub candidates_detected: usize,
    #[serde(default)]
    pub attempted: usize,
    #[serde(default)]
    pub completed: usize,
    #[serde(default)]
    pub failed: usize,
    #[serde(default)]
    pub skipped: usize,
    #[serde(default)]
    pub budget_exhausted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<JourneyRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyRun {
    pub journey: String,
    pub status: crate::audit::ExecutionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

/// One reproducible journey — an ordered list of interaction steps and the
/// snapshots captured along the way. The trace is the *evidence* attached to
/// every interactive finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyTrace {
    /// Journey identifier: "tab_walk", "skip_link", "modal_contact", ...
    pub journey: String,
    /// Ordered steps that compose the journey.
    pub steps: Vec<JourneyStep>,
}

/// A single step in a journey. Designed to read naturally as JSON so a
/// developer can reproduce the journey by replaying the actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStep {
    /// "tab" | "shift_tab" | "enter" | "escape" | "arrow_down" | "click"
    /// | "synthetic_click" (fallback) | "type" | "wait"
    pub action: String,
    /// Selector or descriptive label of the target, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Selector of `document.activeElement` after the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
    /// Human-readable outcome marker, e.g. "modal_opened",
    /// "focus_lost_to_body", "no_change".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Label of the AXSnapshot captured after this step (matches
    /// `AXSnapshot.label`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_label: Option<String>,
}

/// Finding produced by an interactive (journey) test. Distinct from WCAG
/// `findings[]` — does not feed `severity_counts` or `legal_flags`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveFinding {
    /// "TabOrder" | "FocusTrap" | "StateTransition" | "FocusRestoration"
    /// | "FormError" | "SpaNavigation" | "HiddenFocusable" | "SkipLink"
    /// | "FocusIndicator" | "MenuJourney" | "TabsJourney"
    pub category: String,
    /// Stable identifier for the concrete message shape (for localized
    /// re-derivation by [`interactive_finding_text`], #406).
    pub kind: InteractiveFindingKind,
    /// WCAG finding rule ID this journey finding confirms, when it maps 1:1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maps_to_finding: Option<String>,
    pub severity: Severity,
    /// Which journey produced this finding (matches `JourneyTrace.journey`).
    pub journey: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_snapshot_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_snapshot_label: Option<String>,
    /// Message (canonical English; derived from `kind` + `values`)
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_suggestion: Option<String>,
    /// Interpolated values needed to reproduce `message`/`fix_suggestion` in
    /// another language (see [`interactive_finding_text`]).
    #[serde(default)]
    pub values: InteractiveFindingValues,
    /// Set when the observation behind the finding may have missed the
    /// page's reaction (plan 53). Absent means no known reason for doubt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<FindingUncertainty>,
}

/// Why a journey finding may not hold, with the raw values to phrase it.
///
/// Canonical `kind` plus values (#406): [`finding_uncertainty_text`] is the
/// only text source, English for the JSON consumers, localized in the PDF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FindingUncertainty {
    /// The page went quiet `waited_ms` after the action, before the settle
    /// budget ran out. A reaction arriving later was not observed — measured
    /// on fixtures, a reaction later than ~200 ms flips the verdict.
    LateReactionPossible { waited_ms: u64 },
}

pub fn finding_uncertainty_text(uncertainty: &FindingUncertainty, en: bool) -> String {
    match uncertainty {
        FindingUncertainty::LateReactionPossible { waited_ms } => {
            if en {
                format!(
                    "Uncertain: the page settled {waited_ms} ms after the click; \
                     a slower reaction would not have been observed."
                )
            } else {
                format!(
                    "Unsicher: Die Seite kam {waited_ms} ms nach dem Klick zur Ruhe; \
                     eine langsamere Reaktion wäre nicht erfasst worden."
                )
            }
        }
    }
}

impl InteractiveFinding {
    /// Build an `InteractiveFinding`, baking canonical-English `message`/
    /// `fix_suggestion` from `kind` + `values` (#406).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        category: &str,
        kind: InteractiveFindingKind,
        maps_to_finding: Option<String>,
        severity: Severity,
        journey: String,
        before_snapshot_label: Option<String>,
        after_snapshot_label: Option<String>,
        values: InteractiveFindingValues,
    ) -> Self {
        let (message, fix_suggestion) = interactive_finding_text(kind, &values, true);
        InteractiveFinding {
            category: category.to_string(),
            kind,
            maps_to_finding,
            severity,
            journey,
            before_snapshot_label,
            after_snapshot_label,
            message,
            fix_suggestion,
            values,
            uncertainty: None,
        }
    }

    pub fn with_uncertainty(mut self, uncertainty: Option<FindingUncertainty>) -> Self {
        self.uncertainty = uncertainty;
        self
    }
}

/// Stable identifier for a concrete [`InteractiveFinding`] message shape.
///
/// One variant per distinct problem/fix-suggestion template. Together with
/// [`InteractiveFindingValues`] this fully reproduces the human-readable
/// strings in any language (#406).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractiveFindingKind {
    HiddenFocusableAriaHidden,
    HiddenFocusableInert,
    HiddenFocusableStyle,
    FocusIndicatorNotDetected,
    TabOrderBackwardJumps,
    FocusTrapNotEntered,
    FocusTrapBackgroundNotHidden,
    FocusTrapEscaped,
    FocusTrapEscapeNotClosing,
    FocusRestorationLostToBody,
    ModalNotOpened,
    MenuNotOpened,
    MenuFocusNotMoved,
    MenuEscapeNotClosing,
    TabsSelectionNotMoved,
    TabsFocusNotOnTab,
    DisclosureNotOpened,
    DisclosureNotClosed,
    DisclosureStateWithoutContent,
    DisclosureContentWithoutState,
    SpaNoAnnouncementSignal,
    SpaTitleUnchanged,
    SpaFocusNotMoved,
    SkipLinkFocusNotMoved,
    FormErrorSilentFailure,
    FormErrorInvalidWithoutLiveRegion,
    FormErrorUnlinkedFields,
    FormErrorFocusNotManaged,
    FormErrorLiveRegionLateInsertion,
    AddToCartNoStatusAnnouncement,
    AddToCartNoFeedbackDetected,
    QuantityStepperKeyboardInoperable,
    QuantityStepperValueNotExposed,
    LinkTextGeneric,
    LinkTextDuplicate,
    HeadingMissingH1,
    HeadingMultipleH1,
    HeadingLevelSkip,
    LandmarkMissingMain,
    LandmarkNavWithoutLabels,
    LandmarkDuplicateUnique,
    MediaControlsMissingName,
    MediaControlsNotReachable,
}

/// The interpolated values an [`InteractiveFinding`] message may reference.
///
/// Stored on every `InteractiveFinding` so that [`interactive_finding_text`]
/// can reproduce the strings in any locale. Only the fields relevant to the
/// finding's `kind` are populated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InteractiveFindingValues {
    /// CSS selector of the affected element (hidden-focusable / focus-indicator kinds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// A generic count (tab-order jumps, unlinked form fields, generic/duplicate
    /// link texts, multiple H1s, unlabeled nav landmarks, duplicate landmarks).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Comma-joined example list (tab-order jump preview, link-text examples,
    /// heading-skip examples).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<String>,
    /// Whether `examples` was truncated (appends "…" marker to the message).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    /// Page title before an SPA navigation (SpaNoAnnouncementSignal/SpaTitleUnchanged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_before: Option<String>,
    /// Landmark role name (LandmarkDuplicateUnique).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// The single source of truth for `InteractiveFinding` `message`/`fix_suggestion`.
///
/// Returns `(message, fix_suggestion)` in German or English for the given
/// `kind` and interpolated `values`. Producers in `src/a11y_journey/` call
/// this via [`InteractiveFinding::new`] with `en = true` to bake canonical
/// English; the PDF layer re-derives in the run language (#406).
pub fn interactive_finding_text(
    kind: InteractiveFindingKind,
    values: &InteractiveFindingValues,
    en: bool,
) -> (String, Option<String>) {
    use InteractiveFindingKind::*;
    let selector = values.selector.as_deref().unwrap_or("");
    let count = values.count.unwrap_or(0);
    let examples = values.examples.as_deref().unwrap_or("");
    let truncated = values.truncated.unwrap_or(false);
    let title_before = values.title_before.as_deref().unwrap_or("");
    let role = values.role.as_deref().unwrap_or("");

    let (message, fix): (String, Option<String>) = match kind {
        HiddenFocusableAriaHidden => (
            if en {
                format!(
                    "Keyboard focus lands on an element inside an aria-hidden \
                     region ({selector}). Screen reader users reach an element \
                     that is hidden from the accessibility tree."
                )
            } else {
                format!(
                    "Der Tastaturfokus landet auf einem Element innerhalb eines \
                     aria-hidden-Bereichs ({selector}). Screenreader-Nutzer erreichen \
                     ein Element, das im Accessibility Tree verborgen ist."
                )
            },
            Some(if en {
                "Remove the element from the aria-hidden region or set \
                 tabindex=\"-1\" on it."
                    .to_string()
            } else {
                "Element aus dem aria-hidden-Bereich entfernen oder tabindex=\"-1\" \
                 darauf setzen."
                    .to_string()
            }),
        ),
        HiddenFocusableInert => (
            if en {
                format!(
                    "Keyboard focus lands on an element inside an inert \
                     region ({selector}). Inert regions should not be \
                     reachable by keyboard."
                )
            } else {
                format!(
                    "Der Tastaturfokus landet auf einem Element innerhalb eines \
                     inert-Bereichs ({selector}). Inert-Bereiche sollten per Tastatur \
                     nicht erreichbar sein."
                )
            },
            Some(if en {
                "Remove the element from the inert region or \
                 correct the tabindex/focus chain."
                    .to_string()
            } else {
                "Element aus dem inert-Bereich entfernen oder die tabindex-/Fokuskette \
                 korrigieren."
                    .to_string()
            }),
        ),
        HiddenFocusableStyle => (
            if en {
                format!(
                    "Keyboard focus lands on a visually hidden element \
                     ({selector}: display:none, visibility:hidden, or \
                     opacity:0). Keyboard users lose orientation."
                )
            } else {
                format!(
                    "Der Tastaturfokus landet auf einem visuell versteckten Element \
                     ({selector}: display:none, visibility:hidden oder opacity:0). \
                     Tastaturnutzer verlieren die Orientierung."
                )
            },
            Some(if en {
                "Remove the element from the tab sequence (tabindex=\"-1\") \
                 or make it visible before it receives focus."
                    .to_string()
            } else {
                "Element aus der Tab-Reihenfolge entfernen (tabindex=\"-1\") oder es \
                 sichtbar machen, bevor es fokussiert wird."
                    .to_string()
            }),
        ),
        FocusIndicatorNotDetected => (
            if en {
                format!(
                    "Element ({selector}) shows no visible focus indicator when focused \
                     (no outline, no box-shadow, no border change). \
                     Keyboard users lose orientation."
                )
            } else {
                format!(
                    "Element ({selector}) zeigt im fokussierten Zustand keinen sichtbaren \
                     Fokusindikator (kein Outline, kein Box-Shadow, keine Rahmenänderung). \
                     Tastaturnutzer verlieren die Orientierung."
                )
            },
            Some(if en {
                "Add a CSS :focus-visible rule with a clear outline, \
                 box-shadow, or border change compared to the unfocused state."
                    .to_string()
            } else {
                "Eine CSS-:focus-visible-Regel mit deutlichem Outline, Box-Shadow oder \
                 Rahmenwechsel gegenüber dem unfokussierten Zustand ergänzen."
                    .to_string()
            }),
        ),
        TabOrderBackwardJumps => {
            let suffix = if truncated { " (…)" } else { "" };
            (
                if en {
                    format!(
                        "Tab order deviates from DOM order: {count} backward {} \
                         observed. First affected elements: {examples}{suffix}. \
                         Keyboard users may not be able to follow the reading flow.",
                        if count == 1 { "jump" } else { "jumps" }
                    )
                } else {
                    format!(
                        "Die Tab-Reihenfolge weicht von der DOM-Reihenfolge ab: {count} \
                         rückwärtige {} beobachtet. Zuerst betroffene Elemente: \
                         {examples}{suffix}. Tastaturnutzer können dem Lesefluss \
                         möglicherweise nicht folgen.",
                        if count == 1 { "Sprung" } else { "Sprünge" }
                    )
                },
                Some(if en {
                    "Avoid negative or high tabindex values. \
                     Arrange the reading/DOM order to match the visual order."
                        .to_string()
                } else {
                    "Negative oder hohe tabindex-Werte vermeiden. Lese-/DOM-Reihenfolge \
                     an die visuelle Reihenfolge angleichen."
                        .to_string()
                }),
            )
        }
        FocusTrapNotEntered => (
            if en {
                "After opening modal, focus did not move inside the dialog. \
                 Keyboard users cannot interact with it."
                    .to_string()
            } else {
                "Nach dem Öffnen des Modals wechselt der Fokus nicht in den Dialog. \
                 Tastaturnutzer können nicht damit interagieren."
                    .to_string()
            },
            Some(if en {
                "Move focus to the first focusable element inside the dialog when it opens, \
                 or to the dialog element itself (tabindex=\"-1\")."
                    .to_string()
            } else {
                "Beim Öffnen den Fokus auf das erste fokussierbare Element im Dialog setzen \
                 oder auf den Dialog selbst (tabindex=\"-1\")."
                    .to_string()
            }),
        ),
        FocusTrapBackgroundNotHidden => (
            if en {
                "Background content is not hidden from assistive technology when modal is open."
                    .to_string()
            } else {
                "Der Hintergrundinhalt ist bei geöffnetem Modal nicht vor assistiven \
                 Technologien verborgen."
                    .to_string()
            },
            Some(if en {
                "Set aria-hidden=\"true\" on the application root when a modal is open, \
                 or use the inert attribute."
                    .to_string()
            } else {
                "aria-hidden=\"true\" auf dem Anwendungs-Root setzen, solange das Modal \
                 geöffnet ist, oder das inert-Attribut verwenden."
                    .to_string()
            }),
        ),
        FocusTrapEscaped => (
            if en {
                "Focus is not trapped inside the modal dialog. \
                 Keyboard users can navigate to background content."
                    .to_string()
            } else {
                "Der Fokus ist nicht im Modal-Dialog eingeschlossen. Tastaturnutzer können \
                 zum Hintergrundinhalt navigieren."
                    .to_string()
            },
            Some(if en {
                "Intercept Tab and Shift+Tab inside the dialog to cycle focus among \
                 dialog descendants only."
                    .to_string()
            } else {
                "Tab und Umschalt+Tab im Dialog abfangen, sodass der Fokus nur zwischen den \
                 Dialog-Kindelementen wechselt."
                    .to_string()
            }),
        ),
        FocusTrapEscapeNotClosing => (
            if en {
                "Escape key does not close the modal. Keyboard users cannot dismiss it.".to_string()
            } else {
                "Die Escape-Taste schließt das Modal nicht. Tastaturnutzer können es nicht \
                 schließen."
                    .to_string()
            },
            Some(if en {
                "Add a keydown handler on the dialog or document that calls close() \
                 or hides the dialog when Escape is pressed."
                    .to_string()
            } else {
                "Einen keydown-Handler auf dem Dialog oder Dokument ergänzen, der bei Escape \
                 close() aufruft oder den Dialog verbirgt."
                    .to_string()
            }),
        ),
        FocusRestorationLostToBody => (
            if en {
                "After closing the modal, focus returned to body instead of the trigger. \
                 Keyboard users lose their place on the page."
                    .to_string()
            } else {
                "Nach dem Schließen des Modals kehrt der Fokus zu body statt zum \
                 Auslöser-Element zurück. Tastaturnutzer verlieren ihre Position auf der \
                 Seite."
                    .to_string()
            },
            Some(if en {
                "Store a reference to the trigger element before opening the dialog and \
                 call trigger.focus() when the dialog closes."
                    .to_string()
            } else {
                "Vor dem Öffnen des Dialogs eine Referenz auf das Auslöser-Element speichern \
                 und beim Schließen trigger.focus() aufrufen."
                    .to_string()
            }),
        ),
        ModalNotOpened => (
            if en {
                "The trigger announces a dialog via aria-haspopup, but activating it makes \
                 no dialog appear in the accessibility tree."
                    .to_string()
            } else {
                "Der Auslöser kündigt über aria-haspopup einen Dialog an, beim Betätigen \
                 erscheint im Accessibility-Tree aber keiner."
                    .to_string()
            },
            Some(if en {
                "Either open a dialog on activation, or drop aria-haspopup=\"dialog\" from \
                 the trigger so it does not promise something it does not do."
                    .to_string()
            } else {
                "Entweder beim Betätigen einen Dialog öffnen, oder aria-haspopup=\"dialog\" \
                 am Auslöser entfernen, damit er nichts ankündigt, was er nicht tut."
                    .to_string()
            }),
        ),
        MenuNotOpened => (
            if en {
                "Menu trigger was clicked but menu did not open. \
                 Keyboard users cannot access menu items."
                    .to_string()
            } else {
                "Der Menü-Auslöser wurde geklickt, aber das Menü öffnet sich nicht. \
                 Tastaturnutzer erreichen die Menüpunkte nicht."
                    .to_string()
            },
            Some(if en {
                "Set aria-expanded=\"true\" on the trigger and make the menu items visible \
                 when the trigger is activated."
                    .to_string()
            } else {
                "aria-expanded=\"true\" auf dem Auslöser setzen und die Menüpunkte sichtbar \
                 machen, sobald der Auslöser aktiviert wird."
                    .to_string()
            }),
        ),
        MenuFocusNotMoved => (
            if en {
                "After opening menu, focus did not move to menu items. \
                 Keyboard users may not know the menu opened."
                    .to_string()
            } else {
                "Nach dem Öffnen des Menüs wechselt der Fokus nicht zu den Menüpunkten. \
                 Tastaturnutzer bemerken das geöffnete Menü möglicherweise nicht."
                    .to_string()
            },
            Some(if en {
                "Move focus to the first menu item after the menu opens.".to_string()
            } else {
                "Den Fokus nach dem Öffnen auf den ersten Menüpunkt setzen.".to_string()
            }),
        ),
        MenuEscapeNotClosing => (
            if en {
                "Escape key does not close the menu.".to_string()
            } else {
                "Die Escape-Taste schließt das Menü nicht.".to_string()
            },
            Some(if en {
                "Add a keydown handler that closes the menu and returns focus to the trigger \
                 when Escape is pressed."
                    .to_string()
            } else {
                "Einen keydown-Handler ergänzen, der das Menü bei Escape schließt und den \
                 Fokus zum Auslöser zurückgibt."
                    .to_string()
            }),
        ),
        TabsSelectionNotMoved => (
            if en {
                "Arrow key navigation does not move selection between tabs. \
                 Keyboard users cannot navigate the tab list."
                    .to_string()
            } else {
                "Die Pfeiltasten-Navigation verschiebt die Auswahl nicht zwischen den Tabs. \
                 Tastaturnutzer können die Tab-Liste nicht bedienen."
                    .to_string()
            },
            Some(if en {
                "Implement the roving tabindex pattern: ArrowRight moves focus and \
                 aria-selected to the next tab."
                    .to_string()
            } else {
                "Das Roving-Tabindex-Muster implementieren: Pfeil-rechts verschiebt Fokus \
                 und aria-selected zum nächsten Tab."
                    .to_string()
            }),
        ),
        TabsFocusNotOnTab => (
            if en {
                "After pressing ArrowRight in the tab list, focus is not on a tab element."
                    .to_string()
            } else {
                "Nach Pfeil-rechts in der Tab-Liste steht der Fokus nicht auf einem \
                 Tab-Element."
                    .to_string()
            },
            Some(if en {
                "Ensure arrow key navigation also moves focus (not just selection) \
                 to the next tab in the roving tabindex pattern."
                    .to_string()
            } else {
                "Sicherstellen, dass die Pfeiltasten-Navigation im Roving-Tabindex-Muster \
                 auch den Fokus (nicht nur die Auswahl) zum nächsten Tab verschiebt."
                    .to_string()
            }),
        ),
        DisclosureNotOpened => (
            if en {
                "Activating the disclosure trigger does not expand it: neither its own \
                 expanded state nor the accessibility tree changes."
                    .to_string()
            } else {
                "Das Betätigen des Disclosure-Auslösers klappt ihn nicht auf: weder sein \
                 eigener Aufklappzustand noch der Accessibility-Tree ändern sich."
                    .to_string()
            },
            Some(if en {
                "Toggle aria-expanded=\"true|false\" on the activated element and reveal \
                 the controlled region."
                    .to_string()
            } else {
                "aria-expanded=\"true|false\" am betätigten Element umschalten und den \
                 gesteuerten Bereich einblenden."
                    .to_string()
            }),
        ),
        DisclosureNotClosed => (
            if en {
                "Activating the disclosure trigger does not collapse it: neither its own \
                 expanded state nor the accessibility tree changes."
                    .to_string()
            } else {
                "Das Betätigen des Disclosure-Auslösers klappt ihn nicht zu: weder sein \
                 eigener Aufklappzustand noch der Accessibility-Tree ändern sich."
                    .to_string()
            },
            Some(if en {
                "Make the activation handler toggle aria-expanded in both directions, \
                 not just open."
                    .to_string()
            } else {
                "Den Handler aria-expanded in beide Richtungen umschalten lassen, nicht \
                 nur auf."
                    .to_string()
            }),
        ),
        DisclosureStateWithoutContent => (
            if en {
                "The disclosure trigger reports its new expanded state, but nothing enters \
                 or leaves the accessibility tree. Screen reader users hear the state change \
                 and find nothing to read."
                    .to_string()
            } else {
                "Der Disclosure-Auslöser meldet seinen neuen Aufklappzustand, im \
                 Accessibility-Tree kommt aber nichts hinzu und verschwindet nichts. \
                 Screenreader-Nutzende hören den Zustandswechsel und finden nichts zu lesen."
                    .to_string()
            },
            Some(if en {
                "Check that the controlled region really enters the accessibility tree — \
                 display:none, hidden or aria-hidden on the region keep it out."
                    .to_string()
            } else {
                "Prüfen, ob der gesteuerte Bereich tatsächlich in den Accessibility-Tree \
                 kommt — display:none, hidden oder aria-hidden am Bereich halten ihn draußen."
                    .to_string()
            }),
        ),
        DisclosureContentWithoutState => (
            if en {
                "Activating the trigger changes the accessibility tree, but the trigger \
                 itself reports no expanded state. Screen reader users are not told that \
                 something opened or closed."
                    .to_string()
            } else {
                "Beim Betätigen ändert sich der Accessibility-Tree, der Auslöser selbst \
                 meldet aber keinen Aufklappzustand. Screenreader-Nutzende erfahren nicht, \
                 dass sich etwas geöffnet oder geschlossen hat."
                    .to_string()
            },
            Some(if en {
                "Put aria-expanded on the element that is actually activated and toggle it \
                 together with the region."
                    .to_string()
            } else {
                "aria-expanded an das tatsächlich betätigte Element setzen und zusammen mit \
                 dem Bereich umschalten."
                    .to_string()
            }),
        ),
        SpaNoAnnouncementSignal => (
            if en {
                format!(
                    "After SPA navigation neither the page title \
                     (before: {title_before:?}) nor the H1 heading changed, and focus \
                     remained in the same place. Screen readers will not announce \
                     the new content."
                )
            } else {
                format!(
                    "Nach der SPA-Navigation hat sich weder der Seitentitel \
                     (vorher: {title_before:?}) noch die H1-Überschrift geändert, und der \
                     Fokus blieb an derselben Stelle. Screenreader kündigen den neuen Inhalt \
                     nicht an."
                )
            },
            Some(if en {
                "After each client-side navigation: (1) update document.title, \
                 (2) move focus to the <main> element or the new H1 heading, \
                 (3) alternatively populate an aria-live region with the new page name."
                    .to_string()
            } else {
                "Nach jeder clientseitigen Navigation: (1) document.title aktualisieren, \
                 (2) den Fokus auf das <main>-Element oder die neue H1-Überschrift setzen, \
                 (3) alternativ eine aria-live-Region mit dem neuen Seitennamen befüllen."
                    .to_string()
            }),
        ),
        SpaTitleUnchanged => (
            if en {
                format!(
                    "After SPA navigation document.title remains unchanged ({title_before:?}). \
                     Screen readers often primarily announce page transitions via the title."
                )
            } else {
                format!(
                    "Nach der SPA-Navigation bleibt document.title unverändert \
                     ({title_before:?}). Screenreader kündigen Seitenwechsel häufig primär \
                     über den Titel an."
                )
            },
            Some(if en {
                "Update document.title to the new page name after every client-side navigation."
                    .to_string()
            } else {
                "document.title nach jeder clientseitigen Navigation auf den neuen Seitennamen \
                 aktualisieren."
                    .to_string()
            }),
        ),
        SpaFocusNotMoved => (
            if en {
                "After SPA navigation focus is not moved to the new main area. \
                 Keyboard users must manually navigate to the new content."
                    .to_string()
            } else {
                "Nach der SPA-Navigation wird der Fokus nicht in den neuen Hauptbereich \
                 verschoben. Tastaturnutzer müssen manuell zum neuen Inhalt navigieren."
                    .to_string()
            },
            Some(if en {
                "After navigation, move focus to the <main> element or the first \
                 H1 heading of the new content."
                    .to_string()
            } else {
                "Nach der Navigation den Fokus auf das <main>-Element oder die erste \
                 H1-Überschrift des neuen Inhalts setzen."
                    .to_string()
            }),
        ),
        SkipLinkFocusNotMoved => (
            if en {
                "Skip link is present but does not move focus to the target. \
                 Keyboard users cannot bypass navigation."
                    .to_string()
            } else {
                "Der Skip-Link ist vorhanden, verschiebt den Fokus aber nicht zum Ziel. \
                 Tastaturnutzer können die Navigation nicht überspringen."
                    .to_string()
            },
            Some(if en {
                "Ensure the skip link target has tabindex=\"-1\" and receives focus via \
                 an anchor link, or explicitly call target.focus() after navigation."
                    .to_string()
            } else {
                "Sicherstellen, dass das Skip-Link-Ziel tabindex=\"-1\" besitzt und per \
                 Anker-Link fokussiert wird, oder explizit target.focus() nach der \
                 Navigation aufrufen."
                    .to_string()
            }),
        ),
        FormErrorSilentFailure => (
            if en {
                "Form errors are not announced via a live region (role=\"alert\" or \
                 aria-live) and aria-invalid is not set. \
                 Screen reader users receive no feedback when a required field \
                 is left empty."
                    .to_string()
            } else {
                "Formularfehler werden nicht über eine Live-Region (role=\"alert\" oder \
                 aria-live) angekündigt, und aria-invalid wird nicht gesetzt. \
                 Screenreader-Nutzer erhalten kein Feedback, wenn ein Pflichtfeld leer \
                 bleibt."
                    .to_string()
            },
            Some(if en {
                "Output error messages inside a role=\"alert\" element and set \
                 aria-invalid=\"true\" on each invalid field."
                    .to_string()
            } else {
                "Fehlermeldungen in einem role=\"alert\"-Element ausgeben und \
                 aria-invalid=\"true\" auf jedem ungültigen Feld setzen."
                    .to_string()
            }),
        ),
        FormErrorInvalidWithoutLiveRegion => (
            if en {
                "aria-invalid is set after submission, but no live region \
                 (role=\"alert\" or aria-live) announces the error. \
                 Screen reader users will only notice the error state when they \
                 explicitly navigate back to the field."
                    .to_string()
            } else {
                "aria-invalid wird nach dem Absenden gesetzt, aber keine Live-Region \
                 (role=\"alert\" oder aria-live) kündigt den Fehler an. Screenreader-Nutzer \
                 bemerken den Fehlerzustand nur, wenn sie gezielt zum Feld zurücknavigieren."
                    .to_string()
            },
            Some(if en {
                "Add a role=\"alert\" container that outputs the error message \
                 after form submission."
                    .to_string()
            } else {
                "Einen role=\"alert\"-Container ergänzen, der die Fehlermeldung nach dem \
                 Absenden ausgibt."
                    .to_string()
            }),
        ),
        FormErrorUnlinkedFields => (
            if en {
                format!(
                    "{count} {} with aria-invalid=\"true\" are not linked to their error \
                     message via aria-describedby or aria-errormessage. \
                     Screen reader users hear the error state but cannot associate it with \
                     the field.",
                    if count == 1 { "field" } else { "fields" }
                )
            } else {
                format!(
                    "{count} {} mit aria-invalid=\"true\" sind nicht per aria-describedby \
                     oder aria-errormessage mit ihrer Fehlermeldung verknüpft. \
                     Screenreader-Nutzer hören den Fehlerzustand, können ihn aber nicht dem \
                     Feld zuordnen.",
                    if count == 1 { "Feld" } else { "Felder" }
                )
            },
            Some(if en {
                "Add aria-describedby=\"error-message-id\" on each field with \
                 aria-invalid=\"true\"."
                    .to_string()
            } else {
                "aria-describedby=\"error-message-id\" auf jedem Feld mit \
                 aria-invalid=\"true\" ergänzen."
                    .to_string()
            }),
        ),
        FormErrorFocusNotManaged => (
            if en {
                "An error was announced after submission, but keyboard focus stayed on \
                 the document body instead of moving to the first invalid field or an \
                 error summary. Screen reader and keyboard users are not led to the \
                 error and must search for it manually."
                    .to_string()
            } else {
                "Nach dem Absenden wurde ein Fehler angekündigt, der Tastaturfokus blieb \
                 jedoch auf dem Dokument-Body, statt zum ersten ungültigen Feld oder einer \
                 Fehlerzusammenfassung zu wechseln. Screenreader- und Tastaturnutzer werden \
                 nicht zum Fehler geführt und müssen ihn manuell suchen."
                    .to_string()
            },
            Some(if en {
                "Move focus to the first invalid field or to an error summary \
                 (e.g. role=\"alert\") after a failed submission."
                    .to_string()
            } else {
                "Fokus nach einem fehlgeschlagenen Absenden zum ersten ungültigen Feld \
                 oder zu einer Fehlerzusammenfassung (z. B. role=\"alert\") bewegen."
                    .to_string()
            }),
        ),
        FormErrorLiveRegionLateInsertion => (
            if en {
                "The live region (role=\"alert\" or aria-live) announcing the error was not \
                 present on initial page load — it was only inserted into the DOM after form \
                 submission. Screen readers register live regions when the accessibility tree \
                 is first built; some browser/assistive-technology combinations do not reliably \
                 announce a live region that appears only after the page has already loaded."
                    .to_string()
            } else {
                "Die den Fehler ankündigende Live-Region (role=\"alert\" oder aria-live) war \
                 beim initialen Laden der Seite nicht vorhanden — sie wurde erst nach dem \
                 Absenden des Formulars ins DOM eingefügt. Screenreader registrieren \
                 Live-Regions beim initialen Aufbau des Accessibility Tree; manche \
                 Kombinationen aus Browser und Screenreader kündigen eine erst nachträglich \
                 eingefügte Live-Region nicht zuverlässig an."
                    .to_string()
            },
            Some(if en {
                "Render the live-region container empty in the initial markup (e.g. \
                 <div role=\"status\"></div>) and only fill its text content on submission, \
                 instead of inserting the container itself after the interaction."
                    .to_string()
            } else {
                "Den Live-Region-Container bereits im initialen Markup leer rendern (z. B. \
                 <div role=\"status\"></div>) und nur den Textinhalt beim Absenden befüllen, \
                 statt den Container selbst erst nach der Interaktion einzufügen."
                    .to_string()
            }),
        ),
        AddToCartNoStatusAnnouncement => (
            if en {
                "Adding the item to the cart visibly changed the page (e.g. a cart \
                 counter), but no live region (role=\"status\"/\"alert\" or aria-live) \
                 announced it and focus did not move into a cart dialog. Screen reader \
                 users receive no confirmation that the item was added."
                    .to_string()
            } else {
                "Das Hinzufügen zum Warenkorb hat die Seite sichtbar verändert (z. B. \
                 einen Warenkorb-Zähler), aber keine Live-Region (role=\"status\"/\"alert\" \
                 oder aria-live) hat dies angekündigt, und der Fokus ist nicht in einen \
                 Warenkorb-Dialog gewechselt. Screenreader-Nutzer erhalten keine \
                 Bestätigung, dass der Artikel hinzugefügt wurde."
                    .to_string()
            },
            Some(if en {
                "Announce the outcome via a role=\"status\" live region (e.g. \"Item \
                 added to cart\"), or move focus into the cart drawer/dialog when it \
                 opens."
                    .to_string()
            } else {
                "Das Ergebnis über eine role=\"status\"-Live-Region ankündigen (z. B. \
                 „Artikel zum Warenkorb hinzugefügt“), oder den Fokus beim Öffnen in den \
                 Warenkorb-Dialog verschieben."
                    .to_string()
            }),
        ),
        AddToCartNoFeedbackDetected => (
            if en {
                "Clicking the add-to-cart trigger produced no detectable change: no \
                 live-region announcement, no focus change into a dialog, and no \
                 recognizable cart-counter update. Manual review is needed to confirm \
                 whether the action succeeded and how it is communicated."
                    .to_string()
            } else {
                "Der Klick auf den Warenkorb-Button führte zu keiner erkennbaren \
                 Änderung: keine Live-Region-Ankündigung, kein Fokuswechsel in einen \
                 Dialog und keine erkennbare Warenkorb-Zähler-Aktualisierung. Eine \
                 manuelle Prüfung ist nötig, um zu bestätigen, ob die Aktion erfolgreich \
                 war und wie sie kommuniziert wird."
                    .to_string()
            },
            Some(if en {
                "Verify manually that adding to cart succeeds and is announced (e.g. \
                 via a role=\"status\" live region or a focus-managed cart dialog)."
                    .to_string()
            } else {
                "Manuell prüfen, ob das Hinzufügen zum Warenkorb funktioniert und \
                 angekündigt wird (z. B. über eine role=\"status\"-Live-Region oder \
                 einen fokusverwalteten Warenkorb-Dialog)."
                    .to_string()
            }),
        ),
        QuantityStepperKeyboardInoperable => (
            if en {
                "The quantity control could not be operated by keyboard: it either \
                 could not receive focus, or its value did not change after pressing \
                 Arrow Up. Keyboard-only users cannot adjust the quantity."
                    .to_string()
            } else {
                "Das Mengenfeld ließ sich nicht per Tastatur bedienen: Es konnte entweder \
                 nicht fokussiert werden, oder sein Wert änderte sich nach Drücken von \
                 Pfeil-nach-oben nicht. Reine Tastaturnutzer können die Menge nicht \
                 anpassen."
                    .to_string()
            },
            Some(if en {
                "Ensure the quantity control is a native <input type=\"number\"> or a \
                 fully keyboard-operable ARIA spinbutton (focusable, responds to Arrow \
                 Up/Down)."
                    .to_string()
            } else {
                "Sicherstellen, dass das Mengenfeld ein natives <input type=\"number\"> \
                 oder ein vollständig tastaturbedienbares ARIA-Spinbutton ist \
                 (fokussierbar, reagiert auf Pfeil-hoch/-runter)."
                    .to_string()
            }),
        ),
        QuantityStepperValueNotExposed => (
            if en {
                "Arrow Up changed the quantity value, but this custom (non-native) \
                 spinbutton widget did not update aria-valuenow to match. Screen reader \
                 users hear no change even though the value did change."
                    .to_string()
            } else {
                "Pfeil-nach-oben hat den Mengenwert geändert, aber dieses \
                 benutzerdefinierte (nicht native) Spinbutton-Widget hat aria-valuenow \
                 nicht entsprechend aktualisiert. Screenreader-Nutzer hören keine \
                 Änderung, obwohl sich der Wert geändert hat."
                    .to_string()
            },
            Some(if en {
                "Update aria-valuenow (and ideally aria-valuetext) on the spinbutton \
                 element whenever its value changes."
                    .to_string()
            } else {
                "aria-valuenow (und idealerweise aria-valuetext) auf dem \
                 Spinbutton-Element bei jeder Wertänderung aktualisieren."
                    .to_string()
            }),
        ),
        LinkTextGeneric => (
            if en {
                format!(
                    "{count} {} carry generic or non-descriptive text \
                     ({examples}). Without surrounding context they are hard to tell apart for \
                     screen reader users. Heuristic warning on WCAG 2.4.4 — whether the link \
                     purpose is clear from its context needs a human check.",
                    if count == 1 { "link" } else { "links" }
                )
            } else {
                format!(
                    "{count} {} tragen generischen oder wenig aussagekräftigen Text \
                     ({examples}). Ohne den umgebenden Kontext sind sie für \
                     Screenreader-Nutzer schwer unterscheidbar. Heuristische Warnung zu \
                     WCAG 2.4.4 — ob der Linkzweck aus dem Kontext hervorgeht, muss manuell \
                     geprüft werden.",
                    if count == 1 { "Link" } else { "Links" }
                )
            },
            Some(if en {
                "Write link text that is meaningful without the surrounding page context, \
                 e.g. 'Learn more about accessibility' instead of 'Learn more'."
                    .to_string()
            } else {
                "Linktext so formulieren, dass er auch ohne den umgebenden Seitenkontext \
                 verständlich ist, z. B. 'Mehr über Barrierefreiheit erfahren' statt \
                 'Mehr erfahren'."
                    .to_string()
            }),
        ),
        LinkTextDuplicate => (
            if en {
                format!(
                    "{count} {} appear 3 or more times on the page: {examples}. \
                     If they point to different targets, screen reader users cannot \
                     distinguish them.",
                    if count == 1 {
                        "link text"
                    } else {
                        "link texts"
                    }
                )
            } else {
                format!(
                    "{count} {} kommen 3-mal oder häufiger auf der Seite vor: {examples}. \
                     Verweisen sie auf unterschiedliche Ziele, können Screenreader-Nutzer sie \
                     nicht unterscheiden.",
                    if count == 1 { "Linktext" } else { "Linktexte" }
                )
            },
            Some(if en {
                "Replace repeated link texts with unique wording or supplement the visible \
                 text with aria-label / aria-labelledby."
                    .to_string()
            } else {
                "Wiederholte Linktexte durch eindeutige Formulierungen ersetzen oder den \
                 sichtbaren Text durch aria-label / aria-labelledby ergänzen."
                    .to_string()
            }),
        ),
        HeadingMissingH1 => (
            if en {
                "The page has no H1 heading. Screen reader users cannot \
                 identify the main structure of the page without an H1."
                    .to_string()
            } else {
                "Die Seite hat keine H1-Überschrift. Ohne H1 können Screenreader-Nutzer die \
                 Hauptstruktur der Seite nicht erkennen."
                    .to_string()
            },
            Some(if en {
                "Use exactly one H1 heading per page that describes the main content.".to_string()
            } else {
                "Genau eine H1-Überschrift pro Seite verwenden, die den Hauptinhalt \
                 beschreibt."
                    .to_string()
            }),
        ),
        HeadingMultipleH1 => (
            if en {
                format!(
                    "{count} H1 headings found. Multiple H1 elements make it harder for \
                     screen reader users to orient themselves."
                )
            } else {
                format!(
                    "{count} H1-Überschriften gefunden. Mehrere H1-Elemente erschweren \
                     Screenreader-Nutzern die Orientierung."
                )
            },
            Some(if en {
                "Use only one H1 heading per page. Mark further top-level headings as H2."
                    .to_string()
            } else {
                "Nur eine H1-Überschrift pro Seite verwenden. Weitere Top-Level-Überschriften \
                 als H2 auszeichnen."
                    .to_string()
            }),
        ),
        HeadingLevelSkip => (
            if en {
                format!(
                    "Heading hierarchy skips levels ({examples}). Screen reader users may not \
                     be able to reliably parse the page structure."
                )
            } else {
                format!(
                    "Die Heading-Hierarchie überspringt Ebenen ({examples}). \
                     Screenreader-Nutzer können die Seitenstruktur unter Umständen nicht \
                     zuverlässig erfassen."
                )
            },
            Some(if en {
                "Never skip heading levels. After H1 comes H2, after H2 comes H3, and so on."
                    .to_string()
            } else {
                "Heading-Ebenen nie überspringen. Nach H1 folgt H2, nach H2 folgt H3 und so \
                 weiter."
                    .to_string()
            }),
        ),
        LandmarkMissingMain => (
            if en {
                "No <main> landmark found. Screen reader users cannot \
                 jump directly to the main content."
                    .to_string()
            } else {
                "Kein <main>-Landmark gefunden. Screenreader-Nutzer können nicht direkt zum \
                 Hauptinhalt springen."
                    .to_string()
            },
            Some(if en {
                "Wrap the main content in a <main> element or set role=\"main\" \
                 on the appropriate container."
                    .to_string()
            } else {
                "Den Hauptinhalt in ein <main>-Element einbetten oder role=\"main\" auf dem \
                 passenden Container setzen."
                    .to_string()
            }),
        ),
        LandmarkNavWithoutLabels => (
            if en {
                format!(
                    "{count} navigation landmarks without distinct labels. \
                     Screen reader users cannot tell which navigation covers which area."
                )
            } else {
                format!(
                    "{count} Navigations-Landmarks ohne eindeutige Beschriftung. \
                     Screenreader-Nutzer können nicht unterscheiden, welche Navigation \
                     welchen Bereich abdeckt."
                )
            },
            Some(if en {
                "Label each <nav> region with an aria-label, \
                 e.g. aria-label=\"Main navigation\" and aria-label=\"Footer navigation\"."
                    .to_string()
            } else {
                "Jeden <nav>-Bereich mit aria-label beschriften, z. B. \
                 aria-label=\"Hauptnavigation\" und aria-label=\"Footer-Navigation\"."
                    .to_string()
            }),
        ),
        LandmarkDuplicateUnique => (
            if en {
                format!(
                    "Landmark role \"{role}\" appears {count}× on the page. \
                     This role should only occur once per page."
                )
            } else {
                format!(
                    "Die Landmark-Rolle \"{role}\" kommt {count}-mal auf der Seite vor. \
                     Diese Rolle sollte nur einmal pro Seite vorkommen."
                )
            },
            Some(if en {
                format!(
                    "Use only one element with role=\"{role}\" (or the corresponding \
                     HTML element) per page."
                )
            } else {
                format!(
                    "Nur ein Element mit role=\"{role}\" (oder dem entsprechenden \
                     HTML-Element) pro Seite verwenden."
                )
            }),
        ),
        MediaControlsMissingName => (
            if en {
                format!(
                    "Native video player ({selector}) is reachable via keyboard but has no \
                     accessible name (no aria-label, aria-labelledby, or title). Screen reader \
                     users cannot identify which video the control refers to."
                )
            } else {
                format!(
                    "Der native Video-Player ({selector}) ist per Tastatur erreichbar, hat aber \
                     keinen zugänglichen Namen (kein aria-label, aria-labelledby oder title). \
                     Screenreader-Nutzer können nicht erkennen, um welches Video es sich handelt."
                )
            },
            Some(if en {
                "Add an aria-label, aria-labelledby, or title attribute to the video element \
                 describing its content."
                    .to_string()
            } else {
                "Ein aria-label-, aria-labelledby- oder title-Attribut mit einer Beschreibung \
                 des Videoinhalts zum video-Element hinzufügen."
                    .to_string()
            }),
        ),
        MediaControlsNotReachable => {
            let suffix = if truncated { " (…)" } else { "" };
            let (de_adj, de_noun) = if count == 1 {
                ("natives", "Video-Element")
            } else {
                ("native", "Video-Elemente")
            };
            (
                if en {
                    format!(
                        "{count} native video {} with visible controls could not be reached \
                         via keyboard Tab navigation: {examples}{suffix}. Keyboard users may be \
                         unable to operate playback.",
                        if count == 1 { "element" } else { "elements" }
                    )
                } else {
                    format!(
                        "{count} {de_adj} {de_noun} mit sichtbaren Steuerelementen waren beim \
                         Tastatur-Tab-Durchlauf nicht erreichbar: {examples}{suffix}. \
                         Tastaturnutzer können die Wiedergabe möglicherweise nicht bedienen."
                    )
                },
                Some(if en {
                    "Verify no earlier element traps focus and that the video element itself \
                     is not removed from the tab sequence (tabindex=\"-1\")."
                        .to_string()
                } else {
                    "Prüfen, ob ein vorheriges Element den Fokus einfängt und ob das \
                     video-Element selbst nicht per tabindex=\"-1\" aus der Tab-Reihenfolge \
                     entfernt wurde."
                        .to_string()
                }),
            )
        }
    };

    (message, fix)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guard against German leaking into the canonical `InteractiveFinding`
    /// text baked with `en = true` (#406): no English message/fix_suggestion
    /// produced by `interactive_finding_text` may contain German umlauts/ß.
    /// #406 guard for the plan-53 uncertainty note: English is umlaut-free and
    /// the German variant differs.
    #[test]
    fn finding_uncertainty_text_en_has_no_german_umlauts() {
        let u = FindingUncertainty::LateReactionPossible { waited_ms: 210 };
        let en = finding_uncertainty_text(&u, true);
        let de = finding_uncertainty_text(&u, false);
        assert!(!en.chars().any(|c| "äöüÄÖÜß".contains(c)), "{en}");
        assert_ne!(en, de);
        assert!(en.contains("210 ms"));
    }

    /// Also checks that the German variant actually differs.
    #[test]
    fn interactive_finding_text_en_has_no_german_umlauts() {
        use InteractiveFindingKind::*;
        let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
        let sample_values = InteractiveFindingValues {
            selector: Some("a#one".to_string()),
            count: Some(2),
            examples: Some("a, b".to_string()),
            truncated: Some(true),
            title_before: Some("Home".to_string()),
            role: Some("main".to_string()),
        };
        let all_kinds = [
            HiddenFocusableAriaHidden,
            HiddenFocusableInert,
            HiddenFocusableStyle,
            FocusIndicatorNotDetected,
            TabOrderBackwardJumps,
            FocusTrapNotEntered,
            FocusTrapBackgroundNotHidden,
            FocusTrapEscaped,
            FocusTrapEscapeNotClosing,
            FocusRestorationLostToBody,
            ModalNotOpened,
            MenuNotOpened,
            MenuFocusNotMoved,
            MenuEscapeNotClosing,
            TabsSelectionNotMoved,
            TabsFocusNotOnTab,
            DisclosureNotOpened,
            DisclosureNotClosed,
            DisclosureStateWithoutContent,
            DisclosureContentWithoutState,
            SpaNoAnnouncementSignal,
            SpaTitleUnchanged,
            SpaFocusNotMoved,
            SkipLinkFocusNotMoved,
            FormErrorSilentFailure,
            FormErrorInvalidWithoutLiveRegion,
            FormErrorUnlinkedFields,
            FormErrorFocusNotManaged,
            FormErrorLiveRegionLateInsertion,
            AddToCartNoStatusAnnouncement,
            AddToCartNoFeedbackDetected,
            QuantityStepperKeyboardInoperable,
            QuantityStepperValueNotExposed,
            LinkTextGeneric,
            LinkTextDuplicate,
            HeadingMissingH1,
            HeadingMultipleH1,
            HeadingLevelSkip,
            LandmarkMissingMain,
            LandmarkNavWithoutLabels,
            LandmarkDuplicateUnique,
            MediaControlsMissingName,
            MediaControlsNotReachable,
        ];
        for kind in all_kinds {
            let (message, fix) = interactive_finding_text(kind, &sample_values, true);
            assert!(
                !has_umlaut(&message),
                "EN message for {kind:?} contains German umlaut: {message}"
            );
            if let Some(fix) = &fix {
                assert!(
                    !has_umlaut(fix),
                    "EN fix_suggestion for {kind:?} contains German umlaut: {fix}"
                );
            }

            let (de_message, _) = interactive_finding_text(kind, &sample_values, false);
            assert_ne!(message, de_message, "DE/EN message identical for {kind:?}");
        }
    }
}
