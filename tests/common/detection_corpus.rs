//! Ground-truth corpus format for detection-accuracy fixtures (#553).
//!
//! Format:
//! - `tests/fixtures/detection_corpus/<case>.html` — a self-contained,
//!   real-world pattern (may and should combine 2-3 issues the way real
//!   pages do, not one isolated element).
//! - `tests/fixtures/detection_corpus/<case>.expected.json` — ground truth
//!   for that page: `{ "case": "...", "expectations": [{ "rule_id": "...",
//!   "selector": "...", "verdict": "violation" | "pass" | "needs_review" }] }`.
//!   `verdict` mirrors the tool's own three-way outcome, so the corpus can
//!   catch both false negatives and a false "confirmed violation" on a
//!   genuinely ambiguous case.
//! - `tests/fixtures/detection_corpus/structurally_deferred.json` — a flat
//!   list of `{ "rule_id": "...", "reason": "..." }` for rules that can't be
//!   exercised in a static HTML page at all (timing/session-based rules,
//!   rules needing real audio/video playback) — documented explicitly
//!   instead of silently missing from the completeness check (#554).
//!
//! The corpus directory starts empty; population is #556's job. Loaders
//! here tolerate a missing directory/file (empty result) rather than
//! erroring, so #554's completeness check can run meaningfully before any
//! fixture exists.

use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Violation,
    Pass,
    NeedsReview,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Expectation {
    pub rule_id: String,
    #[serde(default)]
    pub selector: Option<String>,
    pub verdict: Verdict,
    /// Optional, `violation` only: the exact number of violations this rule
    /// (and selector, if given) must produce — one per offending element, so
    /// a rule reported twice for the same element shows up as a mismatch.
    #[serde(default)]
    pub occurrences: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpectedCase {
    pub case: String,
    #[serde(default)]
    pub source_file: Option<PathBuf>,
    #[serde(default)]
    pub expectations: Vec<Expectation>,
    /// Security-domain corpus only (#558): HTTP response headers the
    /// fixture server should return for this case. Empty/absent for the
    /// HTML-based WCAG corpus, which has no equivalent concept.
    #[serde(default)]
    pub response_headers: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeferredRule {
    pub rule_id: String,
    pub reason: String,
}

pub fn detection_corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("detection_corpus")
}

/// Same format, separate namespace for the non-WCAG detection corpus (#558):
/// `tests/fixtures/detection_corpus_nonwcag/<domain>/`, one subdirectory per
/// enumerable non-WCAG check family (`security`, `vulnerable_libs`,
/// `schema_rules`, ...). Kept apart from `detection_corpus_dir()` rather than
/// merged into one flat directory — `rule_id` values across domains aren't
/// from the same namespace (a security check id and a WCAG axe_id could
/// collide by coincidence) and the completeness check needs to diff each
/// domain against its own separate canonical inventory.
pub fn nonwcag_detection_corpus_dir(domain: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("detection_corpus_nonwcag")
        .join(domain)
}

/// Parse one `<case>.expected.json` file's contents.
pub fn parse_case(json: &str) -> Result<ExpectedCase, serde_json::Error> {
    serde_json::from_str(json)
}

/// Load every `*.expected.json` directly inside `dir`, tagging each with
/// the path it was loaded from. Returns an empty list if `dir` doesn't
/// exist yet — the corpus starts empty (#556 populates it incrementally).
pub fn load_corpus_dir(dir: &Path) -> Vec<ExpectedCase> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".expected.json"))
                .unwrap_or(false)
        })
        .map(|path| {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            let mut case = parse_case(&content)
                .unwrap_or_else(|e| panic!("invalid expected.json at {}: {e}", path.display()));
            case.source_file = Some(path);
            case
        })
        .collect()
}

/// Load `structurally_deferred.json` from `dir`, or an empty list if it
/// doesn't exist yet.
pub fn load_structurally_deferred(dir: &Path) -> Vec<DeferredRule> {
    let path = dir.join("structurally_deferred.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&content).unwrap_or_else(|e| {
        panic!(
            "invalid structurally_deferred.json at {}: {e}",
            path.display()
        )
    })
}
