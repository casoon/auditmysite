//! The reading side of `compare`: the few fields of a JSON report the
//! comparison needs, checked. Anything else in the report is ignored, so the
//! additive growth of the report schema never breaks a comparison.

use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;

use crate::audit::baseline::{finding_id_of, FINDING_ID_SCHEME};
use crate::compare::RunInfo;
use crate::error::{AuditError, Result};

#[derive(Debug, Deserialize)]
pub(super) struct Finding {
    pub id: String,
    pub rule_id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ModuleScore {
    pub name: String,
    pub score: u32,
}

#[derive(Debug, Deserialize)]
struct Decision {
    verdict: String,
}

#[derive(Debug, Deserialize)]
struct PageWire {
    url: String,
    accessibility_score: u32,
    site_quality_score: u32,
    #[serde(default)]
    module_scores: Vec<ModuleScore>,
    findings: Vec<Finding>,
    #[serde(default)]
    decision: Option<Decision>,
}

#[derive(Debug, Deserialize)]
struct ReportWire {
    report_type: String,
    #[serde(default)]
    tool_version: Option<String>,
    pages: Vec<PageWire>,
    #[serde(default)]
    decision: Option<Decision>,
}

/// One audited page of a report.
#[derive(Debug)]
pub(super) struct Page {
    pub url: String,
    pub accessibility_score: u32,
    pub site_quality_score: u32,
    pub module_scores: Vec<ModuleScore>,
    pub findings: Vec<Finding>,
    pub verdict: Option<String>,
}

/// A JSON report (single or batch), reduced to what the comparison reads.
#[derive(Debug)]
pub(crate) struct Report {
    report_type: String,
    tool_version: Option<String>,
    pub(super) pages: Vec<Page>,
    pub(super) verdict: Option<String>,
}

impl Report {
    pub(super) fn is_single(&self) -> bool {
        self.report_type == "single"
    }

    pub(super) fn info(&self) -> RunInfo {
        RunInfo {
            report_type: self.report_type.clone(),
            tool_version: self.tool_version.clone(),
            page_count: self.pages.len(),
        }
    }

    /// Parses and checks a report. `label` names it in the messages.
    pub(crate) fn parse(bytes: &[u8], label: &str) -> Result<Self> {
        let bad = |message: String| AuditError::ConfigError(format!("compare: {label}: {message}"));
        let wire: ReportWire = serde_json::from_slice(bytes).map_err(|e| {
            bad(format!(
                "not an auditmysite JSON report (expected report_type and pages[] as written by \
                 `-f json`): line {}, column {}",
                e.line(),
                e.column()
            ))
        })?;
        match wire.report_type.as_str() {
            "single" if wire.pages.len() != 1 => {
                return Err(bad(format!(
                    "a single report has one page, this one has {}",
                    wire.pages.len()
                )))
            }
            "single" | "batch" => {}
            other => {
                return Err(bad(format!(
                    "report_type {other:?} is neither \"single\" nor \"batch\""
                )))
            }
        }
        let mut urls = BTreeSet::new();
        for page in &wire.pages {
            if !urls.insert(page.url.as_str()) {
                return Err(bad(format!("page {} is listed twice", page.url)));
            }
            for finding in &page.findings {
                // The id must be what this tool derives from the rule: a
                // report of another id scheme is not comparable.
                if finding.id != finding_id_of(&finding.rule_id) {
                    return Err(bad(format!(
                        "finding id {} does not belong to rule {} under finding id scheme \
                         {FINDING_ID_SCHEME}; the report was written with another scheme, run \
                         the audit again with this version",
                        finding.id, finding.rule_id
                    )));
                }
            }
        }
        Ok(Self {
            report_type: wire.report_type,
            tool_version: wire.tool_version,
            verdict: wire.decision.map(|d| d.verdict),
            pages: wire
                .pages
                .into_iter()
                .map(|p| Page {
                    url: p.url,
                    accessibility_score: p.accessibility_score,
                    site_quality_score: p.site_quality_score,
                    module_scores: p.module_scores,
                    findings: p.findings,
                    verdict: p.decision.map(|d| d.verdict),
                })
                .collect(),
        })
    }
}

/// Reads a report file.
pub(crate) fn read_report(path: &Path) -> Result<Report> {
    let bytes = std::fs::read(path).map_err(|e| AuditError::FileError {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    Report::parse(&bytes, &path.display().to_string())
}
