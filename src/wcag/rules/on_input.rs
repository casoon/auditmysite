//! WCAG 3.2.2 On Input
//!
//! Changing the setting of any user interface component does not automatically
//! cause a change of context unless the user has been advised of the behavior
//! before using the component.
//! Level A
//!
//! Den Handlertext im Markup beurteilt seit #693 `context/on-input` im
//! geteilten Bestand (siehe `wcag::shared`): Verstoss, wenn er sichtbar
//! navigiert, absendet, ein Fenster oeffnet oder den Fokus verschiebt, sonst
//! `REVIEW`. Hier bleibt, was nur die laufende Seite weiss:
//!
//! - Ruft der Handler eine globale Funktion auf (`onchange="go(this.value)"`),
//!   wird ihr Quelltext ueber `window` nachgeschlagen. Wechselt erst sie den
//!   Kontext, ist das ein Verstoss -- die geteilte Regel sieht nur den
//!   Aufruf und bleibt bei `REVIEW`, beide stehen dann am selben Element.
//! - Eine Auswahl ohne Handler, deren Name nach Navigation klingt
//!   („Language", „Country"), ohne Absende-Button auf der Seite: `REVIEW`.
//!
//! Only a change of *context* counts — navigation, a form submit, a new
//! window or a focus move. Updating content on the same page (a filter, a
//! sort order, a grid's locale) is not one (#657).
//!
//! DOM-level rule: `onchange` is an HTML attribute, never exposed as an AX
//! property — an earlier tree-based implementation of this check read a
//! non-existent AX property and never fired in production (#QA-030).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const ON_INPUT_RULE: RuleMetadata = RuleMetadata {
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

  // The source of a global function the inline handler calls directly
  // (onchange="applyFilter(this.value)"), so the Rust side can see what the
  // handler does. The inline text itself is judged by the shared rule.
  function calledSource(src) {
    var call = /^\s*([A-Za-z_$][\w$]*)\s*\(/.exec(src);
    if (call && typeof window[call[1]] === 'function') {
      try { return Function.prototype.toString.call(window[call[1]]); } catch (e) {}
    }
    return null;
  }

  var buttonNames = [];
  var buttons = document.querySelectorAll('button, input[type="submit"], [role="button"]');
  for (var b = 0; b < buttons.length && buttonNames.length < CAP; b++) {
    buttonNames.push(accessibleName(buttons[b]).toLowerCase());
  }

  var controls = [];
  var nodes = document.querySelectorAll('select, [role="combobox"], [role="listbox"], input[type="radio"], [role="radio"]');
  for (var i = 0; i < nodes.length && __amsReal(controls) < CAP; i++) {
    var el = nodes[i];
    var handler = el.getAttribute('onchange');
    __amsPush(controls, el, {
      role: (el.getAttribute('role') || el.tagName.toLowerCase()),
      name: accessibleName(el).toLowerCase(),
      selector: __amsCssSelector(el),
      handler: handler,
      called: handler !== null && isChangeControl(el) ? calledSource(handler) : null
    }, CAP);
  }

  return { button_names: buttonNames, controls: controls };
"#;

#[derive(Debug, Clone, serde::Deserialize)]
struct Control {
    role: String,
    name: String,
    selector: String,
    /// The inline `onchange` text, if any.
    handler: Option<String>,
    /// The source of the global function the handler calls, if any.
    called: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Scan {
    button_names: Vec<String>,
    controls: Vec<Control>,
}

/// Whole words in a button's accessible name that mark it as an explicit
/// submit. Word-aware so "Google", "category" or "logo" don't count as "go".
/// English and German merged, independent of the output language.
const SUBMIT_WORDS: &[&str] = &[
    "submit", "send", "go", "search", "absenden", "senden", "suchen",
];

/// Whether any button on the page is an explicit submit by its name.
fn has_submit_button(button_names: &[String]) -> bool {
    button_names.iter().any(|name| {
        name.split(|c: char| !c.is_alphanumeric())
            .any(|word| SUBMIT_WORDS.contains(&word))
    })
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
    let submit_button = has_submit_button(&scan.button_names);
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
                // Dieselbe Kennung wie die geteilte Regel (`context/on-input`):
                // Es ist derselbe Befund, nur hier mit Blick in die
                // aufgerufene Funktion. So verdraengt der Verstoss den
                // Pruefhinweis der geteilten Regel am selben Element (#527)
                // statt daneben zu stehen.
                .with_rule_id("context/on-input")
                .with_help_url(ON_INPUT_RULE.help_url)
            };

            // Ein Handler im Markup gehoert der geteilten Regel. Hier nur
            // der Fall, den sie nicht sehen kann: Erst die aufgerufene
            // Funktion wechselt den Kontext.
            if let Some(handler) = c.handler.as_deref() {
                let only_called = !changes_context(handler)
                    && c.called.as_deref().is_some_and(changes_context);
                return only_called.then(|| {
                    finding(
                        Severity::Medium,
                        format!(
                            "{} element's onchange handler calls a function that navigates, submits, opens a window or moves focus",
                            c.role
                        ),
                        "Use a submit button instead of changing context on selection change",
                    )
                });
            }

            if !submit_button && NAVIGATION_HINTS.iter().any(|h| c.name.contains(h)) {
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

    fn control(name: &str, handler: Option<&str>, called: Option<&str>) -> Control {
        Control {
            role: "select".to_string(),
            name: name.to_string(),
            selector: "select#x".to_string(),
            handler: handler.map(str::to_string),
            called: called.map(str::to_string),
        }
    }

    fn scan(controls: Vec<Control>) -> Scan {
        Scan {
            button_names: Vec::new(),
            controls,
        }
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    /// "go" and friends count only as whole words: "Google", "category" and
    /// "logo" are no submit buttons.
    #[test]
    fn submit_words_match_whole_words_only() {
        for name in [
            "go",
            "go!",
            "submit order",
            "send message",
            "search",
            "jetzt absenden",
            "suchen",
        ] {
            assert!(has_submit_button(&names(&[name])), "{name}");
        }
        for name in [
            "google",
            "sign in with google",
            "category",
            "logo",
            "resend-code",
            "research",
            "gold",
        ] {
            assert!(!has_submit_button(&names(&[name])), "{name}");
        }
    }

    /// A "Google" or "logo" button no longer hides a navigation-hinted select.
    #[test]
    fn non_submit_buttons_do_not_suppress_navigation_hint() {
        let mut s = scan(vec![control("language", None, None)]);
        s.button_names = names(&["sign in with google", "category", "logo"]);
        assert_eq!(evaluate(&s).len(), 1);
    }

    /// Erst die aufgerufene Funktion wechselt den Kontext: Das sieht nur die
    /// laufende Seite, die geteilte Regel bliebe bei `REVIEW`.
    #[test]
    fn called_function_that_changes_context_is_violation() {
        let v = evaluate(&scan(vec![control(
            "pages",
            Some("go(this.value)"),
            Some("function go(v) { document.location.assign(v); }"),
        )]));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, Outcome::Fail);
        assert_eq!(v[0].severity, Severity::Medium);
    }

    /// Was der Handler im Markup selbst tut, urteilt `context/on-input`;
    /// hier entsteht dazu kein zweiter Befund.
    #[test]
    fn inline_handler_is_left_to_the_shared_rule() {
        for (handler, called) in [
            ("location.href=this.value", None),
            ("this.form.submit()", None),
            ("grid.setSort(this.value)", None),
            (
                "applyFilter(this.value)",
                Some("function applyFilter(v) { grid.filter(v); }"),
            ),
            (
                "go(this.value); location.href=x",
                Some("function go(v) { location.href = v; }"),
            ),
        ] {
            let v = evaluate(&scan(vec![control("language", Some(handler), called)]));
            assert!(v.is_empty(), "{handler}: {v:?}");
        }
    }

    /// #657: og-vanilla's filter/sort presets update the grid in place.
    #[test]
    fn filter_and_sort_selects_without_handler_are_not_reported() {
        let v = evaluate(&scan(vec![
            control("filter preset", None, None),
            control("sort preset", None, None),
        ]));
        assert!(v.is_empty());
    }

    /// #657: a language select may only update content — review at most.
    #[test]
    fn navigation_hint_without_handler_is_review() {
        let v = evaluate(&scan(vec![control("language", None, None)]));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, Outcome::Review);
    }

    #[test]
    fn navigation_hint_with_submit_button_is_not_reported() {
        let mut s = scan(vec![control("language", None, None)]);
        s.button_names = names(&["go"]);
        assert!(evaluate(&s).is_empty());
    }
}
