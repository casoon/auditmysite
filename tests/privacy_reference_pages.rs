//! #873: hand-labelled live reference pages for the privacy findings.
//!
//! Labels: `tests/fixtures/privacy/reference_pages.json`. Each page lists the
//! findings a person expects, written after reading the page's raw first-visit
//! network log and checking it independently (see `docs/privacy-findings.md`,
//! "Reference pages"). The log each label was written from is frozen in
//! `reference_captures.json`; `reference_pages.json.sha256` seals the labels
//! (`scripts/privacy-reference.py seal|check`).
//!
//! * Offline (runs everywhere): the seals hold, and `analyze_privacy` over the
//!   frozen captures reproduces the labels - a regression guard for the rules
//!   and for label typos. This is NOT a measurement of the live pages.
//! * `--ignored` (Chrome and network): visits every page now, scores the
//!   findings against the labels per finding kind, prints precision and
//!   recall, and fails on any false positive. Live pages change: re-label the
//!   page from a fresh raw capture before suspecting the tool - but never tune
//!   a label to the tool's output.
//!
//! `PRIVACY_DUMP_URLS` (comma-separated) with `PRIVACY_DUMP_DIR` makes the
//! `dump_raw_first_visit_capture` helper write the raw capture of any URL, the
//! starting point for labelling a new page.

use std::collections::{BTreeMap, BTreeSet};

use auditmysite::audit::collection::{Collected, CollectionState};
use auditmysite::privacy::{
    analyze_privacy, collect_first_visit, FirstVisitCapture, FirstVisitOptions, PolicyDocument,
    PrivacyAnalysis, PrivacyCheck, PrivacyFindingKind,
};
use auditmysite::{BrowserManager, BrowserOptions};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const LABELS: &str = include_str!("fixtures/privacy/reference_pages.json");
const LABELS_SEAL: &str = include_str!("fixtures/privacy/reference_pages.json.sha256");
const CAPTURES: &str = include_str!("fixtures/privacy/reference_captures.json");

#[derive(Deserialize)]
struct LabelSet {
    sites: Vec<Site>,
}

#[derive(Deserialize)]
struct Site {
    id: String,
    url: String,
    /// SHA-256 of the capture the labels were written from, as stored.
    capture_sha256: String,
    /// The checks whose complete answer the labels give.
    checks_labelled: Vec<String>,
    expected: Vec<Expected>,
    /// Finding keys the labels deliberately do not decide (reason in the file).
    #[serde(default)]
    unlabelled: Vec<Expected>,
}

#[derive(Deserialize)]
struct Expected {
    /// `<kind>` or `<kind>|<provider id>` or `<kind>|<document>`.
    key: String,
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn labels() -> LabelSet {
    serde_json::from_str(LABELS).expect("reference_pages.json parses")
}

fn captures() -> BTreeMap<String, serde_json::Value> {
    serde_json::from_str(CAPTURES).expect("reference_captures.json parses")
}

fn kind_name(kind: PrivacyFindingKind) -> &'static str {
    use PrivacyFindingKind as K;
    match kind {
        K::TrackingRequestBeforeConsent => "tracking_request_before_consent",
        K::NonEssentialStorageBeforeConsent => "non_essential_storage_before_consent",
        K::VisitorIpTransfer => "visitor_ip_transfer",
        K::ThirdCountryProvider => "third_country_provider",
        K::NoConsentPlatformDetected => "no_consent_platform_detected",
        K::PolicyLinkMissing => "policy_link_missing",
        K::PolicyLinkUnusable => "policy_link_unusable",
    }
}

fn check_name(check: PrivacyCheck) -> &'static str {
    match check {
        PrivacyCheck::TrackingRequests => "tracking_requests",
        PrivacyCheck::NonEssentialStorage => "non_essential_storage",
        PrivacyCheck::VisitorIpTransfer => "visitor_ip_transfer",
        PrivacyCheck::ThirdCountryProvider => "third_country_provider",
        PrivacyCheck::ConsentPlatform => "consent_platform",
        PrivacyCheck::PolicyLinks => "policy_links",
    }
}

fn check_of(kind: &str) -> &'static str {
    match kind {
        "tracking_request_before_consent" => "tracking_requests",
        "non_essential_storage_before_consent" => "non_essential_storage",
        "visitor_ip_transfer" => "visitor_ip_transfer",
        "third_country_provider" => "third_country_provider",
        "no_consent_platform_detected" => "consent_platform",
        "policy_link_missing" | "policy_link_unusable" => "policy_links",
        other => panic!("unknown kind {other}"),
    }
}

