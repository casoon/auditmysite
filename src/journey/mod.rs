//! Journey Module — User-flow analysis from AXTree data
//!
//! Evaluates how well a page supports a user's journey through
//! entry clarity, orientation, navigation, interaction, and conversion.
//! Includes page-intent detection for context-aware weighting.

mod analysis;
mod page_intent;
mod scoring;

pub use analysis::{
    analyze_journey, analyze_journey_with_dom_check, FrictionPoint, JourneyAnalysis,
    JourneyDimension,
};
pub use page_intent::{detect_page_intent, PageIntent};
pub use scoring::journey_dimension_score;
