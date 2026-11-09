//! CLI module for AuditMySit
//!
//! Command-line interface using clap for argument parsing and config file loading.

mod args;
pub mod config;
pub mod doctor;
pub mod report_lint;
pub mod url_filter;

pub use args::{
    AnnexKind, Args, BrowserAction, ColorPolicy, Command, DevOptions, DisplaySelection,
    InteractiveMode, ModuleChoice, OutputFormat, ProgressPolicy, ReportLevel, RequestMode,
    SampleStrategy, WcagLevel,
};
pub use config::Config;
