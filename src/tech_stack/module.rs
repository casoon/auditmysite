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

#[cfg(test)]
mod tests {
    use super::TechStackModule;
    use crate::audit::module::AuditModule;
    use crate::audit::PipelineConfig;
    use crate::cli::{InteractiveMode, WcagLevel};

    fn config_with_stack(check_stack: bool) -> PipelineConfig {
        PipelineConfig {
            wcag_level: WcagLevel::AA,
            timeout_secs: 30,
            verbose: false,
            check_performance: false,
            check_seo: false,
            check_security: false,
            check_mobile: false,
            check_dark_mode: false,
            check_stack,
            persist_artifacts: false,
            capture_screenshots: false,
            dismiss_consent: false,
            interactive: InteractiveMode::Off,
            journey_budget_ms: crate::a11y_journey::DEFAULT_BUDGET_MS,
            lang: "de".to_string(),
        }
    }

    #[test]
    fn tech_stack_module_enablement_follows_pipeline_flag() {
        let module = TechStackModule;

        assert!(module.is_enabled(&config_with_stack(true)));
        assert!(!module.is_enabled(&config_with_stack(false)));
    }
}
