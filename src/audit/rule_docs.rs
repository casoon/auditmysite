//! Which documentation page a finding links to (#929).
//!
//! The page id is the part of the URL after `rules/` (`taxonomy::rule_doc_url`).
//! It is always an id the finding itself carries:
//!
//! - WCAG findings: the engine rule id (`images/alt-missing`).
//! - Security header findings: `security.headers.<issue_type>`. Findings of the
//!   different headers that share an issue type (`missing_header`) are one
//!   finding, so they are one page; the page lists each header's check.
//! - Vulnerable-library findings: `best_practices.vulnerable_libraries.<name>`.
//! - Structured-data findings about missing required properties:
//!   `seo.schema.schema_rule_missing_<feature>_<property>` carries the feature
//!   key inside it; the page is the feature's, `seo.schema.schema_rule_missing_<feature>`.
//!
//! Every other finding has no page and no `help_url`. The generator of the
//! pages (`tests/common/rule_docs.rs`) uses the same functions, so a page and
//! the link to it cannot disagree.

use crate::taxonomy::{rule_doc_url, RuleLookup};
use std::collections::BTreeSet;
use std::sync::OnceLock;

pub const SECURITY_PREFIX: &str = "security.headers.";
pub const LIBRARY_PREFIX: &str = "best_practices.vulnerable_libraries.";
pub const SCHEMA_PREFIX: &str = "seo.schema.schema_rule_missing_";

/// Page id of a security check (`"Header:issue_type"` as the contract names it).
pub fn security_page_id(check_id: &str) -> Option<String> {
    let (_, issue_type) = check_id.split_once(':')?;
    Some(format!("{SECURITY_PREFIX}{issue_type}"))
}

/// Page id of a checked library, by its name.
pub fn library_page_id(name: &str) -> String {
    format!("{LIBRARY_PREFIX}{}", name.to_ascii_lowercase())
}

/// Page id of a structured-data feature, by its key.
pub fn schema_page_id(feature_key: &str) -> String {
    format!("{SCHEMA_PREFIX}{feature_key}")
}

fn security_issue_types() -> &'static BTreeSet<String> {
    static TYPES: OnceLock<BTreeSet<String>> = OnceLock::new();
    TYPES.get_or_init(|| {
        crate::security::all_security_check_ids()
            .iter()
            .filter_map(|id| id.split_once(':').map(|(_, t)| t.to_string()))
            .collect()
    })
}

/// The page that documents the finding with this rule id, if there is one.
pub fn page_id_for_finding(rule_id: &str) -> Option<String> {
    if RuleLookup::by_id(rule_id).is_some() {
        return Some(rule_id.to_string());
    }
    if let Some(issue_type) = rule_id.strip_prefix(SECURITY_PREFIX) {
        return security_issue_types()
            .contains(issue_type)
            .then(|| rule_id.to_string());
    }
    if let Some(name) = rule_id.strip_prefix(LIBRARY_PREFIX) {
        return crate::best_practices::CHECKED_LIBRARIES
            .iter()
            .any(|lib| lib.eq_ignore_ascii_case(name))
            .then(|| rule_id.to_string());
    }
    if let Some(rest) = rule_id.strip_prefix(SCHEMA_PREFIX) {
        return crate::seo::schema_rules::SchemaFeature::ALL
            .iter()
            .map(|f| f.key())
            .filter(|key| {
                rest == *key || rest.strip_prefix(key).is_some_and(|r| r.starts_with('_'))
            })
            .max_by_key(|key| key.len())
            .map(schema_page_id);
    }
    None
}

/// `help_url` of the finding with this rule id: its page on the project site.
pub fn finding_help_url(rule_id: &str) -> Option<String> {
    page_id_for_finding(rule_id).map(|id| rule_doc_url(&id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_family_links_to_the_page_of_the_id_it_carries() {
        assert_eq!(
            page_id_for_finding("images/alt-missing").as_deref(),
            Some("images/alt-missing")
        );
        assert_eq!(
            page_id_for_finding("security.headers.missing_header").as_deref(),
            Some("security.headers.missing_header")
        );
        assert_eq!(
            page_id_for_finding("security.headers.missing_object-src").as_deref(),
            Some("security.headers.missing_object-src")
        );
        assert_eq!(
            page_id_for_finding("best_practices.vulnerable_libraries.angularjs").as_deref(),
            Some("best_practices.vulnerable_libraries.angularjs")
        );
        assert_eq!(
            page_id_for_finding("seo.schema.schema_rule_missing_product_snippet_offers").as_deref(),
            Some("seo.schema.schema_rule_missing_product_snippet")
        );
    }

    #[test]
    fn ids_without_a_page_get_no_link() {
        for id in [
            "seo.meta.description",
            "security.headers.not_a_check",
            "best_practices.vulnerable_libraries.left-pad",
            "seo.schema.jsonld_parse_error",
            "seo.schema.schema_rule_missing_unknown_feature_x",
            "unknown.whatever",
        ] {
            assert_eq!(finding_help_url(id), None, "{id}");
        }
    }
}
