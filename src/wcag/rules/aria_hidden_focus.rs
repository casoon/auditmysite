//! WCAG 4.1.2 - aria-hidden-focus
//!
//! Focusable elements (tabIndex >= 0) must not be inside an aria-hidden="true"
//! subtree. Keyboard users can tab into these elements but screen readers
//! cannot announce them — creating invisible focus traps.

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const ARIA_HIDDEN_FOCUS_RULE: RuleMetadata = RuleMetadata {
    id: "aria-hidden-focus",
    name: "aria-hidden-focus",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Focusable elements must not be contained within an aria-hidden subtree",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value.html",
    axe_id: "aria-hidden-focus",
    tags: &["wcag2a", "wcag412", "cat.aria"],
};

/// Maximum number of detailed findings returned; further matches are still
/// counted and surfaced as an explicit "truncated" note rather than dropped
/// silently.
const ARIA_HIDDEN_FOCUS_CAP: usize = 250;

/// Body of the aria-hidden-focus script (wrapped in an IIFE at call time,
/// after the shared `__amsCssSelector` helper).
const ARIA_HIDDEN_FOCUS_BODY: &str = r#"
  var issues = [];
  var total = 0;
  // A focusable element is only a real focus trap if it is actually reachable:
  // display:none / visibility:hidden / inert ancestors take it out of the tab
  // order, so tabIndex alone is not a reliability indicator.
  function isReachable(el) {
    if (el.closest('[inert]')) return false;
    var cur = el;
    while (cur && cur.nodeType === 1 && cur !== document.documentElement) {
      var s = window.getComputedStyle(cur);
      if (s.display === 'none') return false;
      if (s.visibility === 'hidden' || s.visibility === 'collapse') return false;
      cur = cur.parentElement;
    }
    return true;
  }
  try {
    var elems = document.querySelectorAll(
      'a[href], button:not([disabled]), input:not([disabled]), ' +
      'select:not([disabled]), textarea:not([disabled]), ' +
      '[tabindex]:not([tabindex="-1"])'
    );
    var seen = new Set();
    for (var i = 0; i < elems.length; i++) {
      var el = elems[i];
      if (seen.has(el)) continue;
      seen.add(el);
      if (el.tabIndex < 0) continue;
      var hidden = el.closest('[aria-hidden="true"]');
      if (!hidden) continue;
      if (!isReachable(el)) continue;
      total++;
      if (issues.length < CAP) {
        issues.push({
          selector: __amsCssSelector(el),
          snippet: el.outerHTML.substring(0, 200)
        });
      }
    }
  } catch(e) {}
  return { count: total, returned: issues.length, truncated: total > issues.length, issues: issues };
"#;

pub async fn check_aria_hidden_focus(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &ARIA_HIDDEN_FOCUS_BODY.replace("CAP", &ARIA_HIDDEN_FOCUS_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("aria-hidden-focus JS failed: {}", e);
            return vec![];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let count = val.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    if count == 0 {
        return vec![];
    }

    let issues = val
        .get("issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut violations: Vec<Violation> = issues
        .into_iter()
        .filter_map(|issue| {
            let selector = issue.get("selector")?.as_str()?.to_string();
            let snippet = issue
                .get("snippet")
                .and_then(|v| v.as_str())
                .map(String::from);

            let mut violation = Violation::new(
                ARIA_HIDDEN_FOCUS_RULE.id,
                ARIA_HIDDEN_FOCUS_RULE.name,
                ARIA_HIDDEN_FOCUS_RULE.level,
                Severity::High,
                format!(
                    "Focusable element '{}' is inside an aria-hidden subtree — \
                     keyboard users reach it but screen readers cannot announce it",
                    selector
                ),
                &selector,
            )
            .with_selector(&selector)
            .with_rule_id(ARIA_HIDDEN_FOCUS_RULE.axe_id)
            .with_fix(
                "Remove aria-hidden=\"true\" from the ancestor, move the element outside \
                 the hidden region, or add tabindex=\"-1\" to remove it from tab order.",
            )
            .with_help_url(ARIA_HIDDEN_FOCUS_RULE.help_url);

            if let Some(s) = snippet {
                violation = violation.with_html_snippet(s);
            }

            Some(violation)
        })
        .collect();

    // Surface truncation explicitly instead of silently capping the output.
    if val
        .get("truncated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let returned = val.get("returned").and_then(|v| v.as_u64()).unwrap_or(0);
        let remaining = count.saturating_sub(returned);
        warn!(
            "aria-hidden-focus: {} findings truncated ({} total, {} reported)",
            remaining, count, returned
        );
        let mut note = Violation::new(
            ARIA_HIDDEN_FOCUS_RULE.id,
            ARIA_HIDDEN_FOCUS_RULE.name,
            ARIA_HIDDEN_FOCUS_RULE.level,
            Severity::High,
            format!(
                "Output truncated: {} further focusable elements inside aria-hidden \
                 subtrees were detected but not listed individually ({} total).",
                remaining, count
            ),
            "aria-hidden-focus-truncated",
        )
        .with_rule_id(ARIA_HIDDEN_FOCUS_RULE.axe_id)
        .with_help_url(ARIA_HIDDEN_FOCUS_RULE.help_url);
        note.kind = crate::wcag::types::FindingKind::Warning;
        violations.push(note);
    }

    violations
}
