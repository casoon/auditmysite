//! W3C ACT test cases against auditmysite (#837).
//!
//! The browser run is `#[ignore]` (needs Chrome and, once, a download):
//!
//! ```text
//! cargo test --test act_testcases -- --ignored --nocapture
//! ACT_RULES=full cargo test --test act_testcases -- --ignored --nocapture
//! ACT_RULES=97a4e1,b5c3f8 cargo test --test act_testcases -- --ignored --nocapture
//! ```
//!
//! `ACT_RULES` selects `starter` (default: the `starter` rules of
//! `docs/act-mapping.json`), `full` (every `full` + `active` rule) or a comma
//! separated list of ACT ids (which must still be `full` + `active`).
//! Results go to `$ACT_OUT_DIR` (default `<target>/act/`) as `results.json`,
//! `earl.json` (EARL JSON-LD; never `earl:passed`, see `docs/ACT_COVERAGE.md`) and `report.md`. Deviations are findings about the tool: the run reports them
//! and fails only with `ACT_FAIL_ON_DEVIATION=1`. Expectations come from the
//! ACT example headings, never from a run. Details: `docs/ACT_COVERAGE.md`.
//!
//! The rest of this file is browser-free and runs in every `cargo test`.

mod common;

use auditmysite::cli::{Args, WcagLevel};
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;
use common::act_harness::*;
use common::corpus_diff::{
    observe_wcag, FindingClass, Observation, ObservedFinding, ObservedRun, RunState,
};
use serde_json::Value;
use std::path::PathBuf;

fn mapping() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/act-mapping.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("mapping")).expect("mapping json")
}

fn selected_rules(selection: &str) -> Vec<MappedRule> {
    let doc = mapping();
    let eligible = |r: &Value| r["coverage"] == "full" && r["status"] == "active";
    let wanted: Vec<String> = selection.split(',').map(|s| s.trim().to_string()).collect();
    let mut rules = Vec::new();
    for r in doc["rules"].as_array().expect("rules") {
        let id = r["act_id"].as_str().expect("id");
        let pick = match selection {
            "starter" => r["starter"] == true,
            "full" => eligible(r),
            _ => wanted.iter().any(|w| w == id),
        };
        if !pick {
            continue;
        }
        assert!(
            eligible(r),
            "{id} is not coverage full + status active; no ACT verdict is derived for it"
        );
        rules.push(MappedRule {
            act_id: id.to_string(),
            name: r["name"].as_str().unwrap_or("").to_string(),
            internal_rules: r["internal_rules"]
                .as_array()
                .expect("internal_rules")
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect(),
        });
    }
    assert!(!rules.is_empty(), "ACT_RULES={selection} selects no rule");
    rules
}

#[test]
fn script_and_mapping_pin_the_same_commit() {
    let script = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/act-fetch.sh"),
    )
    .expect("script");
    let pinned = script
        .lines()
        .find_map(|l| l.strip_prefix("COMMIT=\""))
        .and_then(|l| l.strip_suffix('"'))
        .expect("COMMIT line");
    assert_eq!(mapping()["act_source"]["commit"], pinned);
}

#[test]
fn parser_reads_examples_like_the_act_generator() {
    let md = "---\nid: abc123\nname: X\n---\n\n## Applicability\n\n#### Passed Example 9\n```html\nignored\n```\n\n## Examples\n\n### Passed\n\n#### Passed Example 1\n\ntext\n\n```html\n<button>Ok</button>\n```\n\n### Failed\n\n#### Failed Example 1\n\n```html\n<html lang=\"en\">\n\t<body></body>\n</html>\n```\n\n### Inapplicable\n\n#### Inapplicable Example 1\n\n```svg\n<svg/>\n```\n";
    let cases = parse_rule_file(md, "_rules/x-abc123.md");
    let ids: Vec<_> = cases.iter().map(|c| c.id()).collect();
    assert_eq!(
        ids,
        [
            "abc123/passed-example-1",
            "abc123/failed-example-1",
            "abc123/inapplicable-example-1"
        ]
    );
    assert_eq!(cases[0].expected, Expected::Passed);
    assert_eq!(cases[0].body, "<!DOCTYPE html>\n<button>Ok</button>");
    assert_eq!(cases[1].expected, Expected::Failed);
    assert!(cases[1].body.contains("\n\t<body>"));
    assert_eq!(cases[2].body, "<svg/>");
    assert_eq!(cases[2].content_type(), "image/svg+xml");
    assert_eq!(
        cases[2].path(),
        "/testcases/abc123/inapplicable-example-1.svg"
    );
}

