//! Calibration of the heuristic signal scores against a blind,
//! human-labelled reference set (#784). Procedure: `docs/calibration.md`.
//!
//! Two kinds of test live here:
//!
//! - **Integrity** (always run): the reference set and the pre-registration
//!   parse, labels carry no tool output, thresholds were registered before
//!   any label was written, the registered contract versions match the code
//!   and — once labels are frozen — the set is byte-identical to what was
//!   frozen. Plus unit tests of the metrics.
//! - **Calibration run** (`#[ignore]`, needs Chrome and network): audits the
//!   holdout cases, measures false-positive and false-negative rates and the
//!   rank agreement with the human ratings per signal, writes the result to
//!   `reports/calibration-results.json` and fails if a pre-registered
//!   threshold is missed. It refuses to run until labels are frozen.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ─── Format ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceSet {
    format: String,
    version: u32,
    #[allow(dead_code)]
    notes: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    url: String,
    /// `dev` may be inspected while developing; only `holdout` decides.
    split: Split,
    /// Why the case was chosen, recorded before any tool run.
    stratum: Stratum,
    selected_by: String,
    selected_at: String,
    /// Signals this case is labelled for.
    signals: Vec<String>,
    labels: Vec<Label>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Split {
    Dev,
    Holdout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Stratum {
    Positive,
    Negative,
    Borderline,
}

/// One person's judgement of one signal on one case. `deny_unknown_fields`
/// keeps tool output (a score, a band) out of the labels.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Label {
    signal: String,
    labeller: String,
    labelled_at: String,
    judgement: Judgement,
    /// 1 = severe problems … 5 = no problems, for the rank agreement.
    rating: u8,
    #[allow(dead_code)]
    rationale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Judgement {
    Problem,
    NoProblem,
    Undecidable,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Preregistration {
    format: String,
    registered_at: String,
    #[allow(dead_code)]
    notes: String,
    reference_set: String,
    labels_sha256: Option<String>,
    max_days_between_label_and_run: u32,
    requirements: Requirements,
    signals: BTreeMap<String, SignalThresholds>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirements {
    min_labellers_per_case: usize,
    min_holdout_cases_per_signal: usize,
    min_holdout_positive: usize,
    min_holdout_negative: usize,
    min_holdout_borderline: usize,
    min_inter_rater_kappa: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SignalThresholds {
    contract_version: u32,
    /// The tool claims a problem when the score is below this value.
    problem_cutoff: u32,
    max_false_positive_rate: f64,
    max_false_negative_rate: f64,
    min_kendall_tau_b: f64,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_prereg() -> Preregistration {
    let path = root().join("tests/fixtures/calibration/preregistration.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("preregistration readable"))
        .expect("preregistration parses")
}

fn reference_set_bytes(prereg: &Preregistration) -> Vec<u8> {
    std::fs::read(root().join(&prereg.reference_set)).expect("reference set readable")
}

fn load_set(bytes: &[u8]) -> ReferenceSet {
    serde_json::from_slice(bytes).expect("reference set parses")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// ISO dates (`YYYY-MM-DD`) compare correctly as strings; this rejects
/// anything else so the comparison stays sound.
fn iso_date(s: &str) -> &str {
    let b = s.as_bytes();
    assert!(
        b.len() == 10
            && b[4] == b'-'
            && b[7] == b'-'
            && s.chars().filter(|c| c.is_ascii_digit()).count() == 8,
        "not an ISO date: {s}"
    );
    s
}

// ─── Metrics ─────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
struct Confusion {
    tp: usize,
    fp: usize,
    tn: usize,
    fn_: usize,
}

impl Confusion {
    fn from_pairs(pairs: &[(bool, bool)]) -> Self {
        let mut c = Confusion::default();
        for &(tool, human) in pairs {
            match (tool, human) {
                (true, true) => c.tp += 1,
                (true, false) => c.fp += 1,
                (false, false) => c.tn += 1,
                (false, true) => c.fn_ += 1,
            }
        }
        c
    }

    /// Share of human "no problem" cases the tool flags.
    fn false_positive_rate(&self) -> Option<f64> {
        ratio(self.fp, self.fp + self.tn)
    }

    /// Share of human "problem" cases the tool misses.
    fn false_negative_rate(&self) -> Option<f64> {
        ratio(self.fn_, self.fn_ + self.tp)
    }
}

fn ratio(num: usize, den: usize) -> Option<f64> {
    (den > 0).then(|| num as f64 / den as f64)
}

/// 95 % Wilson score interval for `k` of `n`, published next to each rate.
fn wilson(k: usize, n: usize) -> Option<(f64, f64)> {
    if n == 0 {
        return None;
    }
    let (k, n, z) = (k as f64, n as f64, 1.96f64);
    let p = k / n;
    let denom = 1.0 + z * z / n;
    let centre = (p + z * z / (2.0 * n)) / denom;
    let half = z * ((p * (1.0 - p) + z * z / (4.0 * n)) / n).sqrt() / denom;
    Some(((centre - half).max(0.0), (centre + half).min(1.0)))
}

/// Kendall's tau-b between tool scores and human ratings (ties handled).
fn kendall_tau_b(xs: &[f64], ys: &[f64]) -> Option<f64> {
    assert_eq!(xs.len(), ys.len());
    let (mut concordant, mut discordant, mut ties_x, mut ties_y) = (0f64, 0f64, 0f64, 0f64);
    for i in 0..xs.len() {
        for j in (i + 1)..xs.len() {
            // `f64::signum` maps 0.0 to 1.0, so compare instead.
            let dx = xs[i].partial_cmp(&xs[j]).expect("no NaN");
            let dy = ys[i].partial_cmp(&ys[j]).expect("no NaN");
            use std::cmp::Ordering::Equal;
            match (dx == Equal, dy == Equal) {
                (true, true) => {}
                (true, false) => ties_x += 1.0,
                (false, true) => ties_y += 1.0,
                (false, false) if dx == dy => concordant += 1.0,
                (false, false) => discordant += 1.0,
            }
        }
    }
    let denom = ((concordant + discordant + ties_x) * (concordant + discordant + ties_y)).sqrt();
    (denom > 0.0).then(|| (concordant - discordant) / denom)
}

/// Cohen's kappa of two labellers over the same cases.
fn cohen_kappa(a: &[Judgement], b: &[Judgement]) -> Option<f64> {
    assert_eq!(a.len(), b.len());
    let n = a.len() as f64;
    if a.is_empty() {
        return None;
    }
    let observed = a.iter().zip(b).filter(|(x, y)| x == y).count() as f64 / n;
    let classes = [
        Judgement::Problem,
        Judgement::NoProblem,
        Judgement::Undecidable,
    ];
    let expected: f64 = classes
        .iter()
        .map(|c| {
            let pa = a.iter().filter(|x| *x == c).count() as f64 / n;
            let pb = b.iter().filter(|x| *x == c).count() as f64 / n;
            pa * pb
        })
        .sum();
    (expected < 1.0).then(|| (observed - expected) / (1.0 - expected))
}

/// Majority judgement of all labellers; a tie or a majority of
/// "undecidable" yields `Undecidable` (excluded from the rates, reported).
fn consensus(judgements: &[Judgement]) -> Judgement {
    let count = |j| judgements.iter().filter(|x| **x == j).count();
    let (p, n) = (count(Judgement::Problem), count(Judgement::NoProblem));
    let u = count(Judgement::Undecidable);
    if p > n && p > u {
        Judgement::Problem
    } else if n > p && n > u {
        Judgement::NoProblem
    } else {
        Judgement::Undecidable
    }
}

// ─── Integrity ───────────────────────────────────────────────────────────────

/// All checks that must hold before, during and after labelling. Returns
/// the violations so the unit tests can exercise them on synthetic input.
fn integrity_errors(prereg: &Preregistration, set_bytes: &[u8]) -> Vec<String> {
    let mut errors = Vec::new();
    let set = load_set(set_bytes);
    if prereg.format != "auditmysite.calibration-preregistration/1" {
        errors.push(format!("unknown preregistration format {}", prereg.format));
    }
    if set.format != "auditmysite.calibration-set/1" || set.version == 0 {
        errors.push(format!("unknown reference set format {}", set.format));
    }
    let registered = iso_date(&prereg.registered_at);

    // Registered contract versions match the code: a changed method needs a
    // new registration (and fresh holdout labels).
    for (signal, t) in &prereg.signals {
        match auditmysite::audit::signal_contract::signal_contract(signal) {
            Some(c) if c.version == t.contract_version => {}
            Some(c) => errors.push(format!(
                "{signal}: registered for contract v{}, code is v{}",
                t.contract_version, c.version
            )),
            None => errors.push(format!("{signal}: no signal contract in code")),
        }
    }

    let mut ids = HashSet::new();
    for case in &set.cases {
        if !ids.insert(case.id.as_str()) {
            errors.push(format!("duplicate case id {}", case.id));
        }
        if !case.url.starts_with("https://") && !case.url.starts_with("http://") {
            errors.push(format!("{}: url is not http(s)", case.id));
        }
        if case.selected_by.trim().is_empty() {
            errors.push(format!("{}: selected_by missing", case.id));
        }
        iso_date(&case.selected_at);
        for signal in &case.signals {
            if !prereg.signals.contains_key(signal) {
                errors.push(format!("{}: signal {signal} not registered", case.id));
            }
        }
        for label in &case.labels {
            if !case.signals.contains(&label.signal) {
                errors.push(format!(
                    "{}: label for {} which the case does not list",
                    case.id, label.signal
                ));
            }
            if !(1..=5).contains(&label.rating) {
                errors.push(format!("{}: rating {} not in 1..=5", case.id, label.rating));
            }
            if label.labeller.trim().is_empty() {
                errors.push(format!("{}: anonymous label", case.id));
            }
            // Thresholds first, labels after: no threshold was fitted to a
            // label that already existed.
            if iso_date(&label.labelled_at) < registered {
                errors.push(format!(
                    "{}: labelled {} before the registration of {}",
                    case.id, label.labelled_at, registered
                ));
            }
        }
    }

    // Once frozen, the set is what was frozen: no label edited after it was
    // known how the tool scores.
    if let Some(frozen) = &prereg.labels_sha256 {
        let actual = sha256_hex(set_bytes);
        if &actual != frozen {
            errors.push(format!(
                "reference set changed after freezing (frozen {frozen}, now {actual})"
            ));
        }
    }
    errors
}

#[test]
fn reference_set_and_preregistration_are_intact() {
    let prereg = load_prereg();
    let bytes = reference_set_bytes(&prereg);
    let errors = integrity_errors(&prereg, &bytes);
    assert!(
        errors.is_empty(),
        "calibration integrity:\n{}",
        errors.join("\n")
    );
}

/// Every signal contract that claims an ordering (a module score) is
/// registered; design_quality has no score, only review candidates.
#[test]
fn every_scored_signal_contract_is_registered() {
    let prereg = load_prereg();
    for c in auditmysite::audit::signal_contract::SIGNAL_CONTRACTS {
        if c.id == "design_quality" {
            continue;
        }
        assert!(prereg.signals.contains_key(c.id), "{} not registered", c.id);
    }
}

#[cfg(test)]
mod integrity_cases {
    use super::*;

    fn prereg(labels_sha256: Option<String>) -> Preregistration {
        let mut p = load_prereg();
        p.labels_sha256 = labels_sha256;
        p
    }

    fn set_with_label(labelled_at: &str, extra: &str) -> Vec<u8> {
        format!(
            r#"{{"format":"auditmysite.calibration-set/1","version":1,"notes":"",
            "cases":[{{"id":"c1","url":"https://example.com/","split":"holdout",
            "stratum":"positive","selected_by":"a","selected_at":"2026-10-27",
            "signals":["ux"],"labels":[{{"signal":"ux","labeller":"a",
            "labelled_at":"{labelled_at}","judgement":"problem","rating":2,
            "rationale":"no clear call to action"{extra}}}]}}]}}"#
        )
        .into_bytes()
    }

    #[test]
    fn a_label_older_than_the_registration_is_rejected() {
        let errors = integrity_errors(&prereg(None), &set_with_label("2026-10-01", ""));
        assert!(errors.iter().any(|e| e.contains("before the registration")));
        assert!(integrity_errors(&prereg(None), &set_with_label("2026-11-02", "")).is_empty());
    }

    #[test]
    fn editing_a_frozen_set_is_rejected() {
        let bytes = set_with_label("2026-11-02", "");
        let frozen = prereg(Some(sha256_hex(&bytes)));
        assert!(integrity_errors(&frozen, &bytes).is_empty());
        let edited = String::from_utf8(bytes)
            .unwrap()
            .replace("\"rating\":2", "\"rating\":4");
        let errors = integrity_errors(&frozen, edited.as_bytes());
        assert!(errors.iter().any(|e| e.contains("changed after freezing")));
    }

    /// A tool score smuggled into a label does not parse.
    #[test]
    #[should_panic(expected = "reference set parses")]
    fn a_label_carrying_a_tool_score_does_not_parse() {
        integrity_errors(
            &prereg(None),
            &set_with_label("2026-11-02", r#","score":42"#),
        );
    }

    #[test]
    fn a_changed_contract_version_needs_a_new_registration() {
        let mut p = prereg(None);
        p.signals.get_mut("ux").unwrap().contract_version = 99;
        let errors = integrity_errors(&p, &set_with_label("2026-11-02", ""));
        assert!(errors
            .iter()
            .any(|e| e.contains("registered for contract v99")));
    }
}

#[cfg(test)]
mod metric_cases {
    use super::*;

    #[test]
    fn confusion_rates() {
        let c = Confusion::from_pairs(&[
            (true, true),
            (true, false),
            (false, false),
            (false, false),
            (false, true),
        ]);
        assert_eq!(
            c,
            Confusion {
                tp: 1,
                fp: 1,
                tn: 2,
                fn_: 1
            }
        );
        assert!((c.false_positive_rate().unwrap() - 1.0 / 3.0).abs() < 1e-9);
        assert!((c.false_negative_rate().unwrap() - 0.5).abs() < 1e-9);
        assert_eq!(Confusion::default().false_positive_rate(), None);
    }

    #[test]
    fn wilson_interval_contains_the_point_estimate() {
        let (lo, hi) = wilson(3, 30).unwrap();
        assert!(lo < 0.1 && 0.1 < hi);
        assert!((lo - 0.0346).abs() < 1e-3 && (hi - 0.2562).abs() < 1e-3);
        assert_eq!(wilson(0, 0), None);
    }

    #[test]
    fn kendall_tau_b_extremes_and_ties() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((kendall_tau_b(&x, &x).unwrap() - 1.0).abs() < 1e-9);
        let rev = [4.0, 3.0, 2.0, 1.0];
        assert!((kendall_tau_b(&x, &rev).unwrap() + 1.0).abs() < 1e-9);
        // tau-b with ties in y: 5 concordant, 0 discordant, 1 tie in y.
        let tied = [1.0, 2.0, 2.0, 3.0];
        let t = kendall_tau_b(&x, &tied).unwrap();
        assert!((t - 5.0 / (6.0f64 * 5.0).sqrt()).abs() < 1e-9);
        assert_eq!(kendall_tau_b(&[1.0, 1.0], &[2.0, 3.0]), None);
    }

    #[test]
    fn cohen_kappa_perfect_and_chance() {
        use Judgement::*;
        let a = [Problem, NoProblem, Problem, NoProblem];
        assert!((cohen_kappa(&a, &a).unwrap() - 1.0).abs() < 1e-9);
        let b = [Problem, Problem, NoProblem, NoProblem];
        assert!(cohen_kappa(&a, &b).unwrap().abs() < 1e-9);
    }

    #[test]
    fn consensus_needs_a_majority() {
        use Judgement::*;
        assert_eq!(consensus(&[Problem, Problem, NoProblem]), Problem);
        assert_eq!(consensus(&[Problem, NoProblem]), Undecidable);
        assert_eq!(consensus(&[NoProblem, NoProblem]), NoProblem);
    }
}

// ─── Calibration run ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct SignalResult {
    signal: String,
    contract_version: u32,
    holdout_cases: usize,
    undecidable: usize,
    confusion: Confusion,
    false_positive_rate: Option<f64>,
    false_positive_rate_ci95: Option<(f64, f64)>,
    false_negative_rate: Option<f64>,
    false_negative_rate_ci95: Option<(f64, f64)>,
    kendall_tau_b: Option<f64>,
    inter_rater_kappa: Option<f64>,
    failures: Vec<String>,
}

/// Score of `signal` in the single-page JSON, the path users read.
fn module_score(json: &serde_json::Value, signal: &str) -> Option<f64> {
    json["pages"][0]["detail"]["modules"][signal]["score"].as_f64()
}

#[tokio::test]
#[ignore = "needs real Chrome, network and a frozen, labelled reference set"]
async fn heuristic_signals_meet_their_preregistered_thresholds() {
    use auditmysite::cli::Args;
    use auditmysite::output::UnifiedReport;
    use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
    use clap::Parser;

    let prereg = load_prereg();
    let bytes = reference_set_bytes(&prereg);
    let errors = integrity_errors(&prereg, &bytes);
    assert!(
        errors.is_empty(),
        "calibration integrity:\n{}",
        errors.join("\n")
    );
    assert!(
        prereg.labels_sha256.is_some(),
        "labels are not frozen: set `labels_sha256` in preregistration.json first \
         (docs/calibration.md, step 4)"
    );
    let set = load_set(&bytes);
    let today = chrono::Utc::now().date_naive();
    for case in set.cases.iter().filter(|c| c.split == Split::Holdout) {
        for label in &case.labels {
            let at = chrono::NaiveDate::parse_from_str(&label.labelled_at, "%Y-%m-%d").unwrap();
            assert!(
                (today - at).num_days() <= prereg.max_days_between_label_and_run as i64,
                "{}: label from {} is older than the registered window — relabel",
                case.id,
                label.labelled_at
            );
        }
    }

    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("Browser launch failed");

    // Audit each holdout case once; keep the JSON for every signal.
    let mut audited: BTreeMap<&str, serde_json::Value> = BTreeMap::new();
    for case in set.cases.iter().filter(|c| c.split == Split::Holdout) {
        let page = manager.new_page().await.expect("New page failed");
        manager
            .navigate(&page, &case.url)
            .await
            .expect("case reachable");
        let args = Args::parse_from(["auditmysite", &case.url, "--full"]);
        let config = PipelineConfig::from_args_and_config(&args, None);
        let result = audit_page(&page, &case.url, &config, &manager).await;
        let _ = page.close().await;
        let (report, _) = result.expect("audit");
        let normalized = auditmysite::audit::normalize(&report);
        let json = UnifiedReport::single(&normalized, &report)
            .to_json(false)
            .expect("json");
        audited.insert(&case.id, serde_json::from_str(&json).unwrap());
    }

    let req = &prereg.requirements;
    let mut results = Vec::new();
    for (signal, t) in &prereg.signals {
        let cases: Vec<&Case> = set
            .cases
            .iter()
            .filter(|c| c.split == Split::Holdout && c.signals.contains(signal))
            .collect();
        let mut failures = Vec::new();
        let count = |s| cases.iter().filter(|c| c.stratum == s).count();
        if cases.len() < req.min_holdout_cases_per_signal
            || count(Stratum::Positive) < req.min_holdout_positive
            || count(Stratum::Negative) < req.min_holdout_negative
            || count(Stratum::Borderline) < req.min_holdout_borderline
        {
            failures.push("holdout coverage below the registered minimum".to_string());
        }

        let (mut pairs, mut scores, mut ratings) = (Vec::new(), Vec::new(), Vec::new());
        let (mut first, mut second) = (Vec::new(), Vec::new());
        let mut undecidable = 0;
        for case in &cases {
            let labels: Vec<&Label> = case.labels.iter().filter(|l| &l.signal == signal).collect();
            if labels.len() < req.min_labellers_per_case {
                failures.push(format!("{}: {} labeller(s)", case.id, labels.len()));
                continue;
            }
            first.push(labels[0].judgement);
            second.push(labels[1].judgement);
            let Some(score) = module_score(&audited[case.id.as_str()], signal) else {
                failures.push(format!("{}: no {signal} score", case.id));
                continue;
            };
            let mean_rating =
                labels.iter().map(|l| l.rating as f64).sum::<f64>() / labels.len() as f64;
            scores.push(score);
            ratings.push(mean_rating);
            match consensus(&labels.iter().map(|l| l.judgement).collect::<Vec<_>>()) {
                Judgement::Undecidable => undecidable += 1,
                human => pairs.push((score < t.problem_cutoff as f64, human == Judgement::Problem)),
            }
        }

        let confusion = Confusion::from_pairs(&pairs);
        let kappa = cohen_kappa(&first, &second);
        let tau = kendall_tau_b(&scores, &ratings);
        if kappa.is_none_or(|k| k < req.min_inter_rater_kappa) {
            failures.push(format!(
                "inter-rater kappa {kappa:?} — the labels cannot judge the tool"
            ));
        }
        let fpr = confusion.false_positive_rate();
        let fnr = confusion.false_negative_rate();
        if fpr.is_none_or(|r| r > t.max_false_positive_rate) {
            failures.push(format!(
                "false-positive rate {fpr:?} > {}",
                t.max_false_positive_rate
            ));
        }
        if fnr.is_none_or(|r| r > t.max_false_negative_rate) {
            failures.push(format!(
                "false-negative rate {fnr:?} > {}",
                t.max_false_negative_rate
            ));
        }
        if tau.is_none_or(|v| v < t.min_kendall_tau_b) {
            failures.push(format!("Kendall tau-b {tau:?} < {}", t.min_kendall_tau_b));
        }
        results.push(SignalResult {
            signal: signal.clone(),
            contract_version: t.contract_version,
            holdout_cases: cases.len(),
            undecidable,
            confusion,
            false_positive_rate: fpr,
            false_positive_rate_ci95: wilson(confusion.fp, confusion.fp + confusion.tn),
            false_negative_rate: fnr,
            false_negative_rate_ci95: wilson(confusion.fn_, confusion.fn_ + confusion.tp),
            kendall_tau_b: tau,
            inter_rater_kappa: kappa,
            failures,
        });
    }

    let out = serde_json::json!({
        "format": "auditmysite.calibration-results/1",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "run_at": today.to_string(),
        "preregistration": { "registered_at": prereg.registered_at, "labels_sha256": prereg.labels_sha256 },
        "signals": results,
    });
    let reports = root().join("reports");
    std::fs::create_dir_all(&reports).ok();
    let path = Path::new(&reports).join("calibration-results.json");
    std::fs::write(&path, serde_json::to_string_pretty(&out).unwrap()).unwrap();
    println!("calibration results written to {}", path.display());

    let failed: Vec<String> = results
        .iter()
        .flat_map(|r| r.failures.iter().map(move |f| format!("{}: {f}", r.signal)))
        .collect();
    assert!(
        failed.is_empty(),
        "calibration missed:\n{}",
        failed.join("\n")
    );
}
