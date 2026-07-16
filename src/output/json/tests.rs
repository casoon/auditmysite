use super::*;
use crate::audit::normalized::normalize;
use crate::cli::WcagLevel;
use crate::wcag::WcagResults;

fn first_page(report: &UnifiedReport) -> &PageEntry {
    &report.pages[0]
}

#[test]
fn test_single_envelope_shape() {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        500,
    );
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);

    assert_eq!(unified.schema_version, "2.0");
    assert_eq!(unified.report_type, "single");
    assert_eq!(unified.pages.len(), 1);
    assert_eq!(unified.summary.url_count, 1);

    let output = unified.to_json(true).unwrap();
    assert!(output.contains("\"schema_version\": \"2.0\""));
    assert!(output.contains("\"report_type\": \"single\""));
    assert!(output.contains("\"metric_context\""));
    assert!(output.contains("\"higher_is_better\": true"));
    assert!(output.contains("example.com"));
    assert!(output.contains("\"accessibility_score\": 100"));
}

#[test]
fn test_single_summary_fields_present() {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        500,
    );
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);

    assert_eq!(
        unified.summary.accessibility_score,
        normalized.normalized.score
    );
    assert_eq!(
        unified.summary.overall_score,
        normalized.normalized.overall_score
    );
    assert_eq!(unified.summary.violation_count, 0);
    assert_eq!(unified.summary.finding_count, 0);
    assert_eq!(unified.summary.finding_occurrence_count, 0);
    assert_eq!(unified.summary.occurrence_counts_scope, "wcag_only");
    assert_eq!(unified.summary.passed_url_count, 1);
    assert_eq!(unified.summary.failed_url_count, 0);
}

#[test]
fn test_assessments_failures_en_status_and_artifact_manifest_are_serialized() {
    use crate::taxonomy::Severity;
    use crate::wcag::Violation;

    let mut results = WcagResults::new();
    results.add_warning(Violation::new(
        "1.3.1",
        "Structure warning",
        WcagLevel::A,
        Severity::Medium,
        "Verify the visual grouping.",
        "n1",
    ));
    results.add_not_testable(Violation::new(
        "1.1.1",
        "Manual alternative review",
        WcagLevel::A,
        Severity::Medium,
        "Verify the text alternative in context.",
        "n2",
    ));
    results.add_positive(Violation::new(
        "2.4.1",
        "Skip link present",
        WcagLevel::A,
        Severity::Low,
        "A skip link was detected.",
        "n3",
    ));
    results.rule_outcomes.push(crate::wcag::RuleOutcome {
        rule_id: "image_alt".to_string(),
        status: crate::wcag::RuleOutcomeStatus::Failed,
        wcag_criterion: Some("1.1.1".to_string()),
        viewport: Some("mobile".to_string()),
        reason_code: Some("dom_evaluation_failed".to_string()),
        finding_count: 0,
    });

    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        500,
    );
    report.accessibility.execution.quality = crate::audit::AuditQuality {
        status: crate::audit::AuditQualityStatus::Insufficient,
        qualified_results: true,
        failed_rule_checks: 1,
        partial_or_failed_modules: 0,
        reasons: vec!["rule_checks_failed".to_string()],
    };
    report.screen_reader_audit = Some(crate::screen_reader::build_sr_audit_report(
        &report.url,
        report.timestamp,
        &crate::AXTree::new(),
        "en",
        None,
    ));

    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let page = first_page(&unified);

    assert_eq!(page.accessibility_assessments.len(), 3);
    assert!(page
        .accessibility_assessments
        .iter()
        .any(|assessment| assessment.kind == "manual_review"));
    assert_eq!(unified.artifacts.len(), 1);
    assert_eq!(unified.artifacts[0].kind, "screen_reader_audit");

    let clause = page
        .detail
        .as_ref()
        .unwrap()
        .en301549_annex
        .clauses
        .iter()
        .find(|clause| clause.wcag == "1.1.1")
        .unwrap();
    assert_eq!(clause.status, En301549ClauseStatusKind::NotEvaluated);

    let output = unified.to_json(false).unwrap();
    assert!(!output.contains("/Users/"));
    assert!(!output.contains("stacktrace"));
}

