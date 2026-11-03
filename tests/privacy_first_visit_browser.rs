//! #870: the first-visit capture against real Chrome and a local fixture page.
//!
//! The fixture sets one cookie by header, one by script, fills both storage
//! areas, requests an image from the page host, and — after a delay, like a
//! late tag — one from another host (`localhost` instead of `127.0.0.1`).

mod common;

use std::collections::BTreeMap;
use std::time::Duration;

use auditmysite::audit::collection::CollectionState;
use auditmysite::privacy::{
    collect_first_visit, CapturedCookie, FirstVisitCapture, FirstVisitOptions, StorageArea,
};
use auditmysite::{BrowserManager, BrowserOptions};

const PAGE: &str = "<!doctype html><html lang=\"en\"><head><title>t</title>\
    <link rel=\"icon\" href=\"data:,\"></head><body><main><h1>Hello</h1>\
    <img src=\"/pixel.gif?own=1\" alt=\"\"></main>\
    <script>\
    document.cookie = '_pref=1; path=/; max-age=86400';\
    localStorage.setItem('ls_key', 'local-value');\
    sessionStorage.setItem('ss_key', 'session-value');\
    localStorage.setItem('dnt_' + String(navigator.doNotTrack), '1');\
    setTimeout(function () {\
      new Image().src = location.origin.replace('127.0.0.1', 'localhost') + '/late.gif?tag=1';\
    }, 400);\
    </script></body></html>";

fn options() -> FirstVisitOptions {
    FirstVisitOptions {
        idle_quiet: Duration::from_millis(300),
        idle_max: Duration::from_secs(8),
        settle: Duration::from_millis(1000),
    }
}

async fn browser() -> BrowserManager {
    BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch")
}

fn serve() -> common::fixture_server::Fixture {
    common::fixture_server::serve(
        PAGE.to_string(),
        BTreeMap::from([(
            "Set-Cookie".to_string(),
            "srv=1; Path=/; HttpOnly; SameSite=Lax".to_string(),
        )]),
    )
}

async fn measured(manager: &BrowserManager, url: &str) -> FirstVisitCapture {
    let collected = collect_first_visit(manager, url, options()).await;
    assert_eq!(
        collected.state(),
        CollectionState::Measured,
        "{collected:?}"
    );
    collected.into_value().expect("measured capture")
}

fn cookie<'a>(capture: &'a FirstVisitCapture, name: &str) -> &'a CapturedCookie {
    capture
        .cookies
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("cookie {name} missing: {:?}", capture.cookies))
}

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test privacy_first_visit_browser -- --ignored`"]
async fn a_first_visit_records_requests_cookies_and_storage_keys() {
    let manager = browser().await;
    let fixture = serve();
    let capture = measured(&manager, &fixture.url).await;
    assert!(fixture.served() > 0, "the fixture answered nothing");

    assert_eq!(capture.page_host, "127.0.0.1");
    assert!(capture.network_idle);
    assert!(capture.profile.fresh_context);
    assert!(!capture.profile.interaction);
    assert!(!capture.profile.privacy_signals_sent);

    let find = |path: &str| {
        capture
            .requests
            .iter()
            .find(|r| r.url.contains(path))
            .unwrap_or_else(|| panic!("no request for {path}: {:?}", capture.requests))
    };
    let document = find(&format!("{}/", fixture.url.trim_end_matches('/')));
    assert_eq!(document.resource_type, "document");
    assert!(!document.third_party);
    let own = find("/pixel.gif?own=1");
    assert_eq!(own.resource_type, "image");
    assert_eq!(own.initiator, "parser");
    assert!(!own.third_party);
    let late = find("/late.gif?tag=1");
    assert_eq!(late.host, "localhost");
    assert!(late.third_party, "{late:?}");

    let srv = cookie(&capture, "srv");
    assert!(srv.http_only && !srv.secure);
    assert_eq!(srv.same_site.as_deref(), Some("Lax"));
    assert_eq!(srv.expires, None);
    let pref = cookie(&capture, "_pref");
    assert!(!pref.http_only);
    assert!(pref.expires.is_some());
    assert!(!pref.third_party);

    let has = |area: StorageArea, key: &str| {
        capture
            .storage
            .iter()
            .any(|s| s.area == area && s.key == key && s.origin == fixture.url)
    };
    assert!(has(StorageArea::Local, "ls_key"), "{:?}", capture.storage);
    assert!(has(StorageArea::Session, "ss_key"));
    // No Do-Not-Track signal reached the page: the visitor is ordinary.
    assert!(has(StorageArea::Local, "dnt_null"), "{:?}", capture.storage);

    let json = serde_json::to_string(&capture).unwrap();
    for value in ["local-value", "session-value"] {
        assert!(!json.contains(value), "a storage value leaked: {value}");
    }
    fixture.stop();
    let _ = manager.close().await;
}

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test privacy_first_visit_browser -- --ignored`"]
async fn three_visits_of_the_same_page_agree() {
    let manager = browser().await;
    let fixture = serve();
    // `expires` is an absolute time that follows the clock of the visit
    // (`max-age=86400`): compare its distance from now, not the instant.
    let normalised = |mut capture: FirstVisitCapture| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        for cookie in &mut capture.cookies {
            if let Some(expires) = cookie.expires.as_mut() {
                assert!(
                    (*expires - now - 86400.0).abs() < 120.0,
                    "{}: expires {expires} is not a day away",
                    cookie.name
                );
                *expires = 86400.0;
            }
        }
        capture
    };
    let first = normalised(measured(&manager, &fixture.url).await);
    for _ in 0..2 {
        assert_eq!(normalised(measured(&manager, &fixture.url).await), first);
    }
    fixture.stop();
    let _ = manager.close().await;
}

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test privacy_first_visit_browser -- --ignored`"]
async fn state_of_another_page_does_not_reach_the_clean_profile() {
    let manager = browser().await;
    let fixture = serve();
    // A page of the default profile that already holds state for the site.
    let other = manager.new_page().await.expect("page");
    manager
        .navigate(&other, &fixture.url)
        .await
        .expect("navigation");
    other
        .evaluate("document.cookie = 'from_other_page=1; path=/'; localStorage.setItem('other_key', '1'); 0")
        .await
        .expect("seed state");

    let capture = measured(&manager, &fixture.url).await;
    assert!(
        !capture.cookies.iter().any(|c| c.name == "from_other_page"),
        "{:?}",
        capture.cookies
    );
    assert!(
        !capture.storage.iter().any(|s| s.key == "other_key"),
        "{:?}",
        capture.storage
    );
    fixture.stop();
    let _ = manager.close().await;
}

/// A page that cannot be loaded is "not measured" — never an empty capture
/// that reads as "no trackers".
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test privacy_first_visit_browser -- --ignored`"]
async fn an_unreachable_page_is_a_failed_collection_not_an_empty_capture() {
    let manager = browser().await;
    // Port 9 (discard) refuses the connection.
    let collected = collect_first_visit(&manager, "http://127.0.0.1:9/", options()).await;
    assert_eq!(collected.state(), CollectionState::Failed, "{collected:?}");
    assert!(collected.value().is_none());
    let _ = manager.close().await;
}

