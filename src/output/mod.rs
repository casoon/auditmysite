//! Output formatting module
//!
//! Provides formatters for different output formats: JSON, CLI tables, PDF (Typst).

mod cli;
pub mod explanations;
mod json;
#[cfg(feature = "pdf")]
mod pdf;
pub mod report_builder;
pub mod report_model;

pub use cli::{format_violations_list, print_batch_table, print_report};
pub use json::{format_json_batch, format_json_cached, format_json_normalized, JsonReport};
#[cfg(feature = "pdf")]
pub use pdf::{generate_batch_pdf, generate_pdf};
