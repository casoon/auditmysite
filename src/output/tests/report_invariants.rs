//! Invariant tests for the report model — each test pins a known-correct
//! property so regressions are caught immediately rather than via manual
//! release review.

use crate::audit::normalized::normalize;
use crate::audit::AuditReport;
use crate::cli::WcagLevel;
use crate::output::json::UnifiedReport;
use crate::taxonomy::Severity;
use crate::wcag::{Violation, WcagResults};

fn make_report(wcag_results: WcagResults) -> AuditReport {
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        wcag_results,
        100,
    )
}

fn make_security_from_headers(
    headers: crate::security::SecurityHeaders,
) -> crate::security::SecurityAnalysis {
    use crate::security::{
        calculate_security_score, generate_security_issues, SecurityAnalysis, SslInfo,
    };

    // No HSTS bonus so the Low-penalty from a missing header is not cancelled out.
    let ssl = SslInfo {
        https: true,
        valid_certificate: true,
        has_hsts: false,
        ..Default::default()
    };
    let issues = generate_security_issues(&headers, true);
    let score = calculate_security_score(&headers, &ssl, &issues);
    SecurityAnalysis {
        score,
        grade: String::new(),
        headers,
        ssl,
        issues,
        recommendations: vec![],
        protection: Default::default(),
    }
}

// ─── Serialization invariants ─────────────────────────────────────────────────

#[test]
fn ux_score_propagated_to_summary() {
    use crate::accessibility::AXTree;
    use crate::ux::analyze_ux;

    let mut report = make_report(WcagResults::new());
    report.ux = Some(analyze_ux(&AXTree::new()));
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);

    assert!(
        unified.summary.ux_score.is_some(),
        "summary.ux_score must be Some when UX module is active"
    );
    let ux_entry = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "UX")
        .unwrap();
    assert_eq!(unified.summary.ux_score, Some(ux_entry.score));
}

#[test]
fn journey_score_propagated_to_summary() {
    use crate::accessibility::AXTree;
    use crate::journey::analyze_journey;

    let mut report = make_report(WcagResults::new());
    report.journey = Some(analyze_journey(&AXTree::new()));
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);

    assert!(
        unified.summary.journey_score.is_some(),
        "summary.journey_score must be Some when Journey module is active"
    );
    let journey_entry = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Journey")
        .unwrap();
    assert_eq!(unified.summary.journey_score, Some(journey_entry.score));
}

// ─── Performance measurement_warnings serialization ──────────────────────────

#[test]
fn measurement_warnings_present_in_json_when_set() {
    use crate::audit::PerformanceResults;
    use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

    let mut report = make_report(WcagResults::new());
    report = report.with_performance(PerformanceResults {
        vitals: WebVitals::default(),
        score: PerformanceScore {
            overall: 72,
            grade: PerformanceGrade::Silver,
            lcp_score: None,
            fcp_score: None,
            cls_score: None,
            interactivity_score: None,
            si_score: None,
            metrics_available: 0,
            size_penalty: None,
            js_penalty: None,
            request_penalty: None,
            dom_penalty: None,
            is_capped: None,
        },
        render_blocking: None,
        content_weight: None,
        third_party: None,
        critical_chain: None,
        minification: None,
        animations: None,
        coverage: None,
        measurement_warnings: vec!["tbt_zero_heavy_page".to_string()],
    });
    let normalized = crate::audit::normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(false).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let warnings = &json["pages"][0]["detail"]["modules"]["performance"]["measurement_warnings"];
    assert!(
        warnings.is_array(),
        "measurement_warnings must be an array in JSON when set"
    );
    assert_eq!(
        warnings[0].as_str(),
        Some("tbt_zero_heavy_page"),
        "first warning key must be 'tbt_zero_heavy_page'"
    );
}

// ─── Accessibility score invariants ──────────────────────────────────────────

#[test]
fn critical_violation_caps_accessibility_score() {
    use crate::audit::AccessibilityScorer;

    let violations = vec![Violation::new(
        "4.1.2",
        "Name, Role, Value",
        WcagLevel::A,
        Severity::Critical,
        "Button has no accessible name",
        "btn1",
    )];
    let score = AccessibilityScorer::calculate_score(&violations);
    assert!(
        score <= 49.0,
        "accessibility_score must be ≤ 49 when critical violations exist, got {}",
        score
    );
}

#[test]
fn severity_counts_low_nonzero_with_low_finding() {
    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "2.4.8",
        "Location",
        WcagLevel::AAA,
        Severity::Low,
        "No location breadcrumb",
        "n1",
    ));

    let report = make_report(results);
    let normalized = normalize(&report);

    assert!(
        normalized.severity_counts.low > 0,
        "severity_counts.low must be > 0 when a Low-severity rule fires, got {}",
        normalized.severity_counts.low
    );
}

#[test]
fn severity_counts_low_not_upgraded_by_taxonomy_floor() {
    // Rule 1.3.1 has taxonomy.severity = High, but list_structure.rs deliberately
    // generates Severity::Low for minor cases (empty list). Before #288 the
    // taxonomy floor max(Low, High) = High silently upgraded these to High,
    // making severity_counts.low always 0 in practice.
    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "1.3.1",
        "Info and Relationships",
        WcagLevel::A,
        Severity::Low,
        "List element has no list items",
        "n1",
    ));

    let report = make_report(results);
    let normalized = normalize(&report);

    assert_eq!(
        normalized.severity_counts.low, 1,
        "Low violation of a Medium/High taxonomy rule must not be upgraded (#288)"
    );
    assert_eq!(
        normalized.severity_counts.high, 0,
        "taxonomy floor must not silently upgrade Low to High (#288)"
    );
}

