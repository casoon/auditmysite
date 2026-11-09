//! SARIF output contract (#977): shape GitHub Code Scanning needs, stable
//! fingerprints, not-run kept out of the results, golden file with own
//! fixtures. The official SARIF 2.1.0 schema is checked by an opt-in test
//! (`sarif_matches_the_official_schema`): its OASIS licence does not allow
//! vendoring it here.

use std::path::PathBuf;

use auditmysite::audit::finding_contract::FindingOutcome;
use auditmysite::audit::{
    normalize_checked, AuditQuality, AuditQualityStatus, AuditReport, ExecutionStatus, ModuleRun,
};
use auditmysite::cli::WcagLevel;
use auditmysite::output::format_sarif;
use auditmysite::wcag::{NotRun, RuleRun, Severity, Violation, WcagResults};
use serde_json::Value;

fn violation(
    criterion: &str,
    name: &str,
    severity: Severity,
    message: &str,
    node: &str,
) -> Violation {
    Violation::new(criterion, name, WcagLevel::A, severity, message, node)
}

/// Page with two located findings of one rule, a document-wide one, a rule
/// that did not run and a module that failed.
fn fixture_report(url: &str, alt_message: &str) -> AuditReport {
    let mut results = WcagResults::new();
    results.violations.push(
        violation(
            "1.1.1",
            "Non-text Content",
            Severity::High,
            alt_message,
            "n1",
        )
        .with_selector("img.hero")
        .with_fix("Add an alt attribute to the image")
        .with_help_url("https://www.w3.org/WAI/WCAG22/Understanding/non-text-content"),
    );
    results.violations.push(
        violation(
            "1.1.1",
            "Non-text Content",
            Severity::High,
            alt_message,
            "n2",
        )
        .with_selector("img.logo")
        .with_help_url("https://www.w3.org/WAI/WCAG22/Understanding/non-text-content"),
    );
    results.violations.push(
        violation(
            "2.4.2",
            "Page Titled",
            Severity::Medium,
            "The page has no title",
            "n3",
        )
        .with_selector("html"),
    );
    results.fold_outcome(RuleRun::not_run("video-captions", NotRun::Disabled));
    results.fold_outcome(
        RuleRun::not_run("focus-order", NotRun::Errored).with_reason("CDP session closed"),
    );
    results.fold_outcome(RuleRun::not_run("no-forms", NotRun::NotApplicable));
    let mut report = AuditReport::new(url.to_string(), WcagLevel::AA, results, 100);
    report.accessibility.execution.module_runs.push(ModuleRun {
        module: "performance".to_string(),
        status: ExecutionStatus::Failed,
        message: Some("metrics unavailable".to_string()),
        ..Default::default()
    });
    // An errored rule and a failed module: the report says so itself.
    report.accessibility.execution.quality = AuditQuality {
        status: AuditQualityStatus::Partial,
        qualified_results: true,
        failed_rule_checks: 1,
        partial_or_failed_modules: 1,
        reasons: vec!["rule_checks_failed".to_string()],
    };
    report
}

fn sarif_for(reports: &[AuditReport]) -> Value {
    let mut ctxs: Vec<_> = reports
        .iter()
        .map(|r| normalize_checked(r).expect("consistent fixture"))
        .collect();
    // One finding is a heuristic one: it must not read as a confirmed defect.
    for f in &mut ctxs[0].normalized.findings {
        if f.rule_id.contains("page_titled") || f.wcag_criterion == "2.4.2" {
            f.contract.outcome = FindingOutcome::Review;
        }
    }
    let refs: Vec<_> = ctxs.iter().map(|c| &c.normalized).collect();
    serde_json::from_str(&format_sarif(&refs).expect("sarif")).expect("json")
}

fn fixture_sarif() -> Value {
    sarif_for(&[
        fixture_report("https://example.com/shop/?q=1#top", "Image has no alt"),
        fixture_report("https://example.com/", "Image has no alt"),
    ])
}

#[test]
fn golden_file_matches() {
    let mut value = fixture_sarif();
    // The tool version changes with every release; the golden stays put.
    value["runs"][0]["tool"]["driver"]["version"] = Value::String("<tool-version>".into());
    let actual = serde_json::to_string_pretty(&value).unwrap() + "\n";
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sarif/golden.sarif.json");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &actual).unwrap();
    }
    let expected = std::fs::read_to_string(&path).expect("golden file (UPDATE_GOLDEN=1 writes it)");
    assert_eq!(
        actual, expected,
        "SARIF output changed; review and rerun with UPDATE_GOLDEN=1"
    );
}

