//! WCAG 2.4.12 Focus Not Obscured (Enhanced) (Level AAA, WCAG 2.2)
//!
//! "When a user interface component receives keyboard focus, no part of the
//! component is hidden by author-created content."
//!
//! The stricter sibling of 2.4.11 (`focus_not_obscured_minimum`, Level AA):
//! that criterion only fails when a focused component is *entirely* hidden,
//! this one fails as soon as *any* meaningful part of it is covered. Both
//! rules share the same static-snapshot geometric approach and the same
//! caveats (no real focus-walk/scroll simulation, no paint/stacking-order
//! check) — see `focus_not_obscured_minimum` for the full rationale. Only
//! the overlap threshold and severity/level differ, matching how
//! `target_size_minimum` (2.5.8, AA) and `target_size_enhanced` (2.5.5, AAA)
//! are already split into two independent rule files in this codebase.
//!
//! Because a fully-hidden element (2.4.11 violation) is by definition also
//! partially hidden, the same element can legitimately show up as both a
//! 2.4.11 and a 2.4.12 finding when the audit runs at WCAG level AAA.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const FOCUS_NOT_OBSCURED_ENHANCED_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.12",
    name: "Focus Not Obscured (Enhanced)",
    level: WcagLevel::AAA,
    severity: Severity::Medium,
    description:
        "No part of a focusable element is hidden behind fixed or sticky-positioned content",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-enhanced.html",
    axe_id: "focus-not-obscured-enhanced",
    tags: &["wcag22aaa", "wcag2412", "cat.keyboard"],
};

/// Minimum fraction of a focusable element's bounding-box area that a
/// fixed/sticky overlay must cover to count as "part of it hidden" per
/// 2.4.12. Not `> 0.0` to avoid flagging 1-2px edge touches between
/// visually-flush but non-overlapping elements as a violation.
const PARTIALLY_HIDDEN_RATIO: f64 = 0.05;

/// A CSS-pixel bounding box, as returned by `getBoundingClientRect()`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Fraction of `focusable`'s area covered by `overlay`'s rect (0.0 when they
/// don't overlap at all).
fn overlap_ratio(focusable: Rect, overlay: Rect) -> f64 {
    let x_overlap =
        (focusable.x + focusable.w).min(overlay.x + overlay.w) - focusable.x.max(overlay.x);
    let y_overlap =
        (focusable.y + focusable.h).min(overlay.y + overlay.h) - focusable.y.max(overlay.y);
    if x_overlap <= 0.0 || y_overlap <= 0.0 {
        return 0.0;
    }
    let focusable_area = focusable.w * focusable.h;
    if focusable_area <= 0.0 {
        return 0.0;
    }
    (x_overlap * y_overlap) / focusable_area
}

/// True if any overlay rect covers at least [`PARTIALLY_HIDDEN_RATIO`] of
/// the focusable element's area.
fn is_partially_hidden(focusable: Rect, overlays: &[Rect]) -> bool {
    overlays
        .iter()
        .any(|overlay| overlap_ratio(focusable, *overlay) >= PARTIALLY_HIDDEN_RATIO)
}

/// Same collection as `focus_not_obscured_minimum::FOCUS_OBSCURED_JS` — kept
/// as an independent copy per this codebase's existing
/// target-size-minimum/target-size-enhanced precedent (self-contained rule
/// files rather than a shared JS constant).
const FOCUS_OBSCURED_JS: &str = r#"
(function() {
  function selectorFor(el) {
    var s = el.tagName.toLowerCase();
    if (el.id) s += '#' + el.id;
    return s;
  }

  var overlays = [];
  var overlayEls = [];
  var all = document.querySelectorAll('*');
  for (var i = 0; i < all.length && overlays.length < 30; i++) {
    var el = all[i];
    var cs = getComputedStyle(el);
    if (cs.position !== 'fixed' && cs.position !== 'sticky') continue;
    if (cs.visibility === 'hidden' || cs.display === 'none' || parseFloat(cs.opacity) === 0) continue;
    var rect = el.getBoundingClientRect();
    if (rect.width < 20 || rect.height < 20) continue;
    overlays.push({ selector: selectorFor(el), x: rect.x, y: rect.y, w: rect.width, h: rect.height });
    overlayEls.push(el);
  }
  if (overlays.length === 0) return { overlays: [], focusables: [] };

  var sel = 'a[href], button, input:not([type="hidden"]), select, textarea, ' +
            '[tabindex], [contenteditable=""], [contenteditable="true"]';
  var els = document.querySelectorAll(sel);
  var focusables = [];
  for (var j = 0; j < els.length && focusables.length < 200; j++) {
    var fel = els[j];
    if (fel.disabled) continue;
    if (typeof fel.tabIndex === 'number' && fel.tabIndex < 0) continue;
    var fcs = getComputedStyle(fel);
    if (fcs.display === 'none' || fcs.visibility === 'hidden') continue;
    var frect = fel.getBoundingClientRect();
    if (frect.width <= 0 || frect.height <= 0) continue;

    var candidates = [];
    for (var k = 0; k < overlayEls.length; k++) {
      if (overlayEls[k] === fel || overlayEls[k].contains(fel)) continue;
      candidates.push(k);
    }
    if (candidates.length === 0) continue;

    focusables.push({
      selector: selectorFor(fel),
      x: frect.x, y: frect.y, w: frect.width, h: frect.height,
      overlayIndices: candidates
    });
  }

  return { overlays: overlays, focusables: focusables };
})()
"#;

