//! Throttled performance passes (single-URL only): one page load per throttle
//! profile after the two viewport passes, recovery of a tab a failed pass left
//! busy, and adoption of the LhMobile measurement as the canonical score.

use chromiumoxide::Page;
use tracing::{info, warn};

use super::assembly::{compute_viewport_overall, update_audit_quality, viewport_weighted_overall};
use super::{set_viewport, PipelineConfig};
use crate::audit::module::Viewport;
use crate::audit::report::{AuditReport, PerformanceResults};
use crate::browser::{throttle, BrowserManager, ThrottleProfile};
use crate::interaction::stability::settle;
use crate::performance::prepare_vitals_collection;

pub(super) fn attach_throttled_profile_subchecks(
    report: &mut AuditReport,
    results: &[crate::audit::report::ThrottledPerfResult],
) {
    let subchecks: Vec<crate::audit::SubcheckRun> = ThrottleProfile::AUTO_PROFILES
        .iter()
        .map(|profile| {
            let result = results.iter().find(|result| result.profile == *profile);
            let (status, reason_code) = match result {
                Some(r) if r.score.is_some() => (crate::audit::ExecutionStatus::Completed, None),
                // Measured, but without all core metrics (#769).
                Some(_) => (
                    crate::audit::ExecutionStatus::Partial,
                    Some("core_metrics_incomplete".to_string()),
                ),
                None => (
                    crate::audit::ExecutionStatus::Failed,
                    Some("profile_measurement_failed".to_string()),
                ),
            };
            crate::audit::SubcheckRun {
                subcheck: format!("throttled_profile:{}", profile.label()),
                status,
                reason_code,
                ..Default::default()
            }
        })
        .collect();

    if let Some(performance) = report
        .accessibility
        .execution
        .module_runs
        .iter_mut()
        .find(|run| run.module == "performance")
    {
        if subchecks
            .iter()
            .any(|check| check.status != crate::audit::ExecutionStatus::Completed)
            && performance.status == crate::audit::ExecutionStatus::Completed
        {
            performance.status = crate::audit::ExecutionStatus::Partial;
            performance.reason_code = Some("throttled_profile_incomplete".to_string());
        }
        performance.subchecks.extend(subchecks);
    }

    update_audit_quality(report);
}

/// Undo a throttled pass whose page did not load, and report whether the tab
/// answers again.
///
/// The page keeps loading under the CPU throttle after the navigation gave up.
/// On www.deutschebahn.com (Slow3G, 6x) the renderer was then too busy to
/// answer: every restore command waited out its 30 s CDP timeout, the next
/// profiles did the same, and a 30 s audit ran past rankinglab's 12-minute
/// limit. So the CPU throttle goes first and loading stops, each step gets a
/// short limit, and a tab that does not come back ends the throttled passes.
async fn recover_after_throttled_failure(page: &Page) -> bool {
    use chromiumoxide::cdp::browser_protocol::page::StopLoadingParams;
    let step = std::time::Duration::from_secs(5);
    let cpu = tokio::time::timeout(step, throttle::disable_cpu_throttling(page)).await;
    let _ = tokio::time::timeout(step, page.execute(StopLoadingParams::default())).await;
    let network = tokio::time::timeout(step, throttle::disable_throttling(page)).await;
    let cache = tokio::time::timeout(step, throttle::enable_cache(page)).await;
    matches!(cpu, Ok(Ok(()))) && matches!(network, Ok(Ok(()))) && matches!(cache, Ok(Ok(())))
}

