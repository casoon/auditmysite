//! WCAG 3.1.1 - Language of Page (lang attribute validity)
//!
//! Extends the basic `html-has-lang` (3.1.1) check with:
//! - `valid-lang`:              the lang attribute must be a recognised BCP 47 primary subtag
//! - `html-xml-lang-mismatch`: when both lang and xml:lang are present they must agree
//!
//! Despite the module name, this does not implement 3.1.2 Language of Parts
//! (per-element `lang` switches for foreign-language passages) — both checks
//! below are tagged and scored as 3.1.1 (see `RULE_VALID_LANG`/
//! `RULE_LANG_MISMATCH`), consistent with `taxonomy/rules.rs` and
//! `page_rules.rs` (`"3.1.1/language-extended"`). A real 3.1.2 check would
//! need per-element lang-switch detection, which this module does not do.
//!
//! DOM-level rule: reads `<html lang>`/`<html xml:lang>` directly via CDP.
//! The AX tree exposes a synthesized `language` property (see `language.rs`,
//! #QA-001), not the raw `lang`/`xmlLang` attribute values this check needs
//! to validate — an earlier tree-based implementation read AX property names
//! (`lang`, `xmlLang`) that don't exist at all, and so never fired in
//! production (#QA-030).

use chromiumoxide::Page;
use tracing::warn;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const RULE_VALID_LANG: RuleMetadata = RuleMetadata {
    id: "3.1.1",
    name: "Valid Language Code",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description: "The lang attribute must contain a valid BCP 47 primary language subtag",
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

const LANG_ATTRS_JS: &str = "({ \
    lang: document.documentElement.getAttribute('lang') || '', \
    xmlLang: document.documentElement.getAttribute('xml:lang') || '' \
})";

/// Extract the primary subtag from a BCP 47 tag (everything before the first `-`).
fn primary_subtag(tag: &str) -> &str {
    tag.split('-').next().unwrap_or(tag)
}

/// Validate a primary BCP 47 subtag: must be 2 or 3 ASCII letters.
fn is_valid_primary_subtag(subtag: &str) -> bool {
    let len = subtag.len();
    (2..=3).contains(&len) && subtag.chars().all(|c| c.is_ascii_alphabetic())
}

/// Run extended language-related checks against the live DOM.
pub async fn check_language_extended_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(LANG_ATTRS_JS).await {
        Ok(r) => r,
        Err(e) => {
            warn!("language-extended JS failed: {}", e);
            return vec![crate::wcag::technical_rule_failure_for(
                "language-extended",
                crate::cli::WcagLevel::A,
                "page_evaluation_failed",
            )];
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure_for(
                "language-extended",
                crate::cli::WcagLevel::A,
                "missing_evaluation_value",
            )]
        }
    };

    let lang = val.get("lang").and_then(|v| v.as_str()).unwrap_or("");
    let xml_lang = val.get("xmlLang").and_then(|v| v.as_str()).unwrap_or("");

    let mut violations = Vec::new();

    // 1. Validate primary subtag if present
    if !lang.is_empty() && !is_valid_primary_subtag(primary_subtag(lang)) {
        violations.push(
            Violation::new(
                RULE_VALID_LANG.id,
                RULE_VALID_LANG.name,
                RULE_VALID_LANG.level,
                RULE_VALID_LANG.severity,
                format!(
                    "lang=\"{}\" is not a recognised BCP 47 primary language subtag",
                    lang
                ),
                "html",
            )
            .with_selector("html")
            .with_fix("Use a valid BCP 47 primary subtag such as lang=\"en\" or lang=\"de\"")
            .with_rule_id(RULE_VALID_LANG.axe_id)
            .with_help_url(RULE_VALID_LANG.help_url),
        );
    }

    // 2. Mismatch between lang and xml:lang
    if !lang.is_empty() && !xml_lang.is_empty() {
        let primary_a = primary_subtag(lang).to_lowercase();
        let primary_b = primary_subtag(xml_lang).to_lowercase();
        if primary_a != primary_b {
            violations.push(
                Violation::new(
                    RULE_LANG_MISMATCH.id,
                    RULE_LANG_MISMATCH.name,
                    RULE_LANG_MISMATCH.level,
                    RULE_LANG_MISMATCH.severity,
                    format!(
                        "lang=\"{}\" and xml:lang=\"{}\" specify different primary languages",
                        lang, xml_lang
                    ),
                    "html",
                )
                .with_selector("html")
                .with_fix("Ensure lang and xml:lang use the same primary language subtag")
                .with_rule_id(RULE_LANG_MISMATCH.axe_id)
                .with_help_url(RULE_LANG_MISMATCH.help_url),
            );
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_primary_subtag() {
        assert!(is_valid_primary_subtag("en"));
        assert!(is_valid_primary_subtag("deu"));
        assert!(!is_valid_primary_subtag(""));
        assert!(!is_valid_primary_subtag("x"));
        assert!(!is_valid_primary_subtag("toolong"));
    }

    #[test]
    fn test_primary_subtag_extraction() {
        assert_eq!(primary_subtag("en-US"), "en");
        assert_eq!(primary_subtag("zh-Hans"), "zh");
        assert_eq!(primary_subtag("de"), "de");
    }
}
