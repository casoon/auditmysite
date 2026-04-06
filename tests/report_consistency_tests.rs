//! Report Consistency Tests
//!
//! Ensures that:
//! - All active modules appear in module_scores and have corresponding detail data
//! - Overall score = weighted average of module scores
//! - Severity counts match finding occurrences
//! - fix_guidance entries match findings
//! - Score/grade/certificate are consistent

use auditmysite::audit::{normalize, AuditReport, PerformanceResults};
use auditmysite::cli::WcagLevel;
use auditmysite::journey::{analyze_journey, JourneyAnalysis};
use auditmysite::mobile::MobileFriendliness;
use auditmysite::output::builder::build_view_model;
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::JsonReport;
use auditmysite::performance::{PerformanceGrade, PerformanceScore, WebVitals};
use auditmysite::security::SecurityAnalysis;
use auditmysite::seo::SeoAnalysis;
use auditmysite::ux::{analyze_ux, UxAnalysis};
use auditmysite::wcag::{Severity, Violation, WcagResults};
use auditmysite::AXTree;

// ─── Helpers ───────────────────────────────────────────────────────

fn make_violations() -> WcagResults {
    let mut results = WcagResults::new();
    results.passes = 50;

    // Critical: name/role violations
    for i in 0..5 {
        results.add_violation(
            Violation::new(
                "4.1.2",
                "Name, Role, Value",
                WcagLevel::A,
                Severity::Critical,
                "Button missing accessible name",
                &format!("node-{}", i),
            )
            .with_selector(&format!("button.icon-{}", i))
            .with_fix("Add aria-label attribute"),
        );
    }

    // Medium: semantic structure
    for i in 0..3 {
        results.add_violation(
            Violation::new(
                "1.3.1",
                "Info and Relationships",
                WcagLevel::A,
                Severity::Medium,
                "Content uses visual formatting instead of semantic HTML",
                &format!("node-sem-{}", i),
            )
            .with_selector(&format!("div.table-{}", i))
            .with_fix("Use semantic HTML elements"),
        );
    }

    results
}

fn make_performance() -> PerformanceResults {
    PerformanceResults {
        vitals: WebVitals::default(),
        score: PerformanceScore {
            overall: 75,
            grade: PerformanceGrade::Silver,
            lcp_score: 20,
            fcp_score: 20,
            cls_score: 20,
            interactivity_score: 15,
        },
        render_blocking: None,
        content_weight: None,
    }
}

fn make_seo() -> SeoAnalysis {
    let mut seo = SeoAnalysis::default();
    seo.score = 90;
    seo
}

fn make_security() -> SecurityAnalysis {
    SecurityAnalysis {
        score: 80,
        grade: "B".to_string(),
        headers: Default::default(),
        ssl: Default::default(),
        issues: vec![],
        recommendations: vec![],
    }
}

fn make_mobile() -> MobileFriendliness {
    MobileFriendliness {
        score: 85,
        viewport: Default::default(),
        touch_targets: Default::default(),
        font_sizes: Default::default(),
        content_sizing: Default::default(),
        issues: vec![],
    }
}

fn make_ux() -> UxAnalysis {
    let tree = AXTree::new();
    analyze_ux(&tree)
}

fn make_journey() -> JourneyAnalysis {
    let tree = AXTree::new();
    analyze_journey(&tree)
}

fn make_full_report() -> AuditReport {
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        make_violations(),
        1000,
    )
    .with_performance(make_performance())
    .with_seo(make_seo())
    .with_security(make_security())
    .with_mobile(make_mobile())
    .with_ux(make_ux())
    .with_journey(make_journey())
}

// ─── Module Presence Tests ─────────────────────────────────────────

#[test]
fn test_all_modules_present_in_module_scores() {
    let report = make_full_report();
    let normalized = normalize(&report);

    let module_names: Vec<&str> = normalized
        .module_scores
        .iter()
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        module_names.contains(&"Accessibility"),
        "Accessibility missing from module_scores"
    );
    assert!(
        module_names.contains(&"Performance"),
        "Performance missing from module_scores"
    );
    assert!(
        module_names.contains(&"SEO"),
        "SEO missing from module_scores"
    );
    assert!(
        module_names.contains(&"Security"),
        "Security missing from module_scores"
    );
    assert!(
        module_names.contains(&"Mobile"),
        "Mobile missing from module_scores"
    );
    assert!(
        module_names.contains(&"UX"),
        "UX missing from module_scores"
    );
    assert!(
        module_names.contains(&"Journey"),
        "Journey missing from module_scores"
    );
}