const FINDINGS_PAGE: &str = "<!doctype html><html lang=\"en\"><head><title>t</title>\
    <link rel=\"icon\" href=\"data:,\"></head><body>\
    <div id=\"cookie-banner\" role=\"dialog\" aria-label=\"Cookies\">We use cookies</div>\
    <main><h1>Hello</h1></main>\
    <footer><a href=\"#\">Datenschutz</a> <a href=\"/impressum\">Impressum</a></footer>\
    <script>\
    document.cookie = '_ga=GA1.1.1; path=/; max-age=86400';\
    localStorage.setItem('_fbp', 'fb.1');\
    </script></body></html>";

const CLEAN_PAGE: &str = "<!doctype html><html lang=\"en\"><head><title>t</title>\
    <link rel=\"icon\" href=\"data:,\"></head><body><main><h1>Hello</h1>\
    <a href=\"/privacy\">Privacy</a> <a href=\"/imprint\">Imprint</a></main></body></html>";

/// #871: the findings of a real first visit. Cookie and storage names are
/// classified, the dead policy link is observed, the banner element counts as
/// a consent trace, and a clean page reads clean.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test privacy_first_visit_browser -- --ignored`"]
async fn findings_follow_what_the_first_visit_shows() {
    use auditmysite::privacy::{analyze_privacy, CheckOutcome, PrivacyCheck, PrivacyFindingKind};

    let manager = browser().await;

    let fixture = common::fixture_server::serve_html(FINDINGS_PAGE.to_string());
    let collected = collect_first_visit(&manager, &fixture.url, options()).await;
    let capture = collected.value().expect("capture");
    assert!(
        capture
            .signals
            .consent_signals
            .contains(&"element:generic-banner".to_string()),
        "{:?}",
        capture.signals
    );
    let analysis = analyze_privacy(&collected);
    let outcome = |check| {
        analysis
            .checks
            .iter()
            .find(|c| c.check == check)
            .unwrap()
            .outcome
    };
    assert_eq!(
        outcome(PrivacyCheck::NonEssentialStorage),
        CheckOutcome::Found
    );
    assert_eq!(outcome(PrivacyCheck::PolicyLinks), CheckOutcome::Found);
    let kinds: Vec<_> = analysis.findings.iter().map(|f| f.kind).collect();
    assert!(kinds.contains(&PrivacyFindingKind::NonEssentialStorageBeforeConsent));
    assert!(kinds.contains(&PrivacyFindingKind::PolicyLinkUnusable));
    assert!(!kinds.contains(&PrivacyFindingKind::PolicyLinkMissing));
    let storage: Vec<_> = analysis
        .findings
        .iter()
        .filter(|f| f.kind == PrivacyFindingKind::NonEssentialStorageBeforeConsent)
        .flat_map(|f| f.evidence.iter().map(|e| e.value.as_str()))
        .collect();
    assert!(
        storage.contains(&"_ga") && storage.contains(&"_fbp"),
        "{storage:?}"
    );
    fixture.stop();

    let fixture = common::fixture_server::serve_html(CLEAN_PAGE.to_string());
    let analysis = analyze_privacy(&collect_first_visit(&manager, &fixture.url, options()).await);
    assert!(analysis.findings.is_empty(), "{:?}", analysis.findings);
    assert!(analysis
        .checks
        .iter()
        .all(|c| c.outcome != CheckOutcome::NotMeasured));
    fixture.stop();

    let _ = manager.close().await;
}
