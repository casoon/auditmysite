// focus/obscured, focus/obscured-enhanced, focus/indicator-missing:
// fokussiert jedes per Tab erreichbare Bedienelement des Dokuments und seiner
// offenen Shadow Roots, scrollt es mit `scrollIntoView({ block: 'nearest' })`
// ins Bild, tastet ein Raster von 3 mal 3 Punkten (1 px eingerückt, nur
// Punkte im Viewport) mit `elementsFromPoint` ab und zählt die Punkte unter
// einem fremden fixierten oder klebenden Element. Dazu: ob sich beim
// Fokussieren ein Stil ändert, der als Indikator taugt.
//
// Berührt die Seite: Fokus und Scrollposition (auch die scrollbarer
// Vorfahren) ändern sich je Element und kommen zurück; focus- und
// blur-Handler der Seite laufen. Übergänge sind währenddessen gesperrt, damit
// ein `transition` des Fokusstils den Messwert nicht auf dem Ausgangswert
// hält.
const MAX = 300;
const GRID = [0, 0.5, 1];
const SCROLL = 'scrollIntoView({ block: "nearest", inline: "nearest" })';

const tracked = ['outline-style', 'outline-width', 'outline-color', 'outline-offset', 'box-shadow',
  'border-top-color', 'border-top-width', 'border-right-color', 'border-right-width',
  'border-bottom-color', 'border-bottom-width', 'border-left-color', 'border-left-width',
  'background-color', 'background-image', 'color', 'text-decoration-line', 'text-decoration-color',
  'text-decoration-thickness', 'filter'];
const styleOf = (el) => {
  const s = viewOf(el).getComputedStyle(el);
  return tracked.map((p) => s.getPropertyValue(p));
};
// Ob die Änderung als Indikator taugt: ein neuer oder anderer Umriss, oder
// ein anderer Schatten, Rahmen, Hintergrund, Text- oder Linienstil.
const indicator = (a, b) => {
  const outlineShown = b[0] !== 'none' && parseFloat(b[1]) > 0;
  if (outlineShown && (a[0] !== b[0] || a[1] !== b[1] || a[2] !== b[2] || a[3] !== b[3])) return true;
  for (let i = 4; i < a.length; i++) if (a[i] !== b[i]) return true;
  return false;
};
const point = (start, length, share) => {
  if (length <= 2 || share === 0.5) return start + length / 2;
  return share === 0 ? start + 1 : start + length - 1;
};
const fixedAncestor = (hit, el) => {
  for (let cur = hit; cur; cur = flatParent(cur)) {
    if (cur.contains(el)) return null;
    const p = viewOf(cur).getComputedStyle(cur).position;
    if (p === 'fixed' || p === 'sticky') return cur;
  }
  return null;
};
const scrollNow = (target, left, top) => {
  try { target.scrollTo({ left, top, behavior: 'instant' }); } catch (e) { target.scrollTo(left, top); }
};
const scrollState = (el) => {
  const state = [{ t: window, left: window.scrollX, top: window.scrollY }];
  for (let cur = flatParent(el); cur; cur = flatParent(cur)) {
    if (cur === document.documentElement) continue;
    state.push({ t: cur, left: cur.scrollLeft, top: cur.scrollTop });
  }
  return state;
};
const scrollBack = (state) => {
  for (const s of state) {
    const now = s.t === window ? [window.scrollX, window.scrollY] : [s.t.scrollLeft, s.t.scrollTop];
    if (now[0] !== s.left || now[1] !== s.top) scrollNow(s.t, s.left, s.top);
  }
};

const targets = [];
const roots = new Set([document]);
walk(document, (el) => {
  if (el.getRootNode() !== document) roots.add(el.getRootNode());
  // Rahmen sind keine Bedienelemente mit eigenem Fokusstil: Der Fokus geht in
  // ihr Dokument, dessen Bedienelemente der Rahmen für sich prüft.
  if (el.localName === 'iframe' || el.localName === 'frame') return;
  if (perTab(el) && typeof el.focus === 'function' && rendered(el)) targets.push(el);
}, false);

const before = deepActive(document);
const noTransition = adoptSheet('*{transition:none!important}', roots);
const els = [];
const rows = [];
let skipped = 0;
try {
  for (const el of targets) {
    if (rows.length >= MAX) { skipped++; continue; }
    const state = scrollState(el);
    try {
      // Der Ausgangsstil ist der eines Elements ohne Fokus: Hat das Element den
      // Fokus schon (der Dialog gab ihn beim Öffnen), nimmt ihn ein Weichen weg.
      const held = deepActive(el.ownerDocument);
      if (held && held !== el.ownerDocument.body && typeof held.blur === 'function') held.blur();
      const styleBefore = styleOf(el);
      el.focus({ preventScroll: true });
      if (deepActive(el.ownerDocument) !== el) {
        // Der Fokus landet nicht (inert hinter einem Modal, vom Skript wieder
        // weggenommen): Es gibt nichts Fokussiertes, das etwas überdecken
        // könnte. Null Abtastpunkte, kein Urteil über den Indikator -- statt
        // keiner Zeile, bei der die Regel auf die statische Geometrie
        // zurückfiele und ein Bedienelement hinter einem Modal meldete.
        rows.push({ e: els.length, p: 0, c: 0, v: null, blocker: null });
        els.push(el);
        continue;
      }
      const styleAfter = styleOf(el);
      try { el.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'instant' }); }
      catch (e) { el.scrollIntoView({ block: 'nearest', inline: 'nearest' }); }

      const box = el.getBoundingClientRect();
      const vw = el.ownerDocument.documentElement.clientWidth;
      const vh = el.ownerDocument.documentElement.clientHeight;
      const root = el.getRootNode();
      let points = 0, covered = 0, blocker = null;
      for (const fy of GRID) {
        for (const fx of GRID) {
          const x = point(box.left, box.width, fx);
          const y = point(box.top, box.height, fy);
          if (x < 0 || y < 0 || x >= vw || y >= vh) continue;
          points++;
          for (const hit of root.elementsFromPoint(x, y)) {
            if (hit === el || el.contains(hit)) break;
            const fixed = fixedAncestor(hit, el);
            if (!fixed) continue;
            covered++;
            if (blocker === null) blocker = fixed;
            break;
          }
        }
      }
      rows.push({ e: els.length, p: points, c: covered, v: indicator(styleBefore, styleAfter), blocker });
      els.push(el);
    } finally {
      scrollBack(state);
    }
  }
} finally {
  if (noTransition) noTransition.undo();
  if (before && before !== document.body && typeof before.focus === 'function') before.focus({ preventScroll: true });
  else { const a = deepActive(document); if (a && typeof a.blur === 'function') a.blur(); }
}
// Blockierende Elemente hängen hinten an `els`; die Zeile nennt deren Index.
const blockerIndex = new Map();
for (const row of rows) {
  if (row.blocker) {
    if (!blockerIndex.has(row.blocker)) { blockerIndex.set(row.blocker, els.length); els.push(row.blocker); }
    row.b = blockerIndex.get(row.blocker);
  } else {
    row.b = -1;
  }
  delete row.blocker;
}
return { els, json: JSON.stringify({ rows, skipped, scroll: SCROLL }) };
