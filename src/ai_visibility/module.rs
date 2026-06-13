//! `AuditModule` implementation for AI-Visibility post-processing (#333).
//!
//! Runs during the derive phase. Reads `seo`, `security`, score, and url
//! from the assembled `AuditReport` and writes `report.ai_visibility`.
//! Always enabled — matches the unconditional call site this replaces in
//! `aggregate_report`.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, PipelineConfig};
use crate::error::Result;

use super::analyze_ai_visibility;

pub struct AiVisibilityModule;

#[async_trait]
impl AuditModule for AiVisibilityModule {
    fn id(&self) -> &'static str {
        "ai_visibility"
    }

    fn label(&self) -> &'static str {
        "AI Visibility"
    }

    fn is_enabled(&self, _cfg: &PipelineConfig) -> bool {
        true
    }

    fn depends_on(&self) -> &'static [&'static str] {
        // Reads report.seo and report.security from collect-phase modules.
        &["seo", "security"]
    }

    async fn collect(&self, _ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        Ok(ModuleData::None)
    }

    fn derive(&self, report: &mut AuditReport, _locale: &str) -> Result<()> {
        report.ai_visibility = Some(analyze_ai_visibility(report));
        Ok(())
    }
}
