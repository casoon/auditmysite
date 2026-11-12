//! Accessibility-journey layer: journey/trace types and the interactive
//! (journey) finding model with its #406 en/de text source.
//!
//! Moved out of `audit::normalized`; every item stays reachable at its old
//! `auditmysite::audit::normalized::…` path via `pub use`.

use fluent_bundle::FluentValue;
use serde::{Deserialize, Serialize};

use crate::taxonomy::Severity;

#[cfg(test)]
mod text_export;
#[cfg(test)]
mod text_guard;

// ---------------------------------------------------------------------------
// Accessibility-Journey-Layer (Phase 1: Foundation only — types live here so
// `NormalizedReport` is schema-stable for all future phases.)
// ---------------------------------------------------------------------------

/// Bundle of accessibility-journey results for one page.
/// Populated only when `--interactive != off`; otherwise the report's
/// `accessibility_journey` field stays `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub struct JourneyTrace {
    /// Journey identifier: "tab_walk", "skip_link", "modal_contact", ...
    pub journey: String,
    /// Ordered steps that compose the journey.
    pub steps: Vec<JourneyStep>,
}

/// A single step in a journey. Designed to read naturally as JSON so a
/// developer can reproduce the journey by replaying the actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum FindingUncertainty {
    /// The page went quiet `waited_ms` after the action, before the settle
    /// budget ran out. A reaction arriving later was not observed — measured
    /// on fixtures, a reaction later than ~200 ms flips the verdict.
    LateReactionPossible { waited_ms: u64 },
}

