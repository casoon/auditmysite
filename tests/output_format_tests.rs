//! Output Format Tests
//!
//! Tests for JSON report generation using NormalizedReport

use chrono::{DateTime, Utc};
use jsonschema::JSONSchema;
use std::path::PathBuf;

use auditmysite::audit::{normalize, AuditReport, BatchReport};
use auditmysite::cli::WcagLevel;
use auditmysite::output::{format_json_batch, JsonReport};
use auditmysite::wcag::{Severity, Violation, WcagResults};

fn create_test_report() -> AuditReport {
    let mut wcag_results = WcagResults::new();

    let violation = Violation::new(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        Severity::High,
        "Image missing alt attribute",
        "node-123",
    )
    .with_role(Some("image".to_string()))
    .with_fix("Add alt attribute to the image")
    .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/non-text-content");

    wcag_results.violations.push(violation);
    wcag_results.passes = 42;

    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        wcag_results,
        1500,
    );
    report.timestamp = fixed_timestamp();
    report
}

fn fixed_timestamp() -> DateTime<Utc> {
    "2026-01-15T12:00:00Z".parse().unwrap()
}

fn load_schema(name: &str) -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read schema {}: {}", path.display(), e));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("Failed to parse schema {}: {}", path.display(), e))
}

fn assert_matches_schema(instance: &serde_json::Value, schema_name: &str) {
    let schema_json = load_schema(schema_name);
    let compiled = JSONSchema::compile(&schema_json)
        .unwrap_or_else(|e| panic!("Failed to compile schema {}: {}", schema_name, e));

    let validation = compiled.validate(instance);
    if let Err(errors) = validation {
        let messages: Vec<String> = errors.map(|err| err.to_string()).collect();
        panic!(
            "JSON did not match schema {}:\n{}",
            schema_name,
            messages.join("\n")
        );
    }
}

#[test]
fn test_json_report_generation() {
    let report = create_test_report();
    let normalized = normalize(&report);
    let json_report = JsonReport::from_normalized(&normalized, &report);

    let json_str = json_report.to_json(false).expect("JSON generation failed");

    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse generated JSON");

    assert_eq!(parsed["report"]["url"], "https://example.com");
    assert_eq!(parsed["metadata"]["wcag_level"], "AA");
    assert_eq!(parsed["metadata"]["timestamp"], "2026-01-15T12:00:00Z");
    // Findings are now grouped by rule, with taxonomy fields
    assert!(parsed["report"]["findings"].is_array());
    let findings = parsed["report"]["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0]["dimension"].is_string());
    assert!(findings[0]["subcategory"].is_string());
    assert!(findings[0]["issue_class"].is_string());
    assert_matches_schema(&parsed, "json-report.schema.json");
}

#[test]
fn test_json_report_pretty_print() {
    let report = create_test_report();
    let normalized = normalize(&report);
    let json_report = JsonReport::from_normalized(&normalized, &report);

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

#[test]
fn test_batch_json_report_generation() {
    let report = create_test_report();
    let batch = BatchReport::from_reports(vec![report], vec![], 1500);

    let json = format_json_batch(&batch, false).expect("Batch JSON generation failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid batch JSON");

    assert_eq!(parsed["metadata"]["timestamp"], "2026-01-15T12:00:00Z");
    assert_eq!(parsed["summary"]["total_urls"], 1);
    assert_eq!(parsed["reports"][0]["url"], "https://example.com");
    assert!(parsed["reports"][0]["findings"].is_array());
    assert_matches_schema(&parsed, "json-batch-report.schema.json");
}
