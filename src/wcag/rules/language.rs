//! WCAG 3.1.1 Language of Page
//!
//! The default human language of each Web page can be programmatically determined.
//! Level A

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

/// Rule metadata for 3.1.1
pub const LANGUAGE_RULE: RuleMetadata = RuleMetadata {
    id: "3.1.1",
    name: "Language of Page",
    level: WcagLevel::A,
    severity: Severity::Serious,
    description: "The default human language of each Web page can be programmatically determined",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/language-of-page.html",
};

/// Check for valid lang attribute on the document
pub fn check_language(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();
    results.nodes_checked += 1;

    // Look for the root document node to check language
    let mut found_document = false;
    let mut has_valid_lang = false;

    for node in tree.iter() {
        if let Some(ref role) = node.role {
            let role_lower = role.to_lowercase();
            if role_lower == "rootwebarea" || role_lower == "document" {
                found_document = true;

                // Check for language property
                if let Some(lang) = node.get_property_str("lang") {
                    if is_valid_language_code(&lang) {
                        has_valid_lang = true;
                    }
                }

                // Also check description which sometimes contains language info
                if let Some(ref desc) = node.description {
                    if desc.to_lowercase().contains("lang=") {
                        // Extract and validate language
                        has_valid_lang = true;
                    }
                }
            }
        }
    }

    if found_document && !has_valid_lang {
        let violation = Violation::new(
            LANGUAGE_RULE.id,
            LANGUAGE_RULE.name,
            LANGUAGE_RULE.level,
            Severity::Serious,
            "Page is missing a valid lang attribute on the html element",
            "document",
        )
        .with_fix("Add a valid lang attribute to the <html> element, e.g., <html lang=\"en\">")
        .with_help_url(LANGUAGE_RULE.help_url);

        results.add_violation(violation);
    } else if has_valid_lang {
        results.passes += 1;
    }

    results
}

/// Check if a language code is valid (basic validation)
fn is_valid_language_code(code: &str) -> bool {
    let code = code.trim().to_lowercase();

    if code.is_empty() {
        return false;
    }

    // Basic validation: language codes are typically 2-3 letters
    // optionally followed by region codes
    // e.g., "en", "en-US", "zh-Hans"
    let parts: Vec<&str> = code.split('-').collect();

    if parts.is_empty() {
        return false;
    }

    // Primary language subtag should be 2-3 letters
    let primary = parts[0];
    if primary.len() < 2 || primary.len() > 3 || !primary.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    // Common language codes
    let common_codes = [
        "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh", "ar", "hi", "nl", "pl", "sv",
        "tr", "vi", "th", "cs", "da", "fi", "el", "he", "hu", "id", "ms", "no", "ro", "sk", "uk",
    ];

    common_codes.contains(&primary) || primary.len() >= 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXValue};

    fn create_document_with_lang(id: &str, lang: Option<&str>) -> AXNode {
        let mut node = AXNode {
            node_id: id.to_string(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("RootWebArea".to_string()),
            name: Some("Test Page".to_string()),
            name_source: None,
            description: None,
            value: None,
            properties: vec![],
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        };

        if let Some(l) = lang {
            node.properties.push(AXProperty {
                name: "lang".to_string(),
                value: AXValue::String(l.to_string()),
            });
        }

        node
    }

    #[test]
    fn test_language_rule_metadata() {
        assert_eq!(LANGUAGE_RULE.id, "3.1.1");
        assert_eq!(LANGUAGE_RULE.level, WcagLevel::A);
    }

    #[test]
    fn test_page_with_valid_lang() {
        let tree = AXTree::from_nodes(vec![create_document_with_lang("1", Some("en"))]);
        let results = check_language(&tree);
        assert!(results.violations.is_empty());
    }

    #[test]
    fn test_page_without_lang() {
        let tree = AXTree::from_nodes(vec![create_document_with_lang("1", None)]);
        let results = check_language(&tree);
        assert!(!results.violations.is_empty());
    }

    #[test]
    fn test_is_valid_language_code() {
        assert!(is_valid_language_code("en"));
        assert!(is_valid_language_code("en-US"));
        assert!(is_valid_language_code("zh-Hans"));
        assert!(is_valid_language_code("pt-BR"));
        assert!(!is_valid_language_code(""));
        assert!(!is_valid_language_code("x"));
    }
}
