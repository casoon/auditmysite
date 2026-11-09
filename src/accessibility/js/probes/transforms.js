// orientation/root-rotated: `transform` und `rotate` von html und body, wie
// der Browser sie serialisiert. Liest nur.
const els = [];
const rows = [];
for (const el of [document.documentElement, document.body]) {
  if (!el) continue;
  const s = getComputedStyle(el);
  rows.push({ e: els.length, t: s.transform, r: s.getPropertyValue('rotate') });
  els.push(el);
}
return { els, json: JSON.stringify({ rows }) };
