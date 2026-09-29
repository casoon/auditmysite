//! WCAG 1.3.5 Identify Input Purpose
//!
//! The purpose of each input field collecting information about the user
//! can be programmatically determined when the input field serves a purpose
//! identified in the Input Purposes for User Interface Components section.
//! Level AA
//!
//! Reads the HTML `autocomplete` attribute from the DOM. It used to read the
//! AX-tree property `autocomplete` — but that is `aria-autocomplete`
//! (`list`/`inline`/`both`), and Chrome does not expose the HTML attribute in
//! the tree at all. Every correctly marked-up personal-data field was reported
//! as lacking `autocomplete`, and a combobox with `aria-autocomplete="list"`
//! as carrying an invalid token.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const INPUT_PURPOSE_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.5",
    name: "Identify Input Purpose",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "The purpose of each input field can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/identify-input-purpose.html",
    axe_id: "autocomplete-valid",
    tags: &["wcag2aa", "wcag135", "cat.forms"],
};

/// Known autocomplete token values per HTML spec
const AUTOCOMPLETE_TOKENS: &[&str] = &[
    "name",
    "given-name",
    "family-name",
    "additional-name",
    "honorific-prefix",
    "honorific-suffix",
    "nickname",
    "username",
    "new-password",
    "current-password",
    "one-time-code",
    "email",
    "tel",
    "tel-national",
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level1",
    "address-level2",
    "postal-code",
    "country",
    "country-name",
    "cc-name",
    "cc-number",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-csc",
    "cc-type",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "sex",
    "url",
    "organization",
    "organization-title",
];

/// Input types that typically collect user information
const USER_INPUT_TYPES: &[&str] = &["text", "email", "tel", "url", "search", "password"];

/// Label words that mark a field as collecting information about the user.
/// "name" is not in this list: it names things as often as people and is
/// classified word-aware by [`name_reference`] instead (#658).
const USER_INFO_KEYWORDS: &[&str] = &[
    "email", "phone", "tel", "address", "city", "zip", "postal", "country", "password", "username",
    "first", "last", "birthday", "birth",
];

/// Single words (incl. closed compounds) that always mean a person's name.
/// Detection vocabulary, not message text: English and German are merged
/// because the rule runs regardless of the page language and sites mix both.
const PERSONAL_NAME_WORDS: &[&str] = &[
    "firstname",
    "lastname",
    "fullname",
    "surname",
    "givenname",
    "familyname",
    "middlename",
    "username",
    "nickname",
    "vorname",
    "nachname",
    "familienname",
    "benutzername",
    "geburtsname",
    "rufname",
];

/// Words that may precede a bare "name" without turning it into the name of a
/// thing: they refer to the person ("your name", "first name", "Ihr Name") or
/// are technical/form-context parts of an id ("input-name", "billing_name").
/// Any other preceding word is a qualifier naming a thing ("view name", "file
/// name", "project name", "company name") and rules the field out.
const PERSON_OR_NEUTRAL_QUALIFIERS: &[&str] = &[
    // the person
    "your",
    "my",
    "full",
    "first",
    "last",
    "given",
    "family",
    "middle",
    "legal",
    "maiden",
    "user",
    "contact",
    "customer",
    "person",
    "member",
    "guest",
    "applicant",
    "author",
    "cardholder",
    "ihr",
    "ihren",
    "dein",
    "deinen",
    "vollständiger",
    "voller",
    // form/id context
    "input",
    "field",
    "form",
    "fld",
    "txt",
    "inp",
    "text",
    "billing",
    "shipping",
    "account",
    "profile",
    "signup",
    "register",
    "registration",
    "checkout",
];

/// Words after a bare "name" that introduce the thing being named ("Name of
/// the view", "Name der Ansicht").
const THING_NAME_FOLLOWERS: &[&str] = &["of", "for", "der", "des", "für", "von"];

// Collects every rendered, non-hidden text-like input with its type, its
// `autocomplete` attribute and a label approximating the accessible name
// (aria-label → aria-labelledby → <label> → title → placeholder). The
// classification happens in Rust (`evaluate`) so it is unit-testable.
const INPUT_PURPOSE_JS: &str = r#"
(function() {
  var types = ['text', 'email', 'tel', 'url', 'search', 'password'];
  var out = [];
  var inputs = document.querySelectorAll('input');
  for (var i = 0; i < inputs.length; i++) {
    var el = inputs[i];
    var type = (el.getAttribute('type') || 'text').toLowerCase();
    if (types.indexOf(type) === -1) continue;
    if (el.getClientRects().length === 0 || el.closest('[aria-hidden="true"]')) continue;
    var label = el.getAttribute('aria-label') || '';
    if (!label && el.getAttribute('aria-labelledby')) {
      label = el.getAttribute('aria-labelledby').split(/\s+/).map(function(id) {
        var ref = document.getElementById(id);
        return ref ? ref.textContent : '';
      }).join(' ');
    }
    if (!label && el.labels && el.labels.length) label = el.labels[0].textContent;
    if (!label) label = el.getAttribute('title') || el.getAttribute('placeholder') || '';
    var selector = el.id ? '#' + CSS.escape(el.id)
      : el.getAttribute('name') ? 'input[name="' + el.getAttribute('name') + '"]'
      : 'input[type="' + type + '"]';
    out.push({
      type: type,
      autocomplete: el.getAttribute('autocomplete'),
      label: label.replace(/\s+/g, ' ').trim(),
      selector: selector,
      id: el.id || '',
      name_attr: el.getAttribute('name') || ''
    });
  }
  return out;
})()
"#;

