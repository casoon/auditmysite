//! Repeated lab runs of the headline vitals: median, spread and stability
//! (#1231).
//!
//! A single lab load varies from run to run. With `--perf-runs N` the page is
//! loaded N times; each rated metric is then the median of the runs that
//! measured it, carries its min/median/max as `spread`, and the LCP spread
//! decides whether the measurement counts as stable. Everything here is a
//! pure function of the runs' values.

use super::vitals::{
    speed_index_metric, VitalMetric, WebVitals, CLS_LIMITS, FCP_LIMITS, LCP_LIMITS, TBT_LIMITS,
    TTI_LIMITS,
};
use serde::{Deserialize, Serialize};

/// Min / median / max of one metric over the repeated runs of a profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MetricSpread {
    pub min: f64,
    pub median: f64,
    pub max: f64,
}

/// What repeated performance runs of one throttle profile measured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RunSpread {
    /// Runs that produced a measurement.
    pub runs: u32,
    /// Runs asked for (`--perf-runs`); fewer `runs` means some failed.
    pub requested_runs: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lcp_ms: Option<MetricSpread>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tbt_ms: Option<MetricSpread>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cls: Option<MetricSpread>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<MetricSpread>,
    /// FCP over the runs (#1231).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fcp_ms: Option<MetricSpread>,
    /// TTI over the runs (#1231); derived from the long-task list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tti_ms: Option<MetricSpread>,
    /// Whether the LCP of these runs agreed (#1231).
    #[serde(default)]
    pub stability: MeasurementStability,
}

/// Whether the repeated lab runs of a page agreed (#1231).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementStability {
    /// One measurement, or LCP in at most one run: no stability check.
    #[default]
    SingleRun,
    /// The LCP of the runs varied by at most 25 % of its median.
    Stable,
    /// The LCP of the runs varied by more than 25 % of its median.
    Unstable,
}

/// Largest relative LCP spread (`(max - min) / median`) that still counts as
/// stable (#1231).
const UNSTABLE_LCP_SPREAD: f64 = 0.25;

impl MeasurementStability {
    /// Judges the LCP spread of repeated runs. `None`, or a spread over a
    /// single value, cannot be checked.
    pub fn from_lcp(runs: u32, lcp: Option<&MetricSpread>) -> Self {
        let Some(lcp) = lcp.filter(|_| runs > 1) else {
            return Self::SingleRun;
        };
        if lcp.max <= lcp.min {
            return Self::Stable;
        }
        if lcp.median <= 0.0 || (lcp.max - lcp.min) / lcp.median > UNSTABLE_LCP_SPREAD {
            Self::Unstable
        } else {
            Self::Stable
        }
    }
}

/// Min / median / max of the finite values; the median of an even count is the
/// mean of the two middle values. `None` without a finite value.
pub(crate) fn spread_of(values: &[f64]) -> Option<MetricSpread> {
    let mut values: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    values.sort_by(f64::total_cmp);
    let n = values.len();
    (n > 0).then(|| MetricSpread {
        min: values[0],
        median: if n % 2 == 1 {
            values[n / 2]
        } else {
            (values[n / 2 - 1] + values[n / 2]) / 2.0
        },
        max: values[n - 1],
    })
}

/// One metric over the runs: its median value, rated against `limits`, with
/// the spread attached. The provenance tag is that of the first run that
/// measured it.
fn aggregate(
    runs: &[&WebVitals],
    pick: impl Fn(&WebVitals) -> Option<&VitalMetric>,
    limits: (f64, f64),
) -> Option<VitalMetric> {
    let metrics: Vec<&VitalMetric> = runs.iter().filter_map(|run| pick(run)).collect();
    let spread = spread_of(&metrics.iter().map(|m| m.value).collect::<Vec<_>>())?;
    let mut metric = VitalMetric::new(spread.median, limits.0, limits.1);
    metric.measurement = metrics.first()?.measurement;
    metric.spread = Some(spread);
    Some(metric)
}

