//! Output formatting module
//!
//! Provides formatters for different output formats: JSON, CLI tables, PDF (Typst).

mod cli;
mod json;
#[cfg(feature = "pdf")]
mod pdf;

pub use cli::{format_violations_list, print_batch_table, print_report};
pub use json::{format_json, JsonReport};
#[cfg(feature = "pdf")]
pub use pdf::{generate_batch_pdf, generate_pdf};
