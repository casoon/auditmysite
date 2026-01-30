//! Audit orchestration module
//!
//! Coordinates the audit pipeline from URL input to report output.

mod batch;
mod pipeline;
mod report;
mod scoring;

pub use batch::{parse_sitemap, read_url_file, run_concurrent_batch, BatchConfig, BatchResult};
pub use pipeline::{audit_page, run_batch_audit, run_single_audit, PipelineConfig};
pub use report::{AuditReport, BatchReport, BatchSummary, PerformanceResults};
pub use scoring::{AccessibilityScorer, PrincipleBreakdown, ViolationStatistics};