/// The vitals of several runs of one page: `base` (a real run, for everything
/// that is not aggregated: TTFB, load times, DOM size, CLS attribution) with
/// LCP, FCP, CLS, TBT and TTI replaced by their medians over `runs`, each
/// carrying its spread. The Speed Index proxy follows from the median FCP and
/// LCP.
pub(crate) fn median_vitals(base: &WebVitals, runs: &[&WebVitals]) -> WebVitals {
    let mut out = base.clone();
    out.lcp = aggregate(runs, |v| v.lcp.as_ref(), LCP_LIMITS);
    out.fcp = aggregate(runs, |v| v.fcp.as_ref(), FCP_LIMITS);
    out.cls = aggregate(runs, |v| v.cls.as_ref(), CLS_LIMITS);
    out.tbt = aggregate(runs, |v| v.tbt.as_ref(), TBT_LIMITS);
    out.tti = aggregate(runs, |v| v.tti.as_ref(), TTI_LIMITS);
    out.speed_index = match (&out.fcp, &out.lcp) {
        (Some(fcp), Some(lcp)) => Some(speed_index_metric(fcp.value, lcp.value)),
        _ => None,
    };
    out
}

/// The spread of repeated runs: every aggregated metric plus the per-run
/// scores, and whether the LCP agreed. `requested` is what `--perf-runs` asked
/// for; fewer `runs` mean some loads failed.
pub(crate) fn run_spread(runs: &[&WebVitals], scores: &[f64], requested: u32) -> RunSpread {
    let spread = |pick: fn(&WebVitals) -> Option<&VitalMetric>| {
        spread_of(
            &runs
                .iter()
                .filter_map(|run| pick(run).map(|m| m.value))
                .collect::<Vec<_>>(),
        )
    };
    let lcp_ms = spread(|v| v.lcp.as_ref());
    let measured = runs.len() as u32;
    RunSpread {
        runs: measured,
        requested_runs: requested,
        stability: MeasurementStability::from_lcp(measured, lcp_ms.as_ref()),
        lcp_ms,
        tbt_ms: spread(|v| v.tbt.as_ref()),
        cls: spread(|v| v.cls.as_ref()),
        score: spread_of(scores),
        fcp_ms: spread(|v| v.fcp.as_ref()),
        tti_ms: spread(|v| v.tti.as_ref()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(lcp: f64, fcp: f64, cls: f64, tbt: f64, tti: f64) -> WebVitals {
        WebVitals {
            lcp: Some(VitalMetric::new(lcp, LCP_LIMITS.0, LCP_LIMITS.1)),
            fcp: Some(VitalMetric::new(fcp, FCP_LIMITS.0, FCP_LIMITS.1)),
            cls: Some(VitalMetric::new(cls, CLS_LIMITS.0, CLS_LIMITS.1)),
            tbt: Some(VitalMetric::new(tbt, TBT_LIMITS.0, TBT_LIMITS.1)),
            tti: Some(VitalMetric::new(tti, TTI_LIMITS.0, TTI_LIMITS.1).estimated()),
            ..Default::default()
        }
    }

    fn spread(min: f64, median: f64, max: f64) -> MetricSpread {
        MetricSpread { min, median, max }
    }

    #[test]
    fn spread_is_min_median_max_and_the_even_median_is_the_mean() {
        assert_eq!(spread_of(&[3.0, 1.0, 2.0]), Some(spread(1.0, 2.0, 3.0)));
        assert_eq!(spread_of(&[4.0, 1.0]), Some(spread(1.0, 2.5, 4.0)));
        assert_eq!(spread_of(&[7.0]), Some(spread(7.0, 7.0, 7.0)));
    }

    #[test]
    fn spread_ignores_values_that_are_not_finite_and_needs_one() {
        assert_eq!(
            spread_of(&[f64::NAN, 2.0, f64::INFINITY]),
            Some(spread(2.0, 2.0, 2.0))
        );
        assert_eq!(spread_of(&[]), None);
        assert_eq!(spread_of(&[f64::NAN]), None);
    }

    #[test]
    fn a_single_run_or_a_missing_lcp_cannot_be_checked() {
        let lcp = spread(1000.0, 2000.0, 9000.0);
        assert_eq!(
            MeasurementStability::from_lcp(1, Some(&lcp)),
            MeasurementStability::SingleRun
        );
        assert_eq!(
            MeasurementStability::from_lcp(3, None),
            MeasurementStability::SingleRun
        );
    }

    #[test]
    fn a_relative_lcp_spread_above_a_quarter_of_the_median_is_unstable() {
        let check =
            |min, median, max| MeasurementStability::from_lcp(3, Some(&spread(min, median, max)));
        // Exactly 25 % of the median stays stable.
        assert_eq!(check(1900.0, 2000.0, 2400.0), MeasurementStability::Stable);
        assert_eq!(
            check(1900.0, 2000.0, 2401.0),
            MeasurementStability::Unstable
        );
        assert_eq!(
            check(1000.0, 2000.0, 3500.0),
            MeasurementStability::Unstable
        );
        // Identical runs, also at 0 ms, agree.
        assert_eq!(check(2000.0, 2000.0, 2000.0), MeasurementStability::Stable);
        assert_eq!(check(0.0, 0.0, 0.0), MeasurementStability::Stable);
        // A spread around a median of 0 has no relative size.
        assert_eq!(check(0.0, 0.0, 10.0), MeasurementStability::Unstable);
    }

    #[test]
    fn every_rated_metric_becomes_the_median_of_its_runs_with_a_spread() {
        let runs = [
            run(6000.0, 2500.0, 0.30, 600.0, 9000.0),
            run(1500.0, 900.0, 0.02, 50.0, 2000.0),
            run(3500.0, 1700.0, 0.12, 200.0, 5000.0),
        ];
        let refs: Vec<&WebVitals> = runs.iter().collect();
        let median = median_vitals(&runs[0], &refs);

        let lcp = median.lcp.unwrap();
        assert_eq!(lcp.value, 3500.0);
        assert_eq!(lcp.spread, Some(spread(1500.0, 3500.0, 6000.0)));
        assert_eq!(lcp.rating, "needs-improvement", "rated on the median");
        assert_eq!(median.fcp.unwrap().value, 1700.0);
        assert_eq!(median.cls.unwrap().value, 0.12);
        assert_eq!(median.tbt.unwrap().value, 200.0);
        let tti = median.tti.unwrap();
        assert_eq!(tti.value, 5000.0);
        assert!(tti.is_estimated(), "TTI stays an estimate");
        // The proxy follows from the median FCP and LCP, not from one run.
        let si = median.speed_index.unwrap();
        assert!((si.value - (0.35 * 1700.0 + 0.65 * 3500.0)).abs() < 1e-9);
    }

    #[test]
    fn a_metric_counts_the_runs_that_measured_it() {
        let mut no_tbt = run(2000.0, 1000.0, 0.0, 0.0, 3000.0);
        no_tbt.tbt = None;
        let runs = [run(1000.0, 800.0, 0.0, 100.0, 2000.0), no_tbt];
        let refs: Vec<&WebVitals> = runs.iter().collect();
        let median = median_vitals(&runs[0], &refs);
        let tbt = median.tbt.unwrap();
        assert_eq!(tbt.value, 100.0);
        assert_eq!(tbt.spread, Some(spread(100.0, 100.0, 100.0)));
        assert_eq!(median.lcp.unwrap().value, 1500.0);
    }

    #[test]
    fn the_run_spread_counts_runs_and_judges_the_lcp() {
        let runs = [
            run(1000.0, 800.0, 0.0, 100.0, 2000.0),
            run(2000.0, 900.0, 0.0, 150.0, 2500.0),
        ];
        let refs: Vec<&WebVitals> = runs.iter().collect();
        let s = run_spread(&refs, &[90.0, 70.0], 3);
        assert_eq!((s.runs, s.requested_runs), (2, 3));
        assert_eq!(s.lcp_ms, Some(spread(1000.0, 1500.0, 2000.0)));
        assert_eq!(s.fcp_ms, Some(spread(800.0, 850.0, 900.0)));
        assert_eq!(s.tti_ms, Some(spread(2000.0, 2250.0, 2500.0)));
        assert_eq!(s.score, Some(spread(70.0, 80.0, 90.0)));
        assert_eq!(s.stability, MeasurementStability::Unstable);

        let one = run_spread(&refs[..1], &[90.0], 3);
        assert_eq!(one.stability, MeasurementStability::SingleRun);
    }
}
