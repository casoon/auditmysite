//! WCAG 1.3.6 Identify Purpose (Level AAA)
//!
//! The purpose of user interface components, icons, and regions can be
//! programmatically determined. Inputs collecting personal data should use
//! autocomplete tokens to allow user agents and AT to adapt the interface.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation};

pub const IDENTIFY_PURPOSE_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.6",
    name: "Identify Purpose",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Purpose of UI components, icons, and regions can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/identify-purpose.html",
    axe_id: "identify-purpose",
    tags: &["wcag2aaa", "wcag136", "cat.semantics"],
};

// Collects every non-hidden, free-text-ish input's name/id/type/autocomplete
// state — classification (does this field actually collect personal data?)
// happens in Rust (`is_personal_data_input`), not here, so it's covered by
// unit tests instead of only being exercisable through a live page (#34,
// plan/34-identify-purpose-substring-false-positive-risk.md). The previous
// version matched `input[type="text"], input[type="search"]` unconditionally
// regardless of name — those two clauses alone made almost any plain text
// input on a page (a search box, a coupon field, a username field) count as
// "collects personal information".
const IDENTIFY_PURPOSE_JS: &str = r#"
(function() {
  var excluded_types = ['hidden', 'submit', 'button', 'checkbox', 'radio', 'password', 'file', 'image', 'reset'];
  var candidates = [];
  var inputs = document.querySelectorAll('input');
  for (var i = 0; i < inputs.length; i++) {
    var el = inputs[i];
    var type = (el.getAttribute('type') || 'text').toLowerCase();
    if (excluded_types.indexOf(type) !== -1) {
      continue;
    }
    candidates.push({
      name: el.getAttribute('name') || '',
      id: el.getAttribute('id') || '',
      type: type,
      has_autocomplete: !!el.getAttribute('autocomplete')
    });
  }
  return { candidates: candidates };
})()
"#;

