//! Named score→label band families shared across the codebase.
//!
//! The SEO-specific family is migrated in a later, separately reviewed step
//! once its call sites move onto the registry.

/// A score threshold ladder with paired DE/EN labels.
///
/// `bands` must be ordered from highest `min_score` to lowest; the first
/// entry whose `min_score` the score meets or exceeds wins.
#[derive(Debug, Clone, Copy)]
pub struct BandSet {
    pub name: &'static str,
    pub bands: &'static [(i64, &'static str, &'static str)],
}

impl BandSet {
    pub fn label(&self, score: f32, en: bool) -> &'static str {
        let rounded = score.round() as i64;
        let entry = self
            .bands
            .iter()
            .find(|entry| rounded >= entry.0)
            .unwrap_or_else(|| self.bands.last().expect("BandSet must not be empty"));
        if en {
            entry.2
        } else {
            entry.1
        }
    }
}

/// The five-level word band used across most module/overall scores.
/// Thresholds match `crate::audit::interpretation::ScoreBand` and the label
/// prefixes documented in CLAUDE.md (Sehr gut/Gut/Verbesserungswürdig/
/// Ausbaufähig/Kritisch, EN Excellent/Good/Needs improvement/Inadequate/Critical).
pub static FIVE_BAND: BandSet = BandSet {
    name: "five_band_90_75_60_40",
    bands: &[
        (90, "Sehr gut", "Excellent"),
        (75, "Gut", "Good"),
        (60, "Verbesserungswürdig", "Needs improvement"),
        (40, "Ausbaufähig", "Inadequate"),
        (i64::MIN, "Kritisch", "Critical"),
    ],
};

/// The same 90/75/60/40 thresholds as [`FIVE_BAND`], rendered as a letter
/// grade instead of a word (used for module-level `grade` fields, e.g.
/// `taxonomy::module_score_grade`). Letters are language-neutral, so the
/// DE/EN label is identical.
pub static FIVE_BAND_LETTERS: BandSet = BandSet {
    name: "five_band_letters_90_75_60_40",
    bands: &[
        (90, "A", "A"),
        (75, "B", "B"),
        (60, "C", "C"),
        (40, "D", "D"),
        (i64::MIN, "F", "F"),
    ],
};

/// A distinct four-cutoff letter-grade family (90/80/70/60) used for
/// `summary.grade`, UX/Journey grades, and their a11y-adjusted variants.
/// Deliberately kept separate from [`FIVE_BAND_LETTERS`] — different cutoffs,
/// not a duplicate.
pub static LETTER_GRADE: BandSet = BandSet {
    name: "letter_grade_90_80_70_60",
    bands: &[
        (90, "A", "A"),
        (80, "B", "B"),
        (70, "C", "C"),
        (60, "D", "D"),
        (i64::MIN, "F", "F"),
    ],
};

/// Security's own five-cutoff letter grade (90/80/70/60/50, with an `A+` top
/// band). Deliberately kept separate from [`LETTER_GRADE`] — a distinct
/// family, not a duplicate.
pub static SECURITY_GRADE: BandSet = BandSet {
    name: "security_grade_90_80_70_60_50",
    bands: &[
        (90, "A+", "A+"),
        (80, "A", "A"),
        (70, "B", "B"),
        (60, "C", "C"),
        (50, "D", "D"),
        (i64::MIN, "F", "F"),
    ],
};

/// SEO's own four-cutoff band (90/70/55/35) — deliberately distinct from
/// [`FIVE_BAND`]. `seo::interpretation::classify_seo_score` is the sole
/// caller (it maps the resolved key back to its `SeoScoreBand` enum); this
/// entry exists so the thresholds are registered and visible rather than an
/// undocumented one-off `if`/`else` chain.
pub static SEO_BAND: BandSet = BandSet {
    name: "seo_band_90_70_55_35",
    bands: &[
        (90, "excellent", "excellent"),
        (70, "good", "good"),
        (55, "partial", "partial"),
        (35, "insufficient", "insufficient"),
        (i64::MIN, "critical", "critical"),
    ],
};

/// The 90/75/60/40 thresholds rendered as the canonical German certificate
/// token (used as a lookup key for badge/colour; the display label is
/// localized separately, see `cover::certificate_label_localized`). Shared by
/// `audit::scoring::AccessibilityScorer::calculate_certificate` (the
/// canonical source) and the PDF's `cover::batch_certificate_label`, which
/// re-implemented the same thresholds independently before this migration.
pub static CERTIFICATE: BandSet = BandSet {
    name: "certificate_90_75_60_40",
    bands: &[
        (90, "SEHR GUT", "SEHR GUT"),
        (75, "GUT", "GUT"),
        (60, "STABIL", "STABIL"),
        (40, "AUSBAUFÄHIG", "AUSBAUFÄHIG"),
        (i64::MIN, "UNGENÜGEND", "UNGENÜGEND"),
    ],
};

/// A distinct five-cutoff letter grade (95/90/80/70/60, `A+` at 95 not 90)
/// used only for the batch PDF cover's grade card. Deliberately kept separate
/// from [`LETTER_GRADE`] and [`SECURITY_GRADE`] — a third distinct family.
pub static BATCH_GRADE: BandSet = BandSet {
    name: "batch_grade_95_90_80_70_60",
    bands: &[
        (95, "A+", "A+"),
        (90, "A", "A"),
        (80, "B", "B"),
        (70, "C", "C"),
        (60, "D", "D"),
        (i64::MIN, "F", "F"),
    ],
};

