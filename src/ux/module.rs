//! `AuditModule` implementation for UX collection (#332).
//!
//! UX is computed synchronously from the AX tree. Gated on
//! `check_seo || check_mobile` to match today's behavior.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_ux;

pub struct UxModule;

#[async_trait]
impl AuditModule for UxModule {
    fn id(&self) -> &'static str {
        "ux"
    }

    fn label(&self) -> &'static str {
        "UX"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_seo || cfg.check_mobile
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        let ux = analyze_ux(ctx.ax_tree, &ctx.pipeline_config.lang);
        Ok(ModuleData::Ux(Box::new(ux)))
    }
}
