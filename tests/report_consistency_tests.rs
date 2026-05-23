//! Report Consistency Tests
//!
//! Ensures that:
//! - All active modules appear in module_scores and have corresponding detail data
//! - Overall score = weighted average of module scores
//! - Severity counts match finding occurrences
//! - fix_guidance entries match findings
//! - Score/grade/certificate are consistent

use auditmysite::audit::{
    normalize, AuditReport, BatchReport, ComparisonReport, PerformanceResults,
};
use auditmysite::cli::WcagLevel;
use auditmysite::journey::{analyze_journey, JourneyAnalysis};
use auditmysite::mobile::MobileFriendliness;
use auditmysite::output::builder::build_view_model;
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::{format_ai_json, format_json_batch, UnifiedReport};
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
                format!("node-{}", i),
            )
            .with_selector(format!("button.icon-{}", i))
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
                format!("node-sem-{}", i),
            )
            .with_selector(format!("div.table-{}", i))
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
            lcp_score: Some(20),
            fcp_score: Some(20),
            cls_score: Some(20),
            interactivity_score: Some(15),
            metrics_available: 4,
        },
        render_blocking: None,
        content_weight: None,
        third_party: None,
        critical_chain: None,
        minification: None,
        animations: None,
        coverage: None,
    }
}

fn make_seo() -> SeoAnalysis {
    SeoAnalysis {
        score: 90,
        ..Default::default()
    }
}

fn make_security() -> SecurityAnalysis {
    SecurityAnalysis {
        score: 80,
        grade: "B".to_string(),
        headers: Default::default(),
        ssl: Default::default(),
        issues: vec![],
        recommendations: vec![],
        protection: Default::default(),
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
    let unified = UnifiedReport::single(&normalize(&report), &report);
    let detail = unified.pages[0]
        .detail
        .as_ref()
        .expect("single report must carry detail");

    // If module has a score, its detail data must be present
    assert!(
        detail.modules.performance.is_some(),
        "Performance score exists but detail data missing"
    );
    assert!(
        detail.modules.seo.is_some(),
        "SEO score exists but detail data missing"
    );
    assert!(
        detail.modules.security.is_some(),
        "Security score exists but detail data missing"
    );
    assert!(
        detail.modules.mobile.is_some(),
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

    // Calculate expected weighted average from modules that contribute to overall_score.
    let weighted_sum: f64 = normalized
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.score as f64 * m.weight_pct as f64)
        .sum();
    let total_weight: f64 = normalized
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
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
            .map(|m| {
                format!(
                    "{}={} ({}%, contributes={})",
                    m.name, m.score, m.weight_pct, m.contributes_to_overall
                )
            })
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

    // severity_counts: zählt Findings (eine Zeile pro Regel/Severity).
    // occurrence_counts: summiert occurrence_count über alle Findings.
    let mut findings_critical = 0usize;
    let mut findings_high = 0usize;
    let mut findings_medium = 0usize;
    let mut findings_low = 0usize;
    let mut occ_critical = 0usize;
    let mut occ_high = 0usize;
    let mut occ_medium = 0usize;
    let mut occ_low = 0usize;

    for f in normalized.findings.iter().filter(|f| f.category == "wcag") {
        match f.severity {
            Severity::Critical => {
                findings_critical += 1;
                occ_critical += f.occurrence_count;
            }
            Severity::High => {
                findings_high += 1;
                occ_high += f.occurrence_count;
            }
            Severity::Medium => {
                findings_medium += 1;
                occ_medium += f.occurrence_count;
            }
            Severity::Low => {
                findings_low += 1;
                occ_low += f.occurrence_count;
            }
        }
    }

    assert_eq!(normalized.severity_counts.critical, findings_critical);
    assert_eq!(normalized.severity_counts.high, findings_high);
    assert_eq!(normalized.severity_counts.medium, findings_medium);
    assert_eq!(normalized.severity_counts.low, findings_low);
    assert_eq!(
        normalized.severity_counts.total,
        findings_critical + findings_high + findings_medium + findings_low
    );

    assert_eq!(normalized.occurrence_counts.critical, occ_critical);
    assert_eq!(normalized.occurrence_counts.high, occ_high);
    assert_eq!(normalized.occurrence_counts.medium, occ_medium);
    assert_eq!(normalized.occurrence_counts.low, occ_low);
    assert_eq!(
        normalized.occurrence_counts.total,
        occ_critical + occ_high + occ_medium + occ_low
    );
}

// ─── SEO-inclusive count tests (#254, #255) ───────────────────────

fn make_report_with_seo_finding() -> AuditReport {
    use auditmysite::seo::{HeadingIssue, HeadingStructure};
    let mut seo = make_seo();
    seo.headings = HeadingStructure {
        issues: vec![
            HeadingIssue {
                issue_type: "long_heading".to_string(),
                message: "Heading too long".to_string(),
                severity: Severity::Medium,
            },
            HeadingIssue {
                issue_type: "long_heading".to_string(),
                message: "Another long heading".to_string(),
                severity: Severity::Medium,
            },
        ],
        ..Default::default()
    };
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        make_violations(),
        1000,
    )
    .with_seo(seo)
}

