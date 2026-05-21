//! CDP-free integration tests using pre-built AXTrees.
//!
//! These tests run the WCAG engine against manually constructed AXTrees and do
//! NOT require Chrome. They exercise the same rule paths as the full pipeline.
//!
//! Run with: cargo test --test integration_mocked

use auditmysite::browser::mock::MockAXSession;
use auditmysite::wcag::{check_all, check_all_with_config, RuleFilterConfig};
use auditmysite::WcagLevel;

#[test]
fn valid_page_has_no_image_alt_violation() {
    let session = MockAXSession::minimal_valid();
    let results = check_all(&session.tree, WcagLevel::AA);
    assert!(
        results.violations.iter().all(|v| v.rule != "1.1.1"),
        "No 1.1.1 violation expected on a page without images"
    );
}

#[test]
fn image_missing_alt_triggers_violation() {
    let session = MockAXSession::image_missing_alt();
    let results = check_all(&session.tree, WcagLevel::A);
    assert!(
        results.violations.iter().any(|v| v.rule == "1.1.1"),
        "Expected 1.1.1 violation for image without alt text"
    );
}

#[test]
fn disabled_rule_suppresses_violation() {
    let filter = RuleFilterConfig {
        disabled_rules: vec!["image-alt".to_string()],
        enabled_only_rules: vec![],
    };
    let session = MockAXSession::image_missing_alt();
    let results = check_all_with_config(&session.tree, WcagLevel::A, &filter);
    assert!(
        results.violations.iter().all(|v| v.rule != "1.1.1"),
        "1.1.1 should be suppressed when image-alt rule is disabled"
    );
}

#[test]
fn enabled_only_runs_just_that_rule() {
    let filter = RuleFilterConfig {
        disabled_rules: vec![],
        enabled_only_rules: vec!["image-alt".to_string()],
    };
    let session = MockAXSession::image_missing_alt();
    let results = check_all_with_config(&session.tree, WcagLevel::A, &filter);
    // Only image-alt runs, so only 1.1.1 violations are possible.
    assert!(
        results.violations.iter().all(|v| v.rule == "1.1.1"),
        "Only image-alt rule should produce violations"
    );
}
