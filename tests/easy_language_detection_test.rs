//! Integration tests for easy-language ("Leichte Sprache") availability
//! detection (plan/09-easy-language-detection.md).
//!
//! Chrome-dependent, mirrors `tests/integration_test.rs`'s structure
//! (`serve_fixture`/`ci_browser`/`default_config` duplicated locally rather
//! than shared — same rationale as `tests/detection_corpus_test.rs`). Run
//! with:
//!   cargo test --test easy_language_detection_test -- --ignored

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig, WcagLevel};

/// Serve a local HTML file over HTTP on a random port.
/// Returns the URL (e.g. "http://127.0.0.1:PORT") and a shutdown handle.
///
/// The server itself lives in `common::fixture_server` — six copies of it
/// existed, all carrying the same race (plan 48).
fn serve_fixture(filename: &str) -> (String, Arc<std::sync::atomic::AtomicBool>) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename);
    let html = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path.display(), e));

    let fixture = common::fixture_server::serve_html(html);
    (fixture.url.clone(), fixture.shutdown.clone())
}

async fn ci_browser() -> BrowserManager {
    let opts = BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    };
    BrowserManager::with_options(opts)
        .await
        .expect("Browser launch failed")
}

fn default_config() -> PipelineConfig {
    PipelineConfig {
        wcag_level: WcagLevel::AA,
        timeout_secs: 30,
        stability_budget_ms: 1500,
        verbose: false,
        full_audit: false,
        check_performance: false,
        check_seo: false,
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
        persist_artifacts: false,
        capture_screenshots: false,
        capture_element_evidence: false,
        dismiss_consent: false,
        interactive: auditmysite::cli::InteractiveMode::Off,
        journey_budget_ms: auditmysite::a11y_journey::DEFAULT_BUDGET_MS,
        lang: "de".to_string(),
    }
}

async fn recognizes_easy_language(fixture: &str) -> bool {
    let (url, shutdown) = serve_fixture(fixture);
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let (report, _snapshot) = audit_page(&page, &url, &default_config(), &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    report
        .patterns
        .as_ref()
        .is_some_and(|p| p.recognized.iter().any(|r| r.pattern == "EasyLanguage"))
}

#[tokio::test]
#[ignore]
async fn detects_easy_language_link_text() {
    assert!(
        recognizes_easy_language("easy_language_linktext.html").await,
        "a link with \"Leichte Sprache\" text should be recognized as an easy-language signal"
    );
}

#[tokio::test]
#[ignore]
async fn detects_easy_language_lang_attribute() {
    assert!(
        recognizes_easy_language("easy_language_lang_attribute.html").await,
        "a lang=\"de-x-simplified\" hreflang alternate should be recognized as an easy-language signal"
    );
}

#[tokio::test]
#[ignore]
async fn detects_easy_language_css_class() {
    assert!(
        recognizes_easy_language("easy_language_css_class.html").await,
        "an easy-language-* CSS class should be recognized as an easy-language signal"
    );
}

#[tokio::test]
#[ignore]
async fn does_not_detect_easy_language_on_clean_fixture() {
    assert!(
        !recognizes_easy_language("perfect.html").await,
        "a page with none of the easy-language markers must not be recognized"
    );
}
