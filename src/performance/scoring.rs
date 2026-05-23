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
            PerformanceGrade::Platinum => "SEHR GUT",
            PerformanceGrade::Gold => "GUT",
            PerformanceGrade::Silver => "SOLIDE",
            PerformanceGrade::Bronze => "AUSBAUFÄHIG",
            PerformanceGrade::NeedsImprovement => "UNGENÜGEND",
        }
    }
}

impl std::fmt::Display for PerformanceGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji(), self.label())
    }
}

// Lighthouse v10/v11 weights — SI (Speed Index) is not measured here, so the
// remaining metrics are renormalized over their combined 90-point budget
// (10 + 25 + 30 + 25). When all four are present, `overall = sum * 100 / 90`.
const W_FCP: u32 = 10;
const W_LCP: u32 = 25;
const W_TBT: u32 = 30;
const W_CLS: u32 = 25;
const W_SI: u32 = 10;

/// Performance score with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceScore {
    /// Overall score (0-100), normalized to available metrics
    pub overall: u32,
    /// Grade based on score
    pub grade: PerformanceGrade,
    /// LCP score contribution (0-25); None = metric not measured
    pub lcp_score: Option<u32>,
    /// FCP score contribution (0-10); None = metric not measured
    pub fcp_score: Option<u32>,
    /// CLS score contribution (0-25); None = metric not measured
    pub cls_score: Option<u32>,
    /// TBT score contribution (0-30); None = metric not measured
    pub interactivity_score: Option<u32>,
    /// Speed Index score contribution (0-10); None = metric not measured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub si_score: Option<u32>,
    /// Number of metrics that were actually measured (0-5)
    pub metrics_available: u32,
}

/// Calculate performance score from Web Vitals.
///
/// Weights follow Lighthouse v10/v11 (FCP 10 %, LCP 25 %, TBT 30 %, CLS 25 %, SI 10 %).
/// The overall score is normalized to the metrics that were actually measured.
pub fn calculate_performance_score(vitals: &WebVitals) -> PerformanceScore {
    let lcp_score = vitals.lcp.as_ref().map(|v| score_lcp(v.value));
    let fcp_score = vitals.fcp.as_ref().map(|v| score_fcp(v.value));
    let cls_score = vitals.cls.as_ref().map(|v| score_cls(v.value));
    let interactivity_score = vitals.tbt.as_ref().map(|v| score_interactivity(v.value));
    let si_score = vitals.speed_index.as_ref().map(|v| score_si(v.value));

    let mut total = 0u32;
    let mut max_possible = 0u32;
    let mut metrics_available = 0u32;
    if let Some(s) = lcp_score {
        total += s;
        max_possible += W_LCP;
        metrics_available += 1;
    }
    if let Some(s) = fcp_score {
        total += s;
        max_possible += W_FCP;
        metrics_available += 1;
    }
    if let Some(s) = cls_score {
        total += s;
        max_possible += W_CLS;
        metrics_available += 1;
    }
    if let Some(s) = interactivity_score {
        total += s;
        max_possible += W_TBT;
        metrics_available += 1;
    }
    if let Some(s) = si_score {
        total += s;
        max_possible += W_SI;
        metrics_available += 1;
    }

    let overall = (total * 100)
        .checked_div(max_possible)
        .map(|n| n.min(100))
        .unwrap_or(0);
    let grade = PerformanceGrade::from_score(overall);

    PerformanceScore {
        overall,
        grade,
        lcp_score,
        fcp_score,
        cls_score,
        interactivity_score,
        si_score,
        metrics_available,
    }
}

/// Lighthouse-style log-normal score in [0, max_points].
///
/// Matches the Lighthouse v10/v11 formulation: the cumulative log-normal
/// distribution with median = p50 and a sigma calibrated so that the score
/// at p10 is exactly 0.9. score = 1 − Φ((ln(value) − ln(p50)) / σ).
fn log_normal_score(value: f64, p10: f64, p50: f64, max_points: u32) -> u32 {
    if value <= 0.0 {
        return max_points;
    }
    // Z-score for the 90th percentile of the standard normal (≈ 1.2816).
    const Z90: f64 = 1.2815515655446004;
    let p10_log = p10.ln();
    let p50_log = p50.ln();
    let value_log = value.ln();
    let sigma = (p50_log - p10_log).abs() / Z90;
    let z = (value_log - p50_log) / sigma;
    let s = (1.0 - standard_normal_cdf(z)).clamp(0.0, 1.0);
    (s * max_points as f64).round() as u32
}

