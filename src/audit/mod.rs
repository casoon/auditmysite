//! Audit orchestration module
//!
//! Coordinates the audit pipeline from URL input to report output.

pub mod artifacts;
pub mod baseline;
mod batch;
pub mod budget;
pub mod comparison;
mod crawl;
pub mod duplicate;
pub mod history;
pub mod normalized;
mod pipeline;
mod report;
mod scoring;

pub use artifacts::{
    content_hash, load_artifacts, save_artifacts, to_audit_report, AuditArtifacts, FetchArtifact,
    SnapshotArtifact,
};
pub use baseline::{Baseline, BaselineDiff, BaselineViolation, WaivedViolation};
pub use batch::{parse_sitemap, read_url_file, run_concurrent_batch, BatchConfig, BatchResult};
pub use budget::{evaluate_budgets, BudgetSeverity, BudgetViolation};
pub use comparison::{ComparisonEntry, ComparisonReport};
pub use crawl::{analyze_crawl_links, crawl_site, CrawlNode, CrawlResult};
pub use duplicate::{detect_near_duplicates, DuplicatePair};
pub use normalized::{normalize, NormalizedReport};
pub use pipeline::{audit_page, run_single_audit, PipelineConfig};
pub use report::{
    AuditReport, BatchError, BatchReport, BatchSummary, BrokenLink, BrokenLinkSeverity,
    CrawlDiagnostics, PerformanceResults, RedirectChain,
};
pub use scoring::{AccessibilityScorer, ViolationStatistics};
