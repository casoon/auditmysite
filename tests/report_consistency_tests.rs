//! Report Consistency Tests
//!
//! Ensures that:
//! - All active modules appear in module_scores and have corresponding detail data
//! - Overall score = weighted average of module scores
//! - Severity counts match finding occurrences
//! - fix_guidance entries match findings
//! - Score/grade/certificate are consistent

use auditmysite::audit::{normalize, AuditReport, BatchReport, PerformanceResults};
use auditmysite::cli::WcagLevel;
use auditmysite::journey::{analyze_journey, JourneyAnalysis};
use auditmysite::mobile::MobileFriendliness;
use auditmysite::output::builder::build_view_model;
use auditmysite::output::report_model::ReportConfig;
use auditmysite::output::{format_ai_json, format_json_batch, UnifiedReport};
use auditmysite::performance::{PerformanceGrade, PerformanceScore, WebVitals};
use auditmysite::registry::LETTER_GRADE;
use auditmysite::security::SecurityAnalysis;
use auditmysite::seo::SeoAnalysis;
use auditmysite::ux::{analyze_ux, UxAnalysis};
use auditmysite::wcag::{Severity, Violation, WcagResults};
use auditmysite::AXTree;

// Cross-layer regressions: a locally plausible result must remain truthful
// after normalization, aggregation and export.
#[test]
fn foundation_compact_export_grade_describes_its_score() {
    // One grade basis (#781): the compact feed names both scores and bands
    // the accessibility score, like every other output; there is no
    // unqualified `score` that could be read against the wrong grade.
    let normalized = normalize(&make_full_report()).normalized;
    assert_ne!(
        normalized.grade,
        auditmysite::audit::AccessibilityScorer::calculate_grade(normalized.overall_score as f32)
    );
    let json: serde_json::Value =
        serde_json::from_str(&auditmysite::output::format_summary(&normalized).unwrap()).unwrap();
    assert!(json.get("score").is_none());
    assert_eq!(json["overallScore"], normalized.overall_score);
    assert_eq!(json["accessibilityScore"], normalized.score);
    assert_eq!(
        json["grade"],
        auditmysite::audit::AccessibilityScorer::calculate_grade(normalized.score as f32)
    );
    assert_eq!(json["grade"], normalized.grade.as_str());
}

#[test]
fn foundation_invalid_performance_values_cannot_be_measured_successes() {
    use auditmysite::performance::{calculate_performance_score, VitalMetric};
    for value in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let metric = VitalMetric::new(value, 1.0, 2.0);
        let vitals = WebVitals {
            lcp: Some(metric.clone()),
            fcp: Some(metric.clone()),
            cls: Some(metric.clone()),
            tbt: Some(metric.clone()),
            speed_index: Some(metric),
            ..Default::default()
        };
        let score = calculate_performance_score(&vitals, None);
        assert_eq!(score.metrics_available, 0, "invalid measurement {value}");
        assert!(!score.has_core_metrics());
    }
}

#[test]
fn foundation_coverage_requires_execution_and_no_unresolved_result() {
    use auditmysite::wcag::{NotRun, RuleRun};
    let mut report = AuditReport::new(
        "https://example.com".into(),
        WcagLevel::AA,
        WcagResults::new(),
        1,
    );
    assert_eq!(
        normalize(&report)
            .normalized
            .principle_coverage
            .perceivable
            .passed,
        0
    );
    assert_eq!(
        normalize(&report)
            .normalized
            .principle_coverage
            .perceivable
            .ratio,
        0.0
    );
    let runs = &mut report.accessibility.wcag_results.rule_outcomes;
    runs.push(
        RuleRun::ran("images/alt-missing", 0)
            .with_wcag(["1.1.1"])
            .in_viewport("desktop"),
    );
    runs.push(
        RuleRun::not_run("images/alt-missing", NotRun::Errored)
            .with_wcag(["1.1.1"])
            .in_viewport("mobile"),
    );
    let coverage = normalize(&report).normalized.principle_coverage.perceivable;
    assert_eq!((coverage.passed, coverage.total), (0, 1));
    report.accessibility.wcag_results.rule_outcomes.pop();
    assert_eq!(
        normalize(&report)
            .normalized
            .principle_coverage
            .perceivable
            .passed,
        1
    );
    report
        .accessibility
        .wcag_results
        .warnings
        .push(Violation::new(
            "1.1.1",
            "Review",
            WcagLevel::A,
            Severity::Low,
            "review required",
            "image",
        ));
    assert_eq!(
        normalize(&report)
            .normalized
            .principle_coverage
            .perceivable
            .passed,
        0
    );
}

#[test]
fn foundation_unmeasured_html_never_contributes_to_a_score() {
    let mut report = make_full_report();
    let html = report.html_conform.as_mut().unwrap();
    html.checked = false;
    html.score = 100;
    let normalized = normalize(&report).normalized;
    let entry = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "HTML Conformance")
        .unwrap();
    assert_eq!(entry.measurement_type, "not_measured");
    assert!(!entry.contributes_to_overall);
    assert_eq!(entry.weight_pct, 0);
    let json = serde_json::to_value(UnifiedReport::single(&normalize(&report), &report)).unwrap();
    let entry = json["pages"][0]["module_scores"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "HTML Conformance")
        .unwrap();
    assert_eq!(entry["contributes_to_overall"], false);
}

