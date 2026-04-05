//! Output Format Tests
//!
//! Tests for JSON report generation using NormalizedReport

use chrono::{DateTime, Utc};
use jsonschema::JSONSchema;
use std::path::PathBuf;

use auditmysite::audit::{normalize, AuditReport, BatchReport};
use auditmysite::cli::WcagLevel;
use auditmysite::output::{format_json_batch, JsonReport};
use auditmysite::studio::{StudioAuditResponse, StudioHistoryEntry};
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

// ─── Studio Contract Tests ────────────────────────────────────────

#[test]
fn test_studio_response_matches_schema() {
    let report = create_test_report();
    let normalized = normalize(&report);
    let response =
        StudioAuditResponse::from_normalized(&normalized, &report, "{}".to_string());

    let json_value = serde_json::to_value(&response).expect("serialization must work");
    assert_matches_schema(&json_value, "studio-contract.schema.json");
}

#[test]
fn test_studio_history_entry_roundtrip() {
    let report = create_test_report();
    let normalized = normalize(&report);
    let response =
        StudioAuditResponse::from_normalized(&normalized, &report, "{}".to_string());
    let entry = StudioHistoryEntry::from_response(&response);

    // Serialize and deserialize must round-trip
    let json = serde_json::to_string(&entry).expect("serialization must work");
    let parsed: StudioHistoryEntry =
        serde_json::from_str(&json).expect("deserialization must work");

    assert_eq!(parsed.url, entry.url);
    assert_eq!(parsed.accessibility_score, entry.accessibility_score);
    assert_eq!(parsed.risk_level, entry.risk_level);
    assert_eq!(parsed.module_scores.len(), entry.module_scores.len());

    // Required fields must be present in JSON
    let obj: serde_json::Value = serde_json::from_str(&json).unwrap();
    let map = obj.as_object().unwrap();
    for field in &[
        "id", "url", "timestamp", "accessibility_score", "overall_score",
        "grade", "certificate", "risk_level", "total_issues", "critical_issues",
        "high_issues", "execution_time_ms", "module_scores",
    ] {
        assert!(map.contains_key(*field), "Missing field: {}", field);
    }
}

#[test]
fn test_studio_response_has_all_expected_fields() {
    let report = create_test_report();
    let normalized = normalize(&report);
    let response =
        StudioAuditResponse::from_normalized(&normalized, &report, "{}".to_string());

    let json_value = serde_json::to_value(&response).expect("serialization must work");
    let obj = json_value.as_object().unwrap();

    // Every required field must be present
    let required = vec![
        "url", "timestamp", "accessibility_score", "overall_score", "grade",
        "certificate", "risk_level", "risk_summary", "legal_flags", "blocking_issues",
        "critical_issues", "high_issues", "medium_issues", "low_issues", "total_issues",
        "module_scores", "findings", "nodes_analyzed", "execution_time_ms", "json_report",
    ];
    for field in required {
        assert!(obj.contains_key(field), "Missing required field: {}", field);
    }
}
