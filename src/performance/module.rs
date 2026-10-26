//! `AuditModule` implementation for Performance collection (#332).
//!
//! Wraps the existing performance pipeline:
//! 1. `extract_web_vitals` is the gate — if vitals fail, the whole module
//!    yields `ModuleData::None` (no sub-analyses are run).
//! 2. Seven sub-analyses (render-blocking, content-weight, third-party,
//!    critical-chain, minification, animations, coverage) each record their
//!    collection state (#777) and contribute `None` when they did not
//!    measure.
//! 3. `measurement_warnings` combines vitals validation warnings with the
//!    coverage analysis's own warnings, preserving the previous order.

use async_trait::async_trait;
use tracing::warn;

use crate::audit::collection::{CollectionMethod, CollectorSpec};
use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PerformanceResults;
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::{
    analyze_content_weight, analyze_critical_chain, analyze_minification,
    analyze_non_composited_animations, analyze_render_blocking, analyze_third_party_attribution,
    calculate_performance_score, extract_web_vitals, take_coverage_results, validate_metrics,
};

pub struct PerformanceModule;

#[async_trait]
impl AuditModule for PerformanceModule {
    fn id(&self) -> &'static str {
        "performance"
    }

    fn label(&self) -> &'static str {
        "Performance"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_performance
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        let page = ctx.page;
        let url = ctx.url;
        let log = ctx.collection;

        let vitals = match extract_web_vitals(page, log).await {
            Ok(v) => v,
            Err(e) => {
                warn!("Performance analysis failed: {}", e);
                return Ok(ModuleData::None);
            }
        };

        // Size, JS and request penalties come from the content weight: a
        // failed read is no page of 0 bytes (#769).
        let content_weight = log
            .record(&CONTENT_WEIGHT, analyze_content_weight(page).await)
            .into_value();

        let score = calculate_performance_score(&vitals, content_weight.as_ref());

        // If not a single Web Vital could be measured, the 0 the scorer returns is
        // not a genuine result — it would otherwise enter the weighted overall at
        // 20% and silently deflate the score by ~15-20 points. Treat it like a
        // failed measurement and yield `None` so performance is excluded rather
        // than scored as 0 (#448).
        if score.metrics_available == 0 {
            warn!(
                "Performance analysis produced no measurable Web Vitals; treating as not measured"
            );
            return Ok(ModuleData::None);
        }

        let render_blocking = log
            .record(&RENDER_BLOCKING, analyze_render_blocking(page, url).await)
            .into_value();
        let third_party = log
            .record(
                &THIRD_PARTY,
                analyze_third_party_attribution(page, url).await,
            )
            .into_value();
        let critical_chain = log
            .record(&CRITICAL_CHAIN, analyze_critical_chain(page).await)
            .into_value();
        let minification = log
            .record(&MINIFICATION, analyze_minification(page).await)
            .into_value();
        let animations = log
            .record(&ANIMATIONS, analyze_non_composited_animations(page).await)
            .into_value();
        let coverage = match take_coverage_results(page, log).await {
            Ok(c) => Some(c),
            Err(e) => {
                warn!("Coverage analysis failed: {}", e);
                None
            }
        };

        let mut measurement_warnings = validate_metrics(&vitals);
        if let Some(cov) = &coverage {
            measurement_warnings.extend(cov.measurement_warnings.clone());
        }

        Ok(ModuleData::Performance(Box::new(PerformanceResults {
            vitals,
            score,
            render_blocking,
            content_weight,
            third_party,
            critical_chain,
            minification,
            animations,
            coverage,
            measurement_warnings,
        })))
    }
}

const fn resource_collector(id: &'static str, scoring: bool) -> CollectorSpec {
    CollectorSpec {
        id,
        module: "performance",
        method: CollectionMethod::JavaScript,
        method_version: 1,
        scoring,
    }
}

/// Resource Timing sizes; the score's size, JS and request penalties.
pub(crate) static CONTENT_WEIGHT: CollectorSpec =
    resource_collector("performance.content_weight", true);
pub(crate) static RENDER_BLOCKING: CollectorSpec =
    resource_collector("performance.render_blocking", false);
pub(crate) static THIRD_PARTY: CollectorSpec = resource_collector("performance.third_party", false);
pub(crate) static CRITICAL_CHAIN: CollectorSpec =
    resource_collector("performance.critical_chain", false);
pub(crate) static MINIFICATION: CollectorSpec =
    resource_collector("performance.minification", false);
pub(crate) static ANIMATIONS: CollectorSpec = resource_collector("performance.animations", false);
