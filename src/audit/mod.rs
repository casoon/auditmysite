//! Audit orchestration module
//!
//! Coordinates the audit pipeline from URL input to report output.

pub mod artifacts;
mod batch;
pub mod history;
pub mod normalized;
mod pipeline;
mod report;
mod scoring;

pub use artifacts::{
    content_hash, load_artifacts, save_artifacts, to_audit_report, AuditArtifacts, FetchArtifact,
    SnapshotArtifact,
};
pub use batch::{parse_sitemap, read_url_file, run_concurrent_batch, BatchConfig, BatchResult};
pub use normalized::{normalize, NormalizedReport};
pub use pipeline::{audit_page, run_single_audit, PipelineConfig};
pub use report::{AuditReport, BatchError, BatchReport, BatchSummary, PerformanceResults};
pub use scoring::{AccessibilityScorer, ViolationStatistics};