#[test]
fn test_single_taxonomy_fields() {
    use crate::taxonomy::Severity;
    use crate::wcag::Violation;

    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        Severity::High,
        "Missing alt",
        "n1",
    ));

    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        500,
    );
    let normalized = normalize(&report);
    let output = format_json_normalized(&normalized, &report, true).unwrap();

    assert!(output.contains("\"dimension\""));
    assert!(output.contains("\"subcategory\""));
    assert!(output.contains("\"issue_class\""));
    assert!(output.contains("\"aggregation_key\""));
    assert!(output.contains("\"user_impact\""));
    assert!(output.contains("\"principle_coverage\""));
}

#[test]
fn test_single_score_matches_normalized() {
    use crate::taxonomy::Severity;
    use crate::wcag::Violation;

    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "1.1.1",
        "Alt",
        WcagLevel::A,
        Severity::High,
        "Missing",
        "n1",
    ));

    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        500,
    );
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let page = first_page(&unified);

    assert_eq!(page.accessibility_score, normalized.normalized.score);
    assert_eq!(page.grade, normalized.normalized.grade);
    assert_eq!(page.certificate, normalized.normalized.certificate);
}

#[test]
fn test_single_violations_match_severity_counts() {
    use crate::taxonomy::Severity;
    use crate::wcag::Violation;

    let mut results = WcagResults::new();
    for node in ["n1", "n2", "n3"] {
        results.add_violation(Violation::new(
            "1.1.1",
            "Alt",
            WcagLevel::A,
            Severity::High,
            "Missing alt",
            node,
        ));
    }
    results.add_violation(Violation::new(
        "1.4.3",
        "Contrast",
        WcagLevel::AA,
        Severity::High,
        "Low contrast",
        "n4",
    ));

    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        500,
    );
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let page = first_page(&unified);

    assert_eq!(
        page.violation_count,
        page.findings
            .iter()
            .filter(|finding| finding.category == "wcag")
            .map(|f| f.occurrence_count)
            .sum::<usize>()
    );
    for finding in &page.findings {
        assert!(!finding.occurrences.is_empty());
    }
}

#[test]
fn test_batch_envelope_shape() {
    use crate::audit::BatchReport;

    let reports = vec![
        AuditReport::new(
            "https://example.com/a".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        ),
        AuditReport::new(
            "https://example.com/b".to_string(),
            WcagLevel::AA,
            WcagResults::new(),
            100,
        ),
    ];
    let batch = BatchReport::from_reports(reports, vec![], 200);
    let unified = UnifiedReport::batch(&batch);

    assert_eq!(unified.report_type, "batch");
    assert_eq!(unified.pages.len(), 2);
    assert!(unified.audit_scope.is_some());
    assert!(unified.execution_environment.is_some());
    let site_analysis = unified
        .site_analysis
        .as_ref()
        .expect("batch site analysis present");
    for key in [
        "module_averages",
        "consistency",
        "page_types",
        "content_quality",
        "content_topics",
        "duplicates",
        "performance",
        "structured_data",
        "interactive_summary",
        "accessibility_assessments",
    ] {
        assert!(
            site_analysis.get(key).is_some(),
            "missing site_analysis.{key}"
        );
    }
    let consistency = &site_analysis["consistency"];
    for key in [
        "navigation",
        "headings",
        "canonical",
        "orphan_pages",
        "schema_graph",
    ] {
        assert!(consistency.get(key).is_some(), "missing consistency.{key}");
    }
    // Batch pages carry a compact detail with fix_guidance only (#256).
    // No new data is collected — it is derived from the findings already
    // normalized for each page; with no violations fix_guidance is empty.
    assert!(unified.pages.iter().all(|p| p.detail.is_some()));
    for page in &unified.pages {
        let detail = page.detail.as_ref().expect("batch page detail present");
        assert!(detail.fix_guidance.is_empty());
    }

    let output = unified.to_json(true).unwrap();
    assert!(output.contains("\"report_type\": \"batch\""));
    assert!(output.contains("\"schema_version\": \"2.0\""));
    // No sample metadata attached → the block is omitted.
    assert!(!output.contains("\"sample\""));
}

