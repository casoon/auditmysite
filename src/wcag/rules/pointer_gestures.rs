//! WCAG 2.5.1 Pointer Gestures (Level A)
//!
//! All functionality that uses multipoint or path-based gestures for operation
//! can be operated with a single pointer without a path-based gesture.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub const POINTER_GESTURES_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.1",
    name: "Pointer Gestures",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "All functionality can be operated with single-pointer gestures",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/pointer-gestures.html",
    axe_id: "pointer-gestures",
    tags: &["wcag2a", "wcag251", "cat.sensory-and-visual-cues"],
};

pub async fn check_pointer_gestures_with_page(_page: &Page) -> Vec<Violation> {
    vec![Violation::new(
        POINTER_GESTURES_RULE.id,
        POINTER_GESTURES_RULE.name,
        POINTER_GESTURES_RULE.level,
        Severity::Low,
        "Full compliance with WCAG 2.5.1 requires manual testing to verify that all \
         multipoint gestures have single-pointer alternatives.",
        "page",
    )
    .with_fix(
        "For every multipoint or path-based gesture, provide an equivalent \
         single-pointer alternative (e.g. a button).",
    )
    .with_rule_id(POINTER_GESTURES_RULE.axe_id)
    .with_help_url(POINTER_GESTURES_RULE.help_url)
    .with_kind(Outcome::Untested)]
}
