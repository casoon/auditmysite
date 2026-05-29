//! `AuditModule` implementation for Content-Visibility post-processing (#333).
//!
//! Runs during the derive phase **after** both `source_quality` and
//! `ai_visibility` have produced their fields — the content-visibility
//! analysis reads them for EEAT, content-depth and topical-authority
//! signals. Always enabled — matches the unconditional call site this
//! replaces in `aggregate_report`.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, PipelineConfig};
use crate::error::Result;

use super::analyze_content_visibility;

pub struct ContentVisibilityModule;

#[async_trait]
impl AuditModule for ContentVisibilityModule {
    fn id(&self) -> &'static str {
        "content_visibility"
    }

    fn label(&self) -> &'static str {
        "Content Visibility"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_seo
    }

    fn depends_on(&self) -> &'static [&'static str] {
        // Reads report.seo (collect), report.source_quality (derive),
        // report.ai_visibility (derive), report.patterns. Topo-order must
        // place this after both derive-phase dependencies.
        &["seo", "source_quality", "ai_visibility"]
    }

    async fn collect(&self, _ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        Ok(ModuleData::None)
    }

    fn derive(&self, report: &mut AuditReport) -> Result<()> {
        report.content_visibility = Some(analyze_content_visibility(report));
        Ok(())
    }
}
