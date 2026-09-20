//! Diff-based accuracy test: security-header detection corpus against a real
//! `analyze_security` run (#558, mirrors #555's WCAG-corpus accuracy test).
//!
//! Unlike the WCAG corpus, this doesn't need Chrome/CDP at all —
//! `analyze_security(url)` is a standalone async fn that does its own HTTP
//! fetch (`reqwest`), so this test runs as a normal `cargo test` (no
//! `#[ignore]` gate, no browser dependency). It serves each case's
//! `response_headers` from a local plain-HTTP listener and diffs the real
//! `SecurityAnalysis.issues` output against `<case>.expected.json`.

mod common;

use auditmysite::analyze_security;
use common::detection_corpus::{
    load_corpus_dir, load_structurally_deferred, nonwcag_detection_corpus_dir, Verdict,
};
use common::fixture_server::serve;

struct CaseDiff {
    case: String,
    false_negatives: Vec<String>,
    false_positives: Vec<String>,
}

#[tokio::test]
async fn security_detection_corpus_matches_real_analyze_security_run() {
    let corpus_dir = nonwcag_detection_corpus_dir("security");
    let cases = load_corpus_dir(&corpus_dir);
    let deferred = load_structurally_deferred(&corpus_dir);
    let deferred_ids: std::collections::BTreeSet<String> =
        deferred.iter().map(|d| d.rule_id.clone()).collect();
    assert!(
        !cases.is_empty(),
        "security detection corpus is empty — nothing to verify (expected at least the seed cases)"
    );

    let mut diffs: Vec<CaseDiff> = Vec::new();

    for case in &cases {
        // Plain HTTP throughout: this corpus deliberately defers the one
        // HSTS check gated behind an actual HTTPS scheme (see
        // `structurally_deferred.json`).
        let fixture = serve(
            "<html><body>security corpus fixture</body></html>".to_string(),
            case.response_headers.clone(),
        );

        let analysis = analyze_security(&fixture.url)
            .await
            .unwrap_or_else(|e| panic!("analyze_security failed for case '{}': {e}", case.case));

        fixture.stop();

        // Before reading anything into the diff: did the fixture server
        // actually answer? `analyze_security` turns a failed fetch into empty
        // headers, so without this a transport problem arrives as a
        // detection-accuracy mismatch (plan 48).
        let transport_errors = fixture.transport_errors();
        assert!(
            fixture.served() > 0,
            "case '{}': the fixture server never delivered a response, so the analysis below \
             says nothing about detection accuracy. Transport errors: {:?}",
            case.case,
            transport_errors,
        );
        assert!(
            transport_errors.is_empty(),
            "case '{}': fixture server transport errors: {:?}",
            case.case,
            transport_errors,
        );

        let mut false_negatives = Vec::new();
        let mut false_positives = Vec::new();

        for exp in &case.expectations {
            assert!(
                !deferred_ids.contains(&exp.rule_id),
                "case '{}' asserts on '{}', which is also marked structurally_deferred — pick one",
                case.case,
                exp.rule_id
            );
            let found = analysis
                .issues
                .iter()
                .any(|issue| format!("{}:{}", issue.header, issue.issue_type) == exp.rule_id);

            match exp.verdict {
                Verdict::Violation => {
                    if !found {
                        false_negatives
                            .push(format!("{}: expected violation, found none", exp.rule_id));
                    }
                }
                Verdict::Pass => {
                    if found {
                        false_positives
                            .push(format!("{}: expected pass, found violation", exp.rule_id));
                    }
                }
                Verdict::NeedsReview => {
                    // Security-header issues are always a clean violation/pass
                    // binary in this module today (no manual-review bucket
                    // equivalent to WCAG's NotTestable) — not expected to be
                    // used by this corpus, but handled the same as
                    // "should be present" rather than silently ignored.
                    if !found {
                        false_negatives.push(format!(
                            "{}: expected needs_review, found nothing",
                            exp.rule_id
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
        let mut msg = String::from("Security detection corpus accuracy mismatches:\n");
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
