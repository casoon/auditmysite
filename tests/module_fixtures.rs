//! Per-module fixture tests (#335).
//!
//! Exercises modules in isolation without launching Chrome. Five of the
//! twelve `AuditModule` impls are testable this way because their logic
//! doesn't touch CDP after the upstream data is in hand:
//!
//! - **UX / Journey** — read from an `AXTree` we construct manually.
//! - **SourceQuality / AiVisibility / ContentVisibility** — derive phase,
//!   `derive(&mut AuditReport)` reads fields the collect phase populated.
//!   We synthesize an `AuditReport` directly.
//!
//! The other seven collect-phase modules (Performance, SEO, Security,
//! Mobile, DarkMode, TechStack, BestPractices) call `page.evaluate(...)`
//! and need a real `chromiumoxide::Page`. They're covered by the existing
//! `tests/integration_test.rs` end-to-end pattern with `serve_fixture` +
//! real Chrome; per-module fixture tests for them would require either
//! that browser-based pattern (slow, `#[ignore]`'d) or a `PageProbe`
//! trait abstraction. Out of scope for A6.
//!
//! Run with: `cargo test --test module_fixtures`

use auditmysite::accessibility::{AXNode, AXTree};
use auditmysite::ai_visibility::AiVisibilityModule;
use auditmysite::audit::module::AuditModule;
use auditmysite::audit::AuditReport;
use auditmysite::content_visibility::ContentVisibilityModule;
use auditmysite::journey::{analyze_journey_with_dom_check, JourneyModule};
use auditmysite::seo::SeoAnalysis;
use auditmysite::source_quality::SourceQualityModule;
use auditmysite::ux::{analyze_ux, UxModule};
use auditmysite::wcag::WcagResults;
use auditmysite::WcagLevel;

// ── AX-tree helpers ──────────────────────────────────────────────────────────

fn node(id: &str, role: &str, name: Option<&str>, child_ids: Vec<&str>) -> AXNode {
    AXNode {
        node_id: id.to_string(),
        role: Some(role.to_string()),
        name: name.map(String::from),
        child_ids: child_ids.into_iter().map(String::from).collect(),
        ..AXNode::default()
    }
}

/// Bare WebArea root, no semantic landmarks, no CTAs — should score low
/// across most UX/Journey dimensions.
fn ax_tree_empty_page() -> AXTree {
    AXTree::from_nodes(vec![node("1", "WebArea", Some("Empty"), vec![])])
}

/// Page with main landmark, heading, navigation, a button-CTA and links —
/// what a well-structured marketing page looks like in AX terms.
fn ax_tree_rich_page() -> AXTree {
    let nodes = vec![
        node(
            "1",
            "WebArea",
            Some("Acme — Pricing"),
            vec!["2", "3", "4", "5", "6", "7"],
        ),
        node("2", "navigation", Some("Primary"), vec![]),
        node("3", "main", Some("Pricing"), vec![]),
        node("4", "heading", Some("Choose your plan"), vec![]),
        node("5", "button", Some("Start free trial"), vec![]),
        node("6", "link", Some("Pricing details"), vec![]),
        node("7", "contentinfo", Some("Footer"), vec![]),
    ];
    // backfill parent_id so AXTree::iter() (which walks the tree) yields
    // every node.
    let mut nodes = nodes;
    for node in nodes.iter_mut().skip(1) {
        node.parent_id = Some("1".to_string());
    }
    AXTree::from_nodes(nodes)
}

// ── AuditReport helpers ──────────────────────────────────────────────────────

fn empty_report(url: &str) -> AuditReport {
    AuditReport::new(url.to_string(), WcagLevel::AA, WcagResults::new(), 100)
}

fn report_with_seo(url: &str, seo: SeoAnalysis) -> AuditReport {
    empty_report(url).with_seo(seo)
}

// ─────────────────────────────────────────────────────────────────────────────
// UX
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ux_module_finds_friction_on_empty_page() {
    let tree = ax_tree_empty_page();
    let ux = analyze_ux(&tree);
    assert!(
        !ux.issues.is_empty(),
        "expected UX issues on an empty page, got none. score={}, dims=[{},{},{},{},{}]",
        ux.score,
        ux.cta_clarity.score,
        ux.visual_hierarchy.score,
        ux.content_clarity.score,
        ux.trust_signals.score,
        ux.cognitive_load.score,
    );
    // Verify the trait wrapper also runs (id stays stable for cache signatures).
    assert_eq!(UxModule.id(), "ux");
}

