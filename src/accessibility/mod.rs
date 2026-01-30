//! Accessibility analysis module
//!
//! Provides AXTree extraction and accessibility-related utilities.

mod extractor;
mod styles;
mod tree;

pub use extractor::extract_ax_tree;
pub use styles::{extract_text_styles, ComputedStyles};
pub use tree::{AXNode, AXProperty, AXTree, AXValue, NameSource};
