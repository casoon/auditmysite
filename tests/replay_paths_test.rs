//! One measurement, three ways back to a report (#790, #787, #788): the fresh
//! run, the `--reuse-cache` hit and `auditmysite replay` must give the same
//! canonical model — only the named volatile metadata may differ — and a
//! measurement taken under another profile (session policy, declared state)
//! must never be presented as the same one.
//!
//! Local fixture only; the cache lives in a throw-away `$HOME`.
//!
//!   cargo test --test replay_paths_test -- --ignored

mod common;

use auditmysite::audit::normalized::NormalizedReport;
use auditmysite::audit::{
    cached_report, content_hash, normalize_checked, profile_differences, replay, run_single_audit,
    CacheMiss, ExpectedProfile, ReplayError, SnapshotArtifact,
};
use auditmysite::browser::StorageState;
use auditmysite::{BrowserManager, BrowserOptions, PipelineConfig, WcagLevel};
use serde_json::Value;

const PAGE: &str = "<!doctype html><html lang=en><head><title>Replay fixture</title></head>\
<body><main><h1>Replay fixture</h1><p>Plain content.</p><img src=\"data:image/gif;base64,R0lGODlhAQABAAAAACw=\"></main></body></html>";

/// A single-page app: the content changes after a navigation without a
/// reload (`pushState`), driven by the declared state in localStorage — the
/// state a login or a consent choice would leave.
const SPA: &str = "<!doctype html><html lang=en><head><title>App</title></head><body>\
<main id=app><h1>Start</h1><p>Welcome.</p></main><script>\
var route = localStorage.getItem('route') || 'start';\
if (route !== 'start') {\
  history.pushState({}, '', '/' + route);\
  document.getElementById('app').innerHTML = '<h1>Settings</h1><p>Account</p>\
<img src=\"data:image/gif;base64,R0lGODlhAQABAAAAACw=\"><button></button>';\
}</script></body></html>";

/// The cache directory is `$HOME/.auditmysite/cache`; keep the real one out of
/// the test.
fn isolated_home() {
    static HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    HOME.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", dir.path());
        dir
    });
}

fn config() -> PipelineConfig {
    PipelineConfig {
        wcag_level: WcagLevel::AA,
        timeout_secs: 30,
        stability_budget_ms: 1500,
        verbose: false,
        full_audit: false,
        check_performance: false,
        check_seo: true,
        check_security: false,
        check_mobile: false,
        check_dark_mode: false,
        check_design_quality: false,
        check_html_conform: false,
        check_ai_transparency: false,
        check_dns: false,
        check_isolate_third_party_impact: false,
        check_ssr_content: false,
        check_stack: false,
        rule_filter: auditmysite::wcag::RuleFilterConfig::default(),
        persist_artifacts: true,
        capture_screenshots: false,
        capture_element_evidence: false,
        dismiss_consent: false,
        exclude_selectors: Vec::new(),
        interactive: auditmysite::cli::InteractiveMode::Off,
        journey_budget_ms: auditmysite::a11y_journey::DEFAULT_BUDGET_MS,
        session: Default::default(),
        storage_state: None,
        state_id: None,
        lang: "en".to_string(),
        display_mode: None,
        indexability: Default::default(),
        decision_profile: Default::default(),
    }
}

async fn browser() -> BrowserManager {
    BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser")
}

async fn fresh(url: &str, config: &PipelineConfig) -> auditmysite::AuditReport {
    let browser = browser().await;
    let report = run_single_audit(url, &browser, config).await;
    browser.close().await.ok();
    report.expect("audit")
}

/// The named volatile metadata is the only thing allowed to differ.
fn without_volatile(normalized: &NormalizedReport) -> Value {
    let mut value = serde_json::to_value(normalized).unwrap();
    let object = value.as_object_mut().unwrap();
    object.remove("timestamp");
    object.remove("duration_ms");
    object["execution"]["environment"]
        .as_object_mut()
        .unwrap()
        .remove("source");
    value
}