#[test]
fn test_batch_envelope_includes_sample_metadata() {
    use crate::audit::{BatchReport, SampleMetadata};

    let reports = vec![AuditReport::new(
        "https://example.com/a".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    )];
    let batch = BatchReport::from_reports(reports, vec![], 200).with_sample(SampleMetadata {
        source: "sitemap".to_string(),
        total_discovered: 487,
        audited: 20,
        sample_limit: Some(20),
        selection: "first_n".to_string(),
        is_sample: true,
    });

    let json: serde_json::Value =
        serde_json::from_str(&UnifiedReport::batch(&batch).to_json(false).unwrap()).unwrap();
    let sample = &json["sample"];
    assert_eq!(sample["source"], "sitemap");
    assert_eq!(sample["total_discovered"], 487);
    assert_eq!(sample["audited"], 20);
    assert_eq!(sample["sample_limit"], 20);
    assert_eq!(sample["selection"], "first_n");
    assert_eq!(sample["is_sample"], true);
}

#[test]
fn test_worst_risk_all_low() {
    use crate::audit::compute_worst_risk;
    use crate::audit::normalized::RiskLevel;
    use crate::wcag::WcagResults;
    // No critical/high/medium pages — result must be Low
    let reports: Vec<crate::audit::normalized::NormalizedReport> = (0..3)
        .map(|_| {
            crate::audit::normalized::normalize(&AuditReport::new(
                "https://example.com".to_string(),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            ))
            .normalized
        })
        .collect();
    let result = compute_worst_risk(&reports);
    assert_eq!(
        result,
        RiskLevel::Low,
        "all-low batch must report Low risk, got {:?}",
        result
    );
}

#[test]
fn test_modules_under_page_detail() {
    use crate::output::module::active_modules;
    use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        500,
    )
    .with_performance(crate::audit::PerformanceResults {
        vitals: WebVitals::default(),
        score: PerformanceScore {
            overall: 80,
            grade: PerformanceGrade::Gold,
            lcp_score: Some(20),
            fcp_score: Some(20),
            cls_score: Some(20),
            interactivity_score: Some(20),
            si_score: Some(20),
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
        measurement_warnings: vec![],
    })
    .with_seo(crate::seo::SeoAnalysis::default())
    .with_security(crate::security::SecurityAnalysis {
        score: 90,
        grade: "A".to_string(),
        headers: Default::default(),
        ssl: Default::default(),
        issues: vec![],
        recommendations: vec![],
        protection: Default::default(),
    })
    .with_ux(crate::ux::analyze_ux(&crate::AXTree::new()))
    .with_journey(crate::journey::analyze_journey(&crate::AXTree::new()));

    let active_keys: Vec<&'static str> = active_modules(&report)
        .into_iter()
        .map(|(key, _)| key)
        .collect();

    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(true).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let modules = &json_value["pages"][0]["detail"]["modules"];
    for key in &active_keys {
        assert!(
            modules.get(key).is_some(),
            "Module '{}' is active but missing from pages[0].detail.modules",
            key
        );
    }
}

/// Builds an AuditReport with all modules registered in `active_modules()`.
fn all_active_modules_report() -> AuditReport {
    use crate::audit::PerformanceResults;
    use crate::dark_mode::DarkModeAnalysis;
    use crate::mobile::{
        ContentSizing, FontSizeAnalysis, MobileFriendliness, TouchTargetAnalysis, ViewportAnalysis,
    };
    use crate::performance::{PerformanceGrade, PerformanceScore, WebVitals};

    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        0,
    )
    .with_performance(PerformanceResults {
        vitals: WebVitals::default(),
        score: PerformanceScore {
            overall: 80,
            grade: PerformanceGrade::Gold,
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
        measurement_warnings: vec![],
    })
    .with_seo(crate::seo::SeoAnalysis::default())
    .with_security(crate::security::SecurityAnalysis {
        score: 80,
        grade: "B".into(),
        headers: Default::default(),
        ssl: Default::default(),
        issues: vec![],
        recommendations: vec![],
        protection: Default::default(),
    })
    .with_mobile(MobileFriendliness {
        score: 75,
        viewport: ViewportAnalysis::default(),
        touch_targets: TouchTargetAnalysis::default(),
        font_sizes: FontSizeAnalysis::default(),
        content_sizing: ContentSizing::default(),
        issues: vec![],
    })
    .with_ux(crate::ux::analyze_ux(&crate::AXTree::new()))
    .with_journey(crate::journey::analyze_journey(&crate::AXTree::new()))
    .with_dark_mode(DarkModeAnalysis {
        supported: false,
        class_based_dark_mode: false,
        score: 50,
        detection_methods: vec![],
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
    })
    .with_best_practices(crate::best_practices::BestPracticesAnalysis {
        console_errors: crate::best_practices::ConsoleErrorsAnalysis {
            errors: vec![],
            warnings: vec![],
            error_count: 0,
            warning_count: 0,
        },
        vulnerable_libraries: crate::best_practices::VulnerableLibrariesAnalysis {
            detected: vec![],
            vulnerable: vec![],
            has_vulnerabilities: false,
        },
        score: 100,
    })
    .with_tech_stack(crate::tech_stack::TechStackAnalysis {
        detected: vec![],
        findings: vec![],
        score: 100,
        grade: "A".into(),
    })
    .with_patterns(crate::patterns::PatternAnalysis {
        recognized: vec![],
        violations: vec![],
        journey_candidates: vec![],
    });
    let sq = crate::source_quality::analyze_source_quality(&report);
    let av = crate::ai_visibility::analyze_ai_visibility(&report);
    report.discoverability.source_quality = Some(sq);
    report.discoverability.ai_visibility = Some(av);
    // content_visibility is set separately per test — its JSON emission
    // is conditional on signal_count > 0.
    report
}

