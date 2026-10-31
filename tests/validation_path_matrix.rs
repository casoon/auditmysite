//! Path matrix of the domain validation (#790).
//!
//! Every productive hand-over of a report — returning it, persisting it or
//! emitting it in a format — has to pass the format-independent contract
//! (`NormalizedReport::validate`, `validate_batch`) first. Two kinds of check:
//!
//! * Behaviour: a report whose data contradicts itself is refused by every
//!   fallible entry point that leaves the process (JSON single / cached /
//!   replay / batch, summary, SARIF, PDF source), not rendered.
//! * Wiring: the infallible compatibility API `normalize` may only be called
//!   in production code where the data was checked on the way in. The calls
//!   that exist today are a baseline that may only shrink
//!   (`tests/fixtures/architecture/unchecked_normalize.txt`); a new call in
//!   `cli`, `output`, `studio` or `lib.rs` fails here until it is justified.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use auditmysite::audit::{normalize, normalize_checked, validate_batch, AuditReport, BatchReport};
use auditmysite::cli::WcagLevel;
use auditmysite::wcag::WcagResults;

/// A report that states two things at once: a read-only measurement profile
/// and a journey that may change state.
fn contradictory() -> AuditReport {
    let mut report = AuditReport::new(
        "https://example.com".to_string(),
        WcagLevel::AA,
        WcagResults::new(),
        10,
    );
    let profile = &mut report.accessibility.execution.scope.measurement_profile;
    profile.version = auditmysite::audit::MEASUREMENT_PROFILE_VERSION;
    profile.read_only = true;
    profile.state_changing_journeys = vec!["checkout".to_string()];
    report
}

fn consistent() -> AuditReport {
    let mut report = contradictory();
    let profile = &mut report.accessibility.execution.scope.measurement_profile;
    profile.state_changing_journeys.clear();
    report
}

fn batch(report: AuditReport) -> BatchReport {
    BatchReport::from_reports(vec![report], Vec::new(), 10)
}

#[test]
fn the_fixtures_differ_only_in_the_contradiction() {
    assert!(normalize_checked(&consistent()).is_ok());
    let error = normalize_checked(&contradictory()).err().unwrap();
    assert!(error
        .violations
        .iter()
        .any(|v| v.code == "measurement_profile_read_only"));
}

#[test]
fn every_fallible_entry_refuses_contradictory_data() {
    let bad = contradictory();
    let ctx = normalize(&bad);
    let normalized = &ctx.normalized;

    assert!(normalize_checked(&bad).is_err(), "normalize_checked");
    assert!(
        auditmysite::output::format_json_normalized(&ctx, &bad, false).is_err(),
        "format_json_normalized"
    );
    assert!(
        auditmysite::output::format_json_cached(normalized, false).is_err(),
        "format_json_cached"
    );
    assert!(
        auditmysite::output::format_summary(normalized).is_err(),
        "format_summary"
    );
    assert!(
        auditmysite::output::format_sarif(&[normalized]).is_err(),
        "format_sarif"
    );
    assert!(
        validate_batch(&batch(contradictory())).is_err(),
        "validate_batch"
    );
    assert!(
        auditmysite::output::format_json_batch(&batch(contradictory()), false).is_err(),
        "format_json_batch"
    );
    #[cfg(feature = "pdf")]
    {
        let config = auditmysite::output::report_model::ReportConfig::default();
        assert!(
            auditmysite::output::generate_typ(&bad, &config).is_err(),
            "generate_typ"
        );
        assert!(
            auditmysite::output::generate_batch_typ(&batch(contradictory()), &config).is_err(),
            "generate_batch_typ"
        );
    }
}

#[test]
fn the_same_entries_accept_the_consistent_report() {
    let good = consistent();
    let ctx = normalize(&good);
    assert!(auditmysite::output::format_json_normalized(&ctx, &good, false).is_ok());
    assert!(auditmysite::output::format_json_cached(&ctx.normalized, false).is_ok());
    assert!(auditmysite::output::format_summary(&ctx.normalized).is_ok());
    assert!(auditmysite::output::format_sarif(&[&ctx.normalized]).is_ok());
    assert!(validate_batch(&batch(consistent())).is_ok());
    assert!(auditmysite::output::format_json_batch(&batch(consistent()), false).is_ok());
}

