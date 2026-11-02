//! Harness pieces for running the W3C ACT test cases against auditmysite
//! (#837). Browser-free: parsing, serving, outcome derivation and the report
//! are here so they can be tested without Chrome (`tests/act_testcases.rs`).
//!
//! Source of the test cases: the rule files `_rules/*.md` of
//! `act-rules/act-rules.github.io` at the commit pinned in
//! `docs/act-mapping.json` (`act_source`). The published `testcases.json` is
//! not used: it is a build artefact that does not follow the repository
//! (the published copy predates rules such as `kb1m8s`). Like the official
//! generator, every example's first code block becomes one document; HTML
//! snippets get a `<!DOCTYPE html>` prefix, other languages (svg, xml, xhtml,
//! js) are served as they are with the matching content type. The expected
//! outcome is the heading of the example (`Passed`, `Failed`, `Inapplicable`),
//! never a result of a run.
//!
//! Output (`results.json`, schema in `docs/ACT_COVERAGE.md`) is the input for
//! the EARL writer (#838).

use crate::common::corpus_diff::{FindingClass, Observation, ObservedFinding, RunState};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// What the ACT rule says about a test case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expected {
    Passed,
    Failed,
    Inapplicable,
}

impl Expected {
    pub fn as_str(self) -> &'static str {
        match self {
            Expected::Passed => "passed",
            Expected::Failed => "failed",
            Expected::Inapplicable => "inapplicable",
        }
    }
}

/// One ACT test case, ready to be served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Testcase {
    pub act_id: String,
    /// `passed-example-1`; unique within the rule.
    pub slug: String,
    /// `Passed Example 1`, as in the rule file.
    pub title: String,
    pub expected: Expected,
    /// Code-block language: `html`, `svg`, `xml`, `xhtml`, `js`.
    pub language: String,
    /// The document as served.
    pub body: String,
    /// Rule file relative to the cache root, e.g. `_rules/x-97a4e1.md`.
    pub source_file: String,
}

impl Testcase {
    /// Stable id of the test case in our results: `<act_id>/<slug>`.
    pub fn id(&self) -> String {
        format!("{}/{}", self.act_id, self.slug)
    }

    /// Path the document is served at.
    pub fn path(&self) -> String {
        format!("/testcases/{}/{}.{}", self.act_id, self.slug, self.language)
    }

    pub fn content_type(&self) -> &'static str {
        match self.language.as_str() {
            "svg" => "image/svg+xml",
            "xml" => "application/xml",
            "xhtml" => "application/xhtml+xml",
            "js" => "text/javascript",
            _ => "text/html; charset=utf-8",
        }
    }
}

/// An example being read: expectation, title, code blocks `(language, code)`.
type Pending = (Expected, String, Vec<(String, String)>);

fn frontmatter_id(md: &str) -> Option<String> {
    let mut lines = md.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            return None;
        }
        if let Some(id) = line.strip_prefix("id:") {
            return Some(id.trim().to_string());
        }
    }
    None
}

fn example_heading(line: &str) -> Option<(Expected, String)> {
    let title = line.strip_prefix("#### ")?.trim();
    let (kind, rest) = title.split_once(' ')?;
    let expected = match kind {
        "Passed" => Expected::Passed,
        "Failed" => Expected::Failed,
        "Inapplicable" => Expected::Inapplicable,
        _ => return None,
    };
    let n = rest.strip_prefix("Example ")?;
    n.parse::<u32>().ok()?;
    Some((expected, title.to_string()))
}