#[test]
fn test_json_all_active_modules_non_null() {
    use crate::output::module::active_modules;

    let report = all_active_modules_report();
    let active_keys: Vec<&'static str> = active_modules(&report)
        .into_iter()
        .map(|(key, _)| key)
        // content_visibility is intentionally skipped: the JSON emitter suppresses it
        // when signal_count == 0 (an empty fixture produces no signals). This is
        // expected behavior, not a bug — tested via the PDF ViewModel path instead.
        .filter(|k| *k != "content_visibility")
        .collect();

    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(true).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let modules = &json_value["pages"][0]["detail"]["modules"];

    for key in &active_keys {
        let value = modules.get(key);
        assert!(
            value.is_some(),
            "Module '{}' missing from pages[0].detail.modules",
            key
        );
        assert!(
            !value.unwrap().is_null(),
            "Module '{}' is null in pages[0].detail.modules",
            key
        );
    }
}

#[test]
fn test_search_experience_serialized_in_single_detail_modules() {
    let report = all_active_modules_report();
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(true).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let search_experience = &json_value["pages"][0]["detail"]["modules"]["search_experience"];

    assert!(
        search_experience.is_object(),
        "search_experience must be serialized in single report detail modules"
    );
    assert_eq!(search_experience["measurement_type"], "composite");
    assert!(search_experience["score"].as_u64().is_some());
    assert!(
        search_experience["components"]
            .as_array()
            .is_some_and(|components| !components.is_empty()),
        "search_experience must include its component inputs"
    );
}

#[test]
fn test_dual_viewport_summary_serialized_in_single_detail_modules() {
    let mut report = all_active_modules_report();
    report.dual_viewport = Some(crate::audit::DualViewportResults {
        desktop: crate::audit::ViewportAuditData {
            wcag_results: WcagResults::new(),
            accessibility_score: 92.0,
            performance: None,
            seo: None,
            mobile: None,
            ux: None,
            journey: None,
            screenshot: None,
            module_runs: vec![],
        },
        mobile: crate::audit::ViewportAuditData {
            wcag_results: WcagResults::new(),
            accessibility_score: 71.0,
            performance: None,
            seo: None,
            mobile: None,
            ux: None,
            journey: None,
            screenshot: None,
            module_runs: vec![],
        },
    });

    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(true).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let dual = &json_value["pages"][0]["detail"]["modules"]["dual_viewport"];

    assert!(dual.is_object(), "dual_viewport summary must be serialized");
    assert_eq!(dual["desktop"]["accessibility_score"], 92);
    assert_eq!(dual["mobile"]["accessibility_score"], 71);
    assert_eq!(dual["desktop"]["wcag"]["violations"], 0);
}

