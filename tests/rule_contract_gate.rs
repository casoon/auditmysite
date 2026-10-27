//! Release gate for detector contracts (#783). Browser-free and not ignored:
//! it runs in every `cargo test`, locally and in CI, so a rule or subcheck
//! added without a complete contract fails verification right away.
//!
//! For every domain the contracts in `tests/fixtures/rule_contracts/` must
//! match the scanned inventory exactly — a new id without contract fails, and
//! so does a contract for an id nothing produces any more (a dead or
//! historical alias). Each contract then has to hold up against the corpus:
//! the cases it names exist and declare the verdict the contract claims.
//! Deferred entries are not accepted.

mod common;

use common::detection_corpus::{
    detection_corpus_dir, load_corpus_dir, load_structurally_deferred,
    nonwcag_detection_corpus_dir, ExpectedCase, Verdict,
};
use common::rule_contracts::{
    load_contracts, Automation, MutationStatus, RuleContract, VARIANT_DIMENSIONS,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

struct Domain {
    name: &'static str,
    inventory: BTreeSet<String>,
    corpus_dir: PathBuf,
}

fn domains() -> Vec<Domain> {
    vec![
        Domain {
            name: "wcag",
            inventory: common::rule_inventory::canonical_rule_ids(),
            corpus_dir: detection_corpus_dir(),
        },
        Domain {
            name: "security",
            inventory: common::nonwcag_rule_inventory::canonical_security_check_ids(),
            corpus_dir: nonwcag_detection_corpus_dir("security"),
        },
        Domain {
            name: "schema_rules",
            inventory: common::nonwcag_rule_inventory::canonical_schema_feature_ids(),
            corpus_dir: nonwcag_detection_corpus_dir("schema_rules"),
        },
        Domain {
            name: "vulnerable_libs",
            inventory: common::nonwcag_rule_inventory::canonical_vulnerable_library_ids(),
            corpus_dir: nonwcag_detection_corpus_dir("vulnerable_libs"),
        },
    ]
}

/// Error-case tests a contract may name: every `fn` in
/// `tests/rule_error_cases.rs` (none while the file does not exist).
fn error_case_tests() -> BTreeSet<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/rule_error_cases.rs");
    let Ok(source) = std::fs::read_to_string(&path) else {
        return BTreeSet::new();
    };
    source
        .lines()
        .filter_map(|line| {
            let rest = line
                .trim_start()
                .strip_prefix("async fn ")
                .or_else(|| line.trim_start().strip_prefix("fn "))?;
            Some(rest.split('(').next()?.to_string())
        })
        .collect()
}

fn verdicts_for(
    cases: &BTreeMap<String, ExpectedCase>,
    case: &str,
    id: &str,
) -> Option<Vec<Verdict>> {
    cases.get(case).map(|c| {
        c.expectations
            .iter()
            .filter(|e| e.rule_id == id)
            .map(|e| e.verdict)
            .collect()
    })
}

/// The cases of one contract field must exist and declare a fitting verdict
/// for the rule.
fn check_cases(
    p: &mut Vec<String>,
    cases: &BTreeMap<String, ExpectedCase>,
    id: &str,
    kind: &str,
    list: &[String],
    ok: &dyn Fn(Verdict) -> bool,
) {
    if list.is_empty() {
        p.push(format!("{id}: no `{kind}` case"));
    }
    for case in list {
        match verdicts_for(cases, case, id) {
            None => p.push(format!("{id}: `{kind}` names unknown case '{case}'")),
            Some(v) if !v.iter().any(|v| ok(*v)) => p.push(format!(
                "{id}: `{kind}` case '{case}' declares {v:?} for this rule"
            )),
            Some(_) => {}
        }
    }
}