/// Run one performance-only page load per throttle profile and return the results.
///
/// Uses the mobile viewport (most relevant for throttling scenarios).
/// Runs sequentially; errors in individual profiles are logged and skipped.
///
/// Returns the per-profile summary plus the LhMobile vitals/score as the canonical
/// throttled measurement (issue #236). LhMobile matches Lighthouse's mobile preset
/// and is used as the reported Performance score; the unthrottled desktop/mobile
/// passes remain available via `dual_viewport` for diagnostics.
pub(super) async fn collect_throttled_performance(
    page: &Page,
    url: &str,
    browser: &BrowserManager,
    config: &PipelineConfig,
    content_weight: Option<&crate::performance::ContentWeight>,
) -> (
    Vec<crate::audit::report::ThrottledPerfResult>,
    Option<(
        crate::performance::WebVitals,
        crate::performance::PerformanceScore,
    )>,
) {
    use crate::audit::report::ThrottledPerfResult;
    use crate::performance::calculate_performance_score;

    let mut results = Vec::new();
    let mut canonical: Option<(
        crate::performance::WebVitals,
        crate::performance::PerformanceScore,
    )> = None;

    'profiles: for &profile in ThrottleProfile::AUTO_PROFILES {
        let mut samples: Vec<ProfileSample> = Vec::new();
        for run in 1..=config.perf_runs.max(1) {
            info!("Throttled perf pass: {:?} (run {run})", profile);

            if let Err(e) = throttle::apply_throttling(page, profile).await {
                warn!("Throttle apply failed for {:?}: {}", profile, e);
                continue;
            }

            // Without CPU throttling or with a warm cache the pass would be an
            // unthrottled measurement stored under this profile's name (#769).
            let setup_failure = match (
                throttle::apply_cpu_throttling(page, profile).await,
                throttle::disable_cache(page).await,
            ) {
                (Err(e), _) => Some(format!("CPU throttle apply failed for {profile:?}: {e}")),
                (_, Err(e)) => Some(format!("Cache disable failed for {profile:?}: {e}")),
                _ => None,
            };
            if let Some(failure) = setup_failure {
                warn!("{failure}");
                let _ = throttle::enable_cache(page).await;
                let _ = throttle::disable_throttling(page).await;
                let _ = throttle::disable_cpu_throttling(page).await;
                continue;
            }

            let failure = match prepare_vitals_collection(page).await {
                Err(e) => Some(format!("Vitals injection failed for {profile:?}: {e}")),
                Ok(()) => browser
                    .navigate(page, url)
                    .await
                    .err()
                    .map(|e| format!("Navigation failed for {profile:?}: {e}")),
            };
            if let Some(failure) = failure {
                warn!("{failure}");
                if !recover_after_throttled_failure(page).await {
                    warn!(
                    "Tab did not recover after the failed {:?} pass; skipping the remaining throttled profiles",
                    profile
                );
                    break 'profiles;
                }
                continue;
            }

            // The profile's per-metric states are folded into its own subcheck
            // (`throttled_profile:*`), not into the headline run.
            match crate::performance::extract_web_vitals(
                page,
                &crate::audit::collection::CollectionLog::new(),
            )
            .await
            {
                Ok(vitals) => {
                    // Pass the headline content_weight so the size/JS/request caps
                    // apply to throttled profiles too — otherwise a throttled
                    // profile can out-score the headline (and Slow3G out-score
                    // Fast3G) purely because its caps were skipped (#456).
                    let score = calculate_performance_score(&vitals, content_weight);
                    // A profile score needs every core metric; a subset would be a
                    // renormalized number that hides what is missing (#769).
                    let final_score = score.has_core_metrics().then_some(score.overall);
                    samples.push(ProfileSample {
                        vitals,
                        score,
                        final_score,
                    });
                    let _ = throttle::enable_cache(page).await;
                }
                Err(e) => {
                    warn!("Vitals collection failed for {:?}: {}", profile, e);
                    let _ = throttle::enable_cache(page).await;
                }
            }

            if let Err(e) = throttle::disable_throttling(page).await {
                warn!("Throttle disable failed for {:?}: {}", profile, e);
            }
            if let Err(e) = throttle::disable_cpu_throttling(page).await {
                warn!("CPU throttle disable failed for {:?}: {}", profile, e);
            }

            if let Err(e) = settle(page).await {
                warn!("Browser settle failed after {:?}: {}", profile, e);
            }
        }

        let Some((chosen, spread)) = summarize_runs(&samples, config.perf_runs.max(1)) else {
            continue;
        };
        let ProfileSample {
            vitals,
            score,
            final_score,
        } = samples.swap_remove(chosen);
        results.push(ThrottledPerfResult {
            profile,
            lcp_ms: vitals.lcp.as_ref().map(|v| v.value),
            tbt_ms: vitals.tbt.as_ref().map(|v| v.value),
            cls: vitals.cls.as_ref().map(|v| v.value),
            score: final_score,
            spread,
        });
        // LhMobile = Lighthouse mobile preset → canonical perf measurement.
        // Only adopt a complete measurement; otherwise the unthrottled
        // mobile pass stays the headline. With repeated runs the canonical
        // one is the median run, a real measurement and not a blend.
        if let (ThrottleProfile::LhMobile, Some(final_score)) = (profile, final_score) {
            let mut adopted_score = score.clone();
            adopted_score.overall = final_score;
            adopted_score.grade = crate::performance::PerformanceGrade::from_score(final_score);
            // The canonical report vitals come from this throttled pass;
            // tag the direct metrics so the JSON reflects that (#406).
            let mut throttled_vitals = vitals.clone();
            crate::performance::mark_throttled_mobile(&mut throttled_vitals);
            canonical = Some((throttled_vitals, adopted_score));
        }
    }

    // Restore mobile viewport for screenshot capture that follows.
    if let Err(e) = set_viewport(page, Viewport::Mobile).await {
        warn!(
            "Failed to restore mobile viewport after throttled pass: {}",
            e
        );
    }

    // Restore unthrottled state for any subsequent operations.
    if !results.is_empty() {
        let _ = throttle::disable_throttling(page).await;
        let _ = throttle::disable_cpu_throttling(page).await;
    }

    (results, canonical)
}

