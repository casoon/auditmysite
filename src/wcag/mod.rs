//! WCAG Accessibility Rules Module
//!
//! Provides WCAG 2.1 rule checking against the Accessibility Tree.

pub mod engine;
pub mod rules;
pub mod types;

pub use engine::check_all;
pub use types::{RuleMetadata, Severity, Violation, WcagResults};
