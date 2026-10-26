//! `AuditModule` implementation for the design-quality module (#528).

use async_trait::async_trait;
use tracing::warn;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, PipelineConfig};
use crate::error::Result;

use super::{analyze_design_quality, layout_transition_advisories};

pub struct DesignQualityModule;

#[async_trait]
impl AuditModule for DesignQualityModule {
    fn id(&self) -> &'static str {
        "design_quality"
    }

    fn label(&self) -> &'static str {
        "Design Quality"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_design_quality
    }

    fn depends_on(&self) -> &'static [&'static str] {
        &["performance"]
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        match analyze_design_quality(ctx.page, ctx.collection).await {
            Ok(dq) => Ok(ModuleData::DesignQuality(Box::new(dq))),
            Err(e) => {
                warn!("Design quality analysis failed: {}", e);
                Ok(ModuleData::None)
            }
        }
    }

    /// Re-projects `performance.animations` (already populated in `report` by
    /// the time `derive_all` runs) as layout-transition advisories, rather
    /// than independently re-detecting the same CSS transitions — see
    /// `layout_transition_advisories`'s doc comment.
    fn derive(&self, report: &mut AuditReport, _locale: &str) -> Result<()> {
        let Some(animations) = report
            .performance
            .as_ref()
            .and_then(|p| p.animations.as_ref())
        else {
            return Ok(());
        };
        let advisories = layout_transition_advisories(animations);
        if advisories.is_empty() {
            return Ok(());
        }
        if let Some(dq) = report.experience.design_quality.as_mut() {
            dq.rules_run.push("design.layout_transition".to_string());
            dq.findings.extend(advisories);
        }
        Ok(())
    }
}
