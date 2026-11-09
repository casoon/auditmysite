//! Findings baseline for CI gating (#930): `--write-baseline` records the
//! finding ids of a run, `--baseline` makes the findings with a recorded id
//! `known`, so only new findings reach the verdict and the exit code.
//!
//! Browser-free: the reports are assembled from violations, as in the other
//! report-contract tests.

use auditmysite::audit::{
    normalize_checked, validate_batch, AuditReport, Baseline, BatchReport, NormalizedReport,
    Verdict, BASELINE_CONTRACT, FINDING_ID_SCHEME,
};
use auditmysite::cli::WcagLevel;
use auditmysite::wcag::{Severity, Violation, WcagResults};
use chrono::{DateTime, Utc};

const BUTTONS: &str = "buttons/name-missing";
const IMAGES: &str = "images/alt-missing";

fn created_at() -> DateTime<Utc> {
    "2026-11-09T10:00:00Z".parse().unwrap()
}

fn violation(rule: &str, selector: &str) -> Violation {
    let (criterion, name) = match rule {
        IMAGES => ("1.1.1", "Non-text Content"),
        _ => ("4.1.2", "Name, Role, Value"),
    };
    Violation::new(
        criterion,
        name,
        WcagLevel::A,
        Severity::High,
        "message",
        "node",
    )
    .with_rule_id(rule)
    .with_selector(selector.to_string())
}

fn report(url: &str, violations: &[(&str, &str)]) -> AuditReport {
    let mut results = WcagResults::new();
    for (rule, selector) in violations {
        results.add_violation(violation(rule, selector));
    }
    AuditReport::new(url.into(), WcagLevel::AA, results, 1)
}

fn normalized(report: &AuditReport) -> NormalizedReport {
    normalize_checked(report).expect("report").normalized
}

fn baseline_of(reports: &[&AuditReport]) -> Baseline {
    let pages: Vec<NormalizedReport> = reports.iter().map(|r| normalized(r)).collect();
    Baseline::from_reports(&pages, created_at())
}

fn gated(mut report: AuditReport, baseline: &Baseline) -> NormalizedReport {
    baseline.apply(&mut report);
    normalized(&report)
}

fn finding_id<'a>(page: &'a NormalizedReport, rule: &str) -> &'a str {
    page.findings
        .iter()
        .find(|f| f.rule_id == rule)
        .unwrap_or_else(|| panic!("no finding for {rule}"))
        .id
        .as_str()
}

// ── Id stability ─────────────────────────────────────────────────────────────

/// The ids a baseline relies on (FNV-1a 64 of `<rule id>\n__pagewide__`, upper
/// 32 bits, derived independently of `a11y-report` when they were pinned). If this test fails, ids of unchanged
/// findings changed: either `FindingId`'s derivation (bump
/// `FINDING_ID_SCHEME`, so old baselines are refused) or a rule was renamed
/// (its findings are new to every existing baseline; say so in the changelog).
#[test]
fn finding_ids_of_known_rules_are_pinned() {
    let page = normalized(&report(
        "https://example.com/",
        &[(BUTTONS, "button.a"), (IMAGES, "img.logo")],
    ));
    assert_eq!(finding_id(&page, BUTTONS), "F-84ea8500");
    assert_eq!(finding_id(&page, IMAGES), "F-12a4e078");
    assert_eq!(FINDING_ID_SCHEME, 1);
}

/// An unchanged finding keeps its id across two runs, when elements are
/// added, removed or reordered, and on another page.
#[test]
fn a_finding_id_survives_reordering_and_element_changes() {
    let first = normalized(&report(
        "https://example.com/",
        &[
            (BUTTONS, "nav > a.t-1"),
            (IMAGES, "img.a"),
            (BUTTONS, "footer a"),
        ],
    ));
    let second = normalized(&report(
        "https://example.com/",
        &[
            (BUTTONS, "footer a"),
            (IMAGES, "img.a"),
            (BUTTONS, "nav > a.t-1"),
        ],
    ));
    let changed = normalized(&report(
        "https://example.com/other",
        &[
            (IMAGES, "img.new"),
            (IMAGES, "img.other"),
            (BUTTONS, "main button"),
        ],
    ));
    for rule in [BUTTONS, IMAGES] {
        assert_eq!(finding_id(&first, rule), finding_id(&second, rule));
        assert_eq!(finding_id(&first, rule), finding_id(&changed, rule));
    }
    // And so does the baseline: same identity, same digest, whatever the order.
    let a = baseline_of(&[&report(
        "https://example.com/",
        &[(BUTTONS, "x"), (IMAGES, "y")],
    )]);
    let b = baseline_of(&[&report(
        "https://example.com/",
        &[(IMAGES, "z"), (BUTTONS, "w"), (IMAGES, "v")],
    )]);
    assert_eq!(a.digest(), b.digest());
}

