//! `AuditModule` implementation for UX collection (#332).
//!
//! UX is computed synchronously from the AX tree. Gated on
//! `Module::Seo` or `Module::Mobile` and on `--experimental ux` (#916).

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{Module, PipelineConfig};
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
        (cfg.modules.contains(Module::Seo) || cfg.modules.contains(Module::Mobile))
            && cfg.runs_experimental_module("ux")
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        let ux = analyze_ux(ctx.ax_tree);
        Ok(ModuleData::Ux(Box::new(ux)))
    }
}