#[test]
fn results_carry_what_code_scanning_reads() {
    let sarif = fixture_sarif();
    let run = &sarif["runs"][0];
    let rules = run["tool"]["driver"]["rules"].as_array().unwrap();
    let results = run["results"].as_array().unwrap();
    assert!(!results.is_empty());

    for rule in rules {
        for field in ["shortDescription", "fullDescription", "help"] {
            assert!(
                rule[field]["text"].as_str().is_some_and(|t| !t.is_empty()),
                "{rule}"
            );
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for r in results {
        let idx = r["ruleIndex"].as_u64().unwrap() as usize;
        assert_eq!(
            rules[idx]["id"], r["ruleId"],
            "ruleIndex points at the rule"
        );
        assert!(matches!(
            r["level"].as_str().unwrap(),
            "error" | "warning" | "note"
        ));
        assert!(r["message"]["text"].as_str().is_some_and(|t| !t.is_empty()));
        let uri = r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
            .as_str()
            .unwrap();
        assert!(
            uri.starts_with("audited-pages/") && !uri.contains("://"),
            "{uri}"
        );
        // No position is known, so none may be made up.
        assert!(r["locations"][0]["physicalLocation"]
            .get("region")
            .is_none());
        let fp = r["partialFingerprints"]["primaryLocationLineHash"]
            .as_str()
            .unwrap();
        assert_eq!(
            fp,
            r["partialFingerprints"]["auditmysite/v1"].as_str().unwrap()
        );
        assert!(seen.insert(fp.to_string()), "fingerprint {fp} repeats");
    }
}

#[test]
fn pseudo_path_keeps_pages_apart_and_the_url_findable() {
    let sarif = fixture_sarif();
    let run = &sarif["runs"][0];
    let artifacts: Vec<(&str, &str)> = run["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            (
                a["location"]["uri"].as_str().unwrap(),
                a["description"]["text"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(artifacts.len(), 2);
    assert!(artifacts[0]
        .0
        .starts_with("audited-pages/example.com/shop/index__q-"));
    // The fragment is not part of the page identity.
    assert_eq!(artifacts[0].1, "Audited page https://example.com/shop/?q=1");
    assert_eq!(artifacts[1].0, "audited-pages/example.com/index");
}

#[test]
fn fingerprint_follows_selector_not_message_and_is_stable() {
    let a = sarif_for(&[fixture_report("https://example.com/", "Image has no alt")]);
    let again = sarif_for(&[fixture_report("https://example.com/", "Image has no alt")]);
    let reworded = sarif_for(&[fixture_report(
        "https://example.com/",
        "Missing alt text, 3 words",
    )]);
    let fps = |v: &Value| -> Vec<String> {
        v["runs"][0]["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                r["partialFingerprints"]["auditmysite/v1"]
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect()
    };
    assert_eq!(fps(&a), fps(&again), "same input, same fingerprints");
    assert_eq!(
        fps(&a),
        fps(&reworded),
        "a measured value in the message does not change the alert"
    );
    let other_page = sarif_for(&[fixture_report(
        "https://example.com/other",
        "Image has no alt",
    )]);
    assert_ne!(
        fps(&a),
        fps(&other_page),
        "the page is part of the identity"
    );
}

#[test]
fn review_findings_are_notes_not_confirmed_defects() {
    let sarif = fixture_sarif();
    let title = sarif["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["properties"]["outcome"] == "review")
        .expect("review result");
    assert_eq!(title["level"], "note");
}

#[test]
fn not_run_is_a_notification_never_a_result() {
    let sarif = fixture_sarif();
    let run = &sarif["runs"][0];
    let results = run["results"].as_array().unwrap();
    for rule in ["video-captions", "focus-order", "no-forms"] {
        assert!(
            results.iter().all(|r| r["ruleId"] != rule),
            "{rule} is no result"
        );
    }
    let invocation = &run["invocations"][0];
    let notes = invocation["toolExecutionNotifications"].as_array().unwrap();
    let find = |rule: &str| notes.iter().find(|n| n["associatedRule"]["id"] == rule);
    assert_eq!(find("video-captions").unwrap()["level"], "note");
    assert_eq!(find("focus-order").unwrap()["level"], "error");
    assert!(
        find("no-forms").is_none(),
        "not applicable is neither tested nor untested"
    );
    assert!(notes
        .iter()
        .any(|n| n["descriptor"]["id"] == "auditmysite.module_not_run" && n["level"] == "error"));
    assert_eq!(invocation["executionSuccessful"], false);
}

/// Opt-in: `AUDITMYSITE_SARIF_SCHEMA=/path/to/sarif-schema-2.1.0.json cargo test
/// --test sarif_contract -- --ignored`. The schema is the OASIS file from
/// `oasis-tcs/sarif-spec` (`sarif-2.1/schema/sarif-schema-2.1.0.json`).
#[test]
#[ignore = "needs the SARIF 2.1.0 schema file: set AUDITMYSITE_SARIF_SCHEMA"]
fn sarif_matches_the_official_schema() {
    let path = std::env::var("AUDITMYSITE_SARIF_SCHEMA")
        .expect("AUDITMYSITE_SARIF_SCHEMA must point at sarif-schema-2.1.0.json");
    let schema: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft4)
        .build(&schema)
        .expect("schema compiles");
    let sarif = fixture_sarif();
    let messages: Vec<String> = validator
        .iter_errors(&sarif)
        .map(|e| format!("{} at {}", e, e.instance_path()))
        .collect();
    assert!(
        messages.is_empty(),
        "SARIF violates the 2.1.0 schema:\n{}",
        messages.join("\n")
    );
}
