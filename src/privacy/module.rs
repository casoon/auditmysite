//! `AuditModule` implementation for the first-visit privacy check (#871).
//!
//! Like security and the extra-reload checks, the capture does not run in the
//! per-viewport `collect` phase: it opens a clean browser context of its own
//! and runs once per page from `pipeline::run_extra_page_checks`, which fills
//! `report.experience.privacy`. The module exists so the catalog, the module
//! labels, the requested scope and the finding contract know the module.

use async_trait::async_trait;

use crate::audit::module::AuditModule;
use crate::audit::PipelineConfig;

pub struct PrivacyModule;

#[async_trait]
impl AuditModule for PrivacyModule {
    fn id(&self) -> &'static str {
        "privacy"
    }

    fn label(&self) -> &'static str {
        "Privacy"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_privacy
    }
}
