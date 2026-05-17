//! WCAG 2.2.3 No Timing (Level AAA)
//!
//! Timing is not an essential part of the event or activity presented by the
//! content, except for non-interactive synchronized media and real-time events.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const NO_TIMING_RULE: RuleMetadata = RuleMetadata {
    id: "2.2.3",
    name: "No Timing",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Timing is not essential for content unless it is synchronized media or real-time",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/no-timing.html",
    axe_id: "no-timing",
    tags: &["wcag2aaa", "wcag223", "cat.time-and-media"],
};

const NO_TIMING_JS: &str = r#"
(function() {
  var hasTimingCalls = false;
  var scripts = document.querySelectorAll('script:not([src])');
  for (var i = 0; i < scripts.length; i++) {
    var content = scripts[i].textContent || '';
    // Remove single-line comments before checking
    var stripped = content.replace(/\/\/[^\n]*/g, '');
    if (/\bsetTimeout\s*\(/.test(stripped) || /\bsetInterval\s*\(/.test(stripped)) {
      hasTimingCalls = true;
      break;
    }
  }
  return { hasTimingCalls: hasTimingCalls };
})()
"#;

pub async fn check_no_timing_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
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
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(NO_TIMING_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let has_timing_calls = val
        .get("hasTimingCalls")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut findings = vec![not_testable];

    if has_timing_calls {
        findings.push(
            Violation::new(
                NO_TIMING_RULE.id,
                NO_TIMING_RULE.name,
                NO_TIMING_RULE.level,
                Severity::Medium,
                "Inline scripts contain setTimeout or setInterval calls. Review whether \
                 timing is essential to any user task on this page.",
                "page",
            )
            .with_fix(
                "Verify that all timed interactions are either non-essential, user-controlled, \
                 or fall under the real-time exception (WCAG 2.2.3).",
            )
            .with_rule_id(NO_TIMING_RULE.axe_id)
            .with_help_url(NO_TIMING_RULE.help_url)
            .with_kind(FindingKind::Warning),
        );
    }

    findings
}
