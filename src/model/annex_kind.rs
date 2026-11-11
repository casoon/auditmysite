//! Optional appendix sections of the PDF report.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Optional regulatory/editorial PDF appendix sections (opt-in, see `--annex`).
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AnnexKind {
    /// EN 301 549 (chapter 9, "Web") clause mapping annex.
    #[value(name = "en301549")]
    En301549,
    /// "BIK für Alle" editorial accessibility guide chapter mapping.
    #[value(name = "bik")]
    Bik,
}
