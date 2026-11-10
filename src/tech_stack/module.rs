//! `AuditModule` implementation for Tech-stack detection (#332).

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{Module, PipelineConfig};
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
        cfg.modules.contains(Module::Stack)
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_tech_stack(ctx.page, ctx.url, ctx.collection).await {
            Ok(ts) => Ok(ModuleData::TechStack(Box::new(ts))),
            Err(e) => {
                warn!("Tech stack analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TechStackModule;
    use crate::audit::module::AuditModule;
    use crate::audit::{Module, ModuleSet, PipelineConfig};

    fn config_with_stack(check_stack: bool) -> PipelineConfig {
        let modules = if check_stack {
            ModuleSet::none().with(Module::Stack)
        } else {
            ModuleSet::none()
        };
        PipelineConfig::default().with_modules(modules)
    }

    #[test]
    fn tech_stack_module_enablement_follows_pipeline_flag() {
        let module = TechStackModule;

        assert!(module.is_enabled(&config_with_stack(true)));
        assert!(!module.is_enabled(&config_with_stack(false)));
    }
}
