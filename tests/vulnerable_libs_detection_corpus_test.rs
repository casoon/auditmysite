//! Diff-based accuracy test: vulnerable-library detection corpus against a
//! real `analyze_vulnerable_libraries` run (plan/11, rest-scope of #558).
//!
//! Needs real Chrome/CDP — `analyze_vulnerable_libraries(page: &Page)` reads
//! `window.jQuery`/`window.angular`/etc. via `page.evaluate()` from a
//! rendered page, unlike `analyze_security`'s plain HTTP fetch. `#[ignore]`-
//! gated like `tests/detection_corpus_test.rs`; run with `cargo test --
//! --ignored`.

mod common;

use auditmysite::best_practices::analyze_vulnerable_libraries;
use auditmysite::{BrowserManager, BrowserOptions};
use common::corpus_diff::{
    diff_case, render_report, DiffRules, FindingClass, Mismatch, MismatchKind, Observation,
    ObservedFinding, ObservedRun, RunState,
};
use common::detection_corpus::{
    load_corpus_dir, load_structurally_deferred, nonwcag_detection_corpus_dir, Verdict,
};
use common::fixture_server::serve_html;

/// Same CI-sandbox handling as `tests/detection_corpus_test.rs`'s `ci_browser`.
async fn ci_browser() -> BrowserManager {
    let opts = BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    };
    BrowserManager::with_options(opts)
        .await
        .expect("browser launch failed")
}

#[tokio::test]
#[ignore = "needs real Chrome; run manually with `cargo test -- --ignored`"]
async fn vulnerable_libs_detection_corpus_matches_real_analyzer_run() {
    let corpus_dir = nonwcag_detection_corpus_dir("vulnerable_libs");
    let cases = load_corpus_dir(&corpus_dir);
    let deferred = load_structurally_deferred(&corpus_dir);
    let deferred_ids: std::collections::BTreeSet<String> =
        deferred.iter().map(|d| d.rule_id.clone()).collect();
    assert!(
        !cases.is_empty(),
        "vulnerable_libs detection corpus is empty — nothing to verify"
    );

    let manager = ci_browser().await;
    let all_ids = common::nonwcag_rule_inventory::canonical_vulnerable_library_ids();
    // A library finding concerns the page as a whole.
    let diff_rules = DiffRules {
        page_level: all_ids.clone(),
        run_counts_findings: false,
        ..Default::default()
    };
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
        let page = manager.new_page().await.expect("new page failed");
        manager
            .navigate(&page, &fixture.url)
            .await
            .expect("navigation failed");

        let collected = analyze_vulnerable_libraries(&page).await;
        fixture.stop();

        // A failed collection (the page broke the detection scripts) is no
        // measurement: every library check is "not run", never clean.
        let Some(analysis) = collected.value().cloned() else {
            let obs = Observation {
                findings: Vec::new(),
                runs: all_ids
                    .iter()
                    .map(|id| ObservedRun {
                        rule_id: id.clone(),
                        state: RunState::NotRun(format!("{:?}", collected.state())),
                    })
                    .collect(),
            };
            let mismatches = diff_case(case, &obs, &diff_rules);
            if !mismatches.is_empty() {
                diffs.push((case.case.clone(), mismatches));
            }
            continue;
        };

        let mut mismatches = Vec::new();
        for exp in &case.expectations {
            assert!(
                !deferred_ids.contains(&exp.rule_id),
                "case '{}' asserts on '{}', which is also marked structurally_deferred — pick one",
                case.case,
                exp.rule_id
            );
            // A pass only means something if the library was detected in
            // the first place: an undetected library trivially "passes" but
            // proves nothing about the version check.
            if exp.verdict == Verdict::Pass
                && !analysis.detected.iter().any(|lib| lib.name == exp.rule_id)
            {
                mismatches.push(Mismatch {
                    kind: MismatchKind::PassWithoutRun,
                    detail: format!(
                        "{}: expected pass (library detected, not vulnerable), but the library \
                         wasn't detected at all",
                        exp.rule_id
                    ),
                });
            }
        }
        // The analyzer returned a value, so the version check ran for every
        // library it knows; a vulnerable entry is the only finding it makes.
        let obs = Observation {
            findings: analysis
                .vulnerable
                .iter()
                .map(|lib| ObservedFinding {
                    rule_id: lib.name.clone(),
                    selector: None,
                    class: FindingClass::Fail,
                    located: false,
                })
                .collect(),
            runs: all_ids
                .iter()
                .map(|id| ObservedRun {
                    rule_id: id.clone(),
                    state: RunState::Ran { findings: 0 },
                })
                .collect(),
        };
        mismatches.extend(diff_case(case, &obs, &diff_rules));
        if !mismatches.is_empty() {
            diffs.push((case.case.clone(), mismatches));
        }
    }

    if !diffs.is_empty() {
        panic!(
            "{}",
            render_report(
                "Vulnerable-libs detection corpus accuracy mismatches",
                &diffs
            )
        );
    }
}
