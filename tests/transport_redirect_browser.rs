//! #775: transport statements follow the served document, not the URL that
//! was asked for -- real Chrome against the routing fixture.
//!
//! An `http://` entry that redirects to `https://` must end as an HTTPS
//! document: the final URL says so, the captured `redirect_chain` holds the
//! hop, and the AI-citation and source-quality modules read the transport
//! from it. The fixture's certificate is issued by a corpus-only CA that
//! Chrome does not know; the test starts Chrome through a wrapper script that
//! adds `--ignore-certificate-errors`, so no product code is relaxed.

#![cfg(unix)]

mod common;

use auditmysite::cli::Args;
use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig};
use clap::Parser;
use common::routing_server::{serve_routes, Route, Transport};
use serde_json::Value;

const PAGE: &str = "<!doctype html><html lang=\"en\"><head><title>Secure page</title>\
    </head><body><main><h1>Hello</h1><p>Some text for the page.</p></main></body></html>";

/// A script that starts the detected Chrome with certificate errors ignored.
fn trusting_chrome(dir: &std::path::Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    let real = auditmysite::browser::find_chrome(None)
        .expect("chrome")
        .path;
    let script = dir.join("chrome-ignore-certs.sh");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\nexec \"{}\" --ignore-certificate-errors \"$@\"\n",
            real.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script.to_string_lossy().into_owned()
}

/// Every object in `value` whose `kind` is `kind`.
fn signals<'a>(value: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if map
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|k| k.eq_ignore_ascii_case(kind))
            {
                out.push(value);
            }
            map.values().for_each(|v| signals(v, kind, out));
        }
        Value::Array(items) => items.iter().for_each(|v| signals(v, kind, out)),
        _ => {}
    }
}

#[tokio::test]
#[ignore = "needs real Chrome; run with `cargo test --test transport_redirect_browser -- --ignored`"]
async fn http_entry_redirecting_to_https_is_judged_as_https() {
    let secure = serve_routes(vec![Route::new("/page", 200, PAGE)], Transport::Https);
    let target = format!("{}/page", secure.url);
    let plain = serve_routes(
        vec![Route::new("/", 301, "").header("Location", &target)],
        Transport::Http,
    );
    let entry = format!("{}/", plain.url);
    assert!(entry.starts_with("http://"));

    let dir = std::env::temp_dir().join(format!("amsite-775-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let manager = BrowserManager::with_options(BrowserOptions {
        chrome_path: Some(trusting_chrome(&dir)),
        no_sandbox: std::env::var("CI").is_ok(),
        ..Default::default()
    })
    .await
    .expect("browser launch");
    let page = manager.new_page().await.expect("new page");
    manager.navigate(&page, &entry).await.expect("navigation");
    let args = Args::parse_from(["auditmysite", entry.as_str(), "--full"]);
    let config = PipelineConfig::from_args_and_config(&args, None);
    let report = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        audit_page(&page, &entry, &config, &manager),
    )
    .await
    .expect("audit finishes in bounded time")
    .expect("audit succeeds")
    .0;
    plain.stop();
    secure.stop();
    let _ = std::fs::remove_dir_all(&dir);

    let nav = &report.accessibility.execution.navigation;
    assert_eq!(nav.final_url.as_deref(), Some(target.as_str()), "{nav:?}");
    assert_eq!(nav.redirect_chain.len(), 1, "{nav:?}");
    assert_eq!(nav.redirect_chain[0].url, entry);
    assert_eq!(nav.redirect_chain[0].status, 301);
    assert!(report.document_is_https());

    let ai =
        serde_json::to_value(auditmysite::ai_visibility::analyze_ai_visibility(&report)).unwrap();
    let mut found = Vec::new();
    signals(&ai, "encryption", &mut found);
    assert!(!found.is_empty(), "no encryption signal");
    assert!(found.iter().all(|s| s["present"] == true), "{found:?}");
    let sq =
        serde_json::to_value(auditmysite::source_quality::analyze_source_quality(&report)).unwrap();
    let mut found = Vec::new();
    signals(&sq, "https", &mut found);
    assert!(!found.is_empty(), "no https signal");
    assert!(found.iter().all(|s| s["present"] == true), "{found:?}");
}