/// The 90/75/60/40 thresholds rendered as a "technical condition" sentence
/// for the single-report PDF cover.
pub static COVER_PHRASE: BandSet = BandSet {
    name: "cover_phrase_90_75_60_40",
    bands: &[
        (
            90,
            "Sehr guter technischer Zustand",
            "Excellent technical condition",
        ),
        (75, "Guter technischer Zustand", "Good technical condition"),
        (
            60,
            "Verbesserungswürdiger Zustand",
            "Technical condition needs improvement",
        ),
        (
            40,
            "Ausbaufähiger technischer Zustand",
            "Inadequate technical condition",
        ),
        (
            i64::MIN,
            "Kritischer technischer Zustand",
            "Critical technical condition",
        ),
    ],
};

/// The 90/75/60/40 thresholds rendered as a "range" sentence for the
/// single-report PDF's desktop/mobile viewport score cards.
pub static SCORE_RANGE: BandSet = BandSet {
    name: "score_range_90_75_60_40",
    bands: &[
        (90, "Sehr guter Bereich", "Excellent range"),
        (75, "Guter Bereich", "Good range"),
        (60, "Verbesserungsbedarf", "Needs improvement"),
        (40, "Ausbaufähiger Bereich", "Inadequate range"),
        (i64::MIN, "Kritischer Bereich", "Critical range"),
    ],
};

/// A distinct three-cutoff medal band (90/80/60) used only for the
/// terminal-table summary (`output::summary::label_and_medal`). Labels are
/// English words regardless of locale, so DE/EN are identical.
pub static MEDAL: BandSet = BandSet {
    name: "medal_90_80_60",
    bands: &[
        (90, "GOLD", "GOLD"),
        (80, "SILVER", "SILVER"),
        (60, "BRONZE", "BRONZE"),
        (i64::MIN, "FAILED", "FAILED"),
    ],
};

/// A distinct four-cutoff band (90/80/70/50) used only to pick a terminal
/// bar color (`output::cli::bar_color`). Not a text label — both slots carry
/// the same internal color-key string, translated to a `colored` call at the
/// call site.
pub static BAR_COLOR_BAND: BandSet = BandSet {
    name: "bar_color_90_80_70_50",
    bands: &[
        (90, "green", "green"),
        (80, "light_green", "light_green"),
        (70, "yellow", "yellow"),
        (50, "orange", "orange"),
        (i64::MIN, "red", "red"),
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seo_band_matches_documented_thresholds() {
        assert_eq!(SEO_BAND.label(90.0, false), "excellent");
        assert_eq!(SEO_BAND.label(70.0, false), "good");
        assert_eq!(SEO_BAND.label(55.0, false), "partial");
        assert_eq!(SEO_BAND.label(35.0, false), "insufficient");
        assert_eq!(SEO_BAND.label(34.0, false), "critical");
    }

    #[test]
    fn five_band_matches_documented_thresholds() {
        assert_eq!(FIVE_BAND.label(95.0, false), "Sehr gut");
        assert_eq!(FIVE_BAND.label(95.0, true), "Excellent");
        assert_eq!(FIVE_BAND.label(75.0, false), "Gut");
        assert_eq!(FIVE_BAND.label(60.0, false), "Verbesserungswürdig");
        assert_eq!(FIVE_BAND.label(40.0, false), "Ausbaufähig");
        assert_eq!(FIVE_BAND.label(0.0, false), "Kritisch");
        assert_eq!(FIVE_BAND.label(0.0, true), "Critical");
    }

    #[test]
    fn five_band_letters_matches_module_score_grade_edges() {
        assert_eq!(FIVE_BAND_LETTERS.label(100.0, false), "A");
        assert_eq!(FIVE_BAND_LETTERS.label(90.0, false), "A");
        assert_eq!(FIVE_BAND_LETTERS.label(89.0, false), "B");
        assert_eq!(FIVE_BAND_LETTERS.label(75.0, false), "B");
        assert_eq!(FIVE_BAND_LETTERS.label(74.0, false), "C");
        assert_eq!(FIVE_BAND_LETTERS.label(60.0, false), "C");
        assert_eq!(FIVE_BAND_LETTERS.label(59.0, false), "D");
        assert_eq!(FIVE_BAND_LETTERS.label(40.0, false), "D");
        assert_eq!(FIVE_BAND_LETTERS.label(39.0, false), "F");
    }

    #[test]
    fn letter_grade_matches_documented_thresholds() {
        assert_eq!(LETTER_GRADE.label(90.0, false), "A");
        assert_eq!(LETTER_GRADE.label(80.0, false), "B");
        assert_eq!(LETTER_GRADE.label(70.0, false), "C");
        assert_eq!(LETTER_GRADE.label(60.0, false), "D");
        assert_eq!(LETTER_GRADE.label(59.0, false), "F");
    }

    #[test]
    fn security_grade_matches_documented_thresholds() {
        assert_eq!(SECURITY_GRADE.label(90.0, false), "A+");
        assert_eq!(SECURITY_GRADE.label(80.0, false), "A");
        assert_eq!(SECURITY_GRADE.label(70.0, false), "B");
        assert_eq!(SECURITY_GRADE.label(60.0, false), "C");
        assert_eq!(SECURITY_GRADE.label(50.0, false), "D");
        assert_eq!(SECURITY_GRADE.label(49.0, false), "F");
    }
}
