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
//!
//! The note appears only where 2.5.2 applies: on a page with something a
//! single pointer can operate. Controls, links, pointer handlers and
//! widget roles show in the markup; a listener added by script does not, so
//! any script (or embedded document) counts as possible functionality too.
//! A static page without any of these gets no note.

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

/// Whether the page has anything a single pointer could operate.
const POINTER_FUNCTIONALITY_JS: &str = r#"
!!document.querySelector(
  'a[href], area[href], button, input:not([type="hidden"]), select, textarea, summary, ' +
  'label, audio[controls], video[controls], iframe, object, embed, script, ' +
  '[onclick], [onmousedown], [onmouseup], [onpointerdown], [onpointerup], ' +
  '[ontouchstart], [ontouchend], [tabindex], [draggable="true"], [contenteditable], ' +
  '[role="button"], [role="link"], [role="checkbox"], [role="radio"], [role="switch"], ' +
  '[role="tab"], [role="menuitem"], [role="menuitemcheckbox"], [role="menuitemradio"], ' +
  '[role="option"], [role="slider"], [role="spinbutton"], [role="scrollbar"], ' +
  '[role="treeitem"], [role="gridcell"], [role="combobox"], [role="textbox"], [role="searchbox"]'
)
"#;

pub async fn check_pointer_cancellation_with_page(page: &Page) -> Vec<Violation> {
    let applies = match crate::wcag::types::evaluate_or_fail(
        page,
        &POINTER_CANCELLATION_RULE,
        POINTER_FUNCTIONALITY_JS,
    )
    .await
    {
        Ok(value) => value,
        Err(failure) => return failure,
    };
    let Some(applies) = applies.as_bool() else {
        return vec![crate::wcag::technical_rule_failure(
            &POINTER_CANCELLATION_RULE,
            "invalid_evaluation_shape",
        )];
    };
    if !applies {
        return Vec::new();
    }
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