#[test]
#[should_panic(expected = "exactly one code block")]
fn parser_refuses_to_drop_a_testcase_silently() {
    parse_rule_file(
        "---\nid: abc123\n---\n\n## Examples\n\n#### Passed Example 1\n\nno code here\n",
        "_rules/x.md",
    );
}

#[test]
fn judge_compares_against_the_expectation_only() {
    assert_eq!(judge(Expected::Failed, Outcome::Failed), Verdict::Ok);
    assert_eq!(
        judge(Expected::Failed, Outcome::PassedOrInapplicable),
        Verdict::FalseNegative
    );
    assert_eq!(
        judge(Expected::Failed, Outcome::CantTell),
        Verdict::CantTell
    );
    assert_eq!(
        judge(Expected::Passed, Outcome::Failed),
        Verdict::FalsePositive
    );
    assert_eq!(
        judge(Expected::Inapplicable, Outcome::Failed),
        Verdict::FalsePositive
    );
    assert_eq!(
        judge(Expected::Passed, Outcome::PassedOrInapplicable),
        Verdict::Ok
    );
    assert_eq!(
        judge(Expected::Inapplicable, Outcome::PassedOrInapplicable),
        Verdict::Ok
    );
    assert_eq!(
        judge(Expected::Passed, Outcome::CantTell),
        Verdict::CantTell
    );
    assert_eq!(judge(Expected::Failed, Outcome::NotRun), Verdict::NotRun);
}

fn finding(rule: &str, class: FindingClass) -> ObservedFinding {
    ObservedFinding {
        rule_id: rule.into(),
        selector: None,
        class,
        located: false,
    }
}

fn run(rule: &str, state: RunState) -> ObservedRun {
    ObservedRun {
        rule_id: rule.into(),
        state,
    }
}

#[test]
fn outcome_is_derived_from_the_mapped_rules_only() {
    let mine = vec!["a/x".to_string(), "a/y".to_string()];
    let ran = RunState::Ran { findings: 0 };
    let na = RunState::NotRun("NotApplicable".into());
    let err = RunState::NotRun("Errored".into());
    let obs = |findings, runs| Observation { findings, runs };

    // A failure of an unmapped rule is not this ACT rule's business.
    let o = obs(
        vec![finding("b/z", FindingClass::Fail)],
        vec![run("a/x", ran.clone())],
    );
    assert_eq!(derive_outcome(&mine, &o), Outcome::PassedOrInapplicable);
    let o = obs(vec![finding("a/y", FindingClass::Fail)], vec![]);
    assert_eq!(derive_outcome(&mine, &o), Outcome::Failed);
    let o = obs(vec![finding("a/y", FindingClass::Review)], vec![]);
    assert_eq!(derive_outcome(&mine, &o), Outcome::CantTell);
    // All mapped rules say not applicable: inapplicable. One ran: not decided.
    let o = obs(vec![], vec![run("a/x", na.clone()), run("a/y", na.clone())]);
    assert_eq!(derive_outcome(&mine, &o), Outcome::Inapplicable);
    let o = obs(vec![], vec![run("a/x", na), run("a/y", ran)]);
    assert_eq!(derive_outcome(&mine, &o), Outcome::PassedOrInapplicable);
    // A rule that errored never counts as a pass.
    let o = obs(vec![], vec![run("a/x", err)]);
    assert_eq!(derive_outcome(&mine, &o), Outcome::NotRun);
}

