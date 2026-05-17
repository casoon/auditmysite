//! WCAG 2.2.5 Re-authenticating (Level AAA)
//!
//! When an authenticated session expires, the user can continue the activity
//! without loss of data after re-authenticating.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const RE_AUTHENTICATE_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.5",
    name: "Re-authenticating",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Data is preserved when users re-authenticate after a session expires",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/re-authenticating.html",
    axe_id: "re-authenticate",
    tags: &["wcag2aaa", "wcag225", "cat.time-and-media"],
};

pub async fn check_re_authenticate_with_page(_page: &Page) -> Vec<Violation> {
    vec![Violation::new(
        RE_AUTHENTICATE_RULE.id,
        RE_AUTHENTICATE_RULE.name,
        RE_AUTHENTICATE_RULE.level,
        Severity::Low,
        "WCAG 2.2.5 (Re-authenticating) requires manual testing. Verify that when an \
         authenticated session expires, users can re-authenticate and continue without \
         losing any data they have entered.",
        "page",
    )
    .with_fix(
        "Save form data and session state server-side so it can be restored after \
         re-authentication. Inform users before session expiry and preserve all entered data.",
    )
    .with_rule_id(RE_AUTHENTICATE_RULE.axe_id)
    .with_help_url(RE_AUTHENTICATE_RULE.help_url)
    .with_kind(FindingKind::NotTestable)]
}
