//! Snapshot tests — safety net before refactoring large output modules.
//!
//! Captured fields are structurally stable (scores, counts, rule IDs, module names).
//! Non-deterministic fields (timestamps, dates) are intentionally excluded.
//!
//! To regenerate snapshots after an intentional change:
//!   INSTA_UPDATE=always cargo test --test snapshot_tests
//! Then review with:
//!   cargo insta review

use auditmysite::audit::{normalize, AuditReport, BatchReport, PerformanceResults};
use auditmysite::cli::WcagLevel;
use auditmysite::journey::analyze_journey;
use auditmysite::mobile::MobileFriendliness;
use auditmysite::output::builder::{build_batch_presentation, build_view_model};
use auditmysite::output::report_model::ReportConfig;
use auditmysite::performance::{PerformanceGrade, PerformanceScore, WebVitals};
use auditmysite::security::SecurityAnalysis;
use auditmysite::seo::SeoAnalysis;
use auditmysite::ux::analyze_ux;
use auditmysite::wcag::{Severity, Violation, WcagResults};
use auditmysite::AXTree;

// ─── Shared Fixtures ────────────────────────────────────────────────────────

fn make_violations() -> WcagResults {
    let mut results = WcagResults::new();
    results.passes = 50;

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
    }
}

fn make_full_report() -> AuditReport {
    let tree = AXTree::new();
    AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        make_violations(),
        1000,
    )
    .with_performance(make_performance())
    .with_seo(SeoAnalysis {
        score: 90,
        ..Default::default()
    })
    .with_security(SecurityAnalysis {
        score: 80,
        grade: "B".to_string(),
        headers: Default::default(),
        ssl: Default::default(),
        issues: vec![],
        recommendations: vec![],
        protection: Default::default(),
    })
    .with_mobile(MobileFriendliness {
        score: 85,
        viewport: Default::default(),
        touch_targets: Default::default(),
        font_sizes: Default::default(),
        content_sizing: Default::default(),
        issues: vec![],
    })
    .with_ux(analyze_ux(&tree))
    .with_journey(analyze_journey(&tree))
}

// ─── Snapshot A: NormalizedReport key fields ────────────────────────────────
//
// Captures scores, grade, certificate, severity counts, module names, and
// finding rule IDs — the structural backbone of the data pipeline.
// Timestamps are excluded (non-deterministic).

#[test]
fn snapshot_normalized_report_fields() {
    let report = make_full_report();
    let normalized = normalize(&report);

    let mut module_names: Vec<&str> = normalized
        .module_scores
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    module_names.sort_unstable();

    let mut finding_ids: Vec<&str> = normalized
        .findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect();
    finding_ids.sort_unstable();

    let value = serde_json::json!({
        "url": normalized.url,
        "overall_score": normalized.overall_score,
        "score": normalized.score,
        "grade": normalized.grade,
        "certificate": normalized.certificate,
        "severity_counts": {
            "critical": normalized.severity_counts.critical,
            "high": normalized.severity_counts.high,
            "medium": normalized.severity_counts.medium,
            "low": normalized.severity_counts.low,
            "total": normalized.severity_counts.total,
        },
        "module_count": normalized.module_scores.len(),
        "module_names": module_names,
        "finding_count": normalized.findings.len(),
        "finding_rule_ids": finding_ids,
    });

    insta::assert_json_snapshot!("normalized_report_fields", value);
}

// ─── Snapshot B: ReportViewModel key fields ─────────────────────────────────
//
// Captures the ViewModel structure produced by build_view_model().
// Validates that the builder maps NormalizedReport → ViewModel correctly
// and that no silent regressions occur during refactoring of single.rs.

#[test]
fn snapshot_view_model_fields() {
    let report = make_full_report();
    let normalized = normalize(&report);
    let config = ReportConfig::default();
    let vm = build_view_model(&normalized, &config);

    let mut module_names: Vec<&str> = vm
        .modules
        .dashboard
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    module_names.sort_unstable();

    let mut finding_ids: Vec<&str> = vm
        .findings
        .all_findings
        .iter()
        .map(|f| f.rule_id.as_str())
        .collect();
    finding_ids.sort_unstable();

    let total_action_items: usize = vm
        .actions
        .roadmap_columns
        .iter()
        .map(|col| col.items.len())
        .sum();

    let value = serde_json::json!({
        "cover": {
            "score": vm.cover.score,
            "grade": vm.cover.grade,
            "certificate": vm.cover.certificate,
            "total_issues": vm.cover.total_issues,
            "critical_issues": vm.cover.critical_issues,
        },
        "summary": {
            "score": vm.summary.score,
            "grade": vm.summary.grade,
            "certificate": vm.summary.certificate,
            "maturity_label": vm.summary.maturity_label,
        },
        "severity": {
            "critical": vm.severity.critical,
            "high": vm.severity.high,
            "medium": vm.severity.medium,
            "low": vm.severity.low,
            "total": vm.severity.total,
        },
        "module_count": vm.modules.dashboard.len(),
        "module_names": module_names,
        "finding_count": vm.findings.all_findings.len(),
        "finding_rule_ids": finding_ids,
        "action_column_count": vm.actions.roadmap_columns.len(),
        "total_action_items": total_action_items,
    });

    insta::assert_json_snapshot!("view_model_fields", value);
}

// ─── Snapshot C: BatchPresentation key fields ────────────────────────────────
//
// Captures the batch builder's structural output.
// Validates that build_batch_presentation() is stable during refactoring of
// pdf/mod.rs and future batch builder changes.

#[test]
fn snapshot_batch_presentation_fields() {
    let reports = vec![make_full_report(), make_full_report()];
    let batch = BatchReport::from_reports(reports, vec![], 200);
    let pres = build_batch_presentation(&batch);

    let mut module_averages: Vec<(&str, u32)> = pres
        .portfolio_summary
        .module_averages
        .iter()
        .map(|(name, score)| (name.as_str(), *score))
        .collect();
    module_averages.sort_unstable_by_key(|(name, _)| *name);

    let value = serde_json::json!({
        "total_urls": pres.portfolio_summary.total_urls,
        "average_score": pres.portfolio_summary.average_score,
        "grade": pres.portfolio_summary.grade,
        "certificate": pres.portfolio_summary.certificate,
        "module_averages": module_averages.iter().map(|(name, score)| {
            serde_json::json!({ "name": name, "score": score })
        }).collect::<Vec<_>>(),
        "top_issue_count": pres.top_issues.len(),
        "url_ranking_count": pres.url_ranking.len(),
    });

    insta::assert_json_snapshot!("batch_presentation_fields", value);
}
