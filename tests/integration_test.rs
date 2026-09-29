//! Integration tests for the auditmysite audit pipeline.
//!
//! These tests require Chrome/Chromium to be installed and are marked `#[ignore]`
//! so they don't run in CI by default. Run with:
//!   cargo test --test integration_test -- --ignored

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

#[tokio::test]
#[ignore = "needs Chrome"]
async fn chrome_axtree_exposes_svg_graphics_roles_and_the_rule_checks_names() {
    let (url, shutdown) = serve_fixture("svg_graphics_roles.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");
    let tree = auditmysite::accessibility::extract_ax_tree(&page)
        .await
        .expect("AXTree extraction failed");
    let roles = tree
        .iter()
        .filter_map(|node| node.role.as_deref())
        .collect::<Vec<_>>();
    assert!(
        roles
            .iter()
            .any(|role| { matches!(*role, "graphics-document" | "graphics-symbol") }),
        "Chrome AXTree did not expose a graphics-* role: {roles:?}"
    );
    // #563: graphics-document/graphics-symbol are unambiguous AX roles (never
    // shared with a native <img>), so check_svg_rules owns this case under
    // the svg-img-alt axe_id — moved out of check_text_alternatives.
    let results = auditmysite::wcag::rules::check_svg_rules(&tree);
    assert!(results.violations.iter().any(|finding| {
        finding.role.as_deref() == Some("graphics-symbol") || finding.message.contains("graphic")
    }));
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
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
        persist_artifacts: true,
        capture_screenshots: false,
        capture_element_evidence: false,
        dismiss_consent: false,
        exclude_selectors: Vec::new(),
        interactive: auditmysite::cli::InteractiveMode::Off,
        journey_budget_ms: auditmysite::a11y_journey::DEFAULT_BUDGET_MS,
        lang: "de".to_string(),
    }
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_wcag_parity_gaps_on_stable_fixture() {
    let (url, shutdown) = serve_fixture("parity_gaps.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let direct_title_findings = auditmysite::wcag::rules::check_page_titled_with_page(&page).await;
    assert!(
        direct_title_findings
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("document-title")),
        "direct DOM document-title check should trigger; got {direct_title_findings:?}"
    );

    let config = PipelineConfig {
        persist_artifacts: false,
        ..default_config()
    };

    let (report, _snapshot) = audit_page(&page, &url, &config, &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let normalized = auditmysite::audit::normalize(&report).normalized;
    let axe_ids: std::collections::BTreeSet<_> = normalized
        .findings
        .iter()
        .filter_map(|finding| finding.axe_id.as_deref())
        .collect();
    let raw_rule_ids: Vec<_> = report
        .accessibility
        .wcag_results
        .violations
        .iter()
        .map(|v| (v.rule.as_str(), v.rule_id.as_deref()))
        .collect();

    assert!(
        axe_ids.contains("document-title"),
        "missing title fixture should trigger document-title; got {axe_ids:?}; raw {raw_rule_ids:?}"
    );
    assert!(
        axe_ids.contains("landmark-main-present") || axe_ids.contains("landmark-one-main"),
        "missing main landmark fixture should trigger a main-landmark finding; got {axe_ids:?}"
    );
    assert!(
        axe_ids.contains("landmark-unique"),
        "duplicate navigation names should trigger landmark-unique; got {axe_ids:?}"
    );
    // One occurrence per defect: the missing main once, each of the two
    // same-named navs once (the AX and a former DOM check both reported them).
    let count = |id: &str| {
        raw_rule_ids
            .iter()
            .filter(|(_, rule_id)| *rule_id == Some(id))
            .count()
    };
    assert_eq!(
        count("landmark-main-present"),
        1,
        "missing main must be reported once; raw {raw_rule_ids:?}"
    );
    assert_eq!(
        count("landmark-unique"),
        2,
        "each duplicate nav must be reported once; raw {raw_rule_ids:?}"
    );
    assert!(
        axe_ids.contains("aria-hidden-focus"),
        "focusable element in aria-hidden subtree should trigger aria-hidden-focus; got {axe_ids:?}"
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_perfect_page_scores_high() {
    let (url, shutdown) = serve_fixture("perfect.html");

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

    // A well-structured page should score above 70 (no score suppression)
    assert!(
        report.accessibility.score >= 70.0,
        "Perfect page scored only {:.1}, expected >= 70",
        report.accessibility.score
    );

    // Should have few or no critical violations
    let critical_count = report
        .accessibility
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
#[ignore = "needs Chrome"]
async fn test_many_violations_page_scores_low() {
    let (url, shutdown) = serve_fixture("many_violations.html");

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

    // Page with many issues should score below 70
    assert!(
        report.accessibility.score < 70.0,
        "Violation-heavy page scored {:.1}, expected < 70",
        report.accessibility.score
    );

    // Should have multiple violations
    assert!(
        report.accessibility.wcag_results.violations.len() >= 3,
        "Expected at least 3 violations, got {}",
        report.accessibility.wcag_results.violations.len()
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
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

    let (report, _snapshot) = audit_page(&page, &url, &config, &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    // All modules should have results
    assert!(
        report.performance.is_some(),
        "Performance results should be present"
    );
    assert!(
        report.discoverability.seo.is_some(),
        "SEO results should be present"
    );
    assert!(
        report.security.is_some(),
        "Security results should be present"
    );
    assert!(
        report.experience.mobile.is_some(),
        "Mobile results should be present"
    );

    // Overall score should be calculated
    let overall = auditmysite::audit::normalize(&report)
        .normalized
        .overall_score;
    assert!(
        overall > 0 && overall <= 100,
        "Overall score {} out of range",
        overall
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_output_formats() {
    let (url, shutdown) = serve_fixture("perfect.html");

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
#[ignore = "needs Chrome"]
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

    let (report, _snapshot) = audit_page(&page, &url, &config, &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let mobile = report
        .experience
        .mobile
        .expect("Mobile analysis should be present");

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

/// Plan 51: "little visible text in the upper area of the page" is a claim
/// about the rendered page, and is now measured there. The old proxy — the
/// first 50 nodes of the accessibility tree in document order — said the
/// opposite on both of these fixtures: tree order puts the heading first
/// either way, so neither page could be told from the other.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_early_text_is_measured_above_the_fold_not_in_tree_order() {
    async fn flagged(fixture: &'static str) -> bool {
        use auditmysite::journey::FrictionKind;

        let (url, shutdown) = serve_fixture(fixture);
        let manager = ci_browser().await;
        let page = manager.new_page().await.expect("New page failed");
        manager
            .navigate(&page, &url)
            .await
            .expect("Navigation failed");
        // `check_seo` is the JourneyModule's gate.
        let mut config = default_config();
        config.check_seo = true;
        let (report, _snapshot) = audit_page(&page, &url, &config, &manager)
            .await
            .expect("Audit failed");
        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        report
            .journey
            .expect("journey analysis present")
            .friction_points
            .iter()
            .any(|f| matches!(f.kind, FrictionKind::LittleEarlyText))
    }

    // Text pushed below a 3000px hero — the visitor meets nothing.
    assert!(
        flagged("early_text_below_fold.html").await,
        "a page whose text starts below the fold must be flagged"
    );
    // Heading and paragraph at the top.
    assert!(
        !flagged("early_text_above_fold.html").await,
        "a page that opens with a heading and a paragraph must not be flagged"
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_modern_contrast_resolution() {
    let (url, shutdown) = serve_fixture("modern_contrast.html");

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

    // Let's debug print violations to see what we found
    for violation in &report.accessibility.wcag_results.violations {
        println!(
            "Violation: rule={}, selector={:?}, message={}",
            violation.rule, violation.selector, violation.message
        );
    }

    // Filter violations by contrast rule "1.4.3"
    let contrast_violations: Vec<_> = report
        .accessibility
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
#[ignore = "needs Chrome"]
async fn test_image_contrast_pixel_sampling() {
    let (url, shutdown) = serve_fixture("image_contrast.html");

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

    // Filter contrast rule "1.4.3" violations and warnings
    let contrast_violations: Vec<_> = report
        .accessibility
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "1.4.3")
        .collect();

    let contrast_warnings: Vec<_> = report
        .accessibility
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

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_opacity_overlay_contrast_pixel_sampling() {
    // Regression for #527: an uncertain-background element (opacity stack,
    // translucent overlay sibling, or an actual <img> behind text) whose
    // CSS-derived colors look fine in isolation must still be pixel-sampled
    // and surfaced, instead of silently producing zero findings.
    let (url, shutdown) = serve_fixture("opacity_overlay_contrast.html");

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

    let contrast_violations: Vec<_> = report
        .accessibility
        .wcag_results
        .violations
        .iter()
        .filter(|v| v.rule == "1.4.3")
        .collect();
    let contrast_warnings: Vec<_> = report
        .accessibility
        .wcag_results
        .warnings
        .iter()
        .filter(|v| v.rule == "1.4.3")
        .collect();

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

    let is_flagged = |needle: &str| -> bool {
        let in_violations = contrast_violations
            .iter()
            .filter(|v| v.selector.as_deref().is_some_and(|s| s.contains(needle)))
            .count();
        let in_warnings = contrast_warnings
            .iter()
            .filter(|w| w.selector.as_deref().is_some_and(|s| s.contains(needle)))
            .count();
        assert!(
            in_violations + in_warnings <= 1,
            "{needle} must be flagged at most once across violations+warnings, got {} violations and {} warnings",
            in_violations,
            in_warnings
        );
        in_violations + in_warnings == 1
    };

    assert!(
        is_flagged("opacity-text") || is_flagged("opacity-box"),
        "opacity-stacked text must now be flagged (previously missed entirely)"
    );
    assert!(
        is_flagged("overlay-text") || is_flagged("overlay-box"),
        "text under a translucent overlay sibling must now be flagged"
    );
    assert!(
        is_flagged("img-text") || is_flagged("img-box"),
        "text layered over an actual <img> must now be flagged"
    );
}

/// #343 — catalog-driven audit produces a complete report structure.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_catalog_driven_audit_complete_structure() {
    let (url, shutdown) = serve_fixture("perfect.html");

    let config = PipelineConfig {
        check_performance: true,
        check_seo: true,
        check_mobile: true,
        ..default_config()
    };

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let (report, _snapshot) = auditmysite::audit_page(&page, &url, &config, &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(
        report.discoverability.source_quality.is_some(),
        "source_quality must be populated"
    );
    assert!(
        report.discoverability.ai_visibility.is_some(),
        "ai_visibility must be populated"
    );
    assert!(
        report.discoverability.content_visibility.is_some(),
        "content_visibility must be populated"
    );
    assert!(report.accessibility.score >= 0.0 && report.accessibility.score <= 100.0);
}

/// #345 — a page that causes a sub-analysis to fail does not abort the audit.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_partial_module_failure_does_not_abort_audit() {
    let (url, shutdown) = serve_fixture("many_violations.html");

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

    let (report, _snapshot) = auditmysite::audit_page(&page, &url, &config, &manager)
        .await
        .expect("Audit must succeed even when sub-analyses fail");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(
        !report.accessibility.wcag_results.violations.is_empty()
            || report.accessibility.wcag_results.passes > 0,
        "WCAG results must be present"
    );
}

/// #346 — auditing two URLs sequentially with the same BrowserManager succeeds.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_sequential_two_url_audit() {
    let (url1, shutdown1) = serve_fixture("perfect.html");
    let (url2, shutdown2) = serve_fixture("many_violations.html");

    let manager = ci_browser().await;

    let page1 = manager.new_page().await.expect("page 1 failed");
    manager.navigate(&page1, &url1).await.expect("nav 1 failed");
    let (report1, _snapshot) = auditmysite::audit_page(&page1, &url1, &default_config(), &manager)
        .await
        .expect("audit 1 failed");

    let page2 = manager.new_page().await.expect("page 2 failed");
    manager.navigate(&page2, &url2).await.expect("nav 2 failed");
    let (report2, _snapshot) = auditmysite::audit_page(&page2, &url2, &default_config(), &manager)
        .await
        .expect("audit 2 failed");

    shutdown1.store(true, std::sync::atomic::Ordering::Relaxed);
    shutdown2.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(
        report1.accessibility.score > report2.accessibility.score,
        "perfect page ({:.1}) should beat violations page ({:.1})",
        report1.accessibility.score,
        report2.accessibility.score
    );
}

/// #347 — JSON output from a catalog-driven audit validates against the v2.0 envelope.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_catalog_audit_json_envelope_v2() {
    let (url, shutdown) = serve_fixture("perfect.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let (report, _snapshot) = auditmysite::audit_page(&page, &url, &default_config(), &manager)
        .await
        .expect("Audit failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let normalized = auditmysite::audit::normalize(&report);
    let json = auditmysite::format_json_normalized(&normalized, &report, true)
        .expect("JSON formatting failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");

    assert_eq!(
        parsed["schema_version"].as_str(),
        Some("2.0"),
        "schema_version must be 2.0"
    );
    assert_eq!(
        parsed["report_type"].as_str(),
        Some("single"),
        "report_type must be single"
    );
    let pages = parsed["pages"].as_array().expect("pages must be array");
    assert!(!pages.is_empty(), "pages must have at least one entry");
    assert!(pages[0]["url"].is_string(), "page entry must have url");
    assert!(
        pages[0]["accessibility_score"].is_number(),
        "page entry must have accessibility_score"
    );
}

/// Regression test for #513: `check_label_in_name_with_page` must not flag a
/// mismatch caused purely by `textContent`'s lack of whitespace at element
/// boundaries, must downgrade a compound widget's plausible-but-unverifiable
/// mismatch to a warning rather than an auto-confirmed violation, and must
/// still catch a genuine label/name mismatch.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_label_in_name_false_positives() {
    let (url, shutdown) = serve_fixture("label_in_name.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let findings = auditmysite::wcag::rules::check_label_in_name_with_page(&page).await;

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let whitespace_boundary = findings.iter().find(|v| v.message.contains("Loads slowly"));
    assert!(
        whitespace_boundary.is_none(),
        "adjacent-element whitespace gap must not be flagged as a mismatch; got {findings:?}"
    );

    let compound_widget = findings
        .iter()
        .find(|v| v.message.contains("Site is slow"))
        .expect("compound widget with a concise aria-label should still surface a finding");
    assert_eq!(
        compound_widget.kind,
        auditmysite::wcag::types::Outcome::Review,
        "compound widget mismatch should be a warning, not an auto-confirmed violation; got {compound_widget:?}"
    );

    let genuine_mismatch = findings
        .iter()
        .find(|v| v.message.contains("Submit order"))
        .expect("a genuine label/name mismatch must still be caught");
    assert_eq!(
        genuine_mismatch.kind,
        auditmysite::wcag::types::Outcome::Fail,
        "genuine mismatch should stay a confirmed violation; got {genuine_mismatch:?}"
    );
}

/// #652: an undersized link passes 2.5.8 and 2.5.5 when another visible,
/// non-inert, non-aria-hidden link to the same destination meets the size
/// ("Equivalent" exception). A fragment into another document is ignored, one
/// into the current page is not. Hidden, undersized and `href="#"` equivalents
/// do not count.
#[tokio::test]
#[ignore]
async fn test_target_size_equivalent_link_exception() {
    let (url, shutdown) = serve_fixture("detection_corpus/target_size_equivalent.html");

    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let minimum = auditmysite::wcag::rules::check_target_size_minimum_with_page(&page).await;
    let enhanced = auditmysite::wcag::rules::check_target_size_enhanced_with_page(&page).await;

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let selectors = |findings: &[auditmysite::Violation]| -> Vec<String> {
        findings.iter().filter_map(|v| v.selector.clone()).collect()
    };
    assert_eq!(
        selectors(&minimum),
        [
            "a#hidden-only",
            "a#both-small",
            "a#no-eq",
            "a#page-fragment",
            "a#hash"
        ],
        "2.5.8 findings: {minimum:?}"
    );
    let enhanced_selectors = selectors(&enhanced);
    assert!(
        !enhanced_selectors.contains(&"a#eq-small".to_string())
            && enhanced_selectors.contains(&"a#hidden-only".to_string())
            && enhanced_selectors.contains(&"a#no-eq".to_string()),
        "2.5.5 findings: {enhanced:?}"
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_design_quality_module_findings_and_score_isolation() {
    // #528: the opt-in design_quality module must (a) actually detect each of
    // its rules against a real rendered page and (b) never change the
    // accessibility score/grade/certificate, whether enabled or not.
    let (url, shutdown) = serve_fixture("design_quality.html");

    let manager = ci_browser().await;

    let page_off = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page_off, &url)
        .await
        .expect("Navigation failed");
    let config_off = PipelineConfig {
        check_performance: true,
        ..default_config()
    };
    let (report_off, _snapshot_off) = audit_page(&page_off, &url, &config_off, &manager)
        .await
        .expect("Audit failed (design_quality off)");

    let page_on = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page_on, &url)
        .await
        .expect("Navigation failed");
    let config_on = PipelineConfig {
        check_performance: true,
        check_design_quality: true,
        ..default_config()
    };
    let (report_on, _snapshot_on) = audit_page(&page_on, &url, &config_on, &manager)
        .await
        .expect("Audit failed (design_quality on)");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(
        report_off.experience.design_quality.is_none(),
        "design_quality must stay unpopulated when the module is off"
    );
    let dq = report_on
        .experience
        .design_quality
        .as_ref()
        .expect("design_quality must be populated when the module is on");

    for rule_id in [
        "design.overflow_clip",
        "design.line_length",
        "design.line_height",
        "design.all_caps",
        "design.layout_transition",
    ] {
        assert!(
            dq.findings.iter().any(|f| f.rule_id == rule_id),
            "expected a {rule_id} finding; got {:?}",
            dq.findings.iter().map(|f| &f.rule_id).collect::<Vec<_>>()
        );
    }

    for safe_selector in [
        "clip-button-safe",
        "short-line-text",
        "normal-line-height-text",
        "mixed-case-text",
        "transition-safe",
    ] {
        assert!(
            !dq.findings
                .iter()
                .any(|f| f.selector.contains(safe_selector)),
            "negative control '{safe_selector}' must not produce a design_quality finding"
        );
    }

    assert_eq!(
        report_off.accessibility.score, report_on.accessibility.score,
        "accessibility score must be byte-identical whether design_quality is on or off"
    );
    assert_eq!(
        report_off.accessibility.grade, report_on.accessibility.grade,
        "accessibility grade must be identical whether design_quality is on or off"
    );
    assert_eq!(
        report_off.accessibility.certificate, report_on.accessibility.certificate,
        "certificate must be identical whether design_quality is on or off"
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
#[cfg(feature = "ai-transparency")]
async fn test_ai_transparency_module_ssrf_guard_and_score_isolation() {
    // The opt-in ai_transparency module must (a) never change the
    // accessibility score/grade/certificate, whether enabled or not, and (b)
    // never fetch the loopback fixture server the test harness itself runs
    // on — `serve_fixture` binds 127.0.0.1, which the module's SSRF guard
    // must reject just like it would any other private/loopback address.
    // (An end-to-end "real AI-generated image" path is covered by the
    // synthetic-fixture unit tests in `src/ai_transparency/image_provenance.rs`
    // — this harness has no way to serve real image bytes at a public IP.)
    let (url, shutdown) = serve_fixture("ai_transparency.html");

    let manager = ci_browser().await;

    let page_off = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page_off, &url)
        .await
        .expect("Navigation failed");
    let config_off = PipelineConfig {
        check_performance: true,
        ..default_config()
    };
    let (report_off, _snapshot_off) = audit_page(&page_off, &url, &config_off, &manager)
        .await
        .expect("Audit failed (ai_transparency off)");

    let page_on = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page_on, &url)
        .await
        .expect("Navigation failed");
    let config_on = PipelineConfig {
        check_performance: true,
        check_ai_transparency: true,
        ..default_config()
    };
    let (report_on, _snapshot_on) = audit_page(&page_on, &url, &config_on, &manager)
        .await
        .expect("Audit failed (ai_transparency on)");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(
        report_off.experience.ai_transparency.is_none(),
        "ai_transparency must stay unpopulated when the module is off"
    );
    let at = report_on
        .experience
        .ai_transparency
        .as_ref()
        .expect("ai_transparency must be populated when the module is on");
    assert_eq!(
        at.images_checked, 0,
        "the loopback fixture server must be rejected by the SSRF guard, not fetched"
    );
    assert!(at.findings.is_empty());

    assert_eq!(
        report_off.accessibility.score, report_on.accessibility.score,
        "accessibility score must be byte-identical whether ai_transparency is on or off"
    );
    assert_eq!(
        report_off.accessibility.grade, report_on.accessibility.grade,
        "accessibility grade must be identical whether ai_transparency is on or off"
    );
    assert_eq!(
        report_off.accessibility.certificate, report_on.accessibility.certificate,
        "certificate must be identical whether ai_transparency is on or off"
    );
}

/// Regression test for a batch-mode hang investigation: `wait_for_stable`
/// (called from `set_viewport` before every navigation, twice per audited
/// page) issued an `awaitPromise: true` `Runtime.evaluate` CDP command with
/// no timeout wrapper of its own — unlike its sibling `wait_for_page_stability`
/// in the same file, which already bounds the equivalent call. A live batch
/// run against real concurrent pages sharing one `BrowserManager` showed one
/// page's `wait_for_stable` call staying pending for ~30s (matching
/// chromiumoxide's internal command timeout) while sibling pages progressed
/// normally — silently eating into that page's overall per-audit timeout
/// budget under concurrency (`--url-file`/`--sitemap` batch mode with
/// concurrency >= 2). `wait_for_stable` now wraps its CDP call in
/// `tokio::time::timeout(duration_ms + 500ms, ..)`, matching the existing
/// pattern. This test asserts N concurrent `wait_for_stable` calls across
/// separate pages of the same browser complete well within that bound
/// instead of being allowed to silently run unbounded.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_concurrent_wait_for_stable_stays_within_its_timeout_budget() {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let manager = Arc::new(ci_browser().await);
    let duration_ms: u64 = 150;

    let mut tasks = Vec::new();
    for _ in 0..3 {
        let manager = Arc::clone(&manager);
        tasks.push(tokio::spawn(async move {
            let page = manager.new_page().await.expect("New page failed");
            let start = Instant::now();
            auditmysite::interaction::stability::wait_for_stable(&page, duration_ms)
                .await
                .expect("wait_for_stable failed");
            start.elapsed()
        }));
    }

    // Bounded generously above wait_for_stable's own `duration_ms + 500ms`
    // ceiling (650ms here), well under the ~30s worst case this regression
    // test guards against.
    let budget = Duration::from_millis(duration_ms + 500 + 4_000);
    // The outer bound is a hang guard for the whole task, and the task also
    // creates a page — which is not what this test measures. It used to carry
    // `budget`, so on a loaded machine the suite failed here with
    // "exceeded its bounded timeout" when page creation, not
    // `wait_for_stable`, had used up the time. Reproduced on the unchanged
    // code: 2 of 2 full `--ignored` runs. `elapsed` below is measured around
    // `wait_for_stable` alone and stays the real assertion.
    let hang_guard = Duration::from_secs(60);
    for task in tasks {
        let elapsed = tokio::time::timeout(hang_guard, task)
            .await
            .expect("wait_for_stable hung under concurrency")
            .expect("task panicked");
        assert!(
            elapsed < budget,
            "wait_for_stable took {elapsed:?}, expected well under {budget:?}"
        );
    }
}

// ── Video caption / transcript / keyboard-operability deepening ───────────
// (video-caption-checks plan): 1.2.1/1.2.2 track resolution, 1.2.8
// transcript-link enrichment, and native <video controls> keyboard
// operability via the tab-walk journey.

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_video_caption_track_resolving_is_a_confirmed_pass() {
    let (url, shutdown) = serve_fixture("video_captions_resolving.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let findings = auditmysite::wcag::rules::check_video_caption_tracks_with_page(&page).await;

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding: {findings:?}"
    );
    assert_eq!(
        findings[0].kind,
        auditmysite::wcag::types::Outcome::Pass,
        "a resolving <track kind=\"captions\"> file should confirm a pass: {:?}",
        findings[0]
    );
    assert!(findings[0].message.contains("resolving"));
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_video_without_track_stays_manual_review() {
    let (url, shutdown) = serve_fixture("video_no_track.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let findings = auditmysite::wcag::rules::check_video_caption_tracks_with_page(&page).await;

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding: {findings:?}"
    );
    assert_eq!(
        findings[0].kind,
        auditmysite::wcag::types::Outcome::Untested,
        "a video with no track element must stay a manual-review notice, not a violation \
         or a false pass: {:?}",
        findings[0]
    );
    assert!(findings[0].message.contains("without a resolving"));
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_video_embed_iframe_gets_platform_specific_manual_review_message() {
    let (url, shutdown) = serve_fixture("video_embed_iframe.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let findings = auditmysite::wcag::rules::check_video_caption_tracks_with_page(&page).await;

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding: {findings:?}"
    );
    assert_eq!(
        findings[0].kind,
        auditmysite::wcag::types::Outcome::Untested
    );
    assert!(
        findings[0].message.contains("embedded video player"),
        "a YouTube-style embed with no native <video> should get its own message: {:?}",
        findings[0]
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_media_alternative_enriches_message_with_nearby_transcript_link() {
    let (url, shutdown) = serve_fixture("video_transcript_link.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let findings = auditmysite::wcag::rules::check_media_alternative_with_page(&page).await;

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding: {findings:?}"
    );
    assert_eq!(
        findings[0].kind,
        auditmysite::wcag::types::Outcome::Untested,
        "presence of a transcript link is evidence, not a confirmed pass: {:?}",
        findings[0]
    );
    assert!(
        findings[0].message.contains("Read the full transcript"),
        "expected the found transcript link text in the message: {:?}",
        findings[0]
    );
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_media_alternative_default_message_without_transcript_link() {
    let (url, shutdown) = serve_fixture("video_no_track.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let findings = auditmysite::wcag::rules::check_media_alternative_with_page(&page).await;

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding: {findings:?}"
    );
    assert!(findings[0]
        .message
        .contains("Verify that a full text alternative"));
}

#[tokio::test]
#[ignore = "needs Chrome"]
async fn test_video_controls_missing_name_flagged_during_tab_walk() {
    let (url, shutdown) = serve_fixture("video_controls_keyboard.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let record = auditmysite::a11y_journey::tab_walk::record(&page, 5)
        .await
        .expect("tab walk failed");
    let findings = auditmysite::a11y_journey::evaluate::tab_walk(&record.trace, &record.snapshots);

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let media_finding = findings
        .iter()
        .find(|f| f.category == "MediaControls")
        .unwrap_or_else(|| {
            panic!(
                "expected a MediaControls finding for the nameless <video controls>: {findings:?}"
            )
        });
    assert_eq!(
        media_finding.kind,
        auditmysite::audit::normalized::InteractiveFindingKind::MediaControlsMissingName
    );
}

/// Eine Live-Region, die von Anfang an leer im Markup steht, wird gefüllt und
/// kurz darauf wieder geleert — der Normalfall bei Formularfehlern, und der
/// Fall, an dem die vorherige Prüfung scheiterte.
///
/// Sie fragte `live_after && !live_before`: *existiert* jetzt eine Region, die
/// vorher nicht existierte. Bei der empfohlenen Umsetzung — Container leer im
/// Markup, damit Screenreader ihn beim Aufbau des Baums registrieren — ist die
/// Antwort immer „nein". Ergebnis war ein `FormErrorInvalidWithoutLiveRegion`
/// mit Severity High auf einer Seite, die alles richtig macht.
///
/// Gemessen wird jetzt über `interaction::live_regions`: beobachtet, **dass**
/// die Region Inhalt bekommen hat. Der Test pinnt beides — die Beobachtung
/// greift, und der Fehlalarm bleibt aus.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn transiente_live_region_wird_beobachtet_und_erzeugt_keinen_fehlalarm() {
    use auditmysite::audit::normalized::InteractiveFindingKind;
    use auditmysite::patterns::{JourneyCandidate, JourneyKind, PatternKind};

    let (url, shutdown) = serve_fixture("live_region_transient_error.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let tree = auditmysite::accessibility::extract_ax_tree(&page)
        .await
        .expect("AXTree extraction failed");
    let submit = tree
        .iter()
        .find(|n| {
            n.role.as_deref() == Some("button")
                && n.name
                    .as_deref()
                    .is_some_and(|name| name.contains("Absenden"))
        })
        .and_then(|n| n.backend_dom_node_id)
        .expect("Absenden-Button im AXTree");

    let candidate = JourneyCandidate {
        pattern_kind: PatternKind::Form,
        trigger_backend_id: Some(submit),
        controlled_backend_id: None,
        confidence: 0.9,
        required_journey: JourneyKind::FormErrorSubmit,
    };

    let (trace, findings) = auditmysite::a11y_journey::form_error::test(&page, &candidate, 0)
        .await
        .expect("form_error journey failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let state = trace
        .steps
        .iter()
        .find(|s| s.action == "check_error_state")
        .and_then(|s| s.result.clone())
        .unwrap_or_default();
    assert!(
        state.contains("announced:true"),
        "die gefüllte Live-Region muss beobachtet werden, Trace: {state:?}"
    );
    assert!(
        state.contains("assertive:true"),
        "role=\"alert\" ist assertive, Trace: {state:?}"
    );
    assert!(
        !findings
            .iter()
            .any(|f| f.kind == InteractiveFindingKind::FormErrorInvalidWithoutLiveRegion),
        "eine korrekt angekündigte Meldung darf keinen Befund erzeugen: {findings:?}"
    );
}

/// Ein korrekt umgesetztes Menü darf keinen Befund erzeugen.
///
/// Das Muster kam im Seitenkorpus **nicht vor** — und das hatte einen Grund:
/// `patterns::disclosure_menu` fragte die CDP-Eigenschaft als `"haspopup"` ab,
/// sie heißt `hasPopup`. Über 171 gelaufene Seiten lief deshalb keine einzige
/// Menü-Journey. Dieser Test hält beides fest: dass der Kandidat entsteht und
/// dass ein korrektes Menü still bleibt.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn korrektes_menue_erzeugt_keine_befunde() {
    use auditmysite::patterns::{JourneyCandidate, JourneyKind, PatternKind};

    let (url, shutdown) = serve_fixture("dialog_and_menu_journeys.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let tree = auditmysite::accessibility::extract_ax_tree(&page)
        .await
        .expect("AXTree extraction failed");
    let trigger = tree
        .iter()
        .find(|n| {
            n.role.as_deref() == Some("button")
                && n.name.as_deref().is_some_and(|name| name.contains("Menue"))
        })
        .expect("Menue-Button im AXTree");
    assert_eq!(
        trigger.haspopup(),
        Some("menu"),
        "die CDP-Eigenschaft heißt hasPopup, nicht haspopup"
    );

    let candidate = JourneyCandidate {
        pattern_kind: PatternKind::Menu,
        trigger_backend_id: trigger.backend_dom_node_id,
        controlled_backend_id: None,
        confidence: 0.8,
        required_journey: JourneyKind::MenuOpen,
    };
    let (trace, findings) = auditmysite::a11y_journey::menu_journey::test(&page, &candidate, 0)
        .await
        .expect("menu journey failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let step = |action: &str| {
        trace
            .steps
            .iter()
            .find(|s| s.action == action)
            .and_then(|s| s.result.clone())
            .unwrap_or_default()
    };
    assert_eq!(
        step("check_menu_open"),
        "menu_open",
        "Zustandswechsel am Auslöser plus sichtbar gewordenes Menü: {trace:?}"
    );
    assert_eq!(
        step("check_focus_in_menu"),
        "focus_in_menu",
        "der Fokus wandert auf den ersten Eintrag: {trace:?}"
    );
    assert_eq!(step("check_menu_closed"), "menu_closed", "{trace:?}");
    assert!(
        findings.is_empty(),
        "ein korrektes Menü darf nichts melden: {findings:?}"
    );
}

/// Ein korrekt umgesetzter modaler Dialog darf keinen Befund erzeugen.
///
/// Auch dieses Muster kam im Korpus nicht vor, aus zwei Gründen: derselbe
/// `hasPopup`-Fehler, und `patterns::modal_dialog` stieg aus, wenn beim Laden
/// kein Dialog im Baum stand — bei einem geschlossenen `<dialog>` also immer.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn korrekter_modaler_dialog_erzeugt_keine_befunde() {
    use auditmysite::patterns::{JourneyCandidate, JourneyKind, PatternKind};

    let (url, shutdown) = serve_fixture("dialog_and_menu_journeys.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let tree = auditmysite::accessibility::extract_ax_tree(&page)
        .await
        .expect("AXTree extraction failed");

    // Der geschlossene <dialog> steht nicht im Baum — der Auslöser muss
    // trotzdem zum Kandidaten werden.
    assert!(
        !tree
            .iter()
            .any(|n| matches!(n.role.as_deref(), Some("dialog") | Some("alertdialog"))),
        "ein geschlossener <dialog> ist nicht gerendert"
    );
    let analysis = auditmysite::patterns::analyze(&tree);
    assert!(
        analysis
            .journey_candidates
            .iter()
            .any(|c| c.required_journey == JourneyKind::ModalOpen),
        "der Auslöser mit aria-haspopup=\"dialog\" muss angeboten werden: {:?}",
        analysis.journey_candidates
    );

    let trigger = tree
        .iter()
        .find(|n| {
            n.role.as_deref() == Some("button")
                && n.name
                    .as_deref()
                    .is_some_and(|name| name.contains("Hinweis"))
        })
        .and_then(|n| n.backend_dom_node_id)
        .expect("Dialog-Button im AXTree");

    let candidate = JourneyCandidate {
        pattern_kind: PatternKind::Modal,
        trigger_backend_id: Some(trigger),
        controlled_backend_id: None,
        confidence: 0.85,
        required_journey: JourneyKind::ModalOpen,
    };
    let (trace, findings) = auditmysite::a11y_journey::modal_journey::test(&page, &candidate, 0)
        .await
        .expect("modal journey failed");

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let step = |action: &str| {
        trace
            .steps
            .iter()
            .find(|s| s.action == action)
            .and_then(|s| s.result.clone())
            .unwrap_or_default()
    };
    assert_eq!(step("check_dialog_opened"), "dialog_opened", "{trace:?}");
    assert_eq!(
        step("check_dialog_modal"),
        "modal",
        "showModal() setzt `modal` im Accessibility-Tree: {trace:?}"
    );
    assert_eq!(
        step("check_focus_in_dialog"),
        "focus_inside_dialog",
        "der Browser verlegt den Fokus in den modalen Dialog: {trace:?}"
    );
    assert!(
        findings.is_empty(),
        "ein korrekter modaler Dialog darf nichts melden: {findings:?}"
    );
}

/// Die Skip-Link-Journey prüft, ob der Fokus am Sprungziel ankommt — nicht nur,
/// ob er `body` verlassen hat (Plan 53).
///
/// Die vorherige Prüfung meldete ein nicht fokussierbares `<main>` als Fehler,
/// obwohl der Browser den Startpunkt der Tab-Navigation versetzt und der
/// nächste Tab im Inhalt landet; und sie ließ einen Fokus durchgehen, der
/// irgendwohin sprang, nur nicht zum Ziel.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn skip_link_journey_prueft_das_sprungziel() {
    use auditmysite::audit::normalized::InteractiveFindingKind;
    use auditmysite::patterns::{JourneyCandidate, JourneyKind, PatternKind};

    let cases = [
        ("skip_link_focusable_target.html", "focus_on_target", false),
        (
            "skip_link_non_focusable_target.html",
            "tab_reaches_target",
            false,
        ),
        ("skip_link_missing_target.html", "target_missing", true),
        (
            "skip_link_focus_before_target.html",
            "focus_before_target",
            true,
        ),
        ("skip_link_no_fragment_inert.html", "focus_not_moved", true),
    ];

    let manager = ci_browser().await;
    for (fixture, expected_result, expect_finding) in cases {
        let (url, shutdown) = serve_fixture(fixture);
        let page = manager.new_page().await.expect("New page failed");
        manager
            .navigate(&page, &url)
            .await
            .expect("Navigation failed");

        let tree = auditmysite::accessibility::extract_ax_tree(&page)
            .await
            .expect("AXTree extraction failed");
        let link = tree
            .iter()
            .find(|n| {
                n.role.as_deref() == Some("link")
                    && n.name.as_deref() == Some("Zum Inhalt springen")
            })
            .and_then(|n| n.backend_dom_node_id)
            .expect("Skip-Link im AXTree");

        let candidate = JourneyCandidate {
            pattern_kind: PatternKind::SkipLink,
            trigger_backend_id: Some(link),
            controlled_backend_id: None,
            confidence: 0.9,
            required_journey: JourneyKind::SkipLinkActivate,
        };
        let (trace, findings) = auditmysite::a11y_journey::skip_link::test(&page, &candidate, 0)
            .await
            .expect("skip_link journey failed");
        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

        let result = trace
            .steps
            .iter()
            .rev()
            .find(|s| s.action.starts_with("check_focus"))
            .and_then(|s| s.result.clone());
        assert_eq!(
            result.as_deref(),
            Some(expected_result),
            "{fixture}: {trace:?}"
        );
        let has_finding = findings
            .iter()
            .any(|f| f.kind == InteractiveFindingKind::SkipLinkFocusNotMoved);
        assert_eq!(has_finding, expect_finding, "{fixture}: {findings:?}");
    }
}

/// Natives `<details>`/`<summary>` erreicht die Disclosure-Journey — genau
/// einmal je Auslöser, mit dem umgebenden `<details>` als gesteuertem Bereich.
///
/// Vor Plan 53 bot `patterns::disclosure_menu` nur Elemente mit
/// `aria-expanded` an. Danach bekam ein `<summary>` zwei Kandidaten — einen aus
/// `disclosure_menu`, einen aus `accordion` —, und die Journey lief zweimal.
/// `<details name>` trägt die Rolle `DisclosureTriangleGrouped`, die weder der
/// Bereichsbestimmung noch dem Akkordeon bekannt war.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn natives_details_erreicht_die_disclosure_journey() {
    use auditmysite::patterns::JourneyKind;

    let (url, shutdown) = serve_fixture("details_disclosure.html");
    let manager = ci_browser().await;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, &url)
        .await
        .expect("Navigation failed");

    let tree = auditmysite::accessibility::extract_ax_tree(&page)
        .await
        .expect("AXTree extraction failed");
    let summaries: Vec<(String, i64)> = tree
        .iter()
        .filter(|n| {
            matches!(
                n.role.as_deref(),
                Some("DisclosureTriangle") | Some("DisclosureTriangleGrouped")
            )
        })
        .map(|n| {
            (
                n.role.clone().unwrap_or_default(),
                n.backend_dom_node_id.expect("summary mit Backend-ID"),
            )
        })
        .collect();
    assert_eq!(summaries.len(), 3, "drei <summary> im Baum: {summaries:?}");
    assert!(
        summaries
            .iter()
            .any(|(role, _)| role == "DisclosureTriangleGrouped"),
        "<details name> trägt die Gruppenrolle: {summaries:?}"
    );

    let analysis = auditmysite::patterns::analyze(&tree);
    assert!(
        !analysis
            .violations
            .iter()
            .any(|v| v.rule_id.as_deref() == Some("accordion-trigger-not-button")),
        "ein natives <summary> ist kein falscher Auslöser: {:?}",
        analysis.violations
    );

    for (role, summary) in summaries {
        let candidates: Vec<_> = analysis
            .journey_candidates
            .iter()
            .filter(|c| c.trigger_backend_id == Some(summary))
            .collect();
        assert_eq!(
            candidates.len(),
            1,
            "{role}: genau ein Kandidat je Auslöser: {candidates:?}"
        );
        let candidate = candidates[0];
        assert_eq!(candidate.required_journey, JourneyKind::DisclosureToggle);
        assert!((candidate.confidence - 0.9).abs() < f32::EPSILON);

        let (trace, findings) =
            auditmysite::a11y_journey::disclosure_journey::test(&page, candidate, 0)
                .await
                .expect("disclosure journey failed");

        let step = |action: &str| {
            trace
                .steps
                .iter()
                .find(|s| s.action == action)
                .and_then(|s| s.result.clone())
                .unwrap_or_default()
        };
        assert_eq!(step("initial_state"), "collapsed", "{role}: {trace:?}");
        assert_eq!(step("controlled_region"), "scoped", "{role}: {trace:?}");
        assert_eq!(
            step("check_expanded"),
            "diff:state+content",
            "{role}: {trace:?}"
        );
        assert_eq!(
            step("check_collapsed"),
            "diff:state+content",
            "{role}: {trace:?}"
        );
        assert!(
            findings.is_empty(),
            "{role}: ein natives <details> darf nichts melden: {findings:?}"
        );
    }

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Batch pages run side by side in one browser. Each must be visible and
/// focused like a single-URL run, or requestAnimationFrame never fires and
/// focus-based checks measure something else (plan 62).
#[tokio::test]
#[ignore = "needs Chrome"]
async fn parallel_pages_are_visible_and_focused() {
    use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;

    // Parallel batch pages get focus emulation (see `BatchConfig`).
    let manager = BrowserManager::with_options(BrowserOptions {
        no_sandbox: std::env::var("CI").is_ok(),
        focus_emulation: true,
        ..Default::default()
    })
    .await
    .expect("Browser launch failed");
    let mut pages = Vec::new();
    for _ in 0..3 {
        pages.push(manager.new_page().await.expect("New page failed"));
    }
    for (i, page) in pages.iter().enumerate() {
        let visible: String = page
            .evaluate("document.visibilityState")
            .await
            .expect("evaluate")
            .into_value()
            .expect("string");
        let focused: bool = page
            .evaluate("document.hasFocus()")
            .await
            .expect("evaluate")
            .into_value()
            .expect("bool");
        let raf = EvaluateParams::builder()
            .expression("new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))")
            .await_promise(true)
            .build()
            .expect("params");
        let frames =
            tokio::time::timeout(std::time::Duration::from_secs(2), page.execute(raf)).await;
        assert_eq!(visible, "visible", "page {i}");
        assert!(focused, "page {i} has no focus");
        assert!(matches!(frames, Ok(Ok(_))), "page {i}: no animation frame");
    }
}

/// #651: a page that gets no browser page in time is retried one at a time
/// after the parallel phase instead of vanishing from the batch. One pool
/// slot, two parallel workers and a 1 s pool wait make the second page time
/// out on the pool for certain — the capacity shortfall heavy WebGL pages
/// caused in production.
///
/// Runs on an 8 MiB thread like the CLI's main thread: in a debug build the
/// batch future overflows the test harness's 2 MiB thread stack.
#[test]
#[ignore]
fn batch_retries_pool_timeouts_serially() {
    std::thread::Builder::new()
        .stack_size(8 << 20)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime")
                .block_on(batch_retries_pool_timeouts_serially_body())
        })
        .expect("test thread")
        .join()
        .expect("test body panicked");
}

async fn batch_retries_pool_timeouts_serially_body() {
    use auditmysite::{run_concurrent_batch, Args, BatchConfig};
    use clap::Parser;

    let (url, shutdown) = serve_fixture("perfect.html");
    let urls: Vec<String> = ["a", "b", "c"]
        .iter()
        .map(|path| format!("{url}/{path}"))
        .collect();

    let args = Args::parse_from(["auditmysite", "--url-file", "unused.txt", "-c", "2"]);
    let mut config = BatchConfig::from(&args);
    config.pool_config.max_pages = 1;
    config.pool_config.acquire_timeout_secs = 1;
    config.pool_config.browser_options.no_sandbox = std::env::var("CI").is_ok();
    config.pipeline.persist_artifacts = false;

    let progress_calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let progress: auditmysite::audit::ProgressCallback = {
        let calls = Arc::clone(&progress_calls);
        Arc::new(move |current, total, url: &str, error: Option<&str>| {
            calls.lock().unwrap().push((
                current,
                total,
                url.to_string(),
                error.map(str::to_string),
            ));
        })
    };

    let batch = run_concurrent_batch(urls.clone(), &config, Some(progress))
        .await
        .expect("batch runs");
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(batch.errors.is_empty(), "errors: {:?}", batch.errors);
    let audited: Vec<&str> = batch.reports.iter().map(|r| r.url.as_str()).collect();
    assert_eq!(audited, urls.iter().map(String::as_str).collect::<Vec<_>>());

    // Every page is reported exactly once and as a success: a deferred page
    // is not announced as failed before its serial retry.
    let calls = progress_calls.lock().unwrap();
    assert_eq!(calls.len(), 3, "progress: {calls:?}");
    assert!(calls
        .iter()
        .all(|(_, total, _, error)| *total == 3 && error.is_none()));
    assert_eq!(
        calls
            .iter()
            .map(|(current, ..)| *current)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

async fn audit_with_cli_args(
    manager: &BrowserManager,
    url: &str,
    extra: &[&str],
) -> auditmysite::AuditReport {
    use clap::Parser;
    let mut argv = vec!["auditmysite", url, "--interactive", "off"];
    argv.extend_from_slice(extra);
    let args = auditmysite::cli::Args::parse_from(argv);
    let mut config = PipelineConfig::from_args_and_config(&args, None);
    config.persist_artifacts = false;
    let page = manager.new_page().await.expect("New page failed");
    manager
        .navigate(&page, url)
        .await
        .expect("Navigation failed");
    let (report, _snapshot) = audit_page(&page, url, &config, manager)
        .await
        .expect("Audit failed");
    report
}

/// #645 — `--exclude-selector` drops the findings inside the matched subtree,
/// keeps the ones outside, and reports every selector with its match count
/// (also zero, also invalid) and what was dropped.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn exclude_selector_drops_specimen_findings_and_reports_them() {
    let (url, shutdown) = serve_fixture("exclude_selector_specimen.html");
    let manager = ci_browser().await;

    let baseline = audit_with_cli_args(&manager, &url, &[]).await;
    let excluded = audit_with_cli_args(
        &manager,
        &url,
        &[
            "--exclude-selector",
            ".specimen",
            "--exclude-selector",
            ".nothing-here",
            "--exclude-selector",
            "[[bad",
        ],
    )
    .await;
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let has = |report: &auditmysite::AuditReport, rule: &str, prefix: &str| {
        report
            .accessibility
            .wcag_results
            .violations
            .iter()
            .any(|v| {
                v.rule_id.as_deref() == Some(rule)
                    && v.selector.as_deref().is_some_and(|s| s.starts_with(prefix))
            })
    };

    // Without the flag the specimen's defects are reported ...
    assert!(has(&baseline, "image-alt", "img#specimen-img"));
    assert!(has(
        &baseline,
        "click-events-have-key-events",
        "div#specimen-click"
    ));
    let base_ex = baseline
        .accessibility
        .execution
        .exclusions
        .as_ref()
        .expect("exclusions block is always recorded");
    assert_eq!(base_ex.selectors.len(), 1, "only the built-in attribute");
    assert_eq!(base_ex.selectors[0].matched_elements, 0);
    assert_eq!(base_ex.excluded_occurrences, 0);

    // ... with it they are gone, while the real defect outside stays.
    assert!(!has(&excluded, "image-alt", "img#specimen-img"));
    assert!(!has(
        &excluded,
        "click-events-have-key-events",
        "div#specimen-click"
    ));
    assert!(has(&excluded, "image-alt", "img#real-img"));

    let ex = excluded
        .accessibility
        .execution
        .exclusions
        .as_ref()
        .expect("exclusions block is always recorded");
    let sel = |s: &str| {
        ex.selectors
            .iter()
            .find(|r| r.selector == s)
            .unwrap_or_else(|| panic!("selector {s} missing from {:?}", ex.selectors))
    };
    assert_eq!(sel(".specimen").matched_elements, 1);
    assert_eq!(sel(".nothing-here").matched_elements, 0);
    assert!(!sel(".nothing-here").invalid);
    assert!(sel("[[bad").invalid);
    assert!(sel("[data-audit-exclude]").builtin);
    assert!(ex.excluded_occurrences >= 2, "{ex:?}");
    assert!(ex.rules.iter().any(|r| r.rule_id == "image-alt"));
    assert!(ex
        .rules
        .iter()
        .any(|r| r.rule_id == "click-events-have-key-events"));
    assert!(
        excluded.accessibility.score >= baseline.accessibility.score,
        "excluding defects must not lower the score"
    );
}

/// #645 — excluded elements must not spend a capped JavaScript rule's cap:
/// with more excluded hits than the cap ahead of it in DOM order, the one
/// real hit is still reported, the excluded hits are dropped and counted
/// (bounded by the same cap), and `rule_outcomes[].findings` counts what the
/// report shows.
#[tokio::test]
#[ignore = "needs Chrome"]
async fn excluded_hits_do_not_spend_a_capped_rules_budget() {
    let (url, shutdown) = serve_fixture("detection_corpus/audit_exclude_cap.html");
    let manager = ci_browser().await;
    let report = audit_with_cli_args(&manager, &url, &["--level", "aaa"]).await;
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let rule_selectors = |rule: &str| -> Vec<String> {
        report
            .accessibility
            .wcag_results
            .violations
            .iter()
            .filter(|v| v.rule_id.as_deref() == Some(rule))
            .filter_map(|v| v.selector.clone())
            .collect()
    };
    for (rule, real) in [
        ("click-events-have-key-events", "div#real-click"),
        ("link-as-button", "a#real-link"),
        ("identify-purpose", "real-mail"),
    ] {
        let found = rule_selectors(rule);
        assert!(
            found.iter().any(|s| s == real),
            "{rule}: real hit {real} missing, got {found:?}"
        );
        assert!(
            found.iter().all(|s| !s.contains("specimen")),
            "{rule}: specimen reported: {found:?}"
        );
    }

    let ex = report
        .accessibility
        .execution
        .exclusions
        .as_ref()
        .expect("exclusions block is always recorded");
    let excluded = |rule: &str| {
        ex.rules
            .iter()
            .find(|r| r.rule_id == rule)
            .map(|r| r.occurrences)
            .unwrap_or(0)
    };
    assert_eq!(excluded("click-events-have-key-events"), 10, "{ex:?}");
    assert_eq!(excluded("link-as-button"), 20, "{ex:?}");
    assert_eq!(excluded("identify-purpose"), 5, "{ex:?}");

    for viewport in ["desktop", "mobile"] {
        let outcome = report
            .accessibility
            .wcag_results
            .rule_outcomes
            .iter()
            .find(|o| o.rule_id == "2.1.1/click-handler" && o.viewport.as_deref() == Some(viewport))
            .unwrap_or_else(|| panic!("no click-handler outcome for {viewport}"));
        assert_eq!(outcome.findings, 1, "{viewport}: {outcome:?}");
    }
}
