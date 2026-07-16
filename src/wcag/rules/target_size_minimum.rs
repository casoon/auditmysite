//! WCAG 2.5.8 Target Size (Minimum) (Level AA, WCAG 2.2)
//!
//! The size of the target for pointer inputs is at least 24 by 24 CSS pixels,
//! except where the target is a link in a sentence or block of text.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const TARGET_SIZE_MINIMUM_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.8",
    name: "Target Size (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Interactive targets are at least 24×24 CSS pixels",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html",
    axe_id: "target-size-minimum",
    tags: &["wcag22aa", "wcag258", "cat.sensory-and-visual-cues"],
};

const TARGET_SIZE_JS: &str = r#"
(function() {
  var MIN_SIZE = 24;
  var violations = [];
  var selectors = ['button', 'a[href]', '[role="button"]', '[role="link"]', 'input[type="submit"]', 'input[type="button"]', 'input[type="reset"]'];
  var elements = [];

  for (var s = 0; s < selectors.length; s++) {
    var els = document.querySelectorAll(selectors[s]);
    for (var i = 0; i < els.length && elements.length < 50; i++) {
      elements.push(els[i]);
    }
  }

  for (var j = 0; j < elements.length && violations.length < 5; j++) {
    var el = elements[j];
    var rect = el.getBoundingClientRect();
    // A near-zero rect (commonly exactly 1x1) is the standard "visually
    // hidden until focus" CSS technique for things like skip links — not
    // actually laid out/visible (display:none, a collapsed mobile-nav toggle
    // at the current viewport), or intentionally not a pointer target in its
    // resting state. A genuinely too-small but real button/icon is virtually
    // never this tiny, so this threshold doesn't mask real violations.
    if (rect.width <= 2 || rect.height <= 2) continue;
    if (rect.width < MIN_SIZE || rect.height < MIN_SIZE) {
      var desc = el.getAttribute('aria-label') || el.textContent.trim().substring(0, 40) || el.tagName.toLowerCase();
      violations.push({
        selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''),
        label: desc,
        width: Math.round(rect.width),
        height: Math.round(rect.height)
      });
    }
  }

  return { violations: violations };
})()
"#;

pub async fn check_target_size_minimum_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(TARGET_SIZE_JS).await {
        Ok(r) => r,
        Err(_) => {
            return vec![crate::wcag::technical_rule_failure(
                &TARGET_SIZE_MINIMUM_RULE,
                "page_evaluation_failed",
            )]
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &TARGET_SIZE_MINIMUM_RULE,
                "missing_evaluation_value",
            )]
        }
    };

    let violations = match val.get("violations").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &TARGET_SIZE_MINIMUM_RULE,
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
                TARGET_SIZE_MINIMUM_RULE.id,
                TARGET_SIZE_MINIMUM_RULE.name,
                TARGET_SIZE_MINIMUM_RULE.level,
                Severity::Medium,
                format!(
                    "Interactive target '{}' is {}×{} CSS pixels, below the 24×24 minimum.",
                    label, width, height
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix(
                "Increase the target size to at least 24×24 CSS pixels using padding, \
                 min-width/min-height, or by enlarging the element.",
            )
            .with_rule_id(TARGET_SIZE_MINIMUM_RULE.axe_id)
            .with_help_url(TARGET_SIZE_MINIMUM_RULE.help_url)
        })
        .collect()
}
