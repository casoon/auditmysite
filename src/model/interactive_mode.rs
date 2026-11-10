//! Depth of the interactive accessibility-journey phase.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Depth of the Accessibility-Journey-Layer.
#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InteractiveMode {
    /// No interactive phase.
    #[value(name = "off")]
    Off,
    /// Tab walk, skip link, obvious pattern journeys.
    #[value(name = "basic")]
    Basic,
    /// Adds SPA navigation, form-error announcement, link/outline export.
    #[default]
    #[value(name = "full")]
    Full,
}

impl InteractiveMode {
    /// The mode named in `audit_scope.interactive_mode`; `None` for a name
    /// this version does not know.
    pub fn from_scope_name(name: &str) -> Option<Self> {
        match name {
            "off" => Some(Self::Off),
            "basic" => Some(Self::Basic),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    pub fn is_enabled(self) -> bool {
        !matches!(self, InteractiveMode::Off)
    }
}
