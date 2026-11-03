//! Corpus of captures with expectations written from the rules (#873).
//!
//! No browser: each case is a synthetic `FirstVisitCapture`. The expectations
//! come from the definitions (tracking = analytics/advertising, embed =
//! fonts/CDN/video/maps/social, tag manager and non-EU seat are indications),
//! never from a run. The clean cases pin the false-positive target: 0.

use auditmysite::privacy::{
    analyze_first_visit, CaptureProfile, CapturedCookie, CapturedRequest, CapturedStorageKey,
    FirstVisitCapture, StorageArea,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    page_host: String,
    requests: Vec<String>,
    cookies: Vec<String>,
    storage: Vec<String>,
    expect_kinds: Vec<String>,
    expect_providers: Vec<String>,
    expect_consent_platforms: Vec<String>,
}

fn capture(case: &Case) -> FirstVisitCapture {
    let page = &case.page_host;
    FirstVisitCapture {
        contract_version: 1,
        requested_url: format!("https://{page}/"),
        final_url: format!("https://{page}/"),
        page_host: page.clone(),
        profile: CaptureProfile {
            fresh_context: true,
            interaction: false,
            privacy_signals_sent: false,
            idle_quiet_ms: 500,
            settle_ms: 2000,
        },
        network_idle: true,
        requests: case
            .requests
            .iter()
            .map(|url| {
                let host = url
                    .split("//")
                    .nth(1)
                    .and_then(|rest| rest.split('/').next())
                    .unwrap()
                    .to_string();
                CapturedRequest {
                    third_party: host != *page,
                    host,
                    url: url.clone(),
                    resource_type: "script".into(),
                    initiator: "parser".into(),
                    initiator_url: None,
                }
            })
            .collect(),
        cookies: case
            .cookies
            .iter()
            .map(|name| CapturedCookie {
                name: name.clone(),
                domain: page.clone(),
                path: "/".into(),
                expires: None,
                secure: true,
                http_only: false,
                same_site: None,
                third_party: false,
            })
            .collect(),
        storage: case
            .storage
            .iter()
            .map(|key| CapturedStorageKey {
                origin: format!("https://{page}"),
                area: StorageArea::Local,
                key: key.clone(),
            })
            .collect(),
    }
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

#[test]
fn every_case_matches_its_rule_derived_expectation() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/privacy/corpus.json"
    ))
    .unwrap();
    let corpus: Corpus = serde_json::from_str(&text).unwrap();
    assert!(corpus.cases.len() >= 10);
    for case in &corpus.cases {
        let analysis = analyze_first_visit(&capture(case), false);
        let kinds = sorted(
            analysis
                .findings
                .iter()
                .map(|f| f.kind.code().to_string())
                .collect(),
        );
        let providers = sorted(
            analysis
                .contacts
                .iter()
                .map(|c| c.provider_id.clone())
                .collect(),
        );
        assert_eq!(
            kinds,
            sorted(case.expect_kinds.clone()),
            "{}: kinds",
            case.name
        );
        assert_eq!(
            providers,
            sorted(case.expect_providers.clone()),
            "{}: providers",
            case.name
        );
        assert_eq!(
            analysis.consent_platforms,
            sorted(case.expect_consent_platforms.clone()),
            "{}: consent platforms",
            case.name
        );
    }
}

#[test]
fn clean_cases_produce_no_false_positive() {
    // Target of #873: 0 false positives on the clean fixtures.
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/privacy/corpus.json"
    ))
    .unwrap();
    let corpus: Corpus = serde_json::from_str(&text).unwrap();
    let clean: Vec<_> = corpus
        .cases
        .iter()
        .filter(|c| c.expect_kinds.is_empty())
        .collect();
    assert!(clean.len() >= 4);
    for case in clean {
        assert!(
            analyze_first_visit(&capture(case), false)
                .findings
                .is_empty(),
            "{}",
            case.name
        );
    }
}
