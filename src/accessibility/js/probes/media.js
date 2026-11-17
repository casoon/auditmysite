// media/audio-autoplay-measured: Wiedergabezustand jedes `audio[autoplay]`
// und `video[autoplay]` des Dokuments, seiner offenen Shadow Roots und
// gleich-ursprünglicher Rahmen. Ohne solches Element sofort fertig.
//
// Wartet auf die Metadaten (alle zusammen höchstens BUDGET_MS), lässt dem
// Autoplay dann SETTLE_MS und liest je Medium mit Metadaten:
// - p: `paused`; ein zu Ende gespieltes Medium (`ended`) hat gespielt.
// - m: `muted`.
// - d: wie lange es von selbst spielt, in Sekunden; `null`, wo `duration`
//   unbekannt ist (NaN, keine Metadaten). Ein Medienfragment `#t=a,b` kürzt
//   auf seine Spanne. i: endet nie (Datenstrom, `loop`).
// - a: hörbarer Ton. Chrome kennt `audioTracks` nur hinter einem Flag; die
//   Spuren von `captureStream()` sagen nach den Metadaten, ob es eine Tonspur
//   gibt. Keine Spur: `false`. Eine Spur kann stumm sein (Videos mit leerer
//   Tonspur sind häufig): Erst ein Pegel über LOUD in den ersten LISTEN_MS
//   eines spielenden, nicht stummen Mediums macht `true`; sonst unbekannt
//   (`null`), ebenso ein Medium fremden Ursprungs, an dem `captureStream`
//   wirft.
// Liest nur; die Spuren des Mitschnitts und der Audio-Kontext werden wieder
// beendet.
const MAX = 50;
const BUDGET_MS = 3000;
const SETTLE_MS = 500;
const LISTEN_MS = 1000;
const LOUD = 0.001; // -60 dBFS
const els = [];
let skipped = 0;
walk(document, (el) => {
  if ((el.localName !== 'audio' && el.localName !== 'video') || !el.hasAttribute('autoplay')) return;
  if (els.length < MAX) els.push(el); else skipped++;
}, true);
if (!els.length) return { els, json: JSON.stringify({ rows: [] }) };

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const metadata = (el) => el.readyState >= 1 ? Promise.resolve() : new Promise((resolve) => {
  el.addEventListener('loadedmetadata', resolve, { once: true });
});
await Promise.race([Promise.all(els.map(metadata)), sleep(BUDGET_MS)]);
await sleep(SETTLE_MS);

const span = (el) => {
  const d = el.duration;
  if (Number.isNaN(d)) return { d: null, i: false };
  if (d === Infinity || el.loop) return { d: null, i: true };
  const m = /#(?:.*&)?t=(?:npt:)?(\d*(?:\.\d+)?)(?:,(\d+(?:\.\d+)?))?(?:&|$)/.exec(el.currentSrc || '');
  if (!m) return { d, i: false };
  const start = parseFloat(m[1]) || 0;
  const end = m[2] !== undefined ? Math.min(parseFloat(m[2]), d) : d;
  return { d: Math.max(0, end - start), i: false };
};

// Tonspuren über einen Mitschnitt; `null`, wo er nicht geht.
const capture = (el) => {
  if (el.readyState < 1 || typeof el.captureStream !== 'function') return null;
  try { return el.captureStream(); } catch (e) { return null; /* fremder Ursprung */ }
};

// Hört in die Mitschnitte, bis jeder einmal hörbar war oder LISTEN_MS um sind.
const listen = async (subjects) => {
  const Ctx = window.AudioContext || window.webkitAudioContext;
  if (!Ctx || !subjects.length) return;
  let ctx;
  try { ctx = new Ctx(); } catch (e) { return; }
  try {
    if (ctx.state === 'suspended') await Promise.race([ctx.resume(), sleep(200)]);
    const taps = subjects.map((s) => {
      const analyser = ctx.createAnalyser();
      analyser.fftSize = 2048;
      ctx.createMediaStreamSource(s.stream).connect(analyser);
      return { s, analyser, buf: new Float32Array(analyser.fftSize) };
    });
    const until = Date.now() + LISTEN_MS;
    while (Date.now() < until && taps.some((t) => !t.s.audible)) {
      await sleep(50);
      for (const t of taps) {
        if (t.s.audible) continue;
        t.analyser.getFloatTimeDomainData(t.buf);
        if (t.buf.some((v) => Math.abs(v) > LOUD)) t.s.audible = true;
      }
    }
  } catch (e) { /* bleibt unbekannt */ } finally {
    ctx.close();
  }
};

const subjects = els.map((el) => {
  const stream = capture(el);
  return { el, stream, tracks: stream ? stream.getAudioTracks().length : null, audible: false };
});
await listen(subjects.filter((s) => s.tracks > 0 && !s.el.paused && !s.el.muted));
for (const s of subjects) if (s.stream) s.stream.getTracks().forEach((t) => t.stop());

// Ohne Zeile (nicht gemessen, das Markup-Urteil bleibt): keine Metadaten im
// Budget (Quelle kaputt oder zu langsam) und ein Autoplay, das noch auf genug
// Daten wartet (`paused`, noch nie gespielt, `readyState` unter
// HAVE_ENOUGH_DATA) -- beides hieße sonst „hält an".
const pending = (el) => el.paused && !el.ended && el.played.length === 0 && el.readyState < 4;
const rows = [];
subjects.forEach((s, e) => {
  if (s.el.readyState < 1 || pending(s.el)) return;
  rows.push(Object.assign(
    {
      e,
      p: s.el.paused && !s.el.ended,
      m: s.el.muted,
      a: s.tracks === 0 ? false : (s.audible ? true : null),
    },
    span(s.el),
  ));
});
return { els, json: JSON.stringify({ rows, skipped }) };
