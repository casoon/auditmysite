//! Throttled performance passes (single-URL only): one page load per throttle
//! profile after the two viewport passes, recovery of a tab a failed pass left
//! busy, and adoption of the LhMobile measurement as the canonical score.

use chromiumoxide::Page;
use tracing::{info, warn};

use super::assembly::{compute_viewport_overall, update_audit_quality};
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
            let completed = results.iter().any(|result| result.profile == *profile);
            crate::audit::SubcheckRun {
                subcheck: format!("throttled_profile:{}", profile.label()),
                status: if completed {
                    crate::audit::ExecutionStatus::Completed
                } else {
                    crate::audit::ExecutionStatus::Failed
                },
                reason_code: (!completed).then(|| "profile_measurement_failed".to_string()),
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
            .any(|check| check.status == crate::audit::ExecutionStatus::Failed)
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
    _config: &PipelineConfig,
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

    for &profile in ThrottleProfile::AUTO_PROFILES {
        info!("Throttled perf pass: {:?}", profile);

        if let Err(e) = throttle::apply_throttling(page, profile).await {
            warn!("Throttle apply failed for {:?}: {}", profile, e);
            continue;
        }

        if let Err(e) = throttle::apply_cpu_throttling(page, profile).await {
            warn!("CPU throttle apply failed for {:?}: {}", profile, e);
        }

        if let Err(e) = throttle::disable_cache(page).await {
            warn!("Cache disable failed for {:?}: {}", profile, e);
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
                break;
            }
            continue;
        }

        match crate::performance::extract_web_vitals(page).await {
            Ok(vitals) => {
                // Pass the headline content_weight so the size/JS/request caps
                // apply to throttled profiles too — otherwise a throttled
                // profile can out-score the headline (and Slow3G out-score
                // Fast3G) purely because its caps were skipped (#456).
                let score = calculate_performance_score(&vitals, content_weight);
                // If LCP could not be measured under throttling (timeout or
                // navigation pre-completion), the most important navigation
                // metric is missing — do not let CLS/TBT alone push the score
                // to 100. Cap to "AUSBAUFÄHIG" tier so the profile reflects
                // that the measurement was incomplete.
                let final_score = if vitals.lcp.is_none() {
                    score.overall.min(50)
                } else {
                    score.overall
                };
                results.push(ThrottledPerfResult {
                    profile,
                    lcp_ms: vitals.lcp.as_ref().map(|v| v.value),
                    tbt_ms: vitals.tbt.as_ref().map(|v| v.value),
                    cls: vitals.cls.as_ref().map(|v| v.value),
                    score: final_score,
                });
                // LhMobile = Lighthouse mobile preset → canonical perf measurement.
                // Only adopt when LCP could actually be measured; otherwise fall
                // back to the unthrottled mobile pass so we don't report a
                // capped-to-50 score that reflects measurement failure.
                if profile == ThrottleProfile::LhMobile && vitals.lcp.is_some() {
                    let mut adopted_score = score.clone();
                    adopted_score.overall = final_score;
                    adopted_score.grade =
                        crate::performance::PerformanceGrade::from_score(final_score);
                    // The canonical report vitals come from this throttled pass;
                    // tag the direct metrics so the JSON reflects that (#406).
                    let mut throttled_vitals = vitals.clone();
                    crate::performance::mark_throttled_mobile(&mut throttled_vitals);
                    canonical = Some((throttled_vitals, adopted_score));
                }
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
    let mobile_seo = report.discoverability.seo.as_ref().map(|s| s.score);
    let mobile_mf = report.experience.mobile.as_ref().map(|m| m.score);
    if let Some(ref mut vps) = report.viewport_scores {
        vps.mobile.performance = new_perf;
        let mobile_overall = compute_viewport_overall(
            vps.mobile.accessibility as f32,
            vps.mobile.performance,
            mobile_seo,
            mobile_mf,
        );
        let desktop_overall = vps.desktop.overall;
        vps.mobile.overall = mobile_overall;
        vps.weighted_overall =
            (mobile_overall as f64 * 0.7 + desktop_overall as f64 * 0.3).round() as u32;
    }
}
