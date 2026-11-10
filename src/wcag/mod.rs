//! WCAG Accessibility Rules Module
//!
//! Provides WCAG 2.1 rule checking against the Accessibility Tree.

pub mod bik_guide;
pub mod coverage;
pub mod en301549;
pub mod engine;
pub mod expectation;
/// Former home of [`WcagLevel`] (2.0.x); `auditmysite::wcag::level::WcagLevel` still resolves.
pub mod level {
    pub use crate::model::WcagLevel;
}
pub mod rules;
pub mod shared;
pub mod types;

pub use crate::model::WcagLevel;
pub use engine::{
    check_all, check_all_excluding, check_all_with_config, check_pointer_cancellation_with_page,
    check_reflow_with_page, check_timeouts_with_page, check_timing_with_page,
    check_visual_presentation_with_page, RuleFilterConfig,
};
pub use types::{
    rule_run_errored, rule_run_skipped, rule_run_unmeasured, technical_failure_reason,
    technical_rule_failure, technical_rule_failure_for, Evidence, NotRun, Outcome, RuleMetadata,
    RuleRun, Severity, Violation, WcagResults,
};
