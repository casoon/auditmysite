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
pub const CSS_SELECTOR_JS: &str = r#"
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

/// `__amsIsVisuallyHidden(el)` — detects the visually-hidden / `.sr-only`
/// pattern (text exposed only to assistive technology).
///
/// Recognises clip-rect `rect(0 0 0 0)`, ≤1px boxes with `overflow:hidden`,
/// `clip-path: inset(50%/100%)`, and far off-screen positioned/indented text.
/// WCAG contrast (1.4.3) does not apply to such elements — axe-core skips
/// them too.
pub const IS_VISUALLY_HIDDEN_JS: &str = r#"
function __amsIsVisuallyHidden(el) {
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
    var hiddenOverflow = s.overflow === 'hidden' || s.overflowX === 'hidden' || s.overflowY === 'hidden';
    if (hiddenOverflow && ((!isNaN(w) && w <= 1) || (!isNaN(h) && h <= 1))) return true;
    if (w === 0 || h === 0) return true;

    try {
      var rect = cur.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return true;
    } catch (e) {}

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
