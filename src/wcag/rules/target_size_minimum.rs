//! WCAG 2.5.8 Target Size (Minimum) (Level AA, WCAG 2.2)
//!
//! The size of the target for pointer inputs is at least 24 by 24 CSS pixels,
//! except where the target is a link in a sentence or block of text, where
//! it is spaced so that a 24 px circle around it touches no other target, or
//! where an equivalent link on the same page meets the minimum.

use chromiumoxide::Page;

use crate::cli::WcagLevel;
use crate::wcag::types::{Outcome, RuleMetadata, Severity, Violation};

pub(super) const TARGET_SIZE_MINIMUM_RULE: RuleMetadata = RuleMetadata {
    id: "2.5.8",
    name: "Target Size (Minimum)",
    level: WcagLevel::AA,
    severity: Severity::Medium,
    description: "Interactive targets are at least 24×24 CSS pixels",
    help_url: "https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html",
    axe_id: "target-size-minimum",
    tags: &["wcag22aa", "wcag258", "cat.sensory-and-visual-cues"],
};

/// Shared with 2.5.5: whether a target is "in a sentence or block of text" —
/// the inline exception both criteria grant. The target is inline, and the
/// line it sits in carries text that is not itself a target: the inline
/// content of its block (text nodes and inline elements, not nested blocks),
/// minus every link and button. Comparing against the block's whole text was
/// wrong — a skip link directly under `<body>` counted as "in running text"
/// because the body holds the whole page, and links side by side in a `<nav>`
/// counted each other's labels as surrounding text.
///
/// Also shared: the "Equivalent" exception both criteria grant, for links only
/// (#652). An undersized link passes when another link on the page leads to
/// the same destination and itself meets the size (`minSize`), is rendered and
/// visible, has pointer events, and sits outside `inert` and
/// `aria-hidden="true"` subtrees. The destination is the resolved absolute
/// URL. For a link into another document the fragment is dropped: `/de/` and
/// `/de/#modules` both open the start page, the fragment only sets the scroll
/// position. For a link into the current page the fragment is kept, because
/// scrolling to that spot is all the link does — `#a` and `#b` differ.
/// `href=""`, `href="#"` and `javascript:` links never count — they name no
/// destination, so equal hrefs say nothing about equal function. Buttons are
/// out of scope: their function is not visible in the markup.
///
/// Also shared: which elements count as targets at all (#705). A target that
/// cannot be seen or clicked is neither measured nor a neighbour: hidden per
/// `checkVisibility` (a closed `<details>` keeps its content's boxes but hides
/// it via `content-visibility: hidden` — geographia.eu got three 2.5.8 findings
/// from such neighbours), inside `inert`, or with `pointer-events: none`.
/// Browsers without `checkVisibility` keep the size-only filter.
///
/// And: whether a target is still animating (#706). The pipeline waits for
/// finite animations first (`wait_for_finite_animations`); a target whose
/// element or ancestor still runs a finite animation or transition after that
/// is reported as not measured instead of with its mid-animation size.
/// Infinite animations are ignored, they never settle.
pub(super) const TARGET_HELPERS_JS: &str = r#"
function isInlineInText(el) {
  if (getComputedStyle(el).display !== 'inline') return false;
  var block = el.parentElement;
  while (block && getComputedStyle(block).display === 'inline') block = block.parentElement;
  if (!block) return false;
  var isTarget = function(n) {
    return n.matches && n.matches('a[href], button, [role="button"], [role="link"], input');
  };
  var text = '';
  (function collect(node) {
    for (var c = node.firstChild; c; c = c.nextSibling) {
      if (c.nodeType === 3) { text += c.textContent; continue; }
      if (c.nodeType !== 1 || isTarget(c)) continue;
      if (getComputedStyle(c).display === 'inline') collect(c);
    }
  })(block);
  return text.replace(/\s+/g, '').length > 0;
}

