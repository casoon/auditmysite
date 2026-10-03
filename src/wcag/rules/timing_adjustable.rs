//! WCAG 2.2.1 Timing Adjustable (Level A)
//!
//! Time limits must be adjustable, extendable, or able to be turned off.
//! `<meta http-equiv="refresh">` runs as `timing/meta-refresh` in the shared
//! rules since #697. What stays here are the page-wide `UNTESTED` notes for
//! script-driven time limits (2.2.1) and inactivity timeouts (2.2.6):
//! `a11y-rules` assigns them to its manual checklist (`manual/timing`), which
//! auditmysite does not run yet.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub(super) const TIMING_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.1",
    name: "Timing Adjustable",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Time limits must be adjustable or removable",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable.html",
    axe_id: "meta-refresh",
    tags: &["wcag2a", "wcag221", "cat.time-and-media"],
};

pub(super) const TIMEOUT_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.6",
    name: "Timeouts",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Users are warned of data loss due to inactivity timeouts",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/timeouts.html",
    axe_id: "timeouts",
    tags: &["wcag21aaa", "wcag226", "cat.time-and-media"],
};

pub async fn check_timeouts_with_page(_page: &Page) -> Vec<Violation> {
    vec![Violation::new(
        TIMEOUT_RULE.id,
        TIMEOUT_RULE.name,
        TIMEOUT_RULE.level,
        Severity::Medium,
        "WCAG 2.2.6 (Timeouts) requires manual testing. Verify that users are warned \
         about inactivity timeouts that could cause data loss, at least 20 seconds \
         before the session expires.",
        "page",
    )
    .with_fix(
        "Display a warning before session expiry that tells users how long they have \
         left and allows them to extend the session. Preserve entered data across \
         authentication timeouts.",
    )
    .with_rule_id(TIMEOUT_RULE.axe_id)
    .with_help_url(TIMEOUT_RULE.help_url)
    .with_kind(Outcome::Untested)]
}

pub async fn check_timing_with_page(_page: &Page) -> Vec<Violation> {
    // JavaScript-driven session timeouts (setTimeout/setInterval) are not
    // detectable from the DOM and always require manual testing.
    let not_testable = Violation::new(
        TIMING_RULE.id,
        TIMING_RULE.name,
        TIMING_RULE.level,
        Severity::Medium,
        "JavaScript-driven time limits (setTimeout/setInterval) are not automatically detectable. \
         If the page has session timeouts or timed interactions, verify that users can turn off, \
         adjust, or extend them.",
        "page",
    )
    .with_fix(
        "Provide a mechanism to disable, adjust (at least 10×), or extend any time limit before it expires, \
         with at least 20 seconds to respond.",
    )
    .with_rule_id(TIMING_RULE.axe_id)
    .with_help_url(TIMING_RULE.help_url)
    .with_kind(Outcome::Untested);

    vec![not_testable]
}
