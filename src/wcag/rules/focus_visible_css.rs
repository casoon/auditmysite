//! WCAG 2.4.7 Focus Visible — CSS-level check
//!
//! Complements the AXTree-based focus_visible rule by inspecting stylesheets
//! for `:focus { outline: none }` (and variants) without a compensating focus
//! indicator (border, box-shadow, background-color). Sites that globally
//! disable the default focus ring without replacement leave keyboard users
//! unable to see what's focused.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const FOCUS_VISIBLE_CSS_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.7",
    name: "Focus Visible",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Keyboard focus indicator must be visible",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/focus-visible.html",
    axe_id: "focus-visible-outline-none",
    tags: &["wcag2aa", "wcag247", "cat.keyboard"],
};

// Scans stylesheets for :focus rules that remove the outline without a
// compensating visual indicator. Returns:
//   { suppressed: bool, hasFocusVisible: bool, detail: string }
const FOCUS_OUTLINE_JS: &str = r#"
(function() {
  let suppressedRules = [];
  let hasFocusVisible = false;
  try {
    for (const sheet of Array.from(document.styleSheets)) {
      let rules;
      try { rules = Array.from(sheet.cssRules || []); }
      catch(e) { continue; } // cross-origin sheet
      const flatten = (rs) => {
        const out = [];
        for (const r of rs) {
          if (r.type === CSSRule.STYLE_RULE) out.push(r);
          else if (r.cssRules) out.push(...flatten(Array.from(r.cssRules)));
        }
        return out;
      };
      for (const rule of flatten(rules)) {
        const sel = (rule.selectorText || '').toLowerCase();
        if (sel.includes(':focus-visible')) hasFocusVisible = true;
        if (!sel.includes(':focus')) continue;
        if (sel.includes(':focus-visible')) continue; // :focus-visible alone is fine
        const style = rule.style;
        if (!style) continue;
        const outline = style.outline || style.outlineStyle || style.outlineWidth || '';
        const suppresses = outline === 'none' || outline === '0' || outline === '0px' ||
          (style.outlineStyle === 'none') || (style.outlineWidth === '0' || style.outlineWidth === '0px');
        if (!suppresses) continue;
        // Compensating indicator?
        const hasBorder = style.border && style.border !== 'none' && style.border !== '0' && style.border !== '';
        const hasBoxShadow = style.boxShadow && style.boxShadow !== 'none' && style.boxShadow !== '';
        const hasBg = style.backgroundColor && style.backgroundColor !== '' && style.backgroundColor !== 'transparent';
        if (!hasBorder && !hasBoxShadow && !hasBg) {
          suppressedRules.push(sel);
        }
      }
    }
  } catch(e) {}
  return {
    suppressed: suppressedRules.length > 0,
    hasFocusVisible,
    detail: suppressedRules.slice(0, 3).join(', ')
  };
})()
"#;

pub async fn check_focus_visible_css_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(FOCUS_OUTLINE_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let suppressed = val
        .get("suppressed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !suppressed {
        return vec![];
    }

    // If :focus-visible is also present, the suppression is likely scoped to
    // mouse-only focus (acceptable pattern). Don't flag in that case.
    let has_focus_visible = val
        .get("hasFocusVisible")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if has_focus_visible {
        return vec![];
    }

    let detail = val
        .get("detail")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let detail_suffix = if detail.is_empty() {
        String::new()
    } else {
        format!(" — selectors: {detail}")
    };

    vec![Violation::new(
        FOCUS_VISIBLE_CSS_RULE.id,
        FOCUS_VISIBLE_CSS_RULE.name,
        FOCUS_VISIBLE_CSS_RULE.level,
        Severity::High,
        format!(
            "Stylesheet removes :focus outline without a compensating visible indicator{detail_suffix}. Keyboard users cannot see what is focused."
        ),
        "stylesheet",
    )
    .with_fix(
        "Either keep the default outline, or pair `outline: none` with `border`, `box-shadow`, or `background-color` on `:focus`. Better: scope suppression to `:focus:not(:focus-visible)` so keyboard focus still shows.",
    )
    .with_rule_id(FOCUS_VISIBLE_CSS_RULE.axe_id)
    .with_help_url(FOCUS_VISIBLE_CSS_RULE.help_url)]
}
