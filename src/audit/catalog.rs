//! Audit catalog — registry of `AuditModule` implementations with
//! deterministic topological ordering (Phase A2-A4 / #331-#333).
//!
//! Drives both phases of the audit:
//! - `collect_all` runs `collect` on every enabled module in topo order
//!   (called from `extract_snapshot`).
//! - `derive_all` runs `derive` on every enabled module in topo order
//!   (called from `aggregate_report` for post-processing modules).
//!
//! Design notes:
//! - Cycles, unknown dependencies and duplicate IDs are catalog configuration
//!   errors and return `AuditError::ConfigError`.
//! - Among modules whose dependencies are satisfied at the same Kahn step,
//!   the lexicographically smallest `id()` wins. That makes the result
//!   invariant to the order modules were registered in, which keeps cache
//!   signatures stable across catalog refactors.

use std::collections::{BTreeMap, BTreeSet};

use crate::audit::module::{AuditModule, ModuleContext, ModuleData};
use crate::audit::pipeline::PipelineConfig;
use crate::audit::report::AuditReport;
use crate::error::{AuditError, Result};
use tracing::warn;

/// Owns every `AuditModule` instance for a run.
pub struct AuditCatalog {
    modules: Vec<Box<dyn AuditModule>>,
}

impl AuditCatalog {
    /// Standard catalog — the eight collection modules registered in A3 plus
    /// the four post-processing modules registered in A4 (Journey via
    /// `collect`, source/AI/content visibility via `derive`).
    pub fn standard() -> Self {
        use crate::ai_visibility::AiVisibilityModule;
        use crate::best_practices::BestPracticesModule;
        use crate::content_visibility::ContentVisibilityModule;
        use crate::dark_mode::DarkModeModule;
        use crate::journey::JourneyModule;
        use crate::mobile::MobileModule;
        use crate::performance::PerformanceModule;
        use crate::security::SecurityModule;
        use crate::seo::SeoModule;
        use crate::source_quality::SourceQualityModule;
        use crate::tech_stack::TechStackModule;
        use crate::ux::UxModule;

        Self::empty()
            .with_module(Box::new(PerformanceModule))
            .with_module(Box::new(SeoModule))
            .with_module(Box::new(SecurityModule))
            .with_module(Box::new(MobileModule))
            .with_module(Box::new(DarkModeModule))
            .with_module(Box::new(TechStackModule))
            .with_module(Box::new(UxModule))
            .with_module(Box::new(BestPracticesModule))
            .with_module(Box::new(JourneyModule))
            .with_module(Box::new(SourceQualityModule))
            .with_module(Box::new(AiVisibilityModule))
            .with_module(Box::new(ContentVisibilityModule))
    }

    /// Catalog with no registered modules.
    pub fn empty() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    /// Register one module (builder-style).
    pub fn with_module(mut self, module: Box<dyn AuditModule>) -> Self {
        self.modules.push(module);
        self
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Every registered module in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &dyn AuditModule> + '_ {
        self.modules.iter().map(|m| m.as_ref())
    }

