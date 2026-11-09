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
    let unaddressable: Vec<&String> = active
        .iter()
        .filter(|id| !is_addressable_rule_id(id))
        .collect();
    assert!(
        unaddressable.is_empty(),
        "rule ids that cannot be part of a page URL: {unaddressable:?}"
    );
}

/// The URL every finding carries as `help_url` (`rule_doc_url`) names the file
/// the site builds the page from: the id below `rules/`.
#[test]
fn every_help_url_points_at_a_generated_page() {
    let pages = generate();
    for rule in auditmysite::taxonomy::rules::RULES {
        let url = rule_doc_url(rule.id);
        let path = url
            .strip_prefix(RULE_DOCS_BASE_URL)
            .and_then(|p| p.strip_suffix('/'))
            .unwrap_or_else(|| panic!("{url} is not below {RULE_DOCS_BASE_URL}"));
        assert_eq!(path, rule.id);
        assert!(pages.contains_key(&page_file(path)), "no page for {url}");
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
