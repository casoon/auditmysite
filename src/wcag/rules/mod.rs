//! WCAG Rules Module
//!
//! Contains individual WCAG rule implementations.

mod bypass_blocks;
mod contrast;
mod headings;
mod info_relationships;
mod instructions;
mod keyboard;
mod labels;
mod language;
mod link_purpose;
mod page_titled;
mod section_headings;
mod text_alternatives;

pub use bypass_blocks::check_bypass_blocks;
pub use contrast::{Color, ContrastRule};
pub use headings::check_headings;
pub use info_relationships::check_info_relationships;
pub use instructions::check_instructions;
pub use keyboard::check_keyboard;
pub use labels::check_labels;
pub use language::check_language;
pub use link_purpose::check_link_purpose;
pub use page_titled::check_page_titled;
pub use section_headings::check_section_headings;
pub use text_alternatives::check_text_alternatives;
