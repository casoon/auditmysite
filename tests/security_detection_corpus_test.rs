//! Diff-based accuracy test: security-header detection corpus against a real
//! `analyze_security` run (#558, mirrors #555's WCAG-corpus accuracy test).
//!
//! Unlike the WCAG corpus, this doesn't need Chrome/CDP at all —
//! `analyze_security(url)` is a standalone async fn that does its own HTTP
//! fetches (`reqwest`), so this test runs as a normal `cargo test` (no
//! `#[ignore]` gate, no browser dependency).
//!
//! Each case is served by the path-routing fixture server
//! (`tests/common/routing_server.rs`), over plain HTTP or over HTTPS with a
//! certificate from the corpus-only CA, and the real `SecurityAnalysis` is
//! compared strictly against `<case>.expected.json` (`common::corpus_diff`).
//! Besides the shared fields, a case may carry a `fixture` object:
//!
//! - `transport`: `"http"` (default), `"https"`, `"https_untrusted"`;
//! - `raw_response_headers`: `[[name, value], ...]` added to the page's
//!   response after `response_headers`, in order — for repeated headers and
//!   name casing a JSON object cannot express;
//! - `page_body`: the HTML at `/` (default: a document without assets);
//! - `page_get_status`: status of `GET /` while `HEAD /` stays 200;
//! - `page_status`: status of `/` for every method;
//! - `routes`: further `{path, method?, status?, headers?, body}` resources.
//!
//! `{{UNREACHABLE}}` in a body becomes a base URL nothing listens on.
//!
//! Issues of context-dependent headers (COOP, CORP) count as needs_review,
//! every other issue as a violation.
//!
//! What counts as "ran": header checks run when the header fetch succeeded
//! (`analyze_security` returns `Ok`); a failed fetch is an error, never a
//! clean result. The source-map check runs when its own collection
//! (`security.source_maps`) was measured; a failed or partial probe without
//! a leak is "not run".

mod common;

use auditmysite::audit::collection::{CollectionLog, CollectionState};
use auditmysite::security::{analyze_security_trusting, HeaderTier};
use common::corpus_diff::{
    diff_case, render_report, DiffRules, FindingClass, Observation, ObservedFinding, ObservedRun,
    RunState,
};
use common::detection_corpus::{
    load_corpus_dir, load_structurally_deferred, nonwcag_detection_corpus_dir,
};
use common::routing_server::{corpus_ca_pem, serve_routes, unreachable_origin, Route, Transport};
use serde::Deserialize;

const SOURCE_MAP_ID: &str = "Source Map:public_source_map";
const DEFAULT_PAGE: &str = "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Security corpus fixture</title></head><body><main><h1>Security corpus fixture</h1></main></body></html>";

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSpec {
    #[serde(default)]
    transport: Option<String>,
    #[serde(default)]
    raw_response_headers: Vec<(String, String)>,
    #[serde(default)]
    page_body: Option<String>,
    #[serde(default)]
    page_get_status: Option<u16>,
    #[serde(default)]
    page_status: Option<u16>,
    #[serde(default)]
    routes: Vec<RouteSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteSpec {
    path: String,
    #[serde(default)]
    method: Option<String>,
    #[serde(default = "ok")]
    status: u16,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default)]
    body: String,
}

fn ok() -> u16 {
    200
}

#[derive(Debug, Deserialize)]
struct CaseFile {
    #[serde(default)]
    fixture: FixtureSpec,
}

fn fixture_spec(path: &std::path::Path) -> FixtureSpec {
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str::<CaseFile>(&text)
        .unwrap_or_else(|e| panic!("invalid `fixture` in {}: {e}", path.display()))
        .fixture
}

fn transport(spec: &FixtureSpec) -> Transport {
    match spec.transport.as_deref().unwrap_or("http") {
        "http" => Transport::Http,
        "https" => Transport::Https,
        "https_untrusted" => Transport::HttpsUntrusted,
        other => panic!("unknown transport '{other}'"),
    }
}

fn routes(spec: &FixtureSpec, headers: &std::collections::BTreeMap<String, String>) -> Vec<Route> {
    let unreachable = unreachable_origin();
    let fill = |body: &str| body.replace("{{UNREACHABLE}}", &unreachable);
    let mut routes: Vec<Route> = spec
        .routes
        .iter()
        .map(|r| {
            let mut route = Route::new(&r.path, r.status, fill(&r.body));
            route.method = r.method.clone();
            route.headers = r.headers.clone();
            route
        })
        .collect();
    if let Some(status) = spec.page_get_status {
        routes.push(Route::new("/", status, "<html><body>error</body></html>").method("GET"));
    }
    let mut page = Route::new(
        "/",
        spec.page_status.unwrap_or(200),
        fill(spec.page_body.as_deref().unwrap_or(DEFAULT_PAGE)),
    );
    page.headers = headers
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .chain(spec.raw_response_headers.iter().cloned())
        .collect();
    routes.push(page);
    routes
}

