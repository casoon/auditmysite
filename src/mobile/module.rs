//! `AuditModule` implementation for Mobile-friendliness collection (#332).

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_mobile_friendliness;

pub struct MobileModule;

#[async_trait]
impl AuditModule for MobileModule {
    fn id(&self) -> &'static str {
        "mobile"
    }

    fn label(&self) -> &'static str {
        "Mobile"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_mobile
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_mobile_friendliness(ctx.page).await {
            Ok(m) => Ok(ModuleData::Mobile(Box::new(m))),
            Err(e) => {
                warn!("Mobile analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}
