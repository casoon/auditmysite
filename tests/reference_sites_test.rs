//! Live reference set: public pages with a manually reviewed expected score
//! band (plan 47, step 2).
//!
//! The fixture bands in `score_calibration_test.rs` pin the scorer on
//! synthetic pages; this pins it on real ones. The expected band of each page
//! comes from a person looking at the page, recorded in
//! `tests/fixtures/reference_sites.json` with the review date. Pages without a
//! review (`expected: null`) are skipped.
//!
//! Run before a release (see CLAUDE.md). Live pages change, so a miss is not
//! automatically a tool bug: re-review the page first, then look at the tool.
//! `#[ignore]`-gated: needs Chrome and network.

use auditmysite::cli::Args;
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize)]
struct ReferenceSet {
    sites: Vec<Site>,
}

#[derive(Deserialize)]
struct Site {
    url: String,
    expected: Option<Band>,
    reviewed: Option<String>,
}

#[derive(Deserialize)]
struct Band {
    min: u32,
    max: u32,
}

#[tokio::test]
#[ignore = "needs real Chrome and network; run before a release"]
async fn reference_sites_stay_in_their_reviewed_bands() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/reference_sites.json");
    let set: ReferenceSet = serde_json::from_str(
        &std::fs::read_to_string(&path).expect("reference_sites.json readable"),
    )
    .expect("reference_sites.json parses");

    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("Browser launch failed");

    let mut misses = Vec::new();
    for site in set.sites.iter().filter(|s| s.expected.is_some()) {
        let band = site.expected.as_ref().expect("filtered");
        let page = manager.new_page().await.expect("New page failed");
        if let Err(e) = manager.navigate(&page, &site.url).await {
            misses.push(format!("{}: unreachable ({e})", site.url));
            continue;
        }
        let args = Args::parse_from(["auditmysite", &site.url]);
        let config = PipelineConfig::from_args_and_config(&args, None);
        let score = match audit_page(&page, &site.url, &config, &manager).await {
            Ok((report, _)) => report.accessibility.score.round() as u32,
            Err(e) => {
                misses.push(format!("{}: audit failed ({e})", site.url));
                continue;
            }
        };
        if !(band.min..=band.max).contains(&score) {
            misses.push(format!(
                "{}: {score}, reviewed band {}–{} ({})",
                site.url,
                band.min,
                band.max,
                site.reviewed.as_deref().unwrap_or("no review date")
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "outside the reviewed band — re-review the page, then the tool:\n{}",
        misses.join("\n")
    );
}
