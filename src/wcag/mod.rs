//! WCAG Accessibility Rules Module
//!
//! Provides WCAG 2.1 rule checking against the Accessibility Tree.

pub mod coverage;
pub mod engine;
pub mod rules;
pub mod types;

pub use engine::{
    check_abbreviations_with_page, check_all, check_all_with_config,
    check_background_audio_with_page, check_click_handlers_with_page,
    check_content_on_hover_with_page, check_focus_visible_css_with_page,
    check_identify_purpose_with_page, check_label_in_name_with_page, check_location_with_page,
    check_motion_actuation_with_page, check_no_interruptions_with_page, check_no_timing_with_page,
    check_orientation_with_page, check_parsing_with_page, check_pointer_cancellation_with_page,
    check_pointer_gestures_with_page, check_re_authenticate_with_page,
    check_reduced_motion_with_page, check_reflow_with_page, check_target_size_enhanced_with_page,
    check_timeouts_with_page, check_timing_with_page, check_use_of_color_with_page,
    check_visual_presentation_with_page, RuleFilterConfig,
};
pub use types::{FindingKind, RuleMetadata, Severity, Violation, ViolationEvidence, WcagResults};
