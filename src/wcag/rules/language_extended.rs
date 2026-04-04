//! WCAG 3.1.2 - Language of Parts + lang attribute validity
//!
//! Extends the basic `html-has-lang` (3.1.1) check with:
//! - `valid-lang`:              the lang attribute must be a recognised BCP 47 primary subtag
//! - `html-xml-lang-mismatch`: when both lang and xml:lang are present they must agree

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation, WcagResults};

pub const RULE_VALID_LANG: RuleMetadata = RuleMetadata {
    id: "3.1.1",
    name: "Valid Language Code",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description:
        "The lang attribute must contain a valid BCP 47 primary language subtag",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/language-of-page.html",
    axe_id: "valid-lang",
    tags: &["wcag2a", "wcag311", "cat.language"],
};

pub const RULE_LANG_MISMATCH: RuleMetadata = RuleMetadata {
    id: "3.1.1",
    name: "Language Attribute Mismatch",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "lang and xml:lang attributes must specify the same language",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/language-of-page.html",
    axe_id: "html-xml-lang-mismatch",
    tags: &["wcag2a", "wcag311", "cat.language"],
};

/// Run extended language-related checks.
pub fn check_language_extended(tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    for node in tree.nodes.values() {
        if node.ignored {
            continue;
        }
        let role = match node.role.as_deref() {
            Some(r) => r,
            None => continue,
        };
        if role.to_lowercase() != "rootwebarea" && role.to_lowercase() != "document" {
            continue;
        }
        results.nodes_checked += 1;

        let lang = node.get_property_str("lang").unwrap_or("").to_string();
        let xml_lang = node.get_property_str("xmlLang")
            .or_else(|| node.get_property_str("xml:lang"))
            .unwrap_or("")
            .to_string();

        // 1. Validate primary subtag if present
        if !lang.is_empty() && !is_valid_primary_subtag(primary_subtag(&lang)) {
            let v = Violation::new(
                RULE_VALID_LANG.id,
                RULE_VALID_LANG.name,
                RULE_VALID_LANG.level,
                RULE_VALID_LANG.severity,
                &format!("lang=\"{}\" is not a recognised BCP 47 primary language subtag", lang),
                &node.node_id,
            )
            .with_fix("Use a valid BCP 47 primary subtag such as lang=\"en\" or lang=\"de\"")
            .with_help_url(RULE_VALID_LANG.help_url);
            results.add_violation(v);
        } else if !lang.is_empty() {
            results.passes += 1;
        }

        // 2. Mismatch between lang and xml:lang
        if !lang.is_empty() && !xml_lang.is_empty() {
            let primary_a = primary_subtag(&lang).to_lowercase();
            let primary_b = primary_subtag(&xml_lang).to_lowercase();
            if primary_a != primary_b {
                let v = Violation::new(
                    RULE_LANG_MISMATCH.id,
                    RULE_LANG_MISMATCH.name,
                    RULE_LANG_MISMATCH.level,
                    RULE_LANG_MISMATCH.severity,
                    &format!(
                        "lang=\"{}\" and xml:lang=\"{}\" specify different primary languages",
                        lang, xml_lang
                    ),
                    &node.node_id,
                )
                .with_fix(
                    "Ensure lang and xml:lang use the same primary language subtag",
                )
                .with_help_url(RULE_LANG_MISMATCH.help_url);
                results.add_violation(v);
            } else {
                results.passes += 1;
            }
        }
    }

    results
}

/// Extract the primary subtag from a BCP 47 tag (everything before the first `-`).
fn primary_subtag(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// Validate a primary BCP 47 subtag: must be 2 or 3 ASCII letters.
fn is_valid_primary_subtag(subtag: &str) -> bool {
    let len = subtag.len();
    (2..=3).contains(&len) && subtag.chars().all(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::{AXNode, AXProperty, AXValue};

    fn doc_node(lang: Option<&str>, xml_lang: Option<&str>) -> AXNode {
        let mut props = vec![];
        if let Some(l) = lang {
            props.push(AXProperty {
                name: "lang".into(),
                value: AXValue::String(l.into()),
            });
        }
        if let Some(x) = xml_lang {
            props.push(AXProperty {
                name: "xmlLang".into(),
                value: AXValue::String(x.into()),
            });
        }
        AXNode {
            node_id: "doc".into(),
            ignored: false,
            ignored_reasons: vec![],
            role: Some("RootWebArea".into()),
            name: Some("Test".into()),
            name_source: None,
            description: None,
            value: None,
            properties: props,
            child_ids: vec![],
            parent_id: None,
            backend_dom_node_id: None,
        }
    }

    #[test]
    fn test_valid_lang_passes() {
        let tree = AXTree::from_nodes(vec![doc_node(Some("en"), None)]);
        let r = check_language_extended(&tree);
        assert!(r.violations.is_empty());
        assert_eq!(r.passes, 1);
    }

    #[test]
    fn test_valid_lang_with_region_passes() {
        let tree = AXTree::from_nodes(vec![doc_node(Some("en-US"), None)]);
        let r = check_language_extended(&tree);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn test_invalid_lang_flagged() {
        let tree = AXTree::from_nodes(vec![doc_node(Some("x"), None)]);
        let r = check_language_extended(&tree);
        assert!(r.violations.iter().any(|v| v.rule_name == "Valid Language Code"));
    }

    #[test]
    fn test_lang_mismatch_flagged() {
        let tree = AXTree::from_nodes(vec![doc_node(Some("en"), Some("de"))]);
        let r = check_language_extended(&tree);
        assert!(r.violations.iter().any(|v| v.rule_name == "Language Attribute Mismatch"));
    }

    #[test]
    fn test_lang_no_mismatch_when_equal() {
        let tree = AXTree::from_nodes(vec![doc_node(Some("en"), Some("en-US"))]);
        let r = check_language_extended(&tree);
        assert!(!r.violations.iter().any(|v| v.rule_name == "Language Attribute Mismatch"));
    }
}
