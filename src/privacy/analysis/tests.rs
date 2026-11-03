use super::*;
use crate::privacy::first_visit::{
    CaptureProfile, CapturedCookie, CapturedRequest, CapturedStorageKey, FirstVisitCapture,
    StorageArea,
};

fn profile() -> CaptureProfile {
    CaptureProfile {
        fresh_context: true,
        interaction: false,
        privacy_signals_sent: false,
        idle_quiet_ms: 500,
        settle_ms: 2000,
    }
}

fn request(url: &str, page_host: &str) -> CapturedRequest {
    let host = url
        .split("//")
        .nth(1)
        .and_then(|r| r.split('/').next())
        .unwrap()
        .to_string();
    CapturedRequest {
        third_party: host != page_host,
        host,
        url: url.into(),
        resource_type: "script".into(),
        initiator: "parser".into(),
        initiator_url: None,
    }
}

fn cookie(name: &str) -> CapturedCookie {
    CapturedCookie {
        name: name.into(),
        domain: "example.com".into(),
        path: "/".into(),
        expires: None,
        secure: true,
        http_only: false,
        same_site: None,
        third_party: false,
    }
}

fn capture(
    page_host: &str,
    urls: &[&str],
    cookies: &[&str],
    storage: &[&str],
) -> FirstVisitCapture {
    FirstVisitCapture {
        contract_version: 1,
        requested_url: format!("https://{page_host}/"),
        final_url: format!("https://{page_host}/"),
        page_host: page_host.into(),
        profile: profile(),
        network_idle: true,
        requests: urls.iter().map(|u| request(u, page_host)).collect(),
        cookies: cookies.iter().map(|c| cookie(c)).collect(),
        storage: storage
            .iter()
            .map(|k| CapturedStorageKey {
                origin: format!("https://{page_host}"),
                area: StorageArea::Local,
                key: (*k).into(),
            })
            .collect(),
    }
}

fn kinds(a: &PrivacyAnalysis) -> Vec<PrivacyFindingKind> {
    a.findings.iter().map(|f| f.kind).collect()
}

#[test]
fn clean_page_has_no_findings() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://example.com/", "https://example.com/app.js"],
            &["session"],
            &[],
        ),
        false,
    );
    assert!(a.findings.is_empty() && a.contacts.is_empty() && a.stored.is_empty());
}

#[test]
fn analytics_request_is_evidence() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://www.google-analytics.com/g/collect?v=2"],
            &[],
            &[],
        ),
        false,
    );
    use PrivacyFindingKind::*;
    assert_eq!(
        kinds(&a),
        vec![
            TrackingRequestBeforeConsent,
            ProviderOutsideEu,
            TrackingWithoutConsentPlatform
        ]
    );
    let f = &a.findings[0];
    assert_eq!(
        (f.confidence, f.subjects.as_slice()),
        (Confidence::Evidence, &["Google Analytics".to_string()][..])
    );
    assert_eq!(a.contacts[0].hosts, vec!["www.google-analytics.com"]);
}

#[test]
fn tracking_cookie_is_evidence_even_without_requests() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://example.com/"],
            &["_ga", "_ga_ABC"],
            &[],
        ),
        false,
    );
    assert!(kinds(&a).contains(&PrivacyFindingKind::TrackingStorageBeforeConsent));
    assert_eq!(a.stored.len(), 2);
    assert!(a.stored.iter().all(|s| s.kind == "cookie"));
}

#[test]
fn local_storage_key_is_classified() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://example.com/"],
            &[],
            &["_hjSessionUser_1"],
        ),
        false,
    );
    assert_eq!(a.stored[0].kind, "local_storage");
    assert_eq!(a.stored[0].provider_id, "hotjar");
}

#[test]
fn fonts_are_an_ip_transfer_not_tracking() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://fonts.googleapis.com/css2?family=Inter"],
            &[],
            &[],
        ),
        false,
    );
    use PrivacyFindingKind::*;
    assert_eq!(kinds(&a), vec![ThirdPartyEmbedContacted, ProviderOutsideEu]);
}

#[test]
fn consent_platform_suppresses_the_missing_platform_finding_and_is_not_flagged() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &[
                "https://consent.cookiebot.com/uc.js",
                "https://www.google-analytics.com/g/collect",
            ],
            &[],
            &[],
        ),
        false,
    );
    assert_eq!(a.consent_platforms, vec!["Cookiebot (Usercentrics)"]);
    assert!(!kinds(&a).contains(&PrivacyFindingKind::TrackingWithoutConsentPlatform));
    // The platform itself is EU-based and no tracking.
    assert!(a
        .findings
        .iter()
        .all(|f| !f.subjects.contains(&"Cookiebot (Usercentrics)".to_string())));
}

