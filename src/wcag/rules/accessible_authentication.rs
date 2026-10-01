//! WCAG 3.3.8 Accessible Authentication (Minimum) (Level AA, WCAG 2.2)
//!
//! "A cognitive function test (such as remembering a password or solving a
//! puzzle) is not required for any step in an authentication process unless
//! that step provides at least one of the following: Alternative, Mechanism,
//! Object Recognition, Personal Content."
//!
//! **Paste blocked on a password or one-time-code field** — a confirmed
//! violation (failure technique F109). Blocking paste defeats the
//! "Mechanism" exception: password managers and copy/paste are exactly what
//! lets a user skip transcribing the secret. Measured, not inferred: a
//! synthetic, cancelable `paste` event carrying a probe string is
//! dispatched on the field. The field counts as blocked only if the event
//! was cancelled **and** the probe did not end up in the value — a handler
//! that cancels the native paste and inserts the (sanitised) text itself is
//! not blocking anything. The field's value is restored afterwards, and
//! `alert`/`confirm`/`prompt` are stubbed for the duration so a "no pasting"
//! dialog can't stall the page.
//!
//! Das Captcha im Anmeldeformular laeuft seit #693 als `auth/captcha` im
//! geteilten Bestand (siehe `wcag::shared`); der Einfuege-Test braucht die
//! laufende Seite und bleibt hier.
//!
//! Not flagged: `autocomplete="off"` or a missing `autocomplete` on password
//! fields. Browsers ignore `off` on login fields and password managers fill
//! them regardless, so neither is a barrier on its own (see plan 45 on
//! criteria resting on signals without substance).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub(super) const ACCESSIBLE_AUTH_PASTE_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.8",
    name: "Accessible Authentication (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Password and one-time-code fields allow pasting, so credentials don't have to be transcribed",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/accessible-authentication-minimum.html",
    axe_id: "accessible-auth-paste-blocked",
    tags: &["wcag22aa", "wcag338", "cat.forms"],
};

/// Cap on reported findings, matching the other form rules.
const MAX_FINDINGS: usize = 5;

const ACCESSIBLE_AUTH_JS: &str = r#"
(function() {
  var PROBE = 'amsPasteProbe7!';
  var AUTH_FIELD = 'input[type="password"], input[autocomplete~="one-time-code"], input[autocomplete~="current-password"]';

  function isVisible(el) {
    var r = el.getBoundingClientRect();
    if (r.width === 0 && r.height === 0) return false;
    var s = getComputedStyle(el);
    return s.display !== 'none' && s.visibility !== 'hidden';
  }

  /*CSS_SELECTOR*/

  function kindOf(el) {
    var ac = (el.getAttribute('autocomplete') || '').toLowerCase();
    return ac.indexOf('one-time-code') !== -1 ? 'one-time code' : 'password';
  }

  function pasteBlocked(el) {
    var original = el.value;
    var dt;
    try {
      dt = new DataTransfer();
      dt.setData('text/plain', PROBE);
    } catch (e) {
      dt = null;
    }
    var ev;
    try {
      ev = new ClipboardEvent('paste', { bubbles: true, cancelable: true, clipboardData: dt });
    } catch (e) {
      ev = new Event('paste', { bubbles: true, cancelable: true });
    }
    var notCancelled = el.dispatchEvent(ev);
    var inserted = (el.value || '').indexOf(PROBE) !== -1;
    el.value = original;
    return !notCancelled && !inserted;
  }

  var savedAlert = window.alert, savedConfirm = window.confirm, savedPrompt = window.prompt;
  window.alert = function() {};
  window.confirm = function() { return false; };
  window.prompt = function() { return null; };

  var blocked = [];
  try {
    var fields = document.querySelectorAll(AUTH_FIELD);
    // Probe and report caps count real and excluded (#645) fields separately.
    var probed = [];
    for (var i = 0; i < fields.length && __amsReal(probed) < 20; i++) {
      var f = fields[i];
      if (f.disabled || f.readOnly || !isVisible(f)) continue;
      if (!__amsPush(probed, f, f, 20)) continue;
      if (pasteBlocked(f)) __amsPush(blocked, f, { selector: __amsCssSelector(f), kind: kindOf(f) }, /*MAX_FINDINGS*/);
    }
  } finally {
    window.alert = savedAlert;
    window.confirm = savedConfirm;
    window.prompt = savedPrompt;
  }

  return { blocked: blocked };
})()
"#;

pub async fn check_accessible_authentication_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &ACCESSIBLE_AUTH_PASTE_RULE,
        &ACCESSIBLE_AUTH_JS
            .replace(
                "/*CSS_SELECTOR*/",
                crate::accessibility::js_helpers::CSS_SELECTOR_JS,
            )
            .replace("/*MAX_FINDINGS*/", &MAX_FINDINGS.to_string()),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let Some(blocked) = val.get("blocked").and_then(|v| v.as_array()) else {
        return vec![crate::wcag::technical_rule_failure(
            &ACCESSIBLE_AUTH_PASTE_RULE,
            "invalid_evaluation_shape",
        )];
    };

    let mut violations = Vec::new();

    for field in blocked {
        let (Some(selector), Some(kind)) = (
            field.get("selector").and_then(|v| v.as_str()),
            field.get("kind").and_then(|v| v.as_str()),
        ) else {
            continue;
        };
        violations.push(
            Violation::new(
                ACCESSIBLE_AUTH_PASTE_RULE.id,
                ACCESSIBLE_AUTH_PASTE_RULE.name,
                ACCESSIBLE_AUTH_PASTE_RULE.level,
                ACCESSIBLE_AUTH_PASTE_RULE.severity,
                format!(
                    "The {kind} field '{selector}' blocks pasting, so the value must be typed from memory or transcribed instead of inserted by a password manager or the clipboard."
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix(
                "Remove the paste handler (or the `preventDefault()` / `return false` in it) \
                 from password and one-time-code fields so users can paste credentials or let a \
                 password manager fill them.",
            )
            .with_rule_id(ACCESSIBLE_AUTH_PASTE_RULE.axe_id)
            .with_help_url(ACCESSIBLE_AUTH_PASTE_RULE.help_url),
        );
    }

    violations
}