// ── File format ──────────────────────────────────────────────────────────────

#[test]
fn a_baseline_round_trips_through_its_file() {
    let page = report("https://example.com/", &[(BUTTONS, "b"), (IMAGES, "i")]);
    let baseline = baseline_of(&[&page]);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("findings.json");
    baseline.write(&path).unwrap();

    let read = Baseline::read(&path).unwrap();
    assert_eq!(read, baseline);
    assert_eq!(read.digest(), baseline.digest());

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(json["contract"], BASELINE_CONTRACT);
    assert_eq!(json["id_scheme"], FINDING_ID_SCHEME);
    assert_eq!(json["tool_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(json["created_at"], "2026-11-09T10:00:00Z");
    assert_eq!(json["pages"][0]["url"], "https://example.com/");
    let findings = json["pages"][0]["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 2);
    for f in findings {
        assert!(f["id"].as_str().unwrap().starts_with("F-"));
        assert!(f["rule"].is_string());
    }
}

#[test]
fn the_digest_follows_the_recorded_identity_only() {
    let page = report("https://example.com/", &[(BUTTONS, "b")]);
    let early = Baseline::from_reports(&[normalized(&page)], created_at());
    let late = Baseline::from_reports(&[normalized(&page)], Utc::now());
    assert_eq!(early.digest(), late.digest());

    let other = baseline_of(&[&report("https://example.com/", &[(IMAGES, "i")])]);
    assert_ne!(early.digest(), other.digest());
    let other_page = baseline_of(&[&report("https://example.com/b", &[(BUTTONS, "b")])]);
    assert_ne!(early.digest(), other_page.digest());
}

fn config_error(bytes: &str) -> String {
    match Baseline::parse(bytes.as_bytes()) {
        Ok(_) => panic!("accepted: {bytes}"),
        Err(auditmysite::error::AuditError::ConfigError(message)) => message,
        Err(other) => panic!("not a usage error: {other}"),
    }
}

#[test]
fn a_malformed_or_incompatible_baseline_is_a_usage_error() {
    assert!(config_error("not json").contains("not a baseline file"));
    assert!(config_error("{}").contains("not a baseline file"));

    let good = baseline_of(&[&report("https://example.com/", &[(BUTTONS, "b")])]).to_json();
    let good: serde_json::Value = serde_json::from_str(&good).unwrap();

    let mut contract = good.clone();
    contract["contract"] = "auditmysite.baseline/99".into();
    assert!(config_error(&contract.to_string()).contains("contract"));

    let mut scheme = good.clone();
    scheme["id_scheme"] = 2.into();
    assert!(config_error(&scheme.to_string()).contains("finding id scheme"));

    // An id this tool would not derive from the rule: a changed derivation or
    // a hand edit. It must not read as "everything is new".
    let mut forged = good.clone();
    forged["pages"][0]["findings"][0]["id"] = "F-00000000".into();
    assert!(config_error(&forged.to_string()).contains("does not belong to rule"));

    let mut twice = good.clone();
    let page = twice["pages"][0].clone();
    twice["pages"].as_array_mut().unwrap().push(page);
    assert!(config_error(&twice.to_string()).contains("listed twice"));

    let missing = Baseline::read(std::path::Path::new("/nonexistent/baseline.json"));
    assert!(missing.is_err());
}

// ── Classification and verdict ───────────────────────────────────────────────

#[test]
fn known_findings_do_not_affect_the_verdict_but_new_ones_do() {
    let accepted = report("https://example.com/", &[(BUTTONS, "b"), (IMAGES, "i")]);
    let baseline = baseline_of(&[&accepted]);

    let without = normalized(&accepted);
    assert_eq!(without.decision.verdict, Verdict::Fail);
    assert_eq!(without.decision.verdict.exit_code(), 2);
    assert!(without.baseline.is_none());
    assert!(without.findings.iter().all(|f| !f.known));
    assert!(without.decision.profile.baseline_digest.is_none());
    assert_eq!(without.decision.profile.name, "default");

    // The same run under its own baseline: everything is known.
    let with = gated(accepted.clone(), &baseline);
    assert!(with.findings.iter().all(|f| f.known));
    let status = with.baseline.as_ref().expect("status");
    assert_eq!((status.known, status.new, status.fixed), (2, 0, 0));
    assert!(status.page_listed);
    assert_eq!(with.decision.verdict, Verdict::Pass);
    assert_eq!(with.decision.verdict.exit_code(), 0);
    assert!(with.decision.reasons.is_empty());

    // A baseline is a gating filter: score and findings are unchanged.
    assert_eq!(with.score, without.score);
    assert_eq!(with.overall_score, without.overall_score);
    assert_eq!(with.findings.len(), without.findings.len());
    assert_eq!(
        with.occurrence_counts.total,
        without.occurrence_counts.total
    );
    assert_eq!(with.severity_counts.total, without.severity_counts.total);
    assert_eq!(with.risk.legal_flags, without.risk.legal_flags);

    // One more rule is new: it alone decides.
    let regression = report(
        "https://example.com/",
        &[(BUTTONS, "b"), (IMAGES, "i"), (IMAGES, "i2")],
    );
    let only_buttons = baseline_of(&[&report("https://example.com/", &[(BUTTONS, "b")])]);
    let page = gated(regression, &only_buttons);
    let status = page.baseline.as_ref().unwrap();
    assert_eq!((status.known, status.new), (1, 1));
    assert_eq!(page.decision.verdict, Verdict::Fail);
    assert!(page
        .findings
        .iter()
        .any(|f| f.rule_id == BUTTONS && f.known));
    assert!(page
        .findings
        .iter()
        .any(|f| f.rule_id == IMAGES && !f.known));
    // The reasons count the open findings only.
    assert_eq!(page.decision.reasons, ["legal_flags: 1"]);
}

#[test]
fn baseline_ids_without_a_finding_are_reported_as_fixed() {
    let before = report("https://example.com/", &[(BUTTONS, "b"), (IMAGES, "i")]);
    let baseline = baseline_of(&[&before]);
    let after = report("https://example.com/", &[(BUTTONS, "b")]);
    let page = gated(after, &baseline);
    let status = page.baseline.as_ref().unwrap();
    assert_eq!((status.known, status.new, status.fixed), (1, 0, 1));
    let images = finding_id(&normalized(&before), IMAGES).to_string();
    assert_eq!(status.fixed_ids, [images]);
    assert_eq!(page.decision.verdict, Verdict::Pass);
}

#[test]
fn a_page_the_baseline_does_not_list_has_no_known_findings() {
    let baseline = baseline_of(&[&report("https://example.com/", &[(BUTTONS, "b")])]);
    let page = gated(
        report("https://example.com/other", &[(BUTTONS, "b")]),
        &baseline,
    );
    let status = page.baseline.as_ref().unwrap();
    assert!(!status.page_listed);
    assert_eq!((status.known, status.new), (0, 1));
    assert_eq!(page.decision.verdict, Verdict::Fail);
}

#[test]
fn the_verdict_profile_records_the_baseline_digest() {
    let accepted = report("https://example.com/", &[(BUTTONS, "b")]);
    let baseline = baseline_of(&[&accepted]);
    let page = gated(accepted.clone(), &baseline);
    assert_eq!(
        page.decision.profile.baseline_digest.as_deref(),
        Some(baseline.digest())
    );
    assert_eq!(page.baseline.as_ref().unwrap().digest, baseline.digest());
    // Never the default profile: with and without baseline cannot be mistaken.
    assert_eq!(page.decision.profile.name, "custom");
    assert_ne!(
        page.decision.profile,
        normalized(&accepted).decision.profile
    );
    let json = serde_json::to_value(&page.decision).unwrap();
    assert_eq!(json["profile"]["baseline_digest"], baseline.digest());
}

#[test]
fn a_gated_report_is_valid_and_its_stored_decision_reproducible() {
    let accepted = report("https://example.com/", &[(BUTTONS, "b"), (IMAGES, "i")]);
    let baseline = baseline_of(&[&accepted]);
    let page = gated(accepted, &baseline);
    page.validate().expect("valid");
    let again = auditmysite::audit::compute_verdict(&page, &page.decision.profile);
    assert_eq!(again, page.decision);
}

#[test]
fn the_score_thresholds_are_not_baselined() {
    let accepted = report("https://example.com/", &[(BUTTONS, "b")]);
    let baseline = baseline_of(&[&accepted]);
    let mut gated_report = accepted;
    baseline.apply(&mut gated_report);
    gated_report.run_context.decision_profile = {
        let mut profile =
            auditmysite::audit::DecisionProfile::resolve(None, None, None, Some(100), None);
        profile.set_baseline_digest(baseline.digest());
        profile
    };
    let page = normalized(&gated_report);
    assert_eq!(page.decision.verdict, Verdict::Fail);
    assert!(page.decision.reasons[0].contains("fail_below_score"));
}

// ── Batch ────────────────────────────────────────────────────────────────────

#[test]
fn a_batch_is_gated_per_page_and_counts_unmatched_pages() {
    let p1 = report("https://example.com/a", &[(BUTTONS, "b")]);
    let p2 = report("https://example.com/b", &[(BUTTONS, "b")]);
    let gone = report("https://example.com/gone", &[(IMAGES, "i")]);
    // /a is recorded, /b is not (same rule, other page), /gone is not in the run.
    let baseline = baseline_of(&[&p1, &gone]);

    let plain = BatchReport::from_reports(vec![p1.clone(), p2.clone()], vec![], 1);
    assert!(plain.summary.baseline.is_none());
    assert_eq!(plain.summary.decision.verdict, Verdict::Fail);

    let batch = BatchReport::from_reports(vec![p1, p2], vec![], 1).with_baseline(&baseline);
    validate_batch(&batch).expect("valid batch");
    let summary = batch.summary.baseline.as_ref().expect("batch baseline");
    assert_eq!(summary.digest, baseline.digest());
    assert_eq!((summary.known, summary.new, summary.fixed), (1, 1, 0));
    assert_eq!(summary.unmatched_pages, 1);
    assert_eq!(
        batch.summary.decision.profile.baseline_digest.as_deref(),
        Some(baseline.digest())
    );
    // /b has a new finding (the same rule, but on a page the baseline does not
    // list), /a is known: the page verdicts and the batch follow.
    assert_eq!(batch.summary.passed, 1);
    assert_eq!(batch.summary.failed, 1);
    assert_eq!(batch.summary.decision.verdict, Verdict::Fail);
    // Totals stay unfiltered; the gate holds what is left.
    assert_eq!(batch.summary.total_violations, 2);
    assert_eq!(summary.gate.open_violations, 1);

    // Everything known: the batch passes, the totals still say what exists.
    let all = baseline_of(&[
        &report("https://example.com/a", &[(BUTTONS, "b")]),
        &report("https://example.com/b", &[(BUTTONS, "b")]),
    ]);
    let accepted = BatchReport::from_reports(
        vec![
            report("https://example.com/a", &[(BUTTONS, "b")]),
            report("https://example.com/b", &[(BUTTONS, "b")]),
        ],
        vec![],
        1,
    )
    .with_baseline(&all);
    validate_batch(&accepted).expect("valid batch");
    assert_eq!(accepted.summary.decision.verdict, Verdict::Pass);
    assert_eq!(accepted.summary.total_violations, 2);
    assert_eq!(accepted.summary.baseline.as_ref().unwrap().known, 2);
}

#[test]
fn the_json_carries_known_findings_and_the_baseline_blocks() {
    use auditmysite::output::UnifiedReport;

    let accepted = report("https://example.com/", &[(BUTTONS, "b")]);
    let baseline = baseline_of(&[&accepted]);

    let mut single = accepted.clone();
    baseline.apply(&mut single);
    let ctx = normalize_checked(&single).expect("report");
    let json = serde_json::to_value(UnifiedReport::single(&ctx, &single)).unwrap();
    assert_eq!(json["pages"][0]["findings"][0]["known"], true);
    assert_eq!(json["pages"][0]["baseline"]["known"], 1);
    assert_eq!(json["pages"][0]["baseline"]["digest"], baseline.digest());
    assert_eq!(
        json["decision"]["profile"]["baseline_digest"],
        baseline.digest()
    );
    assert_eq!(json["decision"]["verdict"], "pass");

    let batch = BatchReport::from_reports(vec![accepted], vec![], 1).with_baseline(&baseline);
    let json = serde_json::to_value(UnifiedReport::batch(&batch)).unwrap();
    assert_eq!(json["baseline"]["known"], 1);
    assert_eq!(json["baseline"]["unmatched_pages"], 0);
    assert_eq!(json["pages"][0]["baseline"]["known"], 1);

    // Without a baseline neither block exists.
    let plain = BatchReport::from_reports(
        vec![report("https://example.com/", &[(BUTTONS, "b")])],
        vec![],
        1,
    );
    let json = serde_json::to_value(UnifiedReport::batch(&plain)).unwrap();
    assert!(json.get("baseline").is_none());
    assert!(json["pages"][0].get("baseline").is_none());
    assert!(json["pages"][0]["findings"][0].get("known").is_none());
}

fn assert_matches_schema(instance: &serde_json::Value, schema_name: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join(schema_name);
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let messages: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| e.to_string())
        .collect();
    assert!(
        messages.is_empty(),
        "{schema_name}:\n{}",
        messages.join("\n")
    );
}

#[test]
fn the_baseline_blocks_match_the_published_schemas() {
    use auditmysite::output::UnifiedReport;

    let accepted = report("https://example.com/", &[(BUTTONS, "b"), (IMAGES, "i")]);
    let baseline = baseline_of(&[&report("https://example.com/", &[(BUTTONS, "b")])]);

    let mut single = accepted.clone();
    baseline.apply(&mut single);
    let ctx = normalize_checked(&single).expect("report");
    let json = serde_json::to_value(UnifiedReport::single(&ctx, &single)).unwrap();
    assert_eq!(json["pages"][0]["baseline"]["new"], 1);
    assert_matches_schema(&json, "json-report.schema.json");

    let batch = BatchReport::from_reports(vec![accepted], vec![], 1).with_baseline(&baseline);
    let json = serde_json::to_value(UnifiedReport::batch(&batch)).unwrap();
    assert_matches_schema(&json, "json-batch-report.schema.json");
}

#[test]
fn the_table_shows_a_compact_known_count_only_under_a_baseline() {
    use clap::Parser;

    let accepted = report("https://example.com/", &[(BUTTONS, "b"), (IMAGES, "i")]);
    let baseline = baseline_of(&[&report("https://example.com/", &[(BUTTONS, "b")])]);
    let args = auditmysite::cli::Args::parse_from(["auditmysite", "https://example.com/"]);

    let plain = auditmysite::output::terminal::render_single_report(&accepted, &args).unwrap();
    assert!(!plain.contains("known"), "{plain}");

    let mut gated_report = accepted;
    baseline.apply(&mut gated_report);
    let text = auditmysite::output::terminal::render_single_report(&gated_report, &args).unwrap();
    assert!(
        text.contains("Baseline: 1 known, 1 new, 0 fixed findings"),
        "{text}"
    );
    // A count, not one line per known finding.
    assert_eq!(text.matches("known").count(), 2, "{text}");
}

#[test]
fn the_baseline_text_is_localized_and_english_has_no_umlauts() {
    use auditmysite::audit::baseline::summary_text;
    let en = summary_text(2, 1, 0, Some(3), true);
    let de = summary_text(2, 1, 0, Some(3), false);
    assert!(en.is_ascii(), "{en}");
    assert!(en.contains("2 known, 1 new, 0 fixed") && en.contains("3 baseline pages"));
    assert!(de.contains("2 bekannte, 1 neue, 0 behobene") && de.contains("3 Baseline-Seiten"));
    assert!(!summary_text(1, 0, 0, Some(0), true).contains("baseline pages"));
}