var linksByHref = null;
function linkDestination(el) {
  if (el.tagName !== 'A' || !el.hasAttribute('href')) return null;
  var raw = el.getAttribute('href').trim();
  if (raw === '' || raw === '#' || /^javascript:/i.test(raw)) return null;
  var doc = el.href.split('#')[0];
  return doc === location.href.split('#')[0] ? el.href : doc;
}
function hasEquivalentLink(el, minSize) {
  var dest = linkDestination(el);
  if (!dest) return false;
  if (!linksByHref) {
    linksByHref = {};
    var links = document.querySelectorAll('a[href]');
    for (var i = 0; i < links.length; i++) {
      var d = linkDestination(links[i]);
      if (d) (linksByHref[d] = linksByHref[d] || []).push(links[i]);
    }
  }
  var same = linksByHref[dest] || [];
  for (var k = 0; k < same.length; k++) {
    var o = same[k];
    if (o === el || o.contains(el) || el.contains(o)) continue;
    if (o.closest('[inert], [aria-hidden="true"]')) continue;
    if (!o.checkVisibility({ opacityProperty: true, visibilityProperty: true })) continue;
    if (getComputedStyle(o).pointerEvents === 'none') continue;
    var r = o.getBoundingClientRect();
    if (r.right + window.scrollX <= 0 || r.bottom + window.scrollY <= 0) continue;
    if (r.width >= minSize && r.height >= minSize) return true;
  }
  return false;
}

function isPointerTarget(el) {
  if (el.closest('[inert]')) return false;
  if (typeof el.checkVisibility === 'function' &&
      // Ohne contentVisibilityAuto: Abschnitte mit `content-visibility: auto`
      // ausserhalb des Viewports sind da und bedienbar, sie wuerden sonst
      // nicht gemessen. Das geschlossene <details> faengt checkVisibility
      // auch so ab (::details-content ist `content-visibility: hidden`).
      !el.checkVisibility({ opacityProperty: true, visibilityProperty: true })) return false;
  return getComputedStyle(el).pointerEvents !== 'none';
}

var animatingRoots = null;
function isAnimating(el) {
  if (!animatingRoots) {
    animatingRoots = [];
    var anims = typeof document.getAnimations === 'function' ? document.getAnimations() : [];
    for (var i = 0; i < anims.length; i++) {
      var a = anims[i];
      if (a.playState !== 'running' || !a.effect || !a.effect.target) continue;
      if (!isFinite(a.effect.getComputedTiming().endTime)) continue;
      animatingRoots.push(a.effect.target);
    }
  }
  for (var k = 0; k < animatingRoots.length; k++) {
    if (animatingRoots[k].contains(el)) return true;
  }
  return false;
}

"#;

/// Undersized targets pass when the spacing exception holds: a 24 px circle
/// centred on the target intersects no other target and no other undersized
/// target's circle. Before, every lone small button and every link in running
/// text was reported — neither is a 2.5.8 failure. Undersized links also pass
/// when an equivalent same-destination link meets 24×24 (`hasEquivalentLink`).
const TARGET_SIZE_JS: &str = r#"
(function() {
  var MIN_SIZE = 24;
  var RADIUS = MIN_SIZE / 2;
  /*TARGET_HELPERS*/
  var selectors = 'button, a[href], [role="button"], [role="link"], input[type="submit"], input[type="button"], input[type="reset"]';
  var targets = [];
  var all = document.querySelectorAll(selectors);
  for (var i = 0; i < all.length && __amsReal(targets) < 1000; i++) {
    var r = all[i].getBoundingClientRect();
    // A near-zero rect (commonly exactly 1x1) is the standard "visually
    // hidden until focus" CSS technique for things like skip links — not
    // actually laid out/visible, or intentionally not a pointer target in its
    // resting state. A genuinely too-small but real button/icon is virtually
    // never this tiny, so this threshold doesn't mask real violations.
    if (r.width <= 2 || r.height <= 2 || !isPointerTarget(all[i])) continue;
    __amsPush(targets, all[i], { el: all[i], rect: r, cx: r.left + r.width / 2, cy: r.top + r.height / 2,
                   small: r.width < MIN_SIZE || r.height < MIN_SIZE }, 1000);
  }
  function distToRect(x, y, r) {
    var dx = Math.max(r.left - x, 0, x - r.right);
    var dy = Math.max(r.top - y, 0, y - r.bottom);
    return Math.sqrt(dx * dx + dy * dy);
  }
  function spaced(t) {
    for (var k = 0; k < targets.length; k++) {
      var o = targets[k];
      if (o === t || o.el.contains(t.el) || t.el.contains(o.el)) continue;
      // The circle may touch no other target at all, and — if that target
      // is undersized too — not its circle either.
      if (distToRect(t.cx, t.cy, o.rect) < RADIUS) return false;
      if (o.small && Math.hypot(t.cx - o.cx, t.cy - o.cy) < 2 * RADIUS) return false;
    }
    return true;
  }
  var violations = [];
  var animating = [];
  for (var j = 0; j < targets.length && __amsReal(violations) < 5; j++) {
    var t = targets[j];
    if (!t.small || isInlineInText(t.el) || spaced(t) || hasEquivalentLink(t.el, MIN_SIZE)) continue;
    var el = t.el;
    var desc = el.getAttribute('aria-label') || el.textContent.trim().substring(0, 40) || el.tagName.toLowerCase();
    if (isAnimating(el)) {
      __amsPush(animating, el, { selector: __amsCssSelector(el), label: desc }, 5);
      continue;
    }
    __amsPush(violations, el, {
      selector: __amsCssSelector(el),
      label: desc,
      width: Math.round(t.rect.width),
      height: Math.round(t.rect.height)
    }, 5);
  }
  return { violations: violations, animating: animating };
})()
"#;

