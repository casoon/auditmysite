//! WCAG 2.4.11 Focus Not Obscured (Minimum) (Level AA, WCAG 2.2)
//!
//! "When a user interface component receives keyboard focus, the component
//! is not entirely hidden due to author-created content."
//!
//! In practice: sticky/fixed headers, footers, cookie banners, or chat
//! widgets can sit on top of page content, so a keyboard user tabbing to a
//! control underneath one of these can land on something the CDP/DOM still
//! considers "focused" but that is no longer visible at all.
//!
//! **What this checks.** This is a static-layout, single-page-load
//! geometric heuristic, not a real focus-walk: it does *not* press Tab and
//! inspect `document.activeElement` for every control (that would require a
//! CDP round-trip per focusable element and does not by itself simulate the
//! post-scroll layout that triggers the classic version of this bug). One
//! `evaluate()` call collects (a) all `position: fixed`/`sticky` elements
//! large enough to plausibly be a header/footer/banner/widget, and (b) all
//! natively-focusable elements, both in their *current* bounding-box
//! position. The pure Rust half then checks whether any overlay's rect
//! covers essentially the *entire* area of a focusable element's rect.
//!
//! This narrower, static-snapshot version does catch real, permanent
//! instances of the bug (an overlay that structurally always sits on top of
//! some focusable control regardless of scroll position — e.g. a
//! badly-positioned fixed newsletter/chat widget, or a fixed bottom banner
//! overlapping the last footer links), but it will **not** catch the
//! textbook variant where an element is only obscured *after* the user
//! scrolls it under a sticky header, since that requires actually walking
//! focus and observing the resulting scroll position, which is out of scope
//! here.
//!
//! Deliberately excluded to avoid false positives:
//! - An overlay that is an *ancestor* of the focusable element itself (a
//!   link inside a sticky header is not "obscured by its own header").
//! - Fixed/sticky elements that are hidden (`display: none`,
//!   `visibility: hidden`, `opacity: 0`) or too small to plausibly be an
//!   overlay (`< 20px` in either dimension).
//! - Overlap ratios below "entirely hidden" (see [`ENTIRELY_HIDDEN_RATIO`])
//!   — a merely partially-covered element still satisfies this (Minimum)
//!   criterion; see `focus_not_obscured_enhanced` (2.4.12, AAA) for the
//!   stricter "no part hidden at all" version.
//!
//! **Not checked:** paint/stacking order (`z-index`). A geometric rect
//! overlap does not guarantee the overlay actually paints on top of the
//! focusable element — a `position: fixed` element with a lower stacking
//! context could sit fully behind it. Combined with the static-snapshot
//! limitation above, findings are reported as
//! [`crate::wcag::types::FindingKind::Warning`] (heuristic, manual-review
//! candidate), matching the precedent set by `redundant_entry.rs` and
//! `meaningful_sequence.rs` for this kind of geometry-based judgment call.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{RuleMetadata, Severity, Violation};

pub const FOCUS_NOT_OBSCURED_MINIMUM_RULE: RuleMetadata = RuleMetadata {
    id: "2.4.11",
    name: "Focus Not Obscured (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description:
        "Focusable elements are not entirely hidden behind fixed or sticky-positioned content",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html",
    axe_id: "focus-not-obscured-minimum",
    tags: &["wcag22aa", "wcag2411", "cat.keyboard"],
};

/// Fraction of a focusable element's bounding-box area that a fixed/sticky
/// overlay must cover for the element to be considered "entirely hidden"
/// per 2.4.11. Slightly below 1.0 to tolerate the sub-pixel rounding
/// `getBoundingClientRect()` commonly produces between visually-flush edges.
const ENTIRELY_HIDDEN_RATIO: f64 = 0.98;

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

/// True if any overlay rect covers at least [`ENTIRELY_HIDDEN_RATIO`] of the
/// focusable element's area.
fn is_entirely_hidden(focusable: Rect, overlays: &[Rect]) -> bool {
    overlays
        .iter()
        .any(|overlay| overlap_ratio(focusable, *overlay) >= ENTIRELY_HIDDEN_RATIO)
}

