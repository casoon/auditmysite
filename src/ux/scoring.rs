//! UX Scoring Engine
//!
//! Saturation curves, group caps, and weighted dimension aggregation.

/// Saturating penalty: many occurrences converge towards max_penalty.
/// `max_penalty * (1 - e^(-count / pivot))`
pub fn saturating_penalty(count: f64, max_penalty: f64, pivot: f64) -> f64 {
    max_penalty * (1.0 - (-count / pivot).exp())
}

/// Calculate a dimension score (100 minus capped penalties).
pub fn dimension_score(penalties: &[f64], cap: f64) -> u32 {
    let sum: f64 = penalties.iter().sum();
    let capped = sum.min(cap);
    (100.0 - capped).max(0.0).round() as u32
}

/// Weighted average of dimension scores.
pub fn weighted_average(items: &[(u32, f64)]) -> u32 {
    let total_weight: f64 = items.iter().map(|(_, w)| w).sum();
    if total_weight == 0.0 {
        return 0;
    }
    let weighted_sum: f64 = items.iter().map(|(s, w)| *s as f64 * w).sum();
    (weighted_sum / total_weight).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_penalty_zero() {
        assert!((saturating_penalty(0.0, 30.0, 5.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_saturating_penalty_converges() {
        let p5 = saturating_penalty(5.0, 30.0, 5.0);
        let p50 = saturating_penalty(50.0, 30.0, 5.0);
        let p500 = saturating_penalty(500.0, 30.0, 5.0);
        assert!(p5 > 15.0 && p5 < 25.0); // ~18.96
        assert!(p50 > 29.0);
        assert!((p500 - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_dimension_score_caps() {
        assert_eq!(dimension_score(&[50.0, 60.0, 70.0], 100.0), 0);
        assert_eq!(dimension_score(&[10.0, 20.0], 100.0), 70);
        assert_eq!(dimension_score(&[], 100.0), 100);
    }

    #[test]
    fn test_weighted_average() {
        // 80 * 0.3 + 60 * 0.7 = 24 + 42 = 66
        assert_eq!(weighted_average(&[(80, 0.3), (60, 0.7)]), 66);
    }
}
