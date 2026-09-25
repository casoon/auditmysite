//! WCAG 3.3.8 Accessible Authentication (Minimum) (Level AA, WCAG 2.2)
//!
//! "A cognitive function test (such as remembering a password or solving a
//! puzzle) is not required for any step in an authentication process unless
//! that step provides at least one of the following: Alternative, Mechanism,
//! Object Recognition, Personal Content."
//!
//! Two signals, deliberately split by how certain they are (plan 54 §3):
//!
//! - **Paste blocked on a password or one-time-code field** — a confirmed
//!   violation (failure technique F109). Blocking paste defeats the
//!   "Mechanism" exception: password managers and copy/paste are exactly what
//!   lets a user skip transcribing the secret. Measured, not inferred: a
//!   synthetic, cancelable `paste` event carrying a probe string is
//!   dispatched on the field. The field counts as blocked only if the event
//!   was cancelled **and** the probe did not end up in the value — a handler
//!   that cancels the native paste and inserts the (sanitised) text itself is
//!   not blocking anything. The field's value is restored afterwards, and
//!   `alert`/`confirm`/`prompt` are stubbed for the duration so a "no pasting"
//!   dialog can't stall the page.
//! - **An interactive CAPTCHA inside an authentication form** — review only.
//!   Whether an alternative exists, or whether the CAPTCHA is an
//!   object-recognition test (allowed at AA), is not decidable from the DOM.
//!   Only forms that contain a password or one-time-code field count as
//!   authentication; a CAPTCHA on a contact form is outside this criterion.
//!   Invisible variants (reCAPTCHA v3 / `data-size="invisible"`) present no
//!   test at all and are skipped.
//!
//! Not flagged: `autocomplete="off"` or a missing `autocomplete` on password
//! fields. Browsers ignore `off` on login fields and password managers fill
//! them regardless, so neither is a barrier on its own (see plan 45 on
//! criteria resting on signals without substance).

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const ACCESSIBLE_AUTH_PASTE_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.8",
    name: "Accessible Authentication (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::High,
    description: "Password and one-time-code fields allow pasting, so credentials don't have to be transcribed",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/accessible-authentication-minimum.html",
    axe_id: "accessible-auth-paste-blocked",
    tags: &["wcag22aa", "wcag338", "cat.forms"],
};

pub const ACCESSIBLE_AUTH_CAPTCHA_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.8",
    name: "Accessible Authentication (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "A CAPTCHA in an authentication form offers an alternative that is not a cognitive function test",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/accessible-authentication-minimum.html",
    axe_id: "accessible-auth-captcha",
    tags: &["wcag22aa", "wcag338", "cat.forms"],
};

/// Cap on reported findings per signal, matching the other form rules.
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

  function selectorFor(el) {
    var tag = el.tagName.toLowerCase();
    if (el.id) return tag + '#' + el.id;
    var name = el.getAttribute('name');
    if (name) return tag + '[name="' + name + '"]';
    var type = el.getAttribute('type');
    return type ? tag + '[type="' + type + '"]' : tag;
  }

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
    for (var i = 0; i < fields.length && i < 20; i++) {
      var f = fields[i];
      if (f.disabled || f.readOnly || !isVisible(f)) continue;
      if (pasteBlocked(f)) blocked.push({ selector: selectorFor(f), kind: kindOf(f) });
    }
  } finally {
    window.alert = savedAlert;
    window.confirm = savedConfirm;
    window.prompt = savedPrompt;
  }

  // Interactive CAPTCHA widgets inside authentication forms.
  var CAPTCHA = [
    '.g-recaptcha:not([data-size="invisible"])',
    'iframe[src*="recaptcha/api2/anchor"]:not([src*="size=invisible"])',
    'iframe[src*="recaptcha/enterprise/anchor"]:not([src*="size=invisible"])',
    '.h-captcha:not([data-size="invisible"])',
    'iframe[src*="hcaptcha.com"][src*="checkbox"]',
    'img[src*="captcha" i]', 'img[alt*="captcha" i]', 'img[id*="captcha" i]', 'img[class*="captcha" i]'
  ].join(', ');

  var captchas = [];
  var forms = document.querySelectorAll('form');
  for (var j = 0; j < forms.length; j++) {
    var form = forms[j];
    if (!form.querySelector(AUTH_FIELD)) continue;
    var widget = form.querySelector(CAPTCHA);
    if (!widget || !isVisible(widget)) continue;
    var formName = form.getAttribute('name');
    captchas.push({
      form: form.id ? 'form#' + form.id : (formName ? 'form[name="' + formName + '"]' : 'form'),
      widget: selectorFor(widget)
    });
  }

  return { blocked: blocked, captchas: captchas };
})()
"#;

pub async fn check_accessible_authentication_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &ACCESSIBLE_AUTH_PASTE_RULE,
        ACCESSIBLE_AUTH_JS,
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let (Some(blocked), Some(captchas)) = (
        val.get("blocked").and_then(|v| v.as_array()),
        val.get("captchas").and_then(|v| v.as_array()),
    ) else {
        return vec![crate::wcag::technical_rule_failure(
            &ACCESSIBLE_AUTH_PASTE_RULE,
            "invalid_evaluation_shape",
        )];
    };

    let mut violations = Vec::new();

    for field in blocked.iter().take(MAX_FINDINGS) {
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

    for captcha in captchas.iter().take(MAX_FINDINGS) {
        let (Some(form), Some(widget)) = (
            captcha.get("form").and_then(|v| v.as_str()),
            captcha.get("widget").and_then(|v| v.as_str()),
        ) else {
            continue;
        };
        violations.push(
            Violation::new(
                ACCESSIBLE_AUTH_CAPTCHA_RULE.id,
                ACCESSIBLE_AUTH_CAPTCHA_RULE.name,
                ACCESSIBLE_AUTH_CAPTCHA_RULE.level,
                ACCESSIBLE_AUTH_CAPTCHA_RULE.severity,
                format!(
                    "The authentication form '{form}' contains a CAPTCHA ('{widget}'). Verify that it is an object-recognition test or that an alternative without a cognitive function test is offered."
                ),
                form,
            )
            .with_selector(form)
            .with_fix(
                "Offer a way to authenticate without solving a text or puzzle CAPTCHA — e.g. an \
                 object-recognition challenge, a passkey or e-mail link, or a non-interactive \
                 bot check.",
            )
            .with_rule_id(ACCESSIBLE_AUTH_CAPTCHA_RULE.axe_id)
            .with_help_url(ACCESSIBLE_AUTH_CAPTCHA_RULE.help_url)
            .as_warning(),
        );
    }

    violations
}