/// Test cases of one rule file. Panics when an example does not have exactly
/// one code block: the parser must not silently drop a test case.
pub fn parse_rule_file(md: &str, source_file: &str) -> Vec<Testcase> {
    let Some(act_id) = frontmatter_id(md) else {
        return Vec::new();
    };
    let mut cases = Vec::new();
    let mut in_examples = false;
    let mut current: Option<Pending> = None;
    let mut fence: Option<(String, Vec<&str>)> = None;

    let finish = |current: &mut Option<Pending>, cases: &mut Vec<Testcase>| {
        if let Some((expected, title, mut blocks)) = current.take() {
            assert_eq!(
                blocks.len(),
                1,
                "{source_file}: `{title}` must have exactly one code block, found {}",
                blocks.len()
            );
            let (language, code) = blocks.remove(0);
            let body = if language == "html" {
                format!("<!DOCTYPE html>\n{code}")
            } else {
                code
            };
            cases.push(Testcase {
                act_id: act_id.clone(),
                slug: title.to_lowercase().replace(' ', "-"),
                title,
                expected,
                language,
                body,
                source_file: source_file.to_string(),
            });
        }
    };

    for line in md.lines() {
        if let Some((lang, code)) = fence.as_mut() {
            if line.starts_with("```") {
                let block = (lang.clone(), code.join("\n"));
                fence = None;
                if let Some(cur) = current.as_mut() {
                    cur.2.push(block);
                }
            } else {
                code.push(line);
            }
            continue;
        }
        if line.starts_with("## ") {
            finish(&mut current, &mut cases);
            in_examples = line.trim() == "## Examples";
            continue;
        }
        if !in_examples {
            continue;
        }
        if let Some((expected, title)) = example_heading(line) {
            finish(&mut current, &mut cases);
            current = Some((expected, title, Vec::new()));
        } else if line.starts_with("### ") {
            finish(&mut current, &mut cases);
        } else if let Some(lang) = line.strip_prefix("```") {
            if current.is_some() {
                fence = Some((lang.trim().to_string(), Vec::new()));
            }
        }
    }
    finish(&mut current, &mut cases);
    cases
}

/// Root of the verified ACT cache. Runs `scripts/act-fetch.sh`, which
/// downloads the pinned commit and checks the content digest when the cache
/// is missing.
pub fn ensure_cache() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = std::process::Command::new("bash")
        .arg(root.join("scripts/act-fetch.sh"))
        .output()
        .expect("cannot run scripts/act-fetch.sh");
    assert!(
        out.status.success(),
        "scripts/act-fetch.sh failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    PathBuf::from(String::from_utf8_lossy(&out.stdout).trim())
}

/// All test cases of the given ACT rule ids found under `cache/_rules`.
pub fn load_testcases(cache: &Path, act_ids: &[String]) -> Vec<Testcase> {
    let dir = cache.join("_rules");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    files.sort();
    let mut cases = Vec::new();
    for path in files {
        let md = std::fs::read_to_string(&path).expect("rule file");
        let rel = format!(
            "_rules/{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("")
        );
        let parsed = parse_rule_file(&md, &rel);
        if parsed.first().is_some_and(|c| act_ids.contains(&c.act_id)) {
            cases.extend(parsed);
        }
    }
    cases
}

// --- serving -------------------------------------------------------------

/// Serves the test cases at their paths and `/test-assets/*` from the cache,
/// on one origin, the way the ACT site does.
pub struct ActServer {
    pub base: String,
}

fn asset_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "text/javascript",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "vtt" => "text/vtt",
        "pdf" => "application/pdf",
        "json" => "application/json",
        "xml" => "application/xml",
        "woff" | "woff2" => "font/woff2",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

impl ActServer {
    pub fn start(cache: &Path, cases: &[Testcase]) -> ActServer {
        let docs: Arc<HashMap<String, (&'static str, Vec<u8>)>> = Arc::new(
            cases
                .iter()
                .map(|c| (c.path(), (c.content_type(), c.body.clone().into_bytes())))
                .collect(),
        );
        let assets = cache.join("test-assets");
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let docs = docs.clone();
                let assets = assets.clone();
                thread::spawn(move || serve_one(stream, &docs, &assets));
            }
        });
        ActServer { base }
    }
}

