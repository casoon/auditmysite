//! Technical error cases of detectors whose failure no corpus page can
//! provoke (#783): the analyzer's own transport. Rule contracts name these
//! tests in `error_test`; `tests/rule_contract_gate.rs` reads the `fn` names
//! from this file. Every test provokes the failure for real through the
//! production entry point and proves it is reported as not measured, never
//! as a clean result.

mod common;

use auditmysite::audit::collection::{CollectionLog, CollectionState};

/// A host that refuses the connection: `analyze_security` must fail and
/// record the header collection as failed — not return a report whose
/// missing headers would read as measured findings, or as a clean pass.
#[tokio::test]
async fn security_unreachable_server_is_not_measured() {
    let url = common::routing_server::unreachable_origin();
    let log = CollectionLog::new();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        auditmysite::security::analyze_security(&url, &log),
    )
    .await
    .expect("bounded failed collection");
    assert!(
        result.is_err(),
        "a failed fetch must not become measured missing-header findings"
    );
    let records = log.into_records();
    let headers = records
        .iter()
        .find(|r| r.collector == "security.headers")
        .expect("the header collection is recorded");
    assert_eq!(headers.state, CollectionState::Failed);
    assert_eq!(headers.reason_code.as_deref(), Some("http_connect_failed"));
}

/// The tree rules (`image-alt`, `description-duplicates-name`) read the
/// accessibility tree. When the page's target is gone before the audit —
/// a crashed or closed tab — no tree can be taken: the audit must fail as a
/// whole instead of reporting the tree rules as clean.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test rule_error_cases -- --ignored`"]
async fn tree_rules_fail_the_audit_without_an_accessibility_tree() {
    use auditmysite::cli::Args;
    use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
    use clap::Parser;

    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch");
    let fixture = common::fixture_server::serve_html(
        "<!doctype html><html lang=\"en\"><head><title>t</title></head>\
         <body><main><img src=\"a.png\"></main></body></html>"
            .to_string(),
    );
    let page = manager.new_page().await.expect("new page");
    manager
        .navigate(&page, &fixture.url)
        .await
        .expect("navigation");
    page.clone().close().await.expect("close the tab");

    let args = Args::parse_from(["auditmysite", &fixture.url]);
    let config = PipelineConfig::from_args_and_config(&args, None);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        audit_page(&page, &fixture.url, &config, &manager),
    )
    .await
    .expect("a closed tab fails in bounded time");
    fixture.stop();
    assert!(
        result.is_err(),
        "an audit without an accessibility tree must fail, not pass the tree rules"
    );
}