    /// Modules currently enabled by the pipeline configuration, in insertion
    /// order. Use [`topo_sorted`](Self::topo_sorted) when dependency ordering
    /// matters.
    pub fn enabled<'a>(
        &'a self,
        cfg: &'a PipelineConfig,
    ) -> impl Iterator<Item = &'a dyn AuditModule> + 'a {
        self.modules
            .iter()
            .filter(move |m| m.is_enabled(cfg))
            .map(|m| m.as_ref())
    }

    /// Every module in dependency order via Kahn's algorithm.
    ///
    /// # Errors
    /// `AuditError::ConfigError` if a module declares an unknown dependency,
    /// if duplicate IDs are registered, or if a dependency cycle is detected.
    pub fn topo_sorted(&self) -> Result<Vec<&dyn AuditModule>> {
        let mut idx_of: BTreeMap<&'static str, usize> = BTreeMap::new();
        for (i, m) in self.modules.iter().enumerate() {
            if idx_of.insert(m.id(), i).is_some() {
                return Err(AuditError::ConfigError(format!(
                    "duplicate audit module id in catalog: {}",
                    m.id()
                )));
            }
        }

        for m in &self.modules {
            for dep in m.depends_on() {
                if !idx_of.contains_key(dep) {
                    return Err(AuditError::ConfigError(format!(
                        "audit module '{}' depends on unknown module '{}'",
                        m.id(),
                        dep
                    )));
                }
            }
        }

        let mut in_degree: BTreeMap<&'static str, usize> = BTreeMap::new();
        for m in &self.modules {
            in_degree.insert(m.id(), m.depends_on().len());
        }

        let mut dependents: BTreeMap<&'static str, BTreeSet<&'static str>> = BTreeMap::new();
        for m in &self.modules {
            for dep in m.depends_on() {
                dependents.entry(*dep).or_default().insert(m.id());
            }
        }

        let mut ready: BTreeSet<&'static str> = in_degree
            .iter()
            .filter_map(|(id, &deg)| if deg == 0 { Some(*id) } else { None })
            .collect();

        let mut sorted: Vec<&dyn AuditModule> = Vec::with_capacity(self.modules.len());
        while let Some(id) = ready.pop_first() {
            let idx = *idx_of.get(id).expect("indexed above");
            sorted.push(self.modules[idx].as_ref());

            if let Some(dependents_of_id) = dependents.get(id) {
                for &dep_id in dependents_of_id {
                    let count = in_degree
                        .get_mut(dep_id)
                        .expect("dependent module is registered");
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(dep_id);
                    }
                }
            }
        }

        if sorted.len() != self.modules.len() {
            let cycle: Vec<&'static str> = in_degree
                .iter()
                .filter_map(|(id, &deg)| if deg > 0 { Some(*id) } else { None })
                .collect();
            return Err(AuditError::ConfigError(format!(
                "cycle detected in audit catalog among modules: {:?}",
                cycle
            )));
        }

        Ok(sorted)
    }

    /// Run `collect` on every enabled module, in dependency order.
    ///
    /// Returns one `(id, payload)` tuple per enabled module. The id lets
    /// callers route variants back into a snapshot struct without relying
    /// on iteration order.
    ///
    /// Errors from any single module's `collect` propagate. Modules that
    /// want to treat their own failures as missing-data should swallow
    /// the error internally and return `Ok(ModuleData::None)` — that
    /// mirrors today's `extract_snapshot` behavior where a failing
    /// sub-analysis logs a warning and yields `None`.
    pub async fn collect_all(
        &self,
        ctx: &ModuleContext<'_>,
    ) -> Result<Vec<(&'static str, ModuleData)>> {
        let mut ordered = self.topo_sorted()?;
        ordered.retain(|m| m.is_enabled(ctx.pipeline_config));

        let mut results = Vec::with_capacity(ordered.len());
        for module in ordered {
            let data = match module.collect(ctx).await {
                Ok(d) => d,
                Err(e) => {
                    warn!("module '{}' collect failed: {}", module.id(), e);
                    ModuleData::Error(e.to_string())
                }
            };
            results.push((module.id(), data));
        }
        Ok(results)
    }

    /// Run `derive` on every enabled module, in dependency order.
    ///
    /// Sequential, by design: post-processing modules depend on the
    /// `AuditReport` fields previous modules wrote. Topo order guarantees
    /// dependencies see populated upstream state — e.g. `content_visibility`
    /// reads `report.source_quality` and `report.ai_visibility`, both of
    /// which run earlier under their declared `depends_on`.
    ///
    /// Collect-only modules' default `derive` is a no-op, so this method
    /// safely walks the whole catalog without per-phase filtering.
    pub fn derive_all(&self, report: &mut AuditReport, cfg: &PipelineConfig) -> Result<()> {
        let ordered = self.topo_sorted()?;
        for module in ordered {
            if module.is_enabled(cfg) {
                module.derive(report, &cfg.lang)?;
            }
        }
        Ok(())
    }
}

impl Default for AuditCatalog {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::module::AuditModule;
    use crate::cli::Args;
    use async_trait::async_trait;
    use clap::Parser;

