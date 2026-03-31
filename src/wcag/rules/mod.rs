//! WCAG Rules Module
//!
//! Contains individual WCAG rule implementations.

mod bypass_blocks;
mod contrast;
mod focus_order;
mod focus_visible;
mod headings;
mod info_relationships;
mod input_purpose;
mod instructions;
mod keyboard;
mod label_in_name;
mod labels;
mod language;
mod link_purpose;
mod non_text_contrast;
mod on_focus;
mod on_input;
mod page_titled;
mod resize_text;
mod section_headings;
mod text_alternatives;

pub use bypass_blocks::check_bypass_blocks;
pub use contrast::{Color, ContrastRule};
pub use focus_order::check_focus_order;
pub use focus_visible::check_focus_visible;
pub use headings::check_headings;
pub use info_relationships::check_info_relationships;
pub use input_purpose::check_input_purpose;
pub use instructions::check_instructions;
pub use keyboard::check_keyboard;
pub use label_in_name::check_label_in_name;
pub use labels::check_labels;
pub use language::check_language;
pub use link_purpose::check_link_purpose;
pub use non_text_contrast::check_non_text_contrast;
pub use on_focus::check_on_focus;
pub use on_input::check_on_input;
pub use page_titled::check_page_titled;
pub use resize_text::check_resize_text;
pub use section_headings::check_section_headings;
pub use text_alternatives::check_text_alternatives;
