//! WCAG 1.3.5 Identify Input Purpose
//!
//! The purpose of each input field collecting information about the user
//! can be programmatically determined when the input field serves a purpose
//! identified in the Input Purposes for User Interface Components section.
//! Level AA

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const INPUT_PURPOSE_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.5",
    name: "Identify Input Purpose",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "The purpose of each input field can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/identify-input-purpose.html",
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

pub fn check_input_purpose(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.form_controls() {
        if node.ignored {
            continue;
        }

        let role = node.role.as_deref().unwrap_or("");

        // Only check text-like inputs
        let is_text_input = role == "textbox"
            || role == "searchbox"
            || role == "spinbutton"
            || node
                .get_property_str("inputType")
                .is_some_and(|t| USER_INPUT_TYPES.contains(&t));

        if !is_text_input {
            continue;
        }

        // Check if autocomplete attribute is present
        let has_autocomplete = node.get_property_str("autocomplete").is_some_and(|ac| {
            let token = ac.trim().to_lowercase();
            token != "off" && token != "on" && !token.is_empty()
        });

        if has_autocomplete {
            // Validate the autocomplete value
            let ac_value = node
                .get_property_str("autocomplete")
                .unwrap_or("")
                .trim()
                .to_lowercase();

            // Check last token (may have section- or shipping/billing prefix)
            let last_token = ac_value.split_whitespace().last().unwrap_or("");
            if !AUTOCOMPLETE_TOKENS.contains(&last_token)
                && last_token != "off"
                && last_token != "on"
            {
                let violation = Violation::new(
                    INPUT_PURPOSE_RULE.id,
                    INPUT_PURPOSE_RULE.name,
                    INPUT_PURPOSE_RULE.level,
                    Severity::Low,
                    format!("Invalid autocomplete value: '{}'", ac_value),
                    node.node_id.clone(),
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix("Use a valid autocomplete token from the HTML specification")
                .with_help_url(INPUT_PURPOSE_RULE.help_url);

                results.add_violation(violation);
            } else {
                results.passes += 1;
            }
        } else {
            // Check if this looks like a user-info field based on name/label
            let name_lower = node.name.as_deref().unwrap_or("").to_lowercase();

            let likely_user_input = [
                "name", "email", "phone", "tel", "address", "city", "zip", "postal", "country",
                "password", "username", "first", "last", "birthday", "birth",
            ]
            .iter()
            .any(|keyword| name_lower.contains(keyword));

            if likely_user_input {
                let violation = Violation::new(
                    INPUT_PURPOSE_RULE.id,
                    INPUT_PURPOSE_RULE.name,
                    INPUT_PURPOSE_RULE.level,
                    Severity::Medium,
                    format!(
                        "Input '{}' appears to collect user info but lacks autocomplete attribute",
                        name_lower
                    ),
                    node.node_id.clone(),
                )
                .with_role(node.role.clone())
                .with_name(node.name.clone())
                .with_fix(
                    "Add an appropriate autocomplete attribute (e.g., autocomplete=\"email\")",
                )
                .with_help_url(INPUT_PURPOSE_RULE.help_url);

                results.add_violation(violation);
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXTree, AXValue};

    fn text_input(id: &str, name: &str, autocomplete: Option<&str>) -> AXNode {
        let mut properties = vec![];
        if let Some(ac) = autocomplete {
            properties.push(AXProperty {
                name: "autocomplete".to_string(),
                value: AXValue::String(ac.to_string()),
            });
        }
        AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("textbox".to_string()),
            name: Some(name.to_string()),
            name_source: None,
            description: None,
            value: None,
            properties,
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_valid_autocomplete() {
        let tree = AXTree::from_nodes(vec![text_input("1", "Email", Some("email"))]);
        let results = check_input_purpose(&tree);
        assert_eq!(results.violations.len(), 0);
        assert_eq!(results.passes, 1);
    }

    #[test]
    fn test_missing_autocomplete_on_email_field() {
        let tree = AXTree::from_nodes(vec![text_input("1", "Email Address", None)]);
        let results = check_input_purpose(&tree);
        assert_eq!(results.violations.len(), 1);
        assert_eq!(results.violations[0].rule, "1.3.5");
    }

    #[test]
    fn test_invalid_autocomplete_token() {
        let tree = AXTree::from_nodes(vec![text_input("1", "Name", Some("foobar"))]);
        let results = check_input_purpose(&tree);
        assert_eq!(results.violations.len(), 1);
    }

    #[test]
    fn test_generic_input_no_violation() {
        // A generic input with no user-info label shouldn't trigger
        let tree = AXTree::from_nodes(vec![text_input("1", "Search query", None)]);
        let results = check_input_purpose(&tree);
        assert_eq!(results.violations.len(), 0);
    }
}
