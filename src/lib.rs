//! AuditMySite - Resource-efficient WCAG 2.2 AA Accessibility Checker
//!
//! **Stability:** the CLI, the JSON report and the curated library surface listed in
//! `docs/reference/api.md` are stable; every other `pub` item, and every `#[doc(hidden)]` one, is
//! public only for the binaries, the integration tests and Studio and may change in any release.
//! Modules are narrowed to `pub(crate)` one by one (#940).
//!
//! A fast, accurate accessibility auditing tool written in Rust.
//! Uses Chrome DevTools Protocol (CDP) to extract the Accessibility Tree
//! and analyze it for WCAG violations.
//!
//! ## Features
//!
//! - **Fast**: Rust performance with async processing
//! - **Accurate**: Uses browser's native Accessibility Tree
//! - **Focused**: automates a documented subset of the 55 WCAG 2.2 A/AA
//!   criteria (see `docs/PARITY_CONTRACT.jsonc`), plus a few AAA criteria;
//!   AAA coverage is not comprehensive
//! - **Flexible**: CLI, library, and API interfaces
//!
//! ## Quick Start
//!
//! ```no_run
//! use auditmysite::browser::BrowserManager;
//! use auditmysite::audit::{run_single_audit, ModuleSet, PipelineConfig};
//! use auditmysite::model::WcagLevel;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create browser manager (auto-detects Chrome)
//!     let browser = BrowserManager::new().await?;
//!
//!     // Configure the audit. `PipelineConfig::default()` is the default CLI
//!     // run (WCAG AA, every module); this narrows it to the accessibility
//!     // checks. Fields without a `with_*` setter are assigned directly.
//!     let mut config = PipelineConfig::default()
//!         .with_wcag_level(WcagLevel::AA)
//!         .with_modules(ModuleSet::none());
//!     config.timeout_secs = 30;
//!
//!     // Run audit
//!     let report = run_single_audit("https://example.com", &browser, &config).await?;
//!
//!     // Print results
//!     println!("Score: {}", report.accessibility.score);
//!     println!("Passed: {}", report.passed());
//!     println!("Violations: {}", report.violation_count());
//!
//!     // Close browser
//!     browser.close().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Public surface
//!
//! The entry points are in [`audit`] (pipeline, batch, report types, normalised report),
//! [`browser`] (browser management), [`output`] (JSON, SARIF, PDF and table writers), [`error`] and
//! [`model`] (shared value types); [`studio`] is the contract with auditmysite_studio. The
//! other `pub` modules are not covered by the stability promise and are being narrowed; the
//! table of what is covered, what is queued and what is already `pub(crate)` is in
//! `docs/reference/api.md`. See `docs/ARCHITECTURE.md` for the full module structure.
//!
//! ## WCAG coverage
//!
//! Which WCAG 2.2 A/AA criteria are automated and which need manual review is
//! frozen in the
//! [parity contract](https://github.com/casoon/auditmysite/blob/main/docs/PARITY_CONTRACT.md).

#![warn(unreachable_pub)]
// docs.rs builds without `--cap-lints warn` since rust-lang/docs.rs#3555, so a
// rustdoc lint fails the docs.rs build instead of shipping broken pages (#588).
#![deny(rustdoc::all)]

pub mod a11y_journey;
pub mod accessibility;
pub mod ai_visibility;
pub mod annotations;
pub(crate) mod assessment;
pub mod audit;
pub mod best_practices;
pub mod browser;
pub mod cli;
pub(crate) mod commerce;
pub mod content_visibility;
pub mod display;
pub mod error;
pub(crate) mod html_conform;
pub mod i18n;
pub mod interaction;
pub mod lint;
pub mod mobile;
pub mod model;
pub mod output;
pub mod page_flow;
pub mod patterns;
pub mod performance;
pub mod privacy;
pub mod registry;
pub mod screen_reader;
pub mod security;
pub mod seo;
pub mod source_quality;
pub mod studio;
pub mod taxonomy;
pub(crate) mod tech_stack;
pub mod util;
pub mod ux;
pub mod wcag;

// Re-export commonly used types
pub use accessibility::{AXNode, AXTree};
pub use ai_visibility::{analyze_ai_visibility, AiVisibilityAnalysis};
pub use assessment::{
    signal_from_check, AssessmentLevel, ContentArea, ContentEvidence, ContentSignal,
    EvidenceConfidence, EvidenceSource,
};
pub use audit::{
    analyze_crawl_links, audit_page, crawl_site, parse_sitemap, read_url_file,
    run_concurrent_batch, AuditReport, BatchConfig, BatchReport, BrokenLink, BrokenLinkSeverity,
    CrawlDiagnostics, CrawlNode, CrawlResult, PerformanceResults, PipelineConfig, RedirectChain,
    Verdict, VerdictResult,
};
pub use browser::{
    detect_all_browsers, resolve_browser, BrowserInstaller, BrowserKind, BrowserManager,
    BrowserMode, BrowserOptions, BrowserPool, BrowserResolveOptions, BrowserSource,
    DetectedBrowser, InstallTarget, PoolConfig, ResolvedBrowser,
};
pub use cli::{Args, BrowserAction, Command, OutputFormat, WcagLevel};
pub use content_visibility::{analyze_content_visibility, ContentVisibilityAnalysis};
pub use error::{AuditError, Result};
pub use html_conform::HtmlConformAnalysis;
pub use mobile::{analyze_mobile_friendliness, MobileFriendliness};
pub use output::format_json_normalized;
pub use page_flow::{analyze_journey, analyze_journey_with_dom_check, JourneyAnalysis, PageIntent};
pub use performance::{
    calculate_performance_score, extract_web_vitals, PerformanceScore, WebVitals,
};
pub use screen_reader::{
    announce, announce_localized, linearize, linearize_with_ignored, navigation_views,
    IgnoredReadingNode, NavigationViews, ReadingItem,
};
pub use security::{analyze_security, SecurityAnalysis};
pub use seo::{analyze_seo, SeoAnalysis};
pub use source_quality::{analyze_source_quality, SourceQualityAnalysis};
pub use tech_stack::TechStackAnalysis;
pub use ux::{analyze_ux, UxAnalysis};
pub use wcag::{Severity, Violation, WcagResults};
