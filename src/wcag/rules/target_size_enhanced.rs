//! WCAG 2.5.5 Target Size (Enhanced) (Level AAA)
//!
//! The size of the target for pointer inputs is at least 44 by 44 CSS pixels,
//! except where the target is a link in a sentence or block of text, or
//! where an equivalent link on the same page meets the size.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const TARGET_SIZE_ENHANCED_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.5",
    name: "Target Size (Enhanced)",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Interactive targets are at least 44×44 CSS pixels",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/target-size-enhanced.html",
    axe_id: "target-size",
    tags: &["wcag2aaa", "wcag255", "cat.sensory-and-visual-cues"],
};

const TARGET_SIZE_JS: &str = r#"
(function() {
  var MIN_SIZE = 44;
  var violations = [];
  var selectors = ['button', 'a[href]', '[role="button"]', '[role="link"]', 'input[type="submit"]', 'input[type="button"]', 'input[type="reset"]'];
  var elements = [];

  for (var s = 0; s < selectors.length; s++) {
    var els = document.querySelectorAll(selectors[s]);
    for (var i = 0; i < els.length && __amsReal(elements) < 50; i++) {
      __amsPush(elements, els[i], els[i], 50);
    }
  }

  /*TARGET_HELPERS*/
  for (var j = 0; j < elements.length && __amsReal(violations) < 5; j++) {
    var el = elements[j];
    var rect = el.getBoundingClientRect();
    // 2.5.5 has no spacing exception, only the inline and equivalent ones.
    if ((rect.width < MIN_SIZE || rect.height < MIN_SIZE) && !isInlineInText(el) && !hasEquivalentLink(el, MIN_SIZE)) {
      var desc = el.getAttribute('aria-label') || el.textContent.trim().substring(0, 40) || el.tagName.toLowerCase();
      __amsPush(violations, el, {
        selector: __amsCssSelector(el),
        label: desc,
        width: Math.round(rect.width),
        height: Math.round(rect.height)
      }, 5);
    }
  }

  return { violations: violations };
})()
"#;

pub async fn check_target_size_enhanced_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &TARGET_SIZE_ENHANCED_RULE,
        &TARGET_SIZE_JS.replace(
            "/*TARGET_HELPERS*/",
            &[
                super::target_size_minimum::TARGET_HELPERS_JS,
                crate::accessibility::js_helpers::CSS_SELECTOR_JS,
            ]
            .concat(),
        ),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let violations = match val.get("violations").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &TARGET_SIZE_ENHANCED_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    violations
        .iter()
        .map(|item| {
            let selector = item
                .get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("element");
            let label = item
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("element");
            let width = item.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
            let height = item.get("height").and_then(|v| v.as_u64()).unwrap_or(0);

            Violation::new(
                TARGET_SIZE_ENHANCED_RULE.id,
                TARGET_SIZE_ENHANCED_RULE.name,
                TARGET_SIZE_ENHANCED_RULE.level,
                Severity::Medium,
                format!(
                    "Interactive target '{}' is {}×{} CSS pixels, below the 44×44 minimum.",
                    label, width, height
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix(
                "Increase the target size to at least 44×44 CSS pixels using padding, \
                 min-width/min-height, or by enlarging the element.",
            )
            .with_rule_id(TARGET_SIZE_ENHANCED_RULE.axe_id)
            .with_help_url(TARGET_SIZE_ENHANCED_RULE.help_url)
        })
        .collect()
}
