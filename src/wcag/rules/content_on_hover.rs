//! WCAG 1.4.13 Content on Hover or Focus (Level AA)
//!
//! Detects two common anti-patterns:
//! - Elements relying solely on the native `title` attribute for important
//!   information (not keyboard-accessible, not screen-reader-friendly on
//!   touch devices).
//! - `role="tooltip"` elements that are not referenced by any
//!   `aria-describedby` attribute (orphaned, never announced).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const CONTENT_ON_HOVER_RULE: RuleMetadata = RuleMetadata {
    id: "1.4.13",
    name: "Content on Hover or Focus",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Content shown on hover/focus must be dismissible, hoverable, and persistent",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/content-on-hover-or-focus.html",
    axe_id: "content-on-hover-focus",
    tags: &["wcag2aa", "wcag1413", "cat.color"],
};

// Counts:
// - interactive elements (buttons, links) carrying a `title` attribute as
//   their only descriptive text (no aria-label, no visible text content)
// - role="tooltip" elements with no inbound aria-describedby reference
const CONTENT_ON_HOVER_JS: &str = r#"
(function() {
  const titleOnly = [];
  const orphanTooltips = [];
  try {
    // Collect all aria-describedby targets for tooltip orphan detection.
    const describedTargets = new Set();
    const allDescribers = document.querySelectorAll('[aria-describedby]');
    for (const el of Array.from(allDescribers)) {
      const ids = (el.getAttribute('aria-describedby') || '').split(/\s+/);
      for (const id of ids) if (id) describedTargets.add(id);
    }

    // Detect title-attribute-only patterns on interactive elements.
    const interactive = document.querySelectorAll('button[title], a[title][href], input[title]');
    for (const el of Array.from(interactive)) {
      const title = el.getAttribute('title');
      if (!title || !title.trim()) continue;
      const hasAriaLabel = !!(el.getAttribute('aria-label') || '').trim();
      const hasAriaLabelledby = !!(el.getAttribute('aria-labelledby') || '').trim();
      const textContent = (el.textContent || '').trim();
      // If accessible name comes from text/aria, title is supplemental — fine.
      if (hasAriaLabel || hasAriaLabelledby || textContent.length > 0) continue;
      const tag = el.tagName.toLowerCase();
      const id = el.id ? '#' + el.id : '';
      titleOnly.push(tag + id);
      if (titleOnly.length >= 10) break;
    }

    // Detect orphan role="tooltip" elements.
    const tooltips = document.querySelectorAll('[role="tooltip"]');
    for (const tip of Array.from(tooltips)) {
      const id = tip.id;
      if (!id || !describedTargets.has(id)) {
        orphanTooltips.push('#' + (id || '(no-id)'));
      }
      if (orphanTooltips.length >= 10) break;
    }
  } catch(e) {}
  return { titleOnly, orphanTooltips };
})()
"#;

pub async fn check_content_on_hover_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(CONTENT_ON_HOVER_JS).await {
        Ok(r) => r,
        Err(_) => {
            return vec![crate::wcag::technical_rule_failure(
                &CONTENT_ON_HOVER_RULE,
                "page_evaluation_failed",
            )]
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &CONTENT_ON_HOVER_RULE,
                "missing_evaluation_value",
            )]
        }
    };

    let title_only: Vec<String> = val
        .get("titleOnly")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let orphans: Vec<String> = val
        .get("orphanTooltips")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut violations = Vec::new();
    for sel in title_only {
        violations.push(
            Violation::new(
                CONTENT_ON_HOVER_RULE.id,
                CONTENT_ON_HOVER_RULE.name,
                CONTENT_ON_HOVER_RULE.level,
                Severity::Medium,
                format!(
                    "Interactive element uses the native `title` attribute as its only descriptive text ({sel}). Tooltips from `title` are not keyboard-accessible and unreliable on touch devices."
                ),
                &sel,
            )
            .with_selector(&sel)
            .with_fix(
                "Provide a visible label, aria-label, or aria-labelledby on the element; reserve `title` for supplemental content only.",
            )
            .with_rule_id(CONTENT_ON_HOVER_RULE.axe_id)
            .with_help_url(CONTENT_ON_HOVER_RULE.help_url),
        );
    }
    for sel in orphans {
        violations.push(
            Violation::new(
                CONTENT_ON_HOVER_RULE.id,
                CONTENT_ON_HOVER_RULE.name,
                CONTENT_ON_HOVER_RULE.level,
                Severity::Low,
                format!(
                    "role=\"tooltip\" element ({sel}) is not referenced by any aria-describedby — assistive tech will never announce it."
                ),
                &sel,
            )
            .with_selector(&sel)
            .with_fix(
                "Reference the tooltip from its trigger via aria-describedby=\"<tooltip-id>\".",
            )
            .with_rule_id(CONTENT_ON_HOVER_RULE.axe_id)
            .with_help_url(CONTENT_ON_HOVER_RULE.help_url),
        );
    }
    violations
}
