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

/// Whether best practices (console and library checks) are collected.
/// Security alone is enough: the library check is a security check and must
/// not depend on performance. Shared by the module gate and the console
/// collector setup so the two cannot drift apart.
pub(crate) fn collects_best_practices(cfg: &PipelineConfig) -> bool {
    cfg.check_performance || cfg.check_security
}

#[async_trait]
impl AuditModule for BestPracticesModule {
    fn id(&self) -> &'static str {
        "best_practices"
    }

    fn label(&self) -> &'static str {
        "Best Practices"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        collects_best_practices(cfg)
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
