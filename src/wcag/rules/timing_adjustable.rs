//! WCAG 2.2.1 Timing Adjustable (Level A)
//!
//! Time limits must be adjustable, extendable, or able to be turned off.
//! This detection only catches the most reliable structural signal:
//! `<meta http-equiv="refresh">` directives that auto-redirect the page.
//! Script-driven timeouts are not detectable from static markup and remain
//! a manual-review concern.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

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

pub async fn check_timing_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(META_REFRESH_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let present = val
        .get("present")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !present {
        return vec![];
    }

    let seconds = val.get("seconds").and_then(|v| v.as_u64());
    let content = val
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Refreshes >= 20 hours are exempt under WCAG (the user has effectively
    // unlimited time). Refreshes that point to the current URL within a
    // long interval are also a softer case. We flag everything with a
    // shorter interval (< 20h) at High severity.
    let exempt = seconds.map(|s| s >= 20 * 60 * 60).unwrap_or(false);
    if exempt {
        return vec![];
    }

    let detail = match seconds {
        Some(s) => format!("after {s} second(s)"),
        None => format!("(content: \"{content}\")"),
    };

    vec![Violation::new(
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
    .with_help_url(TIMING_RULE.help_url)]
}
