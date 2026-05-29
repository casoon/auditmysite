//! `AuditModule` implementation for Dark-mode collection (#332).

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_dark_mode;

pub struct DarkModeModule;

#[async_trait]
impl AuditModule for DarkModeModule {
    fn id(&self) -> &'static str {
        "dark_mode"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_dark_mode
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_dark_mode(ctx.page, ctx.pipeline_config.wcag_level).await {
            Ok(dm) => Ok(ModuleData::DarkMode(Box::new(dm))),
            Err(e) => {
                warn!("Dark mode analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}
