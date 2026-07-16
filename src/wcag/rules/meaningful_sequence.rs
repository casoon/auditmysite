//! WCAG 1.3.2 Meaningful Sequence (Level A)
//!
//! "When the sequence in which content is presented affects its meaning, a
//! correct reading sequence can be programmatically determined."
//!
//! In practice this means: a sighted user's visual reading order and a
//! screen-reader user's announced reading order must match whenever the
//! order carries meaning.
//!
//! **Why this isn't an AX-tree-vs-DOM-order diff.** The obvious-looking
//! approach — walk the accessibility tree (`crate::screen_reader::linearizer`)
//! and compare it against DOM source order — was investigated first. Chrome
//! builds its accessibility tree from DOM structure and mirrors DOM/source
//! order regardless of CSS visual reordering (`order`, `position: absolute`,
//! `float`), unless `aria-owns` is used to explicitly relink the tree. So an
//! AX-order-vs-DOM-order diff would (almost) never fire, and would not catch
//! any of the real-world 1.3.2 anti-patterns — it would only catch `aria-owns`
//! misuse, which is a separate, much rarer case. That approach was dropped.
//!
//! **What this checks instead.** A visual-position-vs-DOM-order diff is the
//! defensible signal, but a fully general version of it is genuinely hard and
//! false-positive-prone: multi-column CSS, `float`, and responsive layouts
//! routinely produce a "different" visual vs. DOM order without being an
//! accessibility problem, since two-dimensional visual layout is not the same
//! thing as a linear reading sequence in most legitimate cases.
//!
//! This check is scoped narrowly to the one well-documented, low-false-positive
//! anti-pattern: **CSS flexbox `order` used to visually reorder children
//! without a matching `aria-owns` to keep the accessibility tree in sync.**
//! Per the flexbox spec, a flex container's *visual* order is determined by
//! sorting children by `order` (ties broken by DOM/source position) — so this
//! doesn't need to reimplement layout or read `getBoundingClientRect()`; it
//! only needs to compare each container's DOM child order against its
//! `order`-sorted order. If they diverge and there is no `aria-owns` on the
//! container, the visual sequence a sighted user perceives differs from the
//! sequence a screen reader announces.
//!
//! Deliberately **not** implemented:
//! - CSS Grid `order`/placement (grid's own explicit `grid-row`/`grid-column`
//!   placement, not `order`, usually governs visual position, making an
//!   `order`-only check unreliable there).
//! - `position: absolute`/`fixed` and `float`-based reordering (would require
//!   a real visual layout diff, not just a single CSS property read, and is
//!   exactly the false-positive-prone case called out above).
//!
//! Because "does the order actually affect meaning" requires human judgment
//! that a static check cannot make, findings are reported as
//! [`crate::wcag::types::FindingKind::Warning`] (heuristic, manual-review
//! candidate), not a confirmed violation — same treatment as
//! `redundant_entry` and the image-background contrast heuristic.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

/// One flex-container child, as extracted from the live DOM: its DOM/source
/// position and its computed `order` value (defaults to 0 when unset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChildOrder {
    dom_index: usize,
    order: i32,
}

/// Mirrors the in-browser JS: sorts children by `(order, dom_index)` — the
/// flexbox spec's own visual-ordering rule — and reports whether that visual
/// sequence diverges from DOM/source order for at least one child.
fn is_visually_reordered(children: &[ChildOrder]) -> bool {
    let mut visual_order: Vec<ChildOrder> = children.to_vec();
    visual_order.sort_by_key(|c| (c.order, c.dom_index));
    children
        .iter()
        .zip(visual_order.iter())
        .any(|(dom, visual)| dom.dom_index != visual.dom_index)
}

pub const MEANINGFUL_SEQUENCE_RULE: RuleMetadata = RuleMetadata {
    id: "1.3.2",
    name: "Meaningful Sequence",
    level: WcagLevel::A,
    severity: Severity::Medium,
    description:
        "The visual reading order matches the order content is exposed to assistive technology",
    help_url: "https://www.w3.org/WAI/WCAG21/Understanding/meaningful-sequence.html",
    axe_id: "css-order-reading-sequence",
    tags: &["wcag2a", "wcag132", "cat.structure"],
};