/// Collects, in one evaluation: (a) sizeable `position: fixed`/`sticky`
/// elements ("overlays"), and (b) natively-focusable elements with, for
/// each, the indices of overlays that are *not* one of its own DOM
/// ancestors (candidates that could actually obscure it).
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

pub async fn check_focus_not_obscured_minimum_with_page(page: &Page) -> Vec<Violation> {
    if let Some(findings) = focus_walk_findings(page).await {
        return findings;
    }
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &FOCUS_NOT_OBSCURED_MINIMUM_RULE,
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
                &FOCUS_NOT_OBSCURED_MINIMUM_RULE,
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
                &FOCUS_NOT_OBSCURED_MINIMUM_RULE,
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

        if is_entirely_hidden(rect, &candidate_overlays) {
            violations.push(
                Violation::new(
                    FOCUS_NOT_OBSCURED_MINIMUM_RULE.id,
                    FOCUS_NOT_OBSCURED_MINIMUM_RULE.name,
                    FOCUS_NOT_OBSCURED_MINIMUM_RULE.level,
                    Severity::Medium,
                    format!(
                        "Focusable element '{}' is entirely covered by a fixed or sticky-positioned \
                         element at its current layout position, so a keyboard user tabbing to it \
                         would not be able to see it.",
                        selector
                    ),
                    selector,
                )
                .with_selector(selector)
                .with_fix(
                    "Make sure focused elements stay visible: add `scroll-margin-top`/`scroll-padding-top` \
                     so sticky headers don't cover the target after it scrolls into view, or reposition/resize \
                     the fixed or sticky element so it never fully covers interactive content.",
                )
                .with_rule_id(FOCUS_NOT_OBSCURED_MINIMUM_RULE.axe_id)
                .with_help_url(FOCUS_NOT_OBSCURED_MINIMUM_RULE.help_url)
                .as_warning(),
            );
            if violations.len() >= 5 {
                break;
            }
        }
    }
    violations
}

