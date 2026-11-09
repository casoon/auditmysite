
(function() {
  /*CSS_SELECTOR*/

  // An overlay that paints below the control (lower stacking order) hides
  // nothing of it. Decided in the browser's own paint order at the centre of
  // the overlap; outside the viewport that cannot be asked, and the overlay
  // stays a candidate.
  function paintsBelow(ov, o, fel, frect) {
    var left = Math.max(frect.x, o.x), right = Math.min(frect.x + frect.width, o.x + o.w);
    var top = Math.max(frect.y, o.y), bottom = Math.min(frect.y + frect.height, o.y + o.h);
    if (right <= left || bottom <= top) return false;
    var cx = (left + right) / 2, cy = (top + bottom) / 2;
    if (cx < 0 || cy < 0 || cx >= innerWidth || cy >= innerHeight) return false;
    var painted = document.elementsFromPoint(cx, cy);
    var control = -1, overlay = -1;
    for (var p = 0; p < painted.length; p++) {
      if (control < 0 && (painted[p] === fel || fel.contains(painted[p]))) control = p;
      if (overlay < 0 && (painted[p] === ov || ov.contains(painted[p]))) overlay = p;
    }
    return control >= 0 && (overlay < 0 || control < overlay);
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
    overlays.push({ selector: __amsCssSelector(el), x: rect.x, y: rect.y, w: rect.width, h: rect.height });
    overlayEls.push(el);
  }
  if (overlays.length === 0) return { overlays: [], focusables: [] };

  var sel = 'a[href], button, input:not([type="hidden"]), select, textarea, ' +
            '[tabindex], [contenteditable=""], [contenteditable="true"]';
  var els = document.querySelectorAll(sel);
  var focusables = [];
  for (var j = 0; j < els.length && __amsReal(focusables) < 200; j++) {
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
      if (paintsBelow(overlayEls[k], overlays[k], fel, frect)) continue;
      candidates.push(k);
    }
    if (candidates.length === 0) continue;

    __amsPush(focusables, fel, {
      selector: __amsCssSelector(fel),
      x: frect.x, y: frect.y, w: frect.width, h: frect.height,
      overlayIndices: candidates
    }, 200);
  }

  return { overlays: overlays, focusables: focusables };
})()
