//! `AuditModule` implementation for Journey collection (#333).
//!
//! Moved out of `extract_snapshot` (where A3 left it inline). The DOM
//! fallback for the `<main>` landmark check is preserved exactly: if the
//! AX tree exposes no `main`, query the DOM to distinguish a truly missing
//! landmark from one hidden by an overlay (e.g. a consent banner covering
//! `<main>` on mobile).
//!
//! Gate matches the previous inline behavior: `check_mobile || check_seo`.

use async_trait::async_trait;

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::PipelineConfig;
use crate::error::Result;

use super::analyze_journey_with_dom_check;

pub struct JourneyModule;

#[async_trait]
impl AuditModule for JourneyModule {
    fn id(&self) -> &'static str {
        "journey"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_mobile || cfg.check_seo
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        let ax_has_main = ctx
            .ax_tree
            .iter()
            .any(|n| n.role.as_deref() == Some("main"));
        let dom_has_main = if ax_has_main {
            true
        } else {
            ctx.page
                .evaluate("!!document.querySelector('main, [role=\"main\"]')")
                .await
                .ok()
                .and_then(|r| r.value().and_then(|v| v.as_bool()))
                .unwrap_or(false)
        };
        let journey = analyze_journey_with_dom_check(ctx.ax_tree, dom_has_main);
        Ok(ModuleData::Journey(Box::new(journey)))
    }
}
