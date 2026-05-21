//! WCAG 2.5.1 Pointer Gestures (Level AAA)
//!
//! All functionality that uses multipoint or path-based gestures for operation
//! can be operated with a single pointer without a path-based gesture.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const POINTER_GESTURES_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.1",
    name: "Pointer Gestures",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "All functionality can be operated with single-pointer gestures",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/pointer-gestures.html",
    axe_id: "pointer-gestures",
    tags: &["wcag2aaa", "wcag251", "cat.sensory-and-visual-cues"],
};

const POINTER_GESTURES_JS: &str = r#"
(function() {
  var gesturePatterns = ['gesturestart', 'gesturechange', 'gestureend', 'ontouchstart', 'ongesturestart'];
  var found = [];

  // Check inline handlers on elements
  var allElements = document.querySelectorAll('*');
  for (var i = 0; i < Math.min(allElements.length, 200); i++) {
    var el = allElements[i];
    for (var j = 0; j < gesturePatterns.length; j++) {
      if (el.hasAttribute(gesturePatterns[j])) {
        found.push(gesturePatterns[j]);
        break;
      }
    }
  }

  // Check inline scripts for gesture event strings
  var scripts = document.querySelectorAll('script:not([src])');
  for (var k = 0; k < scripts.length; k++) {
    var content = scripts[k].textContent || '';
    for (var m = 0; m < gesturePatterns.length; m++) {
      if (content.indexOf(gesturePatterns[m]) !== -1) {
        found.push('script:' + gesturePatterns[m]);
        break;
      }
    }
  }

  return { found: found };
})()
"#;

pub async fn check_pointer_gestures_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
        POINTER_GESTURES_RULE.id,
        POINTER_GESTURES_RULE.name,
        POINTER_GESTURES_RULE.level,
        Severity::Low,
        "Full compliance with WCAG 2.5.1 requires manual testing to verify that all \
         multipoint gestures have single-pointer alternatives.",
        "page",
    )
    .with_fix(
        "For every multipoint or path-based gesture, provide an equivalent \
         single-pointer alternative (e.g. a button).",
    )
    .with_rule_id(POINTER_GESTURES_RULE.axe_id)
    .with_help_url(POINTER_GESTURES_RULE.help_url)
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(POINTER_GESTURES_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let found: Vec<String> = val
        .get("found")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut findings = vec![not_testable];

    if !found.is_empty() {
        let patterns = found.join(", ");
        findings.push(
            Violation::new(
                POINTER_GESTURES_RULE.id,
                POINTER_GESTURES_RULE.name,
                POINTER_GESTURES_RULE.level,
                Severity::Medium,
                format!(
                    "Potential multipoint gesture event handlers detected: {}. \
                     Verify that single-pointer alternatives exist for all gesture-based interactions.",
                    patterns
                ),
                "page",
            )
            .with_fix(
                "Ensure every touch/gesture interaction also works with a single click or tap. \
                 Add equivalent button controls for swipe, pinch, or multi-touch actions.",
            )
            .with_rule_id(POINTER_GESTURES_RULE.axe_id)
            .with_help_url(POINTER_GESTURES_RULE.help_url)
            .with_kind(FindingKind::Warning),
        );
    }

    findings
}
