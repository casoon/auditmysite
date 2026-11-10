//! External assessments of findings (`--annotations <file.json>`, #961).
//!
//! The file format is defined once, in barrierlab
//! (`docs/a11y/reference/annotations.md` and `annotations.schema.json`); this
//! module reads that format and does not define its own. It only decides how
//! the assessments are matched and shown.
//!
//! An annotation never changes a result. Matching reads the audited pages
//! (`findings[].id`, `findings[].patterns[].id`, presence of the screen-reader
//! section) and produces a separate [`Resolved`] value that the outputs render
//! next to, never into, the findings. Annotations that match nothing are kept
//! as [`Unmatched`] with the reason.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audit::normalized::{NormalizedFinding, NormalizedReport};

/// The one `schema` number of the annotation file this reader understands.
pub const SCHEMA: u32 = 1;

// ─── Model (the file format) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Belongs to one finding (`finding_id`).
    #[default]
    Finding,
    /// Belongs to a page's screen-reader audit section.
    ReadingFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    Agent,
    Human,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    pub kind: AuthorKind,
    pub name: String,
}

/// Deliberately neither `fail` nor `pass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assessment {
    LikelyIssue,
    LikelyFine,
    Undetermined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Screenshot,
    Text,
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// One assessment. Unknown fields in the file are ignored (serde default).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub author: Author,
    pub assessment: Assessment,
    pub reasoning: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

/// A validated annotation file.
#[derive(Debug, Clone)]
pub struct AnnotationFile {
    pub annotations: Vec<Annotation>,
    /// Non-fatal observations of the reader, e.g. a screenshot path that does
    /// not exist next to the file.
    pub warnings: Vec<String>,
}

// ─── Reader and validation ────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum AnnotationError {
    #[error("cannot read annotation file {path}: {reason}")]
    Read { path: String, reason: String },
    #[error("annotation file {path} is not valid JSON: {reason}")]
    Json { path: String, reason: String },
    #[error("annotation file {path}: {reason}")]
    Invalid { path: String, reason: String },
}

fn is_hex8(s: &str, prefix: &str) -> bool {
    s.strip_prefix(prefix).is_some_and(|rest| {
        rest.len() == 8
            && rest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

impl AnnotationFile {
    /// Read and validate a file against schema 1. Any violation of the schema
    /// is an error naming the offending annotation; nothing is skipped.
    pub fn load(path: &Path) -> Result<Self, AnnotationError> {
        let shown = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| AnnotationError::Read {
            path: shown.clone(),
            reason: e.to_string(),
        })?;
        let base = path.parent().map(Path::to_path_buf).unwrap_or_default();
        Self::parse(&text, &shown, &base)
    }

    /// Parse and validate annotation file text. `base` is the directory
    /// evidence paths are relative to.
    pub fn parse(text: &str, shown_path: &str, base: &Path) -> Result<Self, AnnotationError> {
        let invalid = |reason: String| AnnotationError::Invalid {
            path: shown_path.to_string(),
            reason,
        };
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|e| AnnotationError::Json {
                path: shown_path.to_string(),
                reason: e.to_string(),
            })?;
        let object = value.as_object().ok_or_else(|| {
            invalid("expected a JSON object with `schema` and `annotations`".into())
        })?;
        match object.get("schema").and_then(serde_json::Value::as_u64) {
            Some(n) if n == u64::from(SCHEMA) => {}
            Some(n) => {
                return Err(invalid(format!(
                    "unsupported `schema` {n}; this reader understands schema {SCHEMA}"
                )))
            }
            None => return Err(invalid("`schema` is missing or not the number 1".into())),
        }
        let items = object
            .get("annotations")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| invalid("`annotations` is missing or not an array".into()))?;

        let mut annotations = Vec::with_capacity(items.len());
        let mut warnings = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let at = |reason: String| invalid(format!("annotations[{i}]: {reason}"));
            let annotation: Annotation =
                serde_json::from_value(item.clone()).map_err(|e| at(e.to_string()))?;
            if annotation.scope == Scope::Finding && annotation.finding_id.is_none() {
                return Err(at("`finding_id` is required for scope `finding`".into()));
            }
            if let Some(id) = &annotation.finding_id {
                if !is_hex8(id, "F-") {
                    return Err(at(format!("`finding_id` {id:?} does not match F-<8 hex>")));
                }
            }
            if let Some(id) = &annotation.pattern_id {
                if !is_hex8(id, "RC-") {
                    return Err(at(format!("`pattern_id` {id:?} does not match RC-<8 hex>")));
                }
            }
            if annotation.author.name.is_empty() {
                return Err(at("`author.name` must not be empty".into()));
            }
            if annotation.reasoning.trim().is_empty() {
                return Err(at("`reasoning` must not be empty".into()));
            }
            if let Some(created) = &annotation.created {
                if chrono::NaiveDate::parse_from_str(created, "%Y-%m-%d").is_err() {
                    return Err(at(format!("`created` {created:?} is not an ISO date")));
                }
            }
            for evidence in &annotation.evidence {
                if let (EvidenceKind::Screenshot, Some(rel)) = (evidence.kind, &evidence.path) {
                    let file: PathBuf = base.join(rel);
                    if !file.is_file() {
                        warnings.push(format!(
                            "annotations[{i}]: screenshot {rel} not found next to the annotation file"
                        ));
                    }
                }
            }
            annotations.push(annotation);
        }
        Ok(Self {
            annotations,
            warnings,
        })
    }
}

