//! The corpus comparison must fail on every kind of wrong detector output
//! (#783). Each test feeds `diff_case` an observation manipulated in exactly
//! one way and asserts the matching mismatch — so a corpus run that is green
//! cannot be green because the comparison looks away. Browser-free.

mod common;

use common::corpus_diff::{
    diff_case, DiffRules, FindingClass, MismatchKind, Observation, ObservedFinding, ObservedRun,
    RunState,
};
use common::detection_corpus::{parse_case, ExpectedCase};

fn case(json: &str) -> ExpectedCase {
    parse_case(json).expect("test case parses")
}

fn fail(rule: &str, selector: &str) -> ObservedFinding {
    ObservedFinding {
        rule_id: rule.into(),
        selector: Some(selector.into()),
        class: FindingClass::Fail,
        located: true,
    }
}

fn review(rule: &str) -> ObservedFinding {
    ObservedFinding {
        rule_id: rule.into(),
        selector: None,
        class: FindingClass::Review,
        located: false,
    }
}

fn ran(rule: &str, findings: usize) -> ObservedRun {
    ObservedRun {
        rule_id: rule.into(),
        state: RunState::Ran { findings },
    }
}

fn errored(rule: &str) -> ObservedRun {
    ObservedRun {
        rule_id: rule.into(),
        state: RunState::NotRun("Errored".into()),
    }
}

fn rules() -> DiffRules {
    DiffRules {
        run_counts_findings: true,
        ..Default::default()
    }
}

fn kinds(case: &ExpectedCase, obs: &Observation) -> Vec<MismatchKind> {
    diff_case(case, obs, &rules())
        .into_iter()
        .map(|m| m.kind)
        .collect()
}

const MIXED: &str = r#"{ "case": "mixed", "expectations": [
    { "rule_id": "image-alt", "selector": "img.a", "verdict": "violation" },
    { "rule_id": "image-alt", "selector": "img.b", "verdict": "pass" },
    { "rule_id": "dialog/modal-unmarked", "verdict": "needs_review" },
    { "rule_id": "headings/empty", "verdict": "not_applicable" }
] }"#;

fn exact() -> Observation {
    Observation {
        findings: vec![fail("image-alt", "img.a"), review("dialog/modal-unmarked")],
        runs: vec![
            ran("image-alt", 1),
            ran("dialog/modal-unmarked", 0),
            ran("headings/empty", 0),
        ],
    }
}

#[test]
fn exact_observation_has_no_mismatch() {
    assert_eq!(kinds(&case(MIXED), &exact()), vec![]);
}

#[test]
fn missing_violation_is_a_false_negative() {
    let mut obs = exact();
    obs.findings.retain(|f| f.rule_id != "image-alt");
    obs.runs[0] = ran("image-alt", 0);
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::MissingExpected]
    );
}

#[test]
fn unexpected_extra_finding_is_reported() {
    let mut obs = exact();
    obs.findings.push(fail("links/name-missing", "a.x"));
    obs.runs.push(ran("links/name-missing", 1));
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::UnexpectedFinding]
    );
}

#[test]
fn unexpected_review_finding_is_reported() {
    let mut obs = exact();
    obs.findings.push(review("manual/help"));
    obs.runs.push(ran("manual/help", 0));
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::UnexpectedFinding]
    );
}

#[test]
fn violation_on_a_pass_element_is_reported() {
    let mut obs = exact();
    obs.findings.push(fail("image-alt", "img.b"));
    obs.runs[0] = ran("image-alt", 2);
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::UnexpectedFinding]
    );
}

#[test]
fn review_escalated_to_fail_is_reported() {
    let mut obs = exact();
    obs.findings
        .retain(|f| f.rule_id != "dialog/modal-unmarked");
    obs.findings.push(fail("dialog/modal-unmarked", "div.dlg"));
    obs.runs[1] = ran("dialog/modal-unmarked", 1);
    let k = kinds(&case(MIXED), &obs);
    assert!(k.contains(&MismatchKind::ReviewEscalatedToFail), "{k:?}");
    assert!(k.contains(&MismatchKind::MissingExpected), "{k:?}");
}

#[test]
fn review_next_to_an_escalated_fail_is_still_reported() {
    let mut obs = exact();
    obs.findings.push(fail("dialog/modal-unmarked", "div.dlg"));
    obs.runs[1] = ran("dialog/modal-unmarked", 1);
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::ReviewEscalatedToFail]
    );
}

#[test]
fn pass_without_a_run_record_is_reported() {
    let mut obs = exact();
    obs.findings.retain(|f| f.rule_id != "image-alt");
    obs.runs.retain(|r| r.rule_id != "image-alt");
    let k = kinds(&case(MIXED), &obs);
    assert!(k.contains(&MismatchKind::PassWithoutRun), "{k:?}");
}

#[test]
fn pass_with_an_errored_run_is_reported() {
    let c =
        case(r#"{ "case": "p", "expectations": [{ "rule_id": "image-alt", "verdict": "pass" }] }"#);
    let obs = Observation {
        findings: vec![],
        runs: vec![errored("image-alt")],
    };
    assert_eq!(kinds(&c, &obs), vec![MismatchKind::PassWithoutRun]);
}

#[test]
fn not_applicable_requires_a_run() {
    let mut obs = exact();
    obs.runs.retain(|r| r.rule_id != "headings/empty");
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::NotApplicableWithoutRun]
    );
}

