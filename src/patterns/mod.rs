//! Pattern Detection — structural recognition of UI patterns in the AXTree.
//!
//! Detects common patterns (MainNavigation, DisclosureMenu, ModalDialog,
//! TabList, SkipLink, Accordion) and produces:
//! - `recognized`: positive signals when the pattern is well-formed
//! - `journey_candidates`: triggers for the interactive journeys
//!
//! WCAG-Befunde über die Struktur der Muster laufen seit #692 und #694 im
//! geteilten Bestand (`dialog/*`, `aria/tab-selected-missing`,
//! `patterns/accordion-controls-missing`, siehe `wcag::shared`); `violations`
//! bleibt für die Pipeline stehen, die Erkennung füllt es nicht mehr.
//!
//! Patterns are inferred from AXTree role/name/property structure, not from
//! runtime interaction. Anything requiring a behavioral test (e.g. actual
//! keyboard focus traversal) is out of scope and remains a manual-review item.

mod accordion;
mod add_to_cart;
mod disclosure_menu;
pub(crate) mod easy_language;
mod form;
pub mod help_mechanisms;
mod main_navigation;
mod modal_dialog;
mod quantity_stepper;
pub(crate) mod skip_link;
mod tab_list;

pub(crate) use disclosure_menu::NATIVE_DISCLOSURE_ROLES;

use serde::{Deserialize, Serialize};

use crate::accessibility::AXTree;
use crate::wcag::types::Violation;

/// Purchase-final / booking button names — never a synthetic-click trigger
/// for any journey candidate, even as a fallback. A live site with session/
/// autofill (or cart) state could otherwise complete a real transaction.
/// Shared between `form.rs` (form-error journey) and `add_to_cart.rs` (an
/// "Add to Cart" button must never double as a "Buy Now"/one-click-purchase
/// trigger).
pub(crate) const PURCHASE_FINAL_HINTS: &[&str] = &[
    "zahlungspflichtig bestellen",
    "jetzt kaufen",
    "kaufen",
    "bestellen",
    "buy now",
    "place order",
    "complete purchase",
    "complete order",
    "pay",
    "checkout",
    "buchen",
    "book now",
    "reservieren",
    "reserve",
];

/// Result of running pattern detection against an AXTree.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// Patterns that were recognized in the page (positive signals).
    pub recognized: Vec<RecognizedPattern>,
    /// Violations emitted when a pattern was found but broken. Seit #694
    /// leer: Die Befunde kommen aus dem geteilten Bestand.
    pub violations: Vec<Violation>,
    /// Candidates handed to the Accessibility-Journey-Layer for interactive
    /// testing. Empty in Phase 1 — detectors stub `Vec::new()`. Phase 2
    /// populates real triggers (modal openers, disclosure buttons, tablists,
    /// menu burgers, accordion headers).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub journey_candidates: Vec<JourneyCandidate>,
    /// Help mechanisms outside `main`, in document order — input for the
    /// cross-page WCAG 3.2.6 comparison in `audit::batch_consistency`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub help_mechanisms: Vec<help_mechanisms::HelpMechanism>,
}

/// A pattern candidate the journey layer can drive interactively.
///
/// Detectors emit these when they find a *probable* trigger and the
/// controlled region. Confidence is used by the journey layer to decide
/// whether to attempt the interaction at all — threshold ≈ 0.7 by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyCandidate {
    pub pattern_kind: PatternKind,
    /// CDP backend-node id of the trigger element.
    pub trigger_backend_id: Option<i64>,
    /// CDP backend-node id of the controlled region (via `aria-controls`).
    pub controlled_backend_id: Option<i64>,
    /// 0.0..1.0 — detector's confidence that this is the real pattern.
    pub confidence: f32,
    pub required_journey: JourneyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    Modal,
    Disclosure,
    Tabs,
    Menu,
    Accordion,
    Form,
    SkipLink,
    AddToCart,
    QuantityStepper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JourneyKind {
    ModalOpen,
    DisclosureToggle,
    TabsNavigate,
    MenuOpen,
    AccordionToggle,
    FormErrorSubmit,
    SkipLinkActivate,
    /// Commerce-only: gated at journey-run time on a detected shop +
    /// `CommercePageKind::ProductDetail` (this module has no commerce
    /// context — see `a11y_journey::run`).
    AddToCart,
    /// Commerce-only: gated at journey-run time on a detected shop +
    /// `CommercePageKind::ProductDetail`.
    QuantityStepper,
}