/// The wording is the `interactive-uncertainty-*` message of
/// `locales/{en,de}/interactive.ftl`.
pub fn finding_uncertainty_text(uncertainty: &FindingUncertainty, en: bool) -> String {
    match uncertainty {
        FindingUncertainty::LateReactionPossible { waited_ms } => crate::i18n::text_args(
            "interactive-uncertainty-late-reaction",
            &[("waited_ms", FluentValue::from(waited_ms.to_string()))],
            en,
        ),
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
#[non_exhaustive]
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
#[non_exhaustive]
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
/// English; the PDF layer re-derives in the run language (#406). The wording
/// is the `interactive-*` message of `locales/{en,de}/interactive.ftl` and
/// its `-fix` message; this function picks them and formats the arguments
/// (missing values as empty or 0, the page title quoted, " (…)" after a
/// truncated example list).
pub fn interactive_finding_text(
    kind: InteractiveFindingKind,
    values: &InteractiveFindingValues,
    en: bool,
) -> (String, Option<String>) {
    use InteractiveFindingKind::*;
    let text = |value: &Option<String>| FluentValue::from(value.clone().unwrap_or_default());
    let selector = || ("selector", text(&values.selector));
    let count = || {
        (
            "count",
            FluentValue::from(i64::from(values.count.unwrap_or(0))),
        )
    };
    let examples = || ("examples", text(&values.examples));
    let suffix = || {
        let truncated = values.truncated.unwrap_or(false);
        (
            "suffix",
            FluentValue::from(if truncated { " (…)" } else { "" }),
        )
    };
    let title = || {
        let title_before = values.title_before.as_deref().unwrap_or("");
        ("title", FluentValue::from(format!("{title_before:?}")))
    };
    let role = || ("role", text(&values.role));

    let (message, fix, args) = match kind {
        HiddenFocusableAriaHidden => (
            "interactive-hidden-focusable-aria-hidden",
            "interactive-hidden-focusable-aria-hidden-fix",
            vec![selector()],
        ),
        HiddenFocusableInert => (
            "interactive-hidden-focusable-inert",
            "interactive-hidden-focusable-inert-fix",
            vec![selector()],
        ),
        HiddenFocusableStyle => (
            "interactive-hidden-focusable-style",
            "interactive-hidden-focusable-style-fix",
            vec![selector()],
        ),
        FocusIndicatorNotDetected => (
            "interactive-focus-indicator-not-detected",
            "interactive-focus-indicator-not-detected-fix",
            vec![selector()],
        ),
        TabOrderBackwardJumps => (
            "interactive-tab-order-backward-jumps",
            "interactive-tab-order-backward-jumps-fix",
            vec![count(), examples(), suffix()],
        ),
        FocusTrapNotEntered => (
            "interactive-focus-trap-not-entered",
            "interactive-focus-trap-not-entered-fix",
            vec![],
        ),
        FocusTrapBackgroundNotHidden => (
            "interactive-focus-trap-background-not-hidden",
            "interactive-focus-trap-background-not-hidden-fix",
            vec![],
        ),
        FocusTrapEscaped => (
            "interactive-focus-trap-escaped",
            "interactive-focus-trap-escaped-fix",
            vec![],
        ),
        FocusTrapEscapeNotClosing => (
            "interactive-focus-trap-escape-not-closing",
            "interactive-focus-trap-escape-not-closing-fix",
            vec![],
        ),
        FocusRestorationLostToBody => (
            "interactive-focus-restoration-lost-to-body",
            "interactive-focus-restoration-lost-to-body-fix",
            vec![],
        ),
        ModalNotOpened => (
            "interactive-modal-not-opened",
            "interactive-modal-not-opened-fix",
            vec![],
        ),
        MenuNotOpened => (
            "interactive-menu-not-opened",
            "interactive-menu-not-opened-fix",
            vec![],
        ),
        MenuFocusNotMoved => (
            "interactive-menu-focus-not-moved",
            "interactive-menu-focus-not-moved-fix",
            vec![],
        ),
        MenuEscapeNotClosing => (
            "interactive-menu-escape-not-closing",
            "interactive-menu-escape-not-closing-fix",
            vec![],
        ),
        TabsSelectionNotMoved => (
            "interactive-tabs-selection-not-moved",
            "interactive-tabs-selection-not-moved-fix",
            vec![],
        ),
        TabsFocusNotOnTab => (
            "interactive-tabs-focus-not-on-tab",
            "interactive-tabs-focus-not-on-tab-fix",
            vec![],
        ),
        DisclosureNotOpened => (
            "interactive-disclosure-not-opened",
            "interactive-disclosure-not-opened-fix",
            vec![],
        ),
        DisclosureNotClosed => (
            "interactive-disclosure-not-closed",
            "interactive-disclosure-not-closed-fix",
            vec![],
        ),
        DisclosureStateWithoutContent => (
            "interactive-disclosure-state-without-content",
            "interactive-disclosure-state-without-content-fix",
            vec![],
        ),
        DisclosureContentWithoutState => (
            "interactive-disclosure-content-without-state",
            "interactive-disclosure-content-without-state-fix",
            vec![],
        ),
        SpaNoAnnouncementSignal => (
            "interactive-spa-no-announcement-signal",
            "interactive-spa-no-announcement-signal-fix",
            vec![title()],
        ),
        SpaTitleUnchanged => (
            "interactive-spa-title-unchanged",
            "interactive-spa-title-unchanged-fix",
            vec![title()],
        ),
        SpaFocusNotMoved => (
            "interactive-spa-focus-not-moved",
            "interactive-spa-focus-not-moved-fix",
            vec![],
        ),
        SkipLinkFocusNotMoved => (
            "interactive-skip-link-focus-not-moved",
            "interactive-skip-link-focus-not-moved-fix",
            vec![],
        ),
        FormErrorSilentFailure => (
            "interactive-form-error-silent-failure",
            "interactive-form-error-silent-failure-fix",
            vec![],
        ),
        FormErrorInvalidWithoutLiveRegion => (
            "interactive-form-error-invalid-without-live-region",
            "interactive-form-error-invalid-without-live-region-fix",
            vec![],
        ),
        FormErrorUnlinkedFields => (
            "interactive-form-error-unlinked-fields",
            "interactive-form-error-unlinked-fields-fix",
            vec![count()],
        ),
        FormErrorFocusNotManaged => (
            "interactive-form-error-focus-not-managed",
            "interactive-form-error-focus-not-managed-fix",
            vec![],
        ),
        FormErrorLiveRegionLateInsertion => (
            "interactive-form-error-live-region-late-insertion",
            "interactive-form-error-live-region-late-insertion-fix",
            vec![],
        ),
        AddToCartNoStatusAnnouncement => (
            "interactive-add-to-cart-no-status-announcement",
            "interactive-add-to-cart-no-status-announcement-fix",
            vec![],
        ),
        AddToCartNoFeedbackDetected => (
            "interactive-add-to-cart-no-feedback-detected",
            "interactive-add-to-cart-no-feedback-detected-fix",
            vec![],
        ),
        QuantityStepperKeyboardInoperable => (
            "interactive-quantity-stepper-keyboard-inoperable",
            "interactive-quantity-stepper-keyboard-inoperable-fix",
            vec![],
        ),
        QuantityStepperValueNotExposed => (
            "interactive-quantity-stepper-value-not-exposed",
            "interactive-quantity-stepper-value-not-exposed-fix",
            vec![],
        ),
        LinkTextGeneric => (
            "interactive-link-text-generic",
            "interactive-link-text-generic-fix",
            vec![count(), examples()],
        ),
        LinkTextDuplicate => (
            "interactive-link-text-duplicate",
            "interactive-link-text-duplicate-fix",
            vec![count(), examples()],
        ),
        HeadingMissingH1 => (
            "interactive-heading-missing-h1",
            "interactive-heading-missing-h1-fix",
            vec![],
        ),
        HeadingMultipleH1 => (
            "interactive-heading-multiple-h1",
            "interactive-heading-multiple-h1-fix",
            vec![count()],
        ),
        HeadingLevelSkip => (
            "interactive-heading-level-skip",
            "interactive-heading-level-skip-fix",
            vec![examples()],
        ),
        LandmarkMissingMain => (
            "interactive-landmark-missing-main",
            "interactive-landmark-missing-main-fix",
            vec![],
        ),
        LandmarkNavWithoutLabels => (
            "interactive-landmark-nav-without-labels",
            "interactive-landmark-nav-without-labels-fix",
            vec![count()],
        ),
        LandmarkDuplicateUnique => (
            "interactive-landmark-duplicate-unique",
            "interactive-landmark-duplicate-unique-fix",
            vec![count(), role()],
        ),
        MediaControlsMissingName => (
            "interactive-media-controls-missing-name",
            "interactive-media-controls-missing-name-fix",
            vec![selector()],
        ),
        MediaControlsNotReachable => (
            "interactive-media-controls-not-reachable",
            "interactive-media-controls-not-reachable-fix",
            vec![count(), examples(), suffix()],
        ),
    };

    (
        crate::i18n::text_args(message, &args, en),
        Some(crate::i18n::text_args(fix, &args, en)),
    )
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
