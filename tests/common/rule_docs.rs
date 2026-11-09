//! Generator for the per-rule documentation pages (#929).
//!
//! Joins the rule catalogue (`taxonomy::rules::RULES`: title, criterion,
//! impact), the rule contracts (`tests/fixtures/rule_contracts/wcag/*.toml`:
//! source, applicability, expectation, automation) and the detection corpus
//! (a failing and a passing fixture page). Nothing is written per rule by
//! hand: a section whose material the contract or corpus does not carry is
//! left out of the page.
//!
//! The output is JSON, one file per rule plus `index.json`, under
//! `site/src/data/rules/`; the Astro pages `site/src/pages/rules/` render it.
//! `tests/rule_docs.rs` holds the committed files to this generator.

use crate::common::detection_corpus::{
    detection_corpus_dir, load_corpus_dir, ExpectedCase, Verdict,
};
use crate::common::rule_contracts::{load_contracts, Automation, RuleContract};
use auditmysite::taxonomy::rules::RULES;
use auditmysite::taxonomy::{rule_doc_url, SeverityExt, RULE_DOCS_BASE_URL};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Largest fixture page shown as an example. A larger one is left out rather
/// than cut: a cut page is no longer the page the corpus asserts on.
const MAX_EXAMPLE_BYTES: usize = 6000;

/// Directory the generated files live in, relative to the repository root.
pub const OUTPUT_DIR: &str = "site/src/data/rules";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// The corpus pins at least one confirmed violation for the rule.
    Automatic,
    /// The rule's findings are review hints: no corpus page makes it fail.
    HintOnly,
    /// The contract says the check cannot be automated at all.
    Manual,
}

#[derive(Debug, Serialize)]
pub struct Example {
    /// Corpus case name (`tests/fixtures/detection_corpus/<case>.html`).
    pub case: String,
    /// Path of the fixture, relative to the repository root.
    pub file: String,
    /// The rule's verdict on the page: `violation`, `needs_review`, `pass`
    /// or `absent`.
    pub verdict: &'static str,
    /// Selector the corpus expectation names, if it names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    pub html: String,
}

#[derive(Debug, Serialize)]
pub struct RulePage {
    pub id: String,
    pub url: String,
    pub title: String,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axe_id: Option<String>,
    pub dimension: String,
    pub subcategory: String,
    pub issue_class: String,
    pub severity: String,
    pub user_impact: String,
    pub technical_impact: String,
    pub source: String,
    pub applicability: String,
    pub expectation: String,
    pub method: String,
    pub capabilities: Vec<String>,
    pub page_level: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing: Option<Example>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passing: Option<Example>,
}

