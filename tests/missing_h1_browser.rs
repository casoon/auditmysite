//! #779: a page with `<h2>` but no `<h1>` end to end -- real Chrome against
//! the fixture server, judged on the normalized JSON.
//!
//! The absence is best practice, not a WCAG failure: the shared rule's hint
//! stays out of the violations and the accessibility score. SEO, UX,
//! Journey and the accessibility journey still see it, but as one cause that
//! only SEO deducts for.
// The infallible `normalize` is the deprecated 1.x compatibility API; these tests
// exercise the model it derives, not the migration path.
#![allow(deprecated)]

mod common;

use auditmysite::audit::normalize;
use auditmysite::cli::Args;
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;

const H2_WITHOUT_H1: &str = "<!doctype html><html lang=\"en\"><head><title>Page title</title>\
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"></head>\
    <body><main><h2>Section</h2><p>Some text for the page.</p></main></body></html>";

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test missing_h1_browser -- --ignored`"]
async fn h2_without_h1_is_one_hint_and_one_charged_cause() {
    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch");
    let fixture = common::fixture_server::serve_html(H2_WITHOUT_H1.to_string());
    let page = manager.new_page().await.expect("new page");
    manager
        .navigate(&page, &fixture.url)
        .await
        .expect("navigation");
    let args = Args::parse_from(["auditmysite", fixture.url.as_str(), "--full"]);
    let config = PipelineConfig::from_args_and_config(&args, None);
    let report = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        audit_page(&page, &fixture.url, &config, &manager),
    )
    .await
    .expect("audit finishes in bounded time")
    .expect("audit succeeds")
    .0;
    fixture.stop();

    // Not a WCAG violation, so not a deduction in the accessibility score.
    let violations = &report.accessibility.wcag_results.violations;
    assert!(
        violations
            .iter()
            .all(|v| v.rule_id.as_deref() != Some("headings/h1-missing")
                || v.kind != auditmysite::wcag::Outcome::Fail),
        "missing h1 is still a WCAG failure: {violations:?}"
    );

    let json = serde_json::to_value(&normalize(&report).normalized).expect("json");
    let findings = json["findings"].as_array().expect("findings");
    let same_cause: Vec<_> = findings
        .iter()
        .filter(|f| f["contract"]["cause"] == "page.h1.missing")
        .collect();
    let mut ids: Vec<_> = same_cause
        .iter()
        .map(|f| f["rule_id"].as_str().unwrap())
        .collect();
    ids.sort();
    assert_eq!(
        ids,
        [
            "a11y_journey.heading_missing_h1",
            "journey.no_h1",
            "seo.headings.missing_h1",
            "ux.no_h1"
        ],
        "{findings:#?}"
    );
    let charged: Vec<_> = same_cause
        .iter()
        .filter(|f| f["contract"]["scoring"]["effect"] == "deduction")
        .map(|f| f["rule_id"].as_str().unwrap())
        .collect();
    assert_eq!(charged, ["seo.headings.missing_h1"]);
    // The accessibility area carries no h1 finding with a criterion.
    assert!(
        findings
            .iter()
            .all(|f| f["rule_id"] != "a11y.main_heading.missing"),
        "{findings:#?}"
    );
}