/// Everything wrong with one contract, against its domain's corpus.
fn contract_problems(
    c: &RuleContract,
    cases: &BTreeMap<String, ExpectedCase>,
    error_tests: &BTreeSet<String>,
) -> Vec<String> {
    let mut p = Vec::new();
    let id = &c.id;
    for (field, value) in [
        ("method", &c.method),
        ("source", &c.source),
        ("applicability", &c.applicability),
        ("expectation", &c.expectation),
    ] {
        if value.trim().is_empty() {
            p.push(format!("{id}: `{field}` is empty"));
        }
    }
    if c.capabilities.is_empty() {
        p.push(format!("{id}: no `capabilities`"));
    }

    match c.automation {
        Automation::Automated => {
            check_cases(&mut p, cases, id, "positive", &c.positive, &|v| {
                matches!(v, Verdict::Violation | Verdict::NeedsReview)
            });
            check_cases(&mut p, cases, id, "negative", &c.negative, &|v| {
                matches!(v, Verdict::Pass | Verdict::Absent)
            });
            if c.always_applicable {
                if !c.not_applicable.is_empty() {
                    p.push(format!(
                        "{id}: always applicable, yet names not_applicable cases"
                    ));
                }
            } else {
                check_cases(
                    &mut p,
                    cases,
                    id,
                    "not_applicable",
                    &c.not_applicable,
                    &|v| v == Verdict::NotApplicable,
                );
            }
            if c.errored.is_empty() && c.error_test.is_none() {
                p.push(format!(
                    "{id}: no technical error case (`errored` or `error_test`)"
                ));
            } else if !c.errored.is_empty() {
                check_cases(&mut p, cases, id, "errored", &c.errored, &|v| {
                    v == Verdict::Errored
                });
            }
            if let Some(t) = &c.error_test {
                if !error_tests.contains(t) {
                    p.push(format!(
                        "{id}: `error_test` '{t}' is not a test in tests/rule_error_cases.rs"
                    ));
                }
            }
            if c.manual_reason.is_some() {
                p.push(format!("{id}: automated rule with a `manual_reason`"));
            }
        }
        Automation::Manual => {
            if c.manual_reason
                .as_deref()
                .is_none_or(|r| r.trim().is_empty())
            {
                p.push(format!("{id}: manual rule without `manual_reason`"));
            }
            // A manual check proves its review flag, never a pass.
            check_cases(&mut p, cases, id, "positive", &c.positive, &|v| {
                v == Verdict::NeedsReview
            });
            // A note that appears on every page has no page it doesn't apply to.
            if !c.ambient && !c.always_applicable {
                check_cases(
                    &mut p,
                    cases,
                    id,
                    "not_applicable",
                    &c.not_applicable,
                    &|v| matches!(v, Verdict::NotApplicable | Verdict::Absent),
                );
            }
            for (name, case) in cases {
                if case.expectations.iter().any(|e| {
                    &e.rule_id == id && matches!(e.verdict, Verdict::Pass | Verdict::Violation)
                }) {
                    p.push(format!(
                        "{id}: manual rule asserted as pass/violation in case '{name}'"
                    ));
                }
            }
        }
    }

    if c.method == "shared" && c.shared_tests.is_empty() {
        p.push(format!("{id}: shared rule without direct `shared_tests`"));
    }

    for dim in VARIANT_DIMENSIONS {
        match c.variants.get(*dim) {
            None => p.push(format!("{id}: variant `{dim}` not addressed")),
            Some(v) => {
                let reason = v.not_relevant.as_deref().map(str::trim).unwrap_or("");
                if v.cases.is_empty() && reason.is_empty() {
                    p.push(format!(
                        "{id}: variant `{dim}` has neither cases nor a reason"
                    ));
                }
                for case in &v.cases {
                    match verdicts_for(cases, case, id) {
                        None => {
                            p.push(format!("{id}: variant `{dim}` names unknown case '{case}'"))
                        }
                        Some(v) if v.is_empty() => p.push(format!(
                            "{id}: variant `{dim}` case '{case}' has no expectation for this rule"
                        )),
                        Some(_) => {}
                    }
                }
            }
        }
    }
    for dim in c.variants.keys() {
        if !VARIANT_DIMENSIONS.contains(&dim.as_str()) {
            p.push(format!("{id}: unknown variant dimension `{dim}`"));
        }
    }

    let s = &c.test_strength;
    match s.status {
        MutationStatus::Killed if s.mutations.is_empty() => {
            p.push(format!("{id}: test strength `killed` without mutations"))
        }
        MutationStatus::NotMutable if s.reason.as_deref().is_none_or(|r| r.trim().is_empty()) => {
            p.push(format!("{id}: test strength `not_mutable` without reason"))
        }
        _ => {}
    }
    if c.ambient && c.automation != Automation::Manual {
        p.push(format!("{id}: only manual checks may be ambient"));
    }
    for m in &s.mutations {
        if m.target.trim().is_empty() || m.change.trim().is_empty() || m.killed_by.trim().is_empty()
        {
            p.push(format!("{id}: incomplete mutation entry {m:?}"));
        }
        if m.target.starts_with("barrierlab:") {
            continue;
        }
        // A local mutation must still apply to the code as it is.
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&m.target);
        match (&m.find, &m.replace, std::fs::read_to_string(&path)) {
            (Some(find), Some(_), Ok(source)) => {
                let n = source.matches(find.as_str()).count();
                if n != 1 {
                    p.push(format!(
                        "{id}: mutation `find` occurs {n} times in {}, expected once",
                        m.target
                    ));
                }
            }
            (_, _, Err(e)) => p.push(format!("{id}: mutation target {}: {e}", m.target)),
            _ => p.push(format!("{id}: local mutation without `find`/`replace`")),
        }
    }
    p
}