pub async fn check_target_size_minimum_with_page(page: &Page) -> Vec<Violation> {
    let val = match crate::wcag::types::evaluate_or_fail(
        page,
        &TARGET_SIZE_MINIMUM_RULE,
        &TARGET_SIZE_JS.replace(
            "/*TARGET_HELPERS*/",
            &[
                TARGET_HELPERS_JS,
                crate::accessibility::js_helpers::CSS_SELECTOR_JS,
            ]
            .concat(),
        ),
    )
    .await
    {
        Ok(v) => v,
        Err(violations) => return violations,
    };

    let violations = match val.get("violations").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => {
            return vec![crate::wcag::technical_rule_failure(
                &TARGET_SIZE_MINIMUM_RULE,
                "invalid_evaluation_shape",
            )]
        }
    };

    let mut findings = animating_findings(&TARGET_SIZE_MINIMUM_RULE, &val);
    findings.extend(violations.iter().map(|item| {
        let selector = item
            .get("selector")
            .and_then(|v| v.as_str())
            .unwrap_or("element");
        let label = item
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("element");
        let width = item.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
        let height = item.get("height").and_then(|v| v.as_u64()).unwrap_or(0);

        Violation::new(
            TARGET_SIZE_MINIMUM_RULE.id,
            TARGET_SIZE_MINIMUM_RULE.name,
            TARGET_SIZE_MINIMUM_RULE.level,
            Severity::Medium,
            format!(
                "Interactive target '{}' is {}×{} CSS pixels, below the 24×24 minimum.",
                label, width, height
            ),
            selector,
        )
        .with_selector(selector)
        .with_fix(
            "Increase the target size to at least 24×24 CSS pixels using padding, \
                 min-width/min-height, or by enlarging the element.",
        )
        .with_rule_id(TARGET_SIZE_MINIMUM_RULE.axe_id)
        .with_help_url(TARGET_SIZE_MINIMUM_RULE.help_url)
    }));
    findings
}

/// Undersized targets that were still animating when measured (#706): the
/// size is a snapshot of the animation, not the target's, so they come back
/// as `Untested` rather than as a failure — "not measured" is not "passed".
pub(super) fn animating_findings(rule: &RuleMetadata, val: &serde_json::Value) -> Vec<Violation> {
    let Some(items) = val.get("animating").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    items
        .iter()
        .map(|item| {
            let selector = item
                .get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("element");
            let label = item
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("element");
            Violation::new(
                rule.id,
                rule.name,
                rule.level,
                Severity::Low,
                format!(
                    "Size of interactive target '{}' was not measured: it was still animating \
                     when the check ran.",
                    label
                ),
                selector,
            )
            .with_selector(selector)
            .with_fix(
                "Check the target's size once its animation has finished, or audit with \
                 prefers-reduced-motion: reduce if the page skips the animation then.",
            )
            .with_rule_id(rule.axe_id)
            .with_help_url(rule.help_url)
            .with_kind(Outcome::Untested)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn animating_targets_are_untested_not_failed() {
        let val = json!({
            "violations": [],
            "animating": [{ "selector": "a.logo-module", "label": "Atmosphere" }]
        });
        let findings = animating_findings(&TARGET_SIZE_MINIMUM_RULE, &val);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, Outcome::Untested);
        assert_eq!(findings[0].selector.as_deref(), Some("a.logo-module"));
        assert_eq!(
            findings[0].rule_id.as_deref(),
            Some(TARGET_SIZE_MINIMUM_RULE.axe_id)
        );
        assert!(findings[0].message.contains("Atmosphere"));
    }

    #[test]
    fn missing_animating_list_yields_nothing() {
        assert!(
            animating_findings(&TARGET_SIZE_MINIMUM_RULE, &json!({ "violations": [] })).is_empty()
        );
    }
}
