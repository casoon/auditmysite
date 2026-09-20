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
//! `#[ignore]`-gated like the existing Chrome-dependent tests in
//! `tests/integration_test.rs` — run with `cargo test -- --ignored`. If the
//! corpus directory is ever empty (e.g. a fresh checkout missing fixtures),
//! this test is a no-op rather than a failure.

mod common;

use auditmysite::cli::{Args, WcagLevel};
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;
use common::detection_corpus::{detection_corpus_dir, load_corpus_dir, Verdict};
use common::fixture_server::serve_html;

async fn ci_browser() -> BrowserManager {
    let opts = BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    };
    BrowserManager::with_options(opts)
        .await
        .expect("Browser launch failed")
}

struct CaseDiff {
    case: String,
    false_negatives: Vec<String>,
    false_positives: Vec<String>,
}

#[tokio::test]
#[ignore = "needs real Chrome; run manually with `cargo test -- --ignored`"]
async fn detection_corpus_matches_real_audit_run() {
    let corpus_dir = detection_corpus_dir();
    let cases = load_corpus_dir(&corpus_dir);
    if cases.is_empty() {
        // Nothing to check yet — the corpus starts empty and is populated
        // incrementally by #556. Not a failure.
        return;
    }

    let manager = ci_browser().await;
    let mut diffs: Vec<CaseDiff> = Vec::new();

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

        let (report, _snapshot) = audit_page(&page, &fixture.url, &config, &manager)
            .await
            .unwrap_or_else(|e| panic!("Audit failed for case '{}': {e}", case.case));

        fixture.stop();

        let wcag = &report.accessibility.wcag_results;
        let mut false_negatives = Vec::new();
        let mut false_positives = Vec::new();

        for exp in &case.expectations {
            let matches_violations = wcag.violations.iter().any(|v| finding_matches(v, exp));
            let matches_warnings = wcag.warnings.iter().any(|v| finding_matches(v, exp));
            // Outcome::Untested findings (always-manual-review checks like
            // media-alt/help's page-wide notice) route to their own bucket, not
            // warnings — treated the same as "needs review" here since both mean
            // "no automated verdict, a human must look."
            let matches_not_testable = wcag.not_testables.iter().any(|v| finding_matches(v, exp));

            match exp.verdict {
                Verdict::Violation => {
                    if !matches_violations {
                        false_negatives
                            .push(expectation_label(exp, "expected violation, found none"));
                    }
                }
                Verdict::Pass => {
                    if matches_violations {
                        false_positives
                            .push(expectation_label(exp, "expected pass, found violation"));
                    }
                }
                Verdict::NeedsReview => {
                    if !matches_violations && !matches_warnings && !matches_not_testable {
                        false_negatives.push(expectation_label(
                            exp,
                            "expected needs_review, found neither",
                        ));
                    }
                }
            }
        }

        if !false_negatives.is_empty() || !false_positives.is_empty() {
            diffs.push(CaseDiff {
                case: case.case.clone(),
                false_negatives,
                false_positives,
            });
        }
    }

    if !diffs.is_empty() {
        let mut msg = String::from("Detection corpus accuracy mismatches:\n");
        for diff in &diffs {
            msg.push_str(&format!("  case '{}':\n", diff.case));
            for fneg in &diff.false_negatives {
                msg.push_str(&format!("    FALSE NEGATIVE: {fneg}\n"));
            }
            for fpos in &diff.false_positives {
                msg.push_str(&format!("    FALSE POSITIVE: {fpos}\n"));
            }
        }
        panic!("{msg}");
    }
}

fn finding_matches(
    v: &auditmysite::Violation,
    exp: &common::detection_corpus::Expectation,
) -> bool {
    v.rule_id.as_deref() == Some(exp.rule_id.as_str())
        && exp
            .selector
            .as_deref()
            .map(|s| v.selector.as_deref() == Some(s))
            .unwrap_or(true)
}

fn expectation_label(exp: &common::detection_corpus::Expectation, detail: &str) -> String {
    match &exp.selector {
        Some(selector) => format!("{} @ {selector}: {detail}", exp.rule_id),
        None => format!("{}: {detail}", exp.rule_id),
    }
}
