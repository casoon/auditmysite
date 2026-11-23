//! CDP-free integration tests using pre-built AXTrees.
//!
//! These tests run the WCAG engine against manually constructed AXTrees and do
//! NOT require Chrome. They exercise the same rule paths as the full pipeline.
//!
//! Run with: cargo test --test integration_mocked

use auditmysite::browser::mock::MockAXSession;
use auditmysite::wcag::{check_all, check_all_with_config, RuleFilterConfig};
use auditmysite::WcagLevel;

fn trap_notes(results: &auditmysite::wcag::WcagResults) -> usize {
    results
        .warnings
        .iter()
        .filter(|v| v.rule == "2.1.2")
        .count()
}

#[test]
fn valid_page_has_no_keyboard_trap_note() {
    let session = MockAXSession::minimal_valid();
    let results = check_all(&session.tree, WcagLevel::AA);
    assert_eq!(trap_notes(&results), 0, "No modal dialog, no 2.1.2 note");
}

#[test]
fn modal_dialog_triggers_review_note() {
    let session = MockAXSession::modal_dialog();
    let results = check_all(&session.tree, WcagLevel::A);
    assert_eq!(
        trap_notes(&results),
        1,
        "Expected a 2.1.2 note for the dialog"
    );
}

#[test]
fn disabled_rule_suppresses_finding() {
    let filter = RuleFilterConfig {
        disabled_rules: vec!["keyboard-trap".to_string()],
        enabled_only_rules: vec![],
    };
    let session = MockAXSession::modal_dialog();
    let results = check_all_with_config(&session.tree, WcagLevel::A, &filter);
    assert_eq!(
        trap_notes(&results),
        0,
        "2.1.2 should be suppressed when keyboard-trap is disabled"
    );
}

#[test]
fn enabled_only_runs_just_that_rule() {
    let filter = RuleFilterConfig {
        disabled_rules: vec![],
        enabled_only_rules: vec!["keyboard-trap".to_string()],
    };
    let session = MockAXSession::modal_dialog();
    let results = check_all_with_config(&session.tree, WcagLevel::AA, &filter);
    assert_eq!(trap_notes(&results), 1);
    assert!(
        results
            .rule_outcomes
            .iter()
            .all(|o| o.rule_id == "keyboard-trap"),
        "Only keyboard-trap should run"
    );
}