/// One complete measurement of a throttle profile.
struct ProfileSample {
    vitals: crate::performance::WebVitals,
    score: crate::performance::PerformanceScore,
    /// `None` when not all core metrics were measured.
    final_score: Option<u32>,
}

/// Picks the run a profile reports and, for repeated runs (`--perf-runs`),
/// the spread of all of them.
///
/// The reported run is the one with the median score (lower median for an
/// even count), so the vitals stay one real, internally consistent
/// measurement. Runs without a complete score never win; if none has one the
/// first run is reported. No spread for a single configured run.
fn summarize_runs(
    samples: &[ProfileSample],
    configured_runs: u32,
) -> Option<(usize, Option<crate::audit::report::RunSpread>)> {
    use crate::audit::report::{MetricSpread, RunSpread};
    if samples.is_empty() {
        return None;
    }
    let mut scored: Vec<(usize, u32)> = samples
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.final_score.map(|score| (i, score)))
        .collect();
    scored.sort_by_key(|&(i, score)| (score, i));
    let chosen = scored
        .get((scored.len().max(1) - 1) / 2)
        .map_or(0, |&(i, _)| i);
    if configured_runs <= 1 {
        return Some((chosen, None));
    }
    let spread_of = |values: Vec<f64>| -> Option<MetricSpread> {
        let mut values = values;
        values.sort_by(|a, b| a.total_cmp(b));
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
    };
    let metric = |f: &dyn Fn(&ProfileSample) -> Option<f64>| {
        spread_of(samples.iter().filter_map(f).collect())
    };
    Some((
        chosen,
        Some(RunSpread {
            runs: samples.len() as u32,
            requested_runs: configured_runs,
            lcp_ms: metric(&|s| s.vitals.lcp.as_ref().map(|v| v.value)),
            tbt_ms: metric(&|s| s.vitals.tbt.as_ref().map(|v| v.value)),
            cls: metric(&|s| s.vitals.cls.as_ref().map(|v| v.value)),
            score: metric(&|s| s.final_score.map(f64::from)),
        }),
    ))
}

