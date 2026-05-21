//! CLI module for AuditMySit
//!
//! Command-line interface using clap for argument parsing and config file loading.

mod args;
pub mod config;
pub mod doctor;

pub use args::{Args, BrowserAction, Command, OutputFormat, ReportLevel, WcagLevel};
pub use config::Config;
