//! WCAG conformance level, shared by the rules, the engine and the CLI.
//! It lives here, not in `cli`, so analysis code needs no import from the CLI layer.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// WCAG conformance levels
///
/// Variant order is significant: derives `PartialOrd`/`Ord` so a higher
/// configured level is `>=` every lower required level. This lets rule
/// catalogs filter with `cfg.wcag_level >= rule.min_level` instead of
/// pattern-matching the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WcagLevel {
    /// Level A - Minimum conformance
    #[value(name = "a", alias = "A")]
    A,
    /// Level AA - Recommended conformance (default)
    #[value(name = "aa", alias = "AA")]
    AA,
    /// Level AAA - Maximum conformance
    #[value(name = "aaa", alias = "AAA")]
    AAA,
}
