//! Output formatting module
//!
//! Provides formatters for different output formats: JSON, CLI tables, PDF (Typst).

pub mod ai;
pub mod builder;
mod cli;
pub mod explanations;
mod json;
#[cfg(feature = "pdf")]
mod pdf;
pub mod report_model;

pub use ai::format_ai_json;
pub use cli::{format_batch_table, format_violations_list, print_batch_table, print_report};
pub use json::{format_json_batch, format_json_cached, format_json_normalized, JsonReport};
#[cfg(feature = "pdf")]
pub use pdf::{generate_batch_pdf, generate_comparison_pdf, generate_pdf};
