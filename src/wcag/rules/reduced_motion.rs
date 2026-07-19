//! WCAG 2.3.3 Animation from Interactions / Best Practice: prefers-reduced-motion
//!
//! Pages with animations should honor the user's `prefers-reduced-motion`
//! preference. People with vestibular disorders can experience nausea or
//! seizures from motion.
//!
//! Check: if any stylesheet contains `animation`/`transition` declarations
//! but no `@media (prefers-reduced-motion: reduce)` block exists, emit a
//! warning-level violation.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const REDUCED_MOTION_RULE: RuleMetadata = RuleMetadata {
    id: "2.3.3",
    name: "Animation from Interactions",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Pages with animation should honor prefers-reduced-motion",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html",
    axe_id: "prefers-reduced-motion",
    tags: &[
        "wcag2aaa",
        "wcag233",
        "best-practice",
        "cat.sensory-and-visual-cues",
    ],
};

// Scans stylesheets and reports:
//   { hasAnimations: bool, hasReducedMotion: bool }
const REDUCED_MOTION_JS: &str = r#"
(function() {
  let hasAnimations = false;
  let hasReducedMotion = false;
  try {
    for (const sheet of Array.from(document.styleSheets)) {
      let rules;
      try { rules = Array.from(sheet.cssRules || []); }
      catch(e) { continue; } // cross-origin sheet
      const walk = (rs) => {
        for (const r of rs) {
          if (r.type === CSSRule.MEDIA_RULE) {
            const condition = (r.conditionText || r.media?.mediaText || '').toLowerCase();
            if (condition.includes('prefers-reduced-motion')) {
              hasReducedMotion = true;
            }
            if (r.cssRules) walk(Array.from(r.cssRules));
            continue;
          }
          if (r.type === CSSRule.STYLE_RULE && r.style) {
            const text = r.cssText || '';
            if (/\banimation\s*(?:-name|-duration|:)/i.test(text)) hasAnimations = true;
            if (/\btransition\s*(?:-property|-duration|:)/i.test(text)) hasAnimations = true;
          }
          if (r.cssRules) walk(Array.from(r.cssRules));
        }
      };
      walk(rules);
    }
  } catch(e) {}
  return { hasAnimations, hasReducedMotion };
})()
"#;

pub async fn check_reduced_motion_with_page(page: &Page) -> Vec<Violation> {
    let val =
        match crate::wcag::types::evaluate_or_fail(page, &REDUCED_MOTION_RULE, REDUCED_MOTION_JS)
            .await
        {
            Ok(v) => v,
            Err(violations) => return violations,
        };

    let has_animations = val
        .get("hasAnimations")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let has_reduced_motion = val
        .get("hasReducedMotion")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Only flag when the page has animations but no reduced-motion handling.
    if !has_animations || has_reduced_motion {
        return vec![];
    }

    vec![Violation::new(
        REDUCED_MOTION_RULE.id,
        REDUCED_MOTION_RULE.name,
        REDUCED_MOTION_RULE.level,
        Severity::Medium,
        "Stylesheets define animations or transitions but the page does not honor `prefers-reduced-motion`. Users with vestibular disorders may experience nausea or seizures.",
        "stylesheet",
    )
    .with_fix(
        "Wrap animation/transition declarations in `@media (prefers-reduced-motion: no-preference) { ... }` or add a `@media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; } }` reset.",
    )
    .with_rule_id(REDUCED_MOTION_RULE.axe_id)
    .with_help_url(REDUCED_MOTION_RULE.help_url)]
}