#[test]
fn foundation_batch_errors_prevent_complete_and_pass() {
    use auditmysite::audit::{compute_batch_verdict, AuditQualityStatus, BatchError, Verdict};
    let clean = AuditReport::new(
        "https://example.com".into(),
        WcagLevel::AA,
        WcagResults::new(),
        1,
    );
    for successful in [0, 1, 3] {
        let batch = BatchReport::from_reports(
            vec![clean.clone(); successful],
            vec![BatchError {
                url: "https://example.com/unreachable".into(),
                error: "timeout".into(),
                blocked_reason: None,
            }],
            1,
        );
        assert_ne!(
            batch.summary.audit_quality.status,
            AuditQualityStatus::Complete
        );
        assert_ne!(
            compute_batch_verdict(&batch.summary, &Default::default()).verdict,
            Verdict::Pass
        );
        let json: serde_json::Value =
            serde_json::from_str(&auditmysite::output::format_json_batch(&batch, false).unwrap())
                .unwrap();
        assert_eq!(json["audit_quality"]["qualified_results"], true);
        assert_eq!(json["summary"]["attempted_url_count"], successful + 1);
    }
}

#[test]
fn foundation_pass_policy_is_the_same_for_raw_and_normalized_results() {
    use auditmysite::audit::{compute_verdict, Verdict};
    for severity in [
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ] {
        let mut wcag = WcagResults::new();
        wcag.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                severity,
                "Missing alt",
                "image",
            )
            .with_rule_id("images/alt-missing"),
        );
        let report = AuditReport::new("https://example.com".into(), WcagLevel::AA, wcag, 1);
        let normalized = normalize(&report).normalized;
        assert_eq!(
            report.passed(),
            compute_verdict(&normalized, &Default::default()).verdict == Verdict::Pass
        );
    }
}

#[test]
fn foundation_viewport_blending_cannot_hide_critical_barriers() {
    use auditmysite::audit::{ViewportScoreSet, ViewportScores};
    let mut report = make_full_report();
    for (desktop, mobile) in [(49, 100), (100, 49)] {
        report.viewport_scores = Some(ViewportScores {
            desktop: ViewportScoreSet {
                accessibility: desktop,
                performance: None,
                overall: desktop,
            },
            mobile: ViewportScoreSet {
                accessibility: mobile,
                performance: None,
                overall: mobile,
            },
            weighted_overall: ((desktop as f64 * 0.3) + mobile as f64 * 0.7).round() as u32,
        });
        let normalized = normalize(&report).normalized;
        assert!(
            normalized.score <= 49,
            "critical finding survived but score was {}",
            normalized.score
        );
        assert_eq!(
            normalized.grade,
            auditmysite::audit::AccessibilityScorer::calculate_grade(normalized.score as f32)
        );
    }
}

#[test]
fn foundation_published_weights_reconstruct_scores_for_every_module_subset() {
    use auditmysite::audit::{ViewportScoreSet, ViewportScores};
    // Independent oracle: the published product weights, including subsets
    // and boundary values. Viewport diagnostic values must not change them.
    for mask in 0..32 {
        for a11y in [0, 49, 70, 89, 100] {
            let mut report = make_full_report();
            report.accessibility.wcag_results = WcagResults::new();
            report.accessibility.score = a11y as f32;
            report.viewport_scores = Some(ViewportScores {
                desktop: ViewportScoreSet {
                    accessibility: a11y,
                    performance: Some(0),
                    overall: 0,
                },
                mobile: ViewportScoreSet {
                    accessibility: a11y,
                    performance: Some(100),
                    overall: 100,
                },
                weighted_overall: 70,
            });
            if mask & 1 == 0 {
                report.performance = None;
            }
            if mask & 2 == 0 {
                report.discoverability.seo = None;
            }
            if mask & 4 == 0 {
                report.security = None;
            }
            if mask & 8 == 0 {
                report.experience.mobile = None;
            }
            if mask & 16 == 0 {
                report.html_conform = None;
            }
            let scores = [
                (Some(a11y), 40u32),
                (report.performance.as_ref().map(|p| p.score.overall), 20),
                (report.discoverability.seo.as_ref().map(|s| s.score), 15),
                (report.security.as_ref().map(|s| s.score), 10),
                (report.experience.mobile.as_ref().map(|m| m.score), 10),
                (report.html_conform.as_ref().map(|h| h.score), 5),
            ];
            let (sum, denominator) = scores
                .into_iter()
                .filter_map(|(s, w)| s.map(|s| (s, w)))
                .fold((0, 0), |(sum, weight), (s, w)| (sum + s * w, weight + w));
            let expected = (sum as f64 / denominator as f64).round() as u32;
            let normalized = normalize(&report).normalized;
            assert_eq!(
                normalized.overall_score, expected,
                "subset {mask}, a11y {a11y}"
            );
        }
    }
}

