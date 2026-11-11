//! Findings baseline for CI gating (#930): `--write-baseline` records the
//! finding ids of a run, `--baseline` makes the findings of a later run whose
//! id is in the file `known`, so only new findings reach the exit code.
//!
//! A baseline is a gating filter, not a suppression: scores and the finding
//! list are untouched. The classification is data of the report
//! ([`RunContext::baseline`](crate::audit::RunContext)), so `normalize`, the
//! stored decision, `validate` and every output derive the same result, and
//! the verdict profile carries the file's digest.
//!
//! Identity: a finding id is `F-` plus a hash of the rule id alone
//! (`a11y_report::FindingId`), so a baseline entry means "this rule is
//! reported on this page" — see "Finding id stability" in
//! `docs/OUTPUT_CONTRACT.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::audit::normalized::{NormalizedFinding, NormalizedReport};
use crate::audit::report::{AuditReport, BatchReport};
use crate::error::{AuditError, Result};

/// Contract of the baseline file.
pub const BASELINE_CONTRACT: &str = "auditmysite.baseline/1";

/// Version of the finding id derivation a baseline relies on. Bump it (and
/// say so in the changelog) when ids of unchanged findings change, e.g. when
/// `a11y_report::FindingId` changes its key; old baseline files are then
/// refused instead of reading as "everything is new".
pub const FINDING_ID_SCHEME: u32 = 1;

/// The id of the finding of `rule_id`: the one derivation both the
/// normalization and the baseline file use.
pub fn finding_id_of(rule_id: &str) -> String {
    a11y_report::FindingId::new(rule_id, std::iter::empty()).to_string()
}

/// One recorded finding: its id and the rule it was derived from. The rule
/// makes the file reviewable in a diff and lets the reader verify the id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BaselineFinding {
    pub id: String,
    pub rule: String,
}

/// The findings recorded for one page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselinePage {
    pub url: String,
    pub findings: Vec<BaselineFinding>,
}

/// The baseline file (`--write-baseline`, `--baseline`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineFile {
    pub contract: String,
    pub id_scheme: u32,
    pub tool_version: String,
    pub created_at: DateTime<Utc>,
    pub pages: Vec<BaselinePage>,
}

/// A loaded baseline: the file plus its digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Baseline {
    file: BaselineFile,
    digest: String,
}

/// What the run knows of the baseline for one page; stored in the page's
/// `run_context`, so a stored report reproduces its classification.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PageBaseline {
    /// Whether the baseline lists this URL at all. A page it does not list
    /// has no known findings.
    pub listed: bool,
    pub findings: Vec<BaselineFinding>,
}

/// The inputs of a page's verdict that are left after the known findings are
/// removed. The score thresholds are not baselined: the score is unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BaselineGate {
    pub legal_flags: usize,
    pub blocking_issues: usize,
    pub keyboard_blocking_issues: usize,
    pub technical_blockers: usize,
    /// WCAG findings that are not known.
    pub open_findings: usize,
}

/// How one page relates to the baseline (`baseline` of the page, JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BaselineStatus {
    /// Digest of the baseline file (also in `decision.profile.baseline_digest`).
    pub digest: String,
    /// Whether the baseline lists this URL.
    pub page_listed: bool,
    /// Findings of this run whose id is in the baseline.
    pub known: usize,
    /// Findings of this run whose id is not.
    pub new: usize,
    /// Baseline entries of this page without a finding in this run — fixed,
    /// or from a module that did not run. Never dropped silently.
    pub fixed: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed_ids: Vec<String>,
    pub gate: BaselineGate,
}

/// The batch inputs of the verdict left after the known findings are removed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BatchBaselineGate {
    pub legal_flags: usize,
    pub blocking_issues: usize,
    pub technical_blockers: usize,
    /// WCAG occurrences of findings that are not known.
    pub open_violations: usize,
}

/// The baseline in a batch summary (`summary.baseline`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BatchBaselineSummary {
    pub digest: String,
    pub known: usize,
    pub new: usize,
    pub fixed: usize,
    /// Pages the baseline lists that are not part of this run.
    pub unmatched_pages: usize,
    pub gate: BatchBaselineGate,
}

/// The compact line the table and the PDF show for a baseline-gated run.
/// The only text source: `en` selects the language, the JSON stays English.
pub fn summary_text(
    known: usize,
    new: usize,
    fixed: usize,
    unmatched_pages: Option<usize>,
    en: bool,
) -> String {
    let unmatched = unmatched_pages.filter(|n| *n > 0);
    if en {
        let mut text = format!(
            "Baseline: {known} known, {new} new, {fixed} fixed findings; known findings do not affect the verdict."
        );
        if let Some(n) = unmatched {
            text.push_str(&format!(" {n} baseline pages were not part of this run."));
        }
        text
    } else {
        let mut text = format!(
            "Baseline: {known} bekannte, {new} neue, {fixed} behobene Befunde; bekannte Befunde beeinflussen das Urteil nicht."
        );
        if let Some(n) = unmatched {
            text.push_str(&format!(
                " {n} Baseline-Seiten waren nicht Teil dieses Laufs."
            ));
        }
        text
    }
}

