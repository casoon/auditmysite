//! WCAG Rules Module
//!
//! Contains individual WCAG rule implementations.

mod accessible_authentication;
mod accessible_name;
mod contrast;
mod display_modes;
mod focus_not_obscured_enhanced;
mod focus_not_obscured_minimum;
mod focus_visible;
mod help;
mod html_content_model;
mod iframe_rules;
mod keyboard;
mod link_purpose;
mod link_purpose_link_only;
mod meaningful_sequence;
mod media_alternative;
mod media_rules;
mod modern_attributes;
mod motion_actuation;
mod no_interruptions;
mod no_timing;
mod non_text_contrast_css;
mod on_input;
mod orientation;
mod page_rules;
mod page_titled;
mod pause_stop_hide;
mod pointer_cancellation;
mod pointer_gestures;
mod re_authenticate;
mod reflow;
mod target_size_enhanced;
mod target_size_minimum;
mod text_alternatives;
mod text_spacing;
mod timing_adjustable;
mod unusual_words;
mod visual_presentation;

pub use accessible_authentication::check_accessible_authentication_with_page;
pub use accessible_name::check_accessible_name;
pub use contrast::{Color, ContrastRule};
pub use display_modes::{
    check_display_modes_with_page, DISPLAY_INIT_MISSING_RULE, DISPLAY_TEXT_HIDDEN_RULE,
    DISPLAY_TEXT_MEDIA_VISIBLE_RULE, DISPLAY_TEXT_NOT_VISIBLE_RULE, DISPLAY_TOGGLE_MISSING_RULE,
};
pub use focus_not_obscured_enhanced::check_focus_not_obscured_enhanced_with_page;
pub use focus_not_obscured_minimum::check_focus_not_obscured_minimum_with_page;
pub use focus_visible::check_focus_visible;
pub use help::check_help;
pub use html_content_model::{check_html_content_model, RULE_META as HTML_CONTENT_MODEL_RULE};
pub use iframe_rules::check_same_origin_iframes_with_page;
pub use keyboard::check_keyboard;
pub use link_purpose::check_link_purpose;
pub use link_purpose_link_only::check_link_purpose_link_only;
pub use meaningful_sequence::check_meaningful_sequence_with_page;
pub use media_alternative::check_media_alternative_with_page;
pub use media_rules::{check_frame_tested_with_page, check_video_caption_tracks_with_page};
pub use modern_attributes::check_modern_attributes_with_page;
pub use motion_actuation::check_motion_actuation_with_page;
pub use no_interruptions::check_no_interruptions_with_page;
pub use no_timing::check_no_timing_with_page;
pub use non_text_contrast_css::check_non_text_contrast_css_with_page;
pub use on_input::check_on_input_with_page;
pub use orientation::check_orientation_with_page;
pub use page_rules::{PageRuleEntry, PAGE_RULES};
pub use page_titled::check_page_titled_with_page;
pub use pause_stop_hide::check_pause_stop_hide_with_page;
pub use pointer_cancellation::check_pointer_cancellation_with_page;
pub use pointer_gestures::check_pointer_gestures_with_page;
pub use re_authenticate::check_re_authenticate_with_page;
pub use reflow::{check_reflow_with_page, REFLOW_RULE};
pub use target_size_enhanced::check_target_size_enhanced_with_page;
pub use target_size_minimum::check_target_size_minimum_with_page;
pub use text_alternatives::{check_text_alternatives, is_svg_finding};
pub use text_spacing::check_text_spacing_with_page;
pub use timing_adjustable::{check_timeouts_with_page, check_timing_with_page};
pub use unusual_words::check_unusual_words;
pub use visual_presentation::check_visual_presentation_with_page;