#[test]
fn test_score_breakdown_present_for_viewport_weighted() {
    use crate::audit::{ViewportScoreSet, ViewportScores};

    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    report.viewport_scores = Some(ViewportScores {
        desktop: ViewportScoreSet {
            accessibility: 100,
            performance: None,
            overall: 100,
        },
        mobile: ViewportScoreSet {
            accessibility: 100,
            performance: None,
            overall: 100,
        },
        weighted_overall: 100,
    });
    let normalized = normalize(&report);
    assert_eq!(
        normalized.normalized.score_calculation_method,
        "viewport_weighted"
    );
    assert!(
        normalized.normalized.score_breakdown.is_some(),
        "NormalizedReport must have score_breakdown for viewport_weighted"
    );
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(true).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(
        json_value["pages"][0].get("score_breakdown").is_some()
            && !json_value["pages"][0]["score_breakdown"].is_null(),
        "score_breakdown must be present and non-null for viewport_weighted pages"
    );
    assert_eq!(
        json_value["pages"][0]["score_breakdown"]["viewport_blended_accessibility"],
        json_value["pages"][0]["accessibility_score"]
    );
}

#[test]
fn test_batch_page_detail_omitted_when_none() {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    let normalized = normalize(&report);
    let page = super::build_page(&normalized.normalized, None, None);
    let json_str = serde_json::to_string(&page).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(
        !json_value.as_object().unwrap().contains_key("detail"),
        "batch page must not emit \"detail\" key when detail is None, got: {}",
        json_str
    );
}

#[test]
fn test_en301549_annex_present_on_single_detail() {
    use crate::taxonomy::Severity;
    use crate::wcag::Violation;

    let mut results = WcagResults::new();
    results.add_violation(Violation::new(
        "1.1.1",
        "Non-text Content",
        WcagLevel::A,
        Severity::High,
        "Missing alt",
        "n1",
    ));
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        results,
        500,
    );
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let page = first_page(&unified);
    let detail = page.detail.as_ref().expect("single report has detail");
    let annex = &detail.en301549_annex;

    assert_eq!(annex.clauses.len(), 50);
    assert_eq!(
        annex.standard_version,
        crate::wcag::en301549::EN301549_VERSION
    );
    assert_eq!(
        annex.mapping_version,
        crate::wcag::en301549::EN301549_MAPPING_VERSION
    );
    assert!(!annex.disclaimer.is_empty());

    let clause_111 = annex
        .clauses
        .iter()
        .find(|c| c.wcag == "1.1.1")
        .expect("1.1.1 present");
    assert!(matches!(
        clause_111.status,
        crate::output::json::En301549ClauseStatusKind::ViolationsFound
    ));
    assert_eq!(clause_111.findings.len(), 1);

    // Summary-level batch roll-up must stay empty on single reports — the
    // full per-page annex lives under detail instead (see doc comment on
    // `UnifiedSummary::en301549_rollup`).
    assert!(unified.summary.en301549_rollup.is_empty());
}

#[test]
fn test_en301549_batch_rollup_present_and_page_detail_carries_full_annex() {
    let reports: Vec<AuditReport> = (0..3)
        .map(|i| {
            AuditReport::new(
                format!("https://example.com/{i}"),
                WcagLevel::AA,
                WcagResults::new(),
                100,
            )
        })
        .collect();
    let batch = crate::audit::BatchReport::from_reports(reports, vec![], 0);
    let unified = UnifiedReport::batch(&batch);

    assert_eq!(unified.summary.en301549_rollup.len(), 50);
    for rollup in &unified.summary.en301549_rollup {
        assert_eq!(rollup.affected_pages, 0, "no violations were seeded");
    }

    for page in &unified.pages {
        let detail = page.detail.as_ref().expect("batch page detail present");
        assert_eq!(detail.en301549_annex.clauses.len(), 50);
    }
}

/// English-locale guard (#406): the EN 301 549 annex (disclaimer + titles)
/// must contain no German umlauts/ß anywhere in the single-report JSON.
#[test]
fn test_en301549_annex_en_guard_no_umlauts() {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        500,
    );
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let page = first_page(&unified);
    let annex_json = serde_json::to_string(&page.detail.as_ref().unwrap().en301549_annex).unwrap();
    let has_umlaut = |s: &str| s.chars().any(|c| "äöüÄÖÜß".contains(c));
    assert!(
        !has_umlaut(&annex_json),
        "en301549_annex must be canonical English, found umlaut/ß in: {annex_json}"
    );
}

