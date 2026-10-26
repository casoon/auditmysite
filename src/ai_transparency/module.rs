//! `AuditModule` implementation for the ai-transparency module.
//!
//! Requires BOTH the `ai-transparency` Cargo feature (pulls in the `c2pa`
//! crate) and the runtime `--ai-transparency` flag to actually fetch/parse
//! anything. If the flag is passed without the feature compiled in, `src/cli/
//! runners.rs` fails loudly with a `ConfigError` before the pipeline ever
//! runs — this module's `collect()` fallback below is a defensive backstop,
//! not the primary gate, so a missing feature never silently no-ops.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

pub struct AiTransparencyModule;

#[async_trait]
impl AuditModule for AiTransparencyModule {
    fn id(&self) -> &'static str {
        "ai_transparency"
    }

    fn label(&self) -> &'static str {
        "AI Transparency"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_ai_transparency
    }

    #[cfg(feature = "ai-transparency")]
    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        use tracing::warn;
        match super::image_provenance::analyze_ai_transparency(ctx.page, ctx.collection).await {
            Ok(a) => Ok(ModuleData::AiTransparency(Box::new(a))),
            Err(e) => {
                warn!("AI transparency analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }

    #[cfg(not(feature = "ai-transparency"))]
    async fn collect(&self, _ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        Ok(ModuleData::None)
    }
}
