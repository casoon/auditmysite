//! Canonical inventories of the enumerable checks in three non-WCAG modules
//! (#558): security headers, `best_practices::vulnerable_libs`, and
//! the `SchemaFeature` catalog (from `web_checks::structured_data`).
//!
//! Unlike `rule_inventory.rs`'s WCAG scan, none of these three modules share
//! a single uniform data shape like `RuleMetadata` — each function below
//! documents the specific source-code pattern it depends on, so a future
//! refactor that breaks the assumption is visible here rather than silently
//! under-counting.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn security_mod_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("security")
        .join("mod.rs")
}

/// Read a source file and drop everything from its `#[cfg(test)]` module
/// onward — several of these files mix production code and test fixtures
/// (fake `SecurityIssue`/etc. literals) in one file, and the test module's
/// own literals must not be double-counted as real checks.
fn production_source(path: &PathBuf) -> String {
    let full = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    match full.find("\n#[cfg(test)]") {
        Some(idx) => full[..idx].to_string(),
        None => full,
    }
}

/// Extract the quoted string value of the first `field_name: "..."` inside
/// `block` — mirrors `rule_inventory.rs`'s helper of the same name.
fn extract_field_str(block: &str, field_name: &str) -> Option<String> {
    let marker = format!("{field_name}:");
    let start = block.find(&marker)? + marker.len();
    let rest = &block[start..];
    let quote_start = rest.find('"')?;
    let after_quote = &rest[quote_start + 1..];
    let quote_end = after_quote.find('"')?;
    Some(after_quote[..quote_end].to_string())
}

/// The canonical, deduplicated list of every `"{header}:{issue_type}"` id
/// `src/security/mod.rs` can currently produce.
///
/// Derived from `SecurityIssueKind::ALL` in the production code rather than
/// scanned out of the source text: the enum is exhaustive by construction, so
/// a renamed or added check cannot silently fall out of the inventory.
pub fn canonical_security_check_ids() -> BTreeSet<String> {
    auditmysite::security::all_security_check_ids()
        .into_iter()
        .collect()
}

/// The canonical, deduplicated list of every library name
/// `best_practices::vulnerable_libs` checks against its advisory table.
///
/// The scope is the public `CHECKED_LIBRARIES` list since the versioned
/// advisory table (#773), so no source scan is needed here.
pub fn canonical_vulnerable_library_ids() -> BTreeSet<String> {
    auditmysite::best_practices::CHECKED_LIBRARIES
        .iter()
        .map(|name| name.to_string())
        .collect()
}

/// The canonical, deduplicated list of every `SchemaFeature` key
/// `seo::schema_rules` can currently produce.
///
/// The catalog lives in `web_checks::structured_data` since web-checks 0.4;
/// `SchemaFeature::ALL` is its own list, so no source scan is needed here.
pub fn canonical_schema_feature_ids() -> BTreeSet<String> {
    auditmysite::seo::schema_rules::SchemaFeature::ALL
        .iter()
        .map(|feature| feature.key().to_string())
        .collect()
}
