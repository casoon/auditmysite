//! CLI module for AuditMySit
//!
//! Command-line interface using clap for argument parsing and config file loading.

mod args;
pub mod config;
pub mod doctor;
pub mod url_filter;

pub use args::{
    AnnexKind, Args, BrowserAction, ColorPolicy, Command, InteractiveMode, OutputFormat,
    ProgressPolicy, ReportLevel, ReportLintFailOn, RequestMode, WcagLevel,
};
pub use config::Config;
