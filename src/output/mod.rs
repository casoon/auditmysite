//! Output formatting module
//!
//! Provides formatters for different output formats: JSON, CLI tables, HTML, PDF (Typst).

mod cli;
mod html;
mod json;
mod pdf;

pub use cli::{format_violations_list, print_report};
pub use html::{format_batch_html, format_html};
pub use json::{format_json, JsonReport};
pub use pdf::{generate_batch_pdf, generate_pdf};