/// A pattern that was recognized in the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedPattern {
    /// Pattern name, e.g. "MainNavigation".
    pub pattern: String,
    /// Human-readable summary of what was recognized.
    pub message: String,
    /// How confident the detection is.
    pub confidence: PatternConfidence,
    /// What was recognized, with its raw values — the only source for
    /// `message` and for the localized PDF text (#406, Plan 77). Not in the
    /// JSON: `message` already carries the canonical English text there.
    #[serde(skip)]
    pub detail: Option<PatternMessage>,
}

/// What a detector recognized. `pattern_message_text` turns it into prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternMessage {
    NavigationUnnamed {
        landmarks: usize,
        unnamed: usize,
    },
    NavigationWithoutLinks {
        landmarks: usize,
    },
    Navigation {
        landmarks: usize,
    },
    DisclosureNative {
        native: usize,
    },
    Disclosure {
        triggers: usize,
        with_controls: usize,
        native: usize,
    },
    ModalDialog {
        dialogs: usize,
        well_formed: usize,
    },
    TabList {
        tablists: usize,
        tabs: usize,
        selected: usize,
        controls: usize,
    },
    SkipLink {
        name: String,
        first: bool,
    },
    Accordion {
        triggers: usize,
        with_controls: usize,
        non_button: usize,
    },
    Form {
        fields: usize,
        trigger: Option<String>,
    },
    AddToCart {
        trigger: Option<String>,
    },
    QuantityStepper {
        name: Option<String>,
    },
    EasyLanguage,
}