/// Replace the report's Performance vitals/score with the LhMobile (throttled)
/// measurement and recompute viewport scores accordingly (issue #236).
///
/// Auxiliary structural data (render_blocking, content_weight, third_party,
/// critical_chain, minification, animations, coverage) stays from the unthrottled
/// pass since these describe page composition rather than timing.
pub(super) fn apply_canonical_perf(
    report: &mut AuditReport,
    vitals: crate::performance::WebVitals,
    score: crate::performance::PerformanceScore,
) {
    let measurement_warnings = crate::performance::validate_metrics(&vitals);
    if let Some(ref mut perf) = report.performance {
        perf.measurement_warnings = measurement_warnings;
        perf.vitals = vitals;
        perf.score = score;
    } else {
        report.performance = Some(PerformanceResults {
            vitals,
            score,
            render_blocking: None,
            content_weight: None,
            third_party: None,
            critical_chain: None,
            minification: None,
            animations: None,
            coverage: None,
            measurement_warnings,
        });
    }

    let new_perf = report.performance.as_ref().map(|p| p.score.overall);
    let perf_input = report
        .performance
        .as_ref()
        .filter(|p| p.score.metrics_available > 0)
        .map(|p| p.score.overall);
    let mobile_seo = report.discoverability.seo.as_ref().map(|s| s.score);
    let mobile_mf = report.experience.mobile.as_ref().map(|m| m.score);
    let mobile_hc = super::measured_html_conform_score(report.html_conform.as_ref());
    if let Some(ref mut vps) = report.viewport_scores {
        vps.mobile.performance = new_perf;
        let mobile_overall = compute_viewport_overall(
            vps.mobile.accessibility as f32,
            &[
                ("Performance", perf_input),
                ("SEO", mobile_seo),
                ("Mobile", mobile_mf),
                ("HTML Conformance", mobile_hc),
            ],
        );
        let desktop_overall = vps.desktop.overall;
        vps.mobile.overall = mobile_overall;
        vps.weighted_overall = viewport_weighted_overall(mobile_overall, desktop_overall);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::{calculate_performance_score, VitalMetric, WebVitals};

    fn metric(value: f64) -> Option<VitalMetric> {
        Some(VitalMetric {
            value,
            rating: "good".to_string(),
            target: 0.0,
            measurement: Default::default(),
        })
    }

    fn sample(lcp: f64, tbt: f64, complete: bool) -> ProfileSample {
        let vitals = WebVitals {
            lcp: metric(lcp),
            fcp: metric(1000.0),
            cls: metric(0.02),
            tbt: if complete { metric(tbt) } else { None },
            ..Default::default()
        };
        let score = calculate_performance_score(&vitals, None);
        let final_score = score.has_core_metrics().then_some(score.overall);
        ProfileSample {
            vitals,
            score,
            final_score,
        }
    }

    #[test]
    fn one_run_reports_itself_without_a_spread() {
        let (chosen, spread) = summarize_runs(&[sample(2000.0, 100.0, true)], 1).unwrap();
        assert_eq!(chosen, 0);
        assert!(spread.is_none());
        assert!(summarize_runs(&[], 3).is_none());
    }

    #[test]
    fn the_median_run_is_reported_with_min_median_max_of_every_metric() {
        // Scores fall with LCP: runs 0, 1, 2 are slow, fast, middle.
        let samples = [
            sample(6000.0, 600.0, true),
            sample(1500.0, 50.0, true),
            sample(3500.0, 200.0, true),
        ];
        assert!(samples[1].final_score > samples[2].final_score);
        assert!(samples[2].final_score > samples[0].final_score);
        let (chosen, spread) = summarize_runs(&samples, 3).unwrap();
        assert_eq!(chosen, 2, "the median-score run, not the first or the best");
        let spread = spread.expect("repeated runs carry a spread");
        assert_eq!((spread.runs, spread.requested_runs), (3, 3));
        let lcp = spread.lcp_ms.unwrap();
        assert_eq!((lcp.min, lcp.median, lcp.max), (1500.0, 3500.0, 6000.0));
        let tbt = spread.tbt_ms.unwrap();
        assert_eq!((tbt.min, tbt.median, tbt.max), (50.0, 200.0, 600.0));
        let score = spread.score.unwrap();
        assert!(score.min < score.median && score.median < score.max);
    }

    #[test]
    fn failed_or_incomplete_runs_are_counted_and_never_chosen() {
        // Three of five requested runs measured; one lacks TBT.
        let samples = [
            sample(1500.0, 50.0, true),
            sample(2500.0, 0.0, false),
            sample(6000.0, 600.0, true),
        ];
        let (chosen, spread) = summarize_runs(&samples, 5).unwrap();
        assert_ne!(chosen, 1, "an incomplete run has no score to be the median");
        let spread = spread.unwrap();
        assert_eq!((spread.runs, spread.requested_runs), (3, 5));
        // The incomplete run still counts for the metrics it did measure.
        assert_eq!(spread.lcp_ms.unwrap().median, 2500.0);
        assert_eq!(spread.tbt_ms.unwrap().max, 600.0);
        assert_eq!(
            spread.score.unwrap().min as u32,
            samples[2].final_score.unwrap()
        );
    }

    #[test]
    fn an_even_number_of_runs_uses_the_lower_median_run() {
        let samples = [sample(1500.0, 50.0, true), sample(6000.0, 600.0, true)];
        let (chosen, spread) = summarize_runs(&samples, 2).unwrap();
        assert_eq!(chosen, 1, "lower median = the worse of two, a real run");
        assert_eq!(spread.unwrap().lcp_ms.unwrap().median, 3750.0);
    }
}
