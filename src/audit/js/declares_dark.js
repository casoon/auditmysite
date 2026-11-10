
(() => {
  const darkQuery = t => /prefers-color-scheme\s*:\s*dark/.test(t || '');
  const walk = rules => {
    for (const r of rules) {
      if (r instanceof CSSMediaRule && darkQuery(r.conditionText || (r.media && r.media.mediaText))) return true;
      if (r.cssRules && walk(r.cssRules)) return true;
    }
    return false;
  };
  for (const sheet of document.styleSheets) {
    if (darkQuery(sheet.media && sheet.media.mediaText)) return true;
    let rules;
    try { rules = sheet.cssRules; } catch (_) { continue; }
    if (rules && walk(rules)) return true;
  }
  const names = v => /(^|\s)dark(\s|$)/.test(v || '');
  const meta = document.querySelector('meta[name="color-scheme"]');
  if (meta && names(meta.getAttribute('content'))) return true;
  return names(getComputedStyle(document.documentElement).colorScheme);
})()
