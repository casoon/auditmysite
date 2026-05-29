//! `AuditModule` implementation for Best-Practices collection (#332).
//!
//! Gated on `check_performance` to match today's pipeline behavior — best
//! practices live alongside the performance pass and only run when perf
//! collection is requested.

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
        cfg.check_performance
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_best_practices(ctx.page).await {
            Ok(bp) => Ok(ModuleData::BestPractices(Box::new(bp))),
            Err(e) => {
                warn!("Best practices analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}
