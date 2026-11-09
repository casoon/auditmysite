//! The `report-lint` maintainer command (#507), shared by the `auditmysite-dev`
//! binary and the release-gate tests.
//!
//! Lives in the library so `tests/report_lint_tests.rs` keeps gating every PR
//! in the plain `cargo test` jobs without building the dev binary (#912).

use std::path::Path;

use clap::ValueEnum;

use crate::error::{AuditError, Result};
use crate::lint::lint;
use crate::taxonomy::Severity;

/// `--fail-on` threshold for `report-lint`.
///
/// A separate CLI-facing enum (rather than deriving `clap::ValueEnum` on
/// `taxonomy::Severity` directly) so the domain type stays free of CLI
/// concerns, matching how `WcagLevel` is kept distinct from any internal
/// WCAG-level representation.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
#[value(rename_all = "lowercase")]
pub enum ReportLintFailOn {
    Low,
    Medium,
    High,
    Critical,
}

impl ReportLintFailOn {
    fn severity(fail_on: Option<Self>) -> Severity {
        match fail_on.unwrap_or(Self::High) {
            Self::Low => Severity::Low,
            Self::Medium => Severity::Medium,
            Self::High => Severity::High,
            Self::Critical => Severity::Critical,
        }
    }
}

/// Lints the report at `input` and returns the human-readable result text.
///
/// `Err` when the worst finding is at or above `fail_on` (default: high), or
/// when a file cannot be read or parsed — the dev binary maps it to exit 3.
pub fn run_report_lint(
    input: &Path,
    fail_on: Option<ReportLintFailOn>,
    typst_source: Option<&Path>,
) -> Result<String> {
    let text = std::fs::read_to_string(input).map_err(|e| AuditError::FileError {
        path: input.to_path_buf(),
        reason: e.to_string(),
    })?;
    let report: serde_json::Value = serde_json::from_str(&text)?;

    let typst_text = typst_source
        .map(|path| {
            std::fs::read_to_string(path).map_err(|e| AuditError::FileError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })
        })
        .transpose()?;

    let result = lint(&report, typst_text.as_deref());
    let threshold = ReportLintFailOn::severity(fail_on);

    let mut out = String::new();
    if result.is_clean() {
        out.push_str("report-lint: no findings\n");
    } else {
        out.push_str(&format!(
            "report-lint: {} finding(s)\n",
            result.findings.len()
        ));
        for finding in &result.findings {
            out.push_str(&format!(
                "  [{}] {} — {}\n      expected: {}\n      actual:   {}\n",
                format!("{:?}", finding.severity).to_uppercase(),
                finding.check_id,
                finding.evidence_path,
                finding.expected,
                finding.actual,
            ));
        }
    }

    match result.worst_severity() {
        Some(worst) if worst >= threshold => Err(AuditError::ConfigError(format!(
            "{out}report-lint found a {worst:?} finding, at or above --fail-on {threshold:?}"
        ))),
        _ => Ok(out),
    }
}
