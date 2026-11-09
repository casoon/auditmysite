use auditmysite::taxonomy::rules::RULES;
use auditmysite::wcag::coverage::{coverage_stats, manual_review_criteria};
use auditmysite::wcag::en301549::{EN301549_MAPPING_VERSION, EN301549_WEB_CLAUSES};
use serde_json::Value;

fn contract() -> Value {
    serde_json::from_str(include_str!("../docs/PARITY_CONTRACT.jsonc"))
        .expect("parity contract must be valid JSON")
}

#[test]
fn frozen_landing_page_numbers_match_wcag_coverage_manifest() {
    let data = contract();
    let frozen = data["frozen_numbers"]
        .as_object()
        .expect("frozen_numbers object");
    let (automated, total) = coverage_stats();

    assert_eq!(
        frozen["wcag_aa_total_criteria"].as_u64(),
        Some(total as u64),
        "landing-page WCAG AA total must stay explicit"
    );
    assert_eq!(
        frozen["automated_wcag_aa_criteria"].as_u64(),
        Some(automated as u64),
        "landing-page automated WCAG AA count must match coverage_stats"
    );
    assert_eq!(
        frozen["manual_review_criteria"].as_u64(),
        Some(manual_review_criteria().len() as u64),
        "landing-page manual-review count must match the manifest"
    );
}

#[test]
fn frozen_en301549_numbers_match_mapping_table() {
    let data = contract();
    let frozen = data["frozen_numbers"]
        .as_object()
        .expect("frozen_numbers object");

    assert_eq!(
        frozen["en301549_web_clauses"].as_u64(),
        Some(EN301549_WEB_CLAUSES.len() as u64),
        "frozen EN 301 549 clause count must match the mapping table — bump deliberately on edit"
    );
    assert_eq!(
        frozen["en301549_mapping_version"].as_u64(),
        Some(EN301549_MAPPING_VERSION as u64),
        "frozen EN 301 549 mapping_version must match the table's own version constant"
    );
}

#[test]
fn parity_axe_mappings_resolve_to_taxonomy_rules() {
    let data = contract();
    let mappings = data["axe_rule_mappings"]
        .as_object()
        .expect("axe_rule_mappings object");

    for (axe_id, rule_ids) in mappings {
        let rule_ids = rule_ids.as_array().expect("mapping rule id array");
        assert!(!rule_ids.is_empty(), "parity mapping for {axe_id} is empty");
        for rule_id in rule_ids {
            let rule_id = rule_id.as_str().expect("mapping rule id string");
            assert!(
                RULES.iter().any(|rule| rule.id == rule_id),
                "parity mapping for {axe_id} names {rule_id}, which is no taxonomy rule id"
            );
        }
    }
}

#[test]
fn stable_parity_fixture_expectations_are_mapped() {
    let data = contract();
    let expected = data["frozen_numbers"]["stable_parity_fixture_expected_axe_ids"]
        .as_array()
        .expect("expected axe ids array");
    let mappings = data["axe_rule_mappings"]
        .as_object()
        .expect("axe_rule_mappings object");

    for axe_id in expected {
        let axe_id = axe_id.as_str().expect("axe id string");
        assert!(
            mappings.contains_key(axe_id),
            "stable parity fixture expectation {axe_id} must be represented in axe_rule_mappings"
        );
    }
}
