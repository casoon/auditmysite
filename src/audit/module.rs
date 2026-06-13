//! Audit module trait and supporting types (scaffold for #330 / Phase A1).
//!
//! Defines the shape that every audit module will conform to once Phase A is
//! complete. This file is purely additive — no existing module uses it yet and
//! `pipeline.rs` is untouched. Migration of the 15 existing modules onto this
//! trait happens in A3 (collection modules) and A4 (post-processing modules).
//!
//! Design choices:
//! - `async_trait` is used so the trait stays dyn-compatible — A2 builds a
//!   `Vec<Box<dyn AuditModule>>` registry on top of this scaffold.
//! - `ModuleData` is a closed enum with one variant per existing `Results`
//!   struct. We deliberately do not introduce a uniform `Score`/`Findings`
//!   shape across modules; each module keeps its native result type and the
//!   normalizer continues to fan them out.
//! - `collect` and `derive` both default to no-ops so a module only needs to
//!   implement the phase it actually participates in.

use async_trait::async_trait;
use chromiumoxide::Page;

use crate::accessibility::AXTree;
use crate::ai_visibility::AiVisibilityAnalysis;
use crate::audit::pipeline::PipelineConfig;
use crate::audit::report::{AuditReport, PerformanceResults};
use crate::best_practices::BestPracticesAnalysis;
use crate::content_visibility::ContentVisibilityAnalysis;
use crate::dark_mode::DarkModeAnalysis;
use crate::error::Result;
use crate::journey::JourneyAnalysis;
use crate::mobile::MobileFriendliness;
use crate::security::SecurityAnalysis;
use crate::seo::SeoAnalysis;
use crate::source_quality::SourceQualityAnalysis;
use crate::tech_stack::TechStackAnalysis;
use crate::ux::UxAnalysis;

/// Viewport pass currently driving the audit.
///
/// Mirrors the desktop/mobile split that `pipeline.rs` already performs. Kept
/// as a small, owned enum so the trait does not pull in viewport setup
/// machinery from the pipeline module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewport {
    Desktop,
    Mobile,
}

/// Inputs handed to an `AuditModule::collect` call.
///
/// Borrowed by the catalog (A2) for the duration of one viewport pass. Modules
/// must not retain references past the call; the borrow only outlives the
/// async collection itself.
pub struct ModuleContext<'a> {
    pub page: &'a Page,
    pub url: &'a str,
    pub viewport: Viewport,
    pub ax_tree: &'a AXTree,
    pub pipeline_config: &'a PipelineConfig,
}

/// Payload returned by an `AuditModule::collect` call.
///
/// One variant per existing module result type. `None` is the trait default
/// (a module that only participates in the `derive` phase, or whose collection
/// was skipped because the module is inactive). Adding a new audit module
/// means extending this enum, which intentionally forces the compiler to flag
/// any consumer that has not been taught about the new variant.
#[derive(Debug, Clone)]
pub enum ModuleData {
    None,
    Performance(Box<PerformanceResults>),
    Seo(Box<SeoAnalysis>),
    Security(Box<SecurityAnalysis>),
    Mobile(Box<MobileFriendliness>),
    Ux(Box<UxAnalysis>),
    Journey(Box<JourneyAnalysis>),
    DarkMode(Box<DarkModeAnalysis>),
    TechStack(Box<TechStackAnalysis>),
    BestPractices(Box<BestPracticesAnalysis>),
    SourceQuality(Box<SourceQualityAnalysis>),
    AiVisibility(Box<AiVisibilityAnalysis>),
    ContentVisibility(Box<ContentVisibilityAnalysis>),
    Error(String),
}

/// Contract every audit module will implement.
///
/// Two phases:
/// 1. `collect` — runs while a browser page is loaded. CDP-bound modules
///    (performance, SEO, security, …) produce a `ModuleData` variant here.
/// 2. `derive` — runs after the `NormalizedReport` exists. Aggregate modules
///    (source quality, AI visibility, content visibility) read the report and
///    write their own field via a side effect on `&mut NormalizedReport`.
///
/// A module typically implements one of the two phases; the other stays at
/// its default no-op.
///
/// Example skeleton (not wired up — Phase A3/A4 will migrate concrete modules):
///
/// ```
/// use async_trait::async_trait;
/// use auditmysite::audit::module::{AuditModule, ModuleContext, ModuleData};
/// use auditmysite::audit::PipelineConfig;
/// use auditmysite::error::Result;
///
/// struct NoopModule;
///
/// #[async_trait]
/// impl AuditModule for NoopModule {
///     fn id(&self) -> &'static str { "noop" }
///     fn is_enabled(&self, _cfg: &PipelineConfig) -> bool { false }
/// }
/// ```
#[async_trait]
pub trait AuditModule: Send + Sync {
    /// Stable identifier. Used by the catalog (A2) for topological ordering
    /// via [`AuditModule::depends_on`], appears in cache signatures and may
    /// surface in CLI/report output.
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use auditmysite::audit::module::AuditModule;
    /// use auditmysite::audit::PipelineConfig;
    ///
    /// struct SeoModule;
    ///
    /// #[async_trait]
    /// impl AuditModule for SeoModule {
    ///     fn id(&self) -> &'static str { "seo" }
    ///     fn is_enabled(&self, cfg: &PipelineConfig) -> bool { cfg.check_seo }
    /// }
    ///
    /// assert_eq!(SeoModule.id(), "seo");
    /// ```
    fn id(&self) -> &'static str;

