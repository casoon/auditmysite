// text-spacing/clipped: misst je Textcontainer mit `overflow: hidden`, ob er
// seinen Text abschneidet, vor und nach dem Überschreiben des Textabstands
// (Zeilenhöhe 1,5, Absatzabstand 2, Zeichenabstand 0,12, Wortabstand 0,16 der
// Schriftgröße). Das Überschreiben ist ein adoptiertes Stylesheet (CSSOM, an
// keine CSP `style-src` gebunden) und kommt danach zurück. Kommt es an einem
// Element nicht an -- die Seite hat es mit einem stärkeren `!important`
// übertrumpft --, bleibt das Element ohne Zeile.
const MAX = 5000;
const CSS = '*{line-height:1.5!important;letter-spacing:.12em!important;word-spacing:.16em!important;' +
  'transition-duration:0s!important;transition-delay:0s!important}p{margin-bottom:2em!important}';
const NO_TRANSITION = '*{transition-duration:0s!important;transition-delay:0s!important}';

const ownText = (el) => Array.prototype.some.call(el.childNodes, (c) => c.nodeType === 3 && c.data.trim() !== '');
const clips = (el, x, y) =>
  (x && el.scrollWidth > el.clientWidth + 2) || (y && el.scrollHeight > el.clientHeight + 2);
const backface = (el) => {
  let cur = el;
  for (let n = 0; n <= 5 && cur; n++, cur = flatParent(cur)) {
    if (viewOf(cur).getComputedStyle(cur).backfaceVisibility === 'hidden') return true;
  }
  return false;
};
const landed = (el) => {
  const s = viewOf(el).getComputedStyle(el);
  const size = parseFloat(s.fontSize);
  if (!(size > 0)) return false;
  return Math.abs(parseFloat(s.letterSpacing) - 0.12 * size) < 0.05 && Math.abs(parseFloat(s.lineHeight) - 1.5 * size) < 0.5;
};

const roots = new Set([document]);
const candidates = [];
let skipped = 0;
walk(document, (el) => {
  if (el.getRootNode() !== document) roots.add(el.getRootNode());
  if (!ownText(el) || !rendered(el)) return;
  if (candidates.length >= MAX) { skipped++; return; }
  candidates.push(el);
}, false);

const measure = [];
let plain = null;
for (const el of candidates) {
  const s = getComputedStyle(el);
  const x = s.overflowX === 'hidden';
  const y = s.overflowY === 'hidden';
  if (!x && !y) { if (plain === null) plain = el; continue; }
  const box = el.getBoundingClientRect();
  if (box.width <= 2 || box.height <= 2 || backface(el)) continue;
  measure.push({ el, x, y, before: clips(el, x, y), after: null });
}

let failed = null;
if (measure.length > 0) {
  const applied = adoptSheet(CSS, roots);
  if (!applied) {
    failed = 'unsupported';
  } else {
    try {
      for (const m of measure) if (landed(m.el)) m.after = clips(m.el, m.x, m.y);
    } finally {
      // Erst die Übergänge sperren, dann zurücknehmen, und die Sperre in einem
      // eigenen Stilabgleich wirken lassen, ehe sie fällt.
      try {
        applied.sheet.replaceSync(NO_TRANSITION);
        void document.documentElement.getBoundingClientRect();
      } finally {
        applied.undo();
      }
    }
  }
}

const els = [];
const rows = [];
if (!failed) {
  for (const m of measure) {
    if (m.after === null) continue;
    rows.push({ e: els.length, b: m.before, a: m.after });
    els.push(m.el);
  }
  // Ohne ein Element, das abschneiden kann, gibt es nichts zu messen: Ein
  // Element ohne `overflow: hidden` schneidet auch mit mehr Textabstand nicht
  // ab. Eine Zeile belegt, dass der Durchgang gelaufen ist.
  if (rows.length === 0 && measure.length === 0 && plain !== null) {
    rows.push({ e: 0, b: false, a: false });
    els.push(plain);
  }
  if (rows.length === 0 && measure.length > 0) failed = 'override-not-applied';
}
return { els, json: JSON.stringify({ rows, skipped, failed }) };
