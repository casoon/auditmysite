//! WCAG 3.2.1 On Focus
//!
//! When any user interface component receives focus, it does not initiate
//! a change of context.
//! Level A
//!
//! Note: Full on-focus testing requires behavioral analysis via CDP.
//! This rule checks for common DOM patterns that may indicate
//! focus-triggered context changes.
//!
//! DOM-level rule: `onfocus`/`autofocus` are HTML attributes, never exposed
//! as AX properties — an earlier tree-based implementation of this check
//! read non-existent AX properties and never fired in production (#QA-030).

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const ON_FOCUS_RULE: RuleMetadata = RuleMetadata {
    id: "3.2.1",
    name: "On Focus",
    level: WcagLevel::A,
    severity: Severity::High,
    description: "Receiving focus does not initiate a change of context",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/on-focus.html",
    axe_id: "focus-no-context-change",
    tags: &["wcag2a", "wcag321", "cat.keyboard"],
};

const ON_FOCUS_CAP: usize = 250;

const ON_FOCUS_BODY: &str = r#"
  var interactiveTags = ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA', 'SUMMARY'];
  var interactiveRoles = [
    'button', 'link', 'checkbox', 'radio', 'switch', 'menuitem', 'menuitemcheckbox',
    'menuitemradio', 'tab', 'textbox', 'combobox', 'searchbox', 'slider', 'spinbutton'
  ];
  var textboxLikeTags = ['INPUT', 'TEXTAREA', 'SELECT'];
  var textboxLikeRoles = ['textbox', 'searchbox', 'combobox'];

  function isInteractive(el) {
    if (interactiveTags.indexOf(el.tagName) !== -1) return true;
    var role = (el.getAttribute('role') || '').toLowerCase();
    return interactiveRoles.indexOf(role) !== -1;
  }

  function isTextboxLike(el) {
    if (textboxLikeTags.indexOf(el.tagName) !== -1) return true;
    var role = (el.getAttribute('role') || '').toLowerCase();
    return textboxLikeRoles.indexOf(role) !== -1;
  }

  var issues = [];

  var focusHandlerElems = document.querySelectorAll('[onfocus]');
  for (var i = 0; i < focusHandlerElems.length && issues.length < CAP; i++) {
    var el = focusHandlerElems[i];
    if (!isInteractive(el)) {
      issues.push({
        kind: 'onfocus',
        role: (el.getAttribute('role') || el.tagName.toLowerCase()),
        selector: __amsCssSelector(el)
      });
    }
  }

  var autofocusElems = document.querySelectorAll('[autofocus]');
  for (var j = 0; j < autofocusElems.length && issues.length < CAP; j++) {
    var ael = autofocusElems[j];
    if (!isTextboxLike(ael)) {
      issues.push({
        kind: 'autofocus',
        role: (ael.getAttribute('role') || ael.tagName.toLowerCase()),
        selector: __amsCssSelector(ael)
      });
    }
  }

  return { issues: issues };
"#;

pub async fn check_on_focus_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &ON_FOCUS_BODY.replace("CAP", &ON_FOCUS_CAP.to_string()),
        "})()",
    ]
    .concat();

    let result = match page.evaluate(js.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            warn!("on-focus JS failed: {}", e);
            return vec![];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
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

            let (message, severity, fix) = if kind == "onfocus" {
                (
                    format!(
                        "Non-interactive {} element has onfocus handler which may cause context change",
                        role
                    ),
                    Severity::High,
                    "Remove onfocus handlers that cause context changes, or use interactive elements",
                )
            } else {
                (
                    format!(
                        "{} element has autofocus which may cause unexpected context change",
                        role
                    ),
                    Severity::Medium,
                    "Avoid autofocus on non-input elements as it can disorient users",
                )
            };

            Some(
                Violation::new(
                    ON_FOCUS_RULE.id,
                    ON_FOCUS_RULE.name,
                    ON_FOCUS_RULE.level,
                    severity,
                    message,
                    selector.clone(),
                )
                .with_selector(selector)
                .with_fix(fix)
                .with_rule_id(ON_FOCUS_RULE.axe_id)
                .with_help_url(ON_FOCUS_RULE.help_url),
            )
        })
        .collect()
}