#[test]
fn test_module_data_present_when_scored() {
    let report = make_full_report();
    let json_report = JsonReport::from_normalized(&normalize(&report), &report);

    // If module has a score, its detail data must be present
    assert!(
        json_report.performance.is_some(),
        "Performance score exists but detail data missing"
    );
    assert!(
        json_report.seo.is_some(),
        "SEO score exists but detail data missing"
    );
    assert!(
        json_report.security.is_some(),
        "Security score exists but detail data missing"
    );
    assert!(
        json_report.mobile.is_some(),
        "Mobile score exists but detail data missing"
    );
}

#[test]
fn test_missing_modules_not_in_scores() {
    // Report with NO modules
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        500,
    );
    let normalized = normalize(&report);

    // Only Accessibility should be in module_scores
    assert_eq!(
        normalized.module_scores.len(),
        1,
        "Only Accessibility should be present without --full"
    );
    assert_eq!(normalized.module_scores[0].name, "Accessibility");
}

// ─── Overall Score Consistency ─────────────────────────────────────

#[test]
fn test_overall_score_equals_weighted_average() {
    let report = make_full_report();
    let normalized = normalize(&report);

    // Calculate expected weighted average
    let weighted_sum: f64 = normalized
        .module_scores
        .iter()
        .map(|m| m.score as f64 * m.weight_pct as f64)
        .sum();
    let total_weight: f64 = normalized
        .module_scores
        .iter()
        .map(|m| m.weight_pct as f64)
        .sum();
    let expected = (weighted_sum / total_weight).round() as u32;

    assert_eq!(
        normalized.overall_score,
        expected,
        "overall_score ({}) != weighted average of module_scores ({}). Modules: {:?}",
        normalized.overall_score,
        expected,
        normalized
            .module_scores
            .iter()
            .map(|m| format!("{}={} ({}%)", m.name, m.score, m.weight_pct))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_overall_score_accessibility_only() {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        500,
    );
    let normalized = normalize(&report);

    // With only Accessibility, overall_score == accessibility score
    assert_eq!(
        normalized.overall_score, normalized.score,
        "With only Accessibility module, overall should equal accessibility score"
    );
}

// ─── Severity Consistency ──────────────────────────────────────────

#[test]
fn test_severity_counts_match_findings() {
    let report = make_full_report();
    let normalized = normalize(&report);

    let mut critical_from_findings = 0usize;
    let mut high_from_findings = 0usize;
    let mut medium_from_findings = 0usize;
    let mut low_from_findings = 0usize;

    for f in &normalized.findings {
        match f.severity {
            Severity::Critical => critical_from_findings += f.occurrence_count,
            Severity::High => high_from_findings += f.occurrence_count,
            Severity::Medium => medium_from_findings += f.occurrence_count,
            Severity::Low => low_from_findings += f.occurrence_count,
        }
    }

    assert_eq!(
        normalized.severity_counts.critical, critical_from_findings,
        "Critical count mismatch"
    );
    assert_eq!(
        normalized.severity_counts.high, high_from_findings,
        "High count mismatch"
    );
    assert_eq!(
        normalized.severity_counts.medium, medium_from_findings,
        "Medium count mismatch"
    );
    assert_eq!(
        normalized.severity_counts.low, low_from_findings,
        "Low count mismatch"
    );

    let expected_total =
        critical_from_findings + high_from_findings + medium_from_findings + low_from_findings;
    assert_eq!(
        normalized.severity_counts.total, expected_total,
        "Total count mismatch"
    );
}

// ─── Fix Guidance Tests ────────────────────────────────────────────

#[test]
fn test_fix_guidance_matches_findings() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let json_report = JsonReport::from_normalized(&normalized, &report);

    assert_eq!(
        json_report.fix_guidance.len(),
        normalized.findings.len(),
        "fix_guidance count should match findings count"
    );

    for (guidance, finding) in json_report.fix_guidance.iter().zip(&normalized.findings) {
        assert_eq!(
            guidance.rule_id, finding.rule_id,
            "fix_guidance rule_id mismatch"
        );
        assert_eq!(
            guidance.occurrence_count, finding.occurrence_count,
            "fix_guidance occurrence_count mismatch for {}",
            guidance.rule_id
        );
    }
}

#[test]
fn test_fix_guidance_has_actionable_content() {
    let report = make_full_report();
    let json_report = JsonReport::from_normalized(&normalize(&report), &report);

    for g in &json_report.fix_guidance {
        assert!(
            !g.problem.is_empty(),
            "fix_guidance for {} has empty problem",
            g.rule_id
        );
        assert!(
            !g.recommendation.is_empty(),
            "fix_guidance for {} has empty recommendation",
            g.rule_id
        );
        assert!(
            !g.severity.is_empty(),
            "fix_guidance for {} has empty severity",
            g.rule_id
        );
    }
}

