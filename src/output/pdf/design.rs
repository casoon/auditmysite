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
}

/// Map a 0–100 score to its status hue, aligned with the report's grade bands
/// (`Gut`/`Sehr gut` ≥ 75 → green, `Verbesserungswürdig`/`Ausbaufähig` 40–74 →
/// orange, `Kritisch` < 40 → red). Not the only score-colour mapping: the
/// module quality bands in `pdf/helpers.rs` (`score_quality_color`,
/// 70/50) use their own thresholds.
pub fn score_color(score: u8) -> &'static str {
    match score {
        75..=100 => tokens::SUCCESS,
        40..=74 => tokens::WARN_DEEP,
        _ => tokens::DANGER,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wcag::Severity;

    #[test]
    fn severity_color_low_is_info_blue_not_gray() {
        // Regression: findings.rs used to re-derive this mapping independently
        // and rendered Low as NEUTRAL (gray) instead of the documented blue
        // "informational" hue.
        assert_eq!(severity_color(Severity::Low), tokens::INFO);
    }

    #[test]
    fn severity_color_matches_documented_four_color_law() {
        assert_eq!(severity_color(Severity::Critical), tokens::DANGER);
        assert_eq!(severity_color(Severity::High), tokens::DANGER);
        assert_eq!(severity_color(Severity::Medium), tokens::WARN_DEEP);
        assert_eq!(severity_color(Severity::Low), tokens::INFO);
    }
}
