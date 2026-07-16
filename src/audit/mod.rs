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
pub mod occurrence_analysis;
pub mod performance_interpretation;
mod pipeline;
pub mod prioritization;
mod report;
mod scoring;
pub mod summary;
pub mod template_dedup;
pub mod verdict;

pub use artifacts::{
    cache_matches_signature, content_hash, hydrate_cached_report, load_artifacts, save_artifacts,
    to_audit_report, AuditArtifacts, FetchArtifact, SnapshotArtifact,
};
pub use baseline::{Baseline, BaselineDiff, BaselineViolation, WaivedViolation};
pub use batch::{
    analyze_sitemap_diagnostics, count_sitemap_entries_shallow, parse_sitemap, read_url_file,
    run_concurrent_batch, BatchAuditError, BatchConfig, BatchResult,
};
pub use budget::{evaluate_budgets, BudgetSeverity, BudgetViolation};
pub use catalog::AuditCatalog;
pub use crawl::{analyze_crawl_links, crawl_site, CrawlNode, CrawlResult};
pub use duplicate::{detect_near_duplicates, DuplicatePair};
pub use module::{AuditModule, ModuleContext, ModuleData, Viewport};
pub use normalized::{normalize, AuditContext, NormalizedReport};
pub use pipeline::{audit_page, run_single_audit, PipelineConfig};
pub use report::{
    compute_recurring_rules, compute_worst_risk, AccessibilitySection, AuditExecution,
    AuditQuality, AuditQualityStatus, AuditReport, AuditScope, AuditedContentState, BatchError,
    BatchReport, BatchSummary, BrokenLink, BrokenLinkSeverity, ConsentAuditState,
    ConsentCookieSignal, ConsentPrivacySnapshot, CrawlDiagnostics, DiscoverabilitySection,
    DualViewportResults, ExecutionEnvironment, ExecutionStatus, ExperienceSection, ModuleRun,
    NavigationSnapshot, PageScreenshots, PerformanceResults, RecurringRule, RedirectChain,
    SampleMetadata, ScreenshotStatus, SitemapDiagnostics, SitemapHttpIssue, SubcheckRun,
    ThrottledPerfResult, ViewportAuditData, ViewportDefinition, ViewportScoreSet, ViewportScores,
    ViewportScreenshot,
};
pub use scoring::{AccessibilityScorer, CoverageRatio, PrincipleCoverage, ViolationStatistics};
pub use template_dedup::{detect_template_clusters, TemplateCluster};
pub use verdict::{compute_batch_verdict, compute_verdict, Verdict, VerdictResult};
