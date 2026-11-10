//! Generator for the per-rule documentation pages (#929).
//!
//! Joins the rule catalogue (`taxonomy::rules::RULES`: title, criterion,
//! impact), the rule contracts (`tests/fixtures/rule_contracts/`: source,
//! applicability, expectation, automation) and the detection corpus (a failing
//! and a passing fixture page) into one page per documented finding id. Nothing
//! is written per rule by hand: a section whose material the contract or
//! corpus does not carry is left out of the page.
//!
//! Four families, one URL scheme (`audit::rule_docs` maps contract to page):
//! the WCAG rules, the security header checks, the structured-data features
//! and the checked JavaScript libraries.
//!
//! The output is JSON, one file per page plus `index.json`, under
//! `site/src/data/rules/`; the Astro pages `site/src/pages/rules/` render it.
//! `tests/rule_docs.rs` holds the committed files to this generator.

use crate::common::detection_corpus::{
    detection_corpus_dir, load_corpus_dir, nonwcag_detection_corpus_dir, ExpectedCase, Verdict,
};
use crate::common::rule_contracts::{load_contracts, Automation, RuleContract};
use auditmysite::audit::finding_contract::check_title;
use auditmysite::audit::rule_docs::{library_page_id, schema_page_id, security_page_id};
use auditmysite::seo::schema_rules::{feature_label, SchemaFeature};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// An automated detector that can report a confirmed violation.
    Automatic,
    /// An automated detector that by design reports review hints only
    /// (`review_only` in its contract).
    HintOnly,
    /// The contract says the check cannot be automated at all.
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    Wcag,
    Security,
    Schema,
    Libraries,
}

impl Domain {
    const ALL: [Domain; 4] = [
        Domain::Wcag,
        Domain::Security,
        Domain::Schema,
        Domain::Libraries,
    ];

