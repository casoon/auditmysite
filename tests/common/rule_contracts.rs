//! Executable rule contracts (#783).
//!
//! `tests/fixtures/rule_contracts/<domain>.toml` holds one `[[rule]]` per
//! active detector id. A contract states, before any run, what the detector
//! must and must not report: method, normative source, required data,
//! applicability, the corpus cases that prove a hit, a conforming look-alike,
//! a page it does not apply to and a technical failure, the variant matrix,
//! and how test strength was checked. `tests/rule_contract_gate.rs` holds the
//! contracts against the scanned inventory and the corpus; the corpus
//! harnesses read the per-rule facts the comparison needs from here.

use crate::common::corpus_diff::DiffRules;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Variant dimensions every contract must address, case by case or with a
/// reason why the dimension does not matter for this method.
pub const VARIANT_DIMENSIONS: &[&str] = &[
    "visibility",
    "native_vs_aria",
    "alternative_labels",
    "frames_shadow_dom",
    "responsive",
    "dynamic",
    "thresholds",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Automation {
    /// The detector decides on its own; it needs positive, negative,
    /// not-applicable and error evidence.
    Automated,
    /// The check can only flag for a human. It is never counted as an
    /// automatically validated detector; its cases prove the review flag.
    Manual,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    #[serde(default)]
    pub cases: Vec<String>,
    #[serde(default)]
    pub not_relevant: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationStatus {
    /// Every listed mutation was applied and turned the named test red.
    Killed,
    /// Detection logic offers no meaningful mutation point (a fixed review
    /// flag, pure data). Requires a reason.
    NotMutable,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    /// Where the mutation was applied: a source file relative to its repo
    /// (`src/...` here, `barrierlab:crates/...` for the shared crate).
    pub target: String,
    /// What was changed: disable detection, invert a condition, shift a
    /// threshold, drop a finding.
    pub change: String,
    /// The test that turned red: `corpus:<case>[,<case>]` (WCAG browser
    /// corpus), `test:<binary>[::<filter>]`, `lib:<filter>`, or for the
    /// shared crate the test named in barrierlab's own mutation catalog.
    pub killed_by: String,
    /// Exact source text to replace (local targets only; must occur once).
    #[serde(default)]
    pub find: Option<String>,
    #[serde(default)]
    pub replace: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestStrength {
    pub status: MutationStatus,
    #[serde(default)]
    pub mutations: Vec<Mutation>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleContract {
    pub id: String,
    pub automation: Automation,
    /// Why the check cannot be automated (manual rules only).
    #[serde(default)]
    pub manual_reason: Option<String>,
    /// `shared` (a11y-rules via `SHARED_RULES`), `tree` (AX-tree rule),
    /// `page` (browser page rule), `pipeline` (wired directly), or the
    /// non-WCAG analyzer name.
    pub method: String,
    /// Normative source: success criterion, spec section.
    pub source: String,
    /// Data the detector needs (`dom`, `ax_tree`, `computed_style`, ...).
    pub capabilities: Vec<String>,
    /// What content the rule applies to.
    pub applicability: String,
    /// The expected behaviour, stated from the source, not from a run.
    pub expectation: String,
    /// Findings concern the whole page and carry no element location.
    #[serde(default)]
    pub page_level: bool,
    /// The check applies to every page or response it can run on (a missing
    /// response header, the document's own title): there is no input it does
    /// not apply to, so no `not_applicable` case. `applicability` says why.
    #[serde(default)]
    pub always_applicable: bool,
    /// A manual check whose review note appears on every page: corpus cases
    /// that don't declare the rule accept the note, as a review only.
    #[serde(default)]
    pub ambient: bool,
    pub positive: Vec<String>,
    #[serde(default)]
    pub negative: Vec<String>,
    #[serde(default)]
    pub not_applicable: Vec<String>,
    /// Corpus cases that provoke a technical failure of the rule's method
    /// (verdict `errored`): the rule is recorded as not run, never as a pass.
    #[serde(default)]
    pub errored: Vec<String>,
    /// Where no page can provoke the failure (the analyzer's own transport),
    /// a test in `tests/rule_error_cases.rs` that does so instead.
    #[serde(default)]
    pub error_test: Option<String>,
    /// Direct function tests in the shared crate (browser-free rules).
    #[serde(default)]
    pub shared_tests: Vec<String>,
    pub variants: BTreeMap<String, Variant>,
    pub test_strength: TestStrength,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractFile {
    #[serde(default, rename = "rule")]
    pub rules: Vec<RuleContract>,
}

pub fn contracts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("rule_contracts")
}

/// Load `<domain>.toml` and every `<domain>/*.toml` (contracts may be split
/// by rule area). Panics on a missing or malformed file: a contract that
/// cannot be read is no contract.
pub fn load_contracts(domain: &str) -> Vec<RuleContract> {
    let mut paths = vec![contracts_dir().join(format!("{domain}.toml"))];
    if let Ok(entries) = std::fs::read_dir(contracts_dir().join(domain)) {
        let mut split: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
            .collect();
        split.sort();
        paths.extend(split);
    }
    let mut rules = Vec::new();
    for path in paths {
        if !path.exists() && path.parent() != Some(&contracts_dir()) {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let file: ContractFile =
            toml::from_str(&text).unwrap_or_else(|e| panic!("invalid {}: {e}", path.display()));
        rules.extend(file.rules);
    }
    rules
}

/// What the WCAG corpus comparison needs from the contracts.
pub fn wcag_diff_rules() -> DiffRules {
    DiffRules {
        page_level: load_contracts("wcag")
            .into_iter()
            .filter(|c| c.page_level)
            .map(|c| c.id)
            .collect(),
        run_counts_findings: true,
        ambient_review: load_contracts("wcag")
            .into_iter()
            .filter(|c| c.ambient)
            .map(|c| c.id)
            .collect(),
    }
}