/// Text for a recognized pattern — canonical English for the JSON, the run
/// language for the PDF.
pub fn pattern_message_text(message: &PatternMessage, en: bool) -> String {
    fn n(count: usize, one: &str, many: &str) -> String {
        format!("{count} {}", if count == 1 { one } else { many })
    }
    match message {
        PatternMessage::NavigationUnnamed { landmarks, unnamed } => {
            if en {
                format!(
                "{} found, but {unnamed} have no accessible name. Screen reader users cannot distinguish them.",
                n(*landmarks, "navigation landmark", "navigation landmarks")
            )
            } else {
                format!(
                "{} gefunden, davon {unnamed} ohne zugänglichen Namen. Screenreader-Nutzende können sie nicht unterscheiden.",
                n(*landmarks, "Navigationsbereich", "Navigationsbereiche")
            )
            }
        }
        PatternMessage::NavigationWithoutLinks { landmarks } => {
            if en {
                format!(
                    "{} found, but none contain link children. Pattern is structurally incomplete.",
                    n(*landmarks, "navigation landmark", "navigation landmarks")
                )
            } else {
                format!(
                    "{} gefunden, aber keiner enthält Links. Die Struktur ist unvollständig.",
                    n(*landmarks, "Navigationsbereich", "Navigationsbereiche")
                )
            }
        }
        PatternMessage::Navigation { landmarks } => {
            if en {
                format!(
                "Semantic main navigation recognized ({} with native links). Good foundation for keyboard and screen reader use.",
                n(*landmarks, "landmark", "landmarks")
            )
            } else {
                format!(
                "Semantische Hauptnavigation erkannt ({} mit nativen Links). Gute Grundlage für Tastatur- und Screenreader-Nutzung.",
                n(*landmarks, "Navigationsbereich", "Navigationsbereiche")
            )
            }
        }
        PatternMessage::DisclosureNative { native } => {
            if en {
                format!(
                    "{} (<details>/<summary>) — semantics come from the element, no ARIA needed.",
                    n(
                        *native,
                        "native disclosure trigger",
                        "native disclosure triggers"
                    )
                )
            } else {
                format!(
                "{} (<details>/<summary>) – die Semantik kommt aus dem Element, ARIA ist nicht nötig.",
                n(*native, "natives Aufklappelement", "native Aufklappelemente")
            )
            }
        }
        PatternMessage::Disclosure {
            triggers,
            with_controls,
            native,
        } => {
            let mut text = match (en, with_controls == triggers) {
                (true, true) => format!(
                    "{} with aria-expanded and aria-controls — well-formed pattern.",
                    n(*triggers, "disclosure trigger", "disclosure triggers")
                ),
                (true, false) => format!(
                    "{} with aria-expanded ({with_controls} with aria-controls). Controls relationship strengthens screen-reader announcements.",
                    n(*triggers, "disclosure trigger", "disclosure triggers")
                ),
                (false, true) => format!(
                    "{} mit aria-expanded und aria-controls – vollständig ausgezeichnet.",
                    n(*triggers, "Aufklapp-Auslöser", "Aufklapp-Auslöser")
                ),
                (false, false) => format!(
                    "{} mit aria-expanded ({with_controls} mit aria-controls). Die Verknüpfung über aria-controls verbessert die Screenreader-Ansage.",
                    n(*triggers, "Aufklapp-Auslöser", "Aufklapp-Auslöser")
                ),
            };
            if *native > 0 {
                text.push_str(&if en {
                    format!(
                        " Plus {} native <details>/<summary> {}.",
                        native,
                        if *native == 1 { "trigger" } else { "triggers" }
                    )
                } else {
                    format!(
                        " Dazu {} mit <details>/<summary>.",
                        n(*native, "natives Element", "native Elemente")
                    )
                });
            }
            text
        }
        PatternMessage::ModalDialog {
            dialogs,
            well_formed,
        } => {
            if en {
                format!(
                "{} detected, {well_formed} structurally well-formed (accessible name, aria-modal, focusable descendants).",
                n(*dialogs, "dialog", "dialogs")
            )
            } else {
                format!(
                "{} erkannt, davon {well_formed} strukturell vollständig (zugänglicher Name, aria-modal, fokussierbarer Inhalt).",
                n(*dialogs, "Dialog", "Dialoge")
            )
            }
        }
        PatternMessage::TabList {
            tablists,
            tabs,
            selected,
            controls,
        } => {
            if en {
                format!(
                    "{} with {}; {selected} have aria-selected, {controls} declare aria-controls.",
                    n(*tablists, "tablist", "tablists"),
                    n(*tabs, "tab", "tabs")
                )
            } else {
                format!(
                    "{} mit {}; {selected} mit aria-selected, {controls} mit aria-controls.",
                    n(*tablists, "Tab-Liste", "Tab-Listen"),
                    n(*tabs, "Tab", "Tabs")
                )
            }
        }
        PatternMessage::SkipLink { name, first: true } => {
            if en {
                format!("Skip link recognized and correctly positioned as first focusable element (\"{name}\").")
            } else {
                format!("Sprunglink erkannt und korrekt als erstes fokussierbares Element platziert („{name}“).")
            }
        }
        PatternMessage::SkipLink { name, first: false } => {
            if en {
                format!("Skip link detected (\"{name}\") but not the first focusable element — keyboard users may have to tab past other content first.")
            } else {
                format!("Sprunglink erkannt („{name}“), aber nicht als erstes fokussierbares Element – Tastaturnutzende müssen eventuell erst an anderen Elementen vorbei.")
            }
        }
        PatternMessage::Accordion {
            triggers,
            with_controls,
            non_button,
        } => {
            if en {
                format!(
                    "{}; {with_controls} with aria-controls; {non_button} non-button triggers.",
                    n(*triggers, "accordion trigger", "accordion triggers")
                )
            } else {
                format!(
                    "{}; {with_controls} mit aria-controls; {non_button} ohne Button-Rolle.",
                    n(*triggers, "Accordion-Auslöser", "Accordion-Auslöser")
                )
            }
        }
        PatternMessage::Form { fields, trigger } => {
            if en {
                format!(
                    "{} detected; submit trigger: {}",
                    n(*fields, "required field", "required fields"),
                    trigger.as_deref().unwrap_or("(unnamed button)")
                )
            } else {
                format!(
                    "{} erkannt; Absenden über: {}",
                    n(*fields, "Pflichtfeld", "Pflichtfelder"),
                    trigger.as_deref().unwrap_or("(Button ohne Namen)")
                )
            }
        }
        PatternMessage::AddToCart { trigger } => {
            if en {
                format!(
                    "Add-to-cart trigger detected: {}",
                    trigger.as_deref().unwrap_or("(unnamed button)")
                )
            } else {
                format!(
                    "Warenkorb-Button erkannt: {}",
                    trigger.as_deref().unwrap_or("(Button ohne Namen)")
                )
            }
        }
        PatternMessage::QuantityStepper { name } => {
            if en {
                format!(
                    "Spinbutton quantity control detected: {}",
                    name.as_deref().unwrap_or("(unnamed)")
                )
            } else {
                format!(
                    "Mengenauswahl (Spinbutton) erkannt: {}",
                    name.as_deref().unwrap_or("(ohne Namen)")
                )
            }
        }
        PatternMessage::EasyLanguage => easy_language::message_text(en).to_string(),
    }
}

