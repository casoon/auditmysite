//! #873: the six privacy checks end to end through real Chrome, against the
//! browser-level fixture corpus `tests/fixtures/privacy/browser_cases.json`.
//!
//! The pure corpus (`privacy_findings_corpus.rs`) feeds hand-built captures
//! to `analyze_privacy`. This one serves real pages and lets Chrome make the
//! requests: scripts, images, stylesheets and iframes to hosts that are
//! *aliases* of well-known providers (`www.google-analytics.com`, ...), cookies
//! from response headers and from script, a first-party banner element,
//! look-alike hosts. `collect_first_visit` and `analyze_privacy` run exactly as
//! in an audit.
//!
//! **Aliases.** Chrome is started through a wrapper script (precedent:
//! `transport_redirect_browser.rs`) with
//! `--host-resolver-rules="MAP * 127.0.0.1:<port>, EXCLUDE 127.0.0.1"`: every
//! host name resolves to the one local fixture server, on its port. Nothing in
//! a case can reach the real provider or the internet, and no product code is
//! relaxed. The server answers by path only; unknown paths are 404, which is
//! all a tracker request needs to be observed.
//!
//! **Expectations** are hand-written from the rules in `src/privacy/findings.rs`
//! (each case states its rule in `why`), never from a run. `truth` lists the
//! tracking providers a case really loads before consent, independent of what
//! the tool is documented to see: the known blind-spot cases carry a `truth`
//! the tool misses by design, so the recall printed by the run includes them.
//!
//! Not coverable at browser level: `not_measured` for check 2 (jar or storage
//! unreadable) and check 6 (document probe unreadable) - Chrome does not let a
//! page make its own jar unreadable; the pure corpus covers both.

#![cfg(unix)]

mod common;

use std::collections::BTreeSet;
use std::time::Duration;

use auditmysite::audit::collection::CollectionState;
use auditmysite::privacy::{
    analyze_privacy, collect_first_visit, CheckOutcome, FirstVisitOptions, PrivacyAnalysis,
    PrivacyCheck,
};
use auditmysite::{BrowserManager, BrowserOptions};
use common::routing_server::{serve_routes, Route, Transport};
use serde_json::Value;

const SITE: &str = "www.corpus-site.test";

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/privacy/browser_cases.json"))
        .expect("valid browser corpus")
}