#[test]
fn every_active_detector_has_a_complete_contract() {
    let error_tests = error_case_tests();
    let mut problems = Vec::new();
    for d in domains() {
        let contracts = load_contracts(d.name);
        let cases: BTreeMap<String, ExpectedCase> = load_corpus_dir(&d.corpus_dir)
            .into_iter()
            .map(|c| (c.case.clone(), c))
            .collect();

        let deferred = load_structurally_deferred(&d.corpus_dir);
        for entry in &deferred {
            problems.push(format!(
                "{}: '{}' is structurally deferred — deferred entries never pass the gate",
                d.name, entry.rule_id
            ));
        }

        let mut seen = BTreeSet::new();
        for c in &contracts {
            if !seen.insert(c.id.clone()) {
                problems.push(format!("{}: duplicate contract '{}'", d.name, c.id));
            }
        }
        for id in d.inventory.difference(&seen) {
            problems.push(format!("{}: '{id}' is active but has no contract", d.name));
        }
        for id in seen.difference(&d.inventory) {
            problems.push(format!(
                "{}: contract '{id}' names no active detector (dead id or old alias)",
                d.name
            ));
        }
        for c in &contracts {
            problems.extend(
                contract_problems(c, &cases, &error_tests)
                    .into_iter()
                    .map(|p| format!("{}: {p}", d.name)),
            );
        }

        // Every id the corpus asserts on must be a contracted detector.
        for case in cases.values() {
            for e in &case.expectations {
                if !seen.contains(&e.rule_id) {
                    problems.push(format!(
                        "{}: case '{}' asserts on '{}', which has no contract",
                        d.name, case.case, e.rule_id
                    ));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{} contract problem(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

/// The levels #783 asks to keep apart. Printed, so a high case count cannot
/// pass for a complete matrix; the hard assertions live in the test above.
#[test]
fn coverage_levels_report() {
    let error_tests = error_case_tests();
    for d in domains() {
        let contracts = load_contracts(d.name);
        let cases: BTreeMap<String, ExpectedCase> = load_corpus_dir(&d.corpus_dir)
            .into_iter()
            .map(|c| (c.case.clone(), c))
            .collect();
        let any_case = d
            .inventory
            .iter()
            .filter(|id| {
                cases
                    .values()
                    .any(|c| c.expectations.iter().any(|e| &&e.rule_id == id))
            })
            .count();
        let automated: Vec<_> = contracts
            .iter()
            .filter(|c| c.automation == Automation::Automated)
            .collect();
        let pos_neg = automated
            .iter()
            .filter(|c| !c.positive.is_empty() && !c.negative.is_empty())
            .count();
        let full = contracts
            .iter()
            .filter(|c| contract_problems(c, &cases, &error_tests).is_empty())
            .count();
        let strength = contracts
            .iter()
            .filter(|c| c.test_strength.status == MutationStatus::Killed)
            .count();
        println!(
            "[{}] {} ids: {any_case} with any case | {} automated, {} manual | {pos_neg} automated with positive+negative | {full} with complete contract (cases, N/A, error, variants) | {strength} with mutation-checked strength",
            d.name,
            d.inventory.len(),
            automated.len(),
            contracts.len() - automated.len(),
        );
    }
}
