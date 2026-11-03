//! #871: the privacy findings against the fixture corpus
//! (`tests/fixtures/privacy/cases.json`). Expectations come from the rules in
//! `src/privacy/findings.rs`, not from a run; every check has a positive,
//! negative, not-applicable and error (`not_measured`) case.

use auditmysite::audit::collection::{Collected, COLLECTION_CONTRACT_VERSION};
use auditmysite::privacy::{
    analyze_privacy, finding_text, CaptureProfile, CapturedCookie, CapturedLink, CapturedRequest,
    CapturedStorageKey, CheckOutcome, FirstVisitCapture, PageSignals, PolicyDocument,
    PrivacyAnalysis, PrivacyCheck, PrivacyFindingKind, StorageArea,
};
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/privacy/cases.json")).expect("valid corpus")
}

fn strings(case: &Value, key: &str) -> Vec<String> {
    case[key]
        .as_array()
        .map(|a| a.iter().map(|v| v.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

fn host_of(url: &str) -> String {
    url.split("://")
        .nth(1)
        .and_then(|rest| rest.split(['/', '?']).next())
        .unwrap()
        .to_string()
}

fn collected(case: &Value) -> Collected<FirstVisitCapture> {
    let state = case["collection"].as_str().unwrap_or("measured");
    if state == "failed" {
        return Collected::Failed("navigation_failed");
    }
    let page = case["page"].as_str().unwrap_or("example.com").to_string();
    let request = |url: &str, resource_type: &str| CapturedRequest {
        url: url.to_string(),
        host: host_of(url),
        resource_type: resource_type.to_string(),
        initiator: "parser".into(),
        initiator_url: None,
        third_party: host_of(url) != page,
    };
    let mut requests = vec![request(&format!("https://{page}/"), "document")];
    requests.extend(
        strings(case, "requests")
            .iter()
            .map(|u| request(u, "script")),
    );
    let cookies = strings(case, "cookies")
        .into_iter()
        .map(|name| CapturedCookie {
            name,
            domain: page.clone(),
            path: "/".into(),
            expires: None,
            secure: true,
            http_only: false,
            same_site: None,
            third_party: false,
        })
        .collect();
    let storage = strings(case, "storage")
        .into_iter()
        .map(|key| CapturedStorageKey {
            origin: format!("https://{page}"),
            area: StorageArea::Local,
            key,
        })
        .collect();
    let links = match case.get("links") {
        Some(list) => list
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| (pair[0].as_str().unwrap(), pair[1].as_str().unwrap()))
            .collect::<Vec<_>>(),
        None => vec![("/datenschutz", "Datenschutz"), ("/impressum", "Impressum")],
    };
    let signals = PageSignals {
        consent_signals: strings(case, "consent_signals"),
        links: links
            .into_iter()
            .map(|(href, text)| CapturedLink {
                href: href.to_string(),
                resolved: format!("https://{page}{href}"),
                text: text.to_string(),
            })
            .collect(),
    };
    let capture = FirstVisitCapture {
        contract_version: COLLECTION_CONTRACT_VERSION,
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
        requests,
        cookies,
        storage,
        signals,
    };
    if state == "partial" {
        let gaps = strings(case, "gaps")
            .into_iter()
            .map(|g| &*Box::leak(g.into_boxed_str()))
            .collect();
        Collected::Partial {
            value: capture,
            gaps,
        }
    } else {
        Collected::Measured(capture)
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
    assert_eq!(analysis.checks.len(), PrivacyCheck::ALL.len());
    for (result, check) in analysis.checks.iter().zip(PrivacyCheck::ALL) {
        assert_eq!(result.check, check);
    }
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

#[test]
fn every_case_yields_what_the_rules_say() {
    for case in cases() {
        let name = case["name"].as_str().unwrap();
        let analysis = analyze_privacy(&collected(&case));
        let expect = &case["expect"];
        assert_eq!(
            outcomes(&analysis),
            expect["outcomes"].as_str().unwrap(),
            "{name}: check outcomes"
        );
        let mut want: Vec<String> = expect["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                let f: Vec<&str> = f
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect();
                f.join("|")
            })
            .collect();
        want.sort();
        assert_eq!(finding_keys(&analysis), want, "{name}: findings");
    }
}

#[test]
fn cases_cover_every_check_and_outcome() {
    let mut seen = std::collections::BTreeSet::new();
    for case in cases() {
        let letters: Vec<char> = case["expect"]["outcomes"]
            .as_str()
            .unwrap()
            .chars()
            .collect();
        for (i, l) in letters.iter().enumerate() {
            seen.insert((i, *l));
        }
    }
    for check in 0..PrivacyCheck::ALL.len() {
        for outcome in ['F', 'N', 'A', 'M'] {
            assert!(
                seen.contains(&(check, outcome)),
                "check {:?} has no {outcome} case",
                PrivacyCheck::ALL[check]
            );
        }
    }
}

#[test]
fn a_failed_collection_reads_not_measured_with_its_reason() {
    let analysis = analyze_privacy(&Collected::Failed("navigation_failed"));
    assert!(analysis.findings.is_empty() && analysis.contacts.is_empty());
    assert!(analysis
        .checks
        .iter()
        .all(|c| c.outcome == CheckOutcome::NotMeasured
            && c.reason.as_deref() == Some("navigation_failed")));
    assert_eq!(
        analysis.measurement.reason.as_deref(),
        Some("navigation_failed")
    );
}

#[test]
fn evidence_never_carries_query_strings_or_values() {
    let case = cases()
        .into_iter()
        .find(|c| c["name"] == "google analytics before consent")
        .unwrap();
    let analysis = analyze_privacy(&collected(&case));
    let json = serde_json::to_string(&analysis).unwrap();
    assert!(!json.contains("tid=G-1") && !json.contains('?'), "{json}");
}

#[test]
fn contacts_and_unclassified_hosts_describe_who_was_contacted() {
    let case = cases()
        .into_iter()
        .find(|c| c["name"] == "analytics with a consent platform request")
        .unwrap();
    let analysis = analyze_privacy(&collected(&case));
    let ids: Vec<&str> = analysis
        .contacts
        .iter()
        .map(|c| c.provider.id.as_str())
        .collect();
    assert_eq!(ids, ["google-analytics", "onetrust"]);
    let case = cases()
        .into_iter()
        .find(|c| c["name"] == "look-alike hosts are not providers")
        .unwrap();
    let analysis = analyze_privacy(&collected(&case));
    assert!(analysis.contacts.is_empty());
    assert_eq!(
        analysis.unclassified_hosts,
        [
            "google-analytics.com.evil.example",
            "notgoogle-analytics.com"
        ]
    );
}

#[test]
fn canonical_analysis_is_english_and_has_no_verdict_wording() {
    let mut texts = Vec::new();
    for case in cases() {
        for f in analyze_privacy(&collected(&case)).findings {
            texts.push(f.message);
        }
    }
    assert!(!texts.is_empty());
    for text in &texts {
        assert!(
            !text.chars().any(|c| "äöüÄÖÜß".contains(c)),
            "umlaut in canonical text: {text}"
        );
        let lower = text.to_lowercase();
        for banned in [
            "violat",
            "illegal",
            "unlawful",
            "non-compliant",
            "gdpr",
            "dsgvo",
            "fine",
        ] {
            assert!(!lower.contains(banned), "{banned} in: {text}");
        }
    }
}

#[test]
fn german_text_exists_for_every_kind_and_differs_from_english() {
    use PrivacyFindingKind as K;
    for kind in [
        K::TrackingRequestBeforeConsent,
        K::NonEssentialStorageBeforeConsent,
        K::VisitorIpTransfer,
        K::ThirdCountryProvider,
        K::NoConsentPlatformDetected,
        K::PolicyLinkMissing,
        K::PolicyLinkUnusable,
    ] {
        let en = finding_text(kind, "X", Some(PolicyDocument::Imprint), true);
        let de = finding_text(kind, "X", Some(PolicyDocument::Imprint), false);
        assert_ne!(en, de, "{kind:?}");
    }
}