#[test]
fn test_collection_errors_absent_when_empty() {
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    let normalized = normalize(&report);
    let unified = UnifiedReport::single(&normalized, &report);
    let json_str = unified.to_json(false).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        !json_value
            .as_object()
            .unwrap()
            .contains_key("collection_errors"),
        "collection_errors must be absent from JSON when there are no errors"
    );
    let detail = &json_value["pages"][0]["detail"];
    assert!(
        !detail
            .as_object()
            .unwrap()
            .contains_key("collection_errors"),
        "detail.collection_errors must be absent from JSON when there are no errors"
    );
}

#[test]
fn test_collection_errors_serialized_when_present() {
    let mut unified = UnifiedReport {
        schema_version: "2.0",
        report_type: "batch",
        tool_version: env!("CARGO_PKG_VERSION"),
        metadata: ReportMetadata {
            tool: "test".to_string(),
            timestamp: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
            wcag_level: "AA".to_string(),
            execution_time_ms: 0,
        },
        audit_scope: None,
        execution_environment: None,
        audit_quality: Default::default(),
        metric_context: metric_context("batch"),
        artifacts: Vec::new(),
        summary: UnifiedSummary {
            url_count: 0,
            accessibility_score: 0,
            overall_score: 0,
            score: 0,
            grade: "F".to_string(),
            certificate: "None".to_string(),
            risk_level: crate::audit::normalized::RiskLevel::Low,
            violation_count: 0,
            finding_count: 0,
            finding_occurrence_count: 0,
            severity_counts: crate::audit::normalized::SeverityCounts {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                total: 0,
            },
            severity_counts_scope: "wcag_only".to_string(),
            occurrence_counts: crate::audit::normalized::SeverityCounts::default(),
            occurrence_counts_scope: "wcag_only".to_string(),
            passed_url_count: 0,
            failed_url_count: 0,
            violated_rule_count: 0,
            top_recurring_rules: vec![],
            template_clusters: vec![],
            en301549_rollup: vec![],
            performance_score: None,
            seo_score: None,
            security_score: None,
            mobile_score: None,
            ux_score: None,
            journey_score: None,
            performance_throttled_avg_score: None,
            lh_mobile_score: None,
            wcag_coverage: build_wcag_coverage_for_level("AA"),
            accessibility_score_breakdown: vec![],
            management_risks: vec![],
            top_actions: vec![],
            duplicate_content: vec![],
            canonical_issues: vec![],
            hreflang_issues: vec![],
            sitemap_http_issues: vec![],
            orphan_sitemap_urls: vec![],
            linked_not_in_sitemap: vec![],
            commerce: None,
        },
        sample: None,
        pages: vec![],
        url_matrix: vec![],
        internal_comparison: None,
        crawl_diagnostics: None,
        sitemap_diagnostics: None,
        site_analysis: None,
        errors: vec![],
        collection_errors: vec![ReportError {
            module: "crawl_diagnostics",
            error_type: "serialization_failed",
            reason: "NaN value in field".to_string(),
        }],
        verdict: Verdict::Pass,
        verdict_reasons: Vec::new(),
    };
    let json_str = unified.to_json(false).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let errs = &json_value["collection_errors"];
    assert!(errs.is_array(), "collection_errors must be an array");
    assert_eq!(errs.as_array().unwrap().len(), 1);
    assert_eq!(errs[0]["module"], "crawl_diagnostics");
    assert_eq!(errs[0]["error_type"], "serialization_failed");
    assert!(errs[0]["reason"].as_str().unwrap().contains("NaN"));

    // Verify detail-level collection_errors work the same way
    let report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        100,
    );
    let normalized = normalize(&report);
    let mut page_unified = UnifiedReport::single(&normalized, &report);
    if let Some(detail) = page_unified.pages[0].detail.as_mut() {
        detail.collection_errors.push(ReportError {
            module: "tech_stack",
            error_type: "serialization_failed",
            reason: "custom serializer error".to_string(),
        });
    }
    let json_str2 = page_unified.to_json(false).unwrap();
    let json_value2: serde_json::Value = serde_json::from_str(&json_str2).unwrap();
    let detail_errs = &json_value2["pages"][0]["detail"]["collection_errors"];
    assert!(detail_errs.is_array());
    assert_eq!(detail_errs[0]["module"], "tech_stack");
    // suppress unused warning from the first mut binding
    let _ = &mut unified;
}
