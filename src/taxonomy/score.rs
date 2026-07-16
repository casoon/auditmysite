//! Score-Taxonomie
//!
//! Definiert Score-Impact-Modell, Gewichtung und Labels.

use super::Dimension;
use serde::{Deserialize, Serialize};

/// Wie skaliert der Score-Abzug bei Mehrfachvorkommen einer Regel?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scaling {
    /// Jedes Auftreten gleicher Abzug (bis max_penalty)
    Linear,
    /// Abnehmender Abzug bei Mehrfachvorkommen (log2)
    Logarithmic,
    /// Einmaliger Abzug, unabhängig von der Anzahl
    Fixed,
}

/// Score-Impact-Definition pro Regel
#[derive(Debug, Clone, Copy)]
pub struct ScoreImpact {
    /// Grundabzug pro Auftreten (oder einmalig bei Fixed)
    pub base_penalty: f32,
    /// Maximalabzug für diese Regel insgesamt
    pub max_penalty: f32,
    /// Skalierungsmodus
    pub occurrence_scaling: Scaling,
}

/// Sättigungsschwelle: ab so vielen Vorkommen greift Phase-2-Scaling.
pub const SATURATION_THRESHOLD: usize = 10;

/// Phase-2-Uplift pro zusätzlichem Vorkommen, als Anteil des `base_penalty`.
///
/// Niedrig genug gewählt, dass auch hohe Occurrence-Counts (50, 200) unterhalb
/// des `2 × max_penalty`-Caps strikt wachsende Penalties ergeben statt
/// gemeinsam zu saturieren.
const PHASE2_RATE: f32 = 0.05;

impl ScoreImpact {
    /// Berechne den tatsächlichen Abzug für n Vorkommen.
    ///
    /// Logarithmische Regeln nutzen Zwei-Phasen-Scaling: bis zur
    /// `SATURATION_THRESHOLD` logarithmisch (gedeckelt auf `max_penalty`),
    /// darüber ein langsamer linearer Uplift. So unterscheidet sich eine
    /// Seite mit 200 fehlenden Alt-Texten von einer mit 10 — ohne dass
    /// Ergebnisse für 1–10 Vorkommen sich ändern.
    pub fn calculate_penalty(&self, occurrences: usize) -> f32 {
        if occurrences == 0 {
            return 0.0;
        }
        match self.occurrence_scaling {
            Scaling::Fixed => self.base_penalty.min(self.max_penalty),
            Scaling::Linear => (occurrences as f32 * self.base_penalty).min(self.max_penalty),
            Scaling::Logarithmic => {
                let phase1_count = occurrences.min(SATURATION_THRESHOLD);
                let phase2_count = occurrences.saturating_sub(SATURATION_THRESHOLD);
                let phase1 =
                    (self.base_penalty * (1.0 + (phase1_count as f32).ln())).min(self.max_penalty);
                let phase2 = phase2_count as f32 * self.base_penalty * PHASE2_RATE;
                (phase1 + phase2).min(self.max_penalty * 2.0)
            }
        }
    }
}

/// Gewichtung der Module für den Gesamtscore
pub const MODULE_WEIGHTS: &[(Dimension, f32)] = &[
    (Dimension::Accessibility, 35.0),
    (Dimension::Performance, 20.0),
    (Dimension::Seo, 20.0),
    (Dimension::Security, 15.0),
    (Dimension::Mobile, 10.0),
];

/// Gewicht für eine Dimension nachschlagen
pub fn weight_for(dimension: Dimension) -> f32 {
    MODULE_WEIGHTS
        .iter()
        .find(|(d, _)| *d == dimension)
        .map(|(_, w)| *w)
        .unwrap_or(0.0)
}

/// Score-Label (deutsch) für Endnutzer
pub fn score_label(score: u32) -> &'static str {
    match score {
        90..=100 => "Sehr gut",
        75..=89 => "Gut",
        60..=74 => "Verbesserungswürdig",
        40..=59 => "Ausbaufähig",
        _ => "Kritisch",
    }
}

