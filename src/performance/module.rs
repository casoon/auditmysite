//! `AuditModule` implementation for Performance collection (#332).
//!
//! Wraps the existing performance pipeline:
//! 1. `extract_web_vitals` is the gate — if vitals fail, the whole module
//!    yields `ModuleData::None` (no sub-analyses are run).
//! 2. Seven sub-analyses (render-blocking, content-weight, third-party,
//!    critical-chain, minification, animations, coverage) each log a
//!    warning on failure and contribute `None` to the result struct.
//! 3. `measurement_warnings` combines vitals validation warnings with the
//!    coverage analysis's own warnings, preserving the previous order.

use async_trait::async_trait;
use tracing::warn;

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

        let vitals = match extract_web_vitals(page).await {
            Ok(v) => v,
            Err(e) => {
                warn!("Performance analysis failed: {}", e);
                return Ok(ModuleData::None);
            }
        };

        let content_weight = match analyze_content_weight(page).await {
            Ok(cw) => Some(cw),
            Err(e) => {
                warn!("Content-weight analysis failed: {}", e);
                None
            }
        };

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

        let render_blocking = match analyze_render_blocking(page, url).await {
            Ok(rb) => Some(rb),
            Err(e) => {
                warn!("Render-blocking analysis failed: {}", e);
                None
            }
        };
        let third_party = match analyze_third_party_attribution(page, url).await {
            Ok(tp) => Some(tp),
            Err(e) => {
                warn!("Third-party attribution analysis failed: {}", e);
                None
            }
        };
        let critical_chain = match analyze_critical_chain(page).await {
            Ok(cc) => Some(cc),
            Err(e) => {
                warn!("Critical chain analysis failed: {}", e);
                None
            }
        };
        let minification = match analyze_minification(page).await {
            Ok(m) => Some(m),
            Err(e) => {
                warn!("Minification analysis failed: {}", e);
                None
            }
        };
        let animations = match analyze_non_composited_animations(page).await {
            Ok(a) => Some(a),
            Err(e) => {
                warn!("Non-composited animation analysis failed: {}", e);
                None
            }
        };
        let coverage = match take_coverage_results(page).await {
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