#[test]
fn test_json_counts_include_seo_findings() {
    let report = make_report_with_seo_finding();
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let page = &unified.pages[0];

    let total_rules = normalized
        .findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let total_occurrences: usize = normalized.findings.iter().map(|f| f.occurrence_count).sum();
    let seo_rules = normalized
        .findings
        .iter()
        .filter(|f| f.category == "seo")
        .count();
    assert!(seo_rules > 0, "test report must contain SEO findings");

    // violated_rule_count + violation_count + occurrence_counts span all categories (#254/#255)
    assert_eq!(page.violated_rule_count, total_rules);
    assert_eq!(page.violation_count, total_occurrences);
    assert_eq!(page.occurrence_counts.total, total_occurrences);

    // severity_counts stays WCAG-only (legal/risk semantics)
    let wcag_rules = normalized
        .findings
        .iter()
        .filter(|f| f.category == "wcag")
        .count();
    assert_eq!(page.severity_counts.total, wcag_rules);
    assert!(
        page.violated_rule_count > page.severity_counts.total,
        "SEO findings must lift violated_rule_count above WCAG-only severity_counts"
    );
}

#[test]
fn test_batch_pages_carry_fix_guidance_detail() {
    let batch = BatchReport::from_reports(vec![make_report_with_seo_finding()], vec![], 100);
    let parsed: serde_json::Value =
        serde_json::from_str(&format_json_batch(&batch, true).expect("batch JSON must render"))
            .expect("batch JSON must parse");

    let detail = &parsed["pages"][0]["detail"];
    assert!(
        detail["fix_guidance"].is_array(),
        "batch page must carry detail.fix_guidance (#256)"
    );
    assert!(
        !detail["fix_guidance"].as_array().unwrap().is_empty(),
        "fix_guidance should list the page's findings"
    );
    // Batch detail stays compact: no heavy module blob.
    assert!(
        detail["modules"]
            .as_object()
            .map(|m| m.is_empty())
            .unwrap_or(true),
        "batch detail.modules should be empty to keep batch reports compact"
    );
}

// ─── Fix Guidance Tests ────────────────────────────────────────────

