//! Centralized report design tokens — the single source of truth for the
//! report's visual language.
//!
//! # The four-color law
//!
//! The report uses **exactly four status hues** plus a neutral scale. Every
//! colored element must resolve to one of these roles — no ad-hoc hex values in
//! section code:
//!
//! | Role        | Token                 | Meaning                          |
//! |-------------|-----------------------|----------------------------------|
//! | Green       | [`tokens::SUCCESS`]   | good / done / above target       |
//! | Blue        | [`tokens::INFO`]      | information / neutral accent      |
//! | Orange      | [`tokens::WARN_DEEP`] | watch / needs improvement         |
//! | Red         | [`tokens::DANGER`]    | problem / critical                |
//!
//! Score-driven and severity-driven colors must go through [`score_color`] and
//! [`severity_color`] so thresholds stay consistent across cover, dashboard,
//! cards, and module sections. Do not re-derive thresholds locally.

/// Hex color tokens used by the report design system.
pub mod tokens {
    // ── The four status hues ────────────────────────────────────────────
    /// Green — good / above-target. Also the primary brand accent.
    pub const SUCCESS: &str = "#0f766e";
    /// Blue — information / neutral accent (links, "what is measured" notes).
    pub const INFO: &str = "#2563eb";
    /// Orange — watch / needs improvement.
    pub const WARN_DEEP: &str = "#d97706";
    /// Red — problem / critical.
    pub const DANGER: &str = "#dc2626";

    // ── Neutral scale ───────────────────────────────────────────────────
    /// Strong ink for headings / dominant numbers.
    pub const INK: &str = "#0f172a";
    /// Secondary metadata / body de-emphasis.
    pub const NEUTRAL: &str = "#475569";
    /// Faint text / captions.
    pub const MUTED: &str = "#94a3b8";
    /// Hairline borders and dividers.
    pub const BORDER: &str = "#e2e8f0";
    #[allow(dead_code)]
    /// Subtle card / panel background.
    pub const SURFACE: &str = "#f8fafc";
}

/// Map a 0–100 score to its status hue, aligned with the report's grade bands
/// (`Gut`/`Sehr gut` ≥ 75 → green, `Verbesserungswürdig`/`Ausbaufähig` 40–74 →
/// orange, `Kritisch` < 40 → red). This is the only place score thresholds for
/// color live.
pub fn score_color(score: u8) -> &'static str {
    match score {
        75..=100 => tokens::SUCCESS,
        40..=74 => tokens::WARN_DEEP,
        _ => tokens::DANGER,
    }
}

#[allow(dead_code)]
/// Map a coarse status keyword (`"good"` / `"warn"` / `"bad"`) to its hue.
/// Used by checklist/diagnosis panels that carry a precomputed status.
pub fn status_color(status: &str) -> &'static str {
    match status {
        "good" => tokens::SUCCESS,
        "warn" => tokens::WARN_DEEP,
        "bad" => tokens::DANGER,
        _ => tokens::INFO,
    }
}

#[allow(dead_code)]
/// Map a WCAG severity to its hue. Critical/High are problems (red), Medium is
/// a watch state (orange), Low/everything else is informational (blue).
pub fn severity_color(severity: crate::wcag::Severity) -> &'static str {
    use crate::wcag::Severity::*;
    match severity {
        Critical | High => tokens::DANGER,
        Medium => tokens::WARN_DEEP,
        _ => tokens::INFO,
    }
}