    fn label(self) -> &'static str {
        match self {
            Domain::Wcag => "WCAG rules",
            Domain::Security => "Security headers",
            Domain::Schema => "Structured data",
            Domain::Libraries => "Vulnerable libraries",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Example {
    /// Corpus case name (`<corpus dir>/<case>.html`).
    pub case: String,
    /// Path of the fixture (page, or the case file for a headers-only case),
    /// relative to the repository root.
    pub file: String,
    /// The rule's verdict on the page: `violation`, `needs_review`, `pass`
    /// or `absent`.
    pub verdict: &'static str,
    /// Selector the corpus expectation names, if it names one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Response headers the fixture is served with (security corpus).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub response_headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub html: String,
}

/// What one contract says about one detector. A page documents one check
/// (a WCAG rule) or several that surface as the same finding (the security
/// headers sharing an issue type).
#[derive(Debug, Serialize)]
pub struct Check {
    /// The contract's id.
    pub contract_id: String,
    /// Which of several checks on the page this is (the header name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub outcome: Outcome,
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
pub struct RulePage {
    /// The page id: what follows `rules/` in the URL.
    pub id: String,
    pub url: String,
    pub domain: Domain,
    pub title: String,
    pub outcome: Outcome,
    /// Where the rule sits in the report, e.g. "Accessibility, Images".
    pub area: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axe_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_impact: Option<String>,
    pub checks: Vec<Check>,
}

#[derive(Debug, Serialize)]
pub struct IndexEntry {
    pub id: String,
    pub title: String,
    pub domain: Domain,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Counts {
    pub domain: Domain,
    pub label: &'static str,
    pub automatic: usize,
    pub hint_only: usize,
    pub manual: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct Index {
    pub base_url: String,
    /// One row per domain, then the total as the last row's sum is left to
    /// the page: `total` carries it.
    pub counts: Vec<Counts>,
    pub total: usize,
    /// WCAG rules by success criterion, then the other families by id.
    pub rules: Vec<IndexEntry>,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Path of a page's file below [`OUTPUT_DIR`]: the id, `/` kept as directory
/// separator, plus `.json`.
pub fn page_file(page_id: &str) -> String {
    format!("{page_id}.json")
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
    contract_id: &str,
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
                .find(|e| e.rule_id == contract_id && e.verdict == *wanted)
            else {
                continue;
            };
            let Some(source) = case.source_file.as_deref() else {
                continue;
            };
            // The security corpus is a set of response headers (possibly
            // none at all); its cases have no page.
            let mut html_path = html_path_of(source);
            let html = match std::fs::read_to_string(&html_path) {
                Ok(html) => html,
                Err(_) => {
                    html_path = source.to_path_buf();
                    String::new()
                }
            };
            if html.len() > MAX_EXAMPLE_BYTES {
                continue;
            }
            let others = case
                .expectations
                .iter()
                .filter(|e| e.rule_id != contract_id && e.verdict == Verdict::Violation)
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
                        response_headers: case.response_headers.clone(),
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

fn outcome_of(contract: &RuleContract) -> Outcome {
    match contract.automation {
        Automation::Manual => Outcome::Manual,
        Automation::Automated if contract.review_only => Outcome::HintOnly,
        Automation::Automated => Outcome::Automatic,
    }
}

/// The strongest class among a page's checks: a page that can fail is
/// automatic, one that only hints is hint-only, only a page of manual checks
/// is manual.
fn page_outcome(checks: &[Check]) -> Outcome {
    checks
        .iter()
        .map(|c| c.outcome)
        .min()
        .expect("a page has a check")
}

fn check_of(
    contract: &RuleContract,
    label: Option<String>,
    corpus: &BTreeMap<String, ExpectedCase>,
) -> Check {
    let outcome = outcome_of(contract);
    let (failing_verdict, passing_verdicts): (Verdict, &[Verdict]) = match outcome {
        Outcome::Automatic => (Verdict::Violation, &[Verdict::Pass, Verdict::Absent]),
        Outcome::HintOnly => (Verdict::NeedsReview, &[Verdict::Pass, Verdict::Absent]),
        // A manual check proves its review flag, never a pass.
        Outcome::Manual => (Verdict::NeedsReview, &[]),
    };
    Check {
        contract_id: contract.id.clone(),
        label,
        outcome,
        source: contract.source.clone(),
        applicability: contract.applicability.clone(),
        expectation: contract.expectation.trim().to_string(),
        method: contract.method.clone(),
        capabilities: contract.capabilities.clone(),
        page_level: contract.page_level,
        manual_reason: contract.manual_reason.clone(),
        failing: example(&contract.id, &contract.positive, corpus, &[failing_verdict]),
        passing: example(&contract.id, &contract.negative, corpus, passing_verdicts),
    }
}

fn corpus_of(dir: &Path) -> BTreeMap<String, ExpectedCase> {
    load_corpus_dir(dir)
        .into_iter()
        .map(|c| (c.case.clone(), c))
        .collect()
}

fn contracts_of(domain: &str) -> BTreeMap<String, RuleContract> {
    load_contracts(domain)
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect()
}

/// Every page, in generation order. Panics when the catalogue and the
/// contracts disagree: the contract gate and `taxonomy_catalogue` fail first
/// in that case, with a better message.
pub fn pages() -> Vec<RulePage> {
    let mut pages = Vec::new();

    // WCAG: one page per catalogue rule.
    let contracts = contracts_of("wcag");
    let corpus = corpus_of(&detection_corpus_dir());
    for rule in RULES {
        let contract = contracts
            .get(rule.id)
            .unwrap_or_else(|| panic!("rule {} has no contract", rule.id));
        let checks = vec![check_of(contract, None, &corpus)];
        pages.push(RulePage {
            id: rule.id.to_string(),
            url: rule_doc_url(rule.id),
            domain: Domain::Wcag,
            title: rule.title(true),
            outcome: page_outcome(&checks),
            area: format!(
                "{}, {}",
                rule.dimension.label(true),
                rule.subcategory.label(true)
            ),
            criterion: rule
                .external_ref
                .and_then(|r| r.strip_prefix("WCAG "))
                .map(str::to_string),
            level: rule.external_level.map(str::to_string),
            axe_id: rule.axe_id.map(str::to_string),
            severity: Some(rule.severity.label_en().to_lowercase()),
            user_impact: Some(rule.user_impact(true)),
            technical_impact: Some(rule.technical_impact(true)),
            checks,
        });
    }

    // Security: the checks of one issue type are one finding, one page.
    let contracts = contracts_of("security");
    let corpus = corpus_of(&nonwcag_detection_corpus_dir("security"));
    let mut by_page: BTreeMap<String, Vec<Check>> = BTreeMap::new();
    for (id, contract) in &contracts {
        let page_id = security_page_id(id).unwrap_or_else(|| panic!("{id} is not header:type"));
        let header = id.split_once(':').map(|(h, _)| h.to_string());
        by_page
            .entry(page_id)
            .or_default()
            .push(check_of(contract, header, &corpus));
    }
    for (page_id, checks) in by_page {
        let issue_type = page_id.rsplit('.').next().unwrap().to_string();
        // The title the finding itself carries in the JSON.
        let title = check_title(&issue_type);
        pages.push(RulePage {
            url: rule_doc_url(&page_id),
            id: page_id,
            domain: Domain::Security,
            title,
            outcome: page_outcome(&checks),
            area: "Security, response headers".to_string(),
            criterion: None,
            level: None,
            axe_id: None,
            severity: None,
            user_impact: None,
            technical_impact: None,
            checks,
        });
    }

    // Structured data: one page per feature.
    let contracts = contracts_of("schema_rules");
    let corpus = corpus_of(&nonwcag_detection_corpus_dir("schema_rules"));
    for feature in SchemaFeature::ALL {
        let contract = contracts
            .get(feature.key())
            .unwrap_or_else(|| panic!("schema feature {} has no contract", feature.key()));
        let page_id = schema_page_id(feature.key());
        let checks = vec![check_of(contract, None, &corpus)];
        pages.push(RulePage {
            url: rule_doc_url(&page_id),
            id: page_id,
            domain: Domain::Schema,
            title: format!(
                "{}: missing required properties",
                feature_label(feature, true)
            ),
            outcome: page_outcome(&checks),
            area: "SEO, structured data".to_string(),
            criterion: None,
            level: None,
            axe_id: None,
            severity: None,
            user_impact: None,
            technical_impact: None,
            checks,
        });
    }

    // Vulnerable libraries: one page per checked library.
    let contracts = contracts_of("vulnerable_libs");
    let corpus = corpus_of(&nonwcag_detection_corpus_dir("vulnerable_libs"));
    for name in auditmysite::best_practices::CHECKED_LIBRARIES {
        let contract = contracts
            .get(*name)
            .unwrap_or_else(|| panic!("library {name} has no contract"));
        let page_id = library_page_id(name);
        let checks = vec![check_of(contract, None, &corpus)];
        pages.push(RulePage {
            url: rule_doc_url(&page_id),
            id: page_id,
            domain: Domain::Libraries,
            title: format!("{name}: version with known vulnerabilities"),
            outcome: page_outcome(&checks),
            area: "Security, vulnerable libraries".to_string(),
            criterion: None,
            level: None,
            axe_id: None,
            severity: None,
            user_impact: None,
            technical_impact: None,
            checks,
        });
    }
    pages
}

/// Every generated file, as `(path relative to OUTPUT_DIR, content)`.
pub fn generate() -> BTreeMap<String, String> {
    let pages = pages();
    let mut files = BTreeMap::new();
    let mut entries = Vec::new();
    for page in &pages {
        let previous = files.insert(page_file(&page.id), to_json(page));
        assert!(previous.is_none(), "two pages with the id {}", page.id);
        entries.push(IndexEntry {
            id: page.id.clone(),
            title: page.title.clone(),
            domain: page.domain,
            outcome: page.outcome,
            criterion: page.criterion.clone(),
            level: page.level.clone(),
        });
    }
    entries.sort_by(|a, b| {
        a.domain
            .cmp(&b.domain)
            .then_with(|| {
                criterion_key(a.criterion.as_deref()).cmp(&criterion_key(b.criterion.as_deref()))
            })
            .then_with(|| a.id.cmp(&b.id))
    });
    let counts = Domain::ALL
        .iter()
        .map(|&domain| {
            let count = |o: Outcome| {
                entries
                    .iter()
                    .filter(|e| e.domain == domain && e.outcome == o)
                    .count()
            };
            Counts {
                domain,
                label: domain.label(),
                automatic: count(Outcome::Automatic),
                hint_only: count(Outcome::HintOnly),
                manual: count(Outcome::Manual),
                total: entries.iter().filter(|e| e.domain == domain).count(),
            }
        })
        .collect();
    files.insert(
        "index.json".to_string(),
        to_json(&Index {
            base_url: RULE_DOCS_BASE_URL.to_string(),
            counts,
            total: entries.len(),
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
