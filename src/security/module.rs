//! `AuditModule` implementation for Security collection (Phase A3 / #332).
//!
//! Security analysis is URL-only and viewport-independent. Today
//! `audit_page` fetches it once before the desktop/mobile passes and the
//! per-pass `PipelineConfig` has `check_security = false`, so this module
//! is registered in the catalog but inert during `extract_snapshot`.
//! A future refactor can hoist the top-level fetch into the catalog.

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_security;

pub struct SecurityModule;

#[async_trait]
impl AuditModule for SecurityModule {
    fn id(&self) -> &'static str {
        "security"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_security
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_security(ctx.url).await {
            Ok(s) => Ok(ModuleData::Security(Box::new(s))),
            Err(e) => {
                warn!("Security analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}