    fn label(&self) -> &'static str {
        self.id()
    }

    /// Whether this module should run in the current pipeline configuration.
    /// Mirrors the existing `PipelineConfig.check_*` flags; A3 wires each
    /// module's gate to the corresponding flag.
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use auditmysite::audit::module::AuditModule;
    /// use auditmysite::audit::PipelineConfig;
    ///
    /// struct AlwaysOn;
    ///
    /// #[async_trait]
    /// impl AuditModule for AlwaysOn {
    ///     fn id(&self) -> &'static str { "always" }
    ///     fn is_enabled(&self, _cfg: &PipelineConfig) -> bool { true }
    /// }
    /// ```
    fn is_enabled(&self, cfg: &PipelineConfig) -> bool;

    /// Modules that must run before this one. Returned IDs match
    /// [`AuditModule::id`]. Default: no dependencies. Used by A2's topological
    /// sort so post-processing modules (e.g. `content_visibility` depends on
    /// `seo`) see populated upstream state.
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use auditmysite::audit::module::AuditModule;
    /// use auditmysite::audit::PipelineConfig;
    ///
    /// struct ContentVisibility;
    ///
    /// #[async_trait]
    /// impl AuditModule for ContentVisibility {
    ///     fn id(&self) -> &'static str { "content_visibility" }
    ///     fn is_enabled(&self, _cfg: &PipelineConfig) -> bool { true }
    ///     fn depends_on(&self) -> &'static [&'static str] { &["seo", "ai_visibility"] }
    /// }
    ///
    /// assert_eq!(ContentVisibility.depends_on(), &["seo", "ai_visibility"]);
    /// ```
    fn depends_on(&self) -> &'static [&'static str] {
        &[]
    }

    /// CDP-bound collection phase. Default: produce no payload. A module that
    /// only participates in `derive` can leave this untouched.
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use auditmysite::audit::module::{AuditModule, ModuleContext, ModuleData};
    /// use auditmysite::audit::PipelineConfig;
    /// use auditmysite::error::Result;
    ///
    /// struct DeriveOnly;
    ///
    /// #[async_trait]
    /// impl AuditModule for DeriveOnly {
    ///     fn id(&self) -> &'static str { "derive_only" }
    ///     fn is_enabled(&self, _cfg: &PipelineConfig) -> bool { true }
    ///     // collect() defaults to ModuleData::None
    /// }
    /// ```
    async fn collect(&self, _ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        Ok(ModuleData::None)
    }

    /// Post-processing phase. Runs after all `collect` results have been
    /// folded into the raw `AuditReport`. Aggregate modules (source quality,
    /// AI visibility, content visibility) read fields the collect phase
    /// populated and write their own field via mutation. Default: no-op.
    ///
    /// Operates on `AuditReport` (the raw report shape) — that is what the
    /// existing post-processing functions consume and what the pipeline has
    /// in hand when this phase runs.
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use auditmysite::audit::module::{AuditModule, ModuleData};
    /// use auditmysite::audit::{AuditReport, PipelineConfig};
    /// use auditmysite::error::Result;
    ///
    /// struct CollectOnly;
    ///
    /// #[async_trait]
    /// impl AuditModule for CollectOnly {
    ///     fn id(&self) -> &'static str { "collect_only" }
    ///     fn is_enabled(&self, _cfg: &PipelineConfig) -> bool { true }
    ///     // derive() defaults to Ok(())
    /// }
    ///
    /// fn assert_default<M: AuditModule>(m: &M, r: &mut AuditReport) -> Result<()> {
    ///     m.derive(r, "de")
    /// }
    /// ```
    ///
    /// `locale` is the active report locale ("de"/"en"); derive modules that
    /// produce report-visible text use it to localize their messages (#406).
    fn derive(&self, _report: &mut AuditReport, _locale: &str) -> Result<()> {
        Ok(())
    }
}
