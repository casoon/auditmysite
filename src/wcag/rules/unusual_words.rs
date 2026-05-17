//! WCAG 3.1.3 Unusual Words (Level AAA)
//!
//! A mechanism is available for identifying specific definitions of words or
//! phrases used in an unusual or restricted way, including idioms and jargon.
//!
//! This criterion is fully not-testable automatically.

use crate::accessibility::AXTree;
use crate::cli::WcagLevel;
use crate::wcag::types::{FindingKind, RuleMetadata, Severity, Violation, WcagResults};

pub const UNUSUAL_WORDS_RULE: RuleMetadata = RuleMetadata {
    id: "3.1.3",
    name: "Unusual Words",
    level: WcagLevel::AAA,
    severity: Severity::Low,
    description: "Definitions are available for unusual words, idioms, and jargon",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/unusual-words.html",
    axe_id: "unusual-words",
    tags: &["wcag2aaa", "wcag313", "cat.language"],
};

pub fn check_unusual_words(_tree: &AXTree) -> WcagResults {
    let mut results = WcagResults::new();

    results.add_violation(
        Violation::new(
            UNUSUAL_WORDS_RULE.id,
            UNUSUAL_WORDS_RULE.name,
            UNUSUAL_WORDS_RULE.level,
            Severity::Low,
            "WCAG 3.1.3 (Unusual Words) requires manual review. Verify that definitions \
             are provided for all words or phrases used in an unusual, restricted, or \
             technical sense, including idioms and jargon.",
            "page",
        )
        .with_fix(
            "Provide a glossary, inline definitions (e.g. using <dfn> or <abbr title=\"...\">), \
             or tooltips for technical terms, idioms, and jargon used on the page.",
        )
        .with_rule_id(UNUSUAL_WORDS_RULE.axe_id)
        .with_help_url(UNUSUAL_WORDS_RULE.help_url)
        .with_kind(FindingKind::NotTestable),
    );

    results
}