#[test]
fn test_fix_guidance_matches_findings() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let fix_guidance = &unified.pages[0]
        .detail
        .as_ref()
        .expect("single report must carry detail")
        .fix_guidance;

    assert_eq!(
        fix_guidance.len(),
        normalized.findings.len(),
        "fix_guidance count should match findings count"
    );

    for (guidance, finding) in fix_guidance.iter().zip(&normalized.findings) {
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
    let unified = UnifiedReport::single(&normalize(&report), &report);
    let detail = unified.pages[0].detail.as_ref().expect("detail");

    for g in &detail.fix_guidance {
        assert!(
            !g.problem.is_empty(),
            "fix_guidance for {} has empty problem",
            g.rule_id
        );
        if let Some(rec) = &g.recommendation {
            assert!(
                !rec.is_empty(),
                "fix_guidance for {} has empty recommendation",
                g.rule_id
            );
        }
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
    let unified = UnifiedReport::single(&normalize(&report), &report);
    let detail = unified.pages[0].detail.as_ref().expect("detail");

    // At least some findings should have code examples (from our explanation database)
    let with_code = detail
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
                    format!("n-{}", i),
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

    let total_weight: u32 = normalized
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.weight_pct)
        .sum();
    assert_eq!(
        total_weight, 100,
        "Contributing module weights should sum to 100%, got {}%",
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
    let unified = UnifiedReport::single(&normalize(&report), &report);
    let json_str = unified.to_json(true).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // All required sections present
    assert!(parsed["metadata"].is_object(), "metadata missing");
    assert!(parsed["summary"].is_object(), "summary missing");
    let detail = &parsed["pages"][0]["detail"];
    assert!(detail["fix_guidance"].is_array(), "fix_guidance missing");
    let modules = &detail["modules"];
    assert!(
        !modules["performance"].is_null(),
        "performance data missing"
    );
    assert!(!modules["seo"].is_null(), "seo data missing");
    assert!(!modules["security"].is_null(), "security data missing");
    assert!(!modules["mobile"].is_null(), "mobile data missing");
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
    let unified = UnifiedReport::single(&normalized, &report);
    let parsed: serde_json::Value = serde_json::from_str(&unified.to_json(true).unwrap()).unwrap();

    let flags = parsed["pages"][0]["audit_flags"]
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
    let mut unified = UnifiedReport::single(&normalize(&report), &report);
    unified.set_history(serde_json::json!({"test": true}));
    let json_str = unified.to_json(true).unwrap();

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
    let unified = UnifiedReport::single(&normalize(&report), &report);
    let json_str = unified.to_json(true).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let risk = &parsed["pages"][0]["risk"];
    assert!(risk.is_object(), "risk object missing from JSON report");
    assert_eq!(risk["level"].as_str().unwrap(), "critical");
    assert!(risk["critical_issues"].as_u64().unwrap() > 0);
    assert!(!risk["summary"].as_str().unwrap().is_empty());
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
            format!("n-{}", i),
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
    let unified = UnifiedReport::single(&normalize(&report), &report);
    let detail = unified.pages[0].detail.as_ref().expect("detail");
    assert!(
        detail.modules.journey.is_some(),
        "Journey detail data should be present in JSON"
    );
}

#[test]
fn test_json_module_detail_scores_match_normalized_scores() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let m = &unified.pages[0].detail.as_ref().expect("detail").modules;

    for (module_name, json_section, score_path) in [
        ("Performance", m.performance.as_ref(), &["score"][..]),
        ("SEO", m.seo.as_ref(), &["score"][..]),
        ("Security", m.security.as_ref(), &["score"][..]),
        ("Mobile", m.mobile.as_ref(), &["score"][..]),
        ("UX", m.ux.as_ref(), &["score"][..]),
        ("Journey", m.journey.as_ref(), &["score"][..]),
    ] {
        let expected = normalized
            .module_scores
            .iter()
            .find(|m| m.name == module_name)
            .map(|m| m.score)
            .expect("module score must exist");
        let actual = score_path
            .iter()
            .try_fold(
                json_section.expect("JSON module detail must exist"),
                |value, key| value.get(*key),
            )
            .and_then(|value| value.as_u64())
            .expect("JSON module detail score must exist") as u32;

        assert_eq!(
            actual, expected,
            "{module_name} JSON detail score must match NormalizedReport.module_scores"
        );
    }
}

#[test]
fn test_ai_json_scores_match_normalized_scores() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let parsed: serde_json::Value =
        serde_json::from_str(&format_ai_json(&report)).expect("AI JSON must parse");

    assert_eq!(
        parsed["score"].as_u64(),
        Some(normalized.score as u64),
        "AI JSON score must use NormalizedReport.score as primary WCAG score"
    );
    assert_eq!(
        parsed["overall_score"].as_u64(),
        Some(normalized.overall_score as u64),
        "AI JSON overall_score must use NormalizedReport.overall_score as secondary weighted score"
    );
}

#[test]
fn test_comparison_scores_match_normalized_scores() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let comparison = ComparisonReport::from_reports(vec![report], 100);
    let entry = comparison.entries.first().expect("entry must exist");

    assert_eq!(entry.accessibility_score, normalized.score);
    assert_eq!(entry.overall_score, normalized.overall_score);
    assert_eq!(
        entry.performance_score,
        normalized
            .module_scores
            .iter()
            .find(|m| m.name == "Performance")
            .map(|m| m.score)
    );
}

