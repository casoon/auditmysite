//! WCAG 2.1.1 Keyboard — detect `onclick` on non-interactive elements
//!
//! Pages that attach click handlers to `<div>`, `<span>`, `<p>`, or `<li>`
//! without making them keyboard-operable leave a subset of users unable to
//! activate the element. This check inspects the DOM for inline `onclick`
//! attributes on non-interactive tags without a corresponding `role` or
//! `tabindex`.
//!
//! Note: this only catches inline `onclick=` attributes. JS-attached listeners
//! (`addEventListener('click', ...)`) are not visible from CSS/DOM inspection
//! and remain a manual-review concern.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const CLICK_HANDLERS_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.1",
    name: "Keyboard",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Click handlers on non-interactive elements must be keyboard-accessible",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/keyboard.html",
    axe_id: "click-events-have-key-events",
    tags: &["wcag2a", "wcag211", "cat.keyboard"],
};

// Scans the DOM for non-interactive elements carrying an inline onclick attribute
// without a focusable role or tabindex. Returns up to 10 affected selectors.
const CLICK_HANDLERS_JS: &str = r#"
(function() {
  const NON_INTERACTIVE = ['div', 'span', 'p', 'li', 'section', 'article'];
  const INTERACTIVE_ROLES = new Set([
    'button', 'link', 'checkbox', 'radio', 'switch',
    'menuitem', 'menuitemcheckbox', 'menuitemradio',
    'tab', 'option', 'treeitem'
  ]);
  const findings = [];
  try {
    const all = document.querySelectorAll('[onclick]');
    for (const el of Array.from(all)) {
      const tag = el.tagName.toLowerCase();
      if (!NON_INTERACTIVE.includes(tag)) continue;
      const role = (el.getAttribute('role') || '').toLowerCase();
      if (INTERACTIVE_ROLES.has(role)) continue;
      const tabindex = el.getAttribute('tabindex');
      const focusable = tabindex !== null && parseInt(tabindex, 10) >= 0;
      if (focusable) continue;
      const id = el.id ? '#' + el.id : '';
      const cls = el.className && typeof el.className === 'string'
        ? '.' + el.className.trim().split(/\s+/).slice(0, 2).join('.')
        : '';
      findings.push(tag + id + cls);
      if (findings.length >= 10) break;
    }
  } catch(e) {}
  return { count: findings.length, selectors: findings };
})()
"#;

pub async fn check_click_handlers_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(CLICK_HANDLERS_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let count = val.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    if count == 0 {
        return vec![];
    }

    let selectors: Vec<String> = val
        .get("selectors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    selectors
        .into_iter()
        .map(|sel| {
            Violation::new(
                CLICK_HANDLERS_RULE.id,
                CLICK_HANDLERS_RULE.name,
                CLICK_HANDLERS_RULE.level,
                Severity::High,
                format!(
                    "Element with onclick handler on non-interactive tag has no keyboard equivalent ({sel})."
                ),
                &sel,
            )
            .with_selector(&sel)
            .with_fix(
                "Replace with a native interactive element (`<button>`, `<a>`), or add `role=\"button\"` + `tabindex=\"0\"` + a keyboard handler (`onkeydown`/`onkeyup`).",
            )
            .with_rule_id(CLICK_HANDLERS_RULE.axe_id)
            .with_help_url(CLICK_HANDLERS_RULE.help_url)
        })
        .collect()
}
