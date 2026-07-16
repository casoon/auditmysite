//! `AuditModule` implementation for SEO collection (Phase A3 / #332).
//!
//! Wraps `analyze_seo` and surfaces the result as `ModuleData::Seo`.
//! Failure mirrors the previous inline behavior: log a warning and yield
//! `ModuleData::None` so downstream consumers see absence-of-data rather
//! than a propagated error.

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, PipelineConfig};
use crate::error::Result;

use super::analyze_seo;

pub struct SeoModule;

#[async_trait]
impl AuditModule for SeoModule {
    fn id(&self) -> &'static str {
        "seo"
    }

    fn label(&self) -> &'static str {
        "SEO"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_seo
    }

    fn depends_on(&self) -> &'static [&'static str] {
        &["journey"]
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_seo(ctx.page, ctx.url).await {
            Ok(seo) => Ok(ModuleData::Seo(Box::new(seo))),
            Err(e) => {
                warn!("SEO analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }

    fn derive(&self, report: &mut AuditReport, _locale: &str) -> Result<()> {
        let Some(page_intent) = report.journey.as_ref().map(|journey| journey.page_intent) else {
            return Ok(());
        };
        let url = report.url.clone();
        let Some(seo) = report.discoverability.seo.as_mut() else {
            return Ok(());
        };

        let fit = crate::seo::schema_fit::assess_schema_fit_with_facts(
            &url,
            page_intent,
            &seo.structured_data,
            &seo.structured_data.visible_facts,
        );
        crate::seo::schema::refresh_rule_assessments(
            &mut seo.structured_data,
            fit.product_context(),
        );
        seo.structured_data.fit_assessment = Some(fit);
        seo.content_profile = Some(crate::seo::profile::build_content_profile(seo, "en"));
        Ok(())
    }
}
