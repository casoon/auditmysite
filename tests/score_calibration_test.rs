//! Score bands for fixtures whose intended severity is uncontroversial (plan 47).
//!
//! The scorer's constants were each tuned to fix a symptom, and nothing
//! checked the result against an expectation set from outside the scorer. These
//! bands are that expectation, decided on 2026-09-26 in certificate terms:
//!
//! - a practically clean page: ≥ 95;
//! - exactly one High failure of a Level A criterion, otherwise (nearly) clean:
//!   75–89 — not conformant, so not the top band (SEHR GUT ≥ 90);
//! - at least one Critical failure: ≤ 49.
//!
//! Bands, not exact values, so tuning stays possible. Runs the real pipeline at
//! the default level (AA), like a user's audit. `#[ignore]`-gated: needs Chrome.

mod common;

use auditmysite::cli::Args;
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;
use common::fixture_server::serve_html;

const CLEAN: &[&str] = &["wcag_fixtures/perfect.html"];

const ONE_LEVEL_A_HIGH: &[&str] = &[
    "wcag_fixtures/missing_image_alt.html",
    "fixtures/detection_corpus/svg_no_title.html",
    "fixtures/detection_corpus/object_no_alt.html",
    "fixtures/detection_corpus/frame_missing_title.html",
    "fixtures/detection_corpus/meta_refresh_present.html",
    "fixtures/detection_corpus/empty_title.html",
    "fixtures/detection_corpus/table_missing_caption.html",
    "fixtures/detection_corpus/summary_empty.html",
    "wcag_fixtures/invalid_aria.html",
    "fixtures/detection_corpus/aria_invalid_attr_value.html",
    "fixtures/detection_corpus/table_headers_no_data.html",
    "fixtures/detection_corpus/patterns_disclosure.html",
];

const WITH_CRITICAL: &[&str] = &[
    "fixtures/detection_corpus/aria_and_widgets.html",
    "fixtures/detection_corpus/aria_attribute_validation.html",
    "fixtures/detection_corpus/aria_naming_roles.html",
    "fixtures/detection_corpus/combobox_missing_expanded.html",
    "fixtures/detection_corpus/media_and_motion.html",
    "fixtures/detection_corpus/misc_content_checks.html",
    "fixtures/detection_corpus/missing_label.html",
    "fixtures/detection_corpus/widget_patterns.html",
];

async fn score(manager: &BrowserManager, fixture: &str) -> u32 {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(fixture);
    let html = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let server = serve_html(html);
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &server.url)
        .await
        .expect("Navigation failed");
    let args = Args::parse_from(["auditmysite", &server.url]);
    let config = PipelineConfig::from_args_and_config(&args, None);
    let (report, _) = audit_page(&page, &server.url, &config, manager)
        .await
        .unwrap_or_else(|e| panic!("audit failed for {fixture}: {e}"));
    server.stop();
    report.accessibility.score.round() as u32
}

#[tokio::test]
#[ignore = "needs real Chrome; run manually with `cargo test -- --ignored`"]
async fn fixture_scores_fall_into_their_calibrated_bands() {
    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("Browser launch failed");

    let mut misses = Vec::new();
    for (fixtures, min, max, band) in [
        (CLEAN, 95, 100, "clean"),
        (ONE_LEVEL_A_HIGH, 75, 89, "one Level A high"),
        (WITH_CRITICAL, 0, 49, "critical"),
    ] {
        for fixture in fixtures {
            let s = score(&manager, fixture).await;
            if !(min..=max).contains(&s) {
                misses.push(format!("{fixture}: {s}, expected {min}–{max} ({band})"));
            }
        }
    }
    assert!(
        misses.is_empty(),
        "outside their band:\n{}",
        misses.join("\n")
    );
}
