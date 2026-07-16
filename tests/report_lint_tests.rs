//! End-to-end tests for the `auditmysite report-lint` CLI subcommand (#507).
//!
//! Spawns the compiled binary against fixture reports under
//! `tests/lint_fixtures/` and asserts on exit code + which check ids fired —
//! these fixtures double as the seed for #511's regression corpus.

use std::path::Path;
use std::process::Command;

fn run_report_lint(fixture: &str, extra_args: &[&str]) -> (i32, String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/lint_fixtures")
        .join(fixture);
    let output = Command::new(env!("CARGO_BIN_EXE_auditmysite"))
        .arg("report-lint")
        .arg(&path)
        .args(extra_args)
        .output()
        .expect("failed to run auditmysite report-lint");

    let mut combined = String::from_utf8_lossy(&output.stdout).to_string();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.code().unwrap_or(-1), combined)
}

#[test]
fn clean_single_report_passes() {
    let (code, output) = run_report_lint("clean_single.json", &[]);
    assert_eq!(code, 0, "output: {output}");
    assert!(output.contains("no findings"), "output: {output}");
}

#[test]
fn broken_grade_mismatch_fails_by_default() {
    let (code, output) = run_report_lint("broken_grade_mismatch.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(
        output.contains("grade_matches_overall_score"),
        "output: {output}"
    );
}

#[test]
fn broken_score_alias_fails_by_default() {
    let (code, output) = run_report_lint("broken_score_alias.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(
        output.contains("score_alias_matches_overall"),
        "output: {output}"
    );
}

#[test]
fn broken_batch_certificate_fails_by_default() {
    let (code, output) = run_report_lint("broken_batch_certificate.json", &[]);
    assert_eq!(code, 3, "output: {output}");
    assert!(
        output.contains("certificate_matches_overall_score"),
        "output: {output}"
    );
}

#[test]
fn fail_on_critical_tolerates_high_findings() {
    // broken_score_alias.json only produces a High finding (score alias
    // mismatch) — raising the bar to --fail-on critical must pass it.
    let (code, output) = run_report_lint("broken_score_alias.json", &["--fail-on", "critical"]);
    assert_eq!(code, 0, "output: {output}");
}

#[test]
fn fail_on_low_still_fails_on_high_findings() {
    let (code, _output) = run_report_lint("broken_score_alias.json", &["--fail-on", "low"]);
    assert_eq!(code, 3);
}

#[test]
fn pdf_certificate_traceable_passes_when_typst_source_matches() {
    let typst_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lint_fixtures/clean_single.typ");
    let (code, output) = run_report_lint(
        "clean_single.json",
        &["--typst-source", typst_path.to_str().unwrap()],
    );
    assert_eq!(code, 0, "output: {output}");
    assert!(output.contains("no findings"), "output: {output}");
}

#[test]
fn pdf_certificate_traceable_flags_missing_token_but_stays_low_severity() {
    let typst_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/lint_fixtures/broken_missing_certificate.typ");
    let (code, output) = run_report_lint(
        "clean_single.json",
        &["--typst-source", typst_path.to_str().unwrap()],
    );
    // Low severity alone must not breach the default --fail-on high.
    assert_eq!(code, 0, "output: {output}");
    assert!(
        output.contains("pdf_certificate_traceable_to_json"),
        "output: {output}"
    );
}
