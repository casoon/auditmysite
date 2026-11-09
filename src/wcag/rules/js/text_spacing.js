
(function() {
  var elements = document.querySelectorAll('*');
  var count = Math.min(elements.length, 2000);

  // A `backface-visibility: hidden` element (checked up to 5 ancestors) is
  // almost always one face of a flip/reveal card animation — present in the
  // DOM with normal dimensions but rotated away from the viewer in its
  // resting state, not something a user actually sees or reads by default.
  function hasHiddenBackface(el) {
    var node = el;
    for (var depth = 0; depth < 5 && node; depth++) {
      if (getComputedStyle(node).backfaceVisibility === 'hidden') return true;
      node = node.parentElement;
    }
    return false;
  }

  function clippedFlags() {
    var flags = new Array(count);
    for (var i = 0; i < count; i++) {
      var el = elements[i];
      // Skip elements whose own visible box is tiny (<=2px) regardless of
      // spacing — the standard "sr-only"/visually-hidden technique clips a
      // 1x1px box permanently and isn't a real pointer/visual target, same
      // class of false positive as the target-size check (#QA-039).
      if (el.clientWidth <= 2 || el.clientHeight <= 2 || hasHiddenBackface(el)) {
        flags[i] = false;
        continue;
      }
      var cs = getComputedStyle(el);
      var clipsX = cs.overflowX === 'hidden' && el.scrollWidth > el.clientWidth + 2;
      var clipsY = cs.overflowY === 'hidden' && el.scrollHeight > el.clientHeight + 2;
      flags[i] = (clipsX || clipsY) && el.textContent.trim().length > 0;
    }
    return flags;
  }

  // Baseline: elements already clipped regardless of spacing (e.g. the
  // standard "sr-only"/visually-hidden technique, permanently 1x1 with
  // overflow:hidden) must not be reported — only content that becomes
  // *newly* clipped once the spacing increases is a real violation.
  var before = clippedFlags();

  var style = document.createElement('style');
  style.textContent =
    '* { line-height: 1.5 !important; letter-spacing: 0.12em !important; word-spacing: 0.16em !important; }' +
    'p { margin-bottom: 2em !important; }';
  document.head.appendChild(style);

  var after = clippedFlags();
  style.remove();

  for (var i = 0; i < count; i++) {
    if (after[i] && !before[i]) {
      var el = elements[i];
      var tag = el.tagName.toLowerCase();
      var id = el.id ? '#' + el.id : '';
      var cls = (el.className && typeof el.className === 'string')
        ? '.' + el.className.trim().split(/\s+/)[0] : '';
      return tag + id + cls;
    }
  }
  return null;
})()