const MEANINGFUL_SEQUENCE_JS: &str = r#"
(function() {
  var results = [];
  var containers = document.querySelectorAll('*');

  function selectorFor(el) {
    var s = el.tagName.toLowerCase();
    if (el.id) s += '#' + el.id;
    return s;
  }

  for (var c = 0; c < containers.length && results.length < 5; c++) {
    var container = containers[c];
    var cs = getComputedStyle(container);
    if (cs.display !== 'flex' && cs.display !== 'inline-flex') continue;
    if (container.hasAttribute('aria-owns')) continue;

    var rect = container.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) continue;

    var children = [];
    for (var i = 0; i < container.children.length; i++) {
      var child = container.children[i];
      var childCs = getComputedStyle(child);
      if (childCs.display === 'none' || childCs.visibility === 'hidden') continue;
      var order = parseInt(childCs.order, 10);
      if (isNaN(order)) order = 0;
      children.push({ selector: selectorFor(child), order: order });
    }
    if (children.length < 2) continue;

    results.push({
      selector: selectorFor(container),
      children: children
    });
  }

  return { results: results };
})()
"#;

pub async fn check_meaningful_sequence_with_page(page: &Page) -> Vec<Violation> {
    let result = match page.evaluate(MEANINGFUL_SEQUENCE_JS).await {
        Ok(r) => r,
        Err(_) => {
            return vec![crate::wcag::technical_rule_failure(
                &MEANINGFUL_SEQUENCE_RULE,
                "page_evaluation_failed",
            )]
        }
    };

    let val = match result.value() {
        Some(v) => v.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &MEANINGFUL_SEQUENCE_RULE,
                "missing_evaluation_value",
            )]
        }
    };

    let entries = match val.get("results").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &MEANINGFUL_SEQUENCE_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    entries
        .iter()
        .filter_map(|entry| {
            let selector = entry.get("selector").and_then(|v| v.as_str())?;
            let raw_children = entry.get("children").and_then(|v| v.as_array())?;

            let child_selectors: Vec<&str> = raw_children
                .iter()
                .filter_map(|c| c.get("selector").and_then(|v| v.as_str()))
                .collect();
            let child_orders: Vec<ChildOrder> = raw_children
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    let order = c.get("order").and_then(|v| v.as_i64())? as i32;
                    Some(ChildOrder {
                        dom_index: i,
                        order,
                    })
                })
                .collect();

            if child_orders.len() != child_selectors.len() || !is_visually_reordered(&child_orders)
            {
                return None;
            }

            let mut visual_order = child_orders.clone();
            visual_order.sort_by_key(|c| (c.order, c.dom_index));

            let dom_sequence = child_selectors.join(", ");
            let visual_sequence = visual_order
                .iter()
                .map(|c| child_selectors[c.dom_index])
                .collect::<Vec<_>>()
                .join(", ");

            Some(
                Violation::new(
                    MEANINGFUL_SEQUENCE_RULE.id,
                    MEANINGFUL_SEQUENCE_RULE.name,
                    MEANINGFUL_SEQUENCE_RULE.level,
                    MEANINGFUL_SEQUENCE_RULE.severity,
                    format!(
                        "Flex container '{}' uses CSS `order` to visually reorder its children \
                         (DOM order: {}; visual order: {}) without an `aria-owns` attribute to \
                         keep the accessibility tree's reading order in sync.",
                        selector, dom_sequence, visual_sequence
                    ),
                    selector,
                )
                .with_selector(selector)
                .with_fix(
                    "If the order carries meaning, reorder the elements in the DOM/source instead \
                     of using CSS `order`, or add `aria-owns` on the container listing the children \
                     in their visual order so assistive technology announces the same sequence.",
                )
                .with_rule_id(MEANINGFUL_SEQUENCE_RULE.axe_id)
                .with_help_url(MEANINGFUL_SEQUENCE_RULE.help_url)
                .as_warning(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata_is_level_a() {
        assert_eq!(MEANINGFUL_SEQUENCE_RULE.id, "1.3.2");
        assert_eq!(MEANINGFUL_SEQUENCE_RULE.level, WcagLevel::A);
    }

    fn child(dom_index: usize, order: i32) -> ChildOrder {
        ChildOrder { dom_index, order }
    }

    #[test]
    fn fires_when_css_order_swaps_sibling_sequence() {
        // Three children in DOM order 0,1,2 — the middle one is pulled first
        // visually via `order: -1`, so a sighted user sees it before its
        // DOM-preceding sibling while a screen reader still announces DOM
        // order first.
        let children = vec![child(0, 0), child(1, -1), child(2, 0)];
        assert!(is_visually_reordered(&children));
    }

    #[test]
    fn does_not_fire_for_uniform_or_monotonically_increasing_order() {
        // All children share the default order (0) — the ordinary,
        // overwhelmingly common case (no `order` used at all).
        let all_default = vec![child(0, 0), child(1, 0), child(2, 0)];
        assert!(!is_visually_reordered(&all_default));

        // Distinct but monotonically increasing order values that match DOM
        // position 1:1 don't change the sequence, even though `order` is set.
        let monotonic = vec![child(0, 1), child(1, 2), child(2, 3)];
        assert!(!is_visually_reordered(&monotonic));
    }
}