pub async fn check_focus_not_obscured_enhanced_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &FOCUS_NOT_OBSCURED_ENHANCED_RULE,
        FOCUS_OBSCURED_JS,
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let overlays_json = match val.get("overlays").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &FOCUS_NOT_OBSCURED_ENHANCED_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };
    if overlays_json.is_empty() {
        return vec![];
    }
    let overlays: Vec<Rect> = overlays_json
        .iter()
        .filter_map(|o| {
            Some(Rect {
                x: o.get("x")?.as_f64()?,
                y: o.get("y")?.as_f64()?,
                w: o.get("w")?.as_f64()?,
                h: o.get("h")?.as_f64()?,
            })
        })
        .collect();

    let focusables_json = match val.get("focusables").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &FOCUS_NOT_OBSCURED_ENHANCED_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    let mut violations = Vec::new();
    for f in &focusables_json {
        let Some(selector) = f.get("selector").and_then(|v| v.as_str()) else {
            continue;
        };
        let (Some(x), Some(y), Some(w), Some(h)) = (
            f.get("x").and_then(|v| v.as_f64()),
            f.get("y").and_then(|v| v.as_f64()),
            f.get("w").and_then(|v| v.as_f64()),
            f.get("h").and_then(|v| v.as_f64()),
        ) else {
            continue;
        };
        let rect = Rect { x, y, w, h };

        let indices: Vec<usize> = f
            .get("overlayIndices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|i| i.as_u64().map(|n| n as usize))
                    .collect()
            })
            .unwrap_or_default();
        let candidate_overlays: Vec<Rect> = indices
            .iter()
            .filter_map(|i| overlays.get(*i).copied())
            .collect();
        if candidate_overlays.is_empty() {
            continue;
        }

        if is_partially_hidden(rect, &candidate_overlays) {
            violations.push(
                Violation::new(
                    FOCUS_NOT_OBSCURED_ENHANCED_RULE.id,
                    FOCUS_NOT_OBSCURED_ENHANCED_RULE.name,
                    FOCUS_NOT_OBSCURED_ENHANCED_RULE.level,
                    Severity::Medium,
                    format!(
                        "Focusable element '{}' is at least partially covered by a fixed or \
                         sticky-positioned element at its current layout position, so a keyboard \
                         user tabbing to it would not be able to see the whole element.",
                        selector
                    ),
                    selector,
                )
                .with_selector(selector)
                .with_fix(
                    "Ensure no part of a focused element is ever covered: add `scroll-margin-top`/`scroll-padding-top` \
                     so sticky headers don't overlap the target after it scrolls into view, or reposition/resize \
                     the fixed or sticky element so it never overlaps interactive content.",
                )
                .with_rule_id(FOCUS_NOT_OBSCURED_ENHANCED_RULE.axe_id)
                .with_help_url(FOCUS_NOT_OBSCURED_ENHANCED_RULE.help_url)
                .as_warning(),
            );
            if violations.len() >= 5 {
                break;
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn fires_on_small_but_meaningful_overlap() {
        // Sticky header clips the top third of a focusable field — not
        // entirely hidden (would fail 2.4.11 too, but well above the
        // "no part hidden at all" bar for the Enhanced criterion).
        let focusable = rect(100.0, 50.0, 200.0, 30.0);
        let header = rect(0.0, 0.0, 1200.0, 60.0);
        assert!(is_partially_hidden(focusable, &[header]));
    }

    #[test]
    fn fires_when_entirely_hidden_too() {
        // An entirely-hidden element (2.4.11 case) is necessarily also
        // partially hidden, so 2.4.12 must fire for it as well.
        let focusable = rect(100.0, 10.0, 200.0, 30.0);
        let header = rect(0.0, 0.0, 1200.0, 60.0);
        assert!(is_partially_hidden(focusable, &[header]));
    }

    #[test]
    fn does_not_fire_when_no_overlap() {
        let focusable = rect(100.0, 500.0, 200.0, 30.0);
        let header = rect(0.0, 0.0, 1200.0, 60.0);
        assert!(!is_partially_hidden(focusable, &[header]));
    }

    #[test]
    fn does_not_fire_for_negligible_edge_touch_overlap() {
        // Only ~1px of a 30px-tall field is clipped — below the meaningful
        // overlap threshold, so this is treated as a rendering/rounding
        // artifact rather than a real obscuring.
        let focusable = rect(100.0, 59.0, 200.0, 30.0);
        let header = rect(0.0, 0.0, 1200.0, 60.0);
        assert!(!is_partially_hidden(focusable, &[header]));
    }

    #[test]
    fn rule_metadata_is_level_aaa() {
        assert_eq!(FOCUS_NOT_OBSCURED_ENHANCED_RULE.id, "2.4.12");
        assert_eq!(FOCUS_NOT_OBSCURED_ENHANCED_RULE.level, WcagLevel::AAA);
    }
}
