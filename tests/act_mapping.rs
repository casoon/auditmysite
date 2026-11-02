//! Guard for `docs/act-mapping.json` (#836): the machine-readable mapping of
//! internal rule ids onto ACT rule ids. Schema and semantics: `docs/ACT_COVERAGE.md`.

mod common;

use common::rule_inventory::canonical_rule_ids;
use serde_json::Value;
use std::collections::BTreeSet;

fn mapping() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/act-mapping.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("docs/act-mapping.json is valid JSON")
}

fn strings(rule: &Value, key: &str) -> Vec<String> {
    rule[key]
        .as_array()
        .unwrap_or_else(|| panic!("{}: `{key}` must be an array", rule["act_id"]))
        .iter()
        .map(|v| v.as_str().expect("string entry").to_string())
        .collect()
}

#[test]
fn every_mapped_internal_rule_id_exists() {
    let known = canonical_rule_ids();
    let doc = mapping();
    let mut unknown = Vec::new();
    for rule in doc["rules"].as_array().expect("rules array") {
        for key in ["internal_rules", "related_rules"] {
            for id in strings(rule, key) {
                if !known.contains(&id) {
                    unknown.push(format!("{} -> {id}", rule["act_id"]));
                }
            }
        }
    }
    assert!(
        unknown.is_empty(),
        "act-mapping.json names rule ids that no detector produces: {unknown:?}"
    );
}

#[test]
fn mapping_is_structurally_consistent() {
    let doc = mapping();
    assert_eq!(doc["schema_version"], 1);
    let mut seen = BTreeSet::new();
    for rule in doc["rules"].as_array().expect("rules array") {
        let id = rule["act_id"].as_str().expect("act_id").to_string();
        assert!(seen.insert(id.clone()), "duplicate ACT id {id}");
        let coverage = rule["coverage"].as_str().expect("coverage");
        let internal = strings(rule, "internal_rules");
        match coverage {
            "full" | "partial" => {
                assert!(
                    !internal.is_empty(),
                    "{id}: {coverage} needs internal_rules"
                )
            }
            "none" | "unclear" => {
                assert!(
                    internal.is_empty(),
                    "{id}: {coverage} must not list internal_rules"
                )
            }
            other => panic!("{id}: unknown coverage `{other}`"),
        }
        assert_eq!(
            rule.get("gap").is_some(),
            coverage == "none",
            "{id}: `gap` is set exactly for coverage none"
        );
        if rule.get("starter").is_some() {
            assert_eq!(
                coverage, "full",
                "{id}: starter rules must be fully covered"
            );
            assert_eq!(rule["status"], "active", "{id}: starter rule is deprecated");
        }
    }
}
