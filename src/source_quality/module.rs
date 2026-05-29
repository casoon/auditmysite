//! `AuditModule` implementation for Source-Quality post-processing (#333).
//!
//! Runs during the derive phase. Reads `seo`, `security`, `ux`, and the
//! accessibility statistics from the assembled `AuditReport` and writes
//! `report.source_quality`. Always enabled — matches the unconditional
//! call site this replaces in `aggregate_report`.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, PipelineConfig};
use crate::error::Result;

use super::analyze_source_quality;

pub struct SourceQualityModule;

#[async_trait]
impl AuditModule for SourceQualityModule {
    fn id(&self) -> &'static str {
        "source_quality"
    }

    fn is_enabled(&self, _cfg: &PipelineConfig) -> bool {
        true
    }

    fn depends_on(&self) -> &'static [&'static str] {
        // Reads report.seo, report.security, report.ux. Topo-orders the
        // collect-phase modules so those fields are populated first.
        &["seo", "security", "ux"]
    }

    async fn collect(&self, _ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        Ok(ModuleData::None)
    }

    fn derive(&self, report: &mut AuditReport) -> Result<()> {
        report.source_quality = Some(analyze_source_quality(report));
        Ok(())
    }
}
