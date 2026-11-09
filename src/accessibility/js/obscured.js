
(() => {
  const vw = window.innerWidth, vh = window.innerHeight;
  const parentOf = (n) => n.assignedSlot || n.parentElement ||
    ((n.getRootNode && n.getRootNode().host) || null);
  const within = (outer, n) => {
    for (; n; n = parentOf(n)) if (n === outer) return true;
    return false;
  };
  const topmostAt = (x, y) => {
    let top = document.elementFromPoint(x, y);
    while (top && top.shadowRoot) {
      const inner = top.shadowRoot.elementFromPoint(x, y);
      if (!inner || inner === top) break;
      top = inner;
    }
    return top;
  };
  const coverOf = (n) => {
    for (; n && n.nodeType === 1; n = parentOf(n)) {
      const p = getComputedStyle(n).position;
      if (p === 'fixed' || p === 'sticky') return n;
    }
    return null;
  };
  const ownText = (el) => Array.prototype.some.call(el.childNodes,
    (c) => c.nodeType === 3 && c.nodeValue.trim() !== '');
  const out = [];
  const roots = [document.documentElement];
  while (roots.length > 0) {
    const walker = document.createTreeWalker(roots.pop(), NodeFilter.SHOW_ELEMENT);
    for (let el = walker.currentNode; el; el = walker.nextNode()) {
      if (el.nodeType !== 1) continue;
      if (el.shadowRoot) roots.push(el.shadowRoot);
      if (!ownText(el)) continue;
      const r = el.getBoundingClientRect();
      if (r.width === 0 || r.height === 0) continue;
      const x = r.left + r.width / 2, y = r.top + r.height / 2;
      if (x < 0 || y < 0 || x >= vw || y >= vh) continue;
      const hit = topmostAt(x, y);
      if (!hit || within(el, hit) || within(hit, el)) continue;
      const cover = coverOf(hit);
      if (cover && !within(cover, el)) out.push(el);
    }
  }
  return out;
})()
