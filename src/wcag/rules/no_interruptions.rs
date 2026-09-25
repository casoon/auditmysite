//! WCAG 2.2.4 Interruptions (Level AAA)
//!
//! Interruptions can be postponed or suppressed by the user, except interruptions
//! involving an emergency.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub const NO_INTERRUPTIONS_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.4",
    name: "Interruptions",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Interruptions can be postponed or suppressed by the user",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/interruptions.html",
    axe_id: "no-interruptions",
    tags: &["wcag2aaa", "wcag224", "cat.time-and-media"],
};

pub async fn check_no_interruptions_with_page(_page: &Page) -> Vec<Violation> {
    vec![Violation::new(
        NO_INTERRUPTIONS_RULE.id,
        NO_INTERRUPTIONS_RULE.name,
        NO_INTERRUPTIONS_RULE.level,
        Severity::Low,
        "WCAG 2.2.4 (Interruptions) requires manual testing to verify that users can \
         postpone or suppress non-emergency interruptions such as alerts, popups, and \
         live region updates.",
        "page",
    )
    .with_fix(
        "Provide users with settings to control or suppress non-essential notifications \
         and interruptions.",
    )
    .with_rule_id(NO_INTERRUPTIONS_RULE.axe_id)
    .with_help_url(NO_INTERRUPTIONS_RULE.help_url)
    .with_kind(Outcome::Untested)]
}
