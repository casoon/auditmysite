//! `AuditModule` implementation for HTML5 conformance collection.
//!
//! Runs on the mobile viewport pass only (see `PipelineConfig::for_viewport`'s
//! `Viewport::Desktop` branch), exactly like SEO/Mobile — a static/URL-level
//! property once per page, not a per-viewport-DOM signal.

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_html_conform;

pub struct HtmlConformModule;

#[async_trait]
impl AuditModule for HtmlConformModule {
    fn id(&self) -> &'static str {
        "html_conform"
    }

    fn label(&self) -> &'static str {
        "HTML Conformance"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_html_conform
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_html_conform(ctx.page, ctx.collection).await {
            Ok(a) => Ok(ModuleData::HtmlConform(Box::new(a))),
            Err(e) => {
                warn!("HTML conformance analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }
}