fn serve_one(
    mut stream: std::net::TcpStream,
    docs: &HashMap<String, (&'static str, Vec<u8>)>,
    assets: &Path,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let mut head = Vec::new();
    let mut buf = [0u8; 2048];
    while !head.windows(4).any(|w| w == b"\r\n\r\n") && head.len() < 16_384 {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => head.extend_from_slice(&buf[..n]),
        }
    }
    // Chrome opens speculative (preconnect) sockets and sends nothing on
    // them. Answering such a socket with a 404 lets Chrome reuse it for the
    // next navigation and read that 404 as the page.
    if head.is_empty() {
        return;
    }
    let text = String::from_utf8_lossy(&head);
    let mut first = text.lines().next().unwrap_or("").split_whitespace();
    let method = first.next().unwrap_or("");
    let target = first.next().unwrap_or("/");
    let path = target.split(['?', '#']).next().unwrap_or("/");

    let found: Option<(&str, Vec<u8>)> = if let Some((ct, body)) = docs.get(path) {
        Some((ct, body.clone()))
    } else if let Some(rel) = path.strip_prefix("/test-assets/") {
        let safe = !rel.split('/').any(|s| s == ".." || s.is_empty());
        let file = assets.join(rel);
        if safe && file.is_file() {
            std::fs::read(&file)
                .ok()
                .map(|bytes| (asset_content_type(&file), bytes))
        } else {
            None
        }
    } else {
        None
    };
    let (status, ct, body) = match found {
        Some((ct, body)) => ("200 OK", ct, body),
        None => ("404 Not Found", "text/plain", b"not found".to_vec()),
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    if method != "HEAD" {
        let _ = stream.write_all(&body);
    }
}

// --- outcome derivation --------------------------------------------------

/// What the run says about one test case for one ACT rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// At least one confirmed violation of a mapped internal rule.
    Failed,
    /// No violation, but a review note of a mapped internal rule.
    CantTell,
    /// All mapped rules ran and found nothing. A rule that ran does not say
    /// whether it found applicable content, so `passed` and `inapplicable`
    /// are not told apart here.
    PassedOrInapplicable,
    /// Every mapped rule recorded `NotRun(NotApplicable)`: the tool itself
    /// says there was nothing to check.
    Inapplicable,
    /// A mapped internal rule did not run and nothing failed.
    NotRun,
    /// The audit of the document itself failed (error or timeout).
    AuditError,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Failed => "failed",
            Outcome::CantTell => "cantTell",
            Outcome::PassedOrInapplicable => "passedOrInapplicable",
            Outcome::Inapplicable => "inapplicable",
            Outcome::NotRun => "notRun",
            Outcome::AuditError => "auditError",
        }
    }
}

pub fn derive_outcome(internal_rules: &[String], obs: &Observation) -> Outcome {
    let mine = |f: &&ObservedFinding| internal_rules.contains(&f.rule_id);
    if obs
        .findings
        .iter()
        .filter(mine)
        .any(|f| f.class == FindingClass::Fail)
    {
        return Outcome::Failed;
    }
    if obs
        .findings
        .iter()
        .any(|f| internal_rules.contains(&f.rule_id))
    {
        return Outcome::CantTell;
    }
    let states: Vec<&RunState> = obs
        .runs
        .iter()
        .filter(|r| internal_rules.contains(&r.rule_id))
        .map(|r| &r.state)
        .collect();
    let not_applicable =
        |s: &&RunState| matches!(s, RunState::NotRun(why) if why == "NotApplicable");
    if states
        .iter()
        .any(|s| matches!(s, RunState::NotRun(_)) && !not_applicable(s))
    {
        Outcome::NotRun
    } else if !states.is_empty() && states.iter().all(not_applicable) {
        Outcome::Inapplicable
    } else {
        Outcome::PassedOrInapplicable
    }
}

/// Comparison of the run against the expectation of the test case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Ok,
    /// Failed on a test case the ACT rule marks passed or inapplicable.
    FalsePositive,
    /// Missed a failed test case.
    FalseNegative,
    /// Only a review note where the test case fails (failed) or where it
    /// passes (cantTell on passed/inapplicable). Not a false positive.
    CantTell,
    /// The tool says inapplicable where the test case is a passed example.
    WrongApplicability,
    NotRun,
    /// The audit of the document failed before any rule could judge it.
    AuditError,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Ok => "ok",
            Verdict::FalsePositive => "false_positive",
            Verdict::FalseNegative => "false_negative",
            Verdict::CantTell => "cant_tell",
            Verdict::WrongApplicability => "wrong_applicability",
            Verdict::NotRun => "not_run",
            Verdict::AuditError => "audit_error",
        }
    }
}

pub fn judge(expected: Expected, outcome: Outcome) -> Verdict {
    match (expected, outcome) {
        (_, Outcome::AuditError) => Verdict::AuditError,
        (_, Outcome::NotRun) => Verdict::NotRun,
        (Expected::Failed, Outcome::Failed) => Verdict::Ok,
        (Expected::Failed, Outcome::CantTell) => Verdict::CantTell,
        (Expected::Failed, Outcome::PassedOrInapplicable | Outcome::Inapplicable) => {
            Verdict::FalseNegative
        }
        (Expected::Passed, Outcome::Inapplicable) => Verdict::WrongApplicability,
        (Expected::Inapplicable, Outcome::Inapplicable) => Verdict::Ok,
        (_, Outcome::Failed) => Verdict::FalsePositive,
        (_, Outcome::CantTell) => Verdict::CantTell,
        (_, Outcome::PassedOrInapplicable) => Verdict::Ok,
    }
}

