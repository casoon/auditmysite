//! Per-rule documentation pages (#929). Browser-free, runs in every
//! `cargo test`.
//!
//! The pages are generated from the rule catalogue, the rule contracts and the
//! detection corpus (`tests/common/rule_docs.rs`) and committed under
//! `site/src/data/rules/`, because the Pages workflow builds the site without
//! a Rust toolchain. These tests keep the committed files in step with their
//! sources and every `help_url` pointing at a page that exists.
//!
//! After changing a rule, contract or corpus case:
//! `UPDATE_RULE_DOCS=1 cargo test --test rule_docs`

mod common;

use auditmysite::taxonomy::{is_addressable_rule_id, rule_doc_url, RULE_DOCS_BASE_URL};
use common::rule_docs::{generate, page_file, repo_root, OUTPUT_DIR};
use std::collections::BTreeSet;
use std::path::Path;

fn committed_files(dir: &Path, prefix: &str, out: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
        if path.is_dir() {
            committed_files(&path, &format!("{name}/"), out);
        } else {
            out.insert(name);
        }
    }
}

#[test]
fn generated_rule_pages_are_up_to_date() {
    let expected = generate();
    let dir = repo_root().join(OUTPUT_DIR);

    if std::env::var_os("UPDATE_RULE_DOCS").is_some() {
        let mut stale = BTreeSet::new();
        committed_files(&dir, "", &mut stale);
        for name in stale.iter().filter(|n| !expected.contains_key(*n)) {
            std::fs::remove_file(dir.join(name)).unwrap();
        }
        for (name, content) in &expected {
            let path = dir.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            if std::fs::read_to_string(&path).ok().as_deref() != Some(content) {
                std::fs::write(&path, content).unwrap();
            }
        }
        return;
    }

    let mut committed = BTreeSet::new();
    committed_files(&dir, "", &mut committed);
    let mut problems = Vec::new();
    for name in expected.keys().filter(|n| !committed.contains(*n)) {
        problems.push(format!("missing: {name}"));
    }
    for name in committed.iter().filter(|n| !expected.contains_key(*n)) {
        problems.push(format!("stale, no rule behind it: {name}"));
    }
    for (name, content) in &expected {
        if committed.contains(name) && std::fs::read_to_string(dir.join(name)).unwrap() != *content
        {
            problems.push(format!("out of date: {name}"));
        }
    }
    assert!(
        problems.is_empty(),
        "{OUTPUT_DIR} does not match the rule catalogue, contracts and corpus; \
         regenerate with `UPDATE_RULE_DOCS=1 cargo test --test rule_docs`:\n{}",
        problems.join("\n")
    );
}

/// The finding ids each family reports that have a page, one sample per page
/// (schema findings carry the property after the feature key).
fn documented_finding_ids() -> Vec<String> {
    let mut ids: Vec<String> = auditmysite::taxonomy::rules::RULES
        .iter()
        .map(|r| r.id.to_string())
        .collect();
    ids.extend(
        common::nonwcag_rule_inventory::canonical_security_check_ids()
            .iter()
            .filter_map(|c| {
                c.split_once(':')
                    .map(|(_, t)| format!("security.headers.{t}"))
            }),
    );
    ids.extend(
        common::nonwcag_rule_inventory::canonical_schema_feature_ids()
            .iter()
            .map(|k| format!("seo.schema.schema_rule_missing_{k}_offers")),
    );
    ids.extend(
        common::nonwcag_rule_inventory::canonical_vulnerable_library_ids()
            .iter()
            .map(|n| format!("best_practices.vulnerable_libraries.{}", n.to_lowercase())),
    );
    ids
}

#[test]
fn every_active_rule_id_has_an_addressable_page() {
    let pages = generate();
    let active = common::rule_inventory::canonical_rule_ids();
    let missing: Vec<&String> = active
        .iter()
        .filter(|id| !pages.contains_key(&page_file(id)))
        .collect();
    assert!(
        missing.is_empty(),
        "active rule ids without a page: {missing:?}"
    );
    let extra: Vec<&str> = auditmysite::taxonomy::rules::RULES
        .iter()
        .map(|r| r.id)
        .filter(|id| !active.contains(*id))
        .collect();
    assert!(extra.is_empty(), "pages for ids no rule reports: {extra:?}");

    // The other families: every contracted check lands on a page.
    let mut non_wcag = Vec::new();
    non_wcag.extend(
        common::nonwcag_rule_inventory::canonical_security_check_ids()
            .iter()
            .map(|c| auditmysite::audit::rule_docs::security_page_id(c).expect("header:type")),
    );
    non_wcag.extend(
        common::nonwcag_rule_inventory::canonical_schema_feature_ids()
            .iter()
            .map(|k| auditmysite::audit::rule_docs::schema_page_id(k)),
    );
    non_wcag.extend(
        common::nonwcag_rule_inventory::canonical_vulnerable_library_ids()
            .iter()
            .map(|n| auditmysite::audit::rule_docs::library_page_id(n)),
    );
    let missing: Vec<&String> = non_wcag
        .iter()
        .filter(|id| !pages.contains_key(&page_file(id)))
        .collect();
    assert!(missing.is_empty(), "checks without a page: {missing:?}");

    // No page that no active check or rule explains.
    let expected: BTreeSet<String> = active.iter().cloned().chain(non_wcag).collect();
    let orphans: Vec<&String> = pages
        .keys()
        .filter(|f| *f != "index.json")
        .filter(|f| !expected.contains(f.strip_suffix(".json").unwrap()))
        .collect();
    assert!(
        orphans.is_empty(),
        "pages without an active check: {orphans:?}"
    );

    let unaddressable: Vec<&String> = expected
        .iter()
        .filter(|id| !is_addressable_rule_id(id))
        .collect();
    assert!(
        unaddressable.is_empty(),
        "page ids that cannot be part of a page URL: {unaddressable:?}"
    );
}

/// The `help_url` a finding gets (`audit::rule_docs::finding_help_url`) names
/// the file the site builds the page from: the page id below `rules/`.
#[test]
fn every_help_url_points_at_a_generated_page() {
    let pages = generate();
    for finding_id in documented_finding_ids() {
        let url = auditmysite::audit::rule_docs::finding_help_url(&finding_id)
            .unwrap_or_else(|| panic!("{finding_id} has no help_url"));
        let path = url
            .strip_prefix(RULE_DOCS_BASE_URL)
            .and_then(|p| p.strip_suffix('/'))
            .unwrap_or_else(|| panic!("{url} is not below {RULE_DOCS_BASE_URL}"));
        assert_eq!(url, rule_doc_url(path));
        assert!(
            pages.contains_key(&page_file(path)),
            "no page for {url} (finding {finding_id})"
        );
    }
}

/// A section is either absent or has text: the generator leaves out what the
/// contract or corpus does not carry instead of writing an empty heading.
#[test]
fn pages_carry_no_empty_sections() {
    fn check(name: &str, path: &str, v: &serde_json::Value) {
        match v {
            serde_json::Value::String(s) => {
                assert!(!s.trim().is_empty(), "{name}: {path} is empty")
            }
            serde_json::Value::Object(m) => {
                for (k, v) in m {
                    check(name, &format!("{path}.{k}"), v)
                }
            }
            _ => {}
        }
    }
    for (name, content) in generate() {
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        check(&name, "", &json);
    }
}