// ─── Matching ─────────────────────────────────────────────────────────────────

/// What matching needs to know about one audited page.
pub struct PageTarget<'a> {
    pub url: &'a str,
    pub findings: &'a [NormalizedFinding],
    /// The page has a screen-reader audit section.
    pub has_reading_flow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmatchedReason {
    /// No audited page carries a finding with this id.
    FindingNotFound,
    /// The finding exists, but not the element pattern.
    PatternNotFound,
    /// The annotation names a page that was not audited in this run.
    UrlNotAudited,
    /// `reading_flow` without `url` in a run of several pages.
    UrlRequired,
    /// `reading_flow` for a page without a screen-reader audit.
    NoReadingFlow,
}

#[derive(Debug, Clone, Serialize)]
pub struct Matched {
    /// Position in the annotation file (not serialized).
    #[serde(skip)]
    pub index: usize,
    pub url: String,
    pub scope: Scope,
    /// `findings[].id` the annotation attached to (scope `finding`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finding_id: Option<String>,
    /// `findings[].patterns[].id`, when the annotation is narrowed to one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_id: Option<String>,
    /// Selector of that pattern, for display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_selector: Option<String>,
    pub annotation: Annotation,
}

#[derive(Debug, Clone, Serialize)]
pub struct Unmatched {
    #[serde(skip)]
    pub index: usize,
    pub reason: UnmatchedReason,
    pub annotation: Annotation,
}

