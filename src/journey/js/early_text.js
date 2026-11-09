
  const fold = window.innerHeight;
  if (!fold) return 0;
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  let total = 0;
  let node;
  while ((node = walker.nextNode())) {
    const text = node.nodeValue ? node.nodeValue.trim() : '';
    if (!text) continue;
    const el = node.parentElement;
    if (!el) continue;
    const styles = window.getComputedStyle(el);
    if (styles.display === 'none' || styles.visibility === 'hidden') continue;
    if (parseFloat(styles.opacity) === 0) continue;
    // Screen-reader-only text is not text the visitor meets — same helper the
    // in-page WCAG checks use.
    if (__amsIsVisuallyHidden(el)) continue;
    let rect;
    try { rect = el.getBoundingClientRect(); } catch (e) { continue; }
    if (rect.width === 0 || rect.height === 0) continue;
    // Document coordinates, not viewport ones: another module may have
    // scrolled the page before this runs, and "the first viewport height"
    // has to mean the top of the document either way.
    const top = rect.top + window.scrollY;
    if (top >= fold) continue;
    total += text.length;
  }
  return total;