fn bad(message: impl std::fmt::Display) -> AuditError {
    AuditError::ConfigError(format!("baseline: {message}"))
}

impl Baseline {
    /// Records the findings of `reports`, one page per URL.
    pub fn from_reports<'a>(
        reports: impl IntoIterator<Item = &'a NormalizedReport>,
        created_at: DateTime<Utc>,
    ) -> Self {
        let mut pages: BTreeMap<String, BTreeSet<BaselineFinding>> = BTreeMap::new();
        for report in reports {
            pages
                .entry(report.url.clone())
                .or_default()
                .extend(report.findings.iter().map(|f| BaselineFinding {
                    id: f.id.clone(),
                    rule: f.rule_id.clone(),
                }));
        }
        Self::from_file(BaselineFile {
            contract: BASELINE_CONTRACT.to_string(),
            id_scheme: FINDING_ID_SCHEME,
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at,
            pages: pages
                .into_iter()
                .map(|(url, findings)| BaselinePage {
                    url,
                    findings: findings.into_iter().collect(),
                })
                .collect(),
        })
    }

    fn from_file(file: BaselineFile) -> Self {
        // Over the recorded identity only, in a fixed order: metadata such as
        // the creation time never changes the digest.
        let mut lines: Vec<String> = file
            .pages
            .iter()
            .flat_map(|p| p.findings.iter().map(|f| format!("{}\t{}", p.url, f.id)))
            .collect();
        lines.sort();
        let mut hasher = Sha256::new();
        hasher.update(format!("{BASELINE_CONTRACT}\n{}\n", file.id_scheme));
        for line in &lines {
            hasher.update(line);
            hasher.update("\n");
        }
        let digest = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Self { file, digest }
    }

    /// Parses and checks a baseline file. A file that is not one, or whose
    /// ids this tool would not derive, is an error, never an empty baseline.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let file: BaselineFile = serde_json::from_slice(bytes).map_err(|e| {
            bad(format!(
                "not a baseline file (expected contract, id_scheme, pages[]): line {}, column {}",
                e.line(),
                e.column()
            ))
        })?;
        if file.contract != BASELINE_CONTRACT {
            return Err(bad(format!(
                "contract {} is not {BASELINE_CONTRACT}",
                file.contract
            )));
        }
        if file.id_scheme != FINDING_ID_SCHEME {
            return Err(bad(format!(
                "written with finding id scheme {}, this tool uses {FINDING_ID_SCHEME}; \
                 write a new baseline with --write-baseline",
                file.id_scheme
            )));
        }
        let mut urls = BTreeSet::new();
        for page in &file.pages {
            if !urls.insert(page.url.as_str()) {
                return Err(bad(format!("page {} is listed twice", page.url)));
            }
            for finding in &page.findings {
                // The id must be what this tool derives from the rule: a
                // hand-edited id or a changed derivation is not a match.
                if finding.id != finding_id_of(&finding.rule) {
                    return Err(bad(format!(
                        "finding id {} does not belong to rule {} under finding id scheme \
                         {FINDING_ID_SCHEME}; write a new baseline with --write-baseline",
                        finding.id, finding.rule
                    )));
                }
            }
        }
        Ok(Self::from_file(file))
    }

    pub fn read(path: &Path) -> Result<Self> {
        let bytes =
            std::fs::read(path).map_err(|e| bad(format!("cannot read {}: {e}", path.display())))?;
        Self::parse(&bytes)
    }

    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(&self.file)
            .expect("a baseline file is plain data and always serializes");
        json.push('\n');
        json
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.to_json())
            .map_err(|e| bad(format!("cannot write {}: {e}", path.display())))
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn file(&self) -> &BaselineFile {
        &self.file
    }

    pub fn finding_count(&self) -> usize {
        self.file.pages.iter().map(|p| p.findings.len()).sum()
    }

    /// What the baseline says about `url`.
    pub fn page(&self, url: &str) -> PageBaseline {
        match self.file.pages.iter().find(|p| p.url == url) {
            Some(page) => PageBaseline {
                listed: true,
                findings: page.findings.clone(),
            },
            None => PageBaseline::default(),
        }
    }

    /// Gates `report` against this baseline: its normalization marks the
    /// known findings and decides under a profile that names the digest.
    pub fn apply(&self, report: &mut AuditReport) {
        report.run_context.baseline = Some(self.page(&report.url));
        report
            .run_context
            .decision_profile
            .set_baseline_digest(&self.digest);
    }
}