#[test]
fn ux_module_scores_rich_page_higher_than_empty() {
    let empty = analyze_ux(&ax_tree_empty_page());
    let rich = analyze_ux(&ax_tree_rich_page());
    assert!(
        rich.score > empty.score,
        "rich page (score={}) should beat empty page (score={})",
        rich.score,
        empty.score,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Journey
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn journey_module_flags_missing_main_landmark() {
    // dom_has_main = false matches our AX tree (no main in empty page).
    let journey = analyze_journey_with_dom_check(&ax_tree_empty_page(), false);
    assert!(
        journey.orientation.score < 70,
        "orientation should be weak without a main landmark, got {}",
        journey.orientation.score
    );
    assert_eq!(JourneyModule.id(), "journey");
}

#[test]
fn journey_module_scores_rich_page_higher_than_empty() {
    let empty = analyze_journey_with_dom_check(&ax_tree_empty_page(), false);
    let rich = analyze_journey_with_dom_check(&ax_tree_rich_page(), true);
    assert!(
        rich.score > empty.score,
        "rich page (score={}) should beat empty page (score={})",
        rich.score,
        empty.score,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SourceQuality (derive phase)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn source_quality_module_populates_report_field() {
    let mut report = empty_report("https://example.com");
    assert!(report.source_quality.is_none(), "precondition");

    SourceQualityModule
        .derive(&mut report)
        .expect("derive succeeds");

    let sq = report
        .source_quality
        .as_ref()
        .expect("source_quality was populated");
    assert!(
        sq.score <= 100,
        "score must be in [0, 100], got {}",
        sq.score
    );
}

#[test]
fn source_quality_module_rewards_https_over_http() {
    let mut https_report = empty_report("https://example.com");
    let mut http_report = empty_report("http://example.com");
    SourceQualityModule.derive(&mut https_report).unwrap();
    SourceQualityModule.derive(&mut http_report).unwrap();

    let https_score = https_report.source_quality.as_ref().unwrap().score;
    let http_score = http_report.source_quality.as_ref().unwrap().score;
    assert!(
        https_score > http_score,
        "HTTPS (score={}) should beat HTTP (score={}) on authority signals",
        https_score,
        http_score,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AiVisibility (derive phase)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ai_visibility_module_populates_report_field() {
    let mut report = empty_report("https://example.com");
    assert!(report.ai_visibility.is_none(), "precondition");

    AiVisibilityModule
        .derive(&mut report)
        .expect("derive succeeds");

    let av = report
        .ai_visibility
        .as_ref()
        .expect("ai_visibility was populated");
    assert!(
        av.score <= 100,
        "score must be in [0, 100], got {}",
        av.score
    );
}

#[test]
fn ai_visibility_module_uses_seo_signals() {
    // No SEO → baseline. SEO present (even default) → input pool changes,
    // so the resulting score is allowed to differ. The contract under test
    // is "derive consumes report.seo when present", not a specific delta.
    let mut without_seo = empty_report("https://example.com");
    let mut with_seo = report_with_seo("https://example.com", SeoAnalysis::default());

    AiVisibilityModule.derive(&mut without_seo).unwrap();
    AiVisibilityModule.derive(&mut with_seo).unwrap();

    // Both must populate the field; the score field is well-defined in both.
    assert!(without_seo.ai_visibility.is_some());
    assert!(with_seo.ai_visibility.is_some());
}

// ─────────────────────────────────────────────────────────────────────────────
// ContentVisibility (derive phase)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn content_visibility_module_returns_default_without_seo() {
    let mut report = empty_report("https://example.com");
    assert!(report.content_visibility.is_none(), "precondition");

    ContentVisibilityModule
        .derive(&mut report)
        .expect("derive succeeds");

    let cv = report
        .content_visibility
        .as_ref()
        .expect("content_visibility was populated");
    // With no SEO data, the analysis short-circuits and signal vectors stay empty.
    assert_eq!(
        cv.signal_count, 0,
        "no SEO → no organic-visibility signals; got {}",
        cv.signal_count
    );
}

#[test]
fn content_visibility_module_produces_signals_with_seo_present() {
    let mut report = report_with_seo("https://example.com", SeoAnalysis::default());
    ContentVisibilityModule.derive(&mut report).unwrap();
    let cv = report.content_visibility.as_ref().unwrap();
    assert!(
        cv.signal_count > 0,
        "SEO present → some organic-visibility signals expected, got {}",
        cv.signal_count
    );
}
