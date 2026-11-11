//! Page flow — user-flow heuristic over the AXTree (formerly `journey`).
//!
//! Evaluates how well a page supports a visitor's way through it: entry
//! clarity, orientation, navigation, interaction, conversion. Includes
//! page-intent detection (`PageIntent`) for context-aware weighting and the
//! friction points the heuristic reports.
//!
//! This is a static analysis of the AXTree and has nothing to do with the
//! interactive accessibility journeys, which drive the browser: those live in
//! [`crate::a11y_journey`]. The module was named `journey` until #938, which
//! made the two look related. The public names stay: the CLI indicator
//! `--experimental journey`, the JSON key `journey`, the `Journey*` types and
//! the `journey.*` rule ids are user-visible contracts; only the Rust module
//! path is `page_flow`.

mod analysis;
pub mod module;
mod page_intent;
mod scoring;

pub use analysis::{
    analyze_journey, analyze_journey_with_dom_check, analyze_journey_with_page_context,
    journey_dimension_name, journey_dimension_summary, journey_friction_text, journey_step_name,
    FrictionKind, FrictionPoint, FrictionValues, JourneyAnalysis, JourneyDimension,
    JourneyDimensionKind,
};
pub use module::JourneyModule;
pub use page_intent::{detect_page_intent, PageIntent};
pub use scoring::journey_dimension_score;
