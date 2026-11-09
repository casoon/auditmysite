
(async () => {
  const tasks = __TASKS__;
  const img = new Image();
  // Laedt das Foto nie (weder `load` noch `error`), bricht das Skript
  // selbst ab, statt die Auswertung haengen zu lassen.
  await Promise.race([
    new Promise((resolve, reject) => {
      img.onload = resolve;
      img.onerror = () => reject(new Error('screenshot not loadable'));
      img.src = 'data:image/png;base64,__DATA__';
    }),
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error('screenshot load timed out')), __LOAD_MS__)),
  ]);
  const canvas = document.createElement('canvas');
  canvas.width = img.width;
  canvas.height = img.height;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  ctx.drawImage(img, 0, 0);
  // Die Aufgaben stehen relativ zum Zuschnitt; das Foto hat dessen Breite in
  // CSS-Pixeln, bis auf Rundung.
  const scale = img.width / __CLIP_W__;
  const lin = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); };
  const out = {};
  for (const t of tasks) {
    const x = Math.round(t.x * scale), y = Math.round(t.y * scale);
    const w = Math.round(t.w * scale), h = Math.round(t.h * scale);
    if (w <= 0 || h <= 0 || x < 0 || y < 0 || x + w > img.width || y + h > img.height) continue;
    const px = ctx.getImageData(x, y, w, h).data;
    const step = Math.max(1, Math.ceil(Math.sqrt(w * h / __PER__)));
    const lum = [];
    for (let j = 0; j < h; j += step) {
      for (let i = 0; i < w; i += step) {
        const k = (j * w + i) * 4;
        const a = px[k + 3] / 255;
        const c = (v) => lin(v * a + 255 * (1 - a));
        lum.push(Math.round((0.2126 * c(px[k]) + 0.7152 * c(px[k + 1]) + 0.0722 * c(px[k + 2])) * 1e4) / 1e4);
      }
    }
    out[t.id] = lum;
  }
  return out;
})()
