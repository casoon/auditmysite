//! auditmysite - Resource-efficient WCAG 2.1 Accessibility Checker
//!
//! A fast, accurate accessibility auditing tool written in Rust.
//! Uses Chrome DevTools Protocol (CDP) to extract the Accessibility Tree
//! and analyze it for WCAG 2.1 violations.
//!
//! ## Features
//!
//! - **Fast**: Rust performance with async processing
//! - **Accurate**: Uses browser's native Accessibility Tree
//! - **Comprehensive**: Checks WCAG 2.1 Level A, AA, and AAA
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
//!         timeout_secs: 30,
//!         verbose: false,
//!     };
//!
//!     // Run audit
//!     let report = run_single_audit("https://example.com", &browser, &config).await?;
//!
//!     // Print results
//!     println!("Score: {}", report.score);
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
//! - [`browser`]: Chrome/Chromium detection and management
//! - [`accessibility`]: Accessibility Tree extraction and structures
//! - [`wcag`]: WCAG rule checking engine
//! - [`audit`]: Audit pipeline and reporting
//! - [`output`]: Report formatters (JSON, CLI, HTML)
//! - [`cli`]: Command-line interface
//! - [`error`]: Error types
//!
//! ## WCAG Rules Implemented
//!
//! | Code | Name | Level |
//! |------|------|-------|
//! | 1.1.1 | Non-text Content | A |
//! | 1.4.3 | Contrast (Minimum) | AA |
//! | 2.1.1 | Keyboard | A |
//! | 2.4.1 | Bypass Blocks | A |
//! | 2.4.4 | Link Purpose (In Context) | A |
//! | 2.4.6 | Headings and Labels | AA |
//! | 3.3.2 | Labels or Instructions | A |
//! | 4.1.2 | Name, Role, Value | A |

pub mod accessibility;
pub mod audit;
pub mod browser;
pub mod cli;
pub mod error;
pub mod mobile;
pub mod output;
pub mod performance;
pub mod security;
pub mod seo;
pub mod wcag;

// Re-export commonly used types
pub use accessibility::{AXNode, AXTree};
pub use audit::{
    parse_sitemap, read_url_file, run_concurrent_batch, AuditReport, BatchConfig, BatchReport,
    PerformanceResults, PipelineConfig,
};
pub use browser::{BrowserManager, BrowserOptions, BrowserPool, PoolConfig};
pub use cli::{Args, OutputFormat, WcagLevel};
pub use error::{AuditError, Result};
pub use mobile::{analyze_mobile_friendliness, MobileFriendliness};
pub use output::{format_batch_html, format_html, format_json, print_report};
pub use performance::{
    calculate_performance_score, extract_web_vitals, PerformanceScore, WebVitals,
};
pub use security::{analyze_security, validate_url, SecurityAnalysis};
pub use seo::{analyze_seo, SeoAnalysis};
pub use wcag::{Severity, Violation, WcagResults};