#[test]
fn test_batch_json_average_score_is_accessibility_with_overall_separate() {
    let reports = vec![make_full_report()];
    let normalized = normalize(&reports[0]);
    let expected_overall = normalized.overall_score as u64;
    let batch = BatchReport::from_reports(reports, vec![], 100);
    let expected_score = batch.summary.average_score.round() as u64;
    let parsed: serde_json::Value =
        serde_json::from_str(&format_json_batch(&batch, true).expect("batch JSON must render"))
            .expect("batch JSON must parse");

    assert_eq!(
        parsed["summary"]["accessibility_score"].as_u64(),
        Some(expected_score),
        "Batch accessibility_score must be the WCAG/accessibility average"
    );
    assert_eq!(
        parsed["summary"]["overall_score"].as_u64(),
        Some(expected_overall)
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
fn test_batch_summary_uses_normalized_primary_score() {
    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    // Score rounds to 80 — meets the new pass criterion (≥ 80, no criticals,
    // no WCAG-A high/critical findings). See issue #253.
    report.score = 79.6;

    let normalized = normalize(&report);
    let batch = BatchReport::from_reports(vec![report], vec![], 100);

    assert_eq!(normalized.score, 80);
    assert_eq!(batch.summary.average_score, 80.0);
    assert_eq!(batch.summary.passed, 1);
    assert_eq!(batch.summary.failed, 0);
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

#[test]
fn test_i18n_en_returns_english_labels() {
    let i18n = auditmysite::i18n::I18n::new("en").unwrap();
    // Core finding labels
    assert_eq!(i18n.t("finding-elements"), "Elements");
    assert_eq!(i18n.t("finding-occurrences"), "Occurrences");
    assert_eq!(i18n.t("finding-recommendation"), "Recommendation");
    assert_eq!(
        i18n.t("finding-representative-occurrences"),
        "Representative occurrences"
    );
    // Module labels
    assert_eq!(i18n.t("mobile-touch-targets"), "Touch targets");
    assert_eq!(i18n.t("security-score-card"), "Security score");
    assert_eq!(i18n.t("seo-serp-readiness"), "SERP readiness");
}

#[test]
fn test_i18n_de_returns_german_labels() {
    let i18n = auditmysite::i18n::I18n::new("de").unwrap();
    assert_eq!(i18n.t("finding-elements"), "Elemente");
    assert_eq!(i18n.t("finding-occurrences"), "Vorkommen");
    assert_eq!(i18n.t("finding-recommendation"), "Empfehlung");
    assert_eq!(i18n.t("mobile-touch-targets"), "Touch Targets");
    assert_eq!(i18n.t("security-score-card"), "Security Score");
    assert_eq!(i18n.t("seo-serp-readiness"), "SERP-Bereitschaft");
}

#[test]
fn test_i18n_missing_key_returns_key_as_fallback() {
    let i18n = auditmysite::i18n::I18n::new("en").unwrap();
    assert_eq!(i18n.t("nonexistent-key-xyz"), "nonexistent-key-xyz");
}

// ─── #62 / #63: Schema contract + module weight semantics ─────────────────────

#[test]
fn test_schema_contains_extra_module_top_level_keys() {
    let schema_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/json-report.schema.json");
    let schema_str = std::fs::read_to_string(&schema_path)
        .expect("docs/json-report.schema.json must be readable");
    let schema: serde_json::Value =
        serde_json::from_str(&schema_str).expect("schema must be valid JSON");

    // v2.0 envelope: module detail lives under pages[].detail.
    let module_props = schema["$defs"]["moduleBlob"]["properties"]
        .as_object()
        .expect("schema must define $defs.moduleBlob.properties");
    for key in &[
        "dark_mode",
        "ai_visibility",
        "source_quality",
        "content_visibility",
        "tech_stack",
        "patterns",
    ] {
        assert!(
            module_props.contains_key(*key),
            "schema $defs.moduleBlob missing module '{key}'"
        );
    }

    let detail_props = schema["$defs"]["pageDetail"]["properties"]
        .as_object()
        .expect("schema must define $defs.pageDetail.properties");
    for key in &[
        "budget_violations",
        "throttled_performance",
        "screenshot_status",
    ] {
        assert!(
            detail_props.contains_key(*key),
            "schema $defs.pageDetail missing '{key}'"
        );
    }
}

#[test]
fn test_json_report_includes_extra_module_keys() {
    use auditmysite::{
        analyze_ai_visibility, analyze_content_visibility, analyze_source_quality, DarkModeAnalysis,
    };

    let base = make_full_report();
    let mut report = make_full_report();
    report.source_quality = Some(analyze_source_quality(&base));
    report.ai_visibility = Some(analyze_ai_visibility(&base));
    report.content_visibility = Some(analyze_content_visibility(&base));
    report.dark_mode = Some(DarkModeAnalysis {
        supported: false,
        score: 0,
        detection_methods: vec![],
        color_scheme_css: false,
        meta_color_scheme: None,
        meta_theme_color_dark: false,
        css_custom_properties: 0,
        dark_contrast_violations: 0,
        light_only_violations: 0,
        dark_only_violations: 0,
        contrast_violations: vec![],
        issues: vec![],
    });

    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(true).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let modules = &parsed["pages"][0]["detail"]["modules"];

    assert!(
        !modules["dark_mode"].is_null(),
        "dark_mode missing from JSON"
    );
    assert!(
        !modules["ai_visibility"].is_null(),
        "ai_visibility missing from JSON"
    );
    assert!(
        !modules["source_quality"].is_null(),
        "source_quality missing from JSON"
    );
    assert!(
        !modules["content_visibility"].is_null(),
        "content_visibility missing from JSON"
    );
    assert_eq!(modules["source_quality"]["measurement_type"], "heuristic");
    assert_eq!(modules["ai_visibility"]["measurement_type"], "heuristic");
    assert_eq!(
        modules["content_visibility"]["measurement_type"],
        "heuristic"
    );
}

#[test]
fn test_json_report_includes_report_artifact_fields() {
    let mut report = make_full_report();
    report.tech_stack = Some(auditmysite::tech_stack::TechStackAnalysis {
        detected: vec![auditmysite::tech_stack::DetectedTech {
            name: "Astro".to_string(),
            category: auditmysite::tech_stack::TechCategory::Framework,
            version: None,
            confidence: auditmysite::tech_stack::Confidence::High,
            signals: vec!["test signal".to_string()],
        }],
        findings: vec![],
        score: 100,
        grade: "A".to_string(),
    });
    report.budget_violations = vec![auditmysite::audit::BudgetViolation {
        metric: "LCP".to_string(),
        budget_label: "<= 2500 ms".to_string(),
        actual_label: "3200 ms".to_string(),
        budget_value: 2500.0,
        actual_value: 3200.0,
        exceeded_by_pct: 28.0,
        severity: auditmysite::audit::BudgetSeverity::Warning,
    }];
    report.throttled_performance = vec![auditmysite::audit::ThrottledPerfResult {
        profile: auditmysite::browser::ThrottleProfile::Slow3G,
        lcp_ms: Some(3200.0),
        tbt_ms: Some(180.0),
        cls: Some(0.03),
        score: 72,
    }];
    report.patterns = Some(auditmysite::patterns::PatternAnalysis {
        recognized: vec![auditmysite::patterns::RecognizedPattern {
            pattern: "MainNavigation".to_string(),
            message: "Main navigation recognized".to_string(),
            confidence: auditmysite::patterns::PatternConfidence::Strong,
        }],
        violations: vec![],
    });
    report.screenshot_status = auditmysite::audit::ScreenshotStatus::Failed("test".to_string());

    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let parsed: serde_json::Value = serde_json::from_str(&unified.to_json(true).unwrap()).unwrap();
    let detail = &parsed["pages"][0]["detail"];
    let modules = &detail["modules"];

    assert!(
        !modules["tech_stack"].is_null(),
        "tech_stack missing from JSON"
    );
    assert_eq!(detail["budget_violations"].as_array().unwrap().len(), 1);
    assert_eq!(detail["throttled_performance"].as_array().unwrap().len(), 1);
    assert!(!modules["patterns"].is_null(), "patterns missing from JSON");
    assert!(
        !detail["screenshot_status"].is_null(),
        "screenshot_status missing from JSON"
    );
}

#[test]
fn test_contributes_to_overall_flags_correct() {
    let report = make_full_report();
    let normalized = normalize(&report);

    let core = ["Accessibility", "Performance", "SEO", "Security", "Mobile"];
    let supplemental = ["UX", "Journey"];

    for m in &normalized.module_scores {
        if core.contains(&m.name.as_str()) {
            assert!(
                m.contributes_to_overall,
                "{} should have contributes_to_overall=true",
                m.name
            );
        } else if supplemental.contains(&m.name.as_str()) {
            assert!(
                !m.contributes_to_overall,
                "{} should have contributes_to_overall=false",
                m.name
            );
        }
        match m.name.as_str() {
            "UX" | "Journey" => assert_eq!(m.measurement_type, "heuristic"),
            _ => assert_eq!(m.measurement_type, "measured"),
        }
    }

    // Core weights must sum to exactly 100 so overall_score is a proper percentage
    let core_weight: u32 = normalized
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.weight_pct)
        .sum();
    assert_eq!(
        core_weight, 100,
        "contributes_to_overall=true modules must sum to 100 weight_pct, got {}",
        core_weight
    );
}

#[test]
fn test_view_model_shows_journey_as_indicator_dashboard_module() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let vm = build_view_model(&normalized, &ReportConfig::default());
    let journey = vm
        .modules
        .dashboard
        .iter()
        .find(|m| m.name == "Journey")
        .expect("Journey must be visible in module dashboard");

    assert_eq!(journey.measurement_type, "heuristic");
}
