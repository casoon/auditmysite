//! #1002-#1007: the rendering probes against real Chrome.
//!
//! A field counts as present once a run has seen it filled. Each test serves
//! one local page, runs `accessibility::probes` through
//! `fetch_dom_capture_with_probes`, and asserts that every measurement
//! delivered rows -- and that the shared rules, fed from them, ran
//! (`rule_outcomes` without `not_run`) and reported what the page contains.
//! A measurement that did not come back fills nothing: its rules are not run.

mod common;

use auditmysite::accessibility::{
    fetch_dom_capture_with_probes, AXTree, DomCapture, ProbeSelection,
};
use auditmysite::wcag::shared::run_shared_rules;
use auditmysite::{BrowserManager, BrowserOptions};

const PAGE: &str = r##"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Probe subjects</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
  body { margin: 0; padding: 200px 16px 16px; }
  #bar { position: fixed; top: 0; left: 0; right: 0; height: 120px; background: #222; }
  #under { position: absolute; top: 40px; left: 24px; width: 120px; height: 44px; }
  #badge { height: 20px; line-height: 1.2; overflow: hidden; border: 1px solid #111; padding: 0 4px; width: 260px; }
  .bare { outline: none; border: 1px solid #767676; background: #fff; }
</style></head>
<body>
<a id="under" href="/privacy">Privacy</a>
<div id="bar"></div>
<main>
  <h1>Subjects</h1>
  <input id="pw" type="password" autocomplete="current-password" onpaste="return false">
  <iframe id="far" title="Far" src="{{CROSS_ORIGIN}}/x" width="200" height="80"></iframe>
  <iframe id="near" title="Near" srcdoc="<p>same origin</p>" width="200" height="80"></iframe>
  <div id="badge">Free delivery on all orders</div>
  <button id="bare" class="bare" type="button">Bare</button>
</main>
</body></html>"##;

async fn capture(
    html: &str,
    selection: ProbeSelection,
) -> (DomCapture, common::fixture_server::Fixture) {
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
    let capture = fetch_dom_capture_with_probes(&page, selection)
        .await
        .expect("capture");
    (capture, fixture)
}

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test rendering_probes_browser -- --ignored`"]
async fn every_probe_comes_back_filled_and_feeds_its_rule() {
    let (capture, fixture) = capture(PAGE, ProbeSelection::ALL).await;
    fixture.stop();
    let probes = capture.probes().expect("layout styles");

    let counts = probes.filled();
    assert!(counts.frames.is_some_and(|n| n >= 2), "frames: {counts:?}");
    assert_eq!(counts.transforms, Some(2), "html and body: {counts:?}");
    assert!(counts.paste.is_some_and(|n| n >= 1), "paste: {counts:?}");
    assert!(counts.focus.is_some_and(|n| n >= 3), "focus: {counts:?}");
    assert!(
        counts.spacing.is_some_and(|n| n >= 1),
        "spacing: {counts:?}"
    );
    assert!(probes.notes().is_empty(), "notes: {:?}", probes.notes());

    // What the rows say, not only that there are rows.
    let frames = probes.frames.as_ref().unwrap();
    assert!(frames.values().any(|r| *r), "a readable frame: {frames:?}");
    assert!(
        frames.values().any(|r| !*r),
        "an unreadable frame: {frames:?}"
    );
    assert!(probes
        .paste
        .as_ref()
        .unwrap()
        .values()
        .any(|p| p.cancelled && !p.inserted));
    let focus = probes.focus.as_ref().unwrap();
    assert!(focus
        .values()
        .any(|m| m.points > 0 && m.covered == m.points));
    assert!(
        focus.values().any(|m| m.indicator == Some(false)),
        "a bare control"
    );
    assert!(
        focus.values().any(|m| m.indicator == Some(true)),
        "a control with the browser ring"
    );
    assert!(probes
        .spacing
        .as_ref()
        .unwrap()
        .values()
        .any(|c| !c.before && c.after));

    let doc = capture.document(&AXTree::new()).expect("document");
    let results = run_shared_rules(&doc, "en");
    for id in [
        "frames/cross-origin",
        "text-spacing/clipped",
        "focus/obscured",
        "focus/obscured-enhanced",
        "auth/paste-blocked",
        "focus/indicator-missing",
    ] {
        let run = results
            .rule_outcomes
            .iter()
            .find(|o| o.rule_id == id)
            .unwrap_or_else(|| panic!("no outcome for {id}"));
        assert!(run.not_run.is_none(), "{id} did not run: {run:?}");
        assert!(run.findings >= 1, "{id} found nothing: {run:?}");
    }
    // The root is not rotated here: the rule ran and has nothing to say.
    let run = results
        .rule_outcomes
        .iter()
        .find(|o| o.rule_id == "orientation/root-rotated")
        .expect("outcome");
    assert!(run.not_run.is_none() && run.findings == 0, "{run:?}");
}

/// A rotated root, in its own page: a rotated body would carry every other
/// subject off screen.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test rendering_probes_browser -- --ignored`"]
async fn a_rotated_root_is_measured_and_reported() {
    let html = r#"<!doctype html><html lang="en" style="transform: rotate(90deg)"><head>
<meta charset="utf-8"><title>t</title></head><body><main><h1>x</h1></main></body></html>"#;
    let (capture, fixture) = capture(html, ProbeSelection::ALL).await;
    fixture.stop();
    let rows = capture.probes().expect("layout styles").transforms.clone();
    let rows = rows.expect("transforms measured");
    assert!(
        rows.values().any(|t| t
            .transform
            .as_deref()
            .is_some_and(|m| m.starts_with("matrix("))),
        "{rows:?}"
    );
    let doc = capture.document(&AXTree::new()).expect("document");
    let results = run_shared_rules(&doc, "en");
    assert!(
        results
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("orientation/root-rotated")),
        "{:?}",
        results.violations
    );
}

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test rendering_probes_browser -- --ignored`"]
async fn unselected_probes_leave_their_rules_not_run() {
    let (capture, fixture) = capture(PAGE, ProbeSelection::NONE).await;
    fixture.stop();
    let probes = capture.probes().expect("layout styles");
    let counts = probes.filled();
    assert_eq!(
        (
            counts.frames,
            counts.transforms,
            counts.paste,
            counts.focus,
            counts.spacing
        ),
        (None, None, None, None, None)
    );

    let doc = capture.document(&AXTree::new()).expect("document");
    let results = run_shared_rules(&doc, "en");
    for id in [
        "frames/cross-origin",
        "orientation/root-rotated",
        "text-spacing/clipped",
        "focus/obscured-enhanced",
        "auth/paste-blocked",
        "focus/indicator-missing",
    ] {
        let run = results
            .rule_outcomes
            .iter()
            .find(|o| o.rule_id == id)
            .unwrap_or_else(|| panic!("no outcome for {id}"));
        assert_eq!(
            run.not_run,
            Some(auditmysite::wcag::NotRun::CapabilityMissing),
            "{id} must be not run without its measurement: {run:?}"
        );
    }
    // No row, no finding.
    assert!(
        results
            .violations
            .iter()
            .chain(&results.warnings)
            .all(|v| !matches!(
                v.rule_id.as_deref(),
                Some("auth/paste-blocked" | "orientation/root-rotated" | "text-spacing/clipped")
            )),
        "{:?}",
        results.violations
    );
}

/// The page wins the override with a layered `!important`: no element is
/// measured, the measurement fails, and the rule stays not run -- never a pass.
#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test rendering_probes_browser -- --ignored`"]
async fn a_defeated_spacing_override_measures_nothing() {
    let html = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>t</title>
<style>@layer lock { * { letter-spacing: 0 !important; line-height: 1.2 !important; } }
#badge { height: 20px; overflow: hidden; width: 200px; }</style></head>
<body><main><h1>x</h1><div id="badge">Free delivery on all orders</div></main></body></html>"#;
    let (capture, fixture) = capture(html, ProbeSelection::ALL).await;
    fixture.stop();
    let probes = capture.probes().expect("layout styles");
    assert_eq!(probes.filled().spacing, None);
    assert!(
        probes
            .notes()
            .iter()
            .any(|n| n.probe == "spacing" && n.text.starts_with("probe_failed")),
        "{:?}",
        probes.notes()
    );
    let doc = capture.document(&AXTree::new()).expect("document");
    let results = run_shared_rules(&doc, "en");
    let run = results
        .rule_outcomes
        .iter()
        .find(|o| o.rule_id == "text-spacing/clipped")
        .expect("outcome");
    assert_eq!(
        run.not_run,
        Some(auditmysite::wcag::NotRun::CapabilityMissing)
    );
    assert!(run
        .reason
        .as_deref()
        .is_some_and(|r| r.contains("probe_failed")));
}
