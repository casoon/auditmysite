//! `AuditModule` implementation for the commerce schema pass.
//!
//! Derive-only: reads the JSON-LD collected by the SEO module
//! (`report.discoverability.seo.structured_data`) and writes `report.commerce`.
//! Self-gating — `analyze_commerce` returns `None` on non-product pages, so the
//! field stays `None` for landing/editorial pages.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, PipelineConfig};
use crate::error::Result;

use super::analyze_commerce;

pub struct CommerceModule;

#[async_trait]
impl AuditModule for CommerceModule {
    fn id(&self) -> &'static str {
        "commerce"
    }

    fn label(&self) -> &'static str {
        "Commerce"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        // Derived from the SEO module's structured data; only meaningful when SEO ran.
        cfg.check_seo
    }

    fn depends_on(&self) -> &'static [&'static str] {
        &["seo"]
    }

    async fn collect(&self, _ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        Ok(ModuleData::None)
    }

    fn derive(&self, report: &mut AuditReport, _locale: &str) -> Result<()> {
        report.commerce = report
            .discoverability
            .seo
            .as_ref()
            .and_then(|seo| analyze_commerce(&seo.structured_data));
        Ok(())
    }
}
