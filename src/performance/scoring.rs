//! Performance scoring and grading
//!
//! Calculates overall performance score and assigns grades.

use serde::{Deserialize, Serialize};

use super::vitals::WebVitals;

/// Performance grade levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceGrade {
    /// 90-100: Excellent performance
    Platinum,
    /// 75-89: Good performance
    Gold,
    /// 60-74: Average performance
    Silver,
    /// 50-59: Below average
    Bronze,
    /// <50: Needs significant improvement
    NeedsImprovement,
}

impl PerformanceGrade {
    pub fn from_score(score: u32) -> Self {
        match score {
            90..=100 => PerformanceGrade::Platinum,
            75..=89 => PerformanceGrade::Gold,
            60..=74 => PerformanceGrade::Silver,
            50..=59 => PerformanceGrade::Bronze,
            _ => PerformanceGrade::NeedsImprovement,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            PerformanceGrade::Platinum => "🏆",
            PerformanceGrade::Gold => "🥇",
            PerformanceGrade::Silver => "🥈",
            PerformanceGrade::Bronze => "🥉",
            PerformanceGrade::NeedsImprovement => "⚠️",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PerformanceGrade::Platinum => "PLATINUM",
            PerformanceGrade::Gold => "GOLD",
            PerformanceGrade::Silver => "SILVER",
            PerformanceGrade::Bronze => "BRONZE",
            PerformanceGrade::NeedsImprovement => "NEEDS IMPROVEMENT",
        }
    }
}

impl std::fmt::Display for PerformanceGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji(), self.label())
    }
}

/// Performance score with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceScore {
    /// Overall score (0-100)
    pub overall: u32,
    /// Grade based on score
    pub grade: PerformanceGrade,
    /// LCP score contribution (0-25)
    pub lcp_score: u32,
    /// FCP score contribution (0-25)
    pub fcp_score: u32,
    /// CLS score contribution (0-25)
    pub cls_score: u32,
    /// INP/TBT score contribution (0-25)
    pub interactivity_score: u32,
}

/// Calculate performance score from Web Vitals
///
/// Scoring weights:
/// - LCP: 25%
/// - FCP: 25%
/// - CLS: 25%
/// - INP/TBT: 25%
pub fn calculate_performance_score(vitals: &WebVitals) -> PerformanceScore {
    // Calculate individual scores (0-25 each)
    let lcp_score = vitals
        .lcp
        .as_ref()
        .map(|v| score_lcp(v.value))
        .unwrap_or(0);

    let fcp_score = vitals
        .fcp
        .as_ref()
        .map(|v| score_fcp(v.value))
        .unwrap_or(0);

    let cls_score = vitals
        .cls
        .as_ref()
        .map(|v| score_cls(v.value))
        .unwrap_or(0);

    let interactivity_score = vitals
        .inp
        .as_ref()
        .or(vitals.tbt.as_ref())
        .map(|v| score_interactivity(v.value))
        .unwrap_or(0);

    let overall = lcp_score + fcp_score + cls_score + interactivity_score;
    let grade = PerformanceGrade::from_score(overall);

    PerformanceScore {
        overall,
        grade,
        lcp_score,
        fcp_score,
        cls_score,
        interactivity_score,
    }
}

/// Score LCP (0-25)
/// Good: ≤2500ms, Poor: >4000ms
fn score_lcp(ms: f64) -> u32 {
    if ms <= 1200.0 {
        25
    } else if ms <= 2500.0 {
        // Linear interpolation 25->20
        25 - ((ms - 1200.0) / 260.0) as u32
    } else if ms <= 4000.0 {
        // Linear interpolation 20->10
        20 - ((ms - 2500.0) / 150.0) as u32
    } else if ms <= 6000.0 {
        // Linear interpolation 10->0
        10 - ((ms - 4000.0) / 200.0) as u32
    } else {
        0
    }
}

/// Score FCP (0-25)
/// Good: ≤1800ms, Poor: >3000ms
fn score_fcp(ms: f64) -> u32 {
    if ms <= 1000.0 {
        25
    } else if ms <= 1800.0 {
        25 - ((ms - 1000.0) / 160.0) as u32
    } else if ms <= 3000.0 {
        20 - ((ms - 1800.0) / 120.0) as u32
    } else if ms <= 5000.0 {
        10 - ((ms - 3000.0) / 200.0) as u32
    } else {
        0
    }
}

/// Score CLS (0-25)
/// Good: ≤0.1, Poor: >0.25
fn score_cls(value: f64) -> u32 {
    if value <= 0.05 {
        25
    } else if value <= 0.1 {
        25 - ((value - 0.05) * 100.0) as u32
    } else if value <= 0.25 {
        20 - ((value - 0.1) * 66.0) as u32
    } else if value <= 0.5 {
        10 - ((value - 0.25) * 40.0) as u32
    } else {
        0
    }
}

/// Score INP/TBT (0-25)
/// Good: ≤200ms, Poor: >500ms
fn score_interactivity(ms: f64) -> u32 {
    if ms <= 100.0 {
        25
    } else if ms <= 200.0 {
        25 - ((ms - 100.0) / 20.0) as u32
    } else if ms <= 500.0 {
        20 - ((ms - 200.0) / 30.0) as u32
    } else if ms <= 1000.0 {
        10 - ((ms - 500.0) / 50.0) as u32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::vitals::VitalMetric;

    #[test]
    fn test_performance_grade_from_score() {
        assert_eq!(PerformanceGrade::from_score(95), PerformanceGrade::Platinum);
        assert_eq!(PerformanceGrade::from_score(80), PerformanceGrade::Gold);
        assert_eq!(PerformanceGrade::from_score(65), PerformanceGrade::Silver);
        assert_eq!(PerformanceGrade::from_score(55), PerformanceGrade::Bronze);
        assert_eq!(PerformanceGrade::from_score(30), PerformanceGrade::NeedsImprovement);
    }

    #[test]
    fn test_score_lcp_good() {
        assert_eq!(score_lcp(1000.0), 25);
        assert!(score_lcp(2000.0) >= 20);
    }

    #[test]
    fn test_score_lcp_poor() {
        assert!(score_lcp(5000.0) < 10);
        assert_eq!(score_lcp(10000.0), 0);
    }

    #[test]
    fn test_calculate_performance_score() {
        let mut vitals = WebVitals::default();
        vitals.lcp = Some(VitalMetric::new(1500.0, 2500.0, 4000.0));
        vitals.fcp = Some(VitalMetric::new(1200.0, 1800.0, 3000.0));
        vitals.cls = Some(VitalMetric::new(0.05, 0.1, 0.25));
        vitals.tbt = Some(VitalMetric::new(150.0, 200.0, 600.0));

        let score = calculate_performance_score(&vitals);
        assert!(score.overall >= 80);
        assert!(matches!(score.grade, PerformanceGrade::Gold | PerformanceGrade::Platinum));
    }
}
