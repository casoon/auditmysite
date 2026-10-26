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

use super::analyze_journey_with_page_context;

/// Sums the visible text that a visitor actually meets first: text whose box
/// lies within the first viewport height.
///
/// The Journey module's "little visible text in the upper area" finding used
/// to approximate this by taking the first N nodes of the accessibility tree
/// in document order. Tree order is not vertical order — on
/// www.inros-lackner.de the first fifty nodes are a skip link and forty
/// wrappers, and its `article` elements appear as empty shells before their
/// own text (plan 51). Measuring it in the page is measuring what the
/// sentence claims.
const EARLY_TEXT_JS: &str = r#"
  const fold = window.innerHeight;
  if (!fold) return 0;
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  let total = 0;
  let node;
  while ((node = walker.nextNode())) {
    const text = node.nodeValue ? node.nodeValue.trim() : '';
    if (!text) continue;
    const el = node.parentElement;
    if (!el) continue;
    const styles = window.getComputedStyle(el);
    if (styles.display === 'none' || styles.visibility === 'hidden') continue;
    if (parseFloat(styles.opacity) === 0) continue;
    // Screen-reader-only text is not text the visitor meets — same helper the
    // in-page WCAG checks use.
    if (__amsIsVisuallyHidden(el)) continue;
    let rect;
    try { rect = el.getBoundingClientRect(); } catch (e) { continue; }
    if (rect.width === 0 || rect.height === 0) continue;
    // Document coordinates, not viewport ones: another module may have
    // scrolled the page before this runs, and "the first viewport height"
    // has to mean the top of the document either way.
    const top = rect.top + window.scrollY;
    if (top >= fold) continue;
    total += text.length;
  }
  return total;
"#;

/// `<main>` in the DOM when the accessibility tree shows none.
pub(crate) static DOM_MAIN: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "journey.dom_main",
        module: "journey",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: false,
    };

/// Visible text length above the fold.
pub(crate) static EARLY_TEXT: crate::audit::collection::CollectorSpec =
    crate::audit::collection::CollectorSpec {
        id: "journey.early_text",
        module: "journey",
        method: crate::audit::collection::CollectionMethod::JavaScript,
        method_version: 1,
        scoring: false,
    };

/// Counter-cases of [`DOM_MAIN`] and [`EARLY_TEXT`] (#777): a missing or
/// mistyped answer is a failure, not `false` or `0`.
#[cfg(test)]
pub(crate) fn counter_cases() {
    crate::audit::collection::assert_js_value_contract::<bool>(serde_json::json!(true));
    crate::audit::collection::assert_js_value_contract::<u64>(serde_json::json!(0));
}

pub struct JourneyModule;

#[async_trait]
impl AuditModule for JourneyModule {
    fn id(&self) -> &'static str {
        "journey"
    }

    fn label(&self) -> &'static str {
        "Journey"
    }

    fn is_enabled(&self, cfg: &PipelineConfig) -> bool {
        cfg.check_mobile || cfg.check_seo
    }

    async fn collect(&self, ctx: &ModuleContext<'_>) -> Result<ModuleData> {
        let ax_has_main = ctx
            .ax_tree
            .iter()
            .any(|n| n.role.as_deref() == Some("main"));
        // Without a DOM answer the accessibility tree's verdict stands: it is
        // what assistive technology sees, not an invented "no".
        let dom_has_main = if ax_has_main {
            true
        } else {
            ctx.collection
                .record(
                    &DOM_MAIN,
                    crate::audit::collection::evaluate_js_value::<bool>(
                        ctx.page,
                        "!!document.querySelector('main, [role=\"main\"]')",
                    )
                    .await,
                )
                .into_value()
                .unwrap_or(false)
        };
        // `None` when the page cannot answer: the fallback in
        // `analyze_entry_clarity` then applies, rather than a zero that would
        // read as "no text at the top".
        let early_text_js = [
            "(() => {",
            crate::accessibility::js_helpers::IS_VISUALLY_HIDDEN_JS,
            EARLY_TEXT_JS,
            "})();",
        ]
        .concat();
        let measured_early_text = ctx
            .collection
            .record(
                &EARLY_TEXT,
                crate::audit::collection::evaluate_js_value::<u64>(
                    ctx.page,
                    early_text_js.as_str(),
                )
                .await,
            )
            .into_value()
            .map(|len| len as usize);

        let journey =
            analyze_journey_with_page_context(ctx.ax_tree, dom_has_main, measured_early_text);
        Ok(ModuleData::Journey(Box::new(journey)))
    }
}
