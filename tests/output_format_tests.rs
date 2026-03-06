//! Output Format Tests
//!
//! Tests for JSON report generation

use auditmysite::audit::AuditReport;
use auditmysite::cli::WcagLevel;
use auditmysite::output::JsonReport;
use auditmysite::wcag::{Severity, Violation, WcagResults};

fn create_test_report() -> AuditReport {
    let mut wcag_results = WcagResults::new();

    let violation = Violation::new(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        Severity::Serious,
        "Image missing alt attribute",
        "node-123",
    )
    .with_role(Some("image".to_string()))
    .with_fix("Add alt attribute to the image")
    .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/non-text-content");

    wcag_results.violations.push(violation);
    wcag_results.passes = 42;

    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        wcag_results,
        1500,
    )
}

#[test]
fn test_json_report_generation() {
    let report = create_test_report();
    let json_report = JsonReport::new(report.clone(), "AA", 1500);

    let json_str = json_report.to_json(false).expect("JSON generation failed");

    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse generated JSON");

    assert_eq!(parsed["report"]["url"], "https://example.com");
    assert_eq!(parsed["metadata"]["wcag_level"], "AA");
    assert!(parsed["report"]["wcag_results"]["violations"].is_array());
    assert_eq!(
        parsed["report"]["wcag_results"]["violations"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn test_json_report_pretty_print() {
    let report = create_test_report();
    let json_report = JsonReport::new(report, "AA", 1500);

    let pretty = json_report.to_json(true).expect("Pretty JSON failed");
    let compact = json_report.to_json(false).expect("Compact JSON failed");

    assert!(pretty.contains('\n'));
    assert!(!compact.contains('\n'));

    let pretty_parsed: serde_json::Value = serde_json::from_str(&pretty).unwrap();
    let compact_parsed: serde_json::Value = serde_json::from_str(&compact).unwrap();
    assert_eq!(pretty_parsed, compact_parsed);
}

#[test]
fn test_report_score_calculation() {
    let clean_report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    assert!(clean_report.score >= 90.0);

    let report_with_issues = create_test_report();
    assert!(report_with_issues.score < clean_report.score);
}