#[test]
fn not_applicable_with_a_finding_is_reported() {
    let mut obs = exact();
    obs.findings.push(review("headings/empty"));
    let k = kinds(&case(MIXED), &obs);
    assert!(k.contains(&MismatchKind::NotApplicableWithFinding), "{k:?}");
}

#[test]
fn finding_without_any_run_record_is_reported() {
    let c = case(
        r#"{ "case": "f", "expectations": [{ "rule_id": "image-alt", "verdict": "violation" }] }"#,
    );
    let obs = Observation {
        findings: vec![fail("image-alt", "img")],
        runs: vec![],
    };
    assert_eq!(kinds(&c, &obs), vec![MismatchKind::FindingWithoutRun]);
}

#[test]
fn clean_run_next_to_a_violation_is_reported() {
    let mut obs = exact();
    obs.runs[0] = ran("image-alt", 0);
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::RunContradictsFindings]
    );
}

#[test]
fn run_counting_review_findings_is_consistent() {
    let mut obs = exact();
    obs.runs[1] = ran("dialog/modal-unmarked", 1);
    assert_eq!(kinds(&case(MIXED), &obs), vec![]);
}

#[test]
fn run_counting_more_than_reported_is_reported() {
    let mut obs = exact();
    obs.runs[0] = ran("image-alt", 3);
    assert_eq!(
        kinds(&case(MIXED), &obs),
        vec![MismatchKind::RunContradictsFindings]
    );
}

#[test]
fn violation_without_location_is_lost_evidence() {
    let mut obs = exact();
    obs.findings[0].located = false;
    assert_eq!(kinds(&case(MIXED), &obs), vec![MismatchKind::LostEvidence]);
}

#[test]
fn page_level_violation_needs_no_location() {
    let c = case(
        r#"{ "case": "t", "expectations": [{ "rule_id": "document/title-missing", "verdict": "violation" }] }"#,
    );
    let mut f = fail("document/title-missing", "");
    f.selector = None;
    f.located = false;
    let obs = Observation {
        findings: vec![f],
        runs: vec![ran("document/title-missing", 1)],
    };
    let mut r = rules();
    r.page_level.insert("document/title-missing".into());
    assert!(diff_case(&c, &obs, &r).is_empty());
}

#[test]
fn wrong_occurrence_count_is_reported() {
    let c = case(
        r#"{ "case": "o", "expectations": [{ "rule_id": "image-alt", "verdict": "violation", "occurrences": 2 }] }"#,
    );
    let obs = Observation {
        findings: vec![fail("image-alt", "img")],
        runs: vec![ran("image-alt", 1)],
    };
    assert_eq!(kinds(&c, &obs), vec![MismatchKind::WrongOccurrences]);
}

#[test]
fn absent_is_violated_by_a_review_finding() {
    let c = case(
        r#"{ "case": "a", "expectations": [{ "rule_id": "manual/help", "verdict": "absent" }] }"#,
    );
    let obs = Observation {
        findings: vec![review("manual/help")],
        runs: vec![ran("manual/help", 0)],
    };
    assert_eq!(kinds(&c, &obs), vec![MismatchKind::UnexpectedFinding]);
}

#[test]
fn ambient_review_note_is_accounted_but_not_as_violation() {
    let mut r = rules();
    r.ambient_review.insert("manual/help".into());
    let mut obs = exact();
    obs.findings.push(review("manual/help"));
    obs.runs.push(ran("manual/help", 1));
    assert!(diff_case(&case(MIXED), &obs, &r).is_empty());

    obs.findings.pop();
    obs.findings.push(fail("manual/help", "html"));
    let k: Vec<_> = diff_case(&case(MIXED), &obs, &r)
        .into_iter()
        .map(|m| m.kind)
        .collect();
    assert_eq!(k, vec![MismatchKind::UnexpectedFinding]);
}

#[test]
fn ambient_rule_declared_by_the_case_follows_the_declaration() {
    let mut r = rules();
    r.ambient_review.insert("manual/help".into());
    let c = case(
        r#"{ "case": "h", "expectations": [{ "rule_id": "manual/help", "verdict": "absent" }] }"#,
    );
    let obs = Observation {
        findings: vec![review("manual/help")],
        runs: vec![ran("manual/help", 1)],
    };
    let k: Vec<_> = diff_case(&c, &obs, &r)
        .into_iter()
        .map(|m| m.kind)
        .collect();
    assert_eq!(k, vec![MismatchKind::UnexpectedFinding]);
}

#[test]
fn technical_failure_must_be_recorded_as_not_run() {
    let c = case(
        r#"{ "case": "e", "expectations": [{ "rule_id": "image-alt", "verdict": "errored" }] }"#,
    );
    let errored_run = Observation {
        findings: vec![],
        runs: vec![errored("image-alt")],
    };
    assert_eq!(kinds(&c, &errored_run), vec![]);

    // The same failure passed off as a clean run.
    let clean = Observation {
        findings: vec![],
        runs: vec![ran("image-alt", 0)],
    };
    assert_eq!(kinds(&c, &clean), vec![MismatchKind::ErrorNotReported]);

    // Silently missing from the record.
    let silent = Observation::default();
    assert_eq!(kinds(&c, &silent), vec![MismatchKind::ErrorNotReported]);
}
