//! The modules a run can select next to the accessibility checks (#933).
//!
//! [`ModuleSet`] is the `modules` field of
//! [`PipelineConfig`](crate::audit::PipelineConfig). The accessibility checks
//! are not a member: they run in every audit.

/// An analysis module a run can select next to the accessibility checks.
///
/// Non-exhaustive: a new module is a new variant, not a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Module {
    /// Core Web Vitals and the throttled performance passes.
    Performance,
    /// Meta tags, headings, structured data.
    Seo,
    /// Security headers.
    Security,
    /// Mobile friendliness.
    Mobile,
    /// HTML5 spec-conformance (`html-conform` crate).
    HtmlConform,
    /// Tech stack detection from in-page signals.
    Stack,
    /// Pre-consent privacy check (#871): one extra visit in a clean browser
    /// context. Opt-in, not part of [`ModuleSet::full`].
    Privacy,
}

impl Module {
    /// Every module, in declaration order.
    const ALL: [Module; 7] = [
        Module::Performance,
        Module::Seo,
        Module::Security,
        Module::Mobile,
        Module::HtmlConform,
        Module::Stack,
        Module::Privacy,
    ];

    const fn bit(self) -> u16 {
        1 << self as u16
    }
}

/// A set of [`Module`]s.
///
/// ```
/// use auditmysite::audit::{Module, ModuleSet};
///
/// let modules = ModuleSet::none().with(Module::Seo).with(Module::Mobile);
/// assert!(modules.contains(Module::Seo));
/// assert!(!modules.contains(Module::Performance));
/// assert_eq!(modules, [Module::Mobile, Module::Seo].into_iter().collect());
///
/// // The default run: every module except the opt-in privacy check.
/// assert!(ModuleSet::full().contains(Module::Performance));
/// assert!(!ModuleSet::full().contains(Module::Privacy));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleSet(u16);

impl ModuleSet {
    /// No module: the accessibility checks alone.
    pub const fn none() -> Self {
        Self(0)
    }

    /// The modules of a full audit (`--full`, the CLI default): performance,
    /// SEO, security, mobile, HTML conformance and stack detection. The
    /// privacy check is opt-in and not included.
    pub const fn full() -> Self {
        Self::none()
            .with(Module::Performance)
            .with(Module::Seo)
            .with(Module::Security)
            .with(Module::Mobile)
            .with(Module::HtmlConform)
            .with(Module::Stack)
    }

    /// Whether `module` is in the set.
    pub const fn contains(self, module: Module) -> bool {
        self.0 & module.bit() != 0
    }

    /// Whether every module of `other` is in the set.
    pub const fn contains_all(self, other: ModuleSet) -> bool {
        self.0 & other.0 == other.0
    }

    /// The set with `module` added.
    #[must_use]
    pub const fn with(self, module: Module) -> Self {
        Self(self.0 | module.bit())
    }

    /// The set with `module` removed.
    #[must_use]
    pub const fn without(self, module: Module) -> Self {
        Self(self.0 & !module.bit())
    }

    /// The modules in the set, in the declaration order of [`Module`].
    pub fn iter(self) -> impl Iterator<Item = Module> {
        Module::ALL
            .into_iter()
            .filter(move |module| self.contains(*module))
    }
}

impl FromIterator<Module> for ModuleSet {
    fn from_iter<I: IntoIterator<Item = Module>>(iter: I) -> Self {
        iter.into_iter().fold(Self::none(), Self::with)
    }
}

impl std::fmt::Debug for ModuleSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_is_the_six_modules_without_privacy() {
        assert_eq!(
            ModuleSet::full().iter().collect::<Vec<_>>(),
            [
                Module::Performance,
                Module::Seo,
                Module::Security,
                Module::Mobile,
                Module::HtmlConform,
                Module::Stack,
            ]
        );
        assert!(!ModuleSet::full().contains(Module::Privacy));
    }

    #[test]
    fn none_contains_no_module() {
        assert_eq!(ModuleSet::none().iter().count(), 0);
        for module in Module::ALL {
            assert!(!ModuleSet::none().contains(module));
        }
    }

    #[test]
    fn with_and_without_change_one_module_only() {
        for module in Module::ALL {
            let only = ModuleSet::none().with(module);
            assert_eq!(only.iter().collect::<Vec<_>>(), [module]);
            assert_eq!(only.with(module), only, "adding twice is adding once");
            assert_eq!(only.without(module), ModuleSet::none());

            let all_but = Module::ALL
                .into_iter()
                .collect::<ModuleSet>()
                .without(module);
            assert!(!all_but.contains(module));
            assert_eq!(all_but.iter().count(), Module::ALL.len() - 1);
        }
    }

    #[test]
    fn contains_all_is_the_superset_test() {
        let full = ModuleSet::full();
        assert!(full.contains_all(full));
        assert!(full.with(Module::Privacy).contains_all(full));
        assert!(!full.contains_all(full.with(Module::Privacy)));
        assert!(!full.without(Module::Seo).contains_all(full));
        assert!(ModuleSet::none().contains_all(ModuleSet::none()));
        assert!(full.contains_all(ModuleSet::none()));
    }

    #[test]
    fn collecting_ignores_order_and_duplicates() {
        let set: ModuleSet = [Module::Stack, Module::Seo, Module::Stack]
            .into_iter()
            .collect();
        assert_eq!(set, ModuleSet::none().with(Module::Seo).with(Module::Stack));
        assert_eq!(format!("{set:?}"), "{Seo, Stack}");
    }
}
