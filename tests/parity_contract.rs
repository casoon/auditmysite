use auditmysite::taxonomy::rules::RULES;
use auditmysite::wcag::coverage::{coverage_stats, MANUAL_REVIEW_CRITERIA};
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
        Some(MANUAL_REVIEW_CRITERIA.len() as u64),
        "landing-page manual-review count must match the manifest"
    );
}

#[test]
fn parity_axe_mappings_resolve_to_taxonomy_rules() {
    let data = contract();
    let mappings = data["axe_rule_mappings"]
        .as_object()
        .expect("axe_rule_mappings object");

    for (axe_id, patterns) in mappings {
        let patterns = patterns.as_array().expect("mapping patterns array");
        assert!(
            RULES.iter().any(|rule| {
                rule.axe_id == Some(axe_id.as_str())
                    || patterns.iter().any(|pattern| {
                        let pattern = pattern.as_str().expect("mapping pattern string");
                        rule.id.replace('.', "_").contains(pattern)
                    })
            }),
            "parity mapping for {axe_id} must resolve to a taxonomy axe id or rule-id pattern"
        );
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
