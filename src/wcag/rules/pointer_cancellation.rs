//! WCAG 2.5.2 Pointer Cancellation (Level A)
//!
//! For functionality that can be operated using a single pointer, at least one
//! of the following is true: no down-event, abort or undo, up reversal, or essential.
//!
//! Only the page-wide `UNTESTED` note is left (#695, B5). The static half —
//! inline `onmousedown`/`ontouchstart` on controls — was not ported to
//! `a11y-rules` for lack of evidence (no hit on 48 real pages or in the
//! corpus) and is deleted here. The note stays until the shared manual
//! checklist (casoon/barrierlab#39) has an item for 2.5.2: whether an action
//! fires on the down-event only shows under a real pointer.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub(super) const POINTER_CANCELLATION_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.2",
    name: "Pointer Cancellation",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Actions are not triggered on the down-event unless essential",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/pointer-cancellation.html",
    axe_id: "pointer-cancellation",
    tags: &["wcag2a", "wcag252", "cat.sensory-and-visual-cues"],
};

pub async fn check_pointer_cancellation_with_page(_page: &Page) -> Vec<Violation> {
    vec![Violation::new(
        POINTER_CANCELLATION_RULE.id,
        POINTER_CANCELLATION_RULE.name,
        POINTER_CANCELLATION_RULE.level,
        Severity::Low,
        "WCAG 2.5.2 requires manual testing to verify that actions are not triggered \
         on the down-event (or that undo/reversal mechanisms exist).",
        "page",
    )
    .with_fix(
        "Trigger actions on the up-event (mouseup/touchend/click) rather than the down-event \
         (mousedown/touchstart), unless the down-event is essential to the function.",
    )
    .with_rule_id(POINTER_CANCELLATION_RULE.axe_id)
    .with_help_url(POINTER_CANCELLATION_RULE.help_url)
    .with_kind(Outcome::Untested)]
}