/// The infallible API stays available for callers that already hold checked
/// data, and it never turns bad data into good: its result still fails the
/// contract, so the checked entry points above refuse it.
#[test]
fn the_infallible_compatibility_api_does_not_launder_contradictions() {
    let bad = contradictory();
    let unchecked = normalize(&bad).normalized;
    assert!(unchecked.validate().is_err());
    // Migration path: `normalize_checked` is `normalize` plus `validate`, and
    // returns the identical model for valid data.
    let good = consistent();
    let checked = normalize_checked(&good).unwrap().normalized;
    assert_eq!(
        serde_json::to_value(&checked).unwrap(),
        serde_json::to_value(&normalize(&good).normalized).unwrap()
    );
}

// ─── Wiring ratchet ──────────────────────────────────────────────────────────

fn rust_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        out.push(path.to_path_buf());
        return;
    }
    for entry in fs::read_dir(path).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The part of a file that is not its trailing `#[cfg(test)] mod … {`.
fn production_part(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let end = (0..lines.len())
        .find(|&i| {
            lines[i].trim() == "#[cfg(test)]"
                && lines[i + 1..]
                    .iter()
                    .find(|l| !l.trim_start().starts_with("#["))
                    .is_some_and(|next| {
                        next.trim_start().starts_with("mod ") && next.trim_end().ends_with('{')
                    })
        })
        .unwrap_or(lines.len());
    lines[..end].join("\n")
}

fn unchecked_calls() -> BTreeMap<String, usize> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for part in [
        "src/cli",
        "src/output",
        "src/studio",
        "src/lib.rs",
        "src/main.rs",
    ] {
        rust_files(&root.join(part), &mut files);
    }
    let mut counts = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        // Files that are test code as a whole.
        if rel.ends_with("tests.rs") || rel.contains("/tests/") {
            continue;
        }
        let n = production_part(&fs::read_to_string(&file).unwrap())
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .filter(|l| l.contains("normalize(") && !l.contains("fn normalize("))
            .count();
        if n > 0 {
            counts.insert(rel, n);
        }
    }
    counts
}

fn baseline() -> BTreeMap<String, usize> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/architecture/unchecked_normalize.txt");
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (path, n) = l.rsplit_once(' ').expect("`path count`");
            (path.to_string(), n.parse().expect("count"))
        })
        .collect()
}

#[test]
fn unchecked_normalize_calls_do_not_grow() {
    let actual = unchecked_calls();
    let allowed = baseline();
    let mut problems = Vec::new();
    for (file, n) in &actual {
        match allowed.get(file) {
            None => problems.push(format!("{file}: {n} new unchecked normalize() call(s)")),
            Some(max) if n > max => {
                problems.push(format!("{file}: {n} unchecked calls, baseline {max}"))
            }
            Some(max) if n < max => problems.push(format!(
                "{file}: now {n}, baseline {max} — lower the baseline"
            )),
            _ => {}
        }
    }
    for file in allowed.keys().filter(|f| !actual.contains_key(*f)) {
        problems.push(format!(
            "{file}: no call left — remove it from the baseline"
        ));
    }
    assert!(
        problems.is_empty(),
        "use normalize_checked / validate_batch on every hand-over (#790):\n{}",
        problems.join("\n")
    );
}

/// The CLI dispatchers are the one place single and batch output start; they
/// validate before they choose a format.
#[test]
fn cli_output_dispatchers_validate_before_choosing_a_format() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = fs::read_to_string(root.join("src/cli/report_writers.rs")).unwrap();
    for (function, check) in [
        ("fn output_single_report(", "normalize_checked("),
        ("fn output_batch_report(", "validate_batch("),
    ] {
        let start = text
            .find(function)
            .unwrap_or_else(|| panic!("{function} moved"));
        let body: Vec<&str> = text[start..].lines().take(6).collect();
        let check_at = body.iter().position(|l| l.contains(check));
        let format_at = body.iter().position(|l| l.contains("effective_format"));
        assert!(
            check_at.is_some() && check_at < format_at,
            "{function} must call {check} before it picks a format"
        );
    }
}

/// A cache hit reaches the formats only through the dispatcher above.
#[test]
fn the_cache_hit_path_emits_through_the_validating_dispatcher() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = production_part(&fs::read_to_string(root.join("src/cli/runners.rs")).unwrap());
    let start = text.find("Cache hit:").expect("cache-hit branch");
    let branch = &text[start..start + text[start..].find("return Ok(RunOutcome").unwrap()];
    assert!(branch.contains("output_single_report("));
    for direct in [
        "UnifiedReport",
        "format_json_",
        "generate_pdf",
        "format_summary",
    ] {
        assert!(
            !branch.contains(direct),
            "cache hit must not format `{direct}` itself"
        );
    }
}
