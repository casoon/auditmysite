//! WCAG 2.5.2 Pointer Cancellation (Level AAA)
//!
//! For functionality that can be operated using a single pointer, at least one
//! of the following is true: no down-event, abort or undo, up reversal, or essential.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const POINTER_CANCELLATION_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.2",
    name: "Pointer Cancellation",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description: "Actions are not triggered on the down-event unless essential",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/pointer-cancellation.html",
    axe_id: "pointer-cancellation",
    tags: &["wcag2aaa", "wcag252", "cat.sensory-and-visual-cues"],
};

const POINTER_CANCELLATION_JS: &str = r#"
(function() {
  var found = [];
  var interactives = document.querySelectorAll('button, a[href], [role="button"], input[type="submit"], input[type="button"]');
  for (var i = 0; i < Math.min(interactives.length, 100); i++) {
    var el = interactives[i];
    if (el.hasAttribute('onmousedown') || el.hasAttribute('ontouchstart')) {
      var desc = el.getAttribute('aria-label') || el.textContent.trim().substring(0, 40) || el.tagName.toLowerCase();
      found.push(desc);
    }
  }
  // Also scan inline scripts
  var hasDownHandlers = false;
  var scripts = document.querySelectorAll('script:not([src])');
  for (var j = 0; j < scripts.length; j++) {
    var content = scripts[j].textContent || '';
    if (/addEventListener\s*\(\s*['"]mousedown['"]/.test(content) || /addEventListener\s*\(\s*['"]touchstart['"]/.test(content)) {
      hasDownHandlers = true;
      break;
    }
  }
  return { found: found, hasDownHandlers: hasDownHandlers };
})()
"#;

pub async fn check_pointer_cancellation_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
        POINTER_CANCELLATION_RULE.id,
        POINTER_CANCELLATION_RULE.name,
        POINTER_CANCELLATION_RULE.level,
        Severity::Low,
        "WCAG 2.5.2 requires manual testing to verify that actions are not triggered \
         on the down-event (or that undo/reversal mechanisms exist).",
        "page",
    )
    .with_fix(
        "Trigger actions on the up-event (mouseup/touchend/click) rather than the down-event \
         (mousedown/touchstart), unless the down-event is essential to the function.",
    )
    .with_rule_id(POINTER_CANCELLATION_RULE.axe_id)
    .with_help_url(POINTER_CANCELLATION_RULE.help_url)
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(POINTER_CANCELLATION_JS).await {
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

    let has_down_handlers = val
        .get("hasDownHandlers")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut findings = vec![not_testable];

    if !found.is_empty() {
        findings.push(
            Violation::new(
                POINTER_CANCELLATION_RULE.id,
                POINTER_CANCELLATION_RULE.name,
                POINTER_CANCELLATION_RULE.level,
                Severity::Medium,
                format!(
                    "Interactive elements with onmousedown/ontouchstart inline handlers found: \
                     '{}'. These may trigger actions on pointer down, preventing cancellation.",
                    found.join("', '")
                ),
                "button, a[href]",
            )
            .with_fix(
                "Use onclick or onmouseup/ontouchend instead, or ensure the action can \
                 be cancelled by moving the pointer off the element before release.",
            )
            .with_rule_id(POINTER_CANCELLATION_RULE.axe_id)
            .with_help_url(POINTER_CANCELLATION_RULE.help_url)
            .with_kind(FindingKind::Warning),
        );
    } else if has_down_handlers {
        findings.push(
            Violation::new(
                POINTER_CANCELLATION_RULE.id,
                POINTER_CANCELLATION_RULE.name,
                POINTER_CANCELLATION_RULE.level,
                Severity::Medium,
                "Inline scripts register mousedown or touchstart event listeners. \
                 Verify these do not trigger irreversible actions.",
                "page",
            )
            .with_fix(
                "Review mousedown/touchstart handlers to ensure they do not perform \
                 actions that cannot be cancelled.",
            )
            .with_rule_id(POINTER_CANCELLATION_RULE.axe_id)
            .with_help_url(POINTER_CANCELLATION_RULE.help_url)
            .with_kind(FindingKind::Warning),
        );
    }

    findings
}
