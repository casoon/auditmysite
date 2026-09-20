//! Diff-based accuracy test: JSON-LD schema-rules detection corpus against a
//! real `detect_structured_data` run (plan/11, rest-scope of #558).
//!
//! Needs real Chrome/CDP for the extraction step — `detect_structured_data`
//! reads `<script type="application/ld+json">` text content from a rendered
//! page via `page.evaluate()`. `#[ignore]`-gated like
//! `tests/detection_corpus_test.rs`; run with `cargo test -- --ignored`.
//!
//! `detect_structured_data` always evaluates rule assessments with
//! `ProductRuleContext::Indeterminate` (the real page-intent-aware context
//! is only established later in the pipeline by `seo::module`'s `derive`
//! step, via `schema_fit`+`refresh_rule_assessments`) — so `merchant_listing`
//! can only ever show up here as `NotEvaluated`/`needs_review`, never as a
//! confirmed violation or pass. That's the real, correct behavior of this
//! entry point, not a limitation of the test.

mod common;

use auditmysite::seo::schema::detect_structured_data;
use auditmysite::seo::schema_rules::SchemaRequirementStatus;
use auditmysite::{BrowserManager, BrowserOptions};
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

struct CaseDiff {
    case: String,
    false_negatives: Vec<String>,
    false_positives: Vec<String>,
}

#[tokio::test]
#[ignore = "needs real Chrome; run manually with `cargo test -- --ignored`"]
async fn schema_rules_detection_corpus_matches_real_detector_run() {
    let corpus_dir = nonwcag_detection_corpus_dir("schema_rules");
    let cases = load_corpus_dir(&corpus_dir);
    let deferred = load_structurally_deferred(&corpus_dir);
    let deferred_ids: std::collections::BTreeSet<String> =
        deferred.iter().map(|d| d.rule_id.clone()).collect();
    assert!(
        !cases.is_empty(),
        "schema_rules detection corpus is empty — nothing to verify"
    );

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
        let page = manager.new_page().await.expect("new page failed");
        manager
            .navigate(&page, &fixture.url)
            .await
            .expect("navigation failed");

        let structured_data = detect_structured_data(&page).await.unwrap_or_else(|e| {
            panic!(
                "detect_structured_data failed for case '{}': {e}",
                case.case
            )
        });

        fixture.stop();

        let mut false_negatives = Vec::new();
        let mut false_positives = Vec::new();

        for exp in &case.expectations {
            assert!(
                !deferred_ids.contains(&exp.rule_id),
                "case '{}' asserts on '{}', which is also marked structurally_deferred — pick one",
                case.case,
                exp.rule_id
            );
            let feature_assessments: Vec<_> = structured_data
                .rule_assessments
                .iter()
                .filter(|a| {
                    serde_json::to_value(a.feature)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_string))
                        .as_deref()
                        == Some(exp.rule_id.as_str())
                })
                .collect();

            if feature_assessments.is_empty() {
                false_negatives.push(format!(
                    "{}: expected a rule_assessments entry, found none",
                    exp.rule_id
                ));
                continue;
            }

            // A feature can appear on more than one node (not exercised by
            // this corpus's fixtures, one node per feature) — matching any
            // instance is sufficient.
            let has_status = |status: SchemaRequirementStatus| {
                feature_assessments
                    .iter()
                    .any(|a| a.requirement_status == status)
            };
            // RecommendationsOnly (no required properties defined for this
            // feature/schema-type combination at all, e.g. Article/
            // Organization/WebPage/WebSite/Person) is treated as Pass —
            // there is no missing-required state to violate.
            let passes = has_status(SchemaRequirementStatus::MeetsRequiredProperties)
                || has_status(SchemaRequirementStatus::RecommendationsOnly);
            let violates = has_status(SchemaRequirementStatus::MissingRequiredProperties);
            let needs_review = has_status(SchemaRequirementStatus::NotEvaluated);

            match exp.verdict {
                Verdict::Violation => {
                    if !violates {
                        false_negatives
                            .push(format!("{}: expected violation, found none", exp.rule_id));
                    }
                }
                Verdict::Pass => {
                    if violates {
                        false_positives
                            .push(format!("{}: expected pass, found violation", exp.rule_id));
                    } else if !passes {
                        false_negatives.push(format!(
                            "{}: expected pass, found neither pass nor violation ({:?})",
                            exp.rule_id,
                            feature_assessments
                                .iter()
                                .map(|a| a.requirement_status)
                                .collect::<Vec<_>>()
                        ));
                    }
                }
                Verdict::NeedsReview => {
                    if !needs_review {
                        false_negatives.push(format!(
                            "{}: expected needs_review, found neither ({:?})",
                            exp.rule_id,
                            feature_assessments
                                .iter()
                                .map(|a| a.requirement_status)
                                .collect::<Vec<_>>()
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
        let mut msg = String::from("Schema-rules detection corpus accuracy mismatches:\n");
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
