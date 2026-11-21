//! Live reference set: public pages with a manually reviewed expected score
//! band.
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
//!
//! A page outside its band is audited a second time (#1110). Geography,
//! language redirects and consent platforms change what a live page shows
//! from one run to the next, so a page that lands in its band on the second
//! run is reported as *unstable* with the finding ids that differ between the
//! two runs, not as a deviation; only a page outside its band in both runs
//! fails the test. Unstable pages are printed, so run with `--nocapture` to
//! see them when nothing else failed. What each run saw (final URL, redirect
//! chain, `Accept-Language`, consent) is in `pages[].navigation` and
//! `pages[].consent` of the JSON report.

use std::collections::BTreeSet;

use auditmysite::audit::normalize_checked;
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

/// What one audit of a page produced: the rounded accessibility score and the
/// rule ids of its findings.
struct Run {
    score: u32,
    findings: BTreeSet<String>,
}

async fn audit_once(manager: &BrowserManager, url: &str) -> Result<Run, String> {
    let page = manager.new_page().await.expect("New page failed");
    if let Err(e) = manager.navigate(&page, url).await {
        let _ = page.close().await;
        return Err(format!("unreachable ({e})"));
    }
    let args = Args::parse_from(["auditmysite", url]);
    let config = PipelineConfig::from_args_and_config(&args, None);
    // Boxed: the audit future is large, and two of them inline in the test
    // body overflow the test thread's stack in a debug build.
    let audited = Box::pin(audit_page(&page, url, &config, manager)).await;
    // Close before the next site: every page stays visible,
    // and a finished live site left open keeps running its scripts.
    let _ = page.close().await;
    let (report, _) = audited.map_err(|e| format!("audit failed ({e})"))?;
    let findings = normalize_checked(&report)
        .map_err(|e| format!("report contract violated ({e:?})"))?
        .normalized
        .findings
        .iter()
        .map(|f| f.rule_id.clone())
        .collect();
    Ok(Run {
        score: report.accessibility.score.round() as u32,
        findings,
    })
}

/// How a page that left its band in the first run fared in the second.
enum Judgement {
    /// Outside the band in both runs.
    Deviation(String),
    /// Back inside the band in the second run.
    Unstable(String),
}

/// Rule ids present in exactly one of the two runs, sorted.
fn differing_findings(first: &Run, second: &Run) -> Vec<String> {
    first
        .findings
        .symmetric_difference(&second.findings)
        .cloned()
        .collect()
}

fn judge(url: &str, band: &Band, reviewed: &str, first: &Run, second: &Run) -> Judgement {
    let differing = differing_findings(first, second);
    let ids = if differing.is_empty() {
        "the finding ids are the same in both runs".to_string()
    } else {
        format!("ids that differ between the runs: {}", differing.join(", "))
    };
    if (band.min..=band.max).contains(&second.score) {
        Judgement::Unstable(format!(
            "{url}: first run {}, second run {}, reviewed band {}–{} ({reviewed}); {ids}",
            first.score, second.score, band.min, band.max
        ))
    } else {
        Judgement::Deviation(format!(
            "{url}: {} and {} in two runs, reviewed band {}–{} ({reviewed}); {ids}",
            first.score, second.score, band.min, band.max
        ))
    }
}

#[test]
fn a_page_back_in_its_band_on_the_second_run_is_unstable_not_a_deviation() {
    let band = Band { min: 80, max: 90 };
    let run = |score: u32, ids: &[&str]| Run {
        score,
        findings: ids.iter().map(|s| s.to_string()).collect(),
    };
    let first = run(70, &["aria/required-children-missing", "x/stable"]);
    let back = run(85, &["x/stable"]);
    match judge("https://a.test/", &band, "2026-11-17", &first, &back) {
        Judgement::Unstable(message) => {
            assert!(
                message.contains("aria/required-children-missing"),
                "{message}"
            );
            assert!(!message.contains("x/stable"), "{message}");
        }
        Judgement::Deviation(message) => panic!("a recovered page is unstable: {message}"),
    }
    let still_out = run(70, &["x/stable", "aria/required-children-missing"]);
    match judge("https://a.test/", &band, "2026-11-17", &first, &still_out) {
        Judgement::Deviation(message) => {
            assert!(
                message.contains("the finding ids are the same"),
                "{message}"
            );
        }
        Judgement::Unstable(message) => panic!("two misses are a deviation: {message}"),
    }
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
    let mut unstable = Vec::new();
    for site in set.sites.iter().filter(|s| s.expected.is_some()) {
        let band = site.expected.as_ref().expect("filtered");
        let reviewed = site.reviewed.as_deref().unwrap_or("no review date");
        let first = match Box::pin(audit_once(&manager, &site.url)).await {
            Ok(run) => run,
            Err(e) => {
                misses.push(format!("{}: {e}", site.url));
                continue;
            }
        };
        if (band.min..=band.max).contains(&first.score) {
            continue;
        }
        // Outside the band: a second look tells a flaky page from a deviation.
        match Box::pin(audit_once(&manager, &site.url)).await {
            Ok(second) => match judge(&site.url, band, reviewed, &first, &second) {
                Judgement::Deviation(message) => misses.push(message),
                Judgement::Unstable(message) => unstable.push(message),
            },
            Err(e) => misses.push(format!(
                "{}: {}, reviewed band {}–{} ({reviewed}); the second run failed: {e}",
                site.url, first.score, band.min, band.max
            )),
        }
    }
    if !unstable.is_empty() {
        println!(
            "unstable (outside the band once, inside on the second run):\n{}",
            unstable.join("\n")
        );
    }
    assert!(
        misses.is_empty(),
        "outside the reviewed band in both runs — re-review the page, then the tool:\n{}",
        misses.join("\n")
    );
}