/// The source-map check's own collection record, as a run state.
fn source_map_run(log: CollectionLog, leaked: bool) -> RunState {
    let record = log
        .into_records()
        .into_iter()
        .find(|r| r.collector == "security.source_maps");
    match record.map(|r| (r.state, r.reason_code)) {
        Some((CollectionState::Measured | CollectionState::Empty, _)) => {
            RunState::Ran { findings: 0 }
        }
        Some((CollectionState::Partial, _)) if leaked => RunState::Ran { findings: 0 },
        Some((state, reason)) => RunState::NotRun(format!("{state:?} {reason:?}")),
        None => RunState::NotRun("no security.source_maps record".to_string()),
    }
}

#[tokio::test]
async fn security_detection_corpus_matches_real_analyze_security_run() {
    let corpus_dir = nonwcag_detection_corpus_dir("security");
    let cases = load_corpus_dir(&corpus_dir);
    assert!(
        load_structurally_deferred(&corpus_dir).is_empty(),
        "deferred entries are not accepted (#783)"
    );
    assert!(
        !cases.is_empty(),
        "security detection corpus is empty — nothing to verify (expected at least the seed cases)"
    );

    let all_ids = auditmysite::security::all_security_check_ids();
    // Header findings concern the whole response, never an element.
    let diff_rules = DiffRules {
        page_level: all_ids.iter().cloned().collect(),
        run_counts_findings: false,
        ..Default::default()
    };
    let ca = reqwest::Certificate::from_pem(&corpus_ca_pem()).expect("corpus CA");
    let mut diffs = Vec::new();

    for case in &cases {
        let spec = fixture_spec(case.source_file.as_deref().unwrap());
        let transport = transport(&spec);
        let fixture = serve_routes(routes(&spec, &case.response_headers), transport);

        let log = CollectionLog::new();
        let result = analyze_security_trusting(&fixture.url, &log, std::slice::from_ref(&ca)).await;
        fixture.stop();

        // Before reading anything into the diff: did the fixture server
        // actually answer? Without this a transport problem of the fixture
        // arrives as a detection-accuracy mismatch (plan 48). The untrusted
        // certificate case is the one where the handshake must fail.
        if transport != Transport::HttpsUntrusted {
            assert!(
                fixture.served() > 0,
                "case '{}': the fixture server never delivered a response. Transport errors: {:?}",
                case.case,
                fixture.transport_errors(),
            );
            assert!(
                fixture.transport_errors().is_empty(),
                "case '{}': fixture server transport errors: {:?}",
                case.case,
                fixture.transport_errors(),
            );
        }

        let obs = match result {
            Err(error) => Observation {
                findings: Vec::new(),
                runs: all_ids
                    .iter()
                    .map(|id| ObservedRun {
                        rule_id: id.clone(),
                        state: RunState::NotRun(format!("analyze_security failed: {error}")),
                    })
                    .collect(),
            },
            Ok(analysis) => {
                let leaked = !analysis.sourcemap_leaks.leaks.is_empty();
                let source_map_state = source_map_run(log, leaked);
                Observation {
                    findings: analysis
                        .issues
                        .iter()
                        .map(|issue| ObservedFinding {
                            rule_id: format!("{}:{}", issue.header, issue.issue_type),
                            selector: None,
                            // A context-dependent header (COOP, CORP) only
                            // matters for some deployments; its message asks
                            // the reader to verify (#578): a review, not a
                            // confirmed gap.
                            class: if issue.tier == HeaderTier::ContextDependent {
                                FindingClass::Review
                            } else {
                                FindingClass::Fail
                            },
                            located: false,
                        })
                        .collect(),
                    runs: all_ids
                        .iter()
                        .map(|id| ObservedRun {
                            rule_id: id.clone(),
                            state: if id == SOURCE_MAP_ID {
                                source_map_state.clone()
                            } else {
                                RunState::Ran { findings: 0 }
                            },
                        })
                        .collect(),
                }
            }
        };
        let mismatches = diff_case(case, &obs, &diff_rules);
        if !mismatches.is_empty() {
            diffs.push((case.case.clone(), mismatches));
        }
    }

    if !diffs.is_empty() {
        panic!(
            "{}",
            render_report("Security detection corpus accuracy mismatches", &diffs)
        );
    }
}
