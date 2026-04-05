//! Journey Scoring Engine
//!
//! Saturation curves and weighted dimension aggregation for journey analysis.

/// Calculate a journey dimension score (100 minus capped penalties).
pub fn journey_dimension_score(penalties: &[f64], cap: f64) -> u32 {
    let sum: f64 = penalties.iter().sum();
    let capped = sum.min(cap);
    (100.0 - capped).max(0.0).round() as u32
}

/// Weighted average with intent-specific weights.
pub fn weighted_average_with_intent(items: &[(u32, f64)]) -> u32 {
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
    fn test_dimension_score_caps() {
        assert_eq!(journey_dimension_score(&[120.0], 100.0), 0);
        assert_eq!(journey_dimension_score(&[10.0, 20.0], 100.0), 70);
        assert_eq!(journey_dimension_score(&[], 100.0), 100);
    }

    #[test]
    fn test_weighted_average() {
        assert_eq!(weighted_average_with_intent(&[(80, 0.25), (60, 0.75)]), 65);
    }

    #[test]
    fn test_saturating_penalty_via_ux() {
        let p = crate::ux::saturating_penalty(0.0, 30.0, 5.0);
        assert!(p.abs() < 0.01);
    }
}