#[test]
fn consent_cookie_name_counts_as_platform() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://www.google-analytics.com/g/collect"],
            &["OptanonConsent"],
            &[],
        ),
        false,
    );
    assert_eq!(a.consent_platforms, vec!["OneTrust"]);
}

#[test]
fn tag_manager_is_an_indication() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://www.googletagmanager.com/gtm.js?id=GTM-1"],
            &[],
            &[],
        ),
        false,
    );
    let f = a
        .findings
        .iter()
        .find(|f| f.kind == PrivacyFindingKind::TagManagerLoaded)
        .unwrap();
    assert_eq!(f.confidence, Confidence::Indication);
    // A tag manager alone is not tracking.
    assert!(!kinds(&a).contains(&PrivacyFindingKind::TrackingRequestBeforeConsent));
    assert!(!kinds(&a).contains(&PrivacyFindingKind::TrackingWithoutConsentPlatform));
}

#[test]
fn facebook_plugin_is_social_pixel_is_advertising() {
    let plugin = analyze_first_visit(
        &capture(
            "example.com",
            &["https://connect.facebook.net/en_US/sdk.js"],
            &[],
            &[],
        ),
        false,
    );
    assert!(!kinds(&plugin).contains(&PrivacyFindingKind::TrackingRequestBeforeConsent));
    let pixel = analyze_first_visit(
        &capture(
            "example.com",
            &["https://connect.facebook.net/en_US/fbevents.js"],
            &[],
            &[],
        ),
        false,
    );
    assert!(kinds(&pixel).contains(&PrivacyFindingKind::TrackingRequestBeforeConsent));
    // Both in one visit: the stronger category wins, regardless of request order.
    let both = analyze_first_visit(
        &capture(
            "example.com",
            &[
                "https://connect.facebook.net/en_US/fbevents.js",
                "https://connect.facebook.net/en_US/sdk.js",
            ],
            &[],
            &[],
        ),
        false,
    );
    assert_eq!(both.contacts[0].category, "advertising");
}

#[test]
fn the_page_is_not_its_own_third_party() {
    let a = analyze_first_visit(
        &capture(
            "www.youtube.com",
            &["https://www.youtube.com/", "https://i.ytimg.com/vi/x.jpg"],
            &[],
            &[],
        ),
        false,
    );
    assert!(a.findings.is_empty(), "{:?}", a.findings);
}

#[test]
fn look_alike_host_is_not_a_finding() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://google-analytics.com.evil.example/x.js"],
            &[],
            &[],
        ),
        false,
    );
    assert!(a.findings.is_empty());
}

#[test]
fn first_party_requests_are_ignored_even_for_a_known_host() {
    // third_party=false for the page's own host; a provider-domain page is covered above,
    // here the flag alone must keep a same-host request out.
    let mut c = capture("example.com", &["https://example.com/"], &[], &[]);
    c.requests[0].url = "https://www.google-analytics.com/x".into();
    c.requests[0].third_party = false;
    assert!(analyze_first_visit(&c, false).contacts.is_empty());
}

#[test]
fn partial_flag_and_ruleset_are_carried() {
    let a = analyze_first_visit(
        &capture("example.com", &["https://example.com/"], &[], &[]),
        true,
    );
    assert!(a.partial);
    assert_eq!(a.ruleset_version, web_checks::third_party::RULESET_VERSION);
    assert!(!a.profile.interaction);
}

#[test]
fn en_text_has_no_german_characters() {
    use PrivacyFindingKind::*;
    for kind in [
        TrackingRequestBeforeConsent,
        TrackingStorageBeforeConsent,
        ThirdPartyEmbedContacted,
        TagManagerLoaded,
        ProviderOutsideEu,
        TrackingWithoutConsentPlatform,
    ] {
        let t = privacy_finding_text(kind, &["X".into()], true);
        assert!(!t.chars().any(|c| "äöüßÄÖÜ".contains(c)), "{t}");
        assert_ne!(t, privacy_finding_text(kind, &["X".into()], false));
    }
}

#[test]
fn analysis_serializes_canonically() {
    let a = analyze_first_visit(
        &capture(
            "example.com",
            &["https://www.google-analytics.com/g/collect"],
            &[],
            &[],
        ),
        false,
    );
    let v = serde_json::to_value(&a).unwrap();
    assert_eq!(v["findings"][0]["kind"], "tracking_request_before_consent");
    assert_eq!(v["findings"][0]["confidence"], "evidence");
}
