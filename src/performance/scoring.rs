//! Performance scoring and grading
//!
//! Calculates overall performance score and assigns grades.

use serde::{Deserialize, Serialize};

use super::content_weight::ContentWeight;
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
            PerformanceGrade::Silver => "STABIL",
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
    /// Penalty deducted for page size (transfer/decoded bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_penalty: Option<u32>,
    /// Penalty deducted for JavaScript size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub js_penalty: Option<u32>,
    /// Penalty deducted for request count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_penalty: Option<u32>,
    /// Penalty deducted for DOM complexity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dom_penalty: Option<u32>,
    /// Whether the score was capped due to a critical threshold
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_capped: Option<bool>,
}

/// Calculate performance score from Web Vitals and Content Weight.
///
/// Weights follow Lighthouse v10/v11 (FCP 10 %, LCP 25 %, TBT 30 %, CLS 25 %, SI 10 %).
/// The overall score is normalized to the metrics that were actually measured.
/// Then deductions and caps are applied based on content size, request count, and DOM complexity.
pub fn calculate_performance_score(
    vitals: &WebVitals,
    content_weight: Option<&ContentWeight>,
) -> PerformanceScore {
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

    let base_overall = (total * 100)
        .checked_div(max_possible)
        .map(|n| n.min(100))
        .unwrap_or(0);

    let mut overall = base_overall;
    let mut size_penalty = None;
    let mut js_penalty = None;
    let mut request_penalty = None;
    let mut dom_penalty = None;
    let mut is_capped = None;

    let mut size_cap = 100u32;
    let mut js_cap = 100u32;
    let mut req_cap = 100u32;
    let mut dom_cap = 100u32;

    if let Some(cw) = content_weight {
        // Page Size (Total Bytes): > 2MB starts linear penalty (5 points per MB, max 30)
        let total_mb = cw.total_bytes as f64 / 1_000_000.0;
        if total_mb > 2.0 {
            let penalty = ((total_mb - 2.0) * 5.0).min(30.0).round() as u32;
            size_penalty = Some(penalty);
            overall = overall.saturating_sub(penalty);
        }

        // JS Size (Decoded): > 1MB starts linear penalty (5 points per 500KB, max 20)
        let js_mb = cw.breakdown.javascript.bytes as f64 / 1_000_000.0;
        if js_mb > 1.0 {
            let penalty = (((js_mb - 1.0) / 0.5) * 5.0).min(20.0).round() as u32;
            js_penalty = Some(penalty);
            overall = overall.saturating_sub(penalty);
        }

        // Request Count: > 60 requests starts linear penalty (2 points per 10 requests, max 15)
        if cw.request_count > 60 {
            let penalty =
                ((((cw.request_count - 60) as f64 / 10.0).round() * 2.0).min(15.0)) as u32;
            request_penalty = Some(penalty);
            overall = overall.saturating_sub(penalty);
        }

        // Caps
        // Page Size caps: > 10MB -> max 39, > 5MB -> max 59, > 3MB -> max 74
        if cw.total_bytes > 10_000_000 {
            size_cap = 39;
        } else if cw.total_bytes > 5_000_000 {
            size_cap = 59;
        } else if cw.total_bytes > 3_000_000 {
            size_cap = 74;
        }

        // JS caps: > 3MB -> max 59, > 1.5MB -> max 74
        if cw.breakdown.javascript.bytes > 3_000_000 {
            js_cap = 59;
        } else if cw.breakdown.javascript.bytes > 1_500_000 {
            js_cap = 74;
        }

        // Request count cap: > 120 -> max 74
        if cw.request_count > 120 {
            req_cap = 74;
        }
    }

    // DOM Nodes (WebVitals): > 1000 nodes starts linear penalty (2 points per 100 nodes, max 15)
    if let Some(nodes) = vitals.dom_nodes {
        if nodes > 1000 {
            let penalty = ((((nodes - 1000) as f64 / 100.0).round() * 2.0).min(15.0)) as u32;
            dom_penalty = Some(penalty);
            overall = overall.saturating_sub(penalty);
        }

        // DOM nodes caps: > 3000 -> max 59, > 2000 -> max 74
        if nodes > 3000 {
            dom_cap = 59;
        } else if nodes > 2000 {
            dom_cap = 74;
        }
    }

    // Apply caps
    let min_cap = size_cap.min(js_cap).min(req_cap).min(dom_cap);
    if overall > min_cap {
        overall = min_cap;
        is_capped = Some(true);
    }

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
        size_penalty,
        js_penalty,
        request_penalty,
        dom_penalty,
        is_capped,
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
    use crate::performance::ResourceBreakdown;

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
        let s = calculate_performance_score(&vitals, None);
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

        let score = calculate_performance_score(&vitals, None);
        assert!(score.overall >= 80);
        assert!(matches!(
            score.grade,
            PerformanceGrade::Gold | PerformanceGrade::Platinum
        ));
    }

    #[test]
    fn test_partial_vitals_scores_on_available_metrics_only() {
        // Only LCP and CLS present; FCP/TBT/SI absent.
        // Score must be computed against the available weight budget (W_LCP + W_CLS = 50),
        // not the full 100-point budget — otherwise missing vitals would silently deflate scores.
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(1000.0, 2500.0, 4000.0)), // → score_lcp = 25 (max)
            cls: Some(VitalMetric::new(0.0, 0.1, 0.25)),         // → score_cls = 25 (max)
            ..Default::default()
        };
        let score = calculate_performance_score(&vitals, None);
        assert_eq!(score.overall, 100, "all available metrics perfect → 100");
        assert_eq!(score.metrics_available, 2);
    }

    #[test]
    fn test_no_vitals_returns_zero() {
        let vitals = WebVitals::default();
        let score = calculate_performance_score(&vitals, None);
        assert_eq!(score.overall, 0, "no vitals available → 0");
        assert_eq!(score.metrics_available, 0);
    }

    #[test]
    fn test_page_weight_capping_and_penalties() {
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(1000.0, 2500.0, 4000.0)),
            fcp: Some(VitalMetric::new(800.0, 1800.0, 3000.0)),
            cls: Some(VitalMetric::new(0.01, 0.1, 0.25)),
            tbt: Some(VitalMetric::new(50.0, 200.0, 600.0)),
            ..Default::default()
        };

        // 1. Size = 2.5 MB (no cap, minor penalty)
        // size_penalty: (2.5 - 2.0) * 5 = 2.5 -> round to 3
        let cw_light = ContentWeight {
            total_bytes: 2_500_000,
            transfer_bytes: 1_800_000,
            breakdown: ResourceBreakdown::default(),
            request_count: 30,
            carbon: crate::performance::CarbonEstimate::default(),
            recommendations: vec![],
        };
        let s_light = calculate_performance_score(&vitals, Some(&cw_light));
        assert_eq!(s_light.size_penalty, Some(3));
        assert_eq!(s_light.overall, 97);
        assert_eq!(s_light.is_capped, None);

        // 2. Size = 4.5 MB (cap 74)
        let cw_medium = ContentWeight {
            total_bytes: 4_500_000,
            transfer_bytes: 3_000_000,
            breakdown: ResourceBreakdown::default(),
            request_count: 50,
            carbon: crate::performance::CarbonEstimate::default(),
            recommendations: vec![],
        };
        let s_medium = calculate_performance_score(&vitals, Some(&cw_medium));
        assert_eq!(s_medium.overall, 74);
        assert_eq!(s_medium.is_capped, Some(true));

        // 3. Size = 12 MB (cap 39 - "geht gar nicht")
        let cw_heavy = ContentWeight {
            total_bytes: 12_000_000,
            transfer_bytes: 9_000_000,
            breakdown: ResourceBreakdown::default(),
            request_count: 80,
            carbon: crate::performance::CarbonEstimate::default(),
            recommendations: vec![],
        };
        let s_heavy = calculate_performance_score(&vitals, Some(&cw_heavy));
        assert_eq!(s_heavy.overall, 39);
        assert_eq!(s_heavy.is_capped, Some(true));
    }

    #[test]
    fn test_js_weight_capping_and_penalties() {
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(1000.0, 2500.0, 4000.0)),
            fcp: Some(VitalMetric::new(800.0, 1800.0, 3000.0)),
            cls: Some(VitalMetric::new(0.01, 0.1, 0.25)),
            tbt: Some(VitalMetric::new(50.0, 200.0, 600.0)),
            ..Default::default()
        };

        // JS size = 1.6 MB decoded -> capped at 74
        let mut breakdown = ResourceBreakdown::default();
        breakdown.javascript.bytes = 1_600_000;
        let cw = ContentWeight {
            total_bytes: 2_500_000,
            transfer_bytes: 1_800_000,
            breakdown,
            request_count: 30,
            carbon: crate::performance::CarbonEstimate::default(),
            recommendations: vec![],
        };

        let s = calculate_performance_score(&vitals, Some(&cw));
        assert_eq!(s.overall, 74);
        assert_eq!(s.is_capped, Some(true));
        assert_eq!(s.js_penalty, Some(6)); // (1.6 - 1.0)/0.5 * 5 = 6
    }

    #[test]
    fn test_dom_nodes_capping_and_penalties() {
        // DOM nodes = 2500 -> capped at 74, penalty applied
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(1000.0, 2500.0, 4000.0)),
            fcp: Some(VitalMetric::new(800.0, 1800.0, 3000.0)),
            cls: Some(VitalMetric::new(0.01, 0.1, 0.25)),
            tbt: Some(VitalMetric::new(50.0, 200.0, 600.0)),
            dom_nodes: Some(2500),
            ..Default::default()
        };

        let s = calculate_performance_score(&vitals, None);
        assert_eq!(s.overall, 74);
        assert_eq!(s.is_capped, Some(true));
        assert_eq!(s.dom_penalty, Some(15)); // (2500-1000)/100 * 2 = 30 -> max 15
    }
}
