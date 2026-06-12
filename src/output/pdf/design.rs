//! Centralized report design tokens.
//!
//! Single source of truth for status colors, score-tier colors, and accent
//! colors used across PDF cover, metrics, callouts, and module sections. New
//! components should reference [`tokens`] rather than hard-coding hex values
//! so the report keeps a coherent visual language.

/// Hex color tokens used by the report design system.
pub mod tokens {
    // Score / status tiers — used for score cards, severity strips, callouts.
    /// Strong success / above-target — also used as primary brand accent.
    pub const SUCCESS: &str = "#0f766e";
    /// Slightly darker amber for solid-warning emphasis.
    pub const WARN_DEEP: &str = "#d97706";
    /// Critical / urgent state.
    pub const DANGER: &str = "#dc2626";

    // Neutrals
    /// Subtle slate gray for secondary metadata.
    pub const NEUTRAL: &str = "#475569";

    // Accent tones
    /// Informational / link-style accent.
    pub const INFO: &str = "#2563eb";
    /// Earthy bronze for date / certificate context.
    pub const ACCENT_BRONZE: &str = "#b45309";
}
