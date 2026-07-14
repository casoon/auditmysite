//! Accessibility analysis module
//!
//! Provides AXTree extraction and accessibility-related utilities.

pub(crate) mod code_gen;
pub mod diff;
mod element_capture;
mod enrichment;
mod extractor;
pub(crate) mod js_helpers;
pub mod snapshot;
mod styles;
mod tree;

pub use diff::{AXTreeDiff, FocusMove, PropertyChange};
pub use element_capture::{capture_element_evidence, ElementEvidenceBudget, MAX_ELEMENT_CROPS};
pub use enrichment::enrich_violations_with_page;
pub use extractor::extract_ax_tree;
pub use snapshot::{AXSnapshot, FocusIndicatorStatus, FocusSnapshot, Rect};
pub use styles::{extract_text_styles, ComputedStyles};
pub use tree::{AXNode, AXProperty, AXTree, AXValue, NameSource, RelatedNode};
