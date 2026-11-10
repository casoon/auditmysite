
(function() {
  const vw = window.innerWidth;
  // Check if the document itself overflows horizontally
  if (document.documentElement.scrollWidth <= vw + 5) return null;
  const TWO_D = 'table:not([role="presentation"]):not([role="none"]), img, picture, svg, ' +
    'canvas, video, iframe, object, embed, [role="img"], [role="table"], [role="grid"], ' +
    '[role="treegrid"], [role="application"]';
  // Find the widest element to give a useful selector hint
  let worst = null;
  let worstRight = vw + 5;
  let exemptOverflow = false;
  for (const el of document.querySelectorAll('*')) {
    const r = el.getBoundingClientRect();
    if (r.right <= vw + 5) continue;
    if (el.closest(TWO_D)) {
      exemptOverflow = true;
      continue;
    }
    if (r.right > worstRight) {
      worstRight = r.right;
      const tag = el.tagName.toLowerCase();
      const id = el.id ? '#' + el.id : '';
      const cls = el.className && typeof el.className === 'string'
        ? '.' + el.className.trim().split(/\s+/)[0] : '';
      worst = tag + id + cls;
    }
  }
  // No element box overflows, yet the document does: overflowing text such
  // as an unbroken string. Still a failure, without a better hint.
  return worst || (exemptOverflow ? null : 'unknown element');
})()
