//! CLI module for AuditMySit
//!
//! Command-line interface using clap for argument parsing and config file loading.

mod args;
pub mod config;

pub use args::{Args, OutputFormat, WcagLevel};
pub use config::Config;