/// `consistent`: every test case agrees. `partially_consistent`: no false
/// positive, but cases missed or undecided. `inconsistent`: at least one
/// false positive.
pub fn consistency(verdicts: &[Verdict]) -> &'static str {
    if verdicts.contains(&Verdict::FalsePositive) {
        "inconsistent"
    } else if verdicts.iter().all(|v| *v == Verdict::Ok) {
        "consistent"
    } else {
        "partially_consistent"
    }
}

// --- report --------------------------------------------------------------

/// One executed test case with its judged result.
pub struct Executed {
    pub case: Testcase,
    pub outcome: Outcome,
    pub verdict: Verdict,
    /// Findings of the mapped internal rules: `(rule_id, "fail"|"review", selector)`.
    pub findings: Vec<(String, &'static str, Option<String>)>,
    /// Why the audit failed, for `Outcome::AuditError`.
    pub error: Option<String>,
    /// Execution records of the mapped internal rules, e.g. `document/lang-missing: Ran(1)`.
    pub runs: Vec<String>,
}

/// One mapped ACT rule as selected from `docs/act-mapping.json`.
pub struct MappedRule {
    pub act_id: String,
    pub name: String,
    pub internal_rules: Vec<String>,
}

pub fn results_json(
    commit: &str,
    selection: &str,
    level: &str,
    rules: &[MappedRule],
    executed: &[Executed],
) -> Value {
    let mut by_rule: BTreeMap<&str, Vec<&Executed>> = BTreeMap::new();
    for e in executed {
        by_rule.entry(e.case.act_id.as_str()).or_default().push(e);
    }
    let count = |v: Verdict| executed.iter().filter(|e| e.verdict == v).count();
    let rule_values: Vec<Value> = rules
        .iter()
        .map(|r| {
            let cases = by_rule.get(r.act_id.as_str()).cloned().unwrap_or_default();
            let verdicts: Vec<Verdict> = cases.iter().map(|e| e.verdict).collect();
            let n = |v: Verdict| verdicts.iter().filter(|x| **x == v).count();
            let exp = |x: Expected| cases.iter().filter(|e| e.case.expected == x).count();
            json!({
                "act_id": r.act_id,
                "name": r.name,
                "internal_rules": r.internal_rules,
                "consistency": consistency(&verdicts),
                "counts": {
                    "testcases": cases.len(),
                    "expected_passed": exp(Expected::Passed),
                    "expected_failed": exp(Expected::Failed),
                    "expected_inapplicable": exp(Expected::Inapplicable),
                    "ok": n(Verdict::Ok),
                    "false_positive": n(Verdict::FalsePositive),
                    "false_negative": n(Verdict::FalseNegative),
                    "cant_tell": n(Verdict::CantTell),
                    "wrong_applicability": n(Verdict::WrongApplicability),
                    "not_run": n(Verdict::NotRun),
                    "audit_error": n(Verdict::AuditError),
                },
                "testcases": cases.iter().map(|e| json!({
                    "testcase_id": e.case.id(),
                    "title": e.case.title,
                    "expected": e.case.expected.as_str(),
                    "outcome": e.outcome.as_str(),
                    "verdict": e.verdict.as_str(),
                    "source_file": e.case.source_file,
                    "language": e.case.language,
                    "error": e.error,
                    "runs": e.runs,
                    "findings": e.findings.iter().map(|(rule, class, selector)| json!({
                        "rule_id": rule, "class": class, "selector": selector,
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "schema_version": 1,
        "act_source": {
            "repository": "https://github.com/act-rules/act-rules.github.io",
            "commit": commit,
        },
        "selection": selection,
        "wcag_level": level,
        "outcome_note": "`inapplicable`: every mapped rule recorded NotRun(NotApplicable). `passedOrInapplicable`: the mapped rules ran and reported nothing; a rule that ran does not report whether it found applicable content, so passed and inapplicable are not told apart.",
        "summary": {
            "rules": rules.len(),
            "testcases": executed.len(),
            "ok": count(Verdict::Ok),
            "false_positive": count(Verdict::FalsePositive),
            "false_negative": count(Verdict::FalseNegative),
            "cant_tell": count(Verdict::CantTell),
            "wrong_applicability": count(Verdict::WrongApplicability),
            "not_run": count(Verdict::NotRun),
            "audit_error": count(Verdict::AuditError),
        },
        "rules": rule_values,
    })
}

/// Human-readable summary of `results_json`.
pub fn render_markdown(results: &Value) -> String {
    let mut out = String::from("# ACT test case run\n\n");
    out.push_str(&format!(
        "Commit `{}`, selection `{}`, WCAG level {}.\n\n",
        results["act_source"]["commit"].as_str().unwrap_or(""),
        results["selection"].as_str().unwrap_or(""),
        results["wcag_level"].as_str().unwrap_or("")
    ));
    out.push_str(
        "| ACT | Rule | Consistency | Test cases | ok | FP | FN | cantTell | notRun | error |\n",
    );
    out.push_str("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for r in results["rules"].as_array().into_iter().flatten() {
        let c = &r["counts"];
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            r["act_id"].as_str().unwrap_or(""),
            r["name"].as_str().unwrap_or(""),
            r["consistency"].as_str().unwrap_or(""),
            c["testcases"],
            c["ok"],
            c["false_positive"],
            c["false_negative"],
            c["cant_tell"],
            c["not_run"],
            c["audit_error"],
        ));
    }
    out.push_str("\n## Deviations\n\n");
    let mut any = false;
    for r in results["rules"].as_array().into_iter().flatten() {
        for t in r["testcases"].as_array().into_iter().flatten() {
            if t["verdict"] != "ok" {
                any = true;
                out.push_str(&format!(
                    "- `{}` {} (expected {}, got {}): {}\n",
                    t["testcase_id"].as_str().unwrap_or(""),
                    t["title"].as_str().unwrap_or(""),
                    t["expected"].as_str().unwrap_or(""),
                    t["outcome"].as_str().unwrap_or(""),
                    t["verdict"].as_str().unwrap_or(""),
                ));
                if let Some(err) = t["error"].as_str() {
                    out.push_str(&format!("  - {err}\n"));
                }
            }
        }
    }
    if !any {
        out.push_str("None.\n");
    }
    out
}

// --- EARL ----------------------------------------------------------------

/// EARL outcome for one derived outcome, or `None` when the run makes no
/// statement. `passedOrInapplicable` never becomes `earl:passed` or
/// `earl:inapplicable`: a rule that ran does not say whether it found
/// applicable content, and the expectation of the example must not leak into
/// the result.
fn earl_outcome(outcome: &str) -> Option<&'static str> {
    match outcome {
        "failed" => Some("earl:failed"),
        "cantTell" => Some("earl:cantTell"),
        "inapplicable" => Some("earl:inapplicable"),
        "notRun" | "auditError" => Some("earl:untested"),
        _ => None,
    }
}

/// EARL JSON-LD report (https://act-rules.github.io/pages/implementations/earl-reports/)
/// from `results_json` and `docs/act-mapping.json`. One `TestSubject` per
/// test case; `source` points at the example in the rule file at the pinned
/// commit (the published hash URLs of `testcases.json` are not reproduced).
/// `test.title` is the internal rule id(s) that produced the result, `isPartOf`
/// the success criteria of the mapping (`WCAG 2: <number>` form).
pub fn earl_json(results: &Value, mapping: &Value) -> Value {
    let repo = results["act_source"]["repository"].as_str().unwrap_or("");
    let commit = results["act_source"]["commit"].as_str().unwrap_or("");
    let mut graph = Vec::new();
    for rule in results["rules"].as_array().into_iter().flatten() {
        let act_id = rule["act_id"].as_str().unwrap_or("");
        let title = rule["internal_rules"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join("+");
        let is_part_of: Vec<Value> = mapping["rules"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|m| m["act_id"] == act_id)
            .and_then(|m| m["wcag_sc"].as_array())
            .into_iter()
            .flatten()
            .filter_map(|sc| sc.as_str())
            .map(|sc| json!({ "title": format!("WCAG 2: {sc}") }))
            .collect();
        for case in rule["testcases"].as_array().into_iter().flatten() {
            let assertions: Vec<Value> = earl_outcome(case["outcome"].as_str().unwrap_or(""))
                .map(|outcome| {
                    json!({
                        "@type": "Assertion",
                        "result": { "outcome": outcome },
                        "test": { "title": title, "isPartOf": is_part_of },
                    })
                })
                .into_iter()
                .collect();
            graph.push(json!({
                "@type": "TestSubject",
                "source": format!(
                    "{repo}/blob/{commit}/{}#{}",
                    case["source_file"].as_str().unwrap_or(""),
                    case["testcase_id"].as_str().unwrap_or("").rsplit('/').next().unwrap_or("")
                ),
                "assertions": assertions,
            }));
        }
    }
    json!({
        "@context": "https://act-rules.github.io/earl-context.json",
        "@graph": graph,
    })
}
