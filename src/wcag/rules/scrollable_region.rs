//! WCAG 2.1.1 Keyboard — scrollable regions must be reachable by keyboard
//! (axe-core `scrollable-region-focusable`, #717).
//!
//! A region whose content overflows (`overflow: auto|scroll`) can only be
//! scrolled with the keyboard when focus can get into it: either the region
//! itself is focusable, or it contains focusable content that scrolls along.
//! Chrome makes such scrollers focusable on its own since version 130, other
//! browsers do not — so the page still fails for their users.
//!
//! Needs the rendered layout (`scrollHeight` against `clientHeight`,
//! computed `overflow`), hence a page rule.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const SCROLLABLE_REGION_RULE: RuleMetadata = RuleMetadata {
    id: "2.1.1",
    name: "Keyboard",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Scrollable regions must be focusable or contain focusable content",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html",
    axe_id: "scrollable-region-focusable",
    tags: &["wcag2a", "wcag211", "wcag213", "cat.keyboard"],
};

/// Overflow a region needs before it counts as scrollable. Sub-pixel layout
/// and small paddings let `scrollHeight` exceed `clientHeight` by a few pixels
/// without anything to scroll to; axe-core uses the same allowance.
const SCROLL_BUFFER_PX: u32 = 13;

/// Maximum findings per page, as for the other element rules.
const MAX_FINDINGS: u32 = 10;

// Body of the scan, wrapped in an IIFE after the shared selector helpers.
// Returns { violations: [{ selector, label }] }.
const SCROLLABLE_REGION_JS: &str = r#"
  var BUFFER = /*BUFFER*/;
  var LIMIT = /*LIMIT*/;
  var FOCUSABLE = 'a[href], area[href], button:not([disabled]), ' +
    'input:not([disabled]):not([type="hidden"]), select:not([disabled]), ' +
    'textarea:not([disabled]), iframe, summary, audio[controls], video[controls], ' +
    '[contenteditable]:not([contenteditable="false"]), [tabindex]';

  function keyboardFocusable(el) {
    if (!el.matches(FOCUSABLE)) return false;
    var tabindex = el.getAttribute('tabindex');
    if (tabindex !== null && parseInt(tabindex, 10) < 0) return false;
    if (el.closest('[inert]')) return false;
    return el.checkVisibility({ visibilityProperty: true });
  }

  function scrolls(el, styles) {
    var y = (styles.overflowY === 'auto' || styles.overflowY === 'scroll') &&
      el.scrollHeight - el.clientHeight > BUFFER;
    var x = (styles.overflowX === 'auto' || styles.overflowX === 'scroll') &&
      el.scrollWidth - el.clientWidth > BUFFER;
    return x || y;
  }

  var violations = [];
  var all = document.body ? document.body.querySelectorAll('*') : [];
  for (var i = 0; i < all.length; i++) {
    var el = all[i];
    // The page's own scroller is always keyboard-scrollable.
    if (el === document.scrollingElement || el === document.body) continue;
    var styles = window.getComputedStyle(el);
    if (!scrolls(el, styles)) continue;
    if (!el.checkVisibility({ visibilityProperty: true })) continue;
    if (el.closest('[aria-hidden="true"]') || el.closest('[inert]')) continue;
    // Nothing to read in it: scrolling reveals nothing (axe-core requires
    // content as well).
    if (!(el.textContent || '').trim()) continue;
    if (keyboardFocusable(el)) continue;
    var inner = el.querySelectorAll(FOCUSABLE);
    var reachable = false;
    for (var j = 0; j < inner.length && !reachable; j++) {
      reachable = keyboardFocusable(inner[j]);
    }
    if (reachable) continue;
    var label = el.getAttribute('aria-label') ||
      (el.textContent || '').trim().replace(/\s+/g, ' ').substring(0, 40);
    __amsPush(violations, el, { selector: __amsCssSelector(el), label: label }, LIMIT);
  }
  return { violations: violations };
"#;

pub async fn check_scrollable_region_focusable_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &SCROLLABLE_REGION_JS
            .replace("/*BUFFER*/", &SCROLL_BUFFER_PX.to_string())
            .replace("/*LIMIT*/", &MAX_FINDINGS.to_string()),
        "})()",
    ]
    .concat();
    let val = match crate::wcag::types::evaluate_or_fail(page, &SCROLLABLE_REGION_RULE, &js).await {
        Ok(v) => v,
        Err(violations) => return violations,
    };
    let Some(items) = val.get("violations").and_then(|v| v.as_array()) else {
        return vec![crate::wcag::technical_rule_failure(
            &SCROLLABLE_REGION_RULE,
            "invalid_evaluation_shape",
        )];
    };
    items.iter().map(build_violation).collect()
}

fn build_violation(item: &serde_json::Value) -> Violation {
    let selector = item
        .get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or("element");
    let label = item.get("label").and_then(|v| v.as_str()).unwrap_or("");
    let quoted = if label.is_empty() {
        String::new()
    } else {
        format!(" ('{label}')")
    };
    Violation::new(
        SCROLLABLE_REGION_RULE.id,
        SCROLLABLE_REGION_RULE.name,
        SCROLLABLE_REGION_RULE.level,
        SCROLLABLE_REGION_RULE.severity,
        format!(
            "Scrollable region{quoted} is not focusable and contains no focusable content, \
             so keyboard users cannot scroll it."
        ),
        selector,
    )
    .with_selector(selector)
    .with_fix(
        "Make the region focusable with tabindex=\"0\" and give it an accessible name \
         (role=\"region\" with aria-label), or remove the fixed height so the content \
         does not need its own scrollbar.",
    )
    .with_rule_id(SCROLLABLE_REGION_RULE.axe_id)
    .with_help_url(SCROLLABLE_REGION_RULE.help_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_names_the_region_and_maps_to_2_1_1() {
        let v = build_violation(&serde_json::json!({
            "selector": "div.menus",
            "label": "Menu",
        }));
        assert_eq!(v.rule, "2.1.1");
        assert_eq!(v.rule_id.as_deref(), Some("scrollable-region-focusable"));
        assert_eq!(v.selector.as_deref(), Some("div.menus"));
        assert!(v.message.contains("('Menu')"));
    }

    #[test]
    fn script_placeholders_are_filled() {
        let js = SCROLLABLE_REGION_JS
            .replace("/*BUFFER*/", &SCROLL_BUFFER_PX.to_string())
            .replace("/*LIMIT*/", &MAX_FINDINGS.to_string());
        assert!(!js.contains("/*"));
    }
}
