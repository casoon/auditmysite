//! `AuditModule` implementation for Tech-stack detection (#332).

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_tech_stack;

pub struct TechStackModule;

#[async_trait]
impl AuditModule for TechStackModule {
    fn id(&self) -> &'static str {
        "tech_stack"
    }

    fn label(&self) -> &'static str {
        "Tech Stack"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_stack
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_tech_stack(ctx.page, ctx.url).await {
            Ok(ts) => Ok(ModuleData::TechStack(Box::new(ts))),
            Err(e) => {
                warn!("Tech stack analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}
