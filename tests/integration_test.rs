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
        check_dark_mode: false,
        check_stack: false,
        persist_artifacts: true,
        capture_screenshots: false,
        dismiss_consent: false,
        interactive: auditmysite::cli::InteractiveMode::Off,
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

    let report = audit_page(&page, &url, &default_config(), &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // A well-structured page should score above 70 (no score suppression)
    assert!(
        report.score >= 70.0,
        "Perfect page scored only {:.1}, expected >= 70",
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

    let report = audit_page(&page, &url, &default_config(), &manager)
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

    let report = audit_page(&page, &url, &config, &manager)
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

    let report = audit_page(&page, &url, &default_config(), &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // JSON output should be valid JSON (v2.0 envelope)
    let normalized = auditmysite::audit::normalize(&report);
    let json = auditmysite::format_json_normalized(&normalized, &report, true)
        .expect("JSON formatting failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON output");
    assert_eq!(
        parsed.get("schema_version").and_then(|v| v.as_str()),
        Some("2.0"),
        "JSON should contain schema_version 2.0"
    );
    let pages = parsed
        .get("pages")
        .expect("JSON should contain pages array");
    let page = pages
        .get(0)
        .expect("pages array should have at least one entry");
    assert!(
        page.get("url").is_some(),
        "page entry should contain url field"
    );
    assert!(
        page.get("accessibility_score").is_some(),
        "page entry should contain accessibility_score field"
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

    let report = audit_page(&page, &url, &config, &manager)
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

#[tokio::test]
#[ignore]
async fn test_modern_contrast_resolution() {
    let (url, shutdown) = serve_fixture("modern_contrast.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let report = audit_page(&page, &url, &default_config(), &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // Let's debug print violations to see what we found
    for violation in &report.wcag_results.violations {
        println!(
            "Violation: rule={}, selector={:?}, message={}",
            violation.rule, violation.selector, violation.message
        );
    }

    // Filter violations by contrast rule "1.4.3"
    let contrast_violations: Vec<_> = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "1.4.3")
        .collect();

    // We expect exactly one contrast violation (the deliberate failure box)
    assert!(
        !contrast_violations.is_empty(),
        "Should have detected the deliberate contrast violation"
    );

    // The deliberate violation is inside the container with class modern-low-contrast-fail, or element with class low-contrast-text
    let has_deliberate_violation = contrast_violations.iter().any(|v| {
        v.selector
            .as_deref()
            .map(|s| {
                s.contains("low-contrast-text")
                    || s.contains("violation-fail-box")
                    || s.contains("modern-low-contrast-fail")
            })
            .unwrap_or(false)
    });
    assert!(
        has_deliberate_violation,
        "Deliberate contrast violation was not identified correctly"
    );

    // Ensure that none of the passing boxes (tailwind, oklch, transparency) were flagged as contrast violations.
    let has_false_positives = contrast_violations.iter().any(|v| {
        if let Some(ref s) = v.selector {
            s.contains("tailwind-pass-box")
                || s.contains("oklch-pass-box")
                || s.contains("transparency-pass-box")
                || s.contains("tailwind-space-separated-pass")
                || s.contains("oklch-pass")
                || s.contains("transparent-text-pass")
        } else {
            false
        }
    });
    assert!(
        !has_false_positives,
        "Detected false positives in modern contrast color checks"
    );
}

#[tokio::test]
#[ignore]
async fn test_image_contrast_pixel_sampling() {
    let (url, shutdown) = serve_fixture("image_contrast.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let report = audit_page(&page, &url, &default_config(), &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // Filter contrast rule "1.4.3" violations and warnings
    let contrast_violations: Vec<_> = report
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "1.4.3")
        .collect();

    let contrast_warnings: Vec<_> = report
        .wcag_results
        .warnings
        .iter()
        .filter(|v| v.rule == "1.4.3")
        .collect();

    // Debug output
    for v in &contrast_violations {
        println!(
            "Confirmed Violation: selector={:?}, message={}",
            v.selector, v.message
        );
    }
    for w in &contrast_warnings {
        println!(
            "NeedsReview Warning: selector={:?}, message={}",
            w.selector, w.message
        );
    }

    // Assertions for pass-text: should be absent from both violations and warnings
    let has_pass_violation = contrast_violations.iter().any(|v| {
        v.selector
            .as_deref()
            .map(|s| s.contains("pass-text") || s.contains("gradient-pass-box"))
            .unwrap_or(false)
    });
    let has_pass_warning = contrast_warnings.iter().any(|w| {
        w.selector
            .as_deref()
            .map(|s| s.contains("pass-text") || s.contains("gradient-pass-box"))
            .unwrap_or(false)
    });
    assert!(
        !has_pass_violation,
        "The dark-gradient text should not be a confirmed violation"
    );
    assert!(
        !has_pass_warning,
        "The dark-gradient text should not be a manual review warning (should pass completely)"
    );

    // Assertions for fail-text: should be present in violations, but absent from warnings
    let has_fail_violation = contrast_violations.iter().any(|v| {
        v.selector
            .as_deref()
            .map(|s| s.contains("fail-text") || s.contains("gradient-fail-box"))
            .unwrap_or(false)
    });
    let has_fail_warning = contrast_warnings.iter().any(|w| {
        w.selector
            .as_deref()
            .map(|s| s.contains("fail-text") || s.contains("gradient-fail-box"))
            .unwrap_or(false)
    });
    assert!(
        has_fail_violation,
        "The light-gradient text should be a confirmed violation"
    );
    assert!(
        !has_fail_warning,
        "The light-gradient text should not be a manual review warning (it should fail)"
    );

    // Assertions for warn-text: should be present in warnings, but absent from violations
    let has_warn_violation = contrast_violations.iter().any(|v| {
        v.selector
            .as_deref()
            .map(|s| s.contains("warn-text") || s.contains("gradient-warn-box"))
            .unwrap_or(false)
    });
    let has_warn_warning = contrast_warnings.iter().any(|w| {
        w.selector
            .as_deref()
            .map(|s| s.contains("warn-text") || s.contains("gradient-warn-box"))
            .unwrap_or(false)
    });
    assert!(
        !has_warn_violation,
        "The split-gradient text should not be a confirmed violation"
    );
    assert!(
        has_warn_warning,
        "The split-gradient text should be a manual review warning"
    );
}
