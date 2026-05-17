//! WCAG 2.2.1 Timing Adjustable (Level A)
//!
//! Time limits must be adjustable, extendable, or able to be turned off.
//! This detection only catches the most reliable structural signal:
//! `<meta http-equiv="refresh">` directives that auto-redirect the page.
//! Script-driven timeouts are not detectable from static markup and remain
//! a manual-review concern.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const TIMING_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.1",
    name: "Timing Adjustable",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Time limits must be adjustable or removable",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/timing-adjustable.html",
    axe_id: "meta-refresh",
    tags: &["wcag2a", "wcag221", "cat.time-and-media"],
};

// Returns the meta-refresh content attribute when one is present.
// Format: "<seconds>; url=<target>" or just "<seconds>".
const META_REFRESH_JS: &str = r#"
(function() {
  const meta = document.querySelector('meta[http-equiv="refresh" i]');
  if (!meta) return { present: false };
  const content = (meta.getAttribute('content') || '').trim();
  const seconds = parseInt(content.split(/[;,]/)[0], 10);
  return {
    present: true,
    content,
    seconds: isNaN(seconds) ? null : seconds
  };
})()
"#;

pub const TIMEOUT_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.6",
    name: "Timeouts",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Users are warned of data loss due to inactivity timeouts",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/timeouts.html",
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
    .with_kind(FindingKind::NotTestable)]
}

pub async fn check_timing_with_page(page: &Page) -> Vec<Violation> {
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
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(META_REFRESH_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let present = val
        .get("present")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !present {
        return vec![not_testable];
    }

    let seconds = val.get("seconds").and_then(|v| v.as_u64());
    let content = val
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut findings = vec![not_testable];

    // Refreshes >= 20 hours are exempt under WCAG (the user has effectively
    // unlimited time). We flag everything with a shorter interval (< 20h).
    let exempt = seconds.map(|s| s >= 20 * 60 * 60).unwrap_or(false);
    if !exempt {
        let detail = match seconds {
            Some(s) => format!("after {s} second(s)"),
            None => format!("(content: \"{content}\")"),
        };
        findings.push(
            Violation::new(
                TIMING_RULE.id,
                TIMING_RULE.name,
                TIMING_RULE.level,
                Severity::High,
                format!(
                    "Page uses <meta http-equiv=\"refresh\"> to auto-redirect or reload {detail} — users cannot pause, stop, or extend the timer."
                ),
                "meta[http-equiv=refresh]",
            )
            .with_selector("meta[http-equiv=refresh]")
            .with_fix(
                "Remove the meta-refresh and use a server-side redirect (HTTP 301/302) for navigation, or offer an explicit user action.",
            )
            .with_rule_id(TIMING_RULE.axe_id)
            .with_help_url(TIMING_RULE.help_url),
        );
    }

    findings
}
