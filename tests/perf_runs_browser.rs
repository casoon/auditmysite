//! `--perf-runs N` repeats every throttled performance pass N times (#791).
//! The unit tests cover the aggregation; this one drives the loop in a real
//! browser against a local server that counts the document loads.
//!
//!   cargo test --test perf_runs_browser -- --ignored

mod common;

use auditmysite::audit::run_single_audit;
use auditmysite::{BrowserManager, BrowserOptions, PipelineConfig, WcagLevel};
use common::routing_server::{serve_routes, Route, Transport};

const PAGE: &str = "<!doctype html><html lang=en><head><title>Perf</title></head>\
<body><main><h1>Perf</h1><p>Some text to paint.</p></main></body></html>";

fn config() -> PipelineConfig {
    PipelineConfig {
        wcag_level: WcagLevel::AA,
        timeout_secs: 30,
        stability_budget_ms: 1500,
        verbose: false,
        full_audit: false,
        check_performance: true,
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

/// Document loads of `/` and the report after a run with `runs` repeats.
fn audit_with_runs(runs: u32) -> (usize, auditmysite::AuditReport) {
    let (tx, rx) = std::sync::mpsc::channel();
    on_big_stack(move || async move {
        let server = serve_routes(
            vec![Route::new("/", 200, PAGE.as_bytes().to_vec())],
            Transport::Http,
        );
        let mut config = config();
        config.perf_runs = runs;
        let browser = browser().await;
        let report = run_single_audit(&server.url, &browser, &config)
            .await
            .expect("audit");
        browser.close().await.ok();
        server.stop();
        let loads = server.requests().iter().filter(|r| *r == "GET /").count();
        tx.send((loads, report)).unwrap();
    });
    rx.recv().unwrap()
}

#[test]
#[ignore = "needs Chrome"]
fn the_perf_runs_loop_repeats_every_throttled_pass() {
    let (single_loads, single) = audit_with_runs(1);
    let (triple_loads, triple) = audit_with_runs(3);

    assert!(
        single
            .throttled_performance
            .iter()
            .all(|r| r.spread.is_none()),
        "one run reports no spread"
    );
    let measured: Vec<_> = triple
        .throttled_performance
        .iter()
        .filter_map(|r| r.spread.as_ref())
        .collect();
    assert!(
        !measured.is_empty(),
        "no throttled profile reported a spread"
    );
    for spread in measured {
        assert_eq!(spread.requested_runs, 3);
        assert!((1..=3).contains(&spread.runs), "runs: {}", spread.runs);
    }
    // Every throttled profile reloads the page once per run: three runs load
    // the document at least two passes per profile more than one run does.
    let profiles = single.throttled_performance.len().max(1);
    assert!(
        triple_loads >= single_loads + 2 * profiles,
        "loads: 1 run = {single_loads}, 3 runs = {triple_loads}, profiles = {profiles}"
    );
}
