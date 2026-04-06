//! Integration tests for the auditmysite audit pipeline.
//!
//! These tests require Chrome/Chromium to be installed and are marked `#[ignore]`
//! so they don't run in CI by default. Run with:
//!   cargo test --test integration_test -- --ignored

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use auditmysite::{audit_page, BrowserManager, BrowserOptions, PipelineConfig, WcagLevel};

/// Serve a local HTML file over HTTP on a random port.
/// Returns the URL (e.g. "http://127.0.0.1:PORT") and a shutdown handle.
fn serve_fixture(filename: &str) -> (String, Arc<std::sync::atomic::AtomicBool>) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename);
    let html = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path.display(), e));

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}", port);

    let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        loop {
            if shutdown_clone.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 1024];
                    let _ = stream.read(&mut buf);

                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        html.len(),
                        html
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    (url, shutdown)
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
        verbose: false,
        check_performance: false,
        check_seo: false,
        check_security: false,
        check_mobile: false,
        persist_artifacts: true,
    }
}

#[tokio::test]
#[ignore]
async fn test_perfect_page_scores_high() {
    let (url, shutdown) = serve_fixture("perfect.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let report = audit_page(&page, &url, &default_config())
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // A well-structured page should score above 80
    assert!(
        report.score >= 80.0,
        "Perfect page scored only {:.1}, expected >= 80",
        report.score
    );

    // Should have few or no critical violations
    let critical_count = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.severity == auditmysite::Severity::Critical)
        .count();
    assert!(
        critical_count == 0,
        "Perfect page has {} critical violations",
        critical_count
    );
}

#[tokio::test]
#[ignore]
async fn test_many_violations_page_scores_low() {
    let (url, shutdown) = serve_fixture("many_violations.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let report = audit_page(&page, &url, &default_config())
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // Page with many issues should score below 70
    assert!(
        report.score < 70.0,
        "Violation-heavy page scored {:.1}, expected < 70",
        report.score
    );

    // Should have multiple violations
    assert!(
        report.wcag_results.violations.len() >= 3,
        "Expected at least 3 violations, got {}",
        report.wcag_results.violations.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_full_audit_with_all_modules() {
    let (url, shutdown) = serve_fixture("perfect.html");

    let config = PipelineConfig {
        check_performance: true,
        check_seo: true,
        check_security: true,
        check_mobile: true,
        ..default_config()
    };

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let report = audit_page(&page, &url, &config)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // All modules should have results
    assert!(
        report.performance.is_some(),
        "Performance results should be present"
    );
    assert!(report.seo.is_some(), "SEO results should be present");
    assert!(
        report.security.is_some(),
        "Security results should be present"
    );
    assert!(report.mobile.is_some(), "Mobile results should be present");

    // Overall score should be calculated
    let overall = report.overall_score();
    assert!(
        overall > 0 && overall <= 100,
        "Overall score {} out of range",
        overall
    );
}

#[tokio::test]
#[ignore]
async fn test_output_formats() {
    let (url, shutdown) = serve_fixture("perfect.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let report = audit_page(&page, &url, &default_config())
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // JSON output should be valid JSON
    let normalized = auditmysite::audit::normalize(&report);
    let json = auditmysite::format_json_normalized(&normalized, &report, true)
        .expect("JSON formatting failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON output");
    let report_obj = parsed
        .get("report")
        .expect("JSON should contain report field");
    assert!(
        report_obj.get("url").is_some(),
        "JSON should contain url field"
    );
    assert!(
        report_obj.get("score").is_some(),
        "JSON should contain score field"
    );
}

#[tokio::test]
#[ignore]
async fn test_mobile_issues_detected() {
    let (url, shutdown) = serve_fixture("mobile_issues.html");

    let config = PipelineConfig {
        check_mobile: true,
        ..default_config()
    };

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let report = audit_page(&page, &url, &config)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let mobile = report.mobile.expect("Mobile analysis should be present");

    // Missing viewport should be detected
    assert!(
        !mobile.viewport.has_viewport,
        "Should detect missing viewport"
    );

    // Should have mobile issues
    assert!(
        !mobile.issues.is_empty(),
        "Should detect mobile issues, got none"
    );

    // Score should be penalized
    assert!(
        mobile.score < 100,
        "Mobile score should be < 100 with issues, got {}",
        mobile.score
    );
}