#[test]
fn applicability_verdicts() {
    assert_eq!(
        judge(Expected::Inapplicable, Outcome::Inapplicable),
        Verdict::Ok
    );
    assert_eq!(
        judge(Expected::Passed, Outcome::Inapplicable),
        Verdict::WrongApplicability
    );
    assert_eq!(
        judge(Expected::Failed, Outcome::Inapplicable),
        Verdict::FalseNegative
    );
    assert_eq!(
        judge(Expected::Passed, Outcome::AuditError),
        Verdict::AuditError
    );
}

#[test]
fn consistency_levels() {
    use Verdict::*;
    assert_eq!(consistency(&[Ok, Ok]), "consistent");
    assert_eq!(consistency(&[Ok, FalseNegative]), "partially_consistent");
    assert_eq!(consistency(&[Ok, CantTell]), "partially_consistent");
    assert_eq!(consistency(&[FalseNegative, FalsePositive]), "inconsistent");
}

fn earl_for(outcome: &str) -> Value {
    let results = serde_json::json!({
        "act_source": { "repository": "https://example.org/r", "commit": "abc" },
        "rules": [{
            "act_id": "97a4e1", "internal_rules": ["buttons/name-missing"],
            "testcases": [{ "testcase_id": "97a4e1/failed-example-1", "outcome": outcome,
                            "source_file": "_rules/x-97a4e1.md" }],
        }],
    });
    let map = serde_json::json!({ "rules": [{ "act_id": "97a4e1", "wcag_sc": ["4.1.2"] }] });
    earl_json(&results, &map)
}

#[test]
fn earl_has_the_documented_structure() {
    let earl = earl_for("failed");
    assert_eq!(
        earl["@context"],
        "https://act-rules.github.io/earl-context.json"
    );
    let subject = &earl["@graph"][0];
    assert_eq!(subject["@type"], "TestSubject");
    assert_eq!(
        subject["source"],
        "https://example.org/r/blob/abc/_rules/x-97a4e1.md#failed-example-1"
    );
    let a = &subject["assertions"][0];
    assert_eq!(a["@type"], "Assertion");
    assert_eq!(a["result"]["outcome"], "earl:failed");
    assert_eq!(a["test"]["title"], "buttons/name-missing");
    assert_eq!(a["test"]["isPartOf"][0]["title"], "WCAG 2: 4.1.2");
}

#[test]
fn earl_never_claims_passed_or_inapplicable_from_an_ambiguous_run() {
    let earl = earl_for("passedOrInapplicable");
    assert!(earl["@graph"][0]["assertions"]
        .as_array()
        .unwrap()
        .is_empty());
    for (outcome, expected) in [
        ("cantTell", "earl:cantTell"),
        ("inapplicable", "earl:inapplicable"),
        ("notRun", "earl:untested"),
        ("auditError", "earl:untested"),
    ] {
        assert_eq!(
            earl_for(outcome)["@graph"][0]["assertions"][0]["result"]["outcome"],
            expected
        );
    }
}

#[test]
fn mapping_selection_defaults_to_the_starter_set() {
    let rules = selected_rules("starter");
    assert!(rules.iter().any(|r| r.act_id == "97a4e1"));
    assert!(rules.iter().all(|r| !r.internal_rules.is_empty()));
}