// ─── Security score invariants ────────────────────────────────────────────────

#[test]
fn security_score_below_100_without_permissions_policy() {
    use crate::security::SecurityHeaders;

    let headers = SecurityHeaders {
        content_security_policy: Some("default-src 'self'".to_string()),
        x_content_type_options: Some("nosniff".to_string()),
        x_frame_options: Some("DENY".to_string()),
        referrer_policy: Some("strict-origin".to_string()),
        strict_transport_security: Some("max-age=31536000".to_string()),
        cross_origin_opener_policy: Some("same-origin".to_string()),
        cross_origin_resource_policy: Some("same-origin".to_string()),
        permissions_policy: None,
    };

    let mut report = make_report(WcagResults::new());
    report.security = Some(make_security_from_headers(headers));
    let normalized = normalize(&report);
    let security_entry = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Security")
        .unwrap();
    assert!(
        security_entry.score < 100,
        "security_score must be < 100 when Permissions-Policy is missing, got {}",
        security_entry.score
    );
}

#[test]
fn security_score_below_100_without_coop() {
    use crate::security::SecurityHeaders;

    let headers = SecurityHeaders {
        content_security_policy: Some("default-src 'self'".to_string()),
        x_content_type_options: Some("nosniff".to_string()),
        x_frame_options: Some("DENY".to_string()),
        referrer_policy: Some("strict-origin".to_string()),
        strict_transport_security: Some("max-age=31536000".to_string()),
        permissions_policy: Some("camera=()".to_string()),
        cross_origin_resource_policy: Some("same-origin".to_string()),
        cross_origin_opener_policy: None,
    };

    let mut report = make_report(WcagResults::new());
    report.security = Some(make_security_from_headers(headers));
    let normalized = normalize(&report);
    let security_entry = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Security")
        .unwrap();
    assert!(
        security_entry.score < 100,
        "security_score must be < 100 when COOP is missing, got {}",
        security_entry.score
    );
}

#[test]
fn security_score_below_100_without_corp() {
    use crate::security::SecurityHeaders;

    let headers = SecurityHeaders {
        content_security_policy: Some("default-src 'self'".to_string()),
        x_content_type_options: Some("nosniff".to_string()),
        x_frame_options: Some("DENY".to_string()),
        referrer_policy: Some("strict-origin".to_string()),
        strict_transport_security: Some("max-age=31536000".to_string()),
        permissions_policy: Some("camera=()".to_string()),
        cross_origin_opener_policy: Some("same-origin".to_string()),
        cross_origin_resource_policy: None,
    };

    let mut report = make_report(WcagResults::new());
    report.security = Some(make_security_from_headers(headers));
    let normalized = normalize(&report);
    let security_entry = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Security")
        .unwrap();
    assert!(
        security_entry.score < 100,
        "security_score must be < 100 when CORP is missing, got {}",
        security_entry.score
    );
}

// ─── Report consistency invariants ────────────────────────────────────────────

#[test]
fn summary_severity_counts_matches_accessibility_module() {
    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        Severity::High,
        "Missing alt",
        "n1",
    ));
    results.add_violation(Violation::new(
        "1.4.3",
        "Contrast",
        WcagLevel::AA,
        Severity::Medium,
        "Low contrast",
        "n2",
    ));

    let report = make_report(results);
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(true).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let summary_counts = &json["summary"]["severity_counts"];
    let module_counts = &json["pages"][0]["detail"]["modules"]["accessibility"]["severity_counts"];

    assert_eq!(
        summary_counts["high"], module_counts["high"],
        "summary.severity_counts.high must match modules.accessibility.severity_counts.high"
    );
    assert_eq!(
        summary_counts["medium"], module_counts["medium"],
        "summary.severity_counts.medium must match modules.accessibility.severity_counts.medium"
    );
    assert_eq!(
        summary_counts["total"], module_counts["total"],
        "summary.severity_counts.total must match modules.accessibility.severity_counts.total"
    );
}

#[test]
fn fix_guidance_is_always_present() {
    // Single report with findings: fix_guidance must be a non-null array
    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        Severity::High,
        "Missing alt",
        "n1",
    ));
    let report = make_report(results);
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(false).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let guidance = &json["pages"][0]["detail"]["fix_guidance"];
    assert!(
        guidance.is_array(),
        "fix_guidance must be an array, got: {}",
        guidance
    );
    assert!(
        !guidance.as_array().unwrap().is_empty(),
        "fix_guidance must not be empty when findings exist"
    );

    // Single report with NO findings: fix_guidance must still be a present empty array
    let report_clean = make_report(WcagResults::new());
    let normalized_clean = normalize(&report_clean);
    let unified_clean = UnifiedReport::single(&normalized_clean, &report_clean);
    let json_clean: serde_json::Value =
        serde_json::from_str(&unified_clean.to_json(false).unwrap()).unwrap();

    let guidance_clean = &json_clean["pages"][0]["detail"]["fix_guidance"];
    assert!(
        guidance_clean.is_array(),
        "fix_guidance must always be an array, even when empty"
    );
}
