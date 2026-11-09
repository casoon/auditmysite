//! Shared JavaScript snippets for in-page accessibility extraction.
//!
//! WCAG rules that evaluate JavaScript in the page (`styles.rs`,
//! `aria_hidden_focus.rs`, …) used to each carry their own CSS-path and
//! visibility heuristics. The divergent copies produced inconsistent results
//! — most visibly bare tag-name selectors like `"a"` that are not locatable.
//! These constants centralise the logic so every rule shares one definition.

/// `__amsCssSelector(el)` — builds a locatable CSS path for an element.
///
/// Never falls back to a bare tag name: walks up to 5 ancestors, prepends
/// `:nth-of-type(n)` when same-tag siblings exist, and stops early at the
/// nearest ancestor with an `id`.
///
/// The same snippet carries the audit-exclusion helpers (#645), so every
/// DOM rule that locates elements has them without extra wiring:
///
/// - `__amsIsExcludedEl(el)` — true when `el` lies inside a subtree the
///   exclusion step marked for this page load (`window.__amsIsExcluded`,
///   installed by `audit::exclusion::resolve_scope`; absent → false).
/// - `__amsPush(list, el, item, limit)` / `__amsReal(list)` — capped rules
///   push through these so excluded elements never spend the rule's cap:
///   real and excluded hits are capped separately, and the loop bound reads
///   the real count. Excluded hits are still returned, so the pipeline can
///   drop and count them like any other excluded finding.
pub(crate) const CSS_SELECTOR_JS: &str = include_str!("js/helpers_css_selector.js");

/// `__amsIsVisuallyHidden(el)` — detects the visually-hidden / `.sr-only`
/// pattern (text exposed only to assistive technology).
///
/// Recognises unrendered elements, clip-rect `rect(0 0 0 0)`, ≤1px boxes that
/// clip their overflow (`hidden`/`clip`),
/// `clip-path: inset(50%/100%)`, and far off-screen positioned/indented text.
/// WCAG contrast (1.4.3) does not apply to such elements — axe-core skips
/// them too.
pub(crate) const IS_VISUALLY_HIDDEN_JS: &str = include_str!("js/helpers_is_visually_hidden.js");
