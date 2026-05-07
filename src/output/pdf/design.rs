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
    /// Lighter positive accent for "trend up" charts.
    pub const SUCCESS_BRIGHT: &str = "#22c55e";
    /// Mid-range amber — caution / attention without alarm.
    pub const WARN: &str = "#f59e0b";
    /// Slightly darker amber for solid-warning emphasis.
    pub const WARN_DEEP: &str = "#d97706";
    /// Critical / urgent state.
    pub const DANGER: &str = "#dc2626";
    /// Slightly lighter red for "trend down" charts.
    pub const DANGER_LIGHT: &str = "#ef4444";

    // Neutrals
    /// Subtle slate gray for secondary metadata.
    pub const NEUTRAL: &str = "#475569";

    // Accent tones
    /// Informational / link-style accent.
    pub const INFO: &str = "#2563eb";
    /// Branded purple for category accents.
    #[allow(dead_code)]
    pub const ACCENT_VIOLET: &str = "#7c3aed";
    /// Earthy bronze for date / certificate context.
    pub const ACCENT_BRONZE: &str = "#b45309";
}

/// Map a 0–100 score to its tier color (green / amber / red).
pub fn score_color(score: u32) -> &'static str {
    if score >= 70 {
        tokens::SUCCESS_BRIGHT
    } else if score >= 50 {
        tokens::WARN
    } else {
        tokens::DANGER_LIGHT
    }
}

/// Map a 0–100 module score to its tier color.
/// Slightly stricter thresholds than [`score_color`] for module dashboards.
pub fn module_score_color(score: u32) -> &'static str {
    if score >= 85 {
        tokens::SUCCESS
    } else if score >= 70 {
        tokens::INFO
    } else if score >= 50 {
        tokens::WARN_DEEP
    } else {
        tokens::DANGER
    }
}

/// Map a 0–100 score to a status keyword consumed by `MetricStrip`/`Callout`.
#[allow(dead_code)]
pub fn score_status(score: u32) -> &'static str {
    if score >= 85 {
        "good"
    } else if score >= 70 {
        "info"
    } else if score >= 50 {
        "warn"
    } else {
        "bad"
    }
}

/// Map a localized risk label (`Critical|High|Medium|Low` or
/// `Kritisch|Hoch|Mittel|Gering`) to a `MetricStrip` status keyword.
pub fn risk_status(label: &str) -> &'static str {
    match label {
        "Kritisch" | "Critical" => "bad",
        "Hoch" | "High" => "warn",
        "Mittel" | "Medium" => "info",
        _ => "good",
    }
}
