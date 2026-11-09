
(() => {
  const out = [];
  const anims = typeof document.getAnimations === 'function' ? document.getAnimations() : [];
  for (const a of anims) {
    if (out.length >= 200) break;
    if (a.playState !== 'running' || !a.effect || !a.effect.target) continue;
    if (!isFinite(a.effect.getComputedTiming().endTime)) continue;
    const t = a.effect.target;
    if (t.nodeType === 1 && !out.includes(t)) out.push(t);
  }
  return out;
})()
