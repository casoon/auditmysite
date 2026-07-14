//! WCAG Rules Module
//!
//! Contains individual WCAG rule implementations.

mod abbreviations;
mod accessible_name;
mod aria_allowed_attr;
mod aria_hidden_focus;
mod aria_naming_rules;
mod aria_prohibited_attr;
mod aria_relationships;
mod aria_required_attr;
mod aria_required_parent;
mod aria_roles;
mod aria_valid_attr_value;
mod background_audio;
mod bypass_blocks;
mod click_handlers;
mod content_on_hover;
mod contrast;
mod dialog_rules;
mod error_identification;
mod focus_order;
mod focus_visible;
mod focus_visible_css;
mod form_rules;
mod headings;
mod help;
mod identify_purpose;
mod iframe_rules;
mod image_input_rules;
mod info_relationships;
mod input_purpose;
mod instructions;
mod keyboard;
mod label_in_name;
mod label_title_only;
mod labels;
mod landmark_granular;
mod landmarks;
mod language;
mod language_extended;
mod link_purpose;
mod link_purpose_link_only;
mod list_structure;
mod location;
mod meaningful_sequence;
mod media_alternative;
mod media_rules;
mod meta_viewport_large;
mod modern_attributes;
mod motion_actuation;
mod no_interruptions;
mod no_timing;
mod non_text_contrast_css;
mod on_focus;
mod on_input;
mod orientation;
mod page_rules;
mod page_titled;
mod parsing;
mod pointer_cancellation;
mod pointer_gestures;
mod re_authenticate;
mod reduced_motion;
mod redundant_entry;
mod reflow;
mod region;
mod resize_text;
mod section_headings;
mod server_side_image_map;
mod status_messages;
mod summary_name;
mod svg_rules;
mod table_extended;
mod table_rules;
mod target_size_enhanced;
mod target_size_minimum;
mod text_alternatives;
mod text_spacing;
mod timing_adjustable;
mod unusual_words;
mod use_of_color;
mod visual_presentation;
mod widget_rules;

pub use abbreviations::check_abbreviations_with_page;
pub use accessible_name::check_accessible_name;
pub use aria_allowed_attr::check_aria_allowed_attr_with_page;
pub use aria_hidden_focus::check_aria_hidden_focus;
pub use aria_naming_rules::check_aria_naming_rules;
pub use aria_prohibited_attr::{check_aria_prohibited_attr, check_aria_prohibited_attr_with_page};
pub use aria_relationships::check_aria_relationships;
pub use aria_required_attr::{check_aria_required_attr, check_checked_state_with_page};
pub use aria_required_parent::check_aria_required_parent;
pub use aria_roles::{
    check_aria_roles, check_invalid_aria_attribute_name_with_page, check_invalid_role_with_page,
};
pub use aria_valid_attr_value::check_aria_valid_attr_value_with_page;
pub use background_audio::check_background_audio_with_page;
pub use bypass_blocks::check_bypass_blocks;
pub use click_handlers::check_click_handlers_with_page;
pub use content_on_hover::check_content_on_hover_with_page;
pub use contrast::{Color, ContrastRule};
pub use dialog_rules::check_dialog_rules;
pub use error_identification::check_error_identification;
pub use focus_order::{check_focus_order, check_positive_tabindex_with_page};
pub use focus_visible::check_focus_visible;
pub use focus_visible_css::check_focus_visible_css_with_page;
pub use form_rules::{check_form_no_submit_with_page, check_form_rules};
pub use headings::check_headings;
pub use help::check_help;
pub use identify_purpose::check_identify_purpose_with_page;
pub use iframe_rules::check_same_origin_iframes_with_page;
pub use image_input_rules::check_image_input_rules_with_page;
pub use info_relationships::{
    check_info_relationships, check_presentation_semantic_children_with_page,
};
pub use input_purpose::check_input_purpose;
pub use instructions::check_instructions;
pub use keyboard::check_keyboard;
pub use label_in_name::check_label_in_name_with_page;
pub use label_title_only::check_label_title_only;
pub use labels::check_labels;
pub use landmark_granular::{
    check_landmark_banner_is_top_level, check_landmark_banner_present,
    check_landmark_contentinfo_is_top_level, check_landmark_main_is_top_level,
    check_landmark_main_present, check_landmark_no_duplicate_banner,
    check_landmark_no_duplicate_contentinfo, check_landmark_no_duplicate_main,
    check_landmark_unique, check_landmarks_with_page, check_skip_link,
};
pub use landmarks::check_landmarks;
pub use language::{check_language, LANGUAGE_RULE};
pub use language_extended::check_language_extended_with_page;
pub use link_purpose::check_link_purpose;
pub use link_purpose_link_only::check_link_purpose_link_only;
pub use list_structure::check_list_structure;
pub use location::check_location_with_page;
pub use meaningful_sequence::check_meaningful_sequence_with_page;
pub use media_alternative::check_media_alternative;
pub use media_rules::{
    check_frame_tested_with_page, check_frame_title_with_page, check_media_rules,
};
pub use meta_viewport_large::check_meta_viewport_large_with_page;
pub use modern_attributes::check_modern_attributes_with_page;
pub use motion_actuation::check_motion_actuation_with_page;
pub use no_interruptions::check_no_interruptions_with_page;
pub use no_timing::check_no_timing_with_page;
pub use non_text_contrast_css::check_non_text_contrast_css_with_page;
pub use on_focus::check_on_focus_with_page;
pub use on_input::check_on_input_with_page;
pub use orientation::check_orientation_with_page;
pub use page_rules::{PageRuleEntry, PAGE_RULES};
pub use page_titled::{check_page_titled, check_page_titled_with_page};
pub use parsing::{check_parsing, check_parsing_with_page};
pub use pointer_cancellation::check_pointer_cancellation_with_page;
pub use pointer_gestures::check_pointer_gestures_with_page;
pub use re_authenticate::check_re_authenticate_with_page;
pub use reduced_motion::check_reduced_motion_with_page;
pub use redundant_entry::check_redundant_entry_with_page;
pub use reflow::check_reflow_with_page;
pub use region::check_region;
pub use resize_text::check_resize_text_with_page;
pub use section_headings::check_section_headings;
pub use server_side_image_map::check_server_side_image_map_with_page;
pub use status_messages::check_status_messages;
pub use summary_name::check_summary_name;
pub use svg_rules::check_svg_rules;
pub use table_extended::{check_table_extended, check_table_headers_attr_with_page};
pub use table_rules::check_table_rules;
pub use target_size_enhanced::check_target_size_enhanced_with_page;
pub use target_size_minimum::check_target_size_minimum_with_page;
pub use text_alternatives::check_text_alternatives;
pub use text_spacing::check_text_spacing_with_page;
pub use timing_adjustable::{check_timeouts_with_page, check_timing_with_page};
pub use unusual_words::check_unusual_words;
pub use use_of_color::check_use_of_color_with_page;
pub use visual_presentation::check_visual_presentation_with_page;
pub use widget_rules::{check_tab_selected_state_with_page, check_widget_rules};