/// `<kind>|<provider id or document>` of every finding.
fn finding_keys(analysis: &PrivacyAnalysis) -> BTreeSet<String> {
    analysis
        .findings
        .iter()
        .map(|f| {
            let qualifier = match (&f.provider, f.document) {
                (Some(p), _) => p.id.clone(),
                (None, Some(PolicyDocument::Privacy)) => "privacy".to_string(),
                (None, Some(PolicyDocument::Imprint)) => "imprint".to_string(),
                (None, None) => String::new(),
            };
            if qualifier.is_empty() {
                kind_name(f.kind).to_string()
            } else {
                format!("{}|{qualifier}", kind_name(f.kind))
            }
        })
        .collect()
}

#[derive(Default, Debug)]
struct Tally {
    tp: usize,
    fp: usize,
    fn_: usize,
}

impl Tally {
    fn precision(&self) -> Option<f64> {
        (self.tp + self.fp > 0).then(|| self.tp as f64 / (self.tp + self.fp) as f64)
    }
    fn recall(&self) -> Option<f64> {
        (self.tp + self.fn_ > 0).then(|| self.tp as f64 / (self.tp + self.fn_) as f64)
    }
}

fn pct(value: Option<f64>) -> String {
    value.map_or("n/a".to_string(), |v| format!("{:.0} %", v * 100.0))
}

/// Score `got` against the labels of `site`, per finding kind, within the
/// checks the site's labels cover. Returns the per-site problems.
fn score(
    site: &Site,
    got: &BTreeSet<String>,
    per_kind: &mut BTreeMap<String, Tally>,
) -> Vec<String> {
    let mut problems = Vec::new();
    let expected: BTreeSet<String> = site.expected.iter().map(|e| e.key.clone()).collect();
    let in_scope = |key: &str| {
        let kind = key.split('|').next().unwrap();
        site.checks_labelled.iter().any(|c| c == check_of(kind))
            && !site.unlabelled.iter().any(|u| u.key == key)
    };
    for key in got.iter().filter(|k| in_scope(k)) {
        let kind = key.split('|').next().unwrap().to_string();
        if expected.contains(key) {
            per_kind.entry(kind).or_default().tp += 1;
        } else {
            per_kind.entry(kind).or_default().fp += 1;
            problems.push(format!("{}: FALSE POSITIVE {key}", site.id));
        }
    }
    for key in expected.iter().filter(|k| !got.contains(*k)) {
        let kind = key.split('|').next().unwrap().to_string();
        per_kind.entry(kind).or_default().fn_ += 1;
        problems.push(format!("{}: missed {key}", site.id));
    }
    problems
}

fn print_table(per_kind: &BTreeMap<String, Tally>) -> (Tally, Tally) {
    println!(
        "{:<40} {:>3} {:>3} {:>3}  precision  recall",
        "finding kind", "TP", "FP", "FN"
    );
    let (mut all, mut providers) = (Tally::default(), Tally::default());
    for (kind, t) in per_kind {
        println!(
            "{kind:<40} {:>3} {:>3} {:>3}  {:>9}  {:>6}",
            t.tp,
            t.fp,
            t.fn_,
            pct(t.precision()),
            pct(t.recall())
        );
        all.tp += t.tp;
        all.fp += t.fp;
        all.fn_ += t.fn_;
        if matches!(
            kind.as_str(),
            "tracking_request_before_consent"
                | "non_essential_storage_before_consent"
                | "visitor_ip_transfer"
        ) {
            providers.tp += t.tp;
            providers.fp += t.fp;
            providers.fn_ += t.fn_;
        }
    }
    println!(
        "{:<40} {:>3} {:>3} {:>3}  {:>9}  {:>6}",
        "all findings",
        all.tp,
        all.fp,
        all.fn_,
        pct(all.precision()),
        pct(all.recall())
    );
    println!(
        "{:<40} {:>3} {:>3} {:>3}  {:>9}  {:>6}",
        "provider findings (tracker/storage/IP)",
        providers.tp,
        providers.fp,
        providers.fn_,
        pct(providers.precision()),
        pct(providers.recall())
    );
    (all, providers)
}

// ── Offline ────────────────────────────────────────────────────────────

