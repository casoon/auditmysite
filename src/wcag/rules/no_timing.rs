//! WCAG 2.2.3 No Timing (Level AAA)
//!
//! Timing is not an essential part of the event or activity presented by the
//! content, except for non-interactive synchronized media and real-time events.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub const NO_TIMING_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.3",
    name: "No Timing",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Timing is not essential for content unless it is synchronized media or real-time",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/no-timing.html",
    axe_id: "no-timing",
    tags: &["wcag2aaa", "wcag223", "cat.time-and-media"],
};

pub async fn check_no_timing_with_page(_page: &Page) -> Vec<Violation> {
    vec![Violation::new(
        NO_TIMING_RULE.id,
        NO_TIMING_RULE.name,
        NO_TIMING_RULE.level,
        Severity::Low,
        "WCAG 2.2.3 requires that timing is not essential to the task. This cannot be \
         fully verified automatically — manual review is required.",
        "page",
    )
    .with_fix(
        "Remove time-based requirements from tasks unless absolutely necessary (e.g. \
         real-time auctions). Ensure users can complete tasks at their own pace.",
    )
    .with_rule_id(NO_TIMING_RULE.axe_id)
    .with_help_url(NO_TIMING_RULE.help_url)
    .with_kind(Outcome::Untested)]
}