#[derive(Debug, Serialize)]
pub struct IndexEntry {
    pub id: String,
    pub title: String,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Counts {
    pub automatic: usize,
    pub hint_only: usize,
    pub manual: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct Index {
    pub base_url: String,
    pub counts: Counts,
    /// Ordered by success criterion, then id.
    pub rules: Vec<IndexEntry>,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Path of a rule's file below [`OUTPUT_DIR`]: the id, `/` kept as directory
/// separator, plus `.json`.
pub fn page_file(rule_id: &str) -> String {
    format!("{rule_id}.json")
}

fn verdict_name(v: Verdict) -> &'static str {
    match v {
        Verdict::Violation => "violation",
        Verdict::NeedsReview => "needs_review",
        Verdict::Pass => "pass",
        Verdict::Absent => "absent",
        Verdict::NotApplicable => "not_applicable",
        Verdict::Errored => "errored",
    }
}

/// Sort key of a criterion ("2.5.10" after "2.5.9"); a rule without one last.
fn criterion_key(criterion: Option<&str>) -> (bool, Vec<u32>) {
    (
        criterion.is_none(),
        criterion
            .map(|c| c.split('.').filter_map(|p| p.parse().ok()).collect())
            .unwrap_or_default(),
    )
}

/// The most focused fixture page among `cases` on which the rule has one of
/// `verdicts` (preferred in the order given), within [`MAX_EXAMPLE_BYTES`]:
/// the page with the fewest confirmed violations of other rules, then the
/// smallest.
fn example(
    rule_id: &str,
    cases: &[String],
    corpus: &BTreeMap<String, ExpectedCase>,
    verdicts: &[Verdict],
) -> Option<Example> {
    for wanted in verdicts {
        let mut best: Option<((usize, usize), Example)> = None;
        for name in cases {
            let Some(case) = corpus.get(name) else {
                continue;
            };
            let Some(exp) = case
                .expectations
                .iter()
                .find(|e| e.rule_id == rule_id && e.verdict == *wanted)
            else {
                continue;
            };
            let Some(source) = case.source_file.as_deref() else {
                continue;
            };
            let html_path = html_path_of(source);
            let Ok(html) = std::fs::read_to_string(&html_path) else {
                continue;
            };
            if html.len() > MAX_EXAMPLE_BYTES {
                continue;
            }
            let others = case
                .expectations
                .iter()
                .filter(|e| e.rule_id != rule_id && e.verdict == Verdict::Violation)
                .count();
            let key = (others, html.len());
            if best.as_ref().is_none_or(|(k, _)| key < *k) {
                best = Some((
                    key,
                    Example {
                        case: name.clone(),
                        file: html_path
                            .strip_prefix(repo_root())
                            .unwrap_or(&html_path)
                            .to_string_lossy()
                            .replace('\\', "/"),
                        verdict: verdict_name(*wanted),
                        selector: exp.selector.clone(),
                        html: html.trim_end().to_string(),
                    },
                ));
            }
        }
        if let Some((_, example)) = best {
            return Some(example);
        }
    }
    None
}

/// `<case>.expected.json` → `<case>.html`.
fn html_path_of(expected: &Path) -> PathBuf {
    let name = expected.file_name().unwrap().to_string_lossy();
    expected.with_file_name(name.replace(".expected.json", ".html"))
}

/// Every generated file, as `(path relative to OUTPUT_DIR, content)`.
/// Panics when the catalogue and the contracts disagree: the contract gate
/// and `taxonomy_catalogue` fail first in that case, with a better message.
pub fn generate() -> BTreeMap<String, String> {
    let contracts: BTreeMap<String, RuleContract> = load_contracts("wcag")
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();
    let corpus: BTreeMap<String, ExpectedCase> = load_corpus_dir(&detection_corpus_dir())
        .into_iter()
        .map(|c| (c.case.clone(), c))
        .collect();

    let mut files = BTreeMap::new();
    let mut entries = Vec::new();
    for rule in RULES {
        let contract = contracts
            .get(rule.id)
            .unwrap_or_else(|| panic!("rule {} has no contract", rule.id));
        let confirmed = corpus.values().any(|c| {
            c.expectations
                .iter()
                .any(|e| e.rule_id == rule.id && e.verdict == Verdict::Violation)
        });
        let outcome = match contract.automation {
            Automation::Manual => Outcome::Manual,
            Automation::Automated if confirmed => Outcome::Automatic,
            Automation::Automated => Outcome::HintOnly,
        };
        let (failing_verdict, passing_verdicts): (Verdict, &[Verdict]) = match outcome {
            Outcome::Automatic => (Verdict::Violation, &[Verdict::Pass, Verdict::Absent]),
            Outcome::HintOnly => (Verdict::NeedsReview, &[Verdict::Pass, Verdict::Absent]),
            // A manual check proves its review flag, never a pass.
            Outcome::Manual => (Verdict::NeedsReview, &[]),
        };
        let failing = example(rule.id, &contract.positive, &corpus, &[failing_verdict]);
        let passing = example(rule.id, &contract.negative, &corpus, passing_verdicts);

        let criterion = rule
            .external_ref
            .and_then(|r| r.strip_prefix("WCAG "))
            .map(str::to_string);
        let level = rule.external_level.map(str::to_string);
        let page = RulePage {
            id: rule.id.to_string(),
            url: rule_doc_url(rule.id),
            title: rule.title_en.to_string(),
            outcome,
            criterion: criterion.clone(),
            level: level.clone(),
            axe_id: rule.axe_id.map(str::to_string),
            dimension: rule.dimension.label(true).to_string(),
            subcategory: rule.subcategory.label(true).to_string(),
            issue_class: rule.issue_class.label(true).to_string(),
            severity: rule.severity.label_en().to_lowercase(),
            user_impact: rule.user_impact_en.to_string(),
            technical_impact: rule.technical_impact_en.to_string(),
            source: contract.source.clone(),
            applicability: contract.applicability.clone(),
            expectation: contract.expectation.trim().to_string(),
            method: contract.method.clone(),
            capabilities: contract.capabilities.clone(),
            page_level: contract.page_level,
            manual_reason: contract.manual_reason.clone(),
            failing,
            passing,
        };
        files.insert(page_file(rule.id), to_json(&page));
        entries.push(IndexEntry {
            id: rule.id.to_string(),
            title: rule.title_en.to_string(),
            outcome,
            criterion,
            level,
        });
    }

    entries.sort_by(|a, b| {
        criterion_key(a.criterion.as_deref())
            .cmp(&criterion_key(b.criterion.as_deref()))
            .then_with(|| a.id.cmp(&b.id))
    });
    let count = |o: Outcome| entries.iter().filter(|e| e.outcome == o).count();
    let counts = Counts {
        automatic: count(Outcome::Automatic),
        hint_only: count(Outcome::HintOnly),
        manual: count(Outcome::Manual),
        total: entries.len(),
    };
    files.insert(
        "index.json".to_string(),
        to_json(&Index {
            base_url: RULE_DOCS_BASE_URL.to_string(),
            counts,
            rules: entries,
        }),
    );
    files
}

fn to_json<T: Serialize>(value: &T) -> String {
    let mut text = serde_json::to_string_pretty(value).expect("serializable");
    text.push('\n');
    text
}
