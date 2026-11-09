// auth/paste-blocked: schickt an jedes Passwort- und Codefeld des Dokuments,
// seiner offenen Shadow Roots und gleich-ursprünglicher Rahmen ein
// synthetisches, abbrechbares `paste` und hält fest, ob ein Handler es
// abbricht und ob der Probetext danach im Feld steht. Berührt die Seite:
// Handler laufen, der Wert des Feldes wird für einen Moment geschrieben und
// kommt zurück; alert/confirm/prompt sind währenddessen still.
const PROBE = 'ams-paste-probe';
const MAX = 50;
const NOT_A_PASTE_FIELD = new Set(['hidden', 'checkbox', 'radio', 'file', 'submit', 'button', 'reset', 'image', 'range', 'color']);
const candidate = (el) => {
  if (el.localName !== 'input' || el.hasAttribute('disabled') || el.hasAttribute('readonly')) return false;
  const kind = (el.getAttribute('type') || '').trim().toLowerCase();
  if (NOT_A_PASTE_FIELD.has(kind)) return false;
  const token = (wanted) => (el.getAttribute('autocomplete') || '').split(/\s+/).some((t) => t.toLowerCase() === wanted);
  return kind === 'password' || token('one-time-code') || token('current-password');
};
const pasteEvent = (win) => {
  const init = { bubbles: true, cancelable: true, composed: true };
  try {
    const data = new win.DataTransfer();
    data.setData('text/plain', PROBE);
    return new win.ClipboardEvent('paste', Object.assign({ clipboardData: data }, init));
  } catch (e) {
    const ev = new win.Event('paste', init);
    Object.defineProperty(ev, 'clipboardData', {
      value: { types: ['text/plain'], getData: (t) => (t === 'text' || t === 'text/plain' ? PROBE : '') },
    });
    return ev;
  }
};

const fields = [];
walk(document, (el) => { if (candidate(el)) fields.push(el); }, true);

const wins = new Set(fields.map(viewOf));
const saved = Array.from(wins).map((w) => ({ w, alert: w.alert, confirm: w.confirm, prompt: w.prompt }));
const els = [];
const rows = [];
let skipped = 0;
try {
  for (const w of wins) {
    w.alert = () => {};
    w.confirm = () => false;
    w.prompt = () => null;
  }
  for (const field of fields) {
    if (!rendered(field)) continue;
    if (els.length >= MAX) { skipped++; continue; }
    const win = viewOf(field);
    const value = field.value;
    let start = null, end = null, dir = null;
    try { start = field.selectionStart; end = field.selectionEnd; dir = field.selectionDirection; } catch (e) {}
    try {
      const cancelled = !field.dispatchEvent(pasteEvent(win));
      const inserted = (field.value || '').indexOf(PROBE) !== -1;
      rows.push({ e: els.length, c: cancelled, i: inserted });
      els.push(field);
    } finally {
      // Nur schreiben, wenn sich etwas geändert hat: Schreiben markiert das
      // Feld als vom Nutzer berührt.
      if (field.value !== value) field.value = value;
      if (start !== null && end !== null) {
        try { field.setSelectionRange(start, end, dir || undefined); } catch (e) {}
      }
    }
  }
} finally {
  for (const s of saved) { s.w.alert = s.alert; s.w.confirm = s.confirm; s.w.prompt = s.prompt; }
}
return { els, json: JSON.stringify({ rows, skipped }) };