#[test]
fn foundation_lint_rejects_consistent_looking_but_false_reports() {
    let report = make_full_report();
    let base = serde_json::to_value(UnifiedReport::single(&normalize(&report), &report)).unwrap();
    let mut wrong_score = base.clone();
    for value in ["score", "overall_score"] {
        wrong_score["summary"][value] = 99.into();
    }
    wrong_score["pages"][0]["overall_score"] = 99.into();
    let mut lost_rule = base.clone();
    lost_rule["pages"][0]["rule_outcomes"] = serde_json::json!([
        {"rule_id":"aria/role-invalid","wcag":["4.1.2"],"findings":1,"viewport":"desktop"}
    ]);
    let mut missing_check = base.clone();
    missing_check["audit_scope"] = serde_json::json!({"requested_modules":["security"]});
    missing_check["pages"][0]["module_runs"] = serde_json::json!([]);
    let mut failed_page = base;
    failed_page["errors"] =
        serde_json::json!([{"url":"https://example.com/failed","error":"timeout"}]);
    for (json, expected) in [
        (wrong_score, "overall_score_matches_module_weights"),
        (lost_rule, "rule_findings_preserved"),
        (missing_check, "requested_module_has_outcome"),
        (failed_page, "audit_completeness_matches_execution"),
    ] {
        let lint = auditmysite::lint::lint(&json, None);
        assert!(
            lint.findings.iter().any(|f| f.check_id == expected),
            "{expected}: {:?}",
            lint.findings
        );
    }
}

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
            si_score: Some(10),
            metrics_available: 5,
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
        measurement_warnings: Vec::new(),
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
        sourcemap_leaks: Default::default(),
        fetch: Default::default(),
    }
}

fn make_html_conform() -> auditmysite::html_conform::HtmlConformAnalysis {
    auditmysite::html_conform::HtmlConformAnalysis {
        score: 90,
        checked: true,
        error_count: 0,
        warning_count: 1,
        info_count: 0,
        distinct_defect_count: 1,
        findings: vec![],
        raw_html: None,
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
    .with_html_conform(make_html_conform())
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
        .normalized
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
        module_names.contains(&"HTML Conformance"),
        "HTML Conformance missing from module_scores"
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
        normalized.normalized.module_scores.len(),
        1,
        "Only Accessibility should be present without --full"
    );
    assert_eq!(normalized.normalized.module_scores[0].name, "Accessibility");
}

// ─── Overall Score Consistency ─────────────────────────────────────

#[test]
fn test_overall_score_equals_weighted_average() {
    let report = make_full_report();
    let normalized = normalize(&report);

    // Calculate expected weighted average from modules that contribute to overall_score.
    let weighted_sum: f64 = normalized
        .normalized
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.score as f64 * m.weight_pct as f64)
        .sum();
    let total_weight: f64 = normalized
        .normalized
        .module_scores
        .iter()
        .filter(|m| m.contributes_to_overall)
        .map(|m| m.weight_pct as f64)
        .sum();
    let expected = (weighted_sum / total_weight).round() as u32;

    assert_eq!(
        normalized.normalized.overall_score,
        expected,
        "overall_score ({}) != weighted average of module_scores ({}). Modules: {:?}",
        normalized.normalized.overall_score,
        expected,
        normalized
            .normalized
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
        normalized.normalized.overall_score, normalized.normalized.score,
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

    for f in normalized
        .normalized
        .findings
        .iter()
        .filter(|f| f.category == "wcag")
    {
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

    assert_eq!(
        normalized.normalized.severity_counts.critical,
        findings_critical
    );
    assert_eq!(normalized.normalized.severity_counts.high, findings_high);
    assert_eq!(
        normalized.normalized.severity_counts.medium,
        findings_medium
    );
    assert_eq!(normalized.normalized.severity_counts.low, findings_low);
    assert_eq!(
        normalized.normalized.severity_counts.total,
        findings_critical + findings_high + findings_medium + findings_low
    );

    assert_eq!(
        normalized.normalized.occurrence_counts.critical,
        occ_critical
    );
    assert_eq!(normalized.normalized.occurrence_counts.high, occ_high);
    assert_eq!(normalized.normalized.occurrence_counts.medium, occ_medium);
    assert_eq!(normalized.normalized.occurrence_counts.low, occ_low);
    assert_eq!(
        normalized.normalized.occurrence_counts.total,
        occ_critical + occ_high + occ_medium + occ_low
    );
}

// ─── WCAG/all-category count separation ──────────────────────────

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
fn test_json_separates_wcag_counts_from_all_category_findings() {
    let report = make_report_with_seo_finding();
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let page = &unified.pages[0];

    let total_rules = normalized
        .normalized
        .findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let total_occurrences: usize = normalized
        .normalized
        .findings
        .iter()
        .map(|f| f.occurrence_count)
        .sum();
    let seo_rules = normalized
        .normalized
        .findings
        .iter()
        .filter(|f| f.category == "seo")
        .count();
    assert!(seo_rules > 0, "test report must contain SEO findings");

    let wcag_rules = normalized
        .normalized
        .findings
        .iter()
        .filter(|f| f.category == "wcag")
        .count();
    let wcag_occurrences: usize = normalized
        .normalized
        .findings
        .iter()
        .filter(|f| f.category == "wcag")
        .map(|f| f.occurrence_count)
        .sum();

    assert_eq!(page.violated_rule_count, wcag_rules);
    assert_eq!(page.violation_count, wcag_occurrences);
    assert_eq!(page.occurrence_counts.total, wcag_occurrences);
    assert_eq!(page.severity_counts.total, wcag_rules);
    assert_eq!(page.finding_count, normalized.normalized.findings.len());
    assert_eq!(page.finding_occurrence_count, total_occurrences);
    assert_eq!(total_rules, page.finding_count);
    assert_eq!(page.occurrence_counts_scope, "wcag_only");
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
        normalized.normalized.findings.len(),
        "fix_guidance count should match findings count"
    );

    for (guidance, finding) in fix_guidance.iter().zip(&normalized.normalized.findings) {
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

        // Grade must be consistent with score, via the same shared BandSet
        // the production code uses (audit::scoring::AccessibilityScorer::calculate_grade) —
        // re-deriving the thresholds inline here would silently drift from
        // the real implementation (#506).
        let grade_from_score = LETTER_GRADE.label(normalized.normalized.score as f32, false);

        assert_eq!(
            normalized.normalized.grade, grade_from_score,
            "Grade '{}' doesn't match score {} (expected '{}')",
            normalized.normalized.grade, normalized.normalized.score, grade_from_score
        );
    }
}

