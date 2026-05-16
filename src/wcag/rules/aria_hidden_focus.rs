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

const ARIA_HIDDEN_FOCUS_JS: &str = r#"
(function() {
  var issues = [];
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
      var tag = el.tagName.toLowerCase();
      var id = el.id ? '#' + el.id : '';
      var cls = el.classList.length
        ? '.' + Array.from(el.classList).slice(0, 2).join('.')
        : '';
      issues.push({
        selector: tag + id + cls,
        snippet: el.outerHTML.substring(0, 200)
      });
      if (issues.length >= 50) break;
    }
  } catch(e) {}
  return { count: issues.length, issues: issues };
})()
"#;

pub async fn check_aria_hidden_focus(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(ARIA_HIDDEN_FOCUS_JS).await {
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

    issues
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
        .collect()
}
