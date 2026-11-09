// Gemeinsamer Vorspann der Rendering-Probes (probes.rs): Hilfen, die jede
// Messung braucht. Wird vor den Rumpf der Messung gesetzt; die Messung gibt
// `{ els, json }` zurück -- die Elemente als Array (Rust löst sie über
// `DOM.describeNode` in Backend-IDs auf) und die Zeilen als JSON-Text, die
// Elemente über ihren Index in `els` benennend.
const flatParent = (el) =>
  el.assignedSlot || el.parentElement || ((el.getRootNode && el.getRootNode().host) || null);

// Elemente des Dokuments, offener Shadow Roots und -- auf Wunsch --
// gleich-ursprünglicher Rahmen.
const walk = (root, visit, frames) => {
  const all = root.querySelectorAll('*');
  for (let i = 0; i < all.length; i++) {
    const el = all[i];
    visit(el);
    if (el.shadowRoot) walk(el.shadowRoot, visit, frames);
    if (frames && (el.tagName === 'IFRAME' || el.tagName === 'FRAME')) {
      let inner = null;
      try { inner = el.contentDocument; } catch (e) { inner = null; }
      if (inner && inner.documentElement) walk(inner, visit, frames);
    }
  }
};

const viewOf = (el) => el.ownerDocument.defaultView || window;

// Dargestellt: Kasten mit Fläche, weder `display: none` noch `visibility: hidden`.
const rendered = (el) => {
  const r = el.getBoundingClientRect();
  if (r.width === 0 || r.height === 0) return false;
  const s = viewOf(el).getComputedStyle(el);
  return s.display !== 'none' && s.visibility !== 'hidden' && s.visibility !== 'collapse';
};

// Tabtaste erreicht das Element: dieselbe Zusammensetzung wie
// `per_tab_reachable` in a11y-rules.
const perTab = (el) => {
  const raw = el.getAttribute('tabindex');
  if (raw !== null && /^\s*[+-]?\d+\s*$/.test(raw)) return parseInt(raw, 10) >= 0;
  const tag = el.localName;
  let native = false;
  if (tag === 'a' || tag === 'area') native = el.hasAttribute('href');
  else if (tag === 'input') native = (el.getAttribute('type') || '').trim().toLowerCase() !== 'hidden';
  else native = tag === 'button' || tag === 'select' || tag === 'textarea' || tag === 'summary' || tag === 'iframe';
  if (!native) return false;
  return !(el.hasAttribute('disabled') && (tag === 'button' || tag === 'input' || tag === 'select' || tag === 'textarea'));
};

const deepActive = (doc) => {
  let a = doc.activeElement;
  while (a && a.shadowRoot && a.shadowRoot.activeElement) a = a.shadowRoot.activeElement;
  return a;
};

// Eine Überschreibung für Übergänge: ein adoptiertes Stylesheet je Baum. Es
// entsteht über das CSSOM, ist an keine CSP `style-src` gebunden und ändert
// weder Knoten noch Attribut. `null`, wo der Browser konstruierbare
// Stylesheets nicht kennt.
const adoptSheet = (css, roots) => {
  const Sheet = window.CSSStyleSheet;
  if (typeof Sheet !== 'function' || !('adoptedStyleSheets' in document)) return null;
  let sheet;
  try { sheet = new Sheet(); sheet.replaceSync(css); } catch (e) { return null; }
  const before = new Map();
  for (const r of roots) {
    before.set(r, Array.from(r.adoptedStyleSheets));
    r.adoptedStyleSheets = before.get(r).concat([sheet]);
  }
  return {
    sheet,
    undo: () => { for (const [r, old] of before) r.adoptedStyleSheets = old; },
  };
};
