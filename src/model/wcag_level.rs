//! WCAG conformance level, shared by the rules, the engine and the CLI.

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

impl std::fmt::Display for WcagLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WcagLevel::A => write!(f, "A"),
            WcagLevel::AA => write!(f, "AA"),
            WcagLevel::AAA => write!(f, "AAA"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_level_display() {
        assert_eq!(WcagLevel::A.to_string(), "A");
        assert_eq!(WcagLevel::AA.to_string(), "AA");
        assert_eq!(WcagLevel::AAA.to_string(), "AAA");
    }
}
