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

impl ScoreImpact {
    /// Berechne den tatsächlichen Abzug für n Vorkommen
    pub fn calculate_penalty(&self, occurrences: usize) -> f32 {
        if occurrences == 0 {
            return 0.0;
        }
        let raw = match self.occurrence_scaling {
            Scaling::Fixed => self.base_penalty,
            Scaling::Linear => occurrences as f32 * self.base_penalty,
            Scaling::Logarithmic => {
                self.base_penalty * (1.0 + (occurrences as f32).ln())
            }
        };
        raw.min(self.max_penalty)
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
        60..=74 => "Befriedigend",
        40..=59 => "Ausbaufähig",
        _ => "Kritisch",
    }
}

/// Grade-Buchstabe aus Score
pub fn score_grade(score: u32) -> &'static str {
    match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
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
    fn test_score_labels() {
        assert_eq!(score_label(95), "Sehr gut");
        assert_eq!(score_label(80), "Gut");
        assert_eq!(score_label(65), "Befriedigend");
        assert_eq!(score_label(45), "Ausbaufähig");
        assert_eq!(score_label(20), "Kritisch");
    }

    #[test]
    fn test_module_weights_sum() {
        let total: f32 = MODULE_WEIGHTS.iter().map(|(_, w)| w).sum();
        assert_eq!(total, 100.0);
    }
}