/// Annotations of one run, matched against its pages.
#[derive(Debug, Clone, Default)]
pub struct Resolved {
    pub matched: Vec<Matched>,
    pub unmatched: Vec<Unmatched>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct AuthorCounts {
    pub agent: usize,
    pub human: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    /// Distinct annotations of the file that are part of this block.
    pub total: usize,
    pub matched: usize,
    pub unmatched: usize,
    pub by_author_kind: AuthorCounts,
}

/// The JSON `annotations` block.
#[derive(Debug, Clone, Serialize)]
pub struct AnnotationsBlock {
    pub schema: u32,
    pub counts: Counts,
    pub matched: Vec<Matched>,
    pub unmatched: Vec<Unmatched>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// URLs compare equal up to a trailing slash.
pub fn same_url(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

impl AnnotationFile {
    pub fn resolve(&self, pages: &[PageTarget<'_>]) -> Resolved {
        let mut out = Resolved {
            warnings: self.warnings.clone(),
            ..Resolved::default()
        };
        for (index, annotation) in self.annotations.iter().enumerate() {
            let candidates: Vec<&PageTarget<'_>> = pages
                .iter()
                .filter(|p| annotation.url.as_deref().is_none_or(|u| same_url(u, p.url)))
                .collect();
            let mut unmatched = |reason| {
                out.unmatched.push(Unmatched {
                    index,
                    reason,
                    annotation: annotation.clone(),
                })
            };
            if candidates.is_empty() {
                unmatched(UnmatchedReason::UrlNotAudited);
                continue;
            }
            match annotation.scope {
                Scope::ReadingFlow => {
                    if annotation.url.is_none() && pages.len() > 1 {
                        unmatched(UnmatchedReason::UrlRequired);
                        continue;
                    }
                    let with_flow: Vec<_> =
                        candidates.iter().filter(|p| p.has_reading_flow).collect();
                    if with_flow.is_empty() {
                        unmatched(UnmatchedReason::NoReadingFlow);
                        continue;
                    }
                    for page in with_flow {
                        out.matched.push(Matched {
                            index,
                            url: page.url.to_string(),
                            scope: Scope::ReadingFlow,
                            finding_id: None,
                            pattern_id: None,
                            pattern_selector: None,
                            annotation: annotation.clone(),
                        });
                    }
                }
                Scope::Finding => {
                    let Some(finding_id) = annotation.finding_id.as_deref() else {
                        unmatched(UnmatchedReason::FindingNotFound);
                        continue;
                    };
                    let mut found_finding = false;
                    let mut hits = Vec::new();
                    for page in &candidates {
                        let Some(finding) = page.findings.iter().find(|f| f.id == finding_id)
                        else {
                            continue;
                        };
                        found_finding = true;
                        match annotation.pattern_id.as_deref() {
                            None => hits.push((page.url, None)),
                            Some(pid) => {
                                if let Some(p) = finding.patterns.iter().find(|p| p.id == pid) {
                                    hits.push((page.url, Some(p.selector.clone())));
                                }
                            }
                        }
                    }
                    if hits.is_empty() {
                        unmatched(if found_finding {
                            UnmatchedReason::PatternNotFound
                        } else {
                            UnmatchedReason::FindingNotFound
                        });
                        continue;
                    }
                    for (url, selector) in hits {
                        out.matched.push(Matched {
                            index,
                            url: url.to_string(),
                            scope: Scope::Finding,
                            finding_id: Some(finding_id.to_string()),
                            pattern_id: annotation.pattern_id.clone(),
                            pattern_selector: selector,
                            annotation: annotation.clone(),
                        });
                    }
                }
            }
        }
        out
    }
}

impl AnnotationFile {
    /// Match against normalized pages, as the report outputs hold them.
    pub fn resolve_normalized(&self, reports: &[&NormalizedReport]) -> Resolved {
        let pages: Vec<PageTarget<'_>> = reports
            .iter()
            .map(|r| PageTarget {
                url: &r.url,
                findings: &r.findings,
                has_reading_flow: r.screen_reader.is_some(),
            })
            .collect();
        self.resolve(&pages)
    }
}

impl Resolved {
    /// What one page's report shows: its own matches, plus every annotation
    /// that matched no page of the run (so the per-page files of a batch lose
    /// nothing).
    pub fn for_url(&self, url: &str) -> Resolved {
        Resolved {
            matched: self
                .matched
                .iter()
                .filter(|m| same_url(&m.url, url))
                .cloned()
                .collect(),
            unmatched: self.unmatched.clone(),
            warnings: self.warnings.clone(),
        }
    }

    /// Matches attached to the finding `finding_id` (scope `finding`).
    pub fn for_finding<'a>(&'a self, finding_id: &'a str) -> impl Iterator<Item = &'a Matched> {
        self.matched.iter().filter(move |m| {
            m.scope == Scope::Finding && m.finding_id.as_deref() == Some(finding_id)
        })
    }

    /// Matches attached to the screen-reader section.
    pub fn reading_flow(&self) -> impl Iterator<Item = &Matched> {
        self.matched
            .iter()
            .filter(|m| m.scope == Scope::ReadingFlow)
    }

    pub fn is_empty(&self) -> bool {
        self.matched.is_empty() && self.unmatched.is_empty()
    }

    pub fn counts(&self) -> Counts {
        let matched: BTreeSet<usize> = self.matched.iter().map(|m| m.index).collect();
        let unmatched: BTreeSet<usize> = self.unmatched.iter().map(|u| u.index).collect();
        let mut by_author_kind = AuthorCounts::default();
        let mut seen = BTreeSet::new();
        let kinds = self
            .matched
            .iter()
            .map(|m| (m.index, m.annotation.author.kind))
            .chain(
                self.unmatched
                    .iter()
                    .map(|u| (u.index, u.annotation.author.kind)),
            );
        for (index, kind) in kinds {
            if seen.insert(index) {
                match kind {
                    AuthorKind::Agent => by_author_kind.agent += 1,
                    AuthorKind::Human => by_author_kind.human += 1,
                }
            }
        }
        Counts {
            total: seen.len(),
            matched: matched.len(),
            unmatched: unmatched.len(),
            by_author_kind,
        }
    }

    pub fn to_block(&self) -> AnnotationsBlock {
        AnnotationsBlock {
            schema: SCHEMA,
            counts: self.counts(),
            matched: self.matched.clone(),
            unmatched: self.unmatched.clone(),
            warnings: self.warnings.clone(),
        }
    }
}

// ─── Text (canonical English; the PDF localizes through its own registry) ─────

/// `Assessment (agent)` / `Assessment (reviewer)`, English.
pub fn author_label_en(kind: AuthorKind) -> &'static str {
    match kind {
        AuthorKind::Agent => "Assessment (agent)",
        AuthorKind::Human => "Assessment (reviewer)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<AnnotationFile, AnnotationError> {
        AnnotationFile::parse(text, "a.json", Path::new("."))
    }

    const OK: &str = r#"{"schema":1,"future":true,"annotations":[
        {"finding_id":"F-1a2b3c4d","pattern_id":"RC-5e6f7a8b","extra":1,
         "author":{"kind":"agent","name":"x 0.1"},"assessment":"likely_issue",
         "reasoning":"r","created":"2026-11-06"},
        {"scope":"reading_flow","author":{"kind":"human","name":"me"},
         "assessment":"undetermined","reasoning":"r"}]}"#;

