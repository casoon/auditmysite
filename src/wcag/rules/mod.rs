//! WCAG Rules Module
//!
//! Contains individual WCAG rule implementations.

mod accessible_name;
mod aria_allowed_attr;
mod aria_naming_rules;
mod aria_prohibited_attr;
mod aria_relationships;
mod aria_required_attr;
mod aria_required_parent;
mod aria_roles;
mod bypass_blocks;
mod contrast;
mod dialog_rules;
mod focus_order;
mod focus_visible;
mod form_rules;
mod headings;
mod image_input_rules;
mod info_relationships;
mod input_purpose;
mod instructions;
mod keyboard;
mod label_in_name;
mod label_title_only;
mod labels;
mod landmark_extended;
mod landmark_granular;
mod landmarks;
mod language;
mod language_extended;
mod link_purpose;
mod list_structure;
mod media_rules;
mod meta_viewport_large;
mod non_text_contrast;
mod on_focus;
mod on_input;
mod page_titled;
mod region;
mod resize_text;
mod section_headings;
mod server_side_image_map;
mod summary_name;
mod svg_rules;
mod table_extended;
mod table_rules;
mod text_alternatives;
mod wcag22_rules;
mod widget_rules;

pub use accessible_name::check_accessible_name;
pub use aria_allowed_attr::check_aria_allowed_attr;
pub use aria_naming_rules::check_aria_naming_rules;
pub use aria_prohibited_attr::check_aria_prohibited_attr;
pub use aria_relationships::check_aria_relationships;
pub use aria_required_attr::check_aria_required_attr;
pub use aria_required_parent::check_aria_required_parent;
pub use aria_roles::check_aria_roles;
pub use bypass_blocks::check_bypass_blocks;
pub use contrast::{Color, ContrastRule};
pub use dialog_rules::check_dialog_rules;
pub use focus_order::check_focus_order;
pub use focus_visible::check_focus_visible;
pub use form_rules::check_form_rules;
pub use headings::check_headings;
pub use image_input_rules::check_image_input_rules;
pub use info_relationships::check_info_relationships;
pub use input_purpose::check_input_purpose;
pub use instructions::check_instructions;
pub use keyboard::check_keyboard;
pub use label_in_name::check_label_in_name;
pub use label_title_only::check_label_title_only;
pub use labels::check_labels;
pub use landmark_extended::check_landmark_extended;
pub use landmark_granular::{
    check_landmark_banner_is_top_level, check_landmark_contentinfo_is_top_level,
    check_landmark_main_is_top_level, check_landmark_no_duplicate_banner,
    check_landmark_no_duplicate_contentinfo, check_landmark_no_duplicate_main,
    check_landmark_unique,
};
pub use landmarks::check_landmarks;
pub use language::check_language;
pub use language_extended::check_language_extended;
pub use link_purpose::check_link_purpose;
pub use list_structure::check_list_structure;
pub use media_rules::check_media_rules;
pub use meta_viewport_large::check_meta_viewport_large;
pub use non_text_contrast::check_non_text_contrast;
pub use on_focus::check_on_focus;
pub use on_input::check_on_input;
pub use page_titled::check_page_titled;
pub use region::check_region;
pub use resize_text::check_resize_text;
pub use section_headings::check_section_headings;
pub use server_side_image_map::check_server_side_image_map;
pub use summary_name::check_summary_name;
pub use svg_rules::check_svg_rules;
pub use table_extended::check_table_extended;
pub use table_rules::check_table_rules;
pub use text_alternatives::check_text_alternatives;
pub use wcag22_rules::check_wcag22_rules;
pub use widget_rules::check_widget_rules;
