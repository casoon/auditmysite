//! Integration test for the WCAG 3.2.6 help-mechanism inventory
//! (`patterns::help_mechanisms`, plan 54 §3). Chrome-dependent, run with:
//!   cargo test --test help_mechanisms_detection_test -- --ignored

mod common;

use auditmysite::patterns::help_mechanisms::{HelpKind, HelpMechanism, HelpRegion};
use auditmysite::{audit_page, Args, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;

#[tokio::test]
#[ignore = "needs real Chrome; run manually with `cargo test -- --ignored`"]
async fn inventories_help_mechanisms_outside_main_in_document_order() {
    let html = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/help_mechanisms.html"),
    )
    .expect("fixture");
    let fixture = common::fixture_server::serve_html(html);
    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("Browser launch failed");
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &fixture.url)
        .await
        .expect("Navigation failed");

    let args = Args::parse_from(["auditmysite", &fixture.url]);
    let config = PipelineConfig::from_args_and_config(&args, None);
    let (report, _snapshot) = audit_page(&page, &fixture.url, &config, &manager)
        .await
        .expect("Audit failed");
    fixture.stop();

    let host = fixture
        .url
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let mechanism = |kind, key: String, region| HelpMechanism { kind, key, region };
    assert_eq!(
        report.patterns.expect("patterns").help_mechanisms,
        vec![
            mechanism(
                HelpKind::ContactPage,
                format!("page:{host}/kontakt"),
                HelpRegion::Header
            ),
            mechanism(
                HelpKind::ContactPage,
                format!("page:{host}/faq"),
                HelpRegion::Navigation
            ),
            mechanism(
                HelpKind::Email,
                "mailto:info@example.com".into(),
                HelpRegion::Footer
            ),
            mechanism(
                HelpKind::Phone,
                "tel:+4930123456".into(),
                HelpRegion::Footer
            ),
            mechanism(HelpKind::Chat, "chat:tidio".into(), HelpRegion::Floating),
        ]
    );
}