/// Grade-Buchstabe aus Score
pub fn score_grade(score: u32) -> &'static str {
    crate::registry::LETTER_GRADE.label(score as f32, false)
}

/// Letter grade for module-level quality indicators.
pub fn module_score_grade(score: u32) -> &'static str {
    crate::registry::FIVE_BAND_LETTERS.label(score as f32, false)
}

/// Status-Farbe für Score
pub fn score_status(score: u32) -> &'static str {
    match score {
        80..=100 => "good",
        60..=79 => "warning",
        _ => "bad",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_penalty() {
        let impact = ScoreImpact {
            base_penalty: 10.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Fixed,
        };
        assert_eq!(impact.calculate_penalty(1), 10.0);
        assert_eq!(impact.calculate_penalty(5), 10.0);
    }

    #[test]
    fn test_linear_penalty_capped() {
        let impact = ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Linear,
        };
        assert_eq!(impact.calculate_penalty(1), 3.0);
        assert_eq!(impact.calculate_penalty(3), 9.0);
        assert_eq!(impact.calculate_penalty(5), 10.0); // capped
    }

    #[test]
    fn test_logarithmic_penalty() {
        let impact = ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 15.0,
            occurrence_scaling: Scaling::Logarithmic,
        };
        let p1 = impact.calculate_penalty(1);
        let p3 = impact.calculate_penalty(3);
        let p10 = impact.calculate_penalty(10);
        // Logarithmic: abnehmender Zuwachs
        assert!(p1 < p3);
        assert!(p3 < p10);
        assert!(p10 <= 15.0);
    }

    #[test]
    fn test_depth_saturation_phase2() {
        // Rule 1.1.1: base=3.0, max=10.0, Logarithmic
        let impact = ScoreImpact {
            base_penalty: 3.0,
            max_penalty: 10.0,
            occurrence_scaling: Scaling::Logarithmic,
        };
        let p10 = impact.calculate_penalty(10);
        let p50 = impact.calculate_penalty(50);
        let p200 = impact.calculate_penalty(200);

        // Phase-2-Uplift: höhere Occurrence-Counts ergeben strikt höhere Penalty
        assert!(p50 > p10, "p50={} should exceed p10={}", p50, p10);
        assert!(p200 > p50, "p200={} should exceed p50={}", p200, p50);
        // Erweiterter Cap: nie über 2 × max_penalty
        assert!(p200 <= 20.0, "p200={} should not exceed 2×max", p200);
        // Vorkommen 1–10 unverändert (Phase 2 inaktiv)
        assert!((impact.calculate_penalty(2) - 3.0 * (1.0 + 2.0_f32.ln())).abs() < 0.01);
    }

    #[test]
    fn test_score_labels() {
        assert_eq!(score_label(95), "Sehr gut");
        assert_eq!(score_label(80), "Gut");
        assert_eq!(score_label(65), "Verbesserungswürdig");
        assert_eq!(score_label(45), "Ausbaufähig");
        assert_eq!(score_label(20), "Kritisch");
    }

    #[test]
    fn test_module_score_grade_edges() {
        assert_eq!(module_score_grade(100), "A");
        assert_eq!(module_score_grade(90), "A");
        assert_eq!(module_score_grade(89), "B");
        assert_eq!(module_score_grade(75), "B");
        assert_eq!(module_score_grade(74), "C");
        assert_eq!(module_score_grade(60), "C");
        assert_eq!(module_score_grade(59), "D");
        assert_eq!(module_score_grade(40), "D");
        assert_eq!(module_score_grade(39), "F");
    }

    #[test]
    fn test_module_weights_sum() {
        let total: f32 = MODULE_WEIGHTS.iter().map(|(_, w)| w).sum();
        assert_eq!(total, 100.0);
    }
}
