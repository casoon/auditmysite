//! Throttled performance passes (single-URL only): one page load per throttle
//! profile after the two viewport passes, recovery of a tab a failed pass left
//! busy, and adoption of the LhMobile measurement as the canonical score.

use chromiumoxide::Page;
use tracing::{info, warn};

use super::assembly::{
    compute_viewport_site_quality, update_audit_quality, viewport_weighted_site_quality,
};
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
    Option<CanonicalPerf>,
) {
    use crate::audit::report::ThrottledPerfResult;
    use crate::performance::calculate_performance_score;

    let mut results = Vec::new();
    let mut canonical: Option<CanonicalPerf> = None;

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

        let Some(summary) = summarize_runs(&samples, config.perf_runs.max(1), content_weight)
        else {
            continue;
        };
        let RunSummary {
            vitals,
            score,
            final_score,
            spread,
        } = summary;
        results.push(ThrottledPerfResult {
            profile,
            lcp_ms: vitals.lcp.as_ref().map(|v| v.value),
            tbt_ms: vitals.tbt.as_ref().map(|v| v.value),
            cls: vitals.cls.as_ref().map(|v| v.value),
            score: final_score,
            spread: spread.clone(),
        });
        // LhMobile = Lighthouse mobile preset → canonical perf measurement.
        // Only adopt a complete measurement; otherwise the unthrottled
        // mobile pass stays the headline. With repeated runs the canonical
        // vitals are the per-metric medians (#1231).
        if let (ThrottleProfile::LhMobile, Some(final_score)) = (profile, final_score) {
            let mut adopted_score = score.clone();
            adopted_score.overall = final_score;
            adopted_score.grade = crate::performance::PerformanceGrade::from_score(final_score);
            // The canonical report vitals come from this throttled pass;
            // tag the direct metrics so the JSON reflects that (#406).
            let mut throttled_vitals = vitals.clone();
            crate::performance::mark_throttled_mobile(&mut throttled_vitals);
            canonical = Some(CanonicalPerf {
                vitals: throttled_vitals,
                score: adopted_score,
                runs: spread.as_ref().map_or(1, |s| s.runs),
                stability: spread.as_ref().map_or(Default::default(), |s| s.stability),
            });
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

/// The measurement a throttle profile reports.
struct RunSummary {
    vitals: crate::performance::WebVitals,
    score: crate::performance::PerformanceScore,
    /// `None` when not all core metrics were measured.
    final_score: Option<u32>,
    /// Spread of the repeated runs; `None` for a single configured run.
    spread: Option<crate::audit::report::RunSpread>,
}

/// The LhMobile measurement the report adopts as its headline (#236) and what
/// it is the median of (#1231).
pub(super) struct CanonicalPerf {
    pub vitals: crate::performance::WebVitals,
    pub score: crate::performance::PerformanceScore,
    /// Page loads the vitals are the median of.
    pub runs: u32,
    pub stability: crate::audit::MeasurementStability,
}

/// Combines the runs of one profile (`--perf-runs`) into what it reports.
///
/// One run reports itself. Several runs report the per-metric medians of LCP,
/// FCP, CLS, TBT and TTI (each with its spread, over the runs that measured
/// it) on top of the median-score run, which supplies everything that is not
/// aggregated; the score is that of the median vitals. A run that loaded but
/// lacks a metric counts for the others. No spread for a single configured
/// run.
fn summarize_runs(
    samples: &[ProfileSample],
    configured_runs: u32,
    content_weight: Option<&crate::performance::ContentWeight>,
) -> Option<RunSummary> {
    if samples.is_empty() {
        return None;
    }
    // The median-score run (lower median for an even count) is the base.
    // Runs without a complete score never win; if none has one, the first.
    let mut scored: Vec<(usize, u32)> = samples
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.final_score.map(|score| (i, score)))
        .collect();
    scored.sort_by_key(|&(i, score)| (score, i));
    let base = scored
        .get((scored.len().max(1) - 1) / 2)
        .map_or(0, |&(i, _)| i);
    let runs: Vec<&crate::performance::WebVitals> = samples.iter().map(|s| &s.vitals).collect();
    let scores: Vec<f64> = samples
        .iter()
        .filter_map(|s| s.final_score.map(f64::from))
        .collect();
    let spread = (configured_runs > 1)
        .then(|| crate::performance::runs::run_spread(&runs, &scores, configured_runs));
    if samples.len() == 1 {
        let ProfileSample {
            vitals,
            score,
            final_score,
        } = &samples[0];
        return Some(RunSummary {
            vitals: vitals.clone(),
            score: score.clone(),
            final_score: *final_score,
            spread,
        });
    }
    let vitals = crate::performance::runs::median_vitals(&samples[base].vitals, &runs);
    let score = crate::performance::calculate_performance_score(&vitals, content_weight);
    let final_score = score.has_core_metrics().then_some(score.overall);
    Some(RunSummary {
        vitals,
        score,
        final_score,
        spread,
    })
}

/// Replace the report's Performance vitals/score with the LhMobile (throttled)
/// measurement and recompute viewport scores accordingly (issue #236).
///
/// Auxiliary structural data (render_blocking, content_weight, third_party,
/// critical_chain, minification, animations, coverage) stays from the unthrottled
/// pass since these describe page composition rather than timing.
pub(super) fn apply_canonical_perf(report: &mut AuditReport, canonical: CanonicalPerf) {
    let CanonicalPerf {
        vitals,
        score,
        runs,
        stability,
    } = canonical;
    let measurement_warnings = crate::performance::validate_metrics(&vitals);
    if let Some(ref mut perf) = report.performance {
        perf.measurement_warnings = measurement_warnings;
        perf.vitals = vitals;
        perf.score = score;
        perf.runs = runs;
        perf.stability = stability;
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
            runs,
            stability,
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
        let mobile_site_quality = compute_viewport_site_quality(
            vps.mobile.accessibility as f32,
            &[
                ("Performance", perf_input),
                ("SEO", mobile_seo),
                ("Mobile", mobile_mf),
                ("HTML Conformance", mobile_hc),
            ],
        );
        let desktop_site_quality = vps.desktop.site_quality;
        vps.mobile.site_quality = mobile_site_quality;
        vps.weighted_site_quality =
            viewport_weighted_site_quality(mobile_site_quality, desktop_site_quality);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::MeasurementStability;
    use crate::performance::{calculate_performance_score, VitalMetric, WebVitals};

    fn sample(lcp: f64, tbt: f64, complete: bool) -> ProfileSample {
        let vitals = WebVitals {
            lcp: Some(VitalMetric::new(lcp, 2500.0, 4000.0)),
            fcp: Some(VitalMetric::new(1000.0, 1800.0, 3000.0)),
            cls: Some(VitalMetric::new(0.02, 0.1, 0.25)),
            tbt: complete.then(|| VitalMetric::new(tbt, 200.0, 600.0)),
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
        let one = [sample(2000.0, 100.0, true)];
        let summary = summarize_runs(&one, 1, None).unwrap();
        assert!(summary.spread.is_none());
        assert_eq!(summary.vitals.lcp.as_ref().unwrap().value, 2000.0);
        assert!(summary.vitals.lcp.unwrap().spread.is_none());
        assert!(summarize_runs(&[], 3, None).is_none());
    }

    #[test]
    fn the_headline_is_the_median_of_every_metric_with_its_spread() {
        let samples = [
            sample(6000.0, 600.0, true),
            sample(1500.0, 50.0, true),
            sample(3500.0, 200.0, true),
        ];
        let summary = summarize_runs(&samples, 3, None).unwrap();
        let lcp = summary.vitals.lcp.as_ref().unwrap();
        assert_eq!(lcp.value, 3500.0);
        let lcp_spread = lcp.spread.as_ref().unwrap();
        assert_eq!((lcp_spread.min, lcp_spread.max), (1500.0, 6000.0));
        assert_eq!(summary.vitals.tbt.as_ref().unwrap().value, 200.0);

        let spread = summary.spread.expect("repeated runs carry a spread");
        assert_eq!((spread.runs, spread.requested_runs), (3, 3));
        assert_eq!(spread.stability, MeasurementStability::Unstable);
        let score = spread.score.unwrap();
        assert!(score.min < score.median && score.median < score.max);
        // The score belongs to the median vitals, not to one of the runs.
        let expected = calculate_performance_score(&summary.vitals, None).overall;
        assert_eq!(summary.final_score, Some(expected));
    }

    #[test]
    fn runs_that_agree_are_stable() {
        let samples = [
            sample(2000.0, 100.0, true),
            sample(2100.0, 100.0, true),
            sample(2200.0, 100.0, true),
        ];
        let spread = summarize_runs(&samples, 3, None).unwrap().spread.unwrap();
        assert_eq!(spread.stability, MeasurementStability::Stable);
    }

    #[test]
    fn a_run_without_a_metric_counts_for_the_others_and_is_counted() {
        // Three of five requested runs loaded; one lacks TBT.
        let samples = [
            sample(1500.0, 50.0, true),
            sample(2500.0, 0.0, false),
            sample(6000.0, 600.0, true),
        ];
        let summary = summarize_runs(&samples, 5, None).unwrap();
        let spread = summary.spread.unwrap();
        assert_eq!((spread.runs, spread.requested_runs), (3, 5));
        assert_eq!(summary.vitals.lcp.unwrap().value, 2500.0);
        let tbt = summary.vitals.tbt.unwrap();
        assert_eq!(tbt.value, 325.0, "median of the two runs that measured it");
        assert_eq!(tbt.spread.unwrap().max, 600.0);
    }

    #[test]
    fn a_single_loaded_run_of_several_is_not_stability_checked() {
        let summary = summarize_runs(&[sample(2000.0, 100.0, true)], 3, None).unwrap();
        let spread = summary.spread.unwrap();
        assert_eq!((spread.runs, spread.requested_runs), (1, 3));
        assert_eq!(spread.stability, MeasurementStability::SingleRun);
        assert!(summary.vitals.lcp.unwrap().spread.is_none());
    }
}
