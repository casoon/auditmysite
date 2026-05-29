//! Audit orchestration module
//!
//! Coordinates the audit pipeline from URL input to report output.

pub mod artifacts;
pub mod baseline;
mod batch;
pub mod batch_consistency;
pub mod budget;
pub mod catalog;
mod crawl;
pub mod duplicate;
pub mod interpretation;
pub mod module;
pub mod normalized;
mod pipeline;
mod report;
mod scoring;
pub mod summary;

pub use artifacts::{
    cache_matches_signature, content_hash, load_artifacts, save_artifacts, to_audit_report,
    AuditArtifacts, FetchArtifact, SnapshotArtifact,
};
pub use baseline::{Baseline, BaselineDiff, BaselineViolation, WaivedViolation};
pub use batch::{
    count_sitemap_entries_shallow, parse_sitemap, read_url_file, run_concurrent_batch, BatchConfig,
    BatchResult,
};
pub use budget::{evaluate_budgets, BudgetSeverity, BudgetViolation};
pub use catalog::AuditCatalog;
pub use crawl::{analyze_crawl_links, crawl_site, CrawlNode, CrawlResult};
pub use duplicate::{detect_near_duplicates, DuplicatePair};
pub use module::{AuditModule, ModuleContext, ModuleData, Viewport};
pub use normalized::{normalize, NormalizedReport};
pub use pipeline::{
    audit_page, run_single_audit, AuditModuleDefinition, ModuleActivation, PipelineConfig,
    AUDIT_MODULES,
};
pub use report::{
    AuditReport, BatchError, BatchReport, BatchSummary, BrokenLink, BrokenLinkSeverity,
    CrawlDiagnostics, PerformanceResults, RedirectChain, SampleMetadata, ScreenshotStatus,
    ThrottledPerfResult, ViewportScoreSet, ViewportScores, ViewportScreenshot,
};
pub use scoring::{AccessibilityScorer, CoverageRatio, PrincipleCoverage, ViolationStatistics};