#[test]
fn labels_are_sealed_and_every_site_has_its_frozen_capture() {
    assert_eq!(
        sha256(LABELS.as_bytes()),
        LABELS_SEAL.trim(),
        "reference_pages.json changed without `scripts/privacy-reference.py seal` - labels are frozen"
    );
    let captures = captures();
    let set = labels();
    assert!(!set.sites.is_empty());
    for site in &set.sites {
        let capture = captures
            .get(&site.id)
            .unwrap_or_else(|| panic!("{}: no frozen capture", site.id));
        let canonical = serde_json::to_string(capture).unwrap();
        assert_eq!(
            sha256(canonical.as_bytes()),
            site.capture_sha256,
            "{}: capture differs from the one the labels were written from",
            site.id
        );
        assert!(!site.checks_labelled.is_empty(), "{}", site.id);
    }
}

/// The rules, replayed over the frozen captures, give the labels. Only a
/// regression guard: the captures are the evidence the labels were written
/// from, not a fresh measurement.
#[test]
fn rules_replayed_over_the_frozen_captures_give_the_labels() {
    let captures = captures();
    let mut per_kind = BTreeMap::new();
    let mut problems = Vec::new();
    for site in labels().sites {
        let capture: FirstVisitCapture = serde_json::from_value(captures[&site.id].clone())
            .unwrap_or_else(|e| panic!("{}: capture parses: {e}", site.id));
        assert!(
            capture.network_idle,
            "{}: label from a non-idle capture",
            site.id
        );
        let analysis = analyze_privacy(&Collected::Measured(capture));
        problems.extend(score(&site, &finding_keys(&analysis), &mut per_kind));
    }
    print_table(&per_kind);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

// ── Live ───────────────────────────────────────────────────────────────

async fn browser() -> BrowserManager {
    BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch")
}

#[tokio::test]
#[ignore = "needs real Chrome and network; run before a release"]
async fn live_reference_pages_have_no_false_positive_and_recall_at_target() {
    let manager = browser().await;
    let mut per_kind = BTreeMap::new();
    let mut problems = Vec::new();
    let mut unmeasured = Vec::new();
    for site in labels().sites {
        let collected =
            collect_first_visit(&manager, &site.url, FirstVisitOptions::default()).await;
        if collected.state() != CollectionState::Measured {
            unmeasured.push(format!("{}: {:?}", site.url, collected.state()));
            continue;
        }
        let analysis = analyze_privacy(&collected);
        println!(
            "{} -> {} finding(s), checks {}",
            site.url,
            analysis.findings.len(),
            analysis
                .checks
                .iter()
                .map(|c| format!("{}={:?}", check_name(c.check), c.outcome))
                .collect::<Vec<_>>()
                .join(" ")
        );
        problems.extend(score(&site, &finding_keys(&analysis), &mut per_kind));
    }
    let (all, providers) = print_table(&per_kind);
    let _ = manager.close().await;

    assert!(
        unmeasured.is_empty(),
        "pages not measured (no number for them, not a pass): {unmeasured:?}"
    );
    assert_eq!(all.fp, 0, "false positives:\n{}", problems.join("\n"));
    assert!(
        providers.recall().is_none_or(|r| r >= 0.9),
        "recall below the 90 % target:\n{}",
        problems.join("\n")
    );
}

/// Writes the raw first-visit capture of every URL in `PRIVACY_DUMP_URLS`
/// (comma-separated) to `PRIVACY_DUMP_DIR/<host>.json`. Labelling starts here.
#[tokio::test]
#[ignore = "labelling helper; needs Chrome, network and PRIVACY_DUMP_URLS / PRIVACY_DUMP_DIR"]
async fn dump_raw_first_visit_capture() {
    let urls = std::env::var("PRIVACY_DUMP_URLS").expect("PRIVACY_DUMP_URLS");
    let dir =
        std::path::PathBuf::from(std::env::var("PRIVACY_DUMP_DIR").expect("PRIVACY_DUMP_DIR"));
    std::fs::create_dir_all(&dir).unwrap();
    let manager = browser().await;
    for url in urls.split(',').map(str::trim).filter(|u| !u.is_empty()) {
        let collected = collect_first_visit(&manager, url, FirstVisitOptions::default()).await;
        println!("{url}: {:?}", collected.state());
        if let Some(capture) = collected.value() {
            let name = format!("{}.json", capture.page_host);
            std::fs::write(
                dir.join(name),
                serde_json::to_string_pretty(capture).unwrap(),
            )
            .unwrap();
        }
    }
    let _ = manager.close().await;
}
