// frames/cross-origin: je <iframe>/<frame> des Dokuments und seiner offenen
// Shadow Roots, ob das Dokument darin lesbar ist. Liest nur.
const els = [];
const rows = [];
walk(document, (el) => {
  if (el.tagName !== 'IFRAME' && el.tagName !== 'FRAME') return;
  let inner = null;
  try { inner = el.contentDocument; } catch (e) { inner = null; }
  rows.push({ e: els.length, r: inner !== null });
  els.push(el);
}, false);
return { els, json: JSON.stringify({ rows }) };
