
(function() {
  /*CSS_SELECTOR*/

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
