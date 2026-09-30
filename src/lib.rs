//! AuditMySite - Resource-efficient WCAG 2.2 AA Accessibility Checker
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
//! use auditmysite::audit::{run_single_audit, PipelineConfig};
//! use auditmysite::cli::WcagLevel;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create browser manager (auto-detects Chrome)
//!     let browser = BrowserManager::new().await?;
//!
//!     // Configure the audit
//!     let config = PipelineConfig {
//!         wcag_level: WcagLevel::AA,
//!         full_audit: false,
//!         timeout_secs: 30,
//!         stability_budget_ms: 1500,
//!         verbose: false,
//!         check_performance: false,
//!         check_seo: false,
//!         check_security: false,
//!         check_mobile: false,
//!         check_dark_mode: false,
//!         check_html_conform: false,
//!         check_design_quality: false,
//!         check_ai_transparency: false,
//!         check_dns: false,
//!         check_isolate_third_party_impact: false,
//!         check_ssr_content: false,
//!         check_stack: false,
//!         rule_filter: auditmysite::wcag::RuleFilterConfig::default(),
//!         persist_artifacts: true,
//!         capture_screenshots: false,
//!         capture_element_evidence: false,
//!         dismiss_consent: false,
//!         exclude_selectors: Vec::new(),
//!         interactive: auditmysite::cli::InteractiveMode::Off,
//!         journey_budget_ms: auditmysite::a11y_journey::DEFAULT_BUDGET_MS,
//!         lang: "de".to_string(),
//!         display_mode: None,
//!     };
//!
//!     // Run audit
//!     let report = run_single_audit("https://example.com", &browser, &config).await?;
//!
//!     // Print results
//!     println!("Score: {}", report.accessibility.score);
//!     println!("Violations: {}", report.violation_count());
//!
//!     // Close browser
//!     browser.close().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - Browser and capture: [`browser`], [`accessibility`] (AXTree extraction),
//!   [`interaction`], [`a11y_journey`]
//! - Accessibility checks: [`wcag`] (rule engine), [`screen_reader`],
//!   [`html_conform`], [`patterns`]
//! - Further analysis modules: [`performance`], [`seo`], [`security`],
//!   [`mobile`], [`best_practices`], [`dark_mode`], [`design_quality`],
//!   [`journey`], [`ux`], [`content_visibility`], [`ai_visibility`],
//!   [`ai_transparency`], [`source_quality`], [`tech_stack`], [`commerce`],
//!   [`network`]
//! - Orchestration and scoring: [`audit`] (pipeline, batch, normalization),
//!   [`taxonomy`], [`assessment`], [`registry`]
//! - Output: [`output`] (JSON, table, PDF), [`lint`] (report-lint),
//!   [`studio`] (types shared with auditmysite_studio), [`i18n`]
//! - Entry points and support: [`cli`], [`error`], [`util`]
//!
//! See `docs/ARCHITECTURE.md` for the full module structure.
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
pub mod ai_transparency;
pub mod ai_visibility;
pub mod assessment;
pub mod audit;
pub mod best_practices;
pub mod browser;
pub mod cli;
pub mod commerce;
pub mod content_visibility;
pub mod dark_mode;
pub mod design_quality;
pub mod display;
pub mod error;
pub mod html_conform;
pub mod i18n;
pub mod interaction;
pub mod journey;
pub mod lint;
pub mod mobile;
pub mod network;
pub mod output;
pub mod patterns;
pub mod performance;
pub mod registry;
pub mod screen_reader;
pub mod security;
pub mod seo;
pub mod source_quality;
pub mod studio;
pub mod taxonomy;
pub mod tech_stack;
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
pub use dark_mode::{analyze_dark_mode, DarkModeAnalysis, DarkModeIssue, DarkModeIssueKind};
pub use error::{AuditError, Result};
pub use journey::{analyze_journey, analyze_journey_with_dom_check, JourneyAnalysis, PageIntent};
pub use mobile::{analyze_mobile_friendliness, MobileFriendliness};
pub use output::format_json_normalized;
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
pub use ux::{analyze_ux, UxAnalysis};
pub use wcag::{Severity, Violation, WcagResults};