// ─── Module Weight Tests ───────────────────────────────────────────

#[test]
fn test_module_weights_sum_to_expected() {
    let report = make_full_report();
    let normalized = normalize(&report);

    let total_weight: u32 = normalized
        .normalized
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

    for m in &normalized.normalized.module_scores {
        let expected_weight = match m.name.as_str() {
            "Accessibility" => 40,
            "Performance" => 20,
            // SEO gave up 5 points to HTML Conformance when that module
            // rejoined the overall score (2026-09-16): the two answer the same
            // question, whether a crawler can parse the document reliably.
            "SEO" => 15,
            "HTML Conformance" => 5,
            "Security" => 10,
            "Mobile" => 10,
            // Indicator modules do not contribute to the overall score, so their
            // displayed weight is 0 (#447).
            "UX" | "Journey" | "Best Practices" | "Dark Mode" | "AI Visibility"
            | "Source Quality" | "Tech Stack" => 0,
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

// ─── Risk Assessment Tests ────────────────────────────────────────

#[test]
fn test_risk_critical_with_level_a_violations() {
    // Full report has 5 Critical + 3 Medium, all WCAG Level A
    let report = make_full_report();
    let normalized = normalize(&report);

    assert_eq!(
        normalized.normalized.risk.level,
        auditmysite::audit::normalized::RiskLevel::Critical,
        "Expected Critical risk for report with critical Level A violations"
    );
    assert!(
        normalized.normalized.risk.legal_flags > 0,
        "Should have legal flags for Level A violations"
    );
    assert!(
        normalized.normalized.risk.blocking_issues > 0,
        "Should have blocking issues for 4.1.2 violations"
    );
}

#[test]
fn normalized_report_survives_cache_round_trip() {
    // The --reuse-cache path serializes the NormalizedReport to audit.json and
    // deserializes it on a cache hit. Every field skipped when empty/None must
    // carry #[serde(default)] or the round-trip fails and the cache is unusable
    // (#405). A full round-trip catches absent Option blocks (viewport_scores,
    // interpretation, …).
    let report = make_full_report();
    let normalized = normalize(&report).normalized;
    let json = serde_json::to_string(&normalized).expect("serialize NormalizedReport");
    let _back: auditmysite::audit::normalized::NormalizedReport =
        serde_json::from_str(&json).expect("NormalizedReport must round-trip through JSON");

    // Findings with empty user_impact/technical_impact (e.g. SEO findings) skip
    // those keys; deserialization must default them rather than error.
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    if let Some(findings) = v.get_mut("findings").and_then(|f| f.as_array_mut()) {
        for f in findings.iter_mut() {
            if let Some(obj) = f.as_object_mut() {
                obj.remove("user_impact");
                obj.remove("technical_impact");
            }
        }
    }
    let stripped = serde_json::to_string(&v).unwrap();
    let _back2: auditmysite::audit::normalized::NormalizedReport = serde_json::from_str(&stripped)
        .expect("findings with absent user_impact/technical_impact must deserialize");
}

#[test]
fn test_risk_breadth_critical_without_critical_occurrences() {
    // Regression for #457: 4 distinct WCAG Level A rules, all High severity (no
    // Critical occurrences). Breadth of legal exposure must reach Critical and
    // be attributed to Legal Compliance — a site like this previously ranked
    // *below* one with a single flag but volume-triggered Critical.
    let mut results = WcagResults::new();
    for (rule, name) in [
        ("1.1.1", "Non-text Content"),
        // Not 2.4.4: its rule needs a manual check and is no legal flag
        // since plan 74.
        ("4.1.2", "Name, Role, Value"),
        ("2.4.6", "Headings"),
        ("3.3.2", "Labels"),
    ] {
        results.add_violation(Violation::new(
            rule,
            name,
            WcagLevel::A,
            Severity::High,
            "issue",
            "node",
        ));
    }
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        100,
    );
    let normalized = normalize(&report);

    assert_eq!(normalized.normalized.risk.critical_issues, 0);
    assert_eq!(normalized.normalized.risk.legal_flags, 4);
    assert_eq!(
        normalized.normalized.risk.level,
        auditmysite::audit::normalized::RiskLevel::Critical,
        "4 distinct Level-A High rules = systemic legal exposure → Critical"
    );
    assert_eq!(normalized.normalized.risk.driven_by, "Legal Compliance");
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
        normalized.normalized.risk.level,
        auditmysite::audit::normalized::RiskLevel::Low,
        "Expected Low risk for report without violations"
    );
    assert_eq!(normalized.normalized.risk.critical_issues, 0);
    assert_eq!(normalized.normalized.risk.legal_flags, 0);
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

    // Score is capped at 49 when any Critical violation is present (semantic
    // score cap in AccessibilityScorer::calculate_score). The point of this
    // test is that Risk is Critical even though Score is not 0 — they are
    // independent signals.
    assert!(
        normalized.normalized.score > 0,
        "Score should be > 0 even with critical violations (got {})",
        normalized.normalized.score
    );
    assert!(
        normalized.normalized.score <= 49,
        "Critical violations cap the score at 49 (got {})",
        normalized.normalized.score
    );
    // A single distinct legal flag (3× the same rule) is High, not Critical —
    // Critical is reserved for systemic exposure (≥3 distinct WCAG-A barriers
    // or ≥5 critical occurrences). The point of this test is that risk is
    // elevated even though the score is low: score != risk.
    assert_eq!(
        normalized.normalized.risk.level,
        auditmysite::audit::normalized::RiskLevel::High,
        "Risk should be elevated (High) — score != risk"
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
            .normalized
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
        Some(normalized.normalized.score as u64),
        "AI JSON score must use NormalizedReport.score as primary WCAG score"
    );
    assert_eq!(
        parsed["overall_score"].as_u64(),
        Some(normalized.normalized.overall_score as u64),
        "AI JSON overall_score must use NormalizedReport.overall_score as secondary weighted score"
    );
}

#[test]
fn test_batch_json_average_score_is_accessibility_with_overall_separate() {
    let reports = vec![make_full_report()];
    let normalized = normalize(&reports[0]);
    let expected_overall = normalized.normalized.overall_score as u64;
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
        .normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Journey");
    assert!(journey_entry.is_some(), "Journey must be in module_scores");
    assert_eq!(
        journey_entry.unwrap().weight_pct,
        0,
        "Journey is an indicator module — weight must be 0% (#447)"
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
    report.accessibility.score = 79.6;

    let normalized = normalize(&report);
    let batch = BatchReport::from_reports(vec![report.clone()], vec![], 100);

    assert_eq!(normalized.normalized.score, 80);
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
        "html_conform",
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
    report.discoverability.source_quality = Some(analyze_source_quality(&base));
    report.discoverability.ai_visibility = Some(analyze_ai_visibility(&base));
    report.discoverability.content_visibility = Some(analyze_content_visibility(&base));
    report.experience.dark_mode = Some(DarkModeAnalysis {
        supported: false,
        score: 0,
        detection_methods: vec![],
        class_based_dark_mode: false,
        color_scheme_css: false,
        meta_color_scheme: None,
        meta_theme_color_dark: false,
        css_custom_properties: 0,
        dark_contrast_violations: 0,
        light_only_violations: 0,
        dark_only_violations: 0,
        contrast_violations: vec![],
        print: Default::default(),
        forced_colors: Default::default(),
        vision_deficiency: Default::default(),
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

    // #784: every heuristic module carries its signal contract and says it
    // is not externally validated.
    let mut heuristic = 0;
    for (key, module) in modules.as_object().unwrap() {
        if module["measurement_type"] != "heuristic" {
            continue;
        }
        heuristic += 1;
        let contract = &module["signal_contract"];
        assert_eq!(
            contract["format"], "auditmysite.signal-contract/1",
            "{key} lacks its signal contract"
        );
        assert_eq!(contract["contract"]["id"], key.as_str());
        assert_eq!(contract["contract"]["validation"]["status"], "unvalidated");
        assert!(!contract["contract"]["not_established"]
            .as_array()
            .unwrap()
            .is_empty());
    }
    assert!(heuristic >= 3, "only {heuristic} heuristic modules checked");
}

#[test]
fn test_json_report_includes_report_artifact_fields() {
    let mut report = make_full_report();
    report.discoverability.tech_stack = Some(auditmysite::tech_stack::TechStackAnalysis {
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
        audited_stacks: Vec::new(),
    });
    report.experience.budget_violations = vec![auditmysite::audit::BudgetViolation {
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
        score: Some(72),
    }];
    report.patterns = Some(auditmysite::patterns::PatternAnalysis {
        recognized: vec![auditmysite::patterns::RecognizedPattern {
            pattern: "MainNavigation".to_string(),
            message: "Main navigation recognized".to_string(),
            confidence: auditmysite::patterns::PatternConfidence::Strong,
            detail: None,
        }],
        violations: vec![],
        journey_candidates: vec![],
        help_mechanisms: vec![],
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

    let core = [
        "Accessibility",
        "Performance",
        "SEO",
        "Security",
        "Mobile",
        "HTML Conformance",
    ];
    let supplemental = ["UX", "Journey"];

    for m in &normalized.normalized.module_scores {
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
            // HTML Conformance reports "measured", not "heuristic": schema
            // conformance is a checkable property of the document, and the
            // upstream false-positive gap that had it paused from the score
            // was closed in html-conform 0.2.1 (2026-09-16).
            "UX" | "Journey" => {
                assert_eq!(m.measurement_type, "heuristic")
            }
            _ => assert_eq!(m.measurement_type, "measured"),
        }
    }

    // Core weights must sum to exactly 100 so overall_score is a proper percentage
    let core_weight: u32 = normalized
        .normalized
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

// ─── JSON ↔ PDF parity (plans 58–60) ───────────────────────────────

/// Mirrors dm.de on 2026-09-28: a medium finding with many occurrences, and
/// several distinct 4.1.2 rules with few. `reverse` feeds the same
/// violations in the opposite order.
fn make_parity_report(reverse: bool) -> AuditReport {
    let mut violations = Vec::new();
    for i in 0..20 {
        violations.push(
            Violation::new(
                "1.3.1",
                "Info and Relationships",
                WcagLevel::A,
                Severity::Medium,
                "Content uses visual formatting instead of semantic HTML",
                format!("node-sem-{i}"),
            )
            .with_selector(format!("div.sem-{i}")),
        );
    }
    for (axe_id, count) in [("aria-valid-attr-value", 2), ("aria-allowed-attr", 1)] {
        for i in 0..count {
            violations.push(
                Violation::new(
                    "4.1.2",
                    "Name, Role, Value",
                    WcagLevel::A,
                    Severity::High,
                    format!("{axe_id} violation"),
                    format!("node-{axe_id}-{i}"),
                )
                .with_rule_id(axe_id)
                .with_selector(format!("div.{axe_id}-{i}")),
            );
        }
    }
    if reverse {
        violations.reverse();
    }
    let mut results = WcagResults::new();
    for v in violations {
        results.add_violation(v);
    }
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        1000,
    )
}

#[test]
fn pdf_top_measures_follow_json_top_actions() {
    let report = make_parity_report(false);
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let vm = build_view_model(&normalized, &ReportConfig::default());

    let json_top: Vec<&str> = unified
        .summary
        .top_actions
        .iter()
        .map(|a| a.rule_id.as_str())
        .collect();
    let pdf_top: Vec<&str> = vm
        .findings
        .top_findings
        .iter()
        .map(|g| g.rule_id.as_str())
        .collect();
    assert_eq!(pdf_top, json_top[..pdf_top.len()]);
    // Urgency first: the high findings lead, the 20 medium occurrences follow.
    assert_eq!(
        json_top,
        [
            "a11y.aria_valid_attr_value.invalid",
            "a11y.aria_allowed_attr.invalid",
            "a11y.structure.missing",
        ]
    );
}

#[test]
fn pdf_finding_groups_are_the_json_findings() {
    let report = make_parity_report(false);
    let normalized = normalize(&report);
    let vm = build_view_model(&normalized, &ReportConfig::default());

    let json: Vec<(&str, usize)> = normalized
        .normalized
        .findings
        .iter()
        .map(|f| (f.rule_id.as_str(), f.occurrence_count))
        .collect();
    let pdf: Vec<(&str, usize)> = vm
        .findings
        .all_findings
        .iter()
        .map(|g| (g.rule_id.as_str(), g.occurrence_count))
        .collect();
    // Same rules, same counts, same order — no criterion-level merge of the
    // two 4.1.2 rules into one "missing name/role" group.
    assert_eq!(pdf, json);
    // Urgency first all the way down: severity never rises along the list.
    assert!(vm
        .findings
        .all_findings
        .windows(2)
        .all(|w| w[0].severity >= w[1].severity));
}

#[test]
fn finding_order_does_not_depend_on_input_order() {
    let forward = make_parity_report(false);
    let reversed = make_parity_report(true);
    let a = normalize(&forward);
    let b = normalize(&reversed);
    let ids = |r: &auditmysite::audit::AuditContext<'_>| -> Vec<String> {
        r.normalized
            .findings
            .iter()
            .map(|f| f.rule_id.clone())
            .collect()
    };
    assert_eq!(ids(&a), ids(&b));
}

#[test]
fn batch_top_actions_match_pdf_and_aggregate_across_pages() {
    use auditmysite::output::builder::build_batch_presentation;

    let mut a = make_parity_report(false);
    a.url = "https://example.com/a".to_string();
    let mut b = make_parity_report(true);
    b.url = "https://example.com/b".to_string();
    let batch = BatchReport::from_reports(vec![a, b], vec![], 100);

    let json: serde_json::Value =
        serde_json::from_str(&format_json_batch(&batch, true).expect("batch JSON must render"))
            .expect("batch JSON must parse");
    let actions = json["summary"]["top_actions"]
        .as_array()
        .expect("top_actions");
    let json_rows: Vec<(String, u64, u64)> = actions
        .iter()
        .map(|a| {
            (
                a["rule_id"].as_str().unwrap().to_string(),
                a["occurrence_count"].as_u64().unwrap(),
                a["url_count"].as_u64().unwrap(),
            )
        })
        .collect();

    let pres = build_batch_presentation(&batch);
    let pdf_rows: Vec<(String, u64, u64)> = pres
        .top_issues
        .iter()
        .take(8)
        .map(|g| {
            (
                g.rule_id.clone(),
                g.occurrence_count as u64,
                g.affected_urls.len() as u64,
            )
        })
        .collect();

    // One row per rule, summed over both pages, same order as the PDF.
    assert_eq!(
        json_rows,
        [
            ("a11y.aria_valid_attr_value.invalid".to_string(), 4, 2),
            ("a11y.aria_allowed_attr.invalid".to_string(), 2, 2),
            ("a11y.structure.missing".to_string(), 40, 2),
        ]
    );
    assert_eq!(pdf_rows, json_rows);
    // The impact sentence quotes the batch-wide count, not one page's.
    let structure = actions
        .iter()
        .find(|a| a["rule_id"] == "a11y.structure.missing")
        .unwrap();
    assert!(
        structure["expected_impact"]
            .as_str()
            .unwrap()
            .contains("40 occurrences"),
        "{}",
        structure["expected_impact"]
    );
}

// The domain model must reject contradictions before any formatter exists.
#[test]
fn domain_contract_rejects_mutated_facts_before_presentation() {
    let valid = normalize(&make_full_report()).normalized;
    valid
        .validate()
        .expect("fixture must satisfy the domain contract");
    let mut cases: Vec<(&str, auditmysite::audit::NormalizedReport)> = Vec::new();
    let mut changed = valid.clone();
    changed.overall_score = (changed.overall_score + 1) % 101;
    cases.push(("overall_score", changed));
    let mut changed = valid.clone();
    changed.grade = "A".into();
    if changed.grade == valid.grade {
        changed.grade = "F".into();
    }
    cases.push(("grade", changed));
    let mut changed = valid.clone();
    changed.severity_counts.total += 1;
    cases.push(("severity_counts", changed));
    let mut changed = valid.clone();
    changed.occurrence_counts.total += 1;
    cases.push(("occurrence_counts", changed));
    let mut changed = valid.clone();
    changed.module_scores.push(changed.module_scores[0].clone());
    cases.push(("module_identity", changed));
    let mut changed = valid.clone();
    changed.module_scores[0].score = 101;
    cases.push(("score_range", changed));
    let mut changed = valid.clone();
    for module in &mut changed.module_scores {
        module.score = u32::MAX;
        module.weight_pct = u32::MAX;
    }
    cases.push(("score_range", changed));
    let mut changed = valid.clone();
    changed.module_scores[0].measurement_type = "not_measured".into();
    cases.push(("weighted_measurement", changed));
    let mut changed = valid.clone();
    changed.module_scores[0].weight_pct += 1;
    cases.push(("module_weight", changed));
    let mut changed = valid.clone();
    changed
        .execution
        .scope
        .requested_modules
        .push("never_executed".into());
    cases.push(("requested_execution", changed));
    let mut changed = valid.clone();
    changed.execution.quality.qualified_results = !changed.execution.quality.qualified_results;
    cases.push(("quality_qualification", changed));
    for (expected, report) in cases {
        let error = report.validate().expect_err(expected);
        assert!(
            error.violations.iter().any(|v| v.code == expected),
            "{expected}: {error}"
        );
    }
}

#[test]
fn domain_contract_collects_errors_and_covers_batch_aggregation() {
    use auditmysite::audit::validate_batch;
    let mut report = normalize(&make_full_report()).normalized;
    report.grade = "INVALID".into();
    report.overall_score = 999;
    let error = report.validate().unwrap_err();
    assert!(error.violations.len() >= 2);

    let clean = AuditReport::new(
        "https://example.com".into(),
        WcagLevel::AA,
        WcagResults::new(),
        1,
    );
    let batch = BatchReport::from_reports(vec![clean], vec![], 1);
    validate_batch(&batch).unwrap();
    for field in ["average_score", "passed", "total_urls"] {
        let mut changed = batch.clone();
        match field {
            "average_score" => changed.summary.average_score = 0.0,
            "passed" => changed.summary.passed = 0,
            _ => changed.summary.total_urls = 2,
        }
        assert!(validate_batch(&changed).is_err(), "{field}");
    }
}

#[test]
fn domain_contract_accepts_honest_partial_results_and_rejects_false_completeness() {
    use auditmysite::audit::{AuditQualityStatus, ExecutionStatus, ModuleRun, SubcheckRun};
    let mut normalized = normalize(&make_full_report()).normalized;
    normalized
        .execution
        .scope
        .requested_modules
        .push("optional_probe".into());
    normalized.execution.module_runs.push(ModuleRun {
        module: "optional_probe".into(),
        status: ExecutionStatus::Partial,
        subchecks: vec![SubcheckRun {
            subcheck: "detail".into(),
            status: ExecutionStatus::Failed,
            reason_code: Some("probe_timeout".into()),
            ..Default::default()
        }],
        ..Default::default()
    });
    normalized.execution.quality.status = AuditQualityStatus::Partial;
    normalized.execution.quality.qualified_results = true;
    normalized.validate().unwrap();
    normalized.execution.quality.status = AuditQualityStatus::Complete;
    normalized.execution.quality.qualified_results = false;
    assert!(normalized
        .validate()
        .unwrap_err()
        .violations
        .iter()
        .any(|v| v.code == "execution_completeness"));
    normalized.execution.module_runs[0].status = ExecutionStatus::NotRequested;
    assert!(normalized
        .validate()
        .unwrap_err()
        .violations
        .iter()
        .any(|v| v.code == "requested_execution"));
}

#[test]
fn domain_contract_blocks_alternative_exports_of_invalid_cached_data() {
    let mut normalized = normalize(&make_full_report()).normalized;
    normalized.overall_score = 101;
    assert!(auditmysite::output::format_summary(&normalized).is_err());
    assert!(auditmysite::output::format_sarif(&[&normalized]).is_err());
    assert!(auditmysite::output::format_json_cached(&normalized, false).is_err());
}

#[test]
fn domain_contract_distinguishes_capture_gaps_from_intentional_scope() {
    use auditmysite::audit::frames::{FrameCoverage, FrameEntry, FrameSkipReason};
    use auditmysite::interaction::stability::{StabilityProvenance, StabilityStatus};
    let valid = normalize(&make_full_report()).normalized;
    for (reason, gap) in [
        (Some(FrameSkipReason::Hidden), false),
        (Some(FrameSkipReason::Excluded), false),
        (Some(FrameSkipReason::CrossOrigin), true),
        (Some(FrameSkipReason::AxTreeFailed), true),
        (Some(FrameSkipReason::DocumentUnavailable), true),
        (None, true),
    ] {
        let mut report = valid.clone();
        report.execution.frames = Some(FrameCoverage {
            skipped: vec![FrameEntry {
                selector: "iframe".into(),
                reason,
                ..Default::default()
            }],
            ..Default::default()
        });
        assert_eq!(report.validate().is_err(), gap, "{reason:?}");
    }
    for (status, gap) in [
        (StabilityStatus::Stable, false),
        (StabilityStatus::ReadySignal, false),
        (StabilityStatus::OngoingAnimation, false),
        (StabilityStatus::Fallback, true),
        (StabilityStatus::BudgetExhausted, true),
    ] {
        let mut report = valid.clone();
        report
            .execution
            .navigation
            .stability
            .push(StabilityProvenance {
                status,
                ..Default::default()
            });
        assert_eq!(report.validate().is_err(), gap, "{status:?}");
    }
}

#[test]
fn domain_contract_detects_lost_findings_even_when_the_remaining_counts_agree() {
    use auditmysite::wcag::RuleRun;
    let mut raw = make_full_report();
    let id = raw.accessibility.wcag_results.violations[0]
        .rule_id
        .clone()
        .unwrap_or_else(|| raw.accessibility.wcag_results.violations[0].rule.clone());
    raw.accessibility
        .wcag_results
        .rule_outcomes
        .push(RuleRun::ran(id, 1));
    let mut normalized = normalize(&raw).normalized;
    normalized.validate().unwrap();
    normalized.findings.clear();
    normalized.severity_counts = Default::default();
    normalized.occurrence_counts = Default::default();
    assert!(normalized
        .validate()
        .unwrap_err()
        .violations
        .iter()
        .any(|v| v.code == "rule_findings_preserved"));
}

#[cfg(feature = "pdf")]
#[test]
fn domain_contract_rejects_invalid_pdf_inputs_before_rendering() {
    let mut report = make_full_report();
    report.security.as_mut().unwrap().score = 101;
    let error = auditmysite::output::generate_pdf(&report, &ReportConfig::default()).unwrap_err();
    assert!(error.to_string().contains("score_range"));
    let mut batch = BatchReport::from_reports(vec![make_full_report()], vec![], 1);
    batch.summary.average_score = 999.0;
    let error =
        auditmysite::output::generate_batch_pdf(&batch, &ReportConfig::default()).unwrap_err();
    assert!(error.to_string().contains("batch_aggregation"));
}