    struct StubModule {
        id: &'static str,
        deps: &'static [&'static str],
        enabled: bool,
    }

    impl StubModule {
        const fn new(id: &'static str) -> Self {
            Self {
                id,
                deps: &[],
                enabled: true,
            }
        }
        const fn with_deps(mut self, deps: &'static [&'static str]) -> Self {
            self.deps = deps;
            self
        }
        const fn disabled(mut self) -> Self {
            self.enabled = false;
            self
        }
    }

    #[async_trait]
    impl AuditModule for StubModule {
        fn id(&self) -> &'static str {
            self.id
        }
        fn is_enabled(&self, _cfg: &PipelineConfig) -> bool {
            self.enabled
        }
        fn depends_on(&self) -> &'static [&'static str] {
            self.deps
        }
    }

    fn boxed(m: StubModule) -> Box<dyn AuditModule> {
        Box::new(m)
    }

    fn cfg() -> PipelineConfig {
        PipelineConfig::from(&Args::parse_from(["auditmysite", "https://example.com"]))
    }

    fn ids(modules: &[&dyn AuditModule]) -> Vec<&'static str> {
        modules.iter().map(|m| m.id()).collect()
    }

    #[test]
    fn standard_catalog_registers_all_modules() {
        let cat = AuditCatalog::standard();
        let ids: std::collections::BTreeSet<&'static str> = cat.iter().map(|m| m.id()).collect();
        let expected: std::collections::BTreeSet<&'static str> = [
            // A3 collection modules
            "performance",
            "seo",
            "security",
            "mobile",
            "dark_mode",
            "tech_stack",
            "ux",
            "best_practices",
            // A4 post-processing (+ journey collect)
            "journey",
            "source_quality",
            "ai_visibility",
            "content_visibility",
        ]
        .into_iter()
        .collect();
        assert_eq!(ids, expected);
        cat.topo_sorted().expect("standard catalog must topo-sort");
    }

    #[test]
    fn standard_catalog_orders_post_processing_after_dependencies() {
        let cat = AuditCatalog::standard();
        let sorted = cat.topo_sorted().expect("sortable");
        let order: Vec<&'static str> = sorted.iter().map(|m| m.id()).collect();

        let pos = |needle: &str| {
            order
                .iter()
                .position(|id| *id == needle)
                .unwrap_or_else(|| panic!("module {needle} missing from topo order"))
        };

        // source_quality reads seo, security, ux
        assert!(pos("seo") < pos("source_quality"));
        assert!(pos("security") < pos("source_quality"));
        assert!(pos("ux") < pos("source_quality"));

        // ai_visibility reads seo, security
        assert!(pos("seo") < pos("ai_visibility"));
        assert!(pos("security") < pos("ai_visibility"));

        // content_visibility reads seo + both derive-phase modules
        assert!(pos("seo") < pos("content_visibility"));
        assert!(pos("source_quality") < pos("content_visibility"));
        assert!(pos("ai_visibility") < pos("content_visibility"));
    }

    #[test]
    fn topo_sort_orders_by_dependencies() {
        let cat = AuditCatalog::empty()
            .with_module(boxed(StubModule::new("seo")))
            .with_module(boxed(
                StubModule::new("content_visibility").with_deps(&["seo", "ai_visibility"]),
            ))
            .with_module(boxed(StubModule::new("ai_visibility")));

        let sorted = cat.topo_sorted().expect("sortable");
        let order = ids(&sorted);

        let pos = |needle: &str| order.iter().position(|id| *id == needle).unwrap();
        assert!(pos("seo") < pos("content_visibility"));
        assert!(pos("ai_visibility") < pos("content_visibility"));
    }

    #[test]
    fn topo_sort_is_deterministic_across_insertion_order() {
        let cat_a = AuditCatalog::empty()
            .with_module(boxed(StubModule::new("alpha")))
            .with_module(boxed(StubModule::new("beta")))
            .with_module(boxed(StubModule::new("gamma")));
        let cat_b = AuditCatalog::empty()
            .with_module(boxed(StubModule::new("gamma")))
            .with_module(boxed(StubModule::new("alpha")))
            .with_module(boxed(StubModule::new("beta")));
        let a = cat_a.topo_sorted().unwrap();
        let b = cat_b.topo_sorted().unwrap();
        assert_eq!(ids(&a), ids(&b));
        assert_eq!(ids(&a), vec!["alpha", "beta", "gamma"]);
    }

    fn expect_config_error(result: Result<Vec<&dyn AuditModule>>) -> String {
        match result {
            Ok(modules) => panic!(
                "expected ConfigError, got Ok with {} modules",
                modules.len()
            ),
            Err(AuditError::ConfigError(msg)) => msg,
            Err(other) => panic!("expected ConfigError, got {other:?}"),
        }
    }

    #[test]
    fn unknown_dependency_is_a_config_error() {
        let cat = AuditCatalog::empty().with_module(boxed(
            StubModule::new("dependent").with_deps(&["does_not_exist"]),
        ));
        let msg = expect_config_error(cat.topo_sorted());
        assert!(msg.contains("dependent"), "got: {msg}");
        assert!(msg.contains("does_not_exist"), "got: {msg}");
    }

    #[test]
    fn duplicate_module_id_is_a_config_error() {
        let cat = AuditCatalog::empty()
            .with_module(boxed(StubModule::new("twin")))
            .with_module(boxed(StubModule::new("twin")));
        let msg = expect_config_error(cat.topo_sorted());
        assert!(msg.contains("twin"));
    }

    #[test]
    fn module_data_error_variant_can_be_constructed() {
        let err = ModuleData::Error("test error".into());
        assert!(matches!(err, ModuleData::Error(_)));
    }

    #[test]
    fn cycle_is_a_config_error() {
        // a -> b -> a
        let cat = AuditCatalog::empty()
            .with_module(boxed(StubModule::new("a").with_deps(&["b"])))
            .with_module(boxed(StubModule::new("b").with_deps(&["a"])));
        let msg = expect_config_error(cat.topo_sorted());
        assert!(msg.contains("cycle detected"), "got: {msg}");
        assert!(msg.contains("a"), "got: {msg}");
        assert!(msg.contains("b"), "got: {msg}");
    }

    #[test]
    fn enabled_filters_by_pipeline_config() {
        let cat = AuditCatalog::empty()
            .with_module(boxed(StubModule::new("on_1")))
            .with_module(boxed(StubModule::new("off").disabled()))
            .with_module(boxed(StubModule::new("on_2")));

        let cfg = cfg();
        let on: Vec<&'static str> = cat.enabled(&cfg).map(|m| m.id()).collect();
        assert_eq!(on, vec!["on_1", "on_2"]);
    }

    #[test]
    fn iter_preserves_insertion_order() {
        let cat = AuditCatalog::empty()
            .with_module(boxed(StubModule::new("gamma")))
            .with_module(boxed(StubModule::new("alpha")))
            .with_module(boxed(StubModule::new("beta")));
        let order: Vec<&'static str> = cat.iter().map(|m| m.id()).collect();
        assert_eq!(order, vec!["gamma", "alpha", "beta"]);
    }
}
