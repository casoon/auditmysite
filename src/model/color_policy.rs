//! Colour policy of the terminal output.

use clap::ValueEnum;

/// Terminal color policy for `--format table` and batch progress output (#529).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ColorPolicy {
    /// Color only for an interactive terminal, honouring `NO_COLOR` (default)
    #[default]
    #[value(name = "auto")]
    Auto,
    /// Always emit ANSI color
    #[value(name = "always")]
    Always,
    /// Never emit ANSI color
    #[value(name = "never")]
    Never,
}