/// A whole report is large and the audit future holds several: the default
/// test thread overflows its stack in an unoptimized build.
fn on_big_stack<F: std::future::Future<Output = ()>>(make: impl FnOnce() -> F + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(128 << 20)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(make())
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
#[ignore = "needs Chrome"]
fn fresh_run_cache_hit_and_replay_give_one_canonical_model() {
    on_big_stack(|| async {
        isolated_home();
        let fixture = common::fixture_server::serve_html(PAGE.to_string());
        let config = config();
        let report = fresh(&fixture.url, &config).await;
        assert!(fixture.served() > 0, "the fresh run measured the page");
        assert_eq!(report.accessibility.execution.environment.source, "live");

        // From here on there is no page to ask.
        fixture.stop();
        let served = fixture.served();

        let fresh_model = normalize_checked(&report).unwrap().normalized;
        let cached = cached_report(&fixture.url, &config.audit_signature())
            .unwrap()
            .expect("cache hit");
        assert_eq!(cached.accessibility.execution.environment.source, "cache");
        let cache_model = normalize_checked(&cached).unwrap().normalized;

        let snapshot = SnapshotArtifact::read_cached(&fixture.url).unwrap();
        let replay_model = replay(&snapshot).unwrap();
        assert_eq!(replay_model.execution.environment.source, "replay");

        let expected = without_volatile(&fresh_model);
        assert_eq!(expected, without_volatile(&cache_model), "cache hit");
        assert_eq!(expected, without_volatile(&replay_model), "replay");
        assert!(fresh_model.findings.len() > 1, "a fixture with findings");
        assert_eq!(fixture.served(), served, "no network access after the run");
    })
}

#[test]
#[ignore = "needs Chrome"]
fn another_measurement_profile_is_not_the_same_measurement() {
    on_big_stack(|| async {
        isolated_home();
        let fixture = common::fixture_server::serve_html(SPA.to_string());
        let plain = config();
        let origin = fixture.url.clone();
        let declared = StorageState::from_bytes(
        format!(
            r#"{{"cookies":[],"origins":[{{"origin":"{origin}","localStorage":[{{"name":"route","value":"settings"}}]}}]}}"#
        )
        .as_bytes(),
    )
    .unwrap();
        let digest = declared.digest.clone();
        let in_state = PipelineConfig {
            storage_state: Some(declared),
            state_id: Some("settings-route".to_string()),
            ..config()
        };

        // Same configuration twice: the same content, no difference to explain.
        let first = fresh(&fixture.url, &plain).await;
        let first_snapshot = SnapshotArtifact::read_cached(&fixture.url).unwrap();
        fresh(&fixture.url, &plain).await;
        let again = SnapshotArtifact::read_cached(&fixture.url).unwrap();
        assert_eq!(content_hash(&first_snapshot), content_hash(&again));
        assert!(profile_differences(&profile(&first_snapshot), &profile(&again)).is_empty());

        // The declared state changes what the page shows after its client-side
        // navigation — and the profile names exactly that.
        fresh(&fixture.url, &in_state).await;
        let moved = SnapshotArtifact::read_cached(&fixture.url).unwrap();
        assert_ne!(
            content_hash(&first_snapshot),
            content_hash(&moved),
            "the SPA changed its content without a reload"
        );
        let explained = profile_differences(&profile(&first_snapshot), &profile(&moved));
        let fields: Vec<_> = explained.iter().map(|d| d.field).collect();
        assert!(
            fields.contains(&"state_id") && fields.contains(&"storage_state"),
            "{fields:?}"
        );
        assert!(
            first.accessibility.wcag_results.violations.len() != replay_violations(&moved),
            "the state shows in the findings"
        );

        // The cache holds the in-state measurement now: the plain run is not
        // answered with it, and the in-state run is.
        assert!(matches!(
            cached_report(&fixture.url, &plain.audit_signature()).unwrap(),
            Err(CacheMiss::OtherConfiguration)
        ));
        assert!(cached_report(&fixture.url, &in_state.audit_signature())
            .unwrap()
            .is_ok());

        // A replay asserting another profile than the stored one is refused.
        let asserts = |f: fn(&mut ExpectedProfile, &str), digest: &str| {
            let mut expected = ExpectedProfile::default();
            f(&mut expected, digest);
            moved.check_measurement(&expected)
        };
        assert!(asserts(|_, _| {}, &digest).is_ok());
        assert!(asserts(
            |e, d| {
                e.state_id = Some("settings-route".into());
                e.storage_state_sha256 = Some(d.into());
            },
            &digest
        )
        .is_ok());
        for refused in [
            asserts(|e, _| e.state_id = Some("logged-out".into()), &digest),
            asserts(|e, _| e.session_policy = Some("shared".into()), &digest),
            asserts(
                |e, _| e.storage_state_sha256 = Some("0".repeat(64)),
                &digest,
            ),
        ] {
            assert!(
                matches!(refused, Err(ReplayError::OtherMeasurement(_))),
                "{refused:?}"
            );
        }
        fixture.stop();
    })
}

fn profile(snapshot: &SnapshotArtifact) -> auditmysite::audit::MeasurementProfile {
    snapshot
        .collected
        .as_ref()
        .unwrap()
        .accessibility
        .execution
        .scope
        .measurement_profile
        .clone()
}

fn replay_violations(snapshot: &SnapshotArtifact) -> usize {
    snapshot
        .collected
        .as_ref()
        .unwrap()
        .accessibility
        .wcag_results
        .violations
        .len()
}