/// Confidence level of a pattern detection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternConfidence {
    /// All structural criteria matched.
    Strong,
    /// Some criteria matched; others uncertain or missing.
    Partial,
}

impl PatternAnalysis {
    pub fn add_recognized(
        &mut self,
        pattern: &str,
        detail: PatternMessage,
        confidence: PatternConfidence,
    ) {
        self.recognized.push(RecognizedPattern {
            pattern: pattern.to_string(),
            message: pattern_message_text(&detail, true),
            confidence,
            detail: Some(detail),
        });
    }

    /// Whether a pattern with the given name was recognized (any confidence).
    pub fn has_recognized(&self, pattern: &str) -> bool {
        self.recognized.iter().any(|r| r.pattern == pattern)
    }
}

/// Run all pattern detectors against an AXTree.
pub fn analyze(tree: &AXTree) -> PatternAnalysis {
    let mut result = PatternAnalysis::default();
    main_navigation::detect(tree, &mut result);
    disclosure_menu::detect(tree, &mut result);
    modal_dialog::detect(tree, &mut result);
    tab_list::detect(tree, &mut result);
    skip_link::detect(tree, &mut result);
    accordion::detect(tree, &mut result);
    form::detect(tree, &mut result);
    add_to_cart::detect(tree, &mut result);
    quantity_stepper::detect(tree, &mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_messages() -> Vec<PatternMessage> {
        vec![
            PatternMessage::NavigationUnnamed {
                landmarks: 2,
                unnamed: 1,
            },
            PatternMessage::NavigationWithoutLinks { landmarks: 1 },
            PatternMessage::Navigation { landmarks: 1 },
            PatternMessage::DisclosureNative { native: 3 },
            PatternMessage::Disclosure {
                triggers: 2,
                with_controls: 1,
                native: 1,
            },
            PatternMessage::ModalDialog {
                dialogs: 1,
                well_formed: 1,
            },
            PatternMessage::TabList {
                tablists: 1,
                tabs: 3,
                selected: 3,
                controls: 2,
            },
            PatternMessage::SkipLink {
                name: "Skip".into(),
                first: true,
            },
            PatternMessage::SkipLink {
                name: "Skip".into(),
                first: false,
            },
            PatternMessage::Accordion {
                triggers: 4,
                with_controls: 4,
                non_button: 0,
            },
            PatternMessage::Form {
                fields: 2,
                trigger: None,
            },
            PatternMessage::AddToCart { trigger: None },
            PatternMessage::QuantityStepper { name: None },
            PatternMessage::EasyLanguage,
        ]
    }

    /// #406: EN-Text ohne deutsche Zeichen, DE-Text tatsächlich übersetzt.
    #[test]
    fn pattern_messages_are_localized() {
        for message in all_messages() {
            let en = pattern_message_text(&message, true);
            let de = pattern_message_text(&message, false);
            assert!(
                !en.chars().any(|c| "äöüÄÖÜß„“".contains(c)),
                "{message:?}: {en}"
            );
            assert_ne!(en, de, "{message:?} is not translated");
        }
    }

    #[test]
    fn json_message_is_canonical_english() {
        let mut out = PatternAnalysis::default();
        out.add_recognized(
            "MainNavigation",
            PatternMessage::Navigation { landmarks: 1 },
            PatternConfidence::Strong,
        );
        assert_eq!(
            out.recognized[0].message,
            "Semantic main navigation recognized (1 landmark with native links). Good foundation for keyboard and screen reader use."
        );
    }
}