/// Bounded focus/scroll walk. `elementsFromPoint` supplies the browser's paint
/// order, avoiding geometric false positives from overlays that are behind the
/// focused control. The previous static geometry path remains a fallback when
/// this live probe cannot execute.
async fn focus_walk_findings(page: &Page) -> Option<Vec<Violation>> {
    let value = page
        .evaluate(
            r#"
            (() => {
              const originalX = scrollX, originalY = scrollY, original = document.activeElement;
              const selectorFor = element => element.id
                ? `${element.tagName.toLowerCase()}#${CSS.escape(element.id)}`
                : element.tagName.toLowerCase();
              const selector = 'a[href],button,input:not([type="hidden"]),select,textarea,[tabindex]:not([tabindex="-1"]),[contenteditable="true"]';
              const controls = [...document.querySelectorAll(selector)].filter(element => {
                const style = getComputedStyle(element);
                const rect = element.getBoundingClientRect();
                return !element.disabled && style.display !== 'none' && style.visibility !== 'hidden' && rect.width > 3 && rect.height > 3;
              }).slice(0, 60);
              const findings = [];
              for (const control of controls) {
                control.focus({preventScroll: true});
                control.scrollIntoView({block: 'nearest', inline: 'nearest'});
                const rect = control.getBoundingClientRect();
                const points = [
                  [rect.left + 1, rect.top + 1], [rect.right - 1, rect.top + 1],
                  [rect.left + 1, rect.bottom - 1], [rect.right - 1, rect.bottom - 1],
                  [rect.left + rect.width / 2, rect.top + rect.height / 2]
                ].filter(([x,y]) => x >= 0 && y >= 0 && x < innerWidth && y < innerHeight);
                if (!points.length) continue;
                const blockers = points.map(([x,y]) => {
                  const painted = document.elementsFromPoint(x, y);
                  const controlIndex = painted.findIndex(element => element === control || control.contains(element));
                  if (controlIndex <= 0) return null;
                  return painted.slice(0, controlIndex).find(element => {
                    if (element.contains(control) || control.contains(element)) return false;
                    const style = getComputedStyle(element);
                    return style.position === 'fixed' || style.position === 'sticky';
                  }) || null;
                });
                if (blockers.every(Boolean)) {
                  findings.push({control: selectorFor(control), blocker: selectorFor(blockers[0])});
                  if (findings.length >= 5) break;
                }
              }
              scrollTo(originalX, originalY);
              if (original && original.focus) original.focus({preventScroll:true});
              return { findings };
            })()
            "#,
        )
        .await
        .ok()?
        .value()
        .cloned()?;
    let findings = value.get("findings")?.as_array()?;
    Some(
        findings
            .iter()
            .filter_map(|finding| {
                let selector = finding.get("control")?.as_str()?;
                let blocker = finding.get("blocker")?.as_str()?;
                Some(
                    Violation::new(
                        FOCUS_NOT_OBSCURED_MINIMUM_RULE.id,
                        FOCUS_NOT_OBSCURED_MINIMUM_RULE.name,
                        FOCUS_NOT_OBSCURED_MINIMUM_RULE.level,
                        FOCUS_NOT_OBSCURED_MINIMUM_RULE.severity,
                        format!(
                            "Keyboard focus on {selector} is entirely hidden in paint order by fixed or sticky element {blocker} after scrolling it into view."
                        ),
                        selector,
                    )
                    .with_selector(selector)
                    .with_rule_id(FOCUS_NOT_OBSCURED_MINIMUM_RULE.axe_id)
                    .with_kind(crate::wcag::types::FindingKind::Warning)
                    .with_fix(
                        "Reserve space for sticky content or add scroll-padding/scroll-margin so focused controls remain visible",
                    )
                    .with_help_url(FOCUS_NOT_OBSCURED_MINIMUM_RULE.help_url),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn fires_when_focusable_entirely_covered_by_sticky_header() {
        // A form field scrolled to sit right under a full-width sticky
        // header — the classic real-world 2.4.11 scenario.
        let focusable = rect(100.0, 10.0, 200.0, 30.0);
        let header = rect(0.0, 0.0, 1200.0, 60.0);
        assert!(is_entirely_hidden(focusable, &[header]));
    }

    #[test]
    fn does_not_fire_when_no_overlap() {
        let focusable = rect(100.0, 500.0, 200.0, 30.0);
        let header = rect(0.0, 0.0, 1200.0, 60.0);
        assert!(!is_entirely_hidden(focusable, &[header]));
    }

    #[test]
    fn does_not_fire_for_partial_edge_touch_overlap() {
        // Sticky header only clips the top 2px of a 30px-tall field — well
        // below the "entirely hidden" bar, and exactly the kind of
        // near-miss this rule must not flag.
        let focusable = rect(100.0, 58.0, 200.0, 30.0);
        let header = rect(0.0, 0.0, 1200.0, 60.0);
        assert!(!is_entirely_hidden(focusable, &[header]));
    }

    #[test]
    fn does_not_fire_when_own_container_excluded_leaves_no_candidates() {
        // Models the effect of the JS-side ancestor exclusion: a link
        // inside a sticky header has no candidate overlays left once its
        // own header is filtered out, so the geometric check never runs
        // against it (an empty overlay list can never look "hidden").
        let focusable = rect(20.0, 15.0, 80.0, 20.0);
        assert!(!is_entirely_hidden(focusable, &[]));
    }

    #[test]
    fn rule_metadata_is_level_aa() {
        assert_eq!(FOCUS_NOT_OBSCURED_MINIMUM_RULE.id, "2.4.11");
        assert_eq!(FOCUS_NOT_OBSCURED_MINIMUM_RULE.level, WcagLevel::AA);
    }
}
