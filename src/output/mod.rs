//! Output formatting module
//!
//! Provides formatters for different output formats: JSON, CLI tables, HTML, PDF (Typst), Markdown.

mod cli;
mod html;
mod json;
mod markdown;
#[cfg(feature = "pdf")]
mod pdf;

pub use cli::{format_violations_list, print_batch_table, print_report};
pub use html::{format_batch_html, format_html};
pub use json::{format_json, JsonReport};
pub use markdown::{format_batch_markdown, format_markdown};
#[cfg(feature = "pdf")]
pub use pdf::{generate_batch_pdf, generate_pdf};
