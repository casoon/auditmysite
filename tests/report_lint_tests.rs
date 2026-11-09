//! End-to-end tests for the `report-lint` command (#507), now a maintainer
//! command of the `auditmysite-dev` binary (#912).
//!
//! Runs the shared command logic against fixture reports under
//! `tests/lint_fixtures/` and asserts on exit code + which check ids fired —
//! these fixtures double as the seed for #511's regression corpus.

use std::path::Path;

use auditmysite::cli::report_lint::{run_report_lint, ReportLintFailOn};

/// Runs the `report-lint` command logic (shared with the `auditmysite-dev`
/// binary) and mirrors its exit code: 0, or 3 when it returns an error.
/// `extra_args` uses the CLI spelling (`--fail-on`, `--typst-source`).
fn run_report_lint_fixture(fixture: &str, extra_args: &[&str]) -> (i32, String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/lint_fixtures")
        .join(fixture);
    let mut fail_on = None;
    let mut typst = None;
    let mut it = extra_args.iter();
    while let Some(flag) = it.next() {
        let value = *it.next().expect("flag value");
        match *flag {
            "--fail-on" => {
                fail_on = Some(match value {
                    "low" => ReportLintFailOn::Low,
                    "medium" => ReportLintFailOn::Medium,
                    "high" => ReportLintFailOn::High,
                    "critical" => ReportLintFailOn::Critical,
                    other => panic!("unknown --fail-on {other}"),
                })
            }
            "--typst-source" => typst = Some(std::path::PathBuf::from(value)),
            other => panic!("unknown flag {other}"),
        }
    }
    match run_report_lint(&path, fail_on, typst.as_deref()) {
        Ok(out) => (0, out),
        Err(e) => (3, e.to_string()),
    }
}

#[test]
fn clean_single_report_passes() {
    let (code, output) = run_report_lint_fixture("clean_single.json", &[]);
    assert_eq!(code, 0, "output: {output}");
    assert!(output.contains("no findings"), "output: {output}");
}

#[test]
fn broken_grade_mismatch_fails_by_default() {
    let (code, output) = run_report_lint_fixture("broken_grade_mismatch.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(
        output.contains("grade_matches_accessibility_score"),
        "output: {output}"
    );
}

/// Plan 29, D1: the grade is derived from `accessibility_score`,
/// not from the weighted overall. This fixture is clean under the old rule and
/// broken under the new one — the two scores agree in every other fixture, so
/// nothing else can tell the change apart.
#[test]
fn grade_derived_from_the_overall_score_is_rejected() {
    let (code, output) =
        run_report_lint_fixture("broken_grade_from_overall_not_accessibility.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(
        output.contains("grade_matches_accessibility_score"),
        "output: {output}"
    );
}

#[test]
fn broken_score_alias_fails_by_default() {
    let (code, output) = run_report_lint_fixture("broken_score_alias.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(output.contains("score_alias_removed"), "output: {output}");
}

#[test]
fn broken_batch_grade_fails_by_default() {
    let (code, output) = run_report_lint_fixture("broken_batch_grade.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(
        output.contains("grade_matches_accessibility_score"),
        "output: {output}"
    );
}

#[test]
fn broken_source_quality_image_contradiction_fails_by_default() {
    let (code, output) =
        run_report_lint_fixture("broken_source_quality_image_contradiction.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(
        output.contains("source_quality_signal_contradicts_findings"),
        "output: {output}"
    );
}

#[test]
fn fail_on_critical_tolerates_high_findings() {
    // broken_score_alias.json only produces a High finding (score alias
    // mismatch) — raising the bar to --fail-on critical must pass it.
    let (code, output) =
        run_report_lint_fixture("broken_score_alias.json", &["--fail-on", "critical"]);
    assert_eq!(code, 0, "output: {output}");
}

#[test]
fn fail_on_low_still_fails_on_high_findings() {
    let (code, _output) = run_report_lint_fixture("broken_score_alias.json", &["--fail-on", "low"]);
    assert_eq!(code, 3);
}

#[test]
fn pdf_score_traceable_passes_when_typst_source_matches() {
    let typst_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lint_fixtures/clean_single.typ");
    let (code, output) = run_report_lint_fixture(
        "clean_single.json",
        &["--typst-source", typst_path.to_str().unwrap()],
    );
    assert_eq!(code, 0, "output: {output}");
    assert!(output.contains("no findings"), "output: {output}");
}

#[test]
fn pdf_score_traceable_flags_missing_score_but_stays_low_severity() {
    let typst_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lint_fixtures/broken_missing_score.typ");
    let (code, output) = run_report_lint_fixture(
        "clean_single.json",
        &["--typst-source", typst_path.to_str().unwrap()],
    );
    // Low severity alone must not breach the default --fail-on high.
    assert_eq!(code, 0, "output: {output}");
    assert!(
        output.contains("pdf_score_traceable_to_json"),
        "output: {output}"
    );
}