/// Standard normal CDF Φ(x) = ½·(1 + erf(x/√2)).
fn standard_normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Abramowitz & Stegun 7.1.26 approximation of erf — accurate enough for
/// Lighthouse-style scoring, no external dependency required.
fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

/// Score LCP (0-25). Lighthouse curve: p10=2500 ms, p50=4000 ms.
fn score_lcp(ms: f64) -> u32 {
    log_normal_score(ms, 2500.0, 4000.0, W_LCP)
}

/// Score FCP (0-10). Lighthouse curve: p10=1800 ms, p50=3000 ms.
fn score_fcp(ms: f64) -> u32 {
    log_normal_score(ms, 1800.0, 3000.0, W_FCP)
}

/// Score CLS (0-25). Lighthouse curve: p10=0.1, p50=0.25.
/// CLS above 0.5 is capped to zero independent of the curve.
fn score_cls(value: f64) -> u32 {
    if value > 0.5 {
        return 0;
    }
    log_normal_score(value.max(0.0001), 0.1, 0.25, W_CLS)
}

/// Score TBT (0-30). Lighthouse curve: p10=200 ms, p50=600 ms.
fn score_interactivity(ms: f64) -> u32 {
    log_normal_score(ms.max(1.0), 200.0, 600.0, W_TBT)
}

/// Score Speed Index (0-10). Lighthouse curve: p10=3400 ms, p50=5800 ms.
fn score_si(ms: f64) -> u32 {
    log_normal_score(ms, 3400.0, 5800.0, W_SI)
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
        assert_eq!(
            PerformanceGrade::from_score(30),
            PerformanceGrade::NeedsImprovement
        );
    }

    #[test]
    fn test_score_lcp_good() {
        // p10 = 2500 ms → ~90 % of max → ~22-23 out of 25.
        assert!(score_lcp(2500.0) >= 22);
        assert_eq!(score_lcp(1000.0), 25);
    }

    #[test]
    fn test_score_lcp_poor() {
        // p50 = 4000 ms → ~50 % of max → ~12-13 out of 25.
        let s_p50 = score_lcp(4000.0);
        assert!((10..=14).contains(&s_p50), "p50 score was {}", s_p50);
        // Lighthouse-parity expectation: LCP > 6 s yields a tiny score (≤ 5/25).
        assert!(score_lcp(7000.0) <= 5);
        assert!(score_lcp(10000.0) <= 1);
    }

    #[test]
    fn test_score_cls_lighthouse_parity() {
        // p10 = 0.1 → ~90 % → ~22-23 / 25
        assert!(score_cls(0.1) >= 22);
        // p50 = 0.25 → ~50 % → ~12-13 / 25
        let s_p50 = score_cls(0.25);
        assert!((10..=14).contains(&s_p50));
        // Extreme CLS (> 0.5) must hard-cap to zero.
        assert_eq!(score_cls(1.7), 0);
    }

    #[test]
    fn test_overall_caps_when_lcp_is_extreme() {
        // amazon.de-like profile: LCP = 7432, CLS = 0.04, FCP ≈ 2400, TBT ≈ 800.
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(7432.0, 2500.0, 4000.0)),
            fcp: Some(VitalMetric::new(2400.0, 1800.0, 3000.0)),
            cls: Some(VitalMetric::new(0.04, 0.1, 0.25)),
            tbt: Some(VitalMetric::new(800.0, 200.0, 600.0)),
            ..Default::default()
        };
        let s = calculate_performance_score(&vitals);
        // Issue #248: a profile with LCP > 7 s should land well below 50,
        // not in the 70s as the old weighting allowed.
        assert!(s.overall < 50, "overall was {}", s.overall);
    }

    #[test]
    fn test_calculate_performance_score() {
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(1500.0, 2500.0, 4000.0)),
            fcp: Some(VitalMetric::new(1200.0, 1800.0, 3000.0)),
            cls: Some(VitalMetric::new(0.05, 0.1, 0.25)),
            tbt: Some(VitalMetric::new(150.0, 200.0, 600.0)),
            ..Default::default()
        };

        let score = calculate_performance_score(&vitals);
        assert!(score.overall >= 80);
        assert!(matches!(
            score.grade,
            PerformanceGrade::Gold | PerformanceGrade::Platinum
        ));
    }
}
