//! `AuditModule` implementation for the commerce audit.
//!
//! Derive-only: reads the SEO module's JSON-LD (`discoverability.seo`), the
//! screen-reader link inventory (`screen_reader_audit`, populated before
//! `derive_all`) for anchor texts, and the tech stack (`discoverability.tech_stack`)
//! for the shop gate. Writes `report.commerce`. Self-gating via `analyze_commerce`.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::{AuditReport, Module, PipelineConfig};
use crate::error::Result;
use crate::tech_stack::TechCategory;

use super::analyze_commerce;

pub(crate) struct CommerceModule;

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
        cfg.modules.contains(Module::Seo) && cfg.runs_experimental_module("commerce")
    }

    fn depends_on(&self) -> &'static [&'static str] {
        &["seo", "tech_stack"]
    }

    async fn collect(&self, _ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        Ok(ModuleData::None)
    }

    fn derive(&self, report: &mut AuditReport, _locale: &str) -> Result<()> {
        let anchor_texts: Vec<String> = report
            .screen_reader_audit
            .as_ref()
            .map(|sr| {
                sr.navigation_views
                    .links
                    .iter()
                    .filter_map(|l| l.text.clone())
                    .collect()
            })
            .unwrap_or_default();

        // BFSGV §19 Nr. 3 scope signal (see `commerce` module doc comment).
        let has_identification_control = report.screen_reader_audit.as_ref().is_some_and(|sr| {
            sr.navigation_views
                .form_controls
                .iter()
                .any(|f| f.is_identification_control)
        });

        let is_ecommerce_stack = report
            .discoverability
            .tech_stack
            .as_ref()
            .map(|t| {
                t.detected
                    .iter()
                    .any(|d| d.category == TechCategory::Ecommerce)
            })
            .unwrap_or(false);

        let url = report.url.clone();
        report.commerce = report.discoverability.seo.as_ref().and_then(|seo| {
            analyze_commerce(
                &url,
                &seo.structured_data,
                &anchor_texts,
                is_ecommerce_stack,
                has_identification_control,
            )
        });
        Ok(())
    }
}
