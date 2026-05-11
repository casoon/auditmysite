//! WCAG Accessibility Rules Module
//!
//! Provides WCAG 2.1 rule checking against the Accessibility Tree.

pub mod coverage;
pub mod engine;
pub mod rules;
pub mod types;

pub use engine::{
    check_all, check_all_with_config, check_click_handlers_with_page,
    check_content_on_hover_with_page, check_focus_visible_css_with_page,
    check_orientation_with_page, check_reduced_motion_with_page, check_reflow_with_page,
    check_timing_with_page, check_use_of_color_with_page, RuleFilterConfig,
};
pub use types::{RuleMetadata, Severity, Violation, WcagResults};
