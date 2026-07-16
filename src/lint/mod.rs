//! Deterministic report-lint (#507).
//!
//! Reproducible, language-model-free checks over a serialized JSON audit
//! report: score/grade/certificate consistency, severity/occurrence count
//! sums, and registry (#506) path coverage. Complements the AI critic (#509)
//! planned for qualitative/visual findings — this module only checks things
//! that are either true or false, never a judgment call.

mod checks;

use serde::Serialize;

use checks::check_pdf_certificate_traceability;
pub use checks::run_all_checks;

use crate::taxonomy::Severity;

/// One lint finding: which check produced it, where in the report, what was
/// expected vs. observed, and how severe the mismatch is.
#[derive(Debug, Clone, Serialize)]
pub struct LintFinding {
    pub check_id: &'static str,
    pub evidence_path: String,
    pub expected: String,
    pub actual: String,
    pub severity: Severity,
}

/// The full result of linting one report.
#[derive(Debug, Clone, Serialize, Default)]
pub struct LintReport {
    pub findings: Vec<LintFinding>,
}

impl LintReport {
    /// The most severe finding, or `None` if the report is clean.
    pub fn worst_severity(&self) -> Option<Severity> {
        self.findings.iter().map(|f| f.severity).max()
    }

    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Run every deterministic check against a parsed JSON report value.
///
/// Accepts a raw [`serde_json::Value`] (not a typed struct) so it can lint
/// reports produced by any tool version, including ones with fields this
/// build of auditmysite no longer recognizes.
///
/// `typst_source` is the optional `--debug-typ` Typst source text for the
/// same report; when given, it additionally runs the scoped PDF-traceability
/// check (does the report's certificate token actually appear in the
/// rendered output).
pub fn lint(report: &serde_json::Value, typst_source: Option<&str>) -> LintReport {
    let mut findings = Vec::new();
    run_all_checks(report, &mut findings);
    if let Some(typst_source) = typst_source {
        check_pdf_certificate_traceability(report, typst_source, &mut findings);
    }
    LintReport { findings }
}
