//! `auditmysite compare <OLD.json> <NEW.json>` (#1235): the real binary on two
//! JSON reports, no browser and no network. The exit code is 0 for a run that is
//! not worse, 2 for a worse one and 3 for an input that cannot be compared.
// The infallible `normalize` is the deprecated 1.x compatibility API; the
// round-trip test needs a real report, not the model it derives.
#![allow(deprecated)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use auditmysite::audit::{normalize, AuditReport, BatchReport};
use auditmysite::cli::WcagLevel;
use auditmysite::output::{format_json_batch, UnifiedReport};
use auditmysite::wcag::{Severity, Violation, WcagResults};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/compare")
        .join(name)
}

fn compare(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_auditmysite"))
        .arg("compare")
        .args(args)
        .output()
        .expect("failed to run auditmysite compare")
}

fn run(old: &str, new: &str, extra: &[&str]) -> Output {
    let (old, new) = (fixture(old), fixture(new));
    let mut args = vec![old.to_str().unwrap(), new.to_str().unwrap()];
    args.extend_from_slice(extra);
    compare(&args)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn a_better_run_lists_what_was_resolved_and_exits_0() {
    let output = run("single-old.json", "single-new-better.json", &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let table = stdout(&output);
    assert!(table.contains("Before/after comparison"), "{table}");
    assert!(table.contains("1 resolved, 0 new, 1 unchanged"), "{table}");
    assert!(table.contains("images/alt-missing"), "{table}");
    assert!(table.contains("fail to warn"), "{table}");
    assert!(table.contains("no new findings, no score loss"), "{table}");
    // Overall 75 to 81.
    let overall = table.lines().find(|l| l.contains("Overall")).unwrap();
    assert!(
        overall.contains("75") && overall.contains("81") && overall.contains("+6"),
        "{overall}"
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_worse_run_exits_2() {
    let output = run("single-old.json", "single-new-worse.json", &[]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let table = stdout(&output);
    assert!(
        table.contains("the new run is worse (new findings)"),
        "{table}"
    );
    assert!(table.contains("contrast/text-insufficient"), "{table}");
}

#[test]
fn german_text_follows_lang_and_english_has_no_umlauts() {
    let de = stdout(&run(
        "single-old.json",
        "single-new-better.json",
        &["--lang", "de"],
    ));
    assert!(de.contains("Vorher/Nachher-Vergleich"), "{de}");
    assert!(de.contains("1 behoben, 0 neu, 1 unverändert"), "{de}");
    let en = stdout(&run("single-old.json", "single-new-better.json", &[]));
    assert!(!en.chars().any(|c| "äöüÄÖÜß".contains(c)), "{en}");
}

#[test]
fn a_batch_is_compared_per_url_and_lists_urls_in_one_report_only() {
    let output = run("batch-old.json", "batch-new.json", &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let table = stdout(&output);
    assert!(table.contains("2 pages compared"), "{table}");
    assert!(table.contains("Only in the old report"), "{table}");
    assert!(table.contains("https://example.com/gone"), "{table}");
    assert!(table.contains("Only in the new report"), "{table}");
    assert!(table.contains("https://example.com/contact"), "{table}");
}

#[test]
fn json_output_matches_the_schema_and_carries_the_exit_decision() {
    let output = run("single-old.json", "single-new-worse.json", &["-f", "json"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON on stdout");
    assert_matches_schema(&value);
    assert_eq!(value["summary"]["worse"], true);
    assert_eq!(
        value["summary"]["reasons"],
        serde_json::json!(["new_findings"])
    );

    for (old, new) in [
        ("single-old.json", "single-new-better.json"),
        ("batch-old.json", "batch-new.json"),
        ("single-old.json", "batch-new.json"),
    ] {
        let value: serde_json::Value =
            serde_json::from_slice(&run(old, new, &["-f", "json"]).stdout).unwrap();
        assert_matches_schema(&value);
    }
}

fn assert_matches_schema(instance: &serde_json::Value) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/json-compare.schema.json");
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let validator = jsonschema::validator_for(&schema).expect("schema compiles");
    let errors: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn output_goes_to_the_file_named_by_dash_o_and_its_extension_picks_the_format() {
    let dir = tempfile::tempdir().unwrap();
    let json_path = dir.path().join("diff.json");
    let output = run(
        "single-old.json",
        "single-new-better.json",
        &["-o", json_path.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(json_path).unwrap()).unwrap();
    assert_eq!(value["contract"], "auditmysite.compare/1");
}

#[test]
fn unusable_input_exits_3_with_the_reason() {
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("junk.json");
    std::fs::write(&junk, "{\"hello\": 1}").unwrap();
    let junk = junk.to_str().unwrap().to_string();
    let good = fixture("single-old.json");
    let good = good.to_str().unwrap();

    let output = compare(&[good, &junk]);
    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not an auditmysite JSON report"));

    let missing = compare(&[good, "/no/such/report.json"]);
    assert_eq!(missing.status.code(), Some(3));

    // Another finding id scheme: refused, never "everything is new".
    let other = dir.path().join("other-scheme.json");
    let text = std::fs::read_to_string(fixture("single-old.json"))
        .unwrap()
        .replace("F-84ea8500", "F-deadbeef");
    std::fs::write(&other, text).unwrap();
    let output = compare(&[good, other.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("finding id scheme"), "{stderr}");

    let sarif = run(
        "single-old.json",
        "single-new-better.json",
        &["-f", "sarif"],
    );
    assert_eq!(sarif.status.code(), Some(3));
}

#[test]
fn compare_is_in_the_user_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_auditmysite"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(
        stdout(&output).contains("\n  compare "),
        "{}",
        stdout(&output)
    );
}

#[cfg(feature = "pdf")]
#[test]
fn the_pdf_is_a_short_report_in_both_languages() {
    let dir = tempfile::tempdir().unwrap();
    for (lang, pairs) in [
        ("en", ("single-old.json", "single-new-worse.json")),
        ("de", ("single-old.json", "single-new-better.json")),
        ("en", ("batch-old.json", "batch-new.json")),
    ] {
        let path = dir.path().join(format!("{lang}-{}.pdf", pairs.1));
        let output = run(
            pairs.0,
            pairs.1,
            &["-f", "pdf", "--lang", lang, "-o", path.to_str().unwrap()],
        );
        assert!(
            matches!(output.status.code(), Some(0) | Some(2)),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let pdf = lopdf::Document::load(&path).expect("a readable PDF");
        let pages = pdf.get_pages().len();
        assert!((1..=3).contains(&pages), "{pages} pages");
    }
    // A PDF needs a file to go to.
    let output = run("single-old.json", "single-new-better.json", &["-f", "pdf"]);
    assert_eq!(output.status.code(), Some(3));
}

// ── Real reports ─────────────────────────────────────────────────────────────

fn page(url: &str, rules: &[&str]) -> AuditReport {
    let mut results = WcagResults::new();
    for rule in rules {
        results.add_violation(
            Violation::new(
                "1.1.1",
                "Non-text Content",
                WcagLevel::A,
                Severity::High,
                "m",
                "n",
            )
            .with_rule_id(*rule)
            .with_selector("img.hero"),
        );
    }
    AuditReport::new(url.to_string(), WcagLevel::AA, results, 1)
}

fn single_json(report: &AuditReport) -> String {
    UnifiedReport::single(&normalize(report), report)
        .to_json(true)
        .unwrap()
}

/// The reports `-f json` writes are the input: the ids they carry are the ids
/// the comparison derives, so a report compared with itself has no movement.
#[test]
fn real_reports_are_comparable() {
    let dir = tempfile::tempdir().unwrap();
    let write = |name: &str, json: String| {
        let path = dir.path().join(name);
        std::fs::write(&path, json).unwrap();
        path
    };
    let old = write(
        "old.json",
        single_json(&page(
            "https://example.com/",
            &["images/alt-missing", "buttons/name-missing"],
        )),
    );
    let new = write(
        "new.json",
        single_json(&page("https://example.com/", &["buttons/name-missing"])),
    );
    let output = compare(&[old.to_str().unwrap(), new.to_str().unwrap(), "-f", "json"]);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_matches_schema(&value);
    assert_eq!(value["summary"]["resolved"], 1);
    assert_eq!(value["summary"]["unchanged"], 1);
    assert_eq!(value["summary"]["new"], 0);
    assert_eq!(
        value["pages"][0]["findings"]["resolved"][0]["rule"],
        "images/alt-missing"
    );
    assert_ne!(
        output.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let same = compare(&[old.to_str().unwrap(), old.to_str().unwrap(), "-f", "json"]);
    assert_eq!(same.status.code(), Some(0));

    let batch = |urls: &[&str]| {
        let reports = urls
            .iter()
            .map(|u| page(u, &["images/alt-missing"]))
            .collect();
        format_json_batch(&BatchReport::from_reports(reports, vec![], 1), true).unwrap()
    };
    let old_batch = write(
        "old-batch.json",
        batch(&["https://a.example/", "https://b.example/"]),
    );
    let new_batch = write(
        "new-batch.json",
        batch(&["https://a.example/", "https://c.example/"]),
    );
    let output = compare(&[
        old_batch.to_str().unwrap(),
        new_batch.to_str().unwrap(),
        "-f",
        "json",
    ]);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_matches_schema(&value);
    assert_eq!(value["pages"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["only_in_old"],
        serde_json::json!(["https://b.example/"])
    );
    assert_eq!(
        value["only_in_new"],
        serde_json::json!(["https://c.example/"])
    );
}
