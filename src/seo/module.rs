//! `AuditModule` implementation for SEO collection (Phase A3 / #332).
//!
//! Wraps `analyze_seo` and surfaces the result as `ModuleData::Seo`.
//! Failure mirrors the previous inline behavior: log a warning and yield
//! `ModuleData::None` so downstream consumers see absence-of-data rather
//! than a propagated error.

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_seo;

pub struct SeoModule;

#[async_trait]
impl AuditModule for SeoModule {
    fn id(&self) -> &'static str {
        "seo"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_seo
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
}
