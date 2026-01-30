//! Output Format Tests
//!
//! Tests for JSON, HTML, and Markdown report generation

use auditmysite::audit::AuditReport;
use auditmysite::cli::WcagLevel;
use auditmysite::output::{format_html, JsonReport};
use auditmysite::wcag::{Severity, Violation, WcagResults};

fn create_test_report() -> AuditReport {
    let mut wcag_results = WcagResults::new();

    // Add a sample violation using the builder pattern
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

    AuditReport::new("https://example.com".to_string(), wcag_results, 1500)
}

#[test]
fn test_json_report_generation() {
    let report = create_test_report();
    let json_report = JsonReport::new(report.clone(), "AA", 1500);

    let json_str = json_report.to_json(false).expect("JSON generation failed");

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse generated JSON");

    // Check key fields - JsonReport wraps report in a 'report' field
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

    // Pretty should have newlines, compact should not
    assert!(pretty.contains('\n'));
    assert!(!compact.contains('\n'));

    // Both should parse to equivalent JSON
    let pretty_parsed: serde_json::Value = serde_json::from_str(&pretty).unwrap();
    let compact_parsed: serde_json::Value = serde_json::from_str(&compact).unwrap();
    assert_eq!(pretty_parsed, compact_parsed);
}

#[test]
fn test_html_report_generation() {
    let report = create_test_report();
    let html = format_html(&report, "AA").expect("HTML generation failed");

    // Check structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("</html>"));

    // Check content
    assert!(html.contains("example.com"));
    assert!(html.contains("WCAG AA"));
    assert!(html.contains("1.1.1"));
    assert!(html.contains("Non-text Content"));
}

#[test]
fn test_html_escaping() {
    let mut wcag_results = WcagResults::new();

    // Add a violation with XSS attempt in message
    let violation = Violation::new(
        "1.1.1",
        "Test <script>alert('xss')</script>",
        WcagLevel::A,
        Severity::Critical,
        "<img src=x onerror=alert('xss')>",
        "node-xss",
    )
    .with_name(Some("Test & \"quotes\" 'apostrophes'".to_string()))
    .with_fix("Fix <script> injection");

    wcag_results.violations.push(violation);

    let report = AuditReport::new("https://example.com/test".to_string(), wcag_results, 100);

    let html = format_html(&report, "AA").expect("HTML generation failed");

    // Verify XSS in rule_name is escaped (the rule_name contains <script>)
    // The html_escape function should convert < to &lt;
    assert!(html.contains("&lt;script&gt;alert"));

    // Verify XSS in message is escaped
    assert!(html.contains("&lt;img src=x"));

    // Verify special chars are escaped
    assert!(html.contains("&amp;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn test_html_report_with_no_violations() {
    let wcag_results = WcagResults::new();
    let report = AuditReport::new("https://perfect-site.com".to_string(), wcag_results, 500);

    let html = format_html(&report, "AAA").expect("HTML generation failed");

    assert!(html.contains("All Checks Passed"));
    assert!(html.contains("No accessibility violations were found"));
}

#[test]
fn test_report_score_calculation() {
    // Report with no violations should have high score
    let clean_report = AuditReport::new("https://example.com".to_string(), WcagResults::new(), 100);
    assert!(clean_report.score >= 90.0);

    // Report with violations should have lower score
    let report_with_issues = create_test_report();
    assert!(report_with_issues.score < clean_report.score);
}
