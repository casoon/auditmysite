
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
