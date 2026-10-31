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
    let config = PipelineConfig::from_args_and_config(&args, None);
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