/// Splits an identifier into lowercase word tokens on separators
/// (`_`, `-`, whitespace, digits) and camelCase boundaries — `"billing_address_2"`
/// and `"billingAddress"` both yield `["billing", "address"]`. Unicode-aware
/// (`str::to_lowercase`, not `to_ascii_lowercase`) so `"Straße"` stays `"straße"`
/// rather than being mangled.
fn split_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_alphabetic() {
            if c.is_uppercase() && i > 0 && chars[i - 1].is_lowercase() && !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            current.extend(c.to_lowercase());
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Word tokens that, on their own, indicate a field collects personal data
/// covered by an HTML `autocomplete` token (name/address/phone/email parts).
/// Includes common English and German field-naming conventions, plus a few
/// no-separator compound spellings (`"firstname"`) that `split_words` can't
/// otherwise decompose.
const PERSONAL_DATA_TOKENS: &[&str] = &[
    // Name
    "name",
    "vorname",
    "nachname",
    "firstname",
    "lastname",
    "fullname",
    "givenname",
    "familyname",
    "surname",
    // Street address
    "address",
    "adresse",
    "street",
    "strasse",
    "straße",
    "hausnummer",
    "housenumber",
    // Postal code / city / region / country
    "plz",
    "postleitzahl",
    "postcode",
    "postalcode",
    "postal",
    "zip",
    "zipcode",
    "city",
    "ort",
    "stadt",
    "town",
    "country",
    "land",
    "region",
    "state",
    "bundesland",
    // Phone
    "phone",
    "telefon",
    "telefonnummer",
    "handynummer",
    "handy",
    "mobil",
    "mobilnummer",
    // Email (type="email" is handled separately; this covers untyped fields
    // named e.g. "email"/"e_mail")
    "email",
    "mail",
];

/// Word-token prefixes that, immediately before an `"address"`/`"adresse"`
/// token, mean the field is not a postal address (e.g. `"ip_address"`,
/// `"wallet_address"`) — the historical false-positive case this rule used
/// to flag via plain substring matching.
const NON_PERSONAL_ADDRESS_PREFIXES: &[&str] = &["ip", "mac", "wallet", "contract"];

fn has_personal_data_token(identifier: &str) -> bool {
    let words = split_words(identifier);
    for (i, word) in words.iter().enumerate() {
        if word == "address" || word == "adresse" {
            let preceded_by_non_personal =
                i > 0 && NON_PERSONAL_ADDRESS_PREFIXES.contains(&words[i - 1].as_str());
            if preceded_by_non_personal {
                continue;
            }
            return true;
        }
        if PERSONAL_DATA_TOKENS.contains(&word.as_str()) {
            return true;
        }
    }
    false
}

/// Whether an input field collects personal data that an `autocomplete`
/// token should describe — either a semantic HTML5 input type (`email`,
/// `tel`, always personal regardless of name) or a name/id matching a known
/// personal-data field-naming convention.
fn is_personal_data_input(input_type: &str, name: &str, id: &str) -> bool {
    if input_type == "email" || input_type == "tel" {
        return true;
    }
    has_personal_data_token(name) || has_personal_data_token(id)
}

pub async fn check_identify_purpose_with_page(page: &Page) -> Vec<Violation> {
    let not_testable = Violation::new(
        IDENTIFY_PURPOSE_RULE.id,
        IDENTIFY_PURPOSE_RULE.name,
        IDENTIFY_PURPOSE_RULE.level,
        Severity::Low,
        "Full WCAG 1.3.6 compliance (identify purpose of icons and regions) requires manual review. \
         Automated checks only cover inputs missing autocomplete tokens.",
        "page",
    )
    .with_fix(
        "Add appropriate ARIA landmark roles and autocomplete attributes to all interactive \
         components that collect personal information.",
    )
    .with_rule_id(IDENTIFY_PURPOSE_RULE.axe_id)
    .with_help_url(IDENTIFY_PURPOSE_RULE.help_url)
    .with_kind(FindingKind::NotTestable);

    let result = match page.evaluate(IDENTIFY_PURPOSE_JS).await {
        Ok(r) => r,
        Err(_) => return vec![not_testable],
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => return vec![not_testable],
    };

    let candidates = val.get("candidates").and_then(|v| v.as_array());
    let missing: Vec<String> = candidates
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let input_type = c.get("type").and_then(|v| v.as_str()).unwrap_or("text");
                    let has_autocomplete = c
                        .get("has_autocomplete")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if has_autocomplete || !is_personal_data_input(input_type, name, id) {
                        return None;
                    }
                    let desc = if !name.is_empty() {
                        name
                    } else if !id.is_empty() {
                        id
                    } else {
                        input_type
                    };
                    Some(desc.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    let mut findings = vec![not_testable];

    for name in missing.iter().take(5) {
        findings.push(
            Violation::new(
                IDENTIFY_PURPOSE_RULE.id,
                IDENTIFY_PURPOSE_RULE.name,
                IDENTIFY_PURPOSE_RULE.level,
                Severity::Low,
                format!(
                    "Input '{}' collects personal information but has no autocomplete attribute.",
                    name
                ),
                name.as_str(),
            )
            .with_selector(name.as_str())
            .with_fix(
                "Add an appropriate autocomplete token (e.g. autocomplete=\"email\", \
                 autocomplete=\"tel\", autocomplete=\"name\") to help users and assistive technologies.",
            )
            .with_rule_id(IDENTIFY_PURPOSE_RULE.axe_id)
            .with_help_url(IDENTIFY_PURPOSE_RULE.help_url),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_words_handles_snake_kebab_and_camel_case() {
        // Digits are separators, not their own tokens — "2" is dropped.
        assert_eq!(split_words("billing_address_2"), vec!["billing", "address"]);
        assert_eq!(split_words("billing-address"), vec!["billing", "address"]);
        assert_eq!(split_words("billingAddress"), vec!["billing", "address"]);
        assert_eq!(split_words("PLZ"), vec!["plz"]);
        assert_eq!(split_words("Straße"), vec!["straße"]);
    }

    #[test]
    fn recognizes_english_and_german_personal_data_fields() {
        for name in [
            "address",
            "street_address",
            "billing_address",
            "shippingAddress",
            "firstname",
            "first_name",
            "vorname",
            "nachname",
            "plz",
            "postal_code",
            "city",
            "ort",
            "phone",
            "telefonnummer",
            "handynummer",
            "email_address",
        ] {
            assert!(
                is_personal_data_input("text", name, ""),
                "{name} should be recognized as a personal-data field"
            );
        }
    }

    #[test]
    fn does_not_flag_technical_or_unrelated_address_fields() {
        // plan/34-identify-purpose-substring-false-positive-risk.md: plain
        // substring matching on "address" used to flag these.
        for name in [
            "ip_address",
            "mac_address",
            "wallet_address",
            "contract_address",
        ] {
            assert!(
                !is_personal_data_input("text", name, ""),
                "{name} must not be treated as a postal-address field"
            );
        }
    }

    #[test]
    fn does_not_flag_generic_text_fields_by_type_alone() {
        // The previous CSS selector matched `input[type="text"]`/`[type="search"]`
        // unconditionally, regardless of the field's name.
        for (input_type, name) in [
            ("text", "promo_code"),
            ("text", "username"),
            ("search", "q"),
            ("text", ""),
        ] {
            assert!(
                !is_personal_data_input(input_type, name, ""),
                "input[type={input_type}][name={name}] must not be flagged without a personal-data name"
            );
        }
    }

    #[test]
    fn email_and_tel_types_always_count_regardless_of_name() {
        assert!(is_personal_data_input("email", "field1", ""));
        assert!(is_personal_data_input("tel", "field2", ""));
    }

    #[test]
    fn id_is_checked_when_name_is_absent() {
        assert!(is_personal_data_input("text", "", "billing-address"));
    }
}
