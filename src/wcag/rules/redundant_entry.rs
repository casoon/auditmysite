//! WCAG 3.3.7 Redundant Entry (Level A, WCAG 2.2)
//!
//! "Information previously entered by or provided to the user that is
//! required to be entered again in the same process is either auto-populated
//! or available for the user to select."
//!
//! Faithfully testing this criterion requires walking a genuine multi-step
//! process (navigating from step to step and correlating field values across
//! steps), which this project's Accessibility-Journey layer
//! (`src/a11y_journey/`) does not currently do — journeys there test
//! individual interaction patterns (disclosure, modal, tabs, form-error
//! announcement, …) on a single loaded page, not cross-page/cross-step field
//! correlation. Building that is out of scope for this check.
//!
//! Instead this implements a conservative, single-page DOM heuristic that
//! catches a real, common instance of the violation: a **single form** on the
//! page that asks for the same semantic piece of information (email, phone
//! number, name, street address, postal code) more than once — e.g. a
//! shipping address repeated as a billing address — without any
//! `autocomplete` hint, prefilled value, or "same as ..." reuse option.
//!
//! Deliberately excluded to avoid false positives:
//! - Any field whose name/id/label/placeholder suggests it is an
//!   intentional confirmation/repeat field (password confirmation, "repeat
//!   email", OTP/CAPTCHA codes) — re-entry there is a standard, deliberate UX
//!   pattern, not a redundant-entry failure.
//! - Fields that already carry any non-empty, non-`off` `autocomplete`
//!   attribute — browser-native autofill can satisfy the criterion for them.
//! - Fields that are already prefilled (`readonly`, `disabled`, or a
//!   non-empty `value`).
//! - Forms that offer an explicit "same as above" / "use shipping address"
//!   style reuse option.
//! - Duplicate field detection is scoped **per `<form>` element**, not
//!   page-wide, so unrelated widgets (e.g. a newsletter signup and a contact
//!   form both having a "name" field) are not conflated as the same process.
//!
//! Because "same process" cannot be verified statically, findings are
//! reported as [`crate::wcag::types::FindingKind::Warning`] (heuristic,
//! manual-review candidate) rather than a confirmed violation.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const REDUNDANT_ENTRY_RULE: RuleMetadata = RuleMetadata {
    id: "3.3.7",
    name: "Redundant Entry",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "Information already entered earlier in the same form is not requested again without reuse support",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/redundant-entry.html",
    axe_id: "redundant-entry",
    tags: &["wcag22a", "wcag337", "cat.forms"],
};

/// Semantic field categories checked for redundant re-entry, with their
/// label/name/id/placeholder keyword fallbacks (only consulted when a field
/// has no usable `autocomplete` hint at all).
const CATEGORIES: &[(&str, &[&str])] = &[
    ("email address", &["email", "e-mail"]),
    (
        "phone number",
        &["telefon", "phone", "mobil", "handynummer"],
    ),
    (
        "name",
        &[
            "vorname",
            "nachname",
            "first name",
            "last name",
            "firstname",
            "lastname",
            "full name",
            "surname",
            "family name",
        ],
    ),
    (
        "street address",
        &["straße", "strasse", "address line", "street address"],
    ),
    (
        "postal code",
        &["postleitzahl", "plz", "zip code", "zipcode", "postal code"],
    ),
];

/// Keywords marking a field as an intentional confirmation/repeat/one-time
/// entry (password confirmation, "repeat email", OTP/CAPTCHA) — excluded from
/// redundant-entry detection since re-entry there is a deliberate, standard
/// pattern rather than an accessibility failure.
const EXCLUDE_KEYWORDS: &[&str] = &[
    "confirm",
    "repeat",
    "again",
    "verify",
    "wiederhol",
    "bestät",
    "bestaet",
    "erneut",
    "captcha",
    "otp",
    "one-time code",
    "one time code",
    "verification code",
];

/// One form field, as extracted from the live DOM.
#[derive(Debug, Clone)]
struct FieldData {
    /// Combined, lowercase-insensitive text from name/id/placeholder/
    /// aria-label/associated label.
    text: String,
    /// Whether the field already carries a usable (non-empty, non-`off`,
    /// non-`on`) `autocomplete` attribute.
    has_autocomplete: bool,
    /// Whether the field is already prefilled (`readonly`, `disabled`, or a
    /// non-empty `value`).
    prefilled: bool,
    /// Selector for reporting.
    selector: String,
}

