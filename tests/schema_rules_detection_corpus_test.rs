//! Diff-based accuracy test: JSON-LD schema-rules detection corpus against a
//! real audit run (plan/11, rest-scope of #558; strict comparison #783).
//!
//! Needs real Chrome/CDP. `#[ignore]`-gated like
//! `tests/detection_corpus_test.rs`; run with `cargo test -- --ignored`.
//!
//! The case runs through `audit_page` with the SEO module on — the same
//! path a real audit takes. That matters for `merchant_listing`: the
//! collection step (`detect_structured_data`) cannot know whether a product
//! is purchasable and leaves the feature `NotEvaluated`; only the SEO
//! module's derive step, with the page intent from the journey module and the
//! URL route, re-assesses it as a merchant product detail page. A case may
//! therefore name the path it is served under (`"fixture": {"path": ...}`).
//!
//! How an assessment counts:
//! - `MissingRequiredProperties` → violation;
//! - `NotEvaluated` (context not established) → needs_review;
//! - required properties met (or none defined) but Google-recommended
//!   properties missing → needs_review. Google's Rich Results Test reports
//!   missing recommended properties as non-critical issues: the result stays
//!   eligible, but a human should decide whether the data exists;
//! - everything present → no finding.
//!
//! Manual-review notes attached to an assessment (event is public, recipe
//! matches the visible recipe, ...) are fixed per type and are not counted.
//! A SEO module that did not produce data is "not run" for every feature.

mod common;

use auditmysite::cli::Args;
use auditmysite::seo::schema_rules::{SchemaRequirementStatus, SchemaRuleAssessment};
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;
use common::corpus_diff::{
    diff_case, render_report, DiffRules, FindingClass, Mismatch, MismatchKind, Observation,
    ObservedFinding, ObservedRun, RunState,
};
use common::detection_corpus::{
    load_corpus_dir, load_structurally_deferred, nonwcag_detection_corpus_dir, Verdict,
};
use common::routing_server::{serve_routes, Route, Transport};
use serde::Deserialize;

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

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSpec {
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CaseFile {
    #[serde(default)]
    fixture: FixtureSpec,
}

fn feature_id(a: &SchemaRuleAssessment) -> String {
    a.feature.key().to_string()
}

fn finding_class(a: &SchemaRuleAssessment) -> Option<FindingClass> {
    match a.requirement_status {
        SchemaRequirementStatus::MissingRequiredProperties => Some(FindingClass::Fail),
        SchemaRequirementStatus::NotEvaluated => Some(FindingClass::Review),
        SchemaRequirementStatus::MeetsRequiredProperties
        | SchemaRequirementStatus::RecommendationsOnly => {
            (!a.missing_recommended.is_empty()).then_some(FindingClass::Review)
        }
    }
}

#[tokio::test]
#[ignore = "needs real Chrome; run manually with `cargo test -- --ignored`"]
async fn schema_rules_detection_corpus_matches_real_detector_run() {
    let corpus_dir = nonwcag_detection_corpus_dir("schema_rules");
    let cases = load_corpus_dir(&corpus_dir);
    assert!(
        load_structurally_deferred(&corpus_dir).is_empty(),
        "deferred entries are not accepted (#783)"
    );
    assert!(
        !cases.is_empty(),
        "schema_rules detection corpus is empty — nothing to verify"
    );
    let only: Option<Vec<String>> = std::env::var("CORPUS_CASES")
        .ok()
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect());

    let manager = ci_browser().await;
    let all_ids = common::nonwcag_rule_inventory::canonical_schema_feature_ids();
    // Assessments concern a JSON-LD node, never a located element.
    let diff_rules = DiffRules {
        page_level: all_ids.clone(),
        run_counts_findings: false,
        ..Default::default()
    };
    let mut diffs = Vec::new();

    for case in &cases {
        if only.as_ref().is_some_and(|o| !o.contains(&case.case)) {
            continue;
        }
        let source = case.source_file.as_deref().unwrap();
        let spec = serde_json::from_str::<CaseFile>(&std::fs::read_to_string(source).unwrap())
            .unwrap_or_else(|e| panic!("invalid `fixture` in {}: {e}", source.display()))
            .fixture;
        let html_path = corpus_dir.join(format!("{}.html", case.case));
        let html = std::fs::read_to_string(&html_path).unwrap_or_else(|e| {
            panic!(
                "cannot read fixture HTML {} for case '{}': {e}",
                html_path.display(),
                case.case
            )
        });

        let path = spec.path.unwrap_or_else(|| "/".to_string());
        let fixture = serve_routes(vec![Route::new(&path, 200, html)], Transport::Http);
        let url = format!("{}{path}", fixture.url);
        let page = manager.new_page().await.expect("new page failed");
        manager
            .navigate(&page, &url)
            .await
            .expect("navigation failed");

        let args = Args::parse_from(["auditmysite", &url, "--seo"]);
        let config = PipelineConfig::from_args_and_config(&args, None);
        let audited = audit_page(&page, &url, &config, &manager).await;
        fixture.stop();
        assert!(
            fixture.transport_errors().is_empty(),
            "case '{}': fixture server transport errors: {:?}",
            case.case,
            fixture.transport_errors()
        );

        let structured_data = match &audited {
            Ok((report, _)) => report
                .discoverability
                .seo
                .as_ref()
                .map(|seo| &seo.structured_data),
            Err(_) => None,
        };
        let Some(structured_data) = structured_data else {
            let reason = match &audited {
                Err(e) => format!("audit failed: {e}"),
                Ok(_) => "SEO module produced no data".to_string(),
            };
            let obs = Observation {
                findings: Vec::new(),
                runs: all_ids
                    .iter()
                    .map(|id| ObservedRun {
                        rule_id: id.clone(),
                        state: RunState::NotRun(reason.clone()),
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
            // A pass needs an assessment of the feature that holds: no node
            // of the feature at all is "not applicable", not "pass".
            if exp.verdict == Verdict::Pass
                && !structured_data
                    .rule_assessments
                    .iter()
                    .any(|a| feature_id(a) == exp.rule_id && finding_class(a).is_none())
            {
                mismatches.push(Mismatch {
                    kind: MismatchKind::PassWithoutRun,
                    detail: format!(
                        "{}: expected a complete assessment, found none",
                        exp.rule_id
                    ),
                });
            }
        }
        // Extraction succeeded, so every feature was evaluated against every
        // node; the assessments are the complete outcome.
        let obs = Observation {
            findings: structured_data
                .rule_assessments
                .iter()
                .filter_map(|a| {
                    Some(ObservedFinding {
                        rule_id: feature_id(a),
                        selector: None,
                        class: finding_class(a)?,
                        located: false,
                    })
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
            render_report("Schema-rules detection corpus accuracy mismatches", &diffs)
        );
    }
}
