//! Shared value types named by the CLI, the audit pipeline, the analysis
//! modules and the output layer (#934).
//!
//! They live here, below every other module, so none of those layers has to
//! import from `cli` to name them. `model` imports nothing else from this crate.

mod annex_kind;
mod color_policy;
mod finding_origin;
mod interactive_mode;
mod report_level;
mod wcag_level;

pub use annex_kind::AnnexKind;
pub use color_policy::ColorPolicy;
pub use finding_origin::{frame_origin, FindingOrigin, FrameOrigin};
pub use interactive_mode::InteractiveMode;
pub use report_level::ReportLevel;
pub use wcag_level::WcagLevel;