/// One `<form>` element, as extracted from the live DOM.
#[derive(Debug, Clone)]
struct FormData {
    selector: String,
    /// Whether the form's text mentions an explicit reuse option (e.g.
    /// "same as shipping address").
    has_reuse_option: bool,
    fields: Vec<FieldData>,
}

fn classify(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    if EXCLUDE_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
        return None;
    }
    CATEGORIES
        .iter()
        .find(|(_, keywords)| keywords.iter().any(|kw| lower.contains(kw)))
        .map(|(category, _)| *category)
}

/// Categorize one field as a redundant-entry candidate, or `None` if it is
/// excluded (already has an autocomplete hint, already prefilled, matches no
/// known category, or matches a confirmation/repeat pattern).
fn candidate_category(field: &FieldData) -> Option<&'static str> {
    if field.prefilled || field.has_autocomplete {
        return None;
    }
    classify(&field.text)
}

/// Group a form's fields by candidate category and return only the
/// categories that occur at least twice (the actual redundant-entry
/// findings), unless the form already offers a reuse option.
fn group_and_flag(form: &FormData) -> Vec<(&'static str, Vec<String>)> {
    if form.has_reuse_option {
        return Vec::new();
    }
    let mut groups: std::collections::BTreeMap<&'static str, Vec<String>> =
        std::collections::BTreeMap::new();
    for field in &form.fields {
        if let Some(category) = candidate_category(field) {
            groups
                .entry(category)
                .or_default()
                .push(field.selector.clone());
        }
    }
    groups
        .into_iter()
        .filter(|(_, sels)| sels.len() >= 2)
        .collect()
}

