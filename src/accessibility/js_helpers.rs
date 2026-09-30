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
pub(crate) const CSS_SELECTOR_JS: &str = r#"
function __amsIsExcludedEl(el) {
  try {
    return !!(el && typeof window.__amsIsExcluded === 'function' && window.__amsIsExcluded(el));
  } catch (e) { return false; }
}
var __amsCapState;
function __amsCapCounts(list) {
  // Lazy: some rules call the helpers above the spot this snippet is spliced in.
  if (!__amsCapState) __amsCapState = new WeakMap();
  var s = __amsCapState.get(list);
  if (!s) { s = { real: 0, excluded: 0 }; __amsCapState.set(list, s); }
  return s;
}
function __amsPush(list, el, item, limit) {
  var s = __amsCapCounts(list);
  if (__amsIsExcludedEl(el)) {
    if (s.excluded >= limit) return false;
    s.excluded++;
  } else {
    if (s.real >= limit) return false;
    s.real++;
  }
  list.push(item);
  return true;
}
function __amsReal(list) {
  return __amsCapCounts(list).real;
}
function __amsCssSelector(el) {
  if (!el || !el.tagName) return '';
  var esc = function(s) {
    return (window.CSS && CSS.escape) ? CSS.escape(s) : String(s).replace(/[^a-zA-Z0-9_-]/g, '\\$&');
  };
  var seg = function(node) {
    var tag = node.tagName.toLowerCase();
    if (node.id) return tag + '#' + esc(node.id);
    var cls = Array.prototype.slice.call(node.classList || [])
      .filter(function(c) { return c.length > 0 && c.length < 30; })
      .slice(0, 2)
      .map(function(c) { return '.' + esc(c); })
      .join('');
    var nth = '';
    var p = node.parentElement;
    if (p) {
      var sameTag = Array.prototype.slice.call(p.children)
        .filter(function(c) { return c.tagName === node.tagName; });
      if (sameTag.length > 1) {
        nth = ':nth-of-type(' + (sameTag.indexOf(node) + 1) + ')';
      }
    }
    return tag + cls + nth;
  };
  var parts = [];
  var cur = el;
  for (var i = 0; i < 5 && cur && cur.tagName && cur !== document.documentElement; i++) {
    parts.unshift(seg(cur));
    if (cur.id) break;
    cur = cur.parentElement;
  }
  return parts.join(' > ');
}
"#;

/// `__amsIsAriaHidden(el)` — returns true when an element is part of an
/// `aria-hidden="true"` subtree or carries `role="presentation"/"none"` itself.
///
/// WCAG 1.4.3 explicitly exempts decorative elements from contrast requirements.
/// Elements inside an `aria-hidden="true"` subtree are not exposed to assistive
/// technology and serve a purely decorative role — they must be skipped.
/// `role="presentation"` and `role="none"` on the element itself also signal
/// decoration with no accessibility semantics.
pub(crate) const IS_ARIA_HIDDEN_JS: &str = r#"
function __amsIsAriaHidden(el) {
  var cur = el;
  while (cur && cur !== document.documentElement) {
    if (cur.getAttribute('aria-hidden') === 'true') return true;
    if (cur.parentElement) { cur = cur.parentElement; continue; }
    // Across a shadow boundary: aria-hidden on the host hides the component.
    var root = cur.getRootNode ? cur.getRootNode() : null;
    cur = (root && root.host) ? root.host : null;
  }
  var role = el.getAttribute('role');
  return role === 'presentation' || role === 'none';
}
"#;

/// `__amsIsVisuallyHidden(el)` — detects the visually-hidden / `.sr-only`
/// pattern (text exposed only to assistive technology).
///
/// Recognises unrendered elements, clip-rect `rect(0 0 0 0)`, ≤1px boxes that
/// clip their overflow (`hidden`/`clip`),
/// `clip-path: inset(50%/100%)`, and far off-screen positioned/indented text.
/// WCAG contrast (1.4.3) does not apply to such elements — axe-core skips
/// them too.
pub(crate) const IS_VISUALLY_HIDDEN_JS: &str = r#"
function __amsIsVisuallyHidden(el) {
  // Not rendered at all (display:none on the element or an ancestor, closed
  // <details>, content-visibility:hidden).
  if (el.checkVisibility && !el.checkVisibility()) return true;
  // An element without content and without a box (a 0×0 <img>) shows
  // nothing; one with content can still overflow visibly, see below.
  try {
    var own = el.getBoundingClientRect();
    if ((own.width === 0 || own.height === 0) && !el.hasChildNodes()) return true;
  } catch (e) {}
  var cur = el;
  for (var depth = 0; cur && cur.nodeType === 1 && depth < 12; depth++) {
    var s = window.getComputedStyle(cur);
    var clip = s.clip;
    if (clip && clip !== 'auto') {
      var m = clip.match(/rect\(([^)]+)\)/);
      if (m) {
        var nums = m[1].split(/[\s,]+/).map(parseFloat);
        if (nums.length === 4 && nums.every(function(n) { return Math.abs(n) <= 1; })) return true;
      }
    }
    var cp = s.clipPath || s.webkitClipPath;
    if (cp && /inset\(\s*(100%|9[0-9](\.\d+)?%|50(\.0*)?%)/.test(cp)) return true;
    var w = parseFloat(s.width);
    var h = parseFloat(s.height);
    // Only a box that clips its overflow hides content by having no size.
    // A container of floats is 0px high and still shows every child
    // (slovensko.sk, #716), and `display: contents` has no box at all.
    var clips = function(v) { return v === 'hidden' || v === 'clip'; };
    var hiddenOverflow = clips(s.overflow) || clips(s.overflowX) || clips(s.overflowY);
    if (hiddenOverflow && ((!isNaN(w) && w <= 1) || (!isNaN(h) && h <= 1))) return true;
    if (hiddenOverflow) {
      try {
        var rect = cur.getBoundingClientRect();
        if (rect.width <= 1 || rect.height <= 1) return true;
      } catch (e) {}
    }

    if (s.position === 'absolute' || s.position === 'fixed') {
      var left = parseFloat(s.left);
      var top = parseFloat(s.top);
      if ((!isNaN(left) && (left <= -999 || left >= 9999)) || (!isNaN(top) && (top <= -999 || top >= 9999))) return true;
    }
    var ti = parseFloat(s.textIndent);
    if (!isNaN(ti) && (ti <= -999 || ti >= 999)) return true;

    var className = cur.className;
    var classStr = '';
    if (typeof className === 'string') {
      classStr = className;
    } else if (className && typeof className.baseVal === 'string') {
      classStr = className.baseVal;
    }
    if (classStr && (
      classStr.indexOf('sr-only') !== -1 ||
      classStr.indexOf('visually-hidden') !== -1 ||
      classStr.indexOf('text-hide') !== -1 ||
      classStr.indexOf('hide-text') !== -1 ||
      classStr.indexOf('hidden') !== -1
    )) return true;

    cur = cur.parentElement;
  }
  return false;
}
"#;
