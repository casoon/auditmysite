//! Accessibility analysis module
//!
//! Erhebung im Browser: AXTree über CDP einsammeln, Fokus und Stile dazu, und
//! die Aufnahme daraus bauen. Baum, Aufnahme, Differenz und Lesereihenfolge
//! selbst sind **Berechnung** und liegen im Crate
//! [`a11y_perception`](https://crates.io/crates/a11y-perception); hier stehen
//! nur noch die Wege, auf denen die Daten hereinkommen.

pub mod accname_diff;
pub(crate) mod code_gen;
pub mod dom_document;
mod element_capture;
mod enrichment;
mod extractor;
pub(crate) mod js_helpers;
pub mod snapshot;
mod styles;

pub use a11y_perception::{AXNode, AXProperty, AXTree, AXValue, NameSource, RelatedNode};
pub use a11y_perception::{AXSnapshot, FocusIndicatorStatus, FocusSnapshot, Rect};
pub use a11y_perception::{AXTreeDiff, FocusMove, PropertyChange};
pub use accname_diff::{compare as compare_accname, AccnameDiff, DEFAULT_MAX_SAMPLES};
pub use dom_document::{build_document, fetch_dom_document, CdpDocument};
pub use element_capture::{capture_element_evidence, ElementEvidenceBudget, MAX_ELEMENT_CROPS};
pub use enrichment::enrich_violations_with_page;
pub use extractor::extract_ax_tree;
pub use snapshot::capture as capture_snapshot;
pub use styles::{extract_text_styles, ComputedStyles};
