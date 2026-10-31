//! State-changing journeys are an opt-in (#787): a default run only observes.
//! A local routing server records every request, so the test sees what a
//! journey really sent.
//!
//!   cargo test --test state_changes_test -- --ignored

mod common;

use auditmysite::audit::run_single_audit;
use auditmysite::{BrowserManager, BrowserOptions, PipelineConfig, WcagLevel};
use common::routing_server::{serve_routes, Route, Transport};

/// A form whose submit button posts to `/order` and that validates nothing:
/// clicking the button sends the order.
const SHOP_FORM: &str =
    "<!doctype html><html lang=en><head><title>Order</title></head><body><main>\
<h1>Order</h1><form id=f action=/order method=post>\
<label for=n>Name</label><input id=n name=n required value=Ann>\
<button type=submit>Send order</button></form></main></body></html>";

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
        allow_state_changes: false,
        perf_runs: 1,
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

/// Requests the server saw after a full-depth audit of the form page.
fn audit_and_requests(allow_state_changes: bool) -> (Vec<String>, auditmysite::AuditReport) {
    let (tx, rx) = std::sync::mpsc::channel();
    on_big_stack(move || async move {
        let server = serve_routes(
            vec![
                Route::new("/", 200, SHOP_FORM.as_bytes().to_vec()),
                Route::new("/order", 200, b"<html><body>ordered</body></html>".to_vec()),
            ],
            Transport::Http,
        );
        let mut config = config();
        config.interactive = auditmysite::cli::InteractiveMode::Full;
        config.allow_state_changes = allow_state_changes;
        let browser = browser().await;
        let report = run_single_audit(&server.url, &browser, &config)
            .await
            .expect("audit");
        browser.close().await.ok();
        server.stop();
        tx.send((server.requests(), report)).unwrap();
    });
    rx.recv().unwrap()
}

#[test]
#[ignore = "needs Chrome"]
fn a_full_run_without_the_opt_in_sends_nothing() {
    let (requests, report) = audit_and_requests(false);
    assert!(
        !requests.iter().any(|r| r.starts_with("POST")),
        "an observing run submitted a form: {requests:?}"
    );
    let profile = &report.accessibility.execution.scope.measurement_profile;
    assert!(profile.read_only && profile.state_changing_journeys.is_empty());
}

#[test]
#[ignore = "needs Chrome"]
fn the_opt_in_allows_the_submit_and_the_profile_says_so() {
    let (requests, report) = audit_and_requests(true);
    let profile = &report.accessibility.execution.scope.measurement_profile;
    assert!(!profile.read_only);
    assert!(
        profile
            .state_changing_journeys
            .iter()
            .any(|j| j == "form_error_submit"),
        "{:?}",
        profile.state_changing_journeys
    );
    // Without the opt-in the same page sends nothing; with it the submit
    // journey runs and posts the form.
    assert!(
        requests.iter().any(|r| r == "POST /order"),
        "the permitted journey did not submit: {requests:?}"
    );
}