#[tokio::test]
#[ignore = "needs real Chrome and the pinned ACT cache; run with `cargo test --test act_testcases -- --ignored --nocapture`"]
async fn act_testcases_against_real_audit_run() {
    let selection = std::env::var("ACT_RULES").unwrap_or_else(|_| "starter".into());
    let rules = selected_rules(&selection);
    let cache = ensure_cache();
    let ids: Vec<String> = rules.iter().map(|r| r.act_id.clone()).collect();
    let cases = load_testcases(&cache, &ids);
    for r in &rules {
        assert!(
            cases.iter().any(|c| c.act_id == r.act_id),
            "no test cases found for ACT rule {}",
            r.act_id
        );
    }
    let server = ActServer::start(&cache, &cases);

    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch");

    let mut executed = Vec::new();
    for case in &cases {
        let rule = rules.iter().find(|r| r.act_id == case.act_id).unwrap();
        let url = format!("{}{}", server.base, case.path());
        let (page, context) = manager.new_isolated_page().await.expect("new page");
        manager.navigate(&page, &url).await.expect("navigation");
        let args = Args::parse_from(["auditmysite", url.as_str(), "--level", "aaa"]);
        let mut config = PipelineConfig::from_args_and_config(&args, None);
        config.wcag_level = WcagLevel::AAA;
        let started = std::time::Instant::now();
        let audited = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            audit_page(&page, &url, &config, &manager),
        )
        .await;
        let _ = page.close().await;
        manager.dispose_context(context).await;

        let (outcome, findings, error, runs) = match audited {
            Ok(Ok((report, _))) => {
                let obs = observe_wcag(&report.accessibility.wcag_results);
                let findings = obs
                    .findings
                    .iter()
                    .filter(|f| rule.internal_rules.contains(&f.rule_id))
                    .map(|f| {
                        let class = match f.class {
                            FindingClass::Fail => "fail",
                            FindingClass::Review => "review",
                        };
                        (f.rule_id.clone(), class, f.selector.clone())
                    })
                    .collect();
                let runs = obs
                    .runs
                    .iter()
                    .filter(|r| rule.internal_rules.contains(&r.rule_id))
                    .map(|r| match &r.state {
                        RunState::Ran { findings } => format!("{}: Ran({findings})", r.rule_id),
                        RunState::NotRun(why) => format!("{}: NotRun({why})", r.rule_id),
                    })
                    .collect();
                (
                    derive_outcome(&rule.internal_rules, &obs),
                    findings,
                    None,
                    runs,
                )
            }
            Ok(Err(e)) => (
                Outcome::AuditError,
                Vec::new(),
                Some(e.to_string()),
                Vec::new(),
            ),
            Err(_) => (
                Outcome::AuditError,
                Vec::new(),
                Some("audit timed out after 120 s".to_string()),
                Vec::new(),
            ),
        };
        let verdict = judge(case.expected, outcome);
        println!(
            "{:<34} {:>5.1}s expected {:<12} got {:<22} {}",
            case.id(),
            started.elapsed().as_secs_f32(),
            case.expected.as_str(),
            outcome.as_str(),
            verdict.as_str()
        );
        executed.push(Executed {
            verdict,
            outcome,
            findings,
            error,
            runs,
            case: case.clone(),
        });
    }

    let commit = mapping()["act_source"]["commit"]
        .as_str()
        .unwrap()
        .to_string();
    let results = results_json(&commit, &selection, "AAA", &rules, &executed);
    let out = std::env::var("ACT_OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(
                std::env::var("CARGO_TARGET_DIR")
                    .unwrap_or_else(|_| format!("{}/target", env!("CARGO_MANIFEST_DIR"))),
            )
            .join("act")
        });
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("results.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
    std::fs::write(
        out.join("earl.json"),
        serde_json::to_string_pretty(&earl_json(&results, &mapping())).unwrap(),
    )
    .unwrap();
    let markdown = render_markdown(&results);
    std::fs::write(out.join("report.md"), &markdown).unwrap();
    println!("{markdown}\nwritten to {}", out.display());

    assert_eq!(executed.len(), cases.len(), "every test case must be run");
    if std::env::var("ACT_FAIL_ON_DEVIATION").is_ok() {
        assert!(
            results["summary"]["ok"] == results["summary"]["testcases"],
            "ACT deviations, see {}",
            out.join("report.md").display()
        );
    }
}
