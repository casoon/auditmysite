//! Output formatting module
//!
//! Provides formatters for different output formats: JSON, CLI tables, PDF (Typst).

pub mod ai;
pub mod builder;
mod cli;
pub mod explanations;
mod json;
pub mod module;
#[cfg(feature = "pdf")]
mod pdf;
pub mod report_model;
pub mod snapshot_export;
pub mod summary;

pub use ai::format_ai_json;
pub use cli::{format_batch_table, format_violations_list, print_batch_table, print_report};
pub use json::{format_json_batch, format_json_cached, format_json_normalized, UnifiedReport};
#[cfg(feature = "pdf")]
pub use pdf::{
    generate_batch_pdf, generate_batch_typ, generate_comparison_pdf, generate_pdf, generate_typ,
};
pub use snapshot_export::export_snapshot_yaml;
pub use summary::format_summary;

#[cfg(test)]
mod tests;
