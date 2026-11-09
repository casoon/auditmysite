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
//!   („Language", „Country"), ohne Absende-Button auf der Seite und ohne
//!   Absende-Element in ihrem eigenen Formular: `REVIEW`. Ein Formular mit
//!   Absende-Element sendet erst auf ausdrueckliche Anforderung ab, gleich
//!   wie der Knopf heisst („Continue", #783).
//!
//! Only a change of *context* counts — navigation, a form submit, a new
//! window or a focus move. Updating content on the same page (a filter, a
//! sort order, a grid's locale) is not one (#657).
//!
//! DOM-level rule: `onchange` is an HTML attribute, never exposed as an AX
//! property — an earlier tree-based implementation of this check read a
//! non-existent AX property and never fired in production (#QA-030).

use chromiumoxide::Page;

use crate::wcag::types::{RuleMetadata, Severity, Violation};
use crate::wcag::WcagLevel;

pub(super) const ON_INPUT_RULE: RuleMetadata = RuleMetadata {
    id: "3.2.2",
    name: "On Input",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Changing a setting does not automatically cause a change of context",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/on-input.html",
    // Every finding carries the shared id; the former own id
    // `input-no-context-change` was never emitted (#783).
    axe_id: "context/on-input",
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

const ON_INPUT_BODY: &str = include_str!("js/on_input_body.js");

#[derive(Debug, Clone, serde::Deserialize)]
struct Control {
    role: String,
    name: String,
    selector: String,
    /// The inline `onchange` text, if any.
    handler: Option<String>,
    /// The source of the global function the handler calls, if any.
    called: Option<String>,
    /// Whether the control's own form has a submit control.
    #[serde(default)]
    form_submit: bool,
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
        crate::wcag::WcagLevel::A,
        js.as_str(),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    // A scan of the wrong shape is no answer, not a page without on-input
    // handlers (#777).
    match serde_json::from_value::<Scan>(val) {
        Ok(scan) => evaluate(&scan),
        Err(_) => vec![crate::wcag::technical_rule_failure_for(
            "on-input",
            crate::wcag::WcagLevel::A,
            "invalid_evaluation_value",
        )],
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

            if !submit_button && !c.form_submit && NAVIGATION_HINTS.iter().any(|h| c.name.contains(h)) {
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

    /// The bare word `CAP` in `js/on_input_body.js` is the cap placeholder.
    #[test]
    fn body_script_gets_the_cap_in_all_three_places() {
        let js = ON_INPUT_BODY.replace("CAP", &ON_INPUT_CAP.to_string());
        assert_eq!(ON_INPUT_BODY.matches("CAP").count(), 3);
        assert!(!js.contains("CAP"));
        assert!(js.contains(&format!("buttonNames.length < {ON_INPUT_CAP}")));
        assert!(js.contains(&format!("__amsReal(controls) < {ON_INPUT_CAP}")));
        assert!(js.contains(&format!("}}, {ON_INPUT_CAP});")));
    }

    fn control(name: &str, handler: Option<&str>, called: Option<&str>) -> Control {
        Control {
            role: "select".to_string(),
            name: name.to_string(),
            selector: "select#x".to_string(),
            handler: handler.map(str::to_string),
            called: called.map(str::to_string),
            form_submit: false,
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

    /// A select in a form with its own submit control waits for that control,
    /// whatever it is called ("Continue" on a checkout form, #783).
    #[test]
    fn navigation_hint_in_form_with_submit_control_is_not_reported() {
        let mut c = control("country", None, None);
        c.form_submit = true;
        assert!(evaluate(&scan(vec![c])).is_empty());
    }

    #[test]
    fn navigation_hint_with_submit_button_is_not_reported() {
        let mut s = scan(vec![control("language", None, None)]);
        s.button_names = names(&["go"]);
        assert!(evaluate(&s).is_empty());
    }
}
