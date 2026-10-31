//! Audit orchestration module
//!
//! Coordinates the audit pipeline from URL input to report output.

mod access_block;
pub mod artifacts;
mod batch;
pub mod batch_consistency;
pub mod budget;
pub(crate) mod capture_collectors;
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
pub mod support_matrix;
pub mod template_dedup;
pub mod validation;
pub mod verdict;

pub use artifacts::{
    cache_matches_signature, cached_report, content_hash, hydrate_cached_report, load_artifacts,
    lookup_artifacts, profile_differences, replay, replay_report, replay_report_as, save_artifacts,
    to_audit_report, AuditArtifacts, CacheLookup, CacheMiss, DocumentIdentity, ExpectedProfile,
    FetchArtifact, ReplayError, SnapshotArtifact, SnapshotVersions,
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
pub use normalized::{AuditContext, NormalizedReport};

/// Infallible normalization of a raw report: it derives a model even from a
/// report that contradicts itself, so a caller can hand out data nobody
/// checked.
///
/// Compatibility surface of 1.x, removed in 2.0. Migrate to
/// [`normalize_checked`], which returns the same model or the
/// [`ReportContractError`] naming the violated contract. The productive paths
/// of this crate never call this function (#790).
#[deprecated(
    since = "1.9.0",
    note = "infallible and unchecked; removed in 2.0 — use `normalize_checked`"
)]
pub fn normalize(report: &AuditReport) -> AuditContext<'_> {
    normalized::normalize(report)
}
pub use pipeline::{audit_page, run_single_audit, PipelineConfig};
pub use report::{
    compute_worst_risk, count_violated_rules, AccessibilitySection, AuditExecution, AuditQuality,
    AuditQualityStatus, AuditReport, AuditScope, AuditedContentState, BatchCoverage, BatchError,
    BatchReport, BatchSummary, BrokenLink, BrokenLinkSeverity, ConsentAuditState,
    ConsentCookieSignal, ConsentPrivacySnapshot, CrawlDepthDiagnostics, CrawlDepthEntry,
    CrawlDiagnostics, DiscoverabilitySection, DualViewportResults, ExecutionEnvironment,
    ExecutionStatus, ExperienceSection, MeasurementProfile, MetricSpread, ModuleRun,
    NavigationSnapshot, PageScreenshots, PerformanceResults, RedirectChain, RedirectHop,
    RequestedRule, RobotsSitemapConflict, RuleFilterRecord, RunContext, RunSpread, SampleMetadata,
    SampleStratum, ScreenshotStatus, SitemapDiagnostics, SitemapHttpIssue, StorageStateRef,
    SubcheckRun, ThrottledPerfResult, ViewportAuditData, ViewportDefinition, ViewportMetaRefresh,
    ViewportScoreSet, ViewportScores, ViewportScreenshot, MEASUREMENT_PROFILE_VERSION,
};
pub(crate) use scoring::{default_impact, score_from_penalties};
pub use scoring::{AccessibilityScorer, CoverageRatio, PrincipleCoverage, ViolationStatistics};
pub use template_dedup::{detect_root_causes, RootCause};
pub use validation::{
    normalize_checked, validate_batch, ContractViolation, ReportContractError, CONTRACT_CODES,
};
pub use verdict::{
    compute_batch_verdict, compute_verdict, DecisionProfile, Verdict, VerdictResult,
    DECISION_CONTRACT,
};
