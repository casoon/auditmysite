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
//! Only a change of *context* counts — navigation, a form submit, a new
//! window or a focus move. Updating content on the same page (a filter, a
//! sort order, a grid's locale) is not one (#657). A violation is therefore
//! only reported when the inline change handler visibly does one of those
//! things; a handler whose effect can't be read, and a name that merely
//! suggests navigation, become a review warning.
//!
//! DOM-level rule: `onchange` is an HTML attribute, never exposed as an AX
//! property — an earlier tree-based implementation of this check read a
//! non-existent AX property and never fired in production (#QA-030).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const ON_INPUT_RULE: RuleMetadata = RuleMetadata {
    id: "3.2.2",
    name: "On Input",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Changing a setting does not automatically cause a change of context",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/on-input.html",
    axe_id: "input-no-context-change",
    tags: &["wcag2a", "wcag322", "cat.keyboard"],
};

const ON_INPUT_CAP: usize = 250;

/// Code fragments (whitespace removed) by which a change handler changes the
/// context: navigation, form submission, a new window, a focus move.
const CONTEXT_CHANGE_PATTERNS: &[&str] = &[
    "location=",
    "location.href=",
    "location.assign(",
    "location.replace(",
    "location.reload(",
    "location.search=",
    "location.pathname=",
    ".submit(",
    ".requestSubmit(",
    "window.open(",
    ".focus(",
    "navigate(",
];

/// Accessible-name words suggesting that a control navigates. "sort" and
/// "filter" are not here: they name controls that update content in place,
/// which is no change of context (#657).
const NAVIGATION_HINTS: &[&str] = &[
    "language", "country", "region", "navigate", "redirect", "go to",
];

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

  // Inline handler source, plus the source of a global function it calls
  // directly (onchange="applyFilter(this.value)"), so the Rust side can see
  // what the handler does.
  function handlerSource(el) {
    var src = el.getAttribute('onchange');
    if (src === null) return null;
    var call = /^\s*([A-Za-z_$][\w$]*)\s*\(/.exec(src);
    if (call && typeof window[call[1]] === 'function') {
      try { src += '\n' + Function.prototype.toString.call(window[call[1]]); } catch (e) {}
    }
    return src;
  }

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

  var controls = [];
  var nodes = document.querySelectorAll('select, [role="combobox"], [role="listbox"], input[type="radio"], [role="radio"]');
  for (var i = 0; i < nodes.length && controls.length < CAP; i++) {
    var el = nodes[i];
    var changeControl = isChangeControl(el);
    controls.push({
      role: (el.getAttribute('role') || el.tagName.toLowerCase()),
      name: accessibleName(el).toLowerCase(),
      selector: __amsCssSelector(el),
      change_control: changeControl,
      handler: changeControl ? handlerSource(el) : null
    });
  }

  return { has_submit_button: hasSubmitButton, controls: controls };
"#;

#[derive(Debug, Clone, serde::Deserialize)]
struct Control {
    role: String,
    name: String,
    selector: String,
    change_control: bool,
    handler: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Scan {
    has_submit_button: bool,
    controls: Vec<Control>,
}

pub async fn check_on_input_with_page(page: &Page) -> Vec<Violation> {
    let js = [
        "(function() {",
        crate::accessibility::js_helpers::CSS_SELECTOR_JS,
        &ON_INPUT_BODY.replace("CAP", &ON_INPUT_CAP.to_string()),
        "})()",
    ]
    .concat();

    let val = match crate::wcag::types::evaluate_or_fail_for(
        page,
        "on-input",
        crate::cli::WcagLevel::A,
        js.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    match serde_json::from_value::<Scan>(val) {
        Ok(scan) => evaluate(&scan),
        Err(_) => vec![],
    }
}

/// Whether handler source visibly changes the context.
fn changes_context(handler: &str) -> bool {
    let code: String = handler.chars().filter(|c| !c.is_whitespace()).collect();
    CONTEXT_CHANGE_PATTERNS.iter().any(|p| code.contains(p))
}

fn evaluate(scan: &Scan) -> Vec<Violation> {
    scan.controls
        .iter()
        .filter_map(|c| {
            let finding = |severity: Severity, message: String, fix: &str| {
                Violation::new(
                    ON_INPUT_RULE.id,
                    ON_INPUT_RULE.name,
                    ON_INPUT_RULE.level,
                    severity,
                    message,
                    c.selector.clone(),
                )
                .with_selector(c.selector.clone())
                .with_fix(fix)
                .with_rule_id(ON_INPUT_RULE.axe_id)
                .with_help_url(ON_INPUT_RULE.help_url)
            };

            if let Some(handler) = c.handler.as_deref().filter(|_| c.change_control) {
                return Some(if changes_context(handler) {
                    finding(
                        Severity::Medium,
                        format!(
                            "{} element's onchange handler navigates, submits, opens a window or moves focus",
                            c.role
                        ),
                        "Use a submit button instead of changing context on selection change",
                    )
                } else {
                    finding(
                        Severity::Low,
                        format!(
                            "{} element has an onchange handler — verify that it only updates content and does not change context",
                            c.role
                        ),
                        "If the handler navigates, submits or moves focus, add a submit button or tell users beforehand",
                    )
                    .as_warning()
                });
            }

            if !scan.has_submit_button && NAVIGATION_HINTS.iter().any(|h| c.name.contains(h)) {
                return Some(
                    finding(
                        Severity::Low,
                        format!(
                            "{} '{}' may navigate on change without explicit submit — verify",
                            c.role, c.name
                        ),
                        "If changing the selection navigates, add a submit button or notify users beforehand",
                    )
                    .as_warning(),
                );
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::types::Outcome;

    fn control(name: &str, handler: Option<&str>) -> Control {
        Control {
            role: "select".to_string(),
            name: name.to_string(),
            selector: "select#x".to_string(),
            change_control: true,
            handler: handler.map(str::to_string),
        }
    }

    fn scan(controls: Vec<Control>) -> Scan {
        Scan {
            has_submit_button: false,
            controls,
        }
    }

    #[test]
    fn navigating_handler_is_violation() {
        for handler in [
            "location.href=this.value",
            "window.location = this.value",
            "this.form.submit()",
            "window.open(this.value)",
            "go(this.value)\nfunction go(v) { document.location.assign(v); }",
        ] {
            let v = evaluate(&scan(vec![control("pages", Some(handler))]));
            assert_eq!(v.len(), 1, "{handler}");
            assert_eq!(v[0].kind, Outcome::Fail, "{handler}");
            assert_eq!(v[0].severity, Severity::Medium);
        }
    }

    /// A handler that only updates content is no context change, but its
    /// effect can't be proven — review, not violation.
    #[test]
    fn content_updating_handler_is_review() {
        let v = evaluate(&scan(vec![control(
            "sort",
            Some("grid.setSort(this.value)"),
        )]));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, Outcome::Review);
    }

    /// #657: og-vanilla's filter/sort presets update the grid in place.
    #[test]
    fn filter_and_sort_selects_without_handler_are_not_reported() {
        let v = evaluate(&scan(vec![
            control("filter preset", None),
            control("sort preset", None),
        ]));
        assert!(v.is_empty());
    }

    /// #657: a language select may only update content — review at most.
    #[test]
    fn navigation_hint_without_handler_is_review() {
        let v = evaluate(&scan(vec![control("language", None)]));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, Outcome::Review);
    }

    #[test]
    fn navigation_hint_with_submit_button_is_not_reported() {
        let mut s = scan(vec![control("language", None)]);
        s.has_submit_button = true;
        assert!(evaluate(&s).is_empty());
    }
}
