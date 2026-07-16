//! WCAG 3.2.2 On Input
//!
//! Changing the setting of any user interface component does not automatically
//! cause a change of context unless the user has been advised of the behavior
//! before using the component.
//! Level A
//!
//! Note: Full on-input testing requires behavioral analysis via CDP.
//! This rule checks for common DOM patterns: select elements and radio
//! buttons that may trigger form submission or navigation without an
//! explicit submit.
//!
//! DOM-level rule: `onchange` is an HTML attribute, never exposed as an AX
//! property — an earlier tree-based implementation of this check read a
//! non-existent AX property and never fired in production (#QA-030).

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const ON_INPUT_RULE: RuleMetadata = RuleMetadata {
    id: "3.2.2",
    name: "On Input",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Changing a setting does not automatically cause a change of context",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/on-input.html",
    axe_id: "input-no-context-change",
    tags: &["wcag2a", "wcag322", "cat.keyboard"],
};

const ON_INPUT_CAP: usize = 250;

const ON_INPUT_BODY: &str = r#"
  function accessibleName(el) {
    var label = el.getAttribute('aria-label');
    if (label && label.trim()) return label.trim();
    var labelledby = el.getAttribute('aria-labelledby');
    if (labelledby) {
      var ref = document.getElementById(labelledby.split(/\s+/)[0]);
      if (ref && ref.textContent.trim()) return ref.textContent.trim();
    }
    if (el.id) {
      var forLabel = document.querySelector('label[for="' + CSS.escape(el.id) + '"]');
      if (forLabel && forLabel.textContent.trim()) return forLabel.textContent.trim();
    }
    var parentLabel = el.closest('label');
    if (parentLabel && parentLabel.textContent.trim()) return parentLabel.textContent.trim();
    return (el.textContent || '').trim();
  }

  function isChangeControl(el) {
    if (el.tagName === 'SELECT') return true;
    var role = (el.getAttribute('role') || '').toLowerCase();
    return role === 'combobox' || role === 'listbox';
  }

  function isRadio(el) {
    if (el.tagName === 'INPUT' && (el.getAttribute('type') || '').toLowerCase() === 'radio') {
      return true;
    }
    return (el.getAttribute('role') || '').toLowerCase() === 'radio';
  }

  var navigationHints = ['sort', 'filter', 'language', 'country', 'region', 'navigate', 'redirect', 'go to'];
  var submitHints = ['submit', 'send', 'go', 'search', 'absenden'];

  var hasSubmitButton = false;
  var buttons = document.querySelectorAll('button, input[type="submit"], [role="button"]');
  for (var b = 0; b < buttons.length; b++) {
    var btnName = accessibleName(buttons[b]).toLowerCase();
    if (submitHints.some(function(h) { return btnName.indexOf(h) !== -1; })) {
      hasSubmitButton = true;
      break;
    }
  }

  var issues = [];
  var controls = document.querySelectorAll('select, [role="combobox"], [role="listbox"], input[type="radio"], [role="radio"]');
  for (var i = 0; i < controls.length && issues.length < CAP; i++) {
    var el = controls[i];
    var name = accessibleName(el);
    var nameLower = name.toLowerCase();

    if (isChangeControl(el) && el.hasAttribute('onchange')) {
      issues.push({ kind: 'onchange', role: (el.getAttribute('role') || el.tagName.toLowerCase()), selector: __amsCssSelector(el) });
      continue;
    }

    if ((isChangeControl(el) || isRadio(el)) && !hasSubmitButton) {
      if (navigationHints.some(function(h) { return nameLower.indexOf(h) !== -1; })) {
        issues.push({
          kind: 'navigation',
          role: (el.getAttribute('role') || el.tagName.toLowerCase()),
          name: nameLower,
          selector: __amsCssSelector(el)
        });
      }
    }
  }

  return { issues: issues };
"#;

pub async fn check_on_input_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &ON_INPUT_BODY.replace("CAP", &ON_INPUT_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("on-input JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "on-input",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure_for(
                "on-input",
                crate::cli::WcagLevel::A,
                "missing_evaluation_value",
            )]
        }
    };

    let issues = match val.get("issues").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return vec![],
    };

    issues
        .iter()
        .filter_map(|issue| {
            let kind = issue.get("kind")?.as_str()?;
            let role = issue.get("role").and_then(|v| v.as_str()).unwrap_or("");
            let selector = issue.get("selector")?.as_str()?.to_string();

            let (message, severity, fix) = if kind == "onchange" {
                (
                    format!(
                        "{} element has onchange handler — may cause unexpected context change",
                        role
                    ),
                    Severity::Medium,
                    "Use a submit button instead of auto-submitting on selection change",
                )
            } else {
                let name = issue.get("name").and_then(|v| v.as_str()).unwrap_or("");
                (
                    format!(
                        "{} '{}' may trigger navigation on change without explicit submit",
                        role, name
                    ),
                    Severity::Low,
                    "Add a submit button or notify users that selection will change context",
                )
            };

            Some(
                Violation::new(
                    ON_INPUT_RULE.id,
                    ON_INPUT_RULE.name,
                    ON_INPUT_RULE.level,
                    severity,
                    message,
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix(fix)
                .with_rule_id(ON_INPUT_RULE.axe_id)
                .with_help_url(ON_INPUT_RULE.help_url),
            )
        })
        .collect()
}
