//! Diff-based accuracy test: detection corpus against a real audit run (#555).
//!
//! For each case in the ground-truth corpus (#553), serves its fixture HTML
//! locally, runs the real audit pipeline (`audit_page`, the same entry
//! point production uses), and diffs the actual `(rule_id, selector)`
//! outcome against `<case>.expected.json`. False negatives (an expected
//! violation that wasn't found — the dangerous direction, since it means a
//! real accessibility problem would go unreported) and false positives (a
//! violation found where a clean pass was expected) are reported
//! separately, since they're not equally severe.
//!
//! The comparison is strict (#783, `tests/common/corpus_diff.rs`): besides
//! missing findings it fails on undeclared extra findings, a review escalated
//! to a violation, a pass or not-applicable verdict without a completed run,
//! run records that contradict the findings, and violations that lost their
//! location.
//!
//! `#[ignore]`-gated like the existing Chrome-dependent tests in
//! `tests/integration_test.rs` — run with `cargo test -- --ignored`. If the
//! corpus directory is ever empty, the test fails: no fixtures are no evidence.

mod common;

use auditmysite::cli::{Args, WcagLevel};
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;
use common::corpus_diff::{diff_case, observe_wcag, render_report, Mismatch, MismatchKind};
use common::detection_corpus::{detection_corpus_dir, load_corpus_dir};
use common::fixture_server::serve_html;
use common::rule_contracts::wcag_diff_rules;

async fn ci_browser() -> BrowserManager {
    let opts = BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    };
    BrowserManager::with_options(opts)
        .await
        .expect("Browser launch failed")
}

#[tokio::test]
#[ignore = "needs real Chrome; run manually with `cargo test -- --ignored`"]
async fn detection_corpus_matches_real_audit_run() {
    let corpus_dir = detection_corpus_dir();
    let cases = load_corpus_dir(&corpus_dir);
    assert!(
        !cases.is_empty(),
        "empty detection corpus cannot verify an audit"
    );
    // `CORPUS_CASES=a,b` runs a subset while authoring; CI runs everything.
    let only: Option<Vec<String>> = std::env::var("CORPUS_CASES")
        .ok()
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect());
    let cases: Vec<_> = cases
        .into_iter()
        .filter(|c| only.as_ref().is_none_or(|o| o.contains(&c.case)))
        .collect();
    assert!(!cases.is_empty(), "CORPUS_CASES selects no case");

    let manager = ci_browser().await;
    let diff_rules = wcag_diff_rules();
    let mut diffs = Vec::new();

    for case in &cases {
        let html_path = corpus_dir.join(format!("{}.html", case.case));
        let html = std::fs::read_to_string(&html_path).unwrap_or_else(|e| {
            panic!(
                "cannot read fixture HTML {} for case '{}': {e}",
                html_path.display(),
                case.case
            )
        });

        let fixture = serve_html(html);
        let page = manager.new_page().await.expect("New page failed");
        manager
            .navigate(&page, &fixture.url)
            .await
            .expect("Navigation failed");

        let args = Args::parse_from(["auditmysite", &fixture.url, "--level", "aaa"]);
        let mut config = PipelineConfig::from_args_and_config(&args, None);
        config.wcag_level = WcagLevel::AAA;

        // A hung case fails by name instead of blocking the whole run.
        let (report, _snapshot) = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            audit_page(&page, &fixture.url, &config, &manager),
        )
        .await
        .unwrap_or_else(|_| panic!("Audit timed out after 300 s for case '{}'", case.case))
        .unwrap_or_else(|e| panic!("Audit failed for case '{}': {e}", case.case));

        let normalized = auditmysite::audit::normalize(&report);
        let exported = auditmysite::output::UnifiedReport::single(&normalized, &report);
        if let Err(error) = exported.to_json(false) {
            let evidence =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/corpus-failures");
            std::fs::create_dir_all(&evidence).unwrap();
            std::fs::write(
                evidence.join(format!("{}.json", case.case)),
                serde_json::to_vec_pretty(&serde_json::json!({"raw":report, "export":exported}))
                    .unwrap(),
            )
            .unwrap();
            panic!(
                "Export invariants failed for case '{}': {error}; evidence: {}",
                case.case,
                evidence.display()
            );
        }

        fixture.stop();

        let observation = observe_wcag(&report.accessibility.wcag_results);
        // `CORPUS_DUMP=1` writes what the run reported, for diagnosing a
        // mismatch. Expectations are written from the rule's source, never
        // copied from this file.
        if std::env::var("CORPUS_DUMP").is_ok() {
            let dir =
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/corpus-observations");
            std::fs::create_dir_all(&dir).unwrap();
            let wcag = &report.accessibility.wcag_results;
            let messages: Vec<String> = wcag
                .violations
                .iter()
                .chain(&wcag.warnings)
                .chain(&wcag.not_testables)
                .map(|v| {
                    format!(
                        "{:?} {:?} {:?}: {}",
                        v.kind, v.rule_id, v.selector, v.message
                    )
                })
                .collect();
            std::fs::write(
                dir.join(format!("{}.txt", case.case)),
                format!("{observation:#?}\n\n{}", messages.join("\n")),
            )
            .unwrap();
        }
        let mut mismatches = diff_case(case, &observation, &diff_rules);

        let _ = page.close().await;

        if let Some(expected) = &case.screen_reader_bfsg_verdict {
            let found = report
                .screen_reader_audit
                .as_ref()
                .map(|sr| serde_json::to_value(sr.bfsg_compliance.verdict).unwrap_or_default());
            if found.as_ref().and_then(|v| v.as_str()) != Some(expected.as_str()) {
                mismatches.push(Mismatch {
                    kind: MismatchKind::UnexpectedFinding,
                    detail: format!(
                        "screen_reader.bfsg_compliance.verdict: expected {expected}, found {found:?}"
                    ),
                });
            }
        }

        if !mismatches.is_empty() {
            diffs.push((case.case.clone(), mismatches));
        }
    }

    if !diffs.is_empty() {
        panic!(
            "{}",
            render_report("Detection corpus accuracy mismatches", &diffs)
        );
    }
}
