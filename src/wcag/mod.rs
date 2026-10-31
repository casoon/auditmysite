//! WCAG Accessibility Rules Module
//!
//! Provides WCAG 2.1 rule checking against the Accessibility Tree.

pub mod bik_guide;
pub mod coverage;
pub mod en301549;
pub mod engine;
pub mod expectation;
pub mod rules;
pub mod shared;
pub mod types;

pub use engine::{
    check_all, check_all_excluding, check_all_with_config, check_orientation_with_page,
    check_pointer_cancellation_with_page, check_reflow_with_page, check_timeouts_with_page,
    check_timing_with_page, check_visual_presentation_with_page, RuleFilterConfig,
};
pub use types::{
    rule_run_errored, rule_run_skipped, technical_failure_reason, technical_rule_failure,
    technical_rule_failure_for, Evidence, NotRun, Outcome, RuleMetadata, RuleRun, Severity,
    Violation, WcagResults,
};