fn strings(case: &Value, key: &str) -> Vec<String> {
    case[key]
        .as_array()
        .map(|a| a.iter().map(|v| v.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

const DEFAULT_FOOTER: &str =
    "<a href='/datenschutz'>Datenschutz</a> <a href='/impressum'>Impressum</a>";

fn page(case: &Value) -> String {
    let footer = case["footer"].as_str().unwrap_or(DEFAULT_FOOTER);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Corpus</title>\
         <link rel=\"icon\" href=\"data:,\"></head><body>{}<main><h1>Corpus page</h1>\
         <p>Fixture.</p></main><footer>{}</footer></body></html>",
        case["body"].as_str().unwrap_or(""),
        footer
    )
}

fn case_path(index: usize) -> String {
    format!("/case/{index}")
}

fn routes() -> Vec<Route> {
    cases()
        .iter()
        .enumerate()
        .map(|(i, case)| {
            let mut route = Route::new(&case_path(i), 200, page(case));
            for cookie in strings(case, "set_cookie") {
                route = route.header("Set-Cookie", &cookie);
            }
            route
        })
        .collect()
}

/// A script that starts the detected Chrome with every host name mapped to
/// the fixture server.
fn aliasing_chrome(dir: &std::path::Path, port: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let real = auditmysite::browser::find_chrome(None)
        .expect("chrome")
        .path;
    let script = dir.join("chrome-alias-hosts.sh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nexec \"{}\" --ignore-certificate-errors --host-resolver-rules=\"MAP * 127.0.0.1:{port}, EXCLUDE 127.0.0.1, EXCLUDE localhost\" \"$@\"\n",
            real.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script.to_string_lossy().into_owned()
}

fn options() -> FirstVisitOptions {
    FirstVisitOptions {
        idle_quiet: Duration::from_millis(300),
        // Short: the "never goes quiet" case waits this long.
        idle_max: Duration::from_secs(3),
        settle: Duration::from_millis(1200),
    }
}

fn letter(outcome: CheckOutcome) -> char {
    match outcome {
        CheckOutcome::Found => 'F',
        CheckOutcome::NotFound => 'N',
        CheckOutcome::NotApplicable => 'A',
        CheckOutcome::NotMeasured => 'M',
    }
}

fn outcomes(analysis: &PrivacyAnalysis) -> String {
    analysis.checks.iter().map(|c| letter(c.outcome)).collect()
}

fn finding_keys(analysis: &PrivacyAnalysis) -> Vec<String> {
    let mut keys: Vec<String> = analysis
        .findings
        .iter()
        .map(|f| {
            let json = serde_json::to_value(f).unwrap();
            let category = f
                .provider
                .as_ref()
                .map(|p| p.category.clone())
                .unwrap_or_default();
            format!(
                "{}|{}|{}|{}",
                json["kind"].as_str().unwrap(),
                json["confidence"].as_str().unwrap(),
                category,
                json["document"].as_str().unwrap_or("")
            )
        })
        .collect();
    keys.sort();
    keys
}

fn wanted_keys(case: &Value) -> Vec<String> {
    let mut want: Vec<String> = case["expect"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            f.as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>()
                .join("|")
        })
        .collect();
    want.sort();
    want
}

// ── Offline: the corpus is well-formed and covers what it claims ───────────

#[test]
fn browser_cases_cover_every_check_and_outcome_they_can() {
    let mut seen = BTreeSet::new();
    for case in cases() {
        let letters = case["expect"]["outcomes"].as_str().unwrap();
        assert_eq!(letters.len(), PrivacyCheck::ALL.len(), "{}", case["name"]);
        for (i, l) in letters.chars().enumerate() {
            seen.insert((i, l));
        }
        assert!(
            case["why"].as_str().is_some_and(|w| !w.is_empty()),
            "{}: every case states the rule it tests",
            case["name"]
        );
    }
    for (i, check) in PrivacyCheck::ALL.iter().enumerate() {
        for outcome in ['F', 'N', 'A'] {
            assert!(
                seen.contains(&(i, outcome)),
                "{check:?} lacks a {outcome} case"
            );
        }
    }
    // `not_measured` is reachable end to end for the request-based checks
    // only (see the module docs).
    for i in [0, 2, 3, 4] {
        assert!(
            seen.contains(&(i, 'M')),
            "{:?} lacks an M case",
            PrivacyCheck::ALL[i]
        );
    }
}

#[test]
fn browser_case_names_are_unique() {
    let names: BTreeSet<String> = cases()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names.len(), cases().len());
}

// ── Chrome ─────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test privacy_quality_browser -- --ignored`"]
async fn every_browser_case_yields_what_the_rules_say() {
    let fixture = serve_routes(routes(), Transport::Https);
    let port = fixture.url.rsplit(':').next().unwrap().to_string();
    let dir = std::env::temp_dir().join(format!("amsite-873-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let manager = BrowserManager::with_options(BrowserOptions {
        chrome_path: Some(aliasing_chrome(&dir, &port)),
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch");

    let mut problems = Vec::new();
    let (mut truth_total, mut truth_found, mut false_positives) = (0usize, 0usize, 0usize);
    for (index, case) in cases().iter().enumerate() {
        let name = case["name"].as_str().unwrap();
        let url = format!("https://{SITE}{}", case_path(index));
        let before = fixture.served();
        let collected = collect_first_visit(&manager, &url, options()).await;
        let never_quiet = name.contains("never goes quiet");
        let want_state = if never_quiet {
            CollectionState::Partial
        } else {
            CollectionState::Measured
        };
        if collected.state() != want_state {
            problems.push(format!(
                "{name}: collection {:?} {collected:?}",
                collected.state()
            ));
            continue;
        }
        if fixture.served() == before {
            problems.push(format!("{name}: the fixture answered nothing"));
            continue;
        }
        let capture = collected.value().expect("capture");

        // Every host the case means to contact was really requested: a
        // negative result is worth nothing if the request never happened.
        for host in strings(case, "requested") {
            if !capture.requests.iter().any(|r| r.host == host) {
                problems.push(format!(
                    "{name}: no request to {host} (hosts: {:?})",
                    capture.requests.iter().map(|r| &r.host).collect::<Vec<_>>()
                ));
            }
        }

        let analysis = analyze_privacy(&collected);
        let want = case["expect"]["outcomes"].as_str().unwrap();
        if outcomes(&analysis) != want {
            problems.push(format!(
                "{name}: outcomes {} instead of {want}",
                outcomes(&analysis)
            ));
        }
        let (got_keys, want_keys) = (finding_keys(&analysis), wanted_keys(case));
        false_positives += got_keys.iter().filter(|k| !want_keys.contains(k)).count();
        if got_keys != want_keys {
            problems.push(format!(
                "{name}: findings {got_keys:?} instead of {want_keys:?}"
            ));
        }
        let want_unclassified = strings(case, "unclassified");
        if !want_unclassified.is_empty() {
            let mut got = analysis.unclassified_hosts.clone();
            got.retain(|h| want_unclassified.contains(h));
            if got != want_unclassified {
                problems.push(format!(
                    "{name}: unclassified {:?} instead of {want_unclassified:?}",
                    analysis.unclassified_hosts
                ));
            }
        }

        // Recall over the trackers a case really loads.
        let found: BTreeSet<String> = analysis
            .findings
            .iter()
            .filter(|f| {
                f.kind == auditmysite::privacy::PrivacyFindingKind::TrackingRequestBeforeConsent
            })
            .filter_map(|f| f.provider.as_ref().map(|p| p.id.clone()))
            .collect();
        for id in strings(case, "truth") {
            truth_total += 1;
            if found.contains(&id) {
                truth_found += 1;
            }
        }
    }

    println!(
        "privacy browser corpus: {} cases, tracking providers present {truth_total}, reported {truth_found} (recall {:.0} %), unexpected findings {false_positives}",
        cases().len(),
        100.0 * truth_found as f64 / truth_total as f64
    );
    fixture.stop();
    let _ = manager.close().await;
    let _ = std::fs::remove_dir_all(&dir);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// A page that cannot be reached is not measured - never a clean page.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test privacy_quality_browser -- --ignored`"]
async fn an_unreachable_page_reads_not_measured_in_every_check() {
    let fixture = serve_routes(routes(), Transport::Https);
    let port = fixture.url.rsplit(':').next().unwrap().to_string();
    let dir = std::env::temp_dir().join(format!("amsite-873-down-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let manager = BrowserManager::with_options(BrowserOptions {
        chrome_path: Some(aliasing_chrome(&dir, &port)),
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch");
    // 127.0.0.1 is excluded from the mapping; port 9 (discard) refuses.
    let collected = collect_first_visit(&manager, "http://127.0.0.1:9/", options()).await;
    assert_eq!(collected.state(), CollectionState::Failed, "{collected:?}");
    let analysis = analyze_privacy(&collected);
    assert_eq!(outcomes(&analysis), "MMMMMM");
    assert!(analysis.findings.is_empty());
    fixture.stop();
    let _ = manager.close().await;
    let _ = std::fs::remove_dir_all(&dir);
}