const REDUNDANT_ENTRY_JS: &str = r#"
(function() {
  var REUSE_PATTERN = /same as|identical to|use (the )?(shipping|billing)|wie oben|wie bei|entspricht|gleich wie/i;

  function fieldText(el) {
    var parts = [];
    parts.push(el.getAttribute('name') || '');
    parts.push(el.id || '');
    parts.push(el.getAttribute('placeholder') || '');
    parts.push(el.getAttribute('aria-label') || '');
    if (el.id) {
      var esc = el.id.replace(/"/g, '\\"');
      var lbl = document.querySelector('label[for="' + esc + '"]');
      if (lbl) parts.push(lbl.textContent || '');
    }
    var wrap = el.closest('label');
    if (wrap) parts.push(wrap.textContent || '');
    return parts.join(' ');
  }

  function isPrefilled(el) {
    if (el.hasAttribute('readonly') || el.disabled) return true;
    return (el.value || '').trim().length > 0;
  }

  function hasAutocompleteHint(el) {
    var v = (el.getAttribute('autocomplete') || '').toLowerCase().trim();
    return v !== '' && v !== 'off' && v !== 'on';
  }

  function fieldSelector(el) {
    var name = el.getAttribute('name');
    if (name) return el.tagName.toLowerCase() + '[name="' + name + '"]';
    if (el.id) return el.tagName.toLowerCase() + '#' + el.id;
    return el.tagName.toLowerCase();
  }

  var forms = document.querySelectorAll('form');
  var result = [];

  for (var f = 0; f < forms.length; f++) {
    var form = forms[f];
    var fields = form.querySelectorAll(
      'input:not([type="hidden"]):not([type="password"]):not([type="submit"]):not([type="button"]):not([type="reset"]):not([type="checkbox"]):not([type="radio"]), textarea, select'
    );
    var fieldData = [];
    for (var i = 0; i < fields.length && fieldData.length < 100; i++) {
      var el = fields[i];
      fieldData.push({
        text: fieldText(el),
        has_autocomplete: hasAutocompleteHint(el),
        prefilled: isPrefilled(el),
        selector: fieldSelector(el)
      });
    }

    var name = form.getAttribute('name');
    var formSelector = form.id
      ? 'form#' + form.id
      : (name ? 'form[name="' + name + '"]' : 'form:nth-of-type(' + (f + 1) + ')');

    result.push({
      selector: formSelector,
      has_reuse_option: REUSE_PATTERN.test(form.textContent || ''),
      fields: fieldData
    });
  }

  return { forms: result };
})()
"#;

pub async fn check_redundant_entry_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(REDUNDANT_ENTRY_JS).await {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![],
    };

    let forms_json = match val.get("forms").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return vec![],
    };

    let forms: Vec<FormData> = forms_json
        .iter()
        .filter_map(|f| {
            let selector = f.get("selector").and_then(|v| v.as_str())?.to_string();
            let has_reuse_option = f
                .get("has_reuse_option")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let fields = f
                .get("fields")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|field| {
                            Some(FieldData {
                                text: field.get("text").and_then(|v| v.as_str())?.to_string(),
                                has_autocomplete: field
                                    .get("has_autocomplete")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                prefilled: field
                                    .get("prefilled")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                selector: field
                                    .get("selector")
                                    .and_then(|v| v.as_str())?
                                    .to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(FormData {
                selector,
                has_reuse_option,
                fields,
            })
        })
        .collect();

    let mut violations = Vec::new();
    for form in &forms {
        for (category, selectors) in group_and_flag(form) {
            let count = selectors.len();
            let sample = selectors
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            violations.push(
                Violation::new(
                    REDUNDANT_ENTRY_RULE.id,
                    REDUNDANT_ENTRY_RULE.name,
                    REDUNDANT_ENTRY_RULE.level,
                    Severity::Medium,
                    format!(
                        "Form '{}' asks for {} separate {} fields ({}) without offering to reuse or autofill a previously entered value.",
                        form.selector, count, category, sample
                    ),
                    &form.selector,
                )
                .with_selector(&form.selector)
                .with_fix(
                    "Add an appropriate `autocomplete` attribute (e.g. autocomplete=\"email\"), \
                     prefill the field with the previously entered value, or offer a \"same as \
                     above\" option so users are not forced to retype information they already \
                     provided.",
                )
                .with_rule_id(REDUNDANT_ENTRY_RULE.axe_id)
                .with_help_url(REDUNDANT_ENTRY_RULE.help_url)
                .as_warning(),
            );
            if violations.len() >= 5 {
                return violations;
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(text: &str, has_autocomplete: bool, prefilled: bool) -> FieldData {
        FieldData {
            text: text.to_string(),
            has_autocomplete,
            prefilled,
            selector: text.to_string(),
        }
    }

    fn form(has_reuse_option: bool, fields: Vec<FieldData>) -> FormData {
        FormData {
            selector: "form#checkout".to_string(),
            has_reuse_option,
            fields,
        }
    }

    #[test]
    fn fires_when_same_field_type_requested_twice_without_reuse_support() {
        let f = form(
            false,
            vec![
                field("Shipping street address", false, false),
                field("Billing street address", false, false),
            ],
        );
        let flagged = group_and_flag(&f);
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].0, "street address");
        assert_eq!(flagged[0].1.len(), 2);
    }

    #[test]
    fn does_not_fire_for_password_confirmation_pattern() {
        let f = form(
            false,
            vec![
                field("email", false, false),
                field("confirm email", false, false),
            ],
        );
        assert!(group_and_flag(&f).is_empty());
    }

    #[test]
    fn does_not_fire_when_autocomplete_hint_present() {
        let f = form(
            false,
            vec![
                field("Shipping street address", true, false),
                field("Billing street address", true, false),
            ],
        );
        assert!(group_and_flag(&f).is_empty());
    }

    #[test]
    fn does_not_fire_when_reuse_option_present() {
        let f = form(
            true,
            vec![
                field("Shipping street address", false, false),
                field("Billing street address", false, false),
            ],
        );
        assert!(group_and_flag(&f).is_empty());
    }

    #[test]
    fn does_not_fire_when_only_one_candidate_remains_after_prefill() {
        let f = form(
            false,
            vec![
                field("Shipping email", false, false),
                field("Billing email", false, true),
            ],
        );
        assert!(group_and_flag(&f).is_empty());
    }

    #[test]
    fn does_not_fire_for_unrelated_single_occurrence_fields() {
        let f = form(
            false,
            vec![field("email", false, false), field("phone", false, false)],
        );
        assert!(group_and_flag(&f).is_empty());
    }
}
