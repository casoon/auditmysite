//! `AuditModule` implementation for Best-Practices collection (#332).
//!
//! Console and library checks accompany performance or security collection.

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_best_practices;

pub struct BestPracticesModule;

#[async_trait]
impl AuditModule for BestPracticesModule {
    fn id(&self) -> &'static str {
        "best_practices"
    }

    fn label(&self) -> &'static str {
        "Best Practices"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_performance || cfg.check_security
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_best_practices(ctx.page, ctx.collection).await {
            Ok(bp) => Ok(ModuleData::BestPractices(Box::new(bp))),
            Err(e) => {
                warn!("Best practices analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}
