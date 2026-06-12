//! UX Analysis Module
//!
//! Evaluates user experience quality through heuristic analysis of the
//! Accessibility Tree. Measures CTA clarity, visual hierarchy, content
//! clarity, trust signals, and cognitive load.

mod analysis;
pub mod module;
mod scoring;

pub use analysis::{
    analyze_ux, ux_dimension_name, ux_dimension_summary, ux_issue_text, UxAnalysis, UxDimension,
    UxDimensionKind, UxIssue, UxIssueKind, UxIssueValues,
};
pub use module::UxModule;
pub use scoring::saturating_penalty;
