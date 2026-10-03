//! WCAG 1.4.8 Visual Presentation (Level AAA)
//!
//! For blocks of text: foreground/background colours can be selected by the
//! user; width is no more than 80 characters; text is not fully justified;
//! line spacing is at least 1.5; text can be resized without AT.
//!
//! Justified text and tight line height run as `text/justified` and
//! `text/line-height-tight` in the shared rules since a11y-rules 0.19
//! (`a11y_rules::run_stylesheets`). Left here is the page-wide `UNTESTED`
//! note for colour choice and column width, which `a11y-rules` assigns to
//! its manual checklist.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub(super) const VISUAL_PRESENTATION_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.8",
    name: "Visual Presentation",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Text blocks must not be fully justified and must have adequate line spacing",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/visual-presentation.html",
    axe_id: "visual-presentation",
    tags: &["wcag2aaa", "wcag148", "cat.sensory-and-visual-cues"],
};

pub async fn check_visual_presentation_with_page(_page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
        VISUAL_PRESENTATION_RULE.id,
        VISUAL_PRESENTATION_RULE.name,
        VISUAL_PRESENTATION_RULE.level,
        Severity::Low,
        "Some aspects of 1.4.8 (user-selectable colours, column width) require manual review \
         and cannot be automatically verified.",
        "page",
    )
    .with_fix(
        "Ensure users can select foreground/background colours, text width does not exceed \
         80 characters, and text can be resized up to 200% without assistive technology.",
    )
    .with_rule_id(VISUAL_PRESENTATION_RULE.axe_id)
    .with_help_url(VISUAL_PRESENTATION_RULE.help_url)
    .with_kind(Outcome::Untested);

    vec![not_testable]
}
