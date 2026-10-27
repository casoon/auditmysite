//! Audit orchestration module
//!
//! Coordinates the audit pipeline from URL input to report output.

mod access_block;
pub mod artifacts;
mod batch;
pub mod batch_consistency;
pub mod budget;
pub mod catalog;
pub mod collection;
mod crawl;
pub mod duplicate;
pub mod exclusion;
pub mod finding_contract;
mod finding_derive;
pub mod frames;
mod interactive_finding;
pub mod interpretation;
pub mod management_risk;
pub mod module;
mod module_scores;
pub mod normalized;
pub mod occurrence_analysis;
pub mod performance_interpretation;
mod pipeline;
pub mod prioritization;
mod report;
mod risk_assessment;
mod scoring;
pub mod signal_contract;
pub mod summary;
pub mod template_dedup;
pub mod validation;
pub mod verdict;

pub use artifacts::{
    cache_matches_signature, content_hash, hydrate_cached_report, load_artifacts, save_artifacts,
    to_audit_report, AuditArtifacts, FetchArtifact, SnapshotArtifact,
};
pub use batch::{
    analyze_sitemap_diagnostics, analyze_sitemap_diagnostics_with_progress,
    count_sitemap_entries_shallow, parse_sitemap, read_url_file, run_concurrent_batch,
    BatchAuditError, BatchConfig, BatchResult, ProgressCallback,
};
pub use budget::{evaluate_budgets, BudgetSeverity, BudgetViolation};
pub use catalog::AuditCatalog;
pub use crawl::{analyze_crawl_links, crawl_site, CrawlNode, CrawlResult};
pub use duplicate::{detect_near_duplicates, DuplicatePair};
pub use module::{AuditModule, ModuleContext, ModuleData, Viewport};
pub use normalized::{normalize, AuditContext, NormalizedReport};
pub use pipeline::{audit_page, run_single_audit, PipelineConfig};
pub use report::{
    compute_worst_risk, count_violated_rules, AccessibilitySection, AuditExecution, AuditQuality,
    AuditQualityStatus, AuditReport, AuditScope, AuditedContentState, BatchError, BatchReport,
    BatchSummary, BrokenLink, BrokenLinkSeverity, ConsentAuditState, ConsentCookieSignal,
    ConsentPrivacySnapshot, CrawlDepthDiagnostics, CrawlDepthEntry, CrawlDiagnostics,
    DiscoverabilitySection, DualViewportResults, ExecutionEnvironment, ExecutionStatus,
    ExperienceSection, ModuleRun, NavigationSnapshot, PageScreenshots, PerformanceResults,
    RedirectChain, RobotsSitemapConflict, RunContext, SampleMetadata, ScreenshotStatus,
    SitemapDiagnostics, SitemapHttpIssue, SubcheckRun, ThrottledPerfResult, ViewportAuditData,
    ViewportDefinition, ViewportMetaRefresh, ViewportScoreSet, ViewportScores, ViewportScreenshot,
};
pub(crate) use scoring::{default_impact, score_from_penalties};
pub use scoring::{AccessibilityScorer, CoverageRatio, PrincipleCoverage, ViolationStatistics};
pub use template_dedup::{detect_root_causes, RootCause};
pub use validation::{normalize_checked, validate_batch, ContractViolation, ReportContractError};
pub use verdict::{
    compute_batch_verdict, compute_verdict, DecisionProfile, Verdict, VerdictResult,
    DECISION_CONTRACT,
};