#[test]
fn test_fix_guidance_code_examples_present() {
    let report = make_full_report();
    let json_report = JsonReport::from_normalized(&normalize(&report), &report);

    // At least some findings should have code examples (from our explanation database)
    let with_code = json_report
        .fix_guidance
        .iter()
        .filter(|g| g.code_example.is_some())
        .count();

    // Our test report has rule 4.1.2 and 1.3.1 which both have explanations with code examples
    assert!(
        with_code > 0,
        "Expected at least one fix_guidance entry with code examples, got none"
    );
}

// ─── Score / Grade / Certificate Consistency ───────────────────────

#[test]
fn test_grade_matches_score() {
    // Test several score ranges
    let test_cases = vec![
        (100, "A"), // 90+ = A
        (50, "F"),  // <60 = F
    ];

    for (expected_score_approx, _expected_grade) in test_cases {
        let mut results = WcagResults::new();
        results.passes = 100;

        // Add violations to lower score (if needed)
        if expected_score_approx < 90 {
            for i in 0..20 {
                results.add_violation(Violation::new(
                    "4.1.2",
                    "Name, Role, Value",
                    WcagLevel::A,
                    Severity::Critical,
                    "Missing label",
                    &format!("n-{}", i),
                ));
            }
        }

        let report = AuditReport::new(
            "https://example.com".to_string(),
            WcagLevel::AA,
            results,
            100,
        );
        let normalized = normalize(&report);

        // Grade must be consistent with score
        let grade_from_score = match normalized.score {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        };

        assert_eq!(
            normalized.grade, grade_from_score,
            "Grade '{}' doesn't match score {} (expected '{}')",
            normalized.grade, normalized.score, grade_from_score
        );
    }
}

// ─── Module Weight Tests ───────────────────────────────────────────

#[test]
fn test_module_weights_sum_to_expected() {
    let report = make_full_report();
    let normalized = normalize(&report);

    let total_weight: u32 = normalized.module_scores.iter().map(|m| m.weight_pct).sum();
    // With UX (15%) + Journey (10%) added to base modules (100%), total is 125%
    // The overall_score calculation divides by actual total weight
    assert_eq!(
        total_weight, 125,
        "Module weights should sum to 125% (with UX + Journey), got {}%",
        total_weight
    );
}

#[test]
fn test_module_weights_correct() {
    let report = make_full_report();
    let normalized = normalize(&report);

    for m in &normalized.module_scores {
        let expected_weight = match m.name.as_str() {
            "Accessibility" => 40,
            "Performance" => 20,
            "SEO" => 20,
            "UX" => 15,
            "Journey" => 10,
            "Security" => 10,
            "Mobile" => 10,
            _ => panic!("Unknown module: {}", m.name),
        };
        assert_eq!(
            m.weight_pct, expected_weight,
            "Weight for {} should be {}%, got {}%",
            m.name, expected_weight, m.weight_pct
        );
    }
}

// ─── JSON Structure Tests ──────────────────────────────────────────

#[test]
fn test_json_contains_all_sections() {
    let report = make_full_report();
    let json_report = JsonReport::from_normalized(&normalize(&report), &report);
    let json_str = json_report.to_json(true).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // All required sections present
    assert!(parsed["metadata"].is_object(), "metadata missing");
    assert!(parsed["report"].is_object(), "report missing");
    assert!(parsed["fix_guidance"].is_array(), "fix_guidance missing");
    assert!(!parsed["performance"].is_null(), "performance data missing");
    assert!(!parsed["seo"].is_null(), "seo data missing");
    assert!(!parsed["security"].is_null(), "security data missing");
    assert!(!parsed["mobile"].is_null(), "mobile data missing");
}

#[test]
fn test_audit_flags_surface_in_json_output() {
    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "3.1.1",
        "Language",
        WcagLevel::A,
        Severity::High,
        "Missing lang",
        "html",
    ));

    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        100,
    );
    let mut seo = SeoAnalysis::default();
    seo.technical.has_lang = true;
    report = report.with_seo(seo);

    let normalized = normalize(&report);
    let json_report = JsonReport::from_normalized(&normalized, &report);
    let parsed: serde_json::Value = serde_json::from_str(&json_report.to_json(true).unwrap()).unwrap();

    let flags = parsed["report"]["audit_flags"]
        .as_array()
        .expect("audit_flags should be present");
    assert_eq!(flags.len(), 1);
    assert_eq!(
        flags[0]["related_rule"].as_str(),
        Some("3.1.1"),
        "expected 3.1.1 conflict flag"
    );
}

#[test]
fn test_json_field_order_fix_guidance_before_history() {
    let report = make_full_report();
    let mut json_report = JsonReport::from_normalized(&normalize(&report), &report);
    json_report.history = Some(serde_json::json!({"test": true}));
    let json_str = json_report.to_json(true).unwrap();

    // fix_guidance should appear before history in serialized output
    let fg_pos = json_str
        .find("fix_guidance")
        .expect("fix_guidance not in JSON");
    let hist_pos = json_str.find("history").expect("history not in JSON");
    assert!(
        fg_pos < hist_pos,
        "fix_guidance should appear before history in JSON"
    );
}

