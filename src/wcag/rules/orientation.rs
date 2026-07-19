//! WCAG 1.3.4 Orientation
//!
//! Content does not restrict its view and operation to a single display
//! orientation, such as portrait or landscape, unless a specific display
//! orientation is essential.
//! Level AA
//!
//! Checks via CSS/JS inspection:
//! - Orientation media queries that hide or transform significant content
//! - CSS transform: rotate on <body> or <html>

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const ORIENTATION_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.4",
    name: "Orientation",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Content must not be restricted to a single display orientation",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/orientation.html",
    axe_id: "css-orientation-lock",
    tags: &["wcag2aa", "wcag134", "cat.sensory-and-visual-cues"],
};

// JavaScript that scans stylesheets for orientation-locking rules.
// Returns a JSON object: { locked: bool, detail: string }
const ORIENTATION_LOCK_JS: &str = r#"
(function() {
  try {
    for (const sheet of Array.from(document.styleSheets)) {
      let rules;
      try { rules = Array.from(sheet.cssRules || []); }
      catch(e) { continue; } // cross-origin sheet
      for (const rule of rules) {
        if (rule.type !== CSSRule.MEDIA_RULE) continue;
        const media = (rule.conditionText || rule.media.mediaText || '').toLowerCase();
        if (!media.includes('orientation')) continue;
        // Rule targets a specific orientation — check if it hides content
        for (const inner of Array.from(rule.cssRules || [])) {
          const style = inner.style;
          if (!style) continue;
          if (style.display === 'none' || style.visibility === 'hidden') {
            return { locked: true, detail: 'orientation media query hides content: ' + media };
          }
          if (style.transform && style.transform.includes('rotate')) {
            return { locked: true, detail: 'orientation media query rotates content: ' + media };
          }
        }
      }
    }
    // Check inline transforms on body/html (JS-driven orientation lock workaround)
    const bodyTransform = window.getComputedStyle(document.body).transform;
    const htmlTransform = window.getComputedStyle(document.documentElement).transform;
    for (const [el, t] of [['body', bodyTransform], ['html', htmlTransform]]) {
      if (t && t !== 'none' && t.includes('rotate')) {
        return { locked: true, detail: el + ' has CSS transform: rotate — may indicate orientation lock' };
      }
    }
  } catch(e) {}
  return { locked: false, detail: '' };
})()
"#;

pub async fn check_orientation_with_page(page: &Page) -> Vec<Violation> {
    let val =
        match crate::wcag::types::evaluate_or_fail(page, &ORIENTATION_RULE, ORIENTATION_LOCK_JS)
            .await
        {
            Ok(v) => v,
            Err(violations) => return violations,
        };

    let locked = val.get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
    if !locked {
        return vec![];
    }

    let detail = val
        .get("detail")
        .and_then(|v| v.as_str())
        .unwrap_or("orientation lock detected")
        .to_string();

    vec![Violation::new(
        ORIENTATION_RULE.id,
        ORIENTATION_RULE.name,
        ORIENTATION_RULE.level,
        Severity::High,
        format!(
            "Page appears to lock display orientation — {detail}. Users who cannot rotate their device will be unable to access content."
        ),
        "document",
    )
    .with_fix(
        "Remove orientation-specific CSS that hides or rotates content. \
         Let the OS and browser handle orientation changes. \
         Only lock orientation if it is essential to the content (e.g. a piano keyboard).",
    )
    .with_rule_id(ORIENTATION_RULE.axe_id)
    .with_help_url(ORIENTATION_RULE.help_url)]
}
