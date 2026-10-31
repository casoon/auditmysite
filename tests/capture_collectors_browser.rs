//! #777: the capture collectors report what the browser really gave them.
//!
//! Real Chrome against the fixture server. A page that breaks one capture
//! step on purpose must end with that collection failed — not with an empty
//! snapshot that reads as "no redirects", "no frames" or "nothing excluded".

mod common;

use auditmysite::audit::collection::CollectionState;
use auditmysite::audit::AuditReport;
use auditmysite::cli::Args;
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;

async fn audit(html: &str, extra_args: &[&str]) -> Box<AuditReport> {
    audit_with(html, extra_args, |_| {}).await
}

async fn audit_with(
    html: &str,
    extra_args: &[&str],
    tweak: impl FnOnce(&mut PipelineConfig),
) -> Box<AuditReport> {
    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch");
    let fixture = common::fixture_server::serve_html(html.to_string());
    let page = manager.new_page().await.expect("new page");
    manager
        .navigate(&page, &fixture.url)
        .await
        .expect("navigation");
    let mut argv = vec!["auditmysite", fixture.url.as_str()];
    argv.extend_from_slice(extra_args);
    let args = Args::parse_from(argv);
    let mut config = PipelineConfig::from_args_and_config(&args, None);
    tweak(&mut config);
    let report = tokio::time::timeout(
        std::time::Duration::from_secs(90),
        audit_page(&page, &fixture.url, &config, &manager),
    )
    .await
    .expect("audit finishes in bounded time")
    .expect("audit succeeds");
    assert!(fixture.served() > 0, "the fixture answered nothing");
    fixture.stop();
    Box::new(report.0)
}

fn collection(report: &AuditReport, id: &str) -> Vec<CollectionState> {
    report
        .accessibility
        .execution
        .module_runs
        .iter()
        .flat_map(|run| &run.subchecks)
        .filter(|s| s.subcheck == id)
        .filter_map(|s| s.state)
        .collect()
}

const CLEAN: &str = "<!doctype html><html lang=\"en\"><head><title>t</title></head>\
    <body><main><h1>Hello</h1><p>text</p></main></body></html>";

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test capture_collectors_browser -- --ignored`"]
async fn a_clean_page_reports_its_captures_as_measured_or_empty() {
    let report = audit(CLEAN, &[]).await;
    for (id, expected) in [
        (
            "accessibility.navigation_snapshot",
            CollectionState::Measured,
        ),
        ("accessibility.ax_tree", CollectionState::Measured),
        ("accessibility.consent_banner", CollectionState::Empty),
        ("accessibility.exclusion_scope", CollectionState::Empty),
        ("accessibility.frames", CollectionState::Empty),
        ("accessibility.rule_execution", CollectionState::Measured),
    ] {
        let states = collection(&report, id);
        assert!(!states.is_empty(), "{id} was not recorded");
        assert!(
            states.iter().all(|s| *s == expected),
            "{id}: {states:?}, expected {expected:?}"
        );
    }
}

/// The navigation script reads `performance`; a page that makes it throw
/// must leave a failed collection, not a snapshot with zero redirects.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test capture_collectors_browser -- --ignored`"]
async fn a_broken_navigation_probe_is_a_failed_collection_not_zero_redirects() {
    let html = "<!doctype html><html lang=\"en\"><head><title>t</title>\
        <script>performance.getEntriesByType = function () { throw new Error('blocked'); };</script>\
        </head><body><main><h1>Hello</h1></main></body></html>";
    let report = audit(html, &[]).await;
    let states = collection(&report, "accessibility.navigation_snapshot");
    assert!(
        !states.is_empty(),
        "the navigation capture was not recorded"
    );
    assert!(
        states.iter().all(|s| *s == CollectionState::Failed),
        "{states:?}"
    );
    assert_eq!(
        report
            .accessibility
            .execution
            .navigation
            .main_document_status,
        None
    );
    let run = report
        .accessibility
        .execution
        .module_runs
        .iter()
        .find(|run| run.module == "accessibility")
        .expect("accessibility run");
    assert_ne!(
        run.status,
        auditmysite::audit::ExecutionStatus::Completed,
        "a lost capture must not leave a completed run"
    );
}

/// A selector the browser rejects cannot be counted. It is published as
/// invalid, never as a selector that matched nothing.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test capture_collectors_browser -- --ignored`"]
async fn a_selector_the_browser_rejects_is_published_as_invalid_not_as_unmatched() {
    let report = audit(CLEAN, &["--exclude-selector", "div[[["]).await;
    let exclusions = report
        .accessibility
        .execution
        .exclusions
        .as_ref()
        .expect("exclusions recorded");
    let selector = exclusions
        .selectors
        .iter()
        .find(|s| s.selector == "div[[[")
        .expect("the rejected selector is published");
    assert!(selector.invalid, "{selector:?}");
    assert_eq!(selector.matched_elements, 0);
}

/// Real probe failure for a scored module: a page that makes the timing and
/// observer probes throw leaves Performance without a measured score, and
/// the overall score does not carry a number the probes never produced.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test capture_collectors_browser -- --ignored`"]
async fn broken_performance_probes_leave_no_measured_performance_score() {
    let html = "<!doctype html><html lang=\"en\"><head><title>t</title>\
        <script>performance.getEntriesByType = function () { throw new Error('blocked'); };\
        window.PerformanceObserver = function () { throw new Error('blocked'); };</script>\
        </head><body><main><h1>Hello</h1></main></body></html>";
    let report = audit(html, &["--performance"]).await;
    let normalized = auditmysite::audit::normalize_checked(&report)
        .expect("a lost probe is honest partial data")
        .normalized;
    let perf = normalized
        .module_scores
        .iter()
        .find(|m| m.name == "Performance")
        .expect("Performance entry");
    assert_ne!(perf.measurement_type, "measured", "{perf:?}");
    assert!(!perf.contributes_to_overall, "{perf:?}");
}

/// #778 on a real page: an audited iframe reports each rule the scope names,
/// once per viewport, and the scope records the rule filter that narrowed it.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test capture_collectors_browser -- --ignored`"]
async fn an_audited_frame_accounts_for_every_frame_rule_in_every_viewport() {
    let html = "<!doctype html><html lang=\"en\"><head><title>t</title></head><body><main>\
        <h1>Hello</h1><iframe title=\"w\" srcdoc=\"&lt;!doctype html&gt;&lt;html lang=en&gt;\
        &lt;title&gt;f&lt;/title&gt;&lt;main&gt;&lt;img src=x.png&gt;&lt;/main&gt;&lt;/html&gt;\">\
        </iframe></main></body></html>";
    let report = audit_with(html, &[], |config| {
        config.rule_filter.disabled_rules = vec!["keyboard-trap".to_string()];
    })
    .await;
    let execution = &report.accessibility.execution;
    let scope = &execution.scope;
    assert_eq!(
        scope.rule_filter.as_ref().map(|f| f.disabled_rules.clone()),
        Some(vec!["keyboard-trap".to_string()])
    );
    assert!(!scope.frame_rules.iter().any(|r| r == "keyboard-trap"));
    let frames = execution.frames.as_ref().expect("frame coverage");
    assert_eq!(frames.audited.len(), 1, "{frames:?}");
    let frame = &frames.audited[0];
    assert!(!frame.viewports.is_empty());
    for viewport in &frame.viewports {
        for rule in &scope.frame_rules {
            let n = frame
                .outcomes
                .iter()
                .filter(|o| &o.rule_id == rule && o.viewport.as_deref() == Some(viewport))
                .count();
            assert_eq!(n, 1, "{rule}@{viewport}");
        }
    }
}