#[derive(Debug, Clone, serde::Deserialize)]
struct InputCandidate {
    #[serde(rename = "type")]
    input_type: String,
    autocomplete: Option<String>,
    label: String,
    selector: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    name_attr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameReference {
    /// The text refers to a person's name ("Name", "Your name", "Vorname").
    Person,
    /// The text names a thing ("View name", "Dateiname", "Name of the view").
    Thing,
}

/// Split into lower-case words at non-alphanumeric characters and camelCase
/// boundaries (`viewName` → `view`, `name`).
fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;
    for ch in text.chars() {
        if !ch.is_alphanumeric() {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            prev_lower = false;
            continue;
        }
        if ch.is_uppercase() && prev_lower {
            out.push(std::mem::take(&mut current));
        }
        prev_lower = ch.is_lowercase();
        current.extend(ch.to_lowercase());
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Classify what a label or id/name attribute says about "name"; `None` when
/// it contains no name word. A person reading anywhere in the text wins over
/// a thing reading.
fn name_reference(text: &str) -> Option<NameReference> {
    let words = words(text);
    let mut result = None;
    for (i, word) in words.iter().enumerate() {
        if PERSONAL_NAME_WORDS.contains(&word.as_str()) {
            return Some(NameReference::Person);
        }
        if word != "name" {
            continue;
        }
        let qualifier = words[..i]
            .iter()
            .rev()
            .find(|w| !w.chars().all(|c| c.is_ascii_digit()));
        let qualified_by_thing =
            qualifier.is_some_and(|q| !PERSON_OR_NEUTRAL_QUALIFIERS.contains(&q.as_str()));
        let followed_by_thing = words
            .get(i + 1)
            .is_some_and(|next| THING_NAME_FOLLOWERS.contains(&next.as_str()));
        if qualified_by_thing || followed_by_thing {
            result = Some(NameReference::Thing);
        } else {
            return Some(NameReference::Person);
        }
    }
    result
}

/// Whether the field asks for the user's name. The label is the positive
/// evidence; a thing qualifier in the id or name attribute (`#view-name`
/// labelled just "Name") rules it out unless the other attribute names the
/// person.
fn asks_for_personal_name(c: &InputCandidate) -> bool {
    if name_reference(&c.label) != Some(NameReference::Person) {
        return false;
    }
    let attr_refs = [name_reference(&c.id), name_reference(&c.name_attr)];
    attr_refs.contains(&Some(NameReference::Person))
        || !attr_refs.contains(&Some(NameReference::Thing))
}

pub async fn check_input_purpose_with_page(page: &Page) -> Vec<Violation> {
    let Ok(result) = page.evaluate(INPUT_PURPOSE_JS).await else {
        return Vec::new();
    };
    let Some(candidates) = result
        .value()
        .and_then(|v| serde_json::from_value::<Vec<InputCandidate>>(v.clone()).ok())
    else {
        return Vec::new();
    };
    evaluate(&candidates)
}

fn evaluate(candidates: &[InputCandidate]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for c in candidates {
        if !USER_INPUT_TYPES.contains(&c.input_type.as_str()) {
            continue;
        }
        let ac_value = c
            .autocomplete
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        let has_autocomplete = !ac_value.is_empty() && ac_value != "off" && ac_value != "on";

        if has_autocomplete {
            // Check last token (may have section- or shipping/billing prefix)
            let last_token = ac_value.split_whitespace().last().unwrap_or("");
            if !AUTOCOMPLETE_TOKENS.contains(&last_token) {
                violations.push(
                    Violation::new(
                        INPUT_PURPOSE_RULE.id,
                        INPUT_PURPOSE_RULE.name,
                        INPUT_PURPOSE_RULE.level,
                        Severity::Low,
                        format!("Invalid autocomplete value: '{}'", ac_value),
                        c.selector.clone(),
                    )
                    .with_selector(c.selector.clone())
                    .with_name(Some(c.label.clone()))
                    .with_fix("Use a valid autocomplete token from the HTML specification")
                    .with_help_url(INPUT_PURPOSE_RULE.help_url)
                    .with_rule_id(INPUT_PURPOSE_RULE.axe_id),
                );
            }
            continue;
        }

        // Check if this looks like a user-info field based on its label
        let label_lower = c.label.to_lowercase();
        if USER_INFO_KEYWORDS.iter().any(|k| label_lower.contains(k)) || asks_for_personal_name(c) {
            violations.push(
                Violation::new(
                    INPUT_PURPOSE_RULE.id,
                    INPUT_PURPOSE_RULE.name,
                    INPUT_PURPOSE_RULE.level,
                    Severity::Medium,
                    format!(
                        "Input '{}' appears to collect user info but lacks autocomplete attribute",
                        label_lower
                    ),
                    c.selector.clone(),
                )
                .with_selector(c.selector.clone())
                .with_name(Some(c.label.clone()))
                .with_fix(
                    "Add an appropriate autocomplete attribute (e.g., autocomplete=\"email\")",
                )
                .with_help_url(INPUT_PURPOSE_RULE.help_url)
                .with_rule_id(INPUT_PURPOSE_RULE.axe_id),
            );
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(label: &str, autocomplete: Option<&str>) -> InputCandidate {
        InputCandidate {
            input_type: "text".to_string(),
            autocomplete: autocomplete.map(str::to_string),
            label: label.to_string(),
            selector: "#x".to_string(),
            id: String::new(),
            name_attr: String::new(),
        }
    }

    fn named_input(label: &str, id: &str, name_attr: &str) -> InputCandidate {
        InputCandidate {
            id: id.to_string(),
            name_attr: name_attr.to_string(),
            ..input(label, None)
        }
    }

    /// #658: og-vanilla's saved-views form — `#view-name` labelled "Name" is
    /// the name of a grid view, not the user's name.
    #[test]
    fn thing_qualifier_in_id_rules_out_bare_name_label() {
        assert!(evaluate(&[named_input("Name", "view-name", "")]).is_empty());
        assert!(evaluate(&[named_input("Name", "", "projectName")]).is_empty());
    }

    /// #658: labels naming a thing are no personal-data fields.
    #[test]
    fn thing_names_in_label_are_not_personal() {
        for label in [
            "View name",
            "File name",
            "Filename",
            "Project name",
            "Company name",
            "Name of the view",
            "Dateiname",
            "Projektname",
            "Name der Ansicht",
            "Rename",
        ] {
            assert!(evaluate(&[input(label, None)]).is_empty(), "{label}");
        }
    }

    /// Fields that clearly ask for the person's name still fire.
    #[test]
    fn personal_name_fields_still_flagged() {
        for label in [
            "Name",
            "Your name",
            "Full name",
            "First name",
            "Last Name *",
            "Surname",
            "Vorname",
            "Nachname",
            "Ihr Name",
            "Benutzername",
        ] {
            assert_eq!(evaluate(&[input(label, None)]).len(), 1, "{label}");
        }
    }

    /// Technical or person-referring id parts don't turn a person's name into
    /// a thing's name.
    #[test]
    fn neutral_id_parts_keep_personal_name() {
        assert_eq!(evaluate(&[named_input("Name", "input-name", "")]).len(), 1);
        assert_eq!(
            evaluate(&[named_input("Name", "billing_first_name", "")]).len(),
            1
        );
        assert_eq!(
            evaluate(&[named_input("Name", "field-2-name", "")]).len(),
            1
        );
        // A person reading in one attribute outweighs a thing reading in the other.
        assert_eq!(
            evaluate(&[named_input("Name", "view-name", "fullName")]).len(),
            1
        );
    }

    #[test]
    fn words_split_camel_case_and_separators() {
        assert_eq!(words("viewName"), ["view", "name"]);
        assert_eq!(words("billing_first-name"), ["billing", "first", "name"]);
        assert_eq!(words("FIRSTNAME"), ["firstname"]);
    }

    #[test]
    fn test_valid_autocomplete() {
        assert!(evaluate(&[input("Email", Some("email"))]).is_empty());
    }

    #[test]
    fn test_missing_autocomplete_on_email_field() {
        let v = evaluate(&[input("Email Address", None)]);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, "1.3.5");
    }

    #[test]
    fn test_invalid_autocomplete_token() {
        assert_eq!(evaluate(&[input("Name", Some("foobar"))]).len(), 1);
    }

    #[test]
    fn test_generic_input_no_violation() {
        // A generic input with no user-info label shouldn't trigger
        assert!(evaluate(&[input("Search query", None)]).is_empty());
    }

    /// The token may carry a section or shipping/billing prefix; only the last
    /// token is the field name.
    #[test]
    fn prefixed_token_is_valid() {
        assert!(evaluate(&[input("Street", Some("shipping street-address"))]).is_empty());
    }

    /// `autocomplete="off"` on a personal-data field is the same as none.
    #[test]
    fn off_counts_as_missing() {
        assert_eq!(evaluate(&[input("Email", Some("off"))]).len(), 1);
    }
}