    #[test]
    fn reads_schema_example_and_ignores_unknown_fields() {
        let file = parse(OK).unwrap();
        assert_eq!(file.annotations.len(), 2);
        assert_eq!(file.annotations[0].scope, Scope::Finding);
        assert_eq!(file.annotations[1].scope, Scope::ReadingFlow);
    }

    #[test]
    fn rejects_what_the_schema_rejects() {
        for bad in [
            "[]",
            "not json",
            r#"{"schema":2,"annotations":[]}"#,
            r#"{"schema":1}"#,
            // finding scope without finding_id
            r#"{"schema":1,"annotations":[{"author":{"kind":"agent","name":"a"},"assessment":"likely_fine","reasoning":"r"}]}"#,
            // bad id shape
            r#"{"schema":1,"annotations":[{"finding_id":"F-xyz","author":{"kind":"agent","name":"a"},"assessment":"likely_fine","reasoning":"r"}]}"#,
            // fail/pass are not assessments
            r#"{"schema":1,"annotations":[{"finding_id":"F-1a2b3c4d","author":{"kind":"agent","name":"a"},"assessment":"fail","reasoning":"r"}]}"#,
            // empty reasoning
            r#"{"schema":1,"annotations":[{"finding_id":"F-1a2b3c4d","author":{"kind":"agent","name":"a"},"assessment":"likely_fine","reasoning":""}]}"#,
            // unknown author kind
            r#"{"schema":1,"annotations":[{"finding_id":"F-1a2b3c4d","author":{"kind":"bot","name":"a"},"assessment":"likely_fine","reasoning":"r"}]}"#,
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn error_names_the_annotation() {
        let err = parse(
            r#"{"schema":1,"annotations":[{"scope":"reading_flow","author":{"kind":"human","name":"a"},"assessment":"likely_fine","reasoning":"r"},
            {"finding_id":"nope","author":{"kind":"human","name":"a"},"assessment":"likely_fine","reasoning":"r"}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("annotations[1]"), "{err}");
    }

    #[test]
    fn missing_screenshot_is_a_warning_not_an_error() {
        let file = parse(
            r#"{"schema":1,"annotations":[{"finding_id":"F-1a2b3c4d","author":{"kind":"human","name":"a"},"assessment":"likely_fine","reasoning":"r","evidence":[{"kind":"screenshot","path":"does/not/exist.png"}]}]}"#,
        )
        .unwrap();
        assert_eq!(file.warnings.len(), 1);
    }

    #[test]
    fn unmatched_without_pages() {
        let file = parse(OK).unwrap();
        let resolved = file.resolve(&[]);
        assert_eq!(resolved.matched.len(), 0);
        assert_eq!(resolved.unmatched.len(), 2);
        let counts = resolved.counts();
        assert_eq!((counts.total, counts.unmatched), (2, 2));
        assert_eq!(
            (counts.by_author_kind.agent, counts.by_author_kind.human),
            (1, 1)
        );
    }

    #[test]
    fn same_url_ignores_trailing_slash() {
        assert!(same_url("https://a.test/", "https://a.test"));
        assert!(!same_url("https://a.test/x", "https://a.test/y"));
    }
}