/// Marks the known findings of a page and computes the status and the
/// verdict inputs without them. `gate` computes the risk of the open
/// findings; it is the one place the risk derivation is reused.
pub(crate) fn classify(
    page: &PageBaseline,
    digest: &str,
    findings: &mut [NormalizedFinding],
    gate: impl FnOnce(&[NormalizedFinding]) -> (usize, usize, usize),
) -> BaselineStatus {
    let recorded: BTreeSet<&str> = page.findings.iter().map(|f| f.id.as_str()).collect();
    for finding in findings.iter_mut() {
        finding.known = recorded.contains(finding.id.as_str());
    }
    let present: BTreeSet<&str> = findings.iter().map(|f| f.id.as_str()).collect();
    let fixed_ids: Vec<String> = page
        .findings
        .iter()
        .filter(|f| !present.contains(f.id.as_str()))
        .map(|f| f.id.clone())
        .collect();
    let known = findings.iter().filter(|f| f.known).count();
    let open: Vec<NormalizedFinding> = findings.iter().filter(|f| !f.known).cloned().collect();
    let (legal_flags, blocking_issues, keyboard_blocking_issues) = gate(&open);
    BaselineStatus {
        digest: digest.to_string(),
        page_listed: page.listed,
        known,
        new: findings.len() - known,
        fixed: fixed_ids.len(),
        fixed_ids,
        gate: BaselineGate {
            legal_flags,
            blocking_issues,
            keyboard_blocking_issues,
            technical_blockers: open
                .iter()
                .filter(|f| {
                    f.contract.scoring.effect == crate::audit::finding_contract::ScoringEffect::Gate
                })
                .count(),
            open_findings: open.iter().filter(|f| f.category == "wcag").count(),
        },
    }
}

/// The batch summary of the baseline, from the pages' statuses. `None` when
/// the run was not gated.
pub(crate) fn summarize_batch(reports: &[NormalizedReport]) -> Option<BatchBaselineSummary> {
    let statuses: Vec<(&NormalizedReport, &BaselineStatus)> = reports
        .iter()
        .filter_map(|r| r.baseline.as_ref().map(|b| (r, b)))
        .collect();
    let digest = statuses.first()?.1.digest.clone();

    // Distinct rules, as the batch's own `legal_flags` counts them; the
    // screen-reader signal has no rule of its own and stays one member.
    let mut legal_rules: BTreeSet<&str> = statuses
        .iter()
        .flat_map(|(r, _)| r.findings.iter())
        .filter(|f| !f.known && f.is_legal_flag())
        .map(|f| f.rule_id.as_str())
        .collect();
    let sr_only = statuses.iter().any(|(r, b)| {
        b.gate.legal_flags
            > r.findings
                .iter()
                .filter(|f| !f.known && f.is_legal_flag())
                .count()
    });
    if sr_only {
        legal_rules.insert("screen_reader.bfsg_noncompliant");
    }

    Some(BatchBaselineSummary {
        digest,
        known: statuses.iter().map(|(_, b)| b.known).sum(),
        new: statuses.iter().map(|(_, b)| b.new).sum(),
        fixed: statuses.iter().map(|(_, b)| b.fixed).sum(),
        unmatched_pages: 0,
        gate: BatchBaselineGate {
            legal_flags: legal_rules.len(),
            blocking_issues: statuses.iter().map(|(_, b)| b.gate.blocking_issues).sum(),
            technical_blockers: statuses
                .iter()
                .map(|(_, b)| b.gate.technical_blockers)
                .sum(),
            open_violations: statuses
                .iter()
                .flat_map(|(r, _)| r.findings.iter())
                .filter(|f| !f.known && f.category == "wcag")
                .map(|f| f.occurrence_count)
                .sum(),
        },
    })
}

impl BatchReport {
    /// Gates every page against `baseline`. The batch is built again from the
    /// gated pages, so the pages' decisions, the counts and the batch decision
    /// all follow from the same classification.
    pub fn with_baseline(self, baseline: &Baseline) -> Self {
        let Self {
            mut reports,
            errors,
            total_duration_ms,
            ..
        } = self;
        for report in &mut reports {
            baseline.apply(report);
        }
        let audited: BTreeSet<&str> = reports.iter().map(|r| r.url.as_str()).collect();
        let unmatched_pages = baseline
            .file
            .pages
            .iter()
            .filter(|p| !audited.contains(p.url.as_str()))
            .count();
        let mut batch = Self::from_reports(reports, errors, total_duration_ms);
        if let Some(summary) = batch.summary.baseline.as_mut() {
            summary.unmatched_pages = unmatched_pages;
        } else {
            // No page produced a report: the baseline still names the run.
            batch
                .summary
                .decision
                .profile
                .set_baseline_digest(&baseline.digest);
            batch.summary.baseline = Some(BatchBaselineSummary {
                digest: baseline.digest.clone(),
                known: 0,
                new: 0,
                fixed: 0,
                unmatched_pages,
                gate: BatchBaselineGate::default(),
            });
        }
        batch
    }
}
