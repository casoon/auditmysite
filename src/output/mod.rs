//! Output formatting module
//!
//! Provides formatters for different output formats: JSON, CLI tables, PDF (Typst).

pub mod ai;
pub mod builder;
pub mod explanations;
mod json;
mod localized;
pub mod module;
#[cfg(feature = "pdf")]
mod pdf;
pub mod renderer;
pub mod report_model;
pub mod sarif;
pub mod search_experience;
pub mod snapshot_export;
pub mod sr_audit_json;
pub mod summary;
pub mod technician;
pub mod terminal;

pub use ai::format_ai_json;
pub use json::{format_json_batch, format_json_cached, format_json_normalized, UnifiedReport};
#[cfg(feature = "pdf")]
pub use pdf::{generate_batch_pdf, generate_batch_typ, generate_pdf, generate_typ};
pub use renderer::{JsonRenderer, ReportRenderer, SummaryRenderer};
pub use sarif::format_sarif;
pub use snapshot_export::export_snapshot_yaml;
pub use sr_audit_json::export_sr_audit;
pub use summary::format_summary;

#[cfg(test)]
mod tests;
