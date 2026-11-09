//! Address of a rule's documentation page on the project site (#929).
//!
//! One function maps a rule id to its URL; every consumer of `help_url` goes
//! through it, so the scheme lives in this file only. The pages themselves are
//! generated from the rule catalogue and the rule contracts
//! (`tests/rule_docs.rs`, committed under `site/src/data/rules/`) and rendered
//! by `site/src/pages/rules/`.
//!
//! URL scheme: `<RULE_DOCS_BASE_URL><rule_id>/` — the id verbatim, with `/` as
//! path separator and a trailing slash (the directory form the static site
//! serves without a redirect). An addressable id consists of lowercase ASCII
//! letters, digits, `-`, `.` and `/` only, has no empty or dot-only segment and
//! no leading or trailing `/`, so it needs no percent-encoding and cannot
//! escape `rules/`. `is_addressable_rule_id` states that rule; the generator
//! test refuses every catalogue id that breaks it.

/// Landing page of the rule documentation; every rule page lives below it.
pub const RULE_DOCS_BASE_URL: &str = "https://casoon.github.io/auditmysite/rules/";

/// URL of the documentation page for `rule_id`.
pub fn rule_doc_url(rule_id: &str) -> String {
    format!("{RULE_DOCS_BASE_URL}{rule_id}/")
}

/// Whether `rule_id` can be part of a rule page URL as it is.
pub fn is_addressable_rule_id(rule_id: &str) -> bool {
    !rule_id.is_empty()
        && rule_id.split('/').all(|segment| {
            !segment.is_empty()
                && segment.chars().any(|c| c != '.')
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-.".contains(c))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_url_carries_the_id_verbatim() {
        assert_eq!(
            rule_doc_url("document/title-missing"),
            "https://casoon.github.io/auditmysite/rules/document/title-missing/"
        );
        assert_eq!(
            rule_doc_url("image-alt"),
            "https://casoon.github.io/auditmysite/rules/image-alt/"
        );
    }

    #[test]
    fn slashes_and_dots_are_addressable_but_nothing_that_escapes() {
        assert!(is_addressable_rule_id("document/title-missing"));
        assert!(is_addressable_rule_id("a11y.x"));
        for bad in [
            "", "/a", "a/", "a//b", "a/../b", ".", "A", "a b", "a?b", "a#b", "a%20b",
        ] {
            assert!(!is_addressable_rule_id(bad), "{bad:?}");
        }
    }
}