// ─── Risk Assessment Tests ────────────────────────────────────────

#[test]
fn test_risk_critical_with_level_a_violations() {
    // Full report has 5 Critical + 3 Medium, all WCAG Level A
    let report = make_full_report();
    let normalized = normalize(&report);

    assert_eq!(
        normalized.risk.level,
        auditmysite::audit::normalized::RiskLevel::Critical,
        "Expected Critical risk for report with critical Level A violations"
    );
    assert!(
        normalized.risk.legal_flags > 0,
        "Should have legal flags for Level A violations"
    );
    assert!(
        normalized.risk.blocking_issues > 0,
        "Should have blocking issues for 4.1.2 violations"
    );
}

#[test]
fn test_risk_low_without_violations() {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        500,
    );
    let normalized = normalize(&report);

    assert_eq!(
        normalized.risk.level,
        auditmysite::audit::normalized::RiskLevel::Low,
        "Expected Low risk for report without violations"
    );
    assert_eq!(normalized.risk.critical_issues, 0);
    assert_eq!(normalized.risk.legal_flags, 0);
}

#[test]
fn test_risk_in_json_output() {
    let report = make_full_report();
    let json_report = JsonReport::from_normalized(&normalize(&report), &report);
    let json_str = json_report.to_json(true).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let risk = &parsed["report"]["risk"];
    assert!(risk.is_object(), "risk object missing from JSON report");
    assert_eq!(risk["level"].as_str().unwrap(), "critical");
    assert!(risk["critical_issues"].as_u64().unwrap() > 0);
    assert!(risk["summary"].as_str().unwrap().len() > 0);
}

#[test]
fn test_risk_independent_from_score() {
    // A report can have good score but still high risk
    let mut results = WcagResults::new();
    results.passes = 200; // lots of passes = high score

    // But 3 critical Level A violations
    for i in 0..3 {
        results.add_violation(Violation::new(
            "4.1.2",
            "Name, Role, Value",
            WcagLevel::A,
            Severity::Critical,
            "Missing name",
            &format!("n-{}", i),
        ));
    }

    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        100,
    );
    let normalized = normalize(&report);

    // Score should be decent (many passes)
    assert!(
        normalized.score >= 50,
        "Score should be decent with many passes"
    );
    // But risk should be critical
    assert_eq!(
        normalized.risk.level,
        auditmysite::audit::normalized::RiskLevel::Critical,
        "Risk should be Critical despite decent score — score != risk"
    );
}

// ─── Journey Module Tests ─────────────────────────────────────────

#[test]
fn test_journey_in_json_output() {
    let report = make_full_report();
    let json_report = JsonReport::from_normalized(&normalize(&report), &report);
    assert!(
        json_report.journey.is_some(),
        "Journey detail data should be present in JSON"
    );
}

#[test]
fn test_journey_module_weight() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let journey_entry = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Journey");
    assert!(journey_entry.is_some(), "Journey must be in module_scores");
    assert_eq!(
        journey_entry.unwrap().weight_pct,
        10,
        "Journey weight must be 10%"
    );
}

#[test]
fn test_journey_has_page_intent() {
    let tree = AXTree::new();
    let journey = analyze_journey(&tree);
    // Empty tree should still return a valid intent
    assert!(!journey.grade.is_empty());
    assert!(journey.score <= 100);
}

#[test]
fn test_journey_dimensions_count() {
    let report = make_full_report();
    let journey = report.journey.as_ref().unwrap();
    // All 5 dimensions present
    assert!(!journey.entry_clarity.name.is_empty());
    assert!(!journey.orientation.name.is_empty());
    assert!(!journey.navigation.name.is_empty());
    assert!(!journey.interaction.name.is_empty());
    assert!(!journey.conversion.name.is_empty());
}

#[test]
fn test_view_model_preserves_cli_facts_and_finding_density() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let vm = build_view_model(&normalized, &ReportConfig::default());

    assert!(vm
        .methodology
        .audit_facts
        .iter()
        .any(|(label, _)| label == "Geprüfte Knoten"));
    assert!(vm
        .methodology
        .audit_facts
        .iter()
        .any(|(label, _)| label == "Laufzeit"));

    let top = vm
        .findings
        .top_findings
        .first()
        .expect("top finding should exist");
    assert!(top.occurrence_count >= top.representative_occurrences.len());
    assert_eq!(
        top.additional_occurrences,
        top.occurrence_count
            .saturating_sub(top.representative_occurrences.len())
    );
}
